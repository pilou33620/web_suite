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
import urllib.parse

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
              "/api/github", "/api/%67ithub/envoyer", "/api/ia/cle?x=1",
              "/api/envoyer", "/api/identite", "/api/appairer",
              # urlparse du lanceur coupe « ;… » : /api/envoyer;x y devient /api/envoyer
              "/api/envoyer;x", "/api/github/envoyer;x", "/api/projets x", "/api/projets\tx"):
    try:
        mcp.appeler_outil_web("web_cao", route, {})
        raise AssertionError("route permise à tort : " + route)
    except mcp.ErreurOutil:
        pass
verifier("aucune requête refusée n'atteint l'outil", FauxOutil.recues, [])
verifier("route permise, en GET", mcp.appeler_outil_web("web_cao", "/api/projets"), {"route": "/api/projets"})
verifier("route permise, en POST", mcp.appeler_outil_web("web_cao", "/api/oeil", {"a": 1})["route"], "/api/oeil")
verifier("requêtes reçues", FauxOutil.recues, [("GET", "/api/projets"), ("POST", "/api/oeil")])

# Le lanceur est en lecture seule : ses POST installent, lancent, arrêtent, publient
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_suite=PORTS["web_cao"]))
FauxOutil.recues.clear()
for route in ("/api/lancer", "/api/arreter", "/api/installer"):
    try:
        mcp.appeler_outil_web("web_suite", route, {"id": "web_3d"})
        raise AssertionError("POST permis au lanceur : " + route)
    except mcp.ErreurOutil:
        pass
verifier("aucun POST n'atteint le lanceur", FauxOutil.recues, [])
verifier("GET permis au lanceur", mcp.appeler_outil_web("web_suite", "/api/etat")["route"], "/api/etat")
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(PORTS)

# Le lanceur tourne et a lancé WEB_3D sur un port décalé (le sien était pris) : on le suit
class FauxLanceur(FauxOutil):
    def _repondre(self):
        corps = json.dumps({"outils": [
            {"id": "web_3d", "phase": "actif", "port": PORTS["web_cao"]},       # port décalé
            {"id": "web_antenna", "phase": "arret", "port": 1},                 # arrêté : ignoré
            {"id": "web_cao", "phase": "actif", "port": "pas un port"},         # illisible : ignoré
        ]}).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(corps)))
        self.end_headers()
        self.wfile.write(corps)


lanceur = http.server.ThreadingHTTPServer(("127.0.0.1", 0), FauxLanceur)
threading.Thread(target=lanceur.serve_forever, daemon=True).start()
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_suite=lanceur.server_address[1]))
vus = {e["outil"]: (e["port"], e["lance"]) for e in mcp.etat_outils()}
verifier("WEB_3D suivi sur son port décalé", vus["web_3d"], (PORTS["web_cao"], True))
verifier("outil arrêté : port du catalogue", vus["web_antenna"], (PORTS["web_antenna"], False))
verifier("port illisible : port du catalogue", vus["web_cao"][0], PORTS["web_cao"])
verifier("appel vers le port décalé", mcp.appeler_outil_web("web_3d", "/api/projets")["route"], "/api/projets")
lanceur.shutdown()
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(PORTS)


# Pannes de l'autre côté : jamais d'exception qui casse etat ou un appel
class Reponse(FauxOutil):
    corps, type_ = b"", "application/json"

    def _repondre(self):
        self.send_response(200)
        self.send_header("Content-Type", self.type_)
        self.send_header("Content-Length", str(len(self.corps)))
        self.end_headers()
        self.wfile.write(self.corps)


class JsonIllisible(Reponse):
    corps = b"{pas du json"


class LanceurBizarre(Reponse):
    corps = json.dumps({"outils": ["texte", None, {"id": "web_3d", "phase": "actif", "port": True}]}).encode()


def servir(classe):
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), classe)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


pas_http = socket.socket()
pas_http.bind(("127.0.0.1", 0))
pas_http.listen(5)


def repondre_ssh():
    while True:
        c, _ = pas_http.accept()
        c.sendall(b"SSH-2.0-OpenSSH\r\n")
        c.close()


threading.Thread(target=repondre_ssh, daemon=True).start()
illisible, bizarre = servir(JsonIllisible), servir(LanceurBizarre)
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_suite=bizarre.server_address[1],
                                                    web_cao=pas_http.getsockname()[1],
                                                    web_antenna=illisible.server_address[1]))
vus = {e["outil"]: (e["port"], e["lance"]) for e in mcp.etat_outils()}
verifier("entrées illisibles du lanceur ignorées", vus["web_3d"], (PORTS["web_3d"], False))
verifier("port pris par un service non HTTP : pas WEB_CAO", vus["web_cao"][1], False)
for outil, attendu in (("web_cao", mcp.OutilArrete), ("web_antenna", mcp.ErreurOutil)):
    try:
        mcp.appeler_outil_web(outil, "/api/projets")
        raise AssertionError("aurait dû échouer : " + outil)
    except attendu:
        pass
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_suite=illisible.server_address[1]))
mcp._cache_ports = (0.0, None, None)
verifier("lanceur au JSON illisible : ports du catalogue", mcp.ports(), dict(PORTS, web_suite=illisible.server_address[1]))
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(PORTS)

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
del mcp.OUTILS_MCP["boum"]


# ---------------------------------------------------------------------------
# api-outils (lanceur/docs/SPEC-mcp-api-outils.md)
class FauxApi(FauxOutil):
    """Faux outil piloté par une table : (méthode, chemin sans requête) -> (statut, JSON).
    Note (méthode, chemin complet, corps) de chaque requête reçue."""
    table, recues = {}, []

    def _repondre(self):
        n = int(self.headers.get("Content-Length") or 0)
        FauxApi.recues.append((self.command, self.path, json.loads(self.rfile.read(n)) if n else None))
        statut, rep = FauxApi.table.get((self.command, self.path.split("?")[0]), (200, {"route": self.path}))
        corps = json.dumps(rep).encode("utf-8")
        self.send_response(statut)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(corps)))
        self.end_headers()
        self.wfile.write(corps)

    do_GET = do_POST = do_PUT = do_DELETE = _repondre


def outil(_outil, **args):
    """Appelle un outil MCP comme le fait Claude : (isError, contenu décodé, ou texte)."""
    r = mcp.appeler({"name": _outil, "arguments": args})
    texte = r["content"][0]["text"]
    try:
        return r["isError"], json.loads(texte)
    except ValueError:
        return r["isError"], texte


def q(**params):
    return "?" + urllib.parse.urlencode(params)


api = servir(FauxApi)
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_cao=api.server_address[1]))
noms = [o["name"] for o in outils]   # tools/list du tout premier échange, par le protocole
verifier("outils du socle listés", {"etat", "cao_projets", "cao_projet_lire", "cao_doc_lire"} <= set(noms), True)

# Méthodes : PUT permis aux outils, DELETE jamais, le lanceur en GET seulement
FauxApi.recues.clear()
for id_, route, corps, methode in (("web_cao", "/api/lib/fichier", None, "DELETE"),
                                   ("web_suite", "/api/etat", {}, "PUT"), ("web_suite", "/api/etat", None, "PUT")):
    try:
        mcp.appeler_outil_web(id_, route, corps, methode=methode)
        raise AssertionError("méthode permise à tort : %s %s" % (methode, id_))
    except mcp.ErreurOutil:
        pass
