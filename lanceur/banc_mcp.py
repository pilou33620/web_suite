"""Banc du serveur MCP (mcp_web_suite.py) : protocole JSON-RPC sur stdio.

    python lanceur/banc_mcp.py

Lance le serveur en sous-processus, lui écrit des lignes sur stdin et lit ses
réponses sur stdout, comme le fait Claude Code. Affiche OK si tout passe.
"""
import json
import os
import subprocess
import sys

ICI = os.path.dirname(os.path.abspath(__file__))
SERVEUR = os.path.join(ICI, "mcp_web_suite.py")


def verifier(nom, obtenu, attendu):
    assert obtenu == attendu, "%s : attendu %r, obtenu %r" % (nom, attendu, obtenu)


def echanger(lignes):
    """Envoie les lignes puis ferme stdin : rend (réponses décodées, code de sortie, stderr)."""
    p = subprocess.run([sys.executable, SERVEUR], input="".join(l + "\n" for l in lignes).encode("utf-8"),
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
    sortie = p.stdout.decode("utf-8")
    # stdout ne porte que du JSON-RPC : chaque ligne doit se décoder
    reponses = [json.loads(l) for l in sortie.split("\n") if l]
    verifier("pas de \\r sur stdout", "\r" in sortie, False)
    return reponses, p.returncode, p.stderr.decode("utf-8", "replace")


def msg(id_, methode, params=None):
    m = {"jsonrpc": "2.0", "method": methode}
    if id_ is not None:
        m["id"] = id_
    if params is not None:
        m["params"] = params
    return json.dumps(m)


# Ouverture de session, notification sans réponse, ping, liste des outils
reponses, code, _ = echanger([
    msg(1, "initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                          "clientInfo": {"name": "banc", "version": "0"}}),
    msg(None, "notifications/initialized"),
    msg(2, "ping"),
    msg(3, "tools/list"),
])
verifier("fin de stdin : sortie propre", code, 0)
verifier("une réponse par requête, aucune pour la notification", [r["id"] for r in reponses], [1, 2, 3])
init = reponses[0]["result"]
verifier("version reprise du client", init["protocolVersion"], "2025-06-18")
verifier("nom du serveur", init["serverInfo"]["name"], "web_suite")
verifier("capacité outils", "tools" in init["capabilities"], True)
verifier("ping", reponses[1]["result"], {})
outils = reponses[2]["result"]["tools"]
verifier("chaque outil a nom, description, schéma",
         all({"name", "description", "inputSchema"} <= set(o) for o in outils), True)

# Version inconnue du client : le serveur propose la sienne
reponses, _, _ = echanger([msg(1, "initialize", {"protocolVersion": "1999-01-01"})])
verifier("version par défaut", reponses[0]["result"]["protocolVersion"], "2025-06-18")

# Erreurs : le serveur répond et continue
reponses, code, _ = echanger([
    "ceci n'est pas du JSON",
    msg(4, "methode/inconnue"),
    msg(5, "tools/call", {"name": "outil_inconnu", "arguments": {}}),
    msg(6, "ping"),
])
verifier("ligne illisible", reponses[0]["error"]["code"], -32700)
verifier("id nul pour une ligne illisible", reponses[0]["id"], None)
verifier("méthode inconnue", (reponses[1]["id"], reponses[1]["error"]["code"]), (4, -32601))
verifier("outil inconnu : résultat en erreur", (reponses[2]["id"], reponses[2]["result"]["isError"]), (5, True))
verifier("toujours vivant", (reponses[3]["id"], reponses[3]["result"]), (6, {}))
verifier("toujours une sortie propre", code, 0)

# Message JSON valide mais pas un objet : requête invalide
reponses, _, _ = echanger(["[1, 2]", msg(7, "ping")])
verifier("requête invalide", reponses[0]["error"]["code"], -32600)
verifier("et la suite répond", reponses[1]["id"], 7)
print("OK")
