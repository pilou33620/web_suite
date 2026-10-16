"""Banc du serveur MCP (mcp_web_suite.py) : protocole JSON-RPC sur stdio, outil etat,
porte vers les outils et routes refusées.

    python lanceur/banc_mcp.py

Lance le serveur en sous-processus, lui écrit des lignes sur stdin et lit ses
réponses sur stdout, comme le fait Claude Code. Un faux WEB_CAO (serveur HTTP
local) compte les requêtes reçues ; les autres outils sont sur des ports fermés.
Affiche OK si tout passe.
"""
import http.server
import json
import os
import socket
import subprocess
import sys
import threading

ICI = os.path.dirname(os.path.abspath(__file__))
SERVEUR = os.path.join(ICI, "mcp_web_suite.py")


class FauxOutil(http.server.BaseHTTPRequestHandler):
    recues = []

    def _repondre(self):
        FauxOutil.recues.append((self.command, self.path))
        corps = json.dumps({"route": self.path}).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(corps)))
        self.end_headers()
        self.wfile.write(corps)

    def do_GET(self):
        self._repondre()

    def do_POST(self):
        self.rfile.read(int(self.headers.get("Content-Length") or 0))
        self._repondre()

    def log_message(self, *_):
        pass


def port_ferme():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


faux = http.server.ThreadingHTTPServer(("127.0.0.1", 0), FauxOutil)
threading.Thread(target=faux.serve_forever, daemon=True).start()
PORTS = {"web_suite": port_ferme(), "web_cao": faux.server_address[1],
         "web_antenna": port_ferme(), "web_3d": port_ferme()}
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(PORTS)   # hérité par le sous-processus


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

# Outil etat, par le protocole : le faux WEB_CAO tourne, le reste est arrêté
verifier("etat est listé", "etat" in [o["name"] for o in outils], True)
reponses, _, _ = echanger([msg(8, "tools/call", {"name": "etat", "arguments": {}})])
resultat = reponses[0]["result"]
verifier("etat sans erreur", resultat["isError"], False)
etat = {e["outil"]: e for e in json.loads(resultat["content"][0]["text"])}
verifier("les quatre", sorted(etat), ["web_3d", "web_antenna", "web_cao", "web_suite"])
verifier("lancés", {k: e["lance"] for k, e in etat.items()},
         {"web_suite": False, "web_cao": True, "web_antenna": False, "web_3d": False})
verifier("ports", {k: e["port"] for k, e in etat.items()}, PORTS)
verifier("adresse", etat["web_cao"]["adresse"], "http://127.0.0.1:%d/" % PORTS["web_cao"])

# La porte vers les outils, appelée directement
sys.path.insert(0, ICI)
import mcp_web_suite as mcp  # noqa: E402

FauxOutil.recues.clear()
for route in ("/api/github/envoyer", "/api/ia/cle", "/API/GitHub/envoyer", "/api/./github/envoyer",
              "/api//ia/cle", "/x/../api/github/identite", "@evil.example/", "api/projets",
              "/api/github", "/api/%67ithub/envoyer", "/api/ia/cle?x=1"):
    try:
        mcp.appeler_outil_web("web_cao", route, {})
        raise AssertionError("route permise à tort : " + route)
    except mcp.ErreurOutil:
        pass
verifier("aucune requête refusée n'atteint l'outil", FauxOutil.recues, [])
verifier("route permise, en GET", mcp.appeler_outil_web("web_cao", "/api/projets"), {"route": "/api/projets"})
verifier("route permise, en POST", mcp.appeler_outil_web("web_cao", "/api/oeil", {"a": 1})["route"], "/api/oeil")
verifier("requêtes reçues", FauxOutil.recues, [("GET", "/api/projets"), ("POST", "/api/oeil")])

try:
    mcp.appeler_outil_web("web_3d", "/api/projets")
    raise AssertionError("un outil arrêté devrait échouer")
except mcp.OutilArrete as exc:
    verifier("message d'outil arrêté", str(exc), "WEB_3D ne tourne pas : lance-le depuis WEB·SUITE")
try:
    mcp.appeler_outil_web("inconnu", "/")
    raise AssertionError("un outil inconnu devrait échouer")
except mcp.ErreurOutil:
    pass

# Un outil MCP qui plante : résultat en erreur, pas d'exception
mcp.OUTILS_MCP["boum"] = {"description": "", "inputSchema": {}, "fn": lambda a: 1 / 0}
verifier("panne d'outil", mcp.appeler({"name": "boum"})["isError"], True)
print("OK")