verifier("aucune méthode refusée n'atteint l'outil", FauxApi.recues, [])
mcp.appeler_outil_web("web_cao", "/api/profil", {"a": 1}, methode="PUT")
verifier("PUT avec son corps", FauxApi.recues, [("PUT", "/api/profil", {"a": 1})])

# Lectures de projets CAO
FauxApi.recues.clear()
verifier("cao_projets", outil("cao_projets"), (False, {"route": "/api/projets"}))
outil("cao_projet_lire", chemin="a b/c", racine="/r")
outil("cao_projet_lire", chemin="seul")
outil("cao_doc_lire", chemin="p", doc="pcb")
verifier("routes et méthodes", FauxApi.recues, [
    ("GET", "/api/projets", None),
    ("GET", "/api/projet" + q(chemin="a b/c", racine="/r"), None),
    ("GET", "/api/projet" + q(chemin="seul"), None),
    ("GET", "/api/projet/doc" + q(chemin="p", doc="pcb"), None)])

# Arguments invalides : erreur qui nomme l'argument, aucune requête
FauxApi.recues.clear()
for nom, args, fautif in (("cao_projet_lire", {}, "chemin"), ("cao_projet_lire", {"chemin": 5}, "chemin"),
                          ("cao_doc_lire", {"chemin": "p", "doc": "autre"}, "doc"),
                          ("cao_doc_lire", {"chemin": "p", "doc": "pcb", "parties": "nom"}, "parties")):
    erreur, texte = outil(nom, **args)
    verifier("refus de %s %r" % (nom, args), (erreur, fautif in texte), (True, True))
verifier("aucun argument invalide n'atteint l'outil", FauxApi.recues, [])

# Grosse réponse : résumé sous le plafond, « modifie » toujours là ; « parties » pour le détail
gros = {"document": {"pistes": [[i, i] for i in range(30000)], "nom": "carte"}, "fichier": "x-PCB.json",
        "modifie": 123}
FauxApi.table[("GET", "/api/projet/doc")] = (200, gros)
r = mcp.appeler({"name": "cao_doc_lire", "arguments": {"chemin": "p", "doc": "pcb"}})
verifier("réponse de plus de 300 Ko", len(json.dumps(gros)) > 300 * 1024, True)
verifier("résumé sous 100 Ko", len(r["content"][0]["text"].encode("utf-8")) < 100 * 1024, True)
_, rendu = outil("cao_doc_lire", chemin="p", doc="pcb")
verifier("modifie gardé", (rendu["modifie"], rendu["fichier"]), (123, "x-PCB.json"))
verifier("document résumé", (rendu["document"]["resume"], rendu["document"]["forme"]["pistes"]["longueur"]),
         (True, 30000))
verifier("parties", outil("cao_doc_lire", chemin="p", doc="pcb", parties=["nom", "absente"])[1]["document"],
         {"nom": "carte"})
verifier("parties absentes signalées",
         outil("cao_doc_lire", chemin="p", doc="pcb", parties=["absente"])[1].get("parties_absentes"), ["absente"])
FauxApi.table.clear()

# Lectures CAO, LIB, WEB_3D (le faux outil sert aussi de WEB_3D)
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_cao=api.server_address[1],
                                                    web_3d=api.server_address[1]))
FauxApi.table[("GET", "/api/oeil")] = (200, {"dispo": True, "gabarits": [
    {"id": "usb2", "nom": "USB 2.0", "polygone": [[0, 0]] * 50, "masque": {}}]})
FauxApi.table[("GET", "/api/crosstalk")] = (500, {"detail": "en panne"})
FauxApi.recues.clear()
erreur, solveurs = outil("cao_solveurs")
verifier("cao_solveurs malgré une panne", erreur, False)
verifier("les sept GET", sorted(r[1] for r in FauxApi.recues), sorted(
    ["/api/simulation", "/api/simulation-dc", "/api/crosstalk", "/api/simulation-rf", "/api/oeil",
     "/api/analyse-carte", "/api/ipc2581"]))
verifier("gabarits réduits", solveurs["oeil"]["gabarits"], [{"id": "usb2", "nom": "USB 2.0"}])
verifier("solveur en panne marqué", "erreur" in solveurs["crosstalk"], True)
verifier("solveur sain intact", solveurs["simulation"], {"route": "/api/simulation"})

FauxApi.recues.clear()
for nom, args in (("cao_profils", {}), ("cao_profil_lire", {"nom": "jlc 4c"}), ("lib_fichiers", {}),
                  ("lib_fichier_lire", {"type": "pcb", "nom": "R0603"}), ("lib_parasites", {}),
                  ("web3d_projets", {})):
    verifier("%s sans erreur" % nom, outil(nom, **args)[0], False)
verifier("routes des lectures", [r[1] for r in FauxApi.recues], [
    "/api/profils", "/api/profil" + q(nom="jlc 4c"), "/api/lib/fichiers",
    "/api/lib/fichier" + q(type="pcb", nom="R0603"), "/api/lib/parasites", "/api/projets"])
FauxApi.recues.clear()
verifier("type LIB inconnu refusé", outil("lib_fichier_lire", type="autre", nom="x")[0], True)
verifier("sans requête", FauxApi.recues, [])

FauxApi.table[("GET", "/api/lib/composants")] = (200, {
    "colonnes": ["ref", "valeur", "boitier"], "total": 120,
    "composants": [{"ref": "R%d" % i, "valeur": "10k" if i % 2 else "1µF", "boitier": "0603", "_id": i}
                   for i in range(120)]})
_, page = outil("lib_composants")
verifier("50 par défaut", (len(page["composants"]), page["total"], page["trouves"]), (50, 120, 120))
_, page = outil("lib_composants", filtre="10K", debut=10, nombre=5, colonnes=["ref"])
verifier("filtre sans casse, page, colonnes", page["composants"],
         [{"ref": "R%d" % i, "_id": i} for i in range(21, 31, 2)])
verifier("trouvés après filtre", page["trouves"], 60)
for args in ({"nombre": 0}, {"debut": -1}, {"colonnes": "ref"}):
    verifier("lib_composants refuse %r" % args, outil("lib_composants", **args)[0], True)
FauxApi.table.clear()

# Écritures CAO gardées : aucune écriture n'atteint l'outil quand un garde-fou refuse
ecritures = lambda: [r for r in FauxApi.recues if r[0] != "GET"]   # noqa: E731
DOC = "/api/projet/doc" + q(chemin="p", doc="pcb")
FauxApi.table[("GET", "/api/projet/doc")] = (200, {"document": {"a": 1, "b": 2}, "modifie": 123})
FauxApi.recues.clear()
verifier("cao_doc_ecrire", outil("cao_doc_ecrire", chemin="p", doc="pcb", document={"b": 3}, modifie=123)[0], False)
verifier("fusion des clés dans le document relu, PUT avec modifie", FauxApi.recues,
         [("GET", DOC, None), ("PUT", "/api/projet/doc" + q(chemin="p", doc="pcb", modifie=123), {"a": 1, "b": 3})])
FauxApi.table[("GET", "/api/projet/doc")] = (404, {"detail": "absent"})
FauxApi.recues.clear()
outil("cao_doc_ecrire", chemin="p", doc="pcb", document={"b": 3}, modifie=0)
verifier("document absent : créé avec les clés données", ecritures(),
         [("PUT", "/api/projet/doc" + q(chemin="p", doc="pcb", modifie=0), {"b": 3})])
