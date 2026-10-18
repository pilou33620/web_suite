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
FauxApi.recues.clear()
doc = {"pistes": [[0, 0]]}
verifier("cao_doc_ecrire", outil("cao_doc_ecrire", chemin="p", doc="pcb", document=doc, modifie=123)[0], False)
verifier("PUT du document avec modifie", FauxApi.recues,
         [("PUT", "/api/projet/doc" + q(chemin="p", doc="pcb", modifie=123), doc)])
FauxApi.recues.clear()
for args in ({"chemin": "p", "doc": "pcb", "document": doc},                     # pas de modifie
             {"chemin": "p", "doc": "autre", "document": doc, "modifie": 1},
             {"chemin": "p", "doc": "pcb", "document": "texte", "modifie": 1}):
    verifier("cao_doc_ecrire refuse %r" % sorted(args), outil("cao_doc_ecrire", **args)[0], True)
verifier("aucune écriture refusée envoyée", FauxApi.recues, [])
FauxApi.table[("PUT", "/api/projet/doc")] = (409, {"detail": "modifié ailleurs"})
erreur, texte = outil("cao_doc_ecrire", chemin="p", doc="pcb", document=doc, modifie=1)
verifier("409 lisible", (erreur, "relis" in texte), (True, True))

FauxApi.table[("GET", "/api/profils")] = (200, {"profils": [{"nom": "JLC 4c", "t": 1}]})
FauxApi.table[("GET", "/api/lib/fichiers")] = (200, {"pcb": ["R0603.json"], "schematique": [], "simulation": []})
FauxApi.recues.clear()
profil = {"format": "cao-profil-1", "couches": 4}
for nom, args in (("cao_profil_ecrire", {"nom": "jlc   4C", "profil": profil}),      # même nom, normalisé
                  ("cao_profil_ecrire", {"nom": "neuf", "profil": "texte"}),
                  ("lib_fichier_ecrire", {"type": "pcb", "nom": "r0603.json", "data": {}}),
                  ("lib_fichier_ecrire", {"type": "pcb", "nom": "x\\R0603.json", "data": {}}),
                  ("lib_fichier_ecrire", {"type": "pcb", "nom": "neuf.json"}),           # ni data ni contenu
                  ("lib_fichier_ecrire", {"type": "pcb", "nom": "neuf.json", "data": {}, "contenu": "x"}),
                  ("lib_fichier_ecrire", {"type": "autre", "nom": "neuf.json", "data": {}})):
    verifier("%s refuse %r" % (nom, args), outil(nom, **args)[0], True)
verifier("aucune écriture refusée envoyée (bis)", ecritures(), [])
FauxApi.recues.clear()
verifier("profil neuf", outil("cao_profil_ecrire", nom="neuf", profil=profil)[0], False)
verifier("fichier LIB neuf", outil("lib_fichier_ecrire", type="pcb", nom="C0402.json", data={"a": 1})[0], False)
verifier("fichier LIB texte", outil("lib_fichier_ecrire", type="simulation", nom="m.sub", contenu="* x")[0], False)
verifier("écritures permises", ecritures(), [
    ("PUT", "/api/profil" + q(nom="neuf"), profil),
    ("POST", "/api/lib/fichier", {"type": "pcb", "nom": "C0402.json", "data": {"a": 1}}),
    ("POST", "/api/lib/fichier", {"type": "simulation", "nom": "m.sub", "contenu": "* x"})])
FauxApi.table.clear()
os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps(PORTS)
print("OK")
