"""web_bench — banc d'essai visible des outils de web_suite (WEB_CAO, WEB_ANTENNA, WEB_3D).

    python web_bench/web_bench.py                 # outils cherchés à la racine de web_suite
    python web_bench/web_bench.py --outil 3d      # ouvre d'abord WEB_3D
    python web_bench/web_bench.py --racine D:\\WEB_TOOLS --port 8200

Ce que fait le serveur :
- il démarre l'outil choisi sur un port libre, avec des dossiers temporaires : projets vides
  et, pour WEB_CAO, une copie de la LIB. Rien de tes vrais projets n'est touché, et tout est
  effacé à l'arrêt ;
- il relaie les pages de l'outil actif sous sa propre adresse : la page Banc et l'outil sont
  ainsi de même origine, et la page Banc peut cliquer dans l'outil ouvert dans son cadre.
  Chaque page HTML relayée reçoit en tête la sonde (page/sonde.js), qui note les erreurs JS,
  les requêtes en échec et les dialogues ;
- il découvre et lance les scripts banc-* de chaque outil, un par un ;
- il garde les rapports du clic-partout dans web_bench/rapports/.

Écoute sur 127.0.0.1 uniquement : le relais parle aux outils depuis la boucle locale, il ne
doit donc jamais être ouvert au réseau (il contournerait l'appairage).
"""
import argparse
import datetime
import http.client
import http.server
import json
import os
import re
import shutil
import signal
import socket
import socketserver
import subprocess
import sys
import tempfile
import threading
import time
import urllib.parse
import urllib.request
import webbrowser

ICI = os.path.dirname(os.path.abspath(__file__))
RACINE_DEFAUT = os.path.dirname(ICI)                 # racine de web_suite, où vivent les outils
DOSSIER_PAGE = os.path.join(ICI, "page")
DOSSIER_RAPPORTS = os.path.join(ICI, "rapports")
PREFIXE = "/__banc"
ENTETE_BANC = "X-Web-Bench"

# Les outils essayés, et comment les lancer sans toucher aux données de l'utilisateur.
# {projets} et {lib} sont remplacés par les dossiers temporaires.
CATALOGUE = [
    {"id": "cao", "nom": "WEB_CAO", "script": "web_CAO.py", "lib": True,
     "args": ["--local", "--sans-navigateur", "--sans-pause", "--sans-maj", "--projets", "{projets}", "--lib", "{lib}"],
     # la simulation se lance à la main ici : rien de lourd par défaut
     "noire": ""},
    {"id": "antenna", "nom": "WEB_ANTENNA", "script": "web_antenna.py", "lib": False,
     # --sans-banc : pas de mesure de vitesse du poste au démarrage
     "args": ["--local", "--sans-navigateur", "--sans-maj", "--sans-banc", "--projets", "{projets}"],
     "noire": "",
     # une simulation openEMS peut durer des heures : le bouton est cliqué (son code est essayé),
     # mais la requête qui la lancerait est arrêtée par le relais
     "routes": r"^/api/openems/.*lancer"},
    {"id": "3d", "nom": "WEB_3D", "script": "web_3D.py", "lib": False,
     "args": ["--local", "--sans-navigateur", "--sans-pause", "--sans-maj", "--projets", "{projets}"],
     "noire": ""},
]
PAR_ID = {o["id"]: o for o in CATALOGUE}
# Requêtes arrêtées par le relais pour tous les outils, quel que soit le bouton qui les envoie :
# rien ne part sur GitHub, rien n'est demandé à Claude Code.
ROUTES_BLOQUEES = r"^/api/github/envoyer|^/api/pilote/demande"

# en-têtes propres à une connexion : ils ne passent pas le relais
SAUTS = {"connection", "keep-alive", "proxy-authenticate", "proxy-authorization", "te",
         "trailer", "transfer-encoding", "upgrade", "content-length", "accept-encoding"}
# dossiers où l'on ne cherche ni bancs ni pages
DOSSIERS_IGNORES = {".git", "graphify-out", "node_modules", "tasks", "__pycache__", "projets", "LIB",
                    "vendor", "env", "venv", ".venv", "dist", "docs"}