FauxApi.table[("GET", "/api/projet/doc")] = (200, {"document": {"a": 1}, "modifie": 123})
FauxApi.recues.clear()
for args in ({"document": {"b": 3}},                                                  # pas de modifie
             {"document": {"b": 3}, "modifie": -1},                                   # -1 : pas de contrôle côté serveur
             {"document": {"b": 3}, "modifie": 1, "doc": "autre"},
             {"document": "texte", "modifie": 1},
             {"document": {}, "modifie": 1},
             {"document": {"resume": True, "taille_octets": 9, "forme": {}}, "modifie": 1},   # un résumé renvoyé
             {"document": {"pistes": {"type": "list", "longueur": 3}}, "modifie": 1},        # une forme renvoyée
             {"document": {"b": 3}, "modifie": 1, "chemin": "/etc"},
             {"document": {"b": 3}, "modifie": 1, "chemin": "a/../../b"},
             {"document": {"b": 3}, "modifie": 1, "chemin": "C:\\x"}):
    args = dict({"chemin": "p", "doc": "pcb"}, **args)
    verifier("cao_doc_ecrire refuse %r" % sorted(args.items()), outil("cao_doc_ecrire", **args)[0], True)
verifier("aucune écriture refusée envoyée", ecritures(), [])
FauxApi.table[("PUT", "/api/projet/doc")] = (409, {"detail": "modifié ailleurs"})
erreur, texte = outil("cao_doc_ecrire", chemin="p", doc="pcb", document={"b": 3}, modifie=1)
verifier("409 lisible", (erreur, "relis" in texte), (True, True))
verifier("chemin absolu refusé en lecture aussi", outil("cao_projet_lire", chemin="/etc")[0], True)

# Profils et fichiers LIB : « existe déjà » demandé au serveur (GET, 404 attendu), qui
# applique sa propre normalisation du nom (casse, lecteur Windows, Unicode)
profil = {"format": "cao-profil-1", "couches": 4}
FauxApi.recues.clear()
for statut in (200, 500):   # existe, ou contrôle impossible : jamais d'écriture
    FauxApi.table[("GET", "/api/profil")] = (statut, {})
    FauxApi.table[("GET", "/api/lib/fichier")] = (statut, {})
    verifier("profil refusé (%d)" % statut, outil("cao_profil_ecrire", nom="JLC 4c", profil=profil)[0], True)
    verifier("LIB refusé (%d)" % statut, outil("lib_fichier_ecrire", type="pcb", nom="C:R0603.json", data={})[0], True)
FauxApi.table[("GET", "/api/profil")] = (404, {})
FauxApi.table[("GET", "/api/lib/fichier")] = (404, {})
for nom, args in (("cao_profil_ecrire", {"nom": "neuf", "profil": "texte"}),
                  ("lib_fichier_ecrire", {"type": "pcb", "nom": "neuf.json"}),           # ni data ni contenu
                  ("lib_fichier_ecrire", {"type": "pcb", "nom": "neuf.json", "data": {}, "contenu": "x"}),
                  ("lib_fichier_ecrire", {"type": "pcb", "nom": "neuf.json", "data": 5}),
                  ("lib_fichier_ecrire", {"type": "autre", "nom": "neuf.json", "data": {}})):
    verifier("%s refuse %r" % (nom, args), outil(nom, **args)[0], True)
verifier("aucune écriture refusée envoyée (bis)", ecritures(), [])
FauxApi.recues.clear()
verifier("profil neuf", outil("cao_profil_ecrire", nom="neuf", profil=profil)[0], False)
verifier("fichier LIB neuf", outil("lib_fichier_ecrire", type="pcb", nom="C0402.json", data={"a": 1})[0], False)
verifier("fichier LIB texte", outil("lib_fichier_ecrire", type="simulation", nom="m.sub", contenu="* x")[0], False)
verifier("contrôles puis écritures", FauxApi.recues, [
    ("GET", "/api/profil" + q(nom="neuf"), None), ("PUT", "/api/profil" + q(nom="neuf"), profil),
    ("GET", "/api/lib/fichier" + q(type="pcb", nom="C0402.json"), None),
    ("POST", "/api/lib/fichier", {"type": "pcb", "nom": "C0402.json", "data": {"a": 1}}),
    ("GET", "/api/lib/fichier" + q(type="simulation", nom="m.sub"), None),
    ("POST", "/api/lib/fichier", {"type": "simulation", "nom": "m.sub", "contenu": "* x"})])
FauxApi.table.clear()

# Plafond tenu sur le texte rendu à Claude, pas seulement sur le JSON compact
for nom_cas, document in (("coordonnées imbriquées de 91 Ko", {"pts": [[[i, i + 1]] for i in range(9000)]}),
                          ("100 clés de 5 Ko", {("k%03d" % i) + "x" * 5000: i for i in range(100)})):
    FauxApi.table[("GET", "/api/projet/doc")] = (200, {"document": document, "modifie": 1})
    r = mcp.appeler({"name": "cao_doc_lire", "arguments": {"chemin": "p", "doc": "pcb"}})
    verifier("sous 100 Ko : " + nom_cas, len(r["content"][0]["text"].encode("utf-8")) < 100 * 1024, True)
FauxApi.table.clear()

# Défense en profondeur : routes « Jamais » refusées par le noyau, quelle que soit la méthode
FauxApi.recues.clear()
for id_, route_ in (("web_antenna", "/api/openems/lancer"), ("web_antenna", "/api/openems/balayage/lancer"),
                    ("web_antenna", "/api/openems/arreter"), ("web_antenna", "/api/projet/racine"),
                    ("web_antenna", "/api/projet/dossier"), ("web_cao", "/api/lib/config"),
                    ("web_cao", "/api/datasheet/telecharger")):
    try:
        mcp.appeler_outil_web(id_, route_, {})
        raise AssertionError("route « Jamais » permise : " + route_)
    except mcp.ErreurOutil:
        pass
verifier("aucune route « Jamais » n'atteint l'outil", FauxApi.recues, [])

# Lecture plafonnée : une réponse énorme ne remplit pas la mémoire du Pi
FauxApi.table[("GET", "/api/projets")] = (200, {"x": "y" * 5000})
plafond, mcp.MAX_LECTURE = mcp.MAX_LECTURE, 1000
try:
    mcp.appeler_outil_web("web_cao", "/api/projets")
    raise AssertionError("réponse trop grosse acceptée")
except mcp.ErreurOutil:
    pass
mcp.MAX_LECTURE = plafond
FauxApi.table.clear()

# pcbparts par le relais de WEB_CAO : seulement les outils de sa liste blanche
FauxApi.table[("GET", "/api/tools")] = (200, {"tools": [{"name": "jlc_search"}, {"name": "autre"}],
                                              "allowed": ["jlc_search"]})
FauxApi.recues.clear()
verifier("pcbparts_outils", outil("pcbparts_outils")[1]["allowed"], ["jlc_search"])
verifier("pcbparts_appeler", outil("pcbparts_appeler", name="jlc_search", arguments={"q": "LM317"})[0], False)
verifier("appel relayé", FauxApi.recues[-1], ("POST", "/api/tool", {"name": "jlc_search", "arguments": {"q": "LM317"}}))
FauxApi.recues.clear()
verifier("outil hors liste refusé", outil("pcbparts_appeler", name="autre", arguments={})[0], True)
verifier("arguments non objet refusés", outil("pcbparts_appeler", name="jlc_search", arguments="x")[0], True)
verifier("arguments de plus de 2 Ko refusés", outil("pcbparts_appeler", name="jlc_search",
                                                    arguments={"q": "x" * 3000})[0], True)
