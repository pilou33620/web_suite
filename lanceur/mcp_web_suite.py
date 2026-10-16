"""Serveur MCP de WEB·SUITE : Claude Code commande les outils du poste.

    claude mcp add web_suite -- python3 <web_suite>/lanceur/mcp_web_suite.py
    claude mcp add web_suite -- ssh -T <poste> python3 <web_suite>/lanceur/mcp_web_suite.py

MCP sur stdio : JSON-RPC 2.0, un message par ligne sur stdin, une réponse par
ligne sur stdout. Claude Code lance ce script lui-même ; stdout ne doit donc
porter QUE le protocole (journaux sur stderr). Bibliothèque standard seulement.
Spec : lanceur/docs/SPEC-mcp-noyau.md.
"""
import json
import sys

VERSION = "0.1.0"
PROTOCOLE_DEFAUT = "2025-06-18"   # celle de WEB_CAO/python/passerelle_mcp.py
PROTOCOLES = ("2025-06-18", "2025-03-26", "2024-11-05")

# Outils MCP : nom -> description, schéma d'entrée, fonction. Les modules suivants
# ajoutent leurs entrées ici ; le protocole ne change pas.
OUTILS_MCP = {}


class ErreurRpc(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code
        self.message = message


def journal(texte):
    sys.stderr.write(texte + "\n")
    sys.stderr.flush()


def initialiser(params):
    demandee = params.get("protocolVersion")
    return {
        "protocolVersion": demandee if demandee in PROTOCOLES else PROTOCOLE_DEFAUT,
        "capabilities": {"tools": {}},
        "serverInfo": {"name": "web_suite", "version": VERSION},
    }


def lister_outils(_params):
    return {"tools": [{"name": nom, "description": o["description"], "inputSchema": o["inputSchema"]}
                      for nom, o in OUTILS_MCP.items()]}


def resultat_texte(texte, erreur=False):
    return {"content": [{"type": "text", "text": texte}], "isError": erreur}


def appeler(params):
    # Une panne d'outil est un résultat en erreur, lisible par Claude, pas une erreur de protocole
    nom = params.get("name")
    outil = OUTILS_MCP.get(nom)
    if outil is None:
        return resultat_texte("Outil inconnu : %s. Outils : %s." % (nom, ", ".join(OUTILS_MCP) or "aucun"), True)
    try:
        rendu = outil["fn"](params.get("arguments") or {})
    except Exception as exc:  # noqa: BLE001 : toute panne revient à Claude, le serveur continue
        journal("Échec de %s : %r" % (nom, exc))
        return resultat_texte("Échec de %s : %s" % (nom, exc), True)
    return resultat_texte(rendu if isinstance(rendu, str) else json.dumps(rendu, ensure_ascii=False, indent=1))


METHODES = {
    "initialize": initialiser,
    "ping": lambda _p: {},
    "tools/list": lister_outils,
    "tools/call": appeler,
}


def traiter(ligne):
    """Une ligne reçue -> la réponse à écrire (dict), ou None pour une notification."""
    try:
        message = json.loads(ligne)
    except ValueError:  # JSON illisible ou octets non UTF-8
        return {"jsonrpc": "2.0", "id": None, "error": {"code": -32700, "message": "JSON illisible"}}
    if not isinstance(message, dict) or not isinstance(message.get("method"), str):
        id_ = message.get("id") if isinstance(message, dict) else None
        return {"jsonrpc": "2.0", "id": id_, "error": {"code": -32600, "message": "Requête invalide"}}
    if "id" not in message:
        return None   # notification (notifications/initialized…) : jamais de réponse
    id_ = message["id"]
    try:
        methode = METHODES.get(message["method"])
        if methode is None:
            raise ErreurRpc(-32601, "Méthode inconnue : " + message["method"])
        params = message.get("params")
        return {"jsonrpc": "2.0", "id": id_, "result": methode(params if isinstance(params, dict) else {})}
    except ErreurRpc as exc:
        return {"jsonrpc": "2.0", "id": id_, "error": {"code": exc.code, "message": exc.message}}


def main():
    # Octets UTF-8 et « \n » seul : sous Windows, le mode texte écrirait \r\n et l'encodage local
    sortie = sys.stdout.buffer
    for brute in sys.stdin.buffer:
        ligne = brute.strip()
        if not ligne:
            continue
        reponse = traiter(ligne)
        if reponse is not None:
            sortie.write(json.dumps(reponse, ensure_ascii=False).encode("utf-8") + b"\n")
            sortie.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