BANC = re.compile(r"^banc[-_].+\.(py|js|mjs)$")
DELAI_BANC = 300          # s : au-delà, un banc est arrêté et compté « délai dépassé »
MAX_SORTIE = 40000        # caractères gardés de la sortie d'un banc
MAX_JOURNAL = 4000        # lignes gardées du journal d'un outil


def port_libre():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def trouver_chromium():
    if os.environ.get("CHROMIUM"):
        return os.environ["CHROMIUM"]
    for nom in ("chromium", "chromium-browser", "google-chrome", "google-chrome-stable", "chrome", "msedge"):
        chemin = shutil.which(nom)
        if chemin:
            return chemin
    for chemin in (r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                   r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"):
        if os.path.isfile(chemin):
            return chemin
    return None


# --------------------------------------------------------------------------------------
# Un outil essayé, lancé à part avec des dossiers temporaires
# --------------------------------------------------------------------------------------
class Outil:
    def __init__(self, conf, dossier):
        self.conf = conf
        self.dossier = dossier
        self.port = None
        self.processus = None
        self.temp = None
        self.journal = []          # lignes de sortie de l'outil
        self.debut_journal = 0     # numéro de la première ligne gardée
        self.verrou = threading.Lock()
        self.verrou_demarrage = threading.Lock()
        self.version = version_git(dossier)

    @property
    def nom(self):
        return self.conf["nom"]

    def lib_reelle(self):
        """La LIB que WEB_CAO prend par défaut (même ordre que lui et ses bancs) : WEB_CAO_LIB,
        LIB/ à côté de l'outil, sinon ../PROJETS/LIB_CAO de web_suite."""
        for c in (os.environ.get("WEB_CAO_LIB"), os.path.join(self.dossier, "LIB"),
                  os.path.join(os.path.dirname(self.dossier), "PROJETS", "LIB_CAO")):
            if c and os.path.isdir(c):
                return c
        return None

    def demarrer(self):
        with self.verrou_demarrage:
            if self.en_marche():
                return
            self._demarrer()

    def _demarrer(self):
        self.temp = tempfile.mkdtemp(prefix="web_bench_%s_" % self.conf["id"])
        projets = os.path.join(self.temp, "projets")
        lib = os.path.join(self.temp, "LIB")
        os.makedirs(projets)
        if self.conf["lib"]:
            # une copie de la LIB que l'outil prendrait : le banc peut y écrire sans risque
            source = self.lib_reelle()
            if source:
                shutil.copytree(source, lib, ignore=shutil.ignore_patterns(".git"))
            else:
                os.makedirs(lib)
        self.port = port_libre()
        args = [a.format(projets=projets, lib=lib) for a in self.conf["args"]]
        env = dict(os.environ, WEB_TOOLS_HTTP="1", PYTHONUNBUFFERED="1", PYTHONIOENCODING="utf-8")
        cmd = [sys.executable, self.conf["script"], "--port", str(self.port)] + args
        self.processus = subprocess.Popen(cmd, cwd=self.dossier, env=env, stdin=subprocess.DEVNULL,
                                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        threading.Thread(target=self._lire, args=(self.processus,), daemon=True).start()
        for _ in range(300):
            if self.processus.poll() is not None:
                raise RuntimeError("%s s'est arrêté au démarrage :\n%s" % (self.nom, "\n".join(self.journal[-30:])))
            try:
                urllib.request.urlopen("http://127.0.0.1:%d/" % self.port, timeout=1).close()
                return
            except OSError:
                time.sleep(0.2)
        raise RuntimeError("%s ne répond pas sur le port %d" % (self.nom, self.port))

    def _lire(self, processus):
        for brut in processus.stdout:
            ligne = brut.decode("utf-8", "replace").rstrip("\r\n")
            with self.verrou:
                self.journal.append(ligne)
                trop = len(self.journal) - MAX_JOURNAL
                if trop > 0:
                    del self.journal[:trop]
                    self.debut_journal += trop

    def noter(self, ligne):
        with self.verrou:
            self.journal.append(ligne)

    def lignes_depuis(self, n):
        with self.verrou:
            fin = self.debut_journal + len(self.journal)
            n = max(n, self.debut_journal)
            return self.journal[n - self.debut_journal:], fin

    def en_marche(self):
        return bool(self.processus and self.processus.poll() is None)

    def redemarrer(self):
        self.arreter()
        with self.verrou:
            self.journal, self.debut_journal = [], 0
        self.demarrer()

    def arreter(self):
        if self.processus and self.processus.poll() is None:
            self.processus.terminate()
            try:
                self.processus.wait(5)
            except subprocess.TimeoutExpired:
                self.processus.kill()
        if self.temp:
            shutil.rmtree(self.temp, ignore_errors=True)
            self.temp = None


def version_git(dossier):
    """« branche · commit · date · message » du dépôt de l'outil : le rapport dit quel code a été essayé."""
    try:
        sortie = subprocess.run(["git", "-C", dossier, "log", "-1", "--format=%h · %ci · %s"],
                                capture_output=True, text=True, encoding="utf-8", timeout=10)
        branche = subprocess.run(["git", "-C", dossier, "rev-parse", "--abbrev-ref", "HEAD"],
                                 capture_output=True, text=True, encoding="utf-8", timeout=10)
        modifie = subprocess.run(["git", "-C", dossier, "status", "--porcelain"],
                                 capture_output=True, text=True, encoding="utf-8", timeout=10)
        if sortie.returncode == 0:
            return "%s · %s%s" % (branche.stdout.strip(), sortie.stdout.strip(),
                                  " · modifications locales" if modifie.stdout.strip() else "")
    except (OSError, subprocess.SubprocessError):
        pass
    return "version inconnue (pas de git)"


def trouver_outils(racine):
    """Les outils installés à la racine de web_suite (un dossier par outil, comme installer.py)."""
    trouves = {}
    for conf in CATALOGUE:
        dossier = os.path.join(racine, conf["nom"])
        if os.path.isfile(os.path.join(dossier, conf["script"])):
            trouves[conf["id"]] = Outil(conf, os.path.abspath(dossier))
    return trouves


# --------------------------------------------------------------------------------------
# Les bancs d'un outil : scripts banc-*.py / .js / .mjs, lancés un par un
# --------------------------------------------------------------------------------------
class Bancs:
    def __init__(self, dossier):
        self.dossier = dossier
        self.verrou = threading.Lock()
        self.liste = self.decouvrir()
        self.file = []
        self.courant = None        # processus en cours
        self.arret = False
        self.fil = None

    def decouvrir(self):
        """Les scripts banc-* de l'outil. Ceux que sa CI lance (.github/workflows/ci.yml) sont
        marqués : ils tournent d'habitude seuls. Les autres demandent parfois une préparation
        (assemblage, LIB privée, dossier PROJETS)."""
        try:
            with open(os.path.join(self.dossier, ".github", "workflows", "ci.yml"), encoding="utf-8") as fh:
                ci = fh.read()
        except OSError:
            ci = ""
        trouves = []
        for racine, dossiers, fichiers in os.walk(self.dossier):
            dossiers[:] = sorted(d for d in dossiers if d not in DOSSIERS_IGNORES)
            for f in sorted(fichiers):
                if BANC.match(f):
                    rel = os.path.relpath(os.path.join(racine, f), self.dossier).replace(os.sep, "/")
                    trouves.append({"id": rel, "ci": rel in ci, "debut": None, "etat": "attente",
                                    "duree": None, "sortie": "", "code": None})
        trouves.sort(key=lambda b: (not b["ci"], b["id"]))
        return trouves

    def etat(self):
        with self.verrou:
            return {"en_cours": bool(self.fil and self.fil.is_alive()),
                    "bancs": [{k: b[k] for k in ("id", "ci", "etat", "duree", "code", "debut")} for b in self.liste]}

    def sortie(self, id_):
        with self.verrou:
            for b in self.liste:
                if b["id"] == id_:
                    return b["sortie"]
        return None

    def lancer(self, ids):
        with self.verrou:
            if self.fil and self.fil.is_alive():
                return False
            connus = {b["id"]: b for b in self.liste}
            self.file = [connus[i] for i in ids if i in connus]
            for b in self.file:
                b.update(etat="prevu", duree=None, sortie="", code=None, debut=None)
            self.arret = False
            self.fil = threading.Thread(target=self._derouler, daemon=True)
            self.fil.start()
            return True

    def arreter(self):
        self.arret = True
        p = self.courant
        if p and p.poll() is None:
            p.kill()

    def _commande(self, banc):
        ext = banc["id"].rsplit(".", 1)[-1]
        if ext == "py":
            return [sys.executable, banc["id"]]
        node = shutil.which("node")
        return [node, banc["id"]] if node else None

    def _derouler(self):
        chromium = trouver_chromium()
        env = dict(os.environ, PYTHONIOENCODING="utf-8", WEB_TOOLS_HTTP="1")
        if chromium:
            env["CHROMIUM"] = chromium
        for banc in list(self.file):
            if self.arret:
                with self.verrou:
                    banc["etat"] = "attente"
                continue
            cmd = self._commande(banc)
            if not cmd:
                with self.verrou:
                    banc.update(etat="ignore", sortie="node est introuvable : ce banc JavaScript ne peut pas tourner.")
                continue
            with self.verrou:
                banc["etat"] = "en_cours"
                banc["debut"] = debut = time.time()
            try:
                self.courant = subprocess.Popen(cmd, cwd=self.dossier, env=env, stdin=subprocess.DEVNULL,
                                                stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
                try:
                    sortie, _ = self.courant.communicate(timeout=DELAI_BANC)
                    code = self.courant.returncode
                    etat = "ok" if code == 0 else ("arrete" if self.arret else "echec")
                except subprocess.TimeoutExpired:
                    self.courant.kill()
                    sortie, _ = self.courant.communicate()
                    code, etat = None, "delai"
                texte = sortie.decode("utf-8", "replace")
            except OSError as exc:
                code, etat, texte = None, "echec", "Lancement impossible : %s" % exc
            with self.verrou:
                banc.update(etat=etat, code=code, duree=round(time.time() - debut, 1),
                            sortie=texte[-MAX_SORTIE:])
        self.courant = None


# --------------------------------------------------------------------------------------
# Rapports du clic-partout
# --------------------------------------------------------------------------------------
def lister_rapports():
    if not os.path.isdir(DOSSIER_RAPPORTS):
        return []
    rendu = []
    for f in sorted(os.listdir(DOSSIER_RAPPORTS), reverse=True):
        if f.endswith(".json"):
            try:
                with open(os.path.join(DOSSIER_RAPPORTS, f), encoding="utf-8") as fh:
                    r = json.load(fh)
                rendu.append({"nom": f, "date": r.get("date"), "outil": r.get("outil"), "bilan": r.get("bilan"),
                              "md": os.path.isfile(os.path.join(DOSSIER_RAPPORTS, f[:-5] + ".md"))})
            except (OSError, ValueError):
                continue
    return rendu


def enregistrer_rapport(rapport):
    """Le rapport complet en .json, et sa version lisible en .md à côté : celle-là se donne
    telle quelle à Claude pour les corrections."""
    os.makedirs(DOSSIER_RAPPORTS, exist_ok=True)
    outil = re.sub(r"[^A-Za-z0-9_]", "", str(rapport.get("outil") or "outil"))[:20]
    nom = datetime.datetime.now().strftime("%Y-%m-%d_%H%M%S") + "_" + outil + ".json"
    markdown = rapport.pop("markdown", None)
    with open(os.path.join(DOSSIER_RAPPORTS, nom), "w", encoding="utf-8") as fh:
        json.dump(rapport, fh, ensure_ascii=False, indent=1)
    if markdown:
        with open(os.path.join(DOSSIER_RAPPORTS, nom[:-5] + ".md"), "w", encoding="utf-8", newline="\n") as fh:
            fh.write(markdown)
    return nom


def pages_de(dossier):
    """Les pages de l'outil à essayer : l'accueil, les autres pages à la racine, et chaque
    <dossier>/<dossier>.html (les sous-outils de WEB_CAO)."""
    pages = []
    racine = sorted(f for f in os.listdir(dossier) if f.endswith(".html"))
    if "index.html" in racine:
        racine.remove("index.html")
        pages.append({"url": "/index.html", "nom": "Accueil"})
    pages += [{"url": "/" + f, "nom": f[:-5]} for f in racine]
    for d in sorted(os.listdir(dossier)):
        if d in DOSSIERS_IGNORES or d.startswith("."):
            continue
        if os.path.isfile(os.path.join(dossier, d, d + ".html")):
            pages.append({"url": "/%s/%s.html" % (d, d), "nom": d})
    return pages


# --------------------------------------------------------------------------------------
# Serveur : page Banc sous /__banc/, tout le reste relayé à l'outil actif
# --------------------------------------------------------------------------------------
TYPES = {".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8",
         ".css": "text/css; charset=utf-8", ".json": "application/json", ".svg": "image/svg+xml"}
SONDE = b'<script src="' + PREFIXE.encode() + b'/sonde.js"></script>'


class Banc:
    """L'état partagé du serveur : les outils, leurs bancs, l'outil affiché."""

    def __init__(self, outils, actif):
        self.outils = outils
        self.bancs = {i: Bancs(o.dossier) for i, o in outils.items()}
        self.actif = actif

    def outil(self):
        return self.outils[self.actif]

    def arreter(self):
        for b in self.bancs.values():
            b.arreter()
        for o in self.outils.values():
            o.arreter()


class Gestionnaire(http.server.BaseHTTPRequestHandler):
    banc = None

    def log_message(self, *_):
        pass

    # ---- garde : seulement la boucle locale, sous un nom de la boucle locale
    def _hote_local(self):
        hote = (self.headers.get("Host") or "").lower().rsplit(":", 1)[0].strip("[]")
        return hote in ("127.0.0.1", "localhost", "::1")

    def _traiter(self):
        if not self._hote_local():
            return self._envoyer(403, b"Host refuse", "text/plain; charset=utf-8")
        chemin = urllib.parse.urlsplit(self.path).path
        if chemin == PREFIXE or chemin.startswith(PREFIXE + "/"):
            return self._page_banc(chemin[len(PREFIXE):] or "/")
        # tout le reste, « / » compris, est à l'outil actif : ses pages y renvoient
        return self._relayer()

    do_GET = do_POST = do_PUT = do_DELETE = do_PATCH = do_HEAD = do_OPTIONS = _traiter

    def _envoyer(self, code, corps, type_):
        self.send_response(code)
        self.send_header("Content-Type", type_)
        self.send_header("Content-Length", str(len(corps)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(corps)

    def _json(self, obj, code=200):
        self._envoyer(code, json.dumps(obj, ensure_ascii=False).encode("utf-8"), "application/json")

    def _corps_json(self):
        n = int(self.headers.get("Content-Length") or 0)
        if n > 20 * 1024 * 1024:
            raise ValueError("corps trop gros")
        return json.loads(self.rfile.read(n) or b"{}")

    # ---- la page Banc et son API
    def _page_banc(self, chemin):
        if chemin.startswith("/api/"):
            if self.command == "POST" and self.headers.get(ENTETE_BANC) != "1":
                return self._json({"erreur": "en-tête %s manquant" % ENTETE_BANC}, 403)
            return self._api(chemin[5:])
        fichier = "index.html" if chemin == "/" else chemin.lstrip("/")
        cible = os.path.realpath(os.path.join(DOSSIER_PAGE, fichier))
        if not cible.startswith(os.path.realpath(DOSSIER_PAGE) + os.sep) or not os.path.isfile(cible):
            return self._envoyer(404, b"introuvable", "text/plain; charset=utf-8")
        with open(cible, "rb") as fh:
            corps = fh.read()
        self._envoyer(200, corps, TYPES.get(os.path.splitext(cible)[1], "application/octet-stream"))

    def _bancs_de(self, params, corps=None):
        id_ = (corps or {}).get("outil") or params.get("outil") or self.banc.actif
        return self.banc.bancs.get(id_)

    def _api(self, route):
        params = dict(urllib.parse.parse_qsl(urllib.parse.urlsplit(self.path).query))
        banc = self.banc
        o = banc.outil()
        if route == "etat":
            return self._json({
                "actif": banc.actif, "chromium": trouver_chromium(), "node": shutil.which("node"),
                "outils": [{"id": c["id"], "nom": c["nom"], "installe": c["id"] in banc.outils,
                            "en_marche": c["id"] in banc.outils and banc.outils[c["id"]].en_marche(),
                            "port": banc.outils[c["id"]].port if c["id"] in banc.outils else None,
                            "dossier": banc.outils[c["id"]].dossier if c["id"] in banc.outils else None,
                            "version": banc.outils[c["id"]].version if c["id"] in banc.outils else None,
                            "noire": c["noire"]} for c in CATALOGUE]})
        if route == "outil/choisir" and self.command == "POST":
            id_ = self._corps_json().get("id")
            if id_ not in banc.outils:
                return self._json({"erreur": "outil non installé : %s" % id_}, 404)
            try:
                banc.outils[id_].demarrer()
            except RuntimeError as exc:
                return self._json({"erreur": str(exc)}, 500)
            banc.actif = id_
            return self._json({"ok": True, "actif": id_})
        if route == "outil/redemarrer" and self.command == "POST":
            try:
                o.redemarrer()
            except RuntimeError as exc:
                return self._json({"erreur": str(exc)}, 500)
            return self._json({"ok": True, "port": o.port})
        if route == "pages":
            return self._json({"outil": banc.actif, "pages": pages_de(o.dossier)})
        if route == "journal":
            lignes, fin = o.lignes_depuis(int(params.get("depuis", 0)))
            return self._json({"outil": banc.actif, "lignes": lignes, "fin": fin})
        if route == "bancs":
            b = self._bancs_de(params)
            return self._json(b.etat()) if b else self._json({"erreur": "outil inconnu"}, 404)
        if route == "bancs/sortie":
            b = self._bancs_de(params)
            s = b.sortie(params.get("id", "")) if b else None
            return self._json({"sortie": s}) if s is not None else self._json({"erreur": "banc inconnu"}, 404)
        if route == "bancs/lancer" and self.command == "POST":
            corps = self._corps_json()
            b = self._bancs_de(params, corps)
            if not b:
                return self._json({"erreur": "outil inconnu"}, 404)
            ids = corps.get("ids") or [x["id"] for x in b.liste if x["ci"]]
            return self._json({"ok": b.lancer(ids)})
        if route == "bancs/arreter" and self.command == "POST":
            b = self._bancs_de(params, self._corps_json())
            if b:
                b.arreter()
            return self._json({"ok": True})
        if route == "rapports" and self.command == "GET":
            return self._json({"rapports": lister_rapports()})
        if route == "rapports" and self.command == "POST":
            return self._json({"nom": enregistrer_rapport(self._corps_json())})
        if route == "rapport.md":
            nom = os.path.basename(params.get("nom", ""))
            chemin = os.path.join(DOSSIER_RAPPORTS, nom[:-5] + ".md") if nom.endswith(".json") else ""
            if not chemin or not os.path.isfile(chemin):
                return self._envoyer(404, b"pas de .md pour ce rapport", "text/plain; charset=utf-8")
            with open(chemin, "rb") as fh:
                return self._envoyer(200, fh.read(), "text/markdown; charset=utf-8")
        if route == "rapport":
            nom = os.path.basename(params.get("nom", ""))
            chemin = os.path.join(DOSSIER_RAPPORTS, nom)
            if not nom.endswith(".json") or not os.path.isfile(chemin):
                return self._json({"erreur": "rapport inconnu"}, 404)
            with open(chemin, encoding="utf-8") as fh:
                return self._json(json.load(fh))
        return self._json({"erreur": "route inconnue"}, 404)

    # ---- le relais vers l'outil actif
    def _relayer(self):
        n = int(self.headers.get("Content-Length") or 0)
        corps = self.rfile.read(n) if n else None
        entetes = {k: v for k, v in self.headers.items() if k.lower() not in SAUTS}
        o = self.banc.outil()
        chemin = urllib.parse.urlsplit(self.path).path
        motif = ROUTES_BLOQUEES + ("|" + o.conf["routes"] if o.conf.get("routes") else "")
        if self.command != "GET" and re.search(motif, chemin):
            o.noter("[web_bench] requête arrêtée par le banc : %s %s" % (self.command, chemin))
            corps = json.dumps({"detail": "Arrêté par web_bench (route protégée pendant les essais)"}).encode("utf-8")
            self.send_response(403)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(corps)))
            self.send_header("X-Web-Bench-Bloque", "1")
            self.end_headers()
            self.wfile.write(corps)
            return
        try:
            conn = http.client.HTTPConnection("127.0.0.1", o.port, timeout=3600)
            conn.request(self.command, self.path, body=corps, headers=entetes)
            rep = conn.getresponse()
        except OSError as exc:
            return self._envoyer(502, ("%s injoignable : %s" % (o.nom, exc)).encode("utf-8"),
                                 "text/plain; charset=utf-8")
        type_ = rep.getheader("Content-Type") or ""
        try:
            if type_.startswith("text/html") and self.command == "GET":
                page = injecter_sonde(rep.read())
                self.send_response(rep.status, rep.reason)
                for k, v in rep.getheaders():
                    if k.lower() not in SAUTS:
                        self.send_header(k, v)
                self.send_header("Content-Length", str(len(page)))
                self.end_headers()
                self.wfile.write(page)
                return
            self.send_response(rep.status, rep.reason)
            for k, v in rep.getheaders():
                if k.lower() not in SAUTS:
                    self.send_header(k, v)
            longueur = rep.getheader("Content-Length")
            if longueur is not None and not rep.getheader("Transfer-Encoding"):
                self.send_header("Content-Length", longueur)
            else:
                self.close_connection = True
            self.end_headers()
            if self.command == "HEAD":
                return
            while True:
                bloc = rep.read1(65536)
                if not bloc:
                    break
                self.wfile.write(bloc)
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            pass
        finally:
            conn.close()


def injecter_sonde(page):
    """La sonde en tout premier dans <head> : elle voit les erreurs dès le premier script."""
    m = re.search(rb"<head[^>]*>", page, re.I)
    if m:
        return page[:m.end()] + SONDE + page[m.end():]
    return SONDE + page


class Serveur(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
    allow_reuse_address = True


def main(argv=None):
    ap = argparse.ArgumentParser(description="web_bench : banc d'essai visible des outils de web_suite")
    ap.add_argument("--racine", default=RACINE_DEFAUT,
                    help="dossier où vivent WEB_CAO, WEB_ANTENNA, WEB_3D (défaut : la racine de web_suite)")
    ap.add_argument("--outil", choices=[c["id"] for c in CATALOGUE], help="outil ouvert au démarrage")
    ap.add_argument("--port", type=int, default=8200, help="port de la page Banc (défaut 8200)")
    ap.add_argument("--sans-navigateur", action="store_true", help="ne pas ouvrir le navigateur")
    args = ap.parse_args(argv)

    outils = trouver_outils(args.racine)
    if not outils:
        print("[!] Aucun outil trouvé dans %s. Installe-les avec installer.cmd / installer.sh,"
              " ou indique leur dossier : --racine CHEMIN" % args.racine)
        return 2
    actif = args.outil if args.outil in outils else next(iter(outils))
    print("[*] Outils trouvés : %s" % ", ".join(o.nom for o in outils.values()))
    print("[*] Démarrage de %s (dossiers temporaires)..." % outils[actif].nom)
    try:
        outils[actif].demarrer()
    except RuntimeError as exc:
        print("[!]", exc)
        outils[actif].arreter()
        return 1

    # fermé autrement que par Ctrl+C (kill, fermeture du terminal) : on arrête quand même
    # les outils et on efface leurs dossiers temporaires
    def interrompre(*_):
        raise KeyboardInterrupt
    for nom in ("SIGTERM", "SIGHUP", "SIGBREAK"):
        if hasattr(signal, nom):
            try:
                signal.signal(getattr(signal, nom), interrompre)
            except (ValueError, OSError):
                pass

    banc = Banc(outils, actif)
    Gestionnaire.banc = banc
    try:
        serveur = Serveur(("127.0.0.1", args.port), Gestionnaire)
    except OSError:
        serveur = Serveur(("127.0.0.1", 0), Gestionnaire)
    adresse = "http://127.0.0.1:%d%s/" % (serveur.server_address[1], PREFIXE)
    print("[*] Bancs trouvés : %s" % ", ".join("%s %d" % (outils[i].nom, len(b.liste)) for i, b in banc.bancs.items()))
    print("[*] Page Banc : %s   (Ctrl+C pour arrêter)" % adresse)
    if not args.sans_navigateur:
        threading.Timer(0.5, webbrowser.open, (adresse,)).start()
    try:
        serveur.serve_forever()
    except KeyboardInterrupt:
        print("\n[*] Arrêt.")
    finally:
        banc.arreter()
        serveur.server_close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