verifier("seulement la lecture de la liste", ecritures(), [])
FauxApi.table.clear()

# Projets WEB_ANTENNA : seulement le projet ouvert ; jamais de null ; en-tête absent repris
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_antenna=api.server_address[1]))
TETE = {"titre": "T", "outil": "o", "source": "s", "fichier": "f", "dessin": {"d": 1}, "antenne": {"a": 1},
        "affichage": {"z": 1}}
FauxApi.table[("GET", "/api/projet")] = (200, {"ouvert": {"nom": "dipole", "dossier": "/x"}, "projets": []})
FauxApi.table[("GET", "/api/projet/ouvrir")] = (200, dict(TETE, nom="dipole", dossier="/x", carte={"c": 1},
                                                          resultats={"r": 1}))
FauxApi.recues.clear()
verifier("antenna_projet_lire", outil("antenna_projet_lire")[1]["nom"], "dipole")
verifier("lecture du projet ouvert", [r[1] for r in FauxApi.recues], ["/api/projet", "/api/projet/ouvrir" + q(nom="dipole")])
FauxApi.recues.clear()
verifier("enregistrer partiel", outil("antenna_projet_enregistrer", projet={"antenne": {"a": 2}})[0], False)
verifier("en-tête repris, ni carte ni résultats", ecritures(), [
    ("POST", "/api/projet/enregistrer", dict(TETE, nom="dipole", antenne={"a": 2}))])
FauxApi.recues.clear()
for args in ({"projet": {"carte": None}}, {"projet": {"nom": "autre", "antenne": {"a": 1}}}, {"projet": "x"},
             {"projet": {"carte": False}}, {"projet": {"resultats": ""}}, {"projet": {"dessin": 0}},
             {"projet": {"antenne": {}}}, {"projet": {"carte": {"type": "dict", "longueur": 3}}}):
    verifier("enregistrer refuse %r" % args, outil("antenna_projet_enregistrer", **args)[0], True)
verifier("aucun enregistrement refusé envoyé", ecritures(), [])
FauxApi.table[("GET", "/api/projet")] = (200, {"ouvert": None, "projets": []})
FauxApi.recues.clear()
verifier("aucun projet ouvert : lecture refusée", outil("antenna_projet_lire")[0], True)
verifier("aucun projet ouvert : enregistrement refusé", outil("antenna_projet_enregistrer", projet=TETE)[0], True)
verifier("jamais d'ouverture ni d'écriture", [r[1] for r in FauxApi.recues], ["/api/projet", "/api/projet"])
FauxApi.recues.clear()
outil("antenna_projets")
outil("antenna_calculs")
verifier("antenna_projets, antenna_calculs", [r[1] for r in FauxApi.recues], ["/api/projet", "/api/projet/calculs"])
FauxApi.table.clear()

# openEMS : chiffrer et suivre, jamais lancer ni arrêter
FauxApi.recues.clear()
modele = {"unite": "mm", "cuivre": []}
for quoi in ("modele", "balayage", "tableau_s", "script"):
    verifier("chiffrer %s" % quoi, outil("antenna_openems_chiffrer", quoi=quoi, document=modele)[0], False)
verifier("routes de chiffrage", FauxApi.recues, [
    ("POST", "/api/openems", modele), ("POST", "/api/openems/balayage", modele),
    ("POST", "/api/openems/tableau-s", modele), ("POST", "/api/openems/script", modele)])
FauxApi.recues.clear()
for args in ({"quoi": "lancer", "document": modele}, {"quoi": "balayage/lancer", "document": modele},
             {"quoi": "modele"}, {"quoi": "modele", "document": []}):
    verifier("chiffrer refuse %r" % args, outil("antenna_openems_chiffrer", **args)[0], True)
verifier("rien d'envoyé", FauxApi.recues, [])
outil("antenna_openems_etat")
outil("antenna_openems_suivre", id="a1b2 c3", depuis=3)
outil("antenna_openems_suivre", id="a1b2c3")
outil("antenna_openems_champs", id="a1b2c3")
verifier("état, suivi, champs", [r[1] for r in FauxApi.recues], [
    "/api/openems", "/api/openems/journal" + q(id="a1b2 c3", depuis=3), "/api/openems/journal" + q(id="a1b2c3"),
    "/api/openems/champs" + q(id="a1b2c3")])
verifier("suivre sans id refusé", outil("antenna_openems_suivre")[0], True)
verifier("jamais lancer ni arrêter", [r for r in FauxApi.recues
                                       if "lancer" in r[1] or "arreter" in r[1]], [])
FauxApi.table.clear()

# ---------------------------------------------------------------------------
# canal-pilote (lanceur/docs/SPEC-mcp-canal-pilote.md) : les outils déposent une commande
# sur /api/pilote/commande de l'outil, qui attend la réponse de la page
import base64  # noqa: E402

os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_cao=api.server_address[1],
                                                    web_antenna=api.server_address[1]))
verifier("outils du canal listés", {"pilote_onglets", "pilote_suivre", "pilote_capture", "cao_calcul",
                                    "cao_valeur_composant", "antenna_proposer"} <= set(noms), True)
FauxApi.table[("GET", "/api/pilote/onglets")] = (200, {"onglets": [{"id": "A", "page": "pcb"}]})
erreur, vus = outil("pilote_onglets")
verifier("onglets des outils lancés, outil arrêté dit", (erreur, vus["web_cao"], vus["web_3d"]),
         (False, [{"id": "A", "page": "pcb"}], "arrêté"))
FauxApi.table[("GET", "/api/pilote/onglets")] = (404, {"detail": "inconnue"})
verifier("outil sans canal (pas à jour)", "canal" in outil("pilote_onglets")[1]["web_cao"], True)

fin = lambda resultat, etat="finie": (200, {"id": "c" * 32, "nom": "x", "etat": etat, "resultat": resultat,  # noqa: E731
                                            "onglet": {"id": "A", "page": "pcb", "projet": "p", "appareil": "ce poste"}})
FauxApi.table[("POST", "/api/pilote/commande")] = fin({"rapport": "Z0 = 50 ohms"})
FauxApi.recues.clear()
erreur, rendu = outil("cao_calcul", analyse="impedance")
verifier("cao_calcul", (erreur, rendu["resultat"]), (False, {"rapport": "Z0 = 50 ohms"}))
outil("cao_calcul", analyse="crosstalk", onglet="B")
outil("cao_valeur_composant", ref="R12", valeur="4k7")
outil("antenna_proposer", carte={"type": "cotes", "cotes": {"L": 30}})
verifier("commandes déposées", [r[2] for r in FauxApi.recues], [
    {"page": ["pcb", "visionneuse-ipc"], "nom": "cao_calcul", "args": {"analyse": "impedance"}, "ecriture": False},
    {"page": ["pcb", "visionneuse-ipc"], "nom": "cao_calcul", "args": {"analyse": "crosstalk"}, "ecriture": False,
     "onglet": "B"},
    {"page": ["schema"], "nom": "cao_valeur_composant", "args": {"ref": "R12", "valeur": "4k7"}, "ecriture": True},
    {"page": ["antenna"], "nom": "antenna_proposer", "args": {"carte": {"type": "cotes", "cotes": {"L": 30}}},
     "ecriture": True}])
