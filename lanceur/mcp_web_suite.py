"""Serveur MCP de WEB·SUITE : Claude Code commande les outils du poste.

    claude mcp add web_suite -- python3 <web_suite>/lanceur/mcp_web_suite.py
    claude mcp add web_suite -- ssh -T <poste> python3 <web_suite>/lanceur/mcp_web_suite.py

MCP sur stdio : JSON-RPC 2.0, un message par ligne sur stdin, une réponse par
ligne sur stdout. Claude Code lance ce script lui-même ; stdout ne doit donc
porter QUE le protocole (journaux sur stderr). Bibliothèque standard seulement.
Spec : lanceur/docs/SPEC-mcp-noyau.md.
"""
import json
import os
import posixpath
import socket
import sys
import urllib.error
import urllib.parse
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))   # Claude Code lance depuis n'importe où
import outils      # noqa: E402  (ports des outils : CATALOGUE)
import web_suite   # noqa: E402  (port du lanceur : PORT_DEFAUT)

VERSION = "0.2.0"
PROTOCOLE_DEFAUT = "2025-06-18"   # celle de WEB_CAO/python/passerelle_mcp.py
PROTOCOLES = ("2025-06-18", "2025-03-26", "2024-11-05")

# Jamais par Claude : publier sur GitHub (/api/github des outils, /api/envoyer et
# /api/identite du lanceur, vers lesquels elles relaient), clé d'IA, appairage.
ROUTES_INTERDITES = ("/api/github", "/api/ia", "/api/envoyer", "/api/identite", "/api/appairer")
DELAI = 30         # s, appel ordinaire
DELAI_SONDE = 1    # s, etat : rester rapide même quand tout est arrêté
# Boucle locale seulement : un http_proxy du poste ne doit pas détourner les appels
OUVREUR = urllib.request.build_opener(urllib.request.ProxyHandler({}))


class ErreurOutil(Exception):
    """Refus ou panne d'un appel à un outil : message lisible, rendu tel quel à Claude."""


class OutilArrete(ErreurOutil):
    pass


class ErreurRpc(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code
        self.message = message


def journal(texte):
    sys.stderr.write(texte + "\n")
    sys.stderr.flush()


def ports():
    """id -> port : le lanceur et les outils du catalogue. Le banc les remplace par
    WEB_SUITE_MCP_PORTS (JSON), comme WEB_TOOLS_JETON_FICHIER pour le jeton."""
    remplaces = os.environ.get("WEB_SUITE_MCP_PORTS")
    if remplaces:
        return {k: int(v) for k, v in json.loads(remplaces).items()}
    p = {"web_suite": web_suite.PORT_DEFAUT}
    p.update((o["id"], o["port"]) for o in outils.CATALOGUE)
    return p


def nom_outil(id_outil):
    return "WEB·SUITE" if id_outil == "web_suite" else outils.PAR_ID.get(id_outil, {}).get("nom", id_outil)


def route_permise(route):
    """Route absolue, canonique, hors des routes interdites. Décodée avant contrôle :
    « /api/%67ithub » ou « /api/./github » ne passent pas."""
    chemin = urllib.parse.unquote(route.split("?", 1)[0].split("#", 1)[0])
    if not chemin.startswith("/") or "//" in chemin or "\\" in chemin:
        return False
    if chemin != "/" and posixpath.normpath(chemin) != chemin.rstrip("/"):
        return False
    bas = chemin.lower().rstrip("/")
    return not any(bas == r or bas.startswith(r + "/") for r in ROUTES_INTERDITES)


def appeler_outil_web(id_outil, route, corps=None, delai=DELAI):
    """Seule porte vers les outils : boucle locale, port du catalogue, routes filtrées.
    GET sans corps, POST JSON avec. Rend le JSON décodé, ou le texte."""
    port = ports().get(id_outil)
    if port is None:
        raise ErreurOutil("Outil inconnu : %s. Outils : %s." % (id_outil, ", ".join(ports())))
    if not route_permise(route):
        raise ErreurOutil("Route refusée au MCP : " + route)
    if id_outil == "web_suite" and corps is not None:
        # Les POST du lanceur installent, lancent, arrêtent ou publient : le MCP ne fait que lire
        raise ErreurOutil("Le lanceur est en lecture seule pour le MCP : " + route)
    nom = nom_outil(id_outil)
    donnees = None if corps is None else json.dumps(corps).encode("utf-8")
    requete = urllib.request.Request("http://127.0.0.1:%d%s" % (port, route), data=donnees,
                                     headers={"Content-Type": "application/json"},
                                     method="GET" if donnees is None else "POST")
    try:
        with OUVREUR.open(requete, timeout=delai) as rep:
            brut = rep.read()
            if "json" in (rep.headers.get("Content-Type") or ""):
                return json.loads(brut.decode("utf-8"))
            return brut.decode("utf-8", "replace")
    except urllib.error.HTTPError as exc:
        raise ErreurOutil("%s a répondu %d : %s" % (nom, exc.code, exc.read()[:300].decode("utf-8", "replace")))
    except urllib.error.URLError as exc:
        if isinstance(exc.reason, (socket.timeout, TimeoutError)):
            raise ErreurOutil("%s ne répond pas en %d s." % (nom, delai))
        raise OutilArrete("%s ne tourne pas : lance-le depuis WEB·SUITE" % nom)
    except (socket.timeout, TimeoutError):
        raise ErreurOutil("%s ne répond pas en %d s." % (nom, delai))
    except ConnectionError:
        raise OutilArrete("%s ne tourne pas : lance-le depuis WEB·SUITE" % nom)


def lance(id_outil):
    try:
        appeler_outil_web(id_outil, "/", delai=DELAI_SONDE)
    except OutilArrete:
        return False
    except ErreurOutil:
        return True   # il a répondu, même mal, ou il écoute sans répondre à temps : il tourne
    return True


def etat_outils():
    return [{"outil": i, "nom": nom_outil(i), "port": p, "adresse": "http://127.0.0.1:%d/" % p, "lance": lance(i)}
            for i, p in ports().items()]


# Outils MCP : nom -> description, schéma d'entrée, fonction. Les modules suivants
# ajoutent leurs entrées ici ; le protocole ne change pas.
OUTILS_MCP = {
    "etat": {
        "description": "Quels outils de WEB·SUITE tournent sur ce poste (lanceur, WEB_CAO, WEB_ANTENNA, "
                       "WEB_3D), sur quel port et à quelle adresse.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda _args: etat_outils(),
    },
}


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