FauxApi.recues.clear()
for nom, args in (("cao_calcul", {"analyse": "tout"}), ("cao_calcul", {}),
                  ("cao_valeur_composant", {"ref": "", "valeur": "1k"}), ("cao_valeur_composant", {"ref": "R1"}),
                  ("antenna_proposer", {"carte": {"type": "lancer"}}), ("antenna_proposer", {"carte": "x"}),
                  ("pilote_capture", {"outil": "web_ferme"}), ("pilote_suivre", {"outil": "web_cao"})):
    verifier("%s refuse %r" % (nom, args), outil(nom, **args)[0], True)
verifier("aucune commande refusée déposée", FauxApi.recues, [])

# États rendus par la page : refusée n'est pas une erreur ; échouée et expirée en sont
FauxApi.table[("POST", "/api/pilote/commande")] = fin(None, "refusee")
verifier("refusée", outil("cao_valeur_composant", ref="R1", valeur="1k"), (False, outil("cao_valeur_composant",
         ref="R1", valeur="1k")[1]))
verifier("refusée dite", "refus" in json.dumps(outil("cao_valeur_composant", ref="R1", valeur="1k")[1]), True)
for etat in ("echouee", "expiree"):
    FauxApi.table[("POST", "/api/pilote/commande")] = fin("Repère R99 introuvable.", etat)
    erreur, texte = outil("cao_valeur_composant", ref="R99", valeur="1k")
    verifier("%s en erreur lisible" % etat, (erreur, "R99" in texte or "fermé" in texte or "expir" in texte),
             (True, True))
FauxApi.table[("POST", "/api/pilote/commande")] = fin("x" * 50000, "echouee")    # texte venu du projet : borné
verifier("échec borné", len(outil("cao_valeur_composant", ref="R1", valeur="1k")[1]) < 2200, True)
# Pas encore de réponse (ta carte attend) : en attente, avec l'id pour pilote_suivre
FauxApi.table[("POST", "/api/pilote/commande")] = fin(None, "proposee")
erreur, att = outil("cao_valeur_composant", ref="R1", valeur="1k")
verifier("en attente avec id", (erreur, att["etat"], att["id"]), (False, "en_attente", "c" * 32))
FauxApi.table[("GET", "/api/pilote/suivre")] = fin("R1 = 1k", "appliquee")
FauxApi.recues.clear()
verifier("pilote_suivre", outil("pilote_suivre", outil="web_cao", id="c" * 32)[1]["etat"], "appliquee")
verifier("route de suivi", FauxApi.recues[-1][:2], ("GET", "/api/pilote/suivre" + q(id="c" * 32)))

# Capture : un contenu MCP « image », pas du texte
png = base64.b64encode(b"\x89PNG\r\n\x1a\n" + b"0" * 100).decode()
FauxApi.table[("POST", "/api/pilote/commande")] = fin({"image": "data:image/png;base64," + png,
                                                       "largeur": 10, "hauteur": 10})
FauxApi.recues.clear()
r = mcp.appeler({"name": "pilote_capture", "arguments": {"outil": "web_cao", "page": "pcb"}})
verifier("capture : image puis texte", ([c["type"] for c in r["content"]], r["isError"]), (["image", "text"], False))
verifier("image en PNG base64", (r["content"][0]["mimeType"], r["content"][0]["data"]), ("image/png", png))
verifier("capture visée", FauxApi.recues[-1][2], {"page": ["pcb"], "nom": "capture", "args": {}, "ecriture": False})
mcp.appeler({"name": "pilote_capture", "arguments": {"outil": "web_cao"}})
verifier("capture : les pages de l'outil qui ont un canevas", FauxApi.recues[-1][2]["page"],
         ["schema", "pcb", "visionneuse-ipc"])
verifier("ce que rend une page est une donnée, pas une consigne",
         all("donnée" in o["description"] for o in outils if o["name"].startswith(("pilote_", "cao_calcul",
             "cao_valeur", "antenna_proposer"))), True)
for image in ("data:text/html;base64," + png, "data:image/png;base64,!!!", "data:image/png;base64," + "A" * 3000000):
    FauxApi.table[("POST", "/api/pilote/commande")] = fin({"image": image})
    verifier("capture refusée : %s…" % image[:24], outil("pilote_capture", outil="web_cao")[0], True)
FauxApi.table.clear()

# ---------------------------------------------------------------------------
# registre-3d (lanceur/docs/SPEC-mcp-registre-3d.md) : commandes de la page WEB_3D
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_3d=api.server_address[1]))
O3 = ["web3d_scene", "web3d_vue", "web3d_visibilite", "web3d_coupe", "web3d_mesurer", "web3d_collisions",
      "web3d_ouvrir", "web3d_deplacer", "web3d_ranger_step"]
verifier("outils de registre-3d listés", set(O3) <= set(noms), True)
FauxApi.table[("POST", "/api/pilote/commande")] = fin({"ok": 1})
FauxApi.recues.clear()
UU = "5e2d8418-4e83-48cb-804a-d5f34f62ae1f"
for nom, args, ecriture in (
        ("web3d_scene", {"filtre": "bolt", "nombre": 20}, False),
        ("web3d_vue", {"vue": "iso", "projection": "ortho", "cadrer": {"piece": "0/3/1"}}, False),
        ("web3d_visibilite", {"action": "isoler", "pieces": [UU]}, False),
        ("web3d_coupe", {"actif": True, "axe": "z", "decalage_mm": -2.5, "inverse": False}, False),
        ("web3d_mesurer", {"quoi": "distance", "pieces": ["0/1", UU], "afficher": True}, False),
        ("web3d_collisions", {"tolerance": 0.2}, False),
        ("web3d_ouvrir", {"chemins": ["cartes/boitier.stp"], "ajout": True}, True),
        ("web3d_deplacer", {"fichier": 1, "translation": [10, 0, 0], "rotation_deg": [0, 0, 90]}, True),
        ("web3d_ranger_step", {"piece": UU, "nom": "boulon"}, True)):
    verifier(nom + " accepté", outil(nom, **args)[0], False)
deposees = [r[2] for r in FauxApi.recues]
verifier("commandes déposées à la page 3d, avec leurs arguments", [(d["nom"], d["page"], d["ecriture"]) for d in deposees],
         [(n, ["3d"], e) for n, e in zip(O3, [False] * 6 + [True] * 3)])
verifier("arguments transmis tels quels", deposees[7]["args"], {"fichier": 1, "translation": [10, 0, 0], "rotation_deg": [0, 0, 90]})

FauxApi.recues.clear()
for nom, args in (("web3d_vue", {"vue": "plongee"}), ("web3d_vue", {"projection": "fisheye"}),
                  ("web3d_visibilite", {"action": "exploser"}), ("web3d_visibilite", {"action": "masquer", "pieces": "x"}),
                  ("web3d_coupe", {"axe": "w"}), ("web3d_coupe", {"decalage_mm": float("nan")}),
                  ("web3d_mesurer", {"quoi": "masse"}), ("web3d_mesurer", {"quoi": "distance", "pieces": [UU]}),
                  ("web3d_collisions", {"tolerance": -1}),
                  ("web3d_ouvrir", {"chemins": ["../web_3D.py"]}), ("web3d_ouvrir", {"chemins": []}),
                  ("web3d_ouvrir", {"chemins": ["/etc/x.stp"]}),
                  ("web3d_deplacer", {"fichier": -1, "translation": [1, 0, 0]}),
                  ("web3d_deplacer", {"fichier": 0, "translation": [1, 0]}),
                  ("web3d_deplacer", {"fichier": 0, "rotation_deg": [0, 0, "90"]}),
                  ("web3d_ranger_step", {"piece": ""}), ("web3d_ranger_step", {"piece": UU, "nom": "a/b"}),
                  # bornes et booléens stricts (revue) : « "false" » valait vrai, 1e9 mm cassait la vue
                  ("web3d_deplacer", {"fichier": 0, "translation": [1e9, 0, 0]}),
                  ("web3d_deplacer", {"fichier": 0, "rotation_deg": [0, 0, 1e5]}),
                  ("web3d_coupe", {"actif": "false"}), ("web3d_coupe", {"decalage_mm": 5e6}),
                  ("web3d_ouvrir", {"chemins": ["a.stp"], "ajout": "oui"}),
                  ("web3d_scene", {"nombre": 5000}), ("web3d_vue", {"capturer": 1})):
    verifier("%s refuse %r" % (nom, args), outil(nom, **args)[0], True)
verifier("aucun argument invalide n'atteint la page", FauxApi.recues, [])

# web3d_vue + capturer : la vue, puis la capture du même onglet, en image
FauxApi.table[("POST", "/api/pilote/commande")] = fin({"vue": "iso"})
FauxApi.recues.clear()
reponses = iter([fin({"vue": "iso", "cadre": "scène"}), fin({"image": "data:image/png;base64," + png, "largeur": 10, "hauteur": 10})])


_table = FauxApi.table
FauxApi.table = type("T", (dict,), {"get": lambda self, k, d=None: next(reponses) if k == ("POST", "/api/pilote/commande") else d})()
r = mcp.appeler({"name": "web3d_vue", "arguments": {"vue": "iso", "capturer": True}})
FauxApi.table = _table
verifier("web3d_vue + capturer : image puis texte", [c["type"] for c in r["content"]], ["image", "text"])
verifier("la capture vise l'onglet de la vue", FauxApi.recues[1][2], {"page": ["3d"], "nom": "capture", "args": {}, "ecriture": False, "onglet": "A"})
FauxApi.table.clear()

# ---------------------------------------------------------------------------
# registre-cao (lanceur/docs/SPEC-mcp-registre-cao.md) : lectures, vue et DRC des pages WEB_CAO
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_cao=api.server_address[1]))
OC = ["cao_schema_lire", "cao_pcb_lire", "cao_ipc_lire", "cao_sim_lire", "cao_vue", "cao_drc"]
verifier("outils de registre-cao listés", set(OC) <= set(noms), True)
verifier("58 outils en tout (registre-cao, ecriture-cao, dessin-schema, registre-antenna, routage)", len(noms), 58)
verifier("registre-cao : ce que rend la page est une donnée",
         all("donnée" in o["description"] for o in outils if o["name"] in OC), True)
FauxApi.table[("POST", "/api/pilote/commande")] = fin("- GND")
FauxApi.recues.clear()
for nom, args in (("cao_schema_lire", {"quoi": "net", "nom": "LOCAL", "feuille": 2}),
                  ("cao_pcb_lire", {"quoi": "nets", "filtre": "USB", "a_router": True}),
                  ("cao_ipc_lire", {"quoi": "composant", "ref": "U1"}),
                  ("cao_sim_lire", {"quoi": "simulation", "analyse": "pdn", "partie": "chiffres", "afficher": True}),
                  ("cao_vue", {"cadrer": "U1", "couches": ["Top"], "ajuster": True, "page": "pcb"}),
                  ("cao_drc", {})):
    verifier(nom + " accepté", outil(nom, **args)[0], False)
deposees = [r[2] for r in FauxApi.recues]
verifier("commandes déposées aux bonnes pages, sans écriture", [(d["nom"], d["page"], d["ecriture"]) for d in deposees],
         [("cao_schema_lire", ["schema"], False), ("cao_pcb_lire", ["pcb"], False),
          ("cao_ipc_lire", ["visionneuse-ipc"], False), ("cao_sim_lire", ["pcb", "visionneuse-ipc"], False),
          ("cao_vue", ["pcb"], False), ("cao_drc", ["pcb"], False)])
verifier("arguments transmis", [deposees[0]["args"], deposees[1]["args"], deposees[4]["args"]],
         [{"quoi": "net", "nom": "LOCAL", "feuille": 2}, {"quoi": "nets", "filtre": "USB", "a_router": True},
          {"cadrer": "U1", "couches": ["Top"], "ajuster": True}])
outil("cao_vue", surligner="GND")
verifier("cao_vue sans page : les trois pages qui ont une vue", FauxApi.recues[-1][2]["page"], ["schema", "pcb", "visionneuse-ipc"])

FauxApi.recues.clear()
for nom, args in (("cao_pcb_lire", {}), ("cao_pcb_lire", {"quoi": "feuilles"}), ("cao_schema_lire", {"quoi": "drc"}),
                  ("cao_ipc_lire", {"quoi": "classes"}), ("cao_sim_lire", {"quoi": "simulation", "partie": "tout"}),
                  ("cao_pcb_lire", {"quoi": "nets", "a_router": "oui"}), ("cao_pcb_lire", {"quoi": "net", "nom": 3}),
                  ("cao_pcb_lire", {"quoi": "nets", "filtre": "x" * 201}), ("cao_pcb_lire", {"quoi": "net", "nom": "a\nb"}),
                  ("cao_schema_lire", {"quoi": "net", "nom": "X", "feuille": 0}),
                  ("cao_schema_lire", {"quoi": "net", "nom": "X", "feuille": "2"}),
                  ("cao_vue", {}), ("cao_vue", {"page": "accueil", "ajuster": True}), ("cao_vue", {"couches": "Top"}),
                  ("cao_vue", {"couches": ["x"] * 65}), ("cao_vue", {"couches": [None]}), ("cao_vue", {"ajuster": 1}), ("cao_vue", {"capturer": "oui", "ajuster": True}),
                  ("cao_sim_lire", {"quoi": "simulations", "afficher": "true"})):
    verifier("%s refuse %r" % (nom, args), outil(nom, **args)[0], True)
verifier("aucun argument invalide n'atteint la page (cao)", FauxApi.recues, [])

# cao_vue + capturer : la vue, puis la capture du même onglet
reponses = iter([fin("Vue : cadré sur U1."), fin({"image": "data:image/png;base64," + png, "largeur": 10, "hauteur": 10})])
_table = FauxApi.table
FauxApi.table = type("T", (dict,), {"get": lambda self, k, d=None: next(reponses) if k == ("POST", "/api/pilote/commande") else d})()
r = mcp.appeler({"name": "cao_vue", "arguments": {"cadrer": "U1", "capturer": True}})
FauxApi.table = _table
verifier("cao_vue + capturer : image puis texte", [c["type"] for c in r["content"]], ["image", "text"])
verifier("la capture vise l'onglet de la vue (cao)", (FauxApi.recues[-1][2]["nom"], FauxApi.recues[-1][2]["onglet"]), ("capture", "A"))

# ecriture-cao (lanceur/docs/SPEC-mcp-ecriture-cao.md) : écritures en lot, par cartes
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_cao=api.server_address[1]))
OE = ["cao_composants_ecrire", "cao_pistes_ecrire", "cao_classes_ecrire", "cao_contraintes_ecrire", "cao_sim_ecrire"]
verifier("outils d'ecriture-cao listés", set(OE) <= set(noms), True)
verifier("ecriture-cao : donnée, carte, lot", all("donnée" in o["description"] and "carte" in o["description"] and "50" in o["description"]
                                                 for o in outils if o["name"] in OE), True)
FauxApi.table[("POST", "/api/pilote/commande")] = fin("ok", "appliquee")
FauxApi.recues.clear()
for nom, args in (("cao_composants_ecrire", {"page": "schema", "lot": [{"ref": "R1", "champs": {"valeur": "4k7", "specs": {"tol": "1%", "x": None}, "non_monte": True}}]}),
                  ("cao_composants_ecrire", {"page": "pcb", "lot": [{"ref": "U1", "champs": {"x": 12.5, "rotation": 90, "face": "dessous"}}]}),
                  ("cao_pistes_ecrire", {"lot": [{"net": "GND", "largeur": 0.5, "couche": "Top"}, {"net": "VCC", "realigner": True}]}),
                  ("cao_classes_ecrire", {"lot": [{"action": "creer", "nom": "P", "largeur": 0.8, "largeur_couches": {"Top": 0.9}},
                                                  {"action": "affecter", "net": "VCC", "classe": "P"}]}),
                  ("cao_contraintes_ecrire", {"lot": [{"action": "paire", "nom": "USB", "impedance": "D90"},
                                                      {"action": "net", "net": "CLK", "l_max": 40, "z": None, "couches": ["Top"]}]}),
                  ("cao_sim_ecrire", {"lot": [{"groupe": "pdn", "cle": "rVrmMOhm", "valeur": 12, "unite": "mΩ"},
                                              {"groupe": "bus", "cle": "protocole", "valeur": "I2C"}]})):
    verifier(nom + " accepté", outil(nom, **args)[0], False)
deposees = [r[2] for r in FauxApi.recues]
verifier("écritures déposées aux bonnes pages, en écriture", [(d["nom"], d["page"], d["ecriture"]) for d in deposees],
         [("cao_composants_ecrire", ["schema"], True), ("cao_composants_ecrire", ["pcb"], True),
          ("cao_pistes_ecrire", ["pcb"], True), ("cao_classes_ecrire", ["pcb"], True),
          ("cao_contraintes_ecrire", ["pcb"], True), ("cao_sim_ecrire", ["pcb", "visionneuse-ipc"], True)])
verifier("le lot passe tel quel", deposees[1]["args"], {"lot": [{"ref": "U1", "champs": {"x": 12.5, "rotation": 90, "face": "dessous"}}]})

FauxApi.recues.clear()
L = lambda *e: {"lot": list(e)}
for nom, args in (("cao_composants_ecrire", L({"ref": "R1", "champs": {"valeur": "1k"}})),                       # page manquante
                  ("cao_composants_ecrire", dict(L({"ref": "R1", "champs": {"x": 1}}), page="schema")),       # x : PCB seulement
                  ("cao_composants_ecrire", dict(L({"ref": "R1", "champs": {"face": "gauche"}}), page="pcb")),
                  ("cao_composants_ecrire", dict(L({"ref": "R1", "champs": {}}), page="pcb")),
                  ("cao_composants_ecrire", dict(L({"ref": "R1"}), page="pcb")),
                  ("cao_composants_ecrire", dict(L({"ref": "", "champs": {"valeur": "1"}}), page="pcb")),
                  ("cao_composants_ecrire", dict(L({"ref": "R1", "champs": {"valeur": "a\nb"}}), page="schema")),
                  ("cao_composants_ecrire", dict(L({"ref": "R1", "champs": {"specs": {"k%d" % i: "v" for i in range(31)}}}), page="schema")),
                  ("cao_composants_ecrire", dict(L({"ref": "R1", "champs": {"x": float("nan")}}), page="pcb")),
                  ("cao_composants_ecrire", dict(L({"ref": "R1", "champs": {"rotation": True}}), page="pcb")),
                  ("cao_composants_ecrire", {"page": "pcb", "lot": [{"ref": "R1", "champs": {"valeur": "1"}}] * 51}),
                  ("cao_composants_ecrire", {"page": "pcb", "lot": []}),
                  ("cao_pistes_ecrire", L({"largeur": 0.3})), ("cao_pistes_ecrire", L({"net": "GND", "largeur": "0.3"})),
                  ("cao_pistes_ecrire", L({"net": "GND", "epaisseur": 1})),
                  ("cao_classes_ecrire", L({"action": "renommer", "nom": "P"})), ("cao_classes_ecrire", L({"nom": "P"})),
                  ("cao_classes_ecrire", L({"action": "creer", "nom": "P", "largeur_couches": {"Top": "0.3"}})),
                  ("cao_contraintes_ecrire", L({"action": "net", "net": "CLK", "couches": []})),
                  ("cao_contraintes_ecrire", L({"action": "net", "net": "CLK", "l_max": "40"})),
                  ("cao_sim_ecrire", L({"groupe": "pdn", "cle": "rVrmMOhm"})),
                  ("cao_sim_ecrire", L({"groupe": "pdn", "cle": "rVrmMOhm", "valeur": None})),
                  ("cao_sim_ecrire", "pas un objet")):
    verifier("%s refuse %r" % (nom, str(args)[:60]), outil(nom, **args)[0] if isinstance(args, dict) else
             mcp.appeler({"name": nom, "arguments": {"lot": args}}).get("isError", False), True)
verifier("aucun lot invalide n'atteint la page", FauxApi.recues, [])

# dessin-schema (lanceur/docs/SPEC-mcp-dessin-schema.md) : une liste d'opérations, une carte
d_outil = next(o for o in outils if o["name"] == "cao_schema_dessiner")
verifier("cao_schema_dessiner : donnée, carte, 40", all(t in d_outil["description"] for t in ("donnée", "carte", "40")), True)
OPS = [{"op": "feuille", "creer": "Alim"}, {"op": "poser", "type": "regulator", "ref": "U1", "x": 0, "y": 0, "rotation": 90},
       {"op": "poser", "catalogue": "C0402_100NF", "feuille": 3}, {"op": "relier", "de": "U1.IN", "a": "C1.1"},
       {"op": "etiqueter", "broche": "U1.GND", "net": "GND", "genre": "gnd"}, {"op": "deplacer", "ref": "U1", "dx": 5, "dy": 0},
       {"op": "tourner", "ref": "U1", "angle": 180}, {"op": "miroir", "ref": "C1"}, {"op": "supprimer_fil", "de": "U1.IN", "a": "C1.1"},
       {"op": "supprimer", "ref": "C1"}, {"op": "feuille", "renommer": 1, "nom": "Entrée"}, {"op": "renumeroter"}]
FauxApi.recues.clear()
verifier("cao_schema_dessiner accepté", outil("cao_schema_dessiner", operations=OPS)[0], False)
d = FauxApi.recues[-1][2]
verifier("déposé au schéma, en écriture, opérations telles quelles", (d["nom"], d["page"], d["ecriture"], d["args"]),
         ("cao_schema_dessiner", ["schema"], True, {"lot": OPS}))
FauxApi.recues.clear()
for ops in ([], [{"op": "poser", "type": "resistor"}] * 41, [{"op": "voler"}], ["poser"], [{"type": "resistor"}],
            [{"op": "poser", "type": "resistor", "angle": 90}], [{"op": "poser", "type": "resistor", "rotation": 45}],
            [{"op": "tourner", "ref": "R1", "angle": 90.0}], [{"op": "relier", "de": "R1.1", "a": 2}],
            [{"op": "etiqueter", "broche": "R1.1", "net": "X", "genre": "bleu"}], [{"op": "poser", "type": "r", "feuille": 0}],
            [{"op": "poser", "type": "r", "x": float("inf"), "y": 0}], [{"op": "renumeroter", "feuille": 1}],
            [{"op": "relier", "de": "R1.1\nR2.1", "a": "R3.1"}]):
    verifier("cao_schema_dessiner refuse %r" % str(ops)[:50], outil("cao_schema_dessiner", operations=ops)[0], True)
verifier("aucune opération invalide n'atteint la page", FauxApi.recues, [])


# registre-antenna (lanceur/docs/SPEC-mcp-registre-antenna.md) : lectures et vue de la page WEB_ANTENNA
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_antenna=api.server_address[1]))
OA = ["antenna_lire", "antenna_vue"]
verifier("outils de registre-antenna listés", set(OA) <= set(noms), True)
verifier("registre-antenna : donnée, jamais de calcul", all("donnée" in o["description"] and "calcul" in o["description"]
                                                          for o in outils if o["name"] in OA), True)
FauxApi.table[("POST", "/api/pilote/commande")] = fin("f0 2.45 GHz")
FauxApi.recues.clear()
for nom, args in (("antenna_lire", {"quoi": "courbe", "grandeur": "z"}), ("antenna_lire", {"quoi": "reglages", "filtre": "bande"}),
                  ("antenna_lire", {"quoi": "composant", "ref": "U1"}),
                  ("antenna_vue", {"vue": "3d", "maillage": False, "resultats": "s21", "etape": 2, "panneau": "resultats", "toile": "3d"})):
    verifier(nom + " accepté", outil(nom, **args)[0], False)
deposees = [r[2] for r in FauxApi.recues]
verifier("antenna : pages, noms, sans écriture", [(d["nom"], d["page"], d["ecriture"]) for d in deposees],
         [("antenna_lire", ["antenna"], False)] * 3 + [("antenna_vue", ["antenna"], False)])
verifier("« resultats » arrive à la page comme onglet de résultats", deposees[3]["args"],
         {"vue": "3d", "maillage": False, "onglet": "s21", "etape": 2, "panneau": "resultats", "toile": "3d"})
FauxApi.recues.clear()
for nom, args in (("antenna_lire", {}), ("antenna_lire", {"quoi": "tout"}), ("antenna_lire", {"quoi": "courbe", "grandeur": "phase"}),
                  ("antenna_lire", {"quoi": "nets", "filtre": "a\nb"}), ("antenna_vue", {}), ("antenna_vue", {"vue": "4d"}),
                  ("antenna_vue", {"maillage": "oui"}), ("antenna_vue", {"resultats": "phase"}), ("antenna_vue", {"etape": 0}),
                  ("antenna_vue", {"point_balayage": 1.5}), ("antenna_vue", {"toile": "pdf"}), ("antenna_vue", {"capturer": True})):
    verifier("%s refuse %r" % (nom, args), outil(nom, **args)[0], True)
verifier("aucun argument invalide n'atteint la page (antenna)", FauxApi.recues, [])
reponses = iter([fin("Vue : vue 3D."), fin({"image": "data:image/png;base64," + png, "largeur": 10, "hauteur": 10})])
_table = FauxApi.table
FauxApi.table = type("T", (dict,), {"get": lambda self, k, d=None: next(reponses) if k == ("POST", "/api/pilote/commande") else d})()
r = mcp.appeler({"name": "antenna_vue", "arguments": {"vue": "3d", "capturer": True}})
FauxApi.table = _table
verifier("antenna_vue + capturer : image puis texte", [c["type"] for c in r["content"]], ["image", "text"])
verifier("la capture vise l'onglet de la vue (antenna)", (FauxApi.recues[-1][2]["nom"], FauxApi.recues[-1][2]["onglet"]), ("capture", "A"))


# routage (lanceur/docs/SPEC-mcp-routage.md) : routes par points de passage, paires, placement, scores
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(dict(PORTS, web_cao=api.server_address[1]))
OR = ["cao_router", "cao_paire", "cao_placement_auto", "cao_score"]
verifier("outils du routage listés", set(OR) <= set(noms), True)
verifier("routage : donnée ; les écritures disent la carte et l'aperçu",
         all("donnée" in o["description"] and (o["name"] == "cao_score" or "aperçu" in o["description"])
             for o in outils if o["name"] in OR), True)
FauxApi.table[("POST", "/api/pilote/commande")] = fin("ok", "appliquee")
FauxApi.recues.clear()
R1 = {"depart": "U1.3", "points": [{"x": 10, "y": 5.5}, {"x": 12, "y": 5.5, "couche": "Bottom"}], "arrivee": "R2.1", "largeur": 0.3}
for nom, args in (("cao_router", {"routes": [R1], "vias": [{"x": 1, "y": 2, "net": "GND"}], "mode": "contourner"}),
                  ("cao_paire", {"paire": "USB", "depart": {"x": 7, "y": 19.7}, "arrivee": "U1.4"}),
                  ("cao_placement_auto", {"methode": "blocs", "motif": "Alim"}),
                  ("cao_score", {"quoi": "routage"}),
                  ("cao_pcb_lire", {"quoi": "zone", "x1": 0, "y1": 0, "x2": 20, "y2": 10.5}),
                  ("cao_pcb_lire", {"quoi": "liaisons", "filtre": "GND"})):
    verifier(nom + " accepté", outil(nom, **args)[0], False)
deposees = [r[2] for r in FauxApi.recues]
verifier("routage : pages et écriture", [(d["nom"], d["page"], d["ecriture"]) for d in deposees],
         [("cao_router", ["pcb"], True), ("cao_paire", ["pcb"], True), ("cao_placement_auto", ["pcb"], True),
          ("cao_score", ["pcb"], False), ("cao_pcb_lire", ["pcb"], False), ("cao_pcb_lire", ["pcb"], False)])
verifier("la route arrive telle quelle", deposees[0]["args"]["routes"][0], R1)
verifier("la fenêtre de zone arrive", deposees[4]["args"], {"quoi": "zone", "x1": 0, "y1": 0, "x2": 20, "y2": 10.5})
FauxApi.recues.clear()
for nom, args in (("cao_router", {}), ("cao_router", {"routes": [R1] * 21}), ("cao_router", {"routes": [{"depart": "U1.3"}]}),
                  ("cao_router", {"routes": [dict(R1, depart={"x": "a", "y": 1})]}),
                  ("cao_router", {"routes": [dict(R1, points=[{"x": 1, "y": 2, "z": 3}])]}),
                  ("cao_router", {"routes": [dict(R1, points=[{"x": 1, "y": 2}] * 201)]}),
                  ("cao_router", {"routes": [R1], "mode": "sauter"}), ("cao_router", {"vias": [{"x": 1, "y": 2}]}),
                  ("cao_paire", {"paire": "USB", "depart": "J1.1"}), ("cao_paire", dict(R1, paire="USB", largeur=0.2)),
                  ("cao_placement_auto", {"methode": "hasard"}), ("cao_placement_auto", {"methode": "blocs", "motif": "a\nb"}),
                  ("cao_score", {"quoi": "tout"}), ("cao_pcb_lire", {"quoi": "zone", "x1": 5000}),
                  ("cao_pcb_lire", {"quoi": "zone", "x1": "5"}), ("cao_pcb_lire", {"quoi": "zone", "x1": float("nan")})):
    verifier("%s refuse %r" % (nom, str(args)[:60]), outil(nom, **args)[0], True)
verifier("aucun argument invalide n'atteint la page (routage)", FauxApi.recues, [])

FauxApi.table.clear()
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(PORTS)
print("OK")
