"""WEB·SUITE — lanceur des outils WEB_CAO, WEB_ANTENNA et WEB_3D.

    python lanceur/web_suite.py              ouvre le lanceur dans le navigateur
                                             (sauf sur Raspberry Pi et Termux : adresse affichée en console)
    python lanceur/web_suite.py --installer  menu de téléchargement des outils seulement

Le lanceur joue l'animation d'intro, puis propose les trois outils : il les
télécharge s'ils manquent (git clone à la racine du dépôt), démarre leur serveur
et ouvre leur page. Fermer cette fenêtre arrête les outils lancés depuis elle.

Bibliothèque standard seule. Par défaut le serveur n'écoute que sur 127.0.0.1 :
il lance des programmes. Avec --reseau (défaut sur Raspberry Pi et Termux), le lanceur et
les outils écoutent sur le réseau local ; le lanceur exige alors des autres
appareils un jeton, qu'ils reçoivent en saisissant le code d'appairage affiché en
console (ou avec l'adresse « Réseau » complète, ?jeton=...). Les outils, eux,
restent sans authentification : à réserver à un réseau de confiance. --local
force l'écoute locale.
"""

import argparse
import hmac
import ipaddress
import secrets
import socket
import json
import os
import subprocess
import sys
import threading
import time
import webbrowser
from http import HTTPStatus
from http.cookies import CookieError, SimpleCookie
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

import connexion_github
import installer
import outils
import projets

ICI = outils.ICI
DEPOT = os.path.dirname(ICI)                           # racine du dépôt git WEB_SUITE
PORT_DEFAUT = 8100
FICHIERS_SERVIS = {"/": "index.html", "/index.html": "index.html", "/websuite-intro.html": "websuite-intro.html"}
CORPS_MAX = 64 * 1024                                  # les actions ne portent qu'un id et un message
FICHIER_JETON = os.path.join(ICI, "jeton-reseau.txt")  # ignoré par git
BISCUIT = "websuite_jeton"
# Pages ouvertes à un appareil qui n'a pas encore le jeton : l'accueil (qui le
# renvoie vers la page du code), la page du code, et la route qui le vérifie.
# La page du code a SA propre adresse : servie à « / » comme le lanceur, Safari
# la ressortait de son cache après un bon code, au lieu du lanceur.
PAGE_APPAIRAGE = "/appairage"
PAGES_APPAIRAGE = ("/", "/index.html", PAGE_APPAIRAGE)


def hote_permis(entete):
    """L'en-tête Host désigne-t-il ce poste ? (parade au DNS rebinding)

    Une page piégée qui fait résoudre son nom vers 127.0.0.1 devient « de même
    origine », ajoute X-WebSuite sans pré-vol et pilote le lanceur ; elle envoie
    alors SON nom dans Host. Une IP littérale ne se rebranche pas : on accepte
    toutes les IP (la tablette tape celle du poste), localhost et le nom du poste.
    """
    h = (entete or "").strip().lower().rstrip(".")
    if h.startswith("["):
        h = h[1:].split("]")[0]
    elif h.count(":") == 1:
        h = h.split(":")[0]
    nom = socket.gethostname().lower()
    if h in ("", "localhost", nom, nom + ".local"):
        return True
    try:
        ipaddress.ip_address(h.split("%")[0])
        return True
    except ValueError:
        return False


class Appairage:
    """Code court affiché dans le terminal, à saisir sur la tablette.

    Taper le jeton (22 caractères) à chaque changement d'adresse du téléphone
    était pénible : l'iPad ne le garde que pour l'adresse où il l'a reçu. Le
    code à 6 chiffres ne remplace pas le jeton, il le DONNE : bon code, la
    tablette reçoit le cookie du jeton, comme avec l'adresse ?jeton=...
    Seul un appareil qui voit l'écran du serveur peut donc entrer.

    Deviner : un code ne vit que DUREE secondes, sert une fois, et ESSAIS
    erreurs le brûlent, avec PAUSE secondes avant le suivant. Chaque nouveau
    code s'affiche dans le terminal : une série d'essais ne passe pas inaperçue.
    """
    DUREE, ESSAIS, PAUSE = 600, 5, 60

    def __init__(self, afficher=None):
        self.afficher = afficher or (lambda texte: print(texte, flush=True))
        self.verrou = threading.Lock()
        self.code = None
        self.expire = 0.0
        self.echecs = 0
        self.bloque = 0.0

    @staticmethod
    def lisible(code):
        return code[:3] + " " + code[3:]

    def _neuf(self, maintenant):
        self.code = "%06d" % secrets.randbelow(10 ** 6)
        self.expire = maintenant + self.DUREE
        self.echecs = 0
        self.afficher("  Code d'appairage : %s   (à saisir sur la tablette, valable %d min)"
                      % (self.lisible(self.code), self.DUREE // 60))

    def preparer(self):
        """Un code valable existe (en créer un, et l'afficher, s'il le faut).
        Renvoie les secondes d'attente si les essais sont suspendus, sinon 0."""
        with self.verrou:
            maintenant = time.monotonic()
            if maintenant < self.bloque:
                return int(self.bloque - maintenant) + 1
            if not self.code or maintenant >= self.expire:
                self._neuf(maintenant)
            return 0

    def essayer(self, saisi):
        """Renvoie (ok, message)."""
        saisi = "".join(c for c in str(saisi or "") if c.isdigit())
        with self.verrou:
            maintenant = time.monotonic()
            if maintenant < self.bloque:
                return False, ("Trop d'essais : attendez %d s, un nouveau code s'affichera "
                               "sur le serveur." % (int(self.bloque - maintenant) + 1))
            if not self.code or maintenant >= self.expire:
                self._neuf(maintenant)
                return False, "Code expiré : un nouveau code est affiché sur le serveur."
            if len(saisi) == 6 and hmac.compare_digest(saisi, self.code):
                self.code = None                   # usage unique
                return True, "Appareil autorisé."
            self.echecs += 1
            if self.echecs >= self.ESSAIS:
                self.code = None
                self.bloque = maintenant + self.PAUSE
                return False, ("Trop d'essais : code annulé. Dans %d s, rechargez la page : "
                               "un nouveau code s'affichera sur le serveur." % self.PAUSE)
            return False, "Code incorrect (%d essai(s) restant(s))." % (self.ESSAIS - self.echecs)


def jeton_reseau():
    """Le jeton du mode réseau, gardé d'un démarrage à l'autre : les favoris
    de la tablette restent bons. Supprimer le fichier en crée un neuf."""
    try:
        with open(FICHIER_JETON, encoding="utf-8") as f:
            jeton = f.read().strip()
        if len(jeton) >= 16:
            return jeton
    except OSError:
        pass
    jeton = secrets.token_urlsafe(16)
    fd = os.open(FICHIER_JETON, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as f:
        f.write(jeton + "\n")
    return jeton


# ---------------------------------------------------------------------------
# Mise à jour de WEB_SUITE elle-même, comme les outils le font au démarrage
# ---------------------------------------------------------------------------
# Attente des vérifications des outils avant d'afficher leur bilan en console.
# Au-delà, le bilan d'un outil s'affiche plus bas, dès qu'il arrive.
DELAI_BILAN_MAJ = 30


def verifier_et_appliquer_maj():
    """-> (True si WEB_SUITE vient d'être mis à jour et doit redémarrer, dernier message)."""
    print("  Recherche de mises à jour de WEB_SUITE...", flush=True)
    dernier = [""]

    def ecrire(texte):
        dernier[0] = texte
        print("    " + texte, flush=True)
    statut = outils.maj_depot(DEPOT, ecrire)
    texte = dernier[0] or outils.BILAN_SANS_TEXTE.get(statut, statut)
    if statut != "maj":
        return False, texte
    print("  [*] WEB_SUITE mis à jour, redémarrage.", flush=True)
    return True, texte


def bilans_outils(gestionnaire, racine, verifie):
    """-> ([(outil, état pour le bandeau)], [outils dont la vérification tourne encore])."""
    installes = [o for o in outils.CATALOGUE if outils.est_installe(o, racine)]
    if verifie and any(not gestionnaire.maj_finie[o["id"]].is_set() for o in installes):
        print("  Recherche de mises à jour des outils...", flush=True)
    fin = time.time() + DELAI_BILAN_MAJ
    etats, en_retard = [], []
    for o in outils.CATALOGUE:
        if o not in installes:
            etats.append((o, "non installé"))
            continue
        bilan = gestionnaire.bilan_maj(o["id"], max(0.0, fin - time.time())) if verifie else ""
        if bilan is None:
            en_retard.append(o)
            bilan = "vérification des mises à jour en cours..."
        etats.append((o, "installé · " + bilan if bilan else "installé"))
    return etats, en_retard


def annoncer_plus_tard(gestionnaire, en_retard):
    """Affiche le bilan des vérifications qui ont dépassé DELAI_BILAN_MAJ."""
    def tache():
        for o in en_retard:
            bilan = gestionnaire.bilan_maj(o["id"], 120)
            print("  Mise à jour %s : %s" % (o["nom"], bilan or "toujours sans réponse."),
                  flush=True)
    threading.Thread(target=tache, daemon=True).start()


def redemarrer(bilan):
    # Chemin absolu reconstruit : sous Python 3.8, __file__ peut être relatif
    # et le dossier courant a déjà changé (os.chdir(ICI)).
    # Le bilan passe au nouveau lanceur, qui l'affiche avec celui des outils.
    env = dict(os.environ, WEB_SUITE_DEJA_MAJ="1", WEB_SUITE_BILAN_MAJ=bilan)
    script = os.path.join(ICI, "web_suite.py")
    cmd = [sys.executable, script] + sys.argv[1:]
    sys.stdout.flush()
    if os.name != "nt":
        # Linux, Pi, Termux : le nouveau lanceur REMPLACE celui-ci, comme le
        # fait WEB_CAO. Un parent qui attendait recevait lui aussi Ctrl+C et
        # l'affichait en trace d'erreur sous le « Arrêt. » de l'enfant ; et un
        # kill visait le parent en laissant l'enfant tourner.
        os.execve(sys.executable, cmd, env)
    proc = subprocess.Popen(cmd, env=env)
    while True:
        try:
            code = proc.wait()
            break
        except KeyboardInterrupt:
            # Ctrl+C atteint aussi l'enfant, qui arrête ses outils et sort
            # proprement : on l'attend, sans trace d'erreur.
            continue
    os._exit(code)


# ---------------------------------------------------------------------------
# Serveur HTTP
# ---------------------------------------------------------------------------
class Lanceur(SimpleHTTPRequestHandler):
    gestionnaire = None                                # outils.Gestionnaire
    jeton = None                                       # en --reseau : exigé des autres appareils
    appairage = None                                   # en --reseau : le code court (Appairage)
    autorise = True                                    # par requête, fixé dans parse_request

    def log_message(self, fmt, *args):                 # la console reste lisible
        pass

    def parse_request(self):
        if not super().parse_request():
            return False
        if not hote_permis(self.headers.get("Host")):
            self.send_error(HTTPStatus.FORBIDDEN, "Host non autorise (protection DNS rebinding)")
            return False
        self.autorise = not (self.jeton and not self._de_ce_poste() and not self._jeton_recu())
        chemin = urlparse(self.path).path
        if not self.autorise and self.appairage and (
                (self.command == "GET" and chemin in PAGES_APPAIRAGE)
                or (self.command == "POST" and chemin == "/api/appairer")):
            return True                                # la page du code, et sa vérification
        if not self.autorise:
            # Sans cela, tout appareil du réseau poussait sur GitHub avec les
            # identifiants de ce poste et changeait son identité git.
            self.send_error(HTTPStatus.FORBIDDEN, "Jeton manquant",
                            "Ouvrir l'adresse Reseau affichee dans la console du lanceur "
                            "(elle se termine par ?jeton=...).")
            return False
        return True

    def _de_ce_poste(self):
        try:
            ip = ipaddress.ip_address(self.client_address[0].split("%")[0])
        except ValueError:
            return False
        return (getattr(ip, "ipv4_mapped", None) or ip).is_loopback

    def _jeton_url(self):
        return parse_qs(urlparse(self.path).query).get("jeton", [""])[0]

    def _jeton_recu(self):
        try:
            biscuit = SimpleCookie(self.headers.get("Cookie") or "")
        except CookieError:
            biscuit = {}
        recus = [self._jeton_url(), biscuit[BISCUIT].value if BISCUIT in biscuit else ""]
        return any(r and hmac.compare_digest(r, self.jeton) for r in recus)

    def _json(self, donnees, statut=HTTPStatus.OK):
        corps = json.dumps(donnees, ensure_ascii=False).encode("utf-8")
        self.send_response(statut)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(corps)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(corps)

    def _fichier(self, nom):
        chemin = os.path.join(ICI, nom)
        with open(chemin, "rb") as f:
            corps = f.read()
        self.send_response(HTTPStatus.OK)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(corps)))
        # no-store : ces pages dépendent de l'autorisation de l'appareil, une
        # copie gardée par le navigateur montrerait la mauvaise
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(corps)

    def _rediriger(self, ou):
        self.send_response(HTTPStatus.SEE_OTHER)
        self.send_header("Location", ou)
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Length", "0")
        self.end_headers()

    def _biscuit_jeton(self):
        return ("%s=%s; Path=/; Max-Age=31536000; HttpOnly; SameSite=Strict"
                % (BISCUIT, self.jeton))

    def do_GET(self):
        url = urlparse(self.path)
        g = self.gestionnaire
        if not self.autorise:
            # Appareil sans jeton : la page qui demande le code d'appairage
            # (le code s'affiche, s'il le faut, dans le terminal du serveur).
            if url.path != PAGE_APPAIRAGE:
                return self._rediriger(PAGE_APPAIRAGE)
            self.appairage.preparer()
            return self._fichier("appairage.html")
        if url.path == PAGE_APPAIRAGE:
            return self._rediriger("/")                # déjà autorisé : au lanceur
        if self.jeton and self._jeton_url():
            # Le jeton passe dans un cookie et quitte l'adresse : il ne reste
            # ni dans l'historique, ni dans un lien recopié.
            self.send_response(HTTPStatus.SEE_OTHER)
            self.send_header("Set-Cookie", self._biscuit_jeton())
            self.send_header("Location", url.path or "/")
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        if url.path in FICHIERS_SERVIS:
            return self._fichier(FICHIERS_SERVIS[url.path])
        if url.path == "/api/etat":
            return self._json({"racine": g.racine, "git": outils.git_disponible(),
                               "projets": projets.dossier(g.racine), "outils": g.etat(),
                               "envoi_auto": getattr(g, "envoi_auto", None)})
        if url.path == "/api/journal":
            oid = parse_qs(url.query).get("id", [""])[0]
            if oid not in outils.PAR_ID:
                return self._json({"erreur": "outil inconnu"}, HTTPStatus.NOT_FOUND)
            try:
                with open(g.journal(oid), encoding="utf-8", errors="replace") as f:
                    texte = f.read()[-20000:]
            except OSError:
                texte = ""
            return self._json({"texte": texte})
        self.send_error(HTTPStatus.NOT_FOUND)

    def do_POST(self):
        # En-tête maison obligatoire : une autre page ouverte dans le navigateur
        # ne peut pas l'ajouter sans pré-vol CORS, que ce serveur ne valide pas.
        # Aucun site tiers ne peut donc lancer ou télécharger quoi que ce soit.
        if self.headers.get("X-WebSuite") != "1":
            return self._json({"erreur": "requête refusée"}, HTTPStatus.FORBIDDEN)
        url = urlparse(self.path)
        try:
            longueur = int(self.headers.get("Content-Length") or 0)
            if not 0 <= longueur <= CORPS_MAX:
                self.close_connection = True
                return self._json({"erreur": "requête trop grande"}, HTTPStatus.REQUEST_ENTITY_TOO_LARGE)
            corps = json.loads(self.rfile.read(longueur) or b"{}")
        except ValueError:
            return self._json({"erreur": "JSON invalide"}, HTTPStatus.BAD_REQUEST)
        if not isinstance(corps, dict):
            return self._json({"erreur": "JSON invalide"}, HTTPStatus.BAD_REQUEST)
        if not self.autorise:
            # Seule route ouverte sans jeton (parse_request) : le bon code
            # donne à l'appareil le cookie du jeton, rien de plus.
            ok, texte = self.appairage.essayer(corps.get("code"))
            donnees = json.dumps({"ok": ok, "message": texte}, ensure_ascii=False).encode("utf-8")
            self.send_response(HTTPStatus.OK if ok else HTTPStatus.FORBIDDEN)
            if ok:
                self.send_header("Set-Cookie", self._biscuit_jeton())
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(donnees)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(donnees)
            return
        if url.path == "/api/appairer":
            # Déjà autorisé (un premier envoi du code a réussi, la page a été
            # validée deux fois) : rien à vérifier, la page passe au lanceur.
            return self._json({"ok": True, "message": "Cet appareil est déjà autorisé."})
        oid = corps.get("id")
        g = self.gestionnaire
        if url.path == "/api/identite":
            ok, texte = projets.definir_identite(g.racine, corps.get("nom"), corps.get("email"))
            return self._json({"ok": ok, "message": texte})
        if url.path == "/api/envoyer":
            if oid not in outils.PAR_ID:
                return self._json({"erreur": "outil inconnu"}, HTTPStatus.BAD_REQUEST)
            if projets.identite_manquante(g.racine):
                # Poste neuf (Raspberry Pi...) : la page demande nom et e-mail, puis renvoie.
                return self._json({"ok": False, "identite": True, "outils": g.etat(),
                                   "message": "git ne connaît pas encore votre nom et e-mail."})
            ok, texte = g.envoyer(oid, str(corps.get("message") or ""))
            return self._json({"ok": ok, "message": texte, "outils": g.etat()})
        actions = {"/api/installer": g.installer, "/api/lancer": g.lancer,
                   "/api/arreter": g.arreter, "/api/openems": g.installer_openems}
        if url.path not in actions:
            return self._json({"erreur": "action inconnue"}, HTTPStatus.NOT_FOUND)
        ids = [o["id"] for o in outils.CATALOGUE] if oid == "tous" else [oid]
        if any(i not in outils.PAR_ID for i in ids):
            return self._json({"erreur": "outil inconnu"}, HTTPStatus.BAD_REQUEST)
        for i in ids:
            if url.path == "/api/installer" and outils.est_installe(outils.PAR_ID[i], g.racine):
                continue
            actions[url.path](i)
        return self._json({"ok": True, "outils": g.etat()})


class Serveur(ThreadingHTTPServer):
    def handle_error(self, request, client_address):
        # Le navigateur a fermé la connexion avant la réponse : rien à dire.
        if isinstance(sys.exc_info()[1], (ConnectionAbortedError, ConnectionResetError, BrokenPipeError)):
            return
        super().handle_error(request, client_address)


def ouvrir_serveur(port, hote="127.0.0.1"):
    for essai in range(port, port + 10):
        try:
            return Serveur((hote, essai), Lanceur)
        except OSError:
            continue
    raise OSError("aucun port libre entre %d et %d" % (port, port + 9))


def adresse_reseau():
    """L'adresse IPv4 du poste sur le réseau local (aucun paquet n'est envoyé)."""
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
            s.connect(("192.168.255.255", 1))
            ip = s.getsockname()[0]
            if not ip.startswith("127."):
                return ip
    except OSError:
        pass
    try:
        ip = socket.gethostbyname(socket.gethostname())
        if not ip.startswith("127."):
            return ip
    except OSError:
        pass
    return None


def est_raspberry_pi():
    """Vrai si la machine est un Raspberry Pi (modèle lu dans le device tree)."""
    for chemin in ("/proc/device-tree/model", "/sys/firmware/devicetree/base/model"):
        try:
            with open(chemin, "rb") as f:
                if b"raspberry pi" in f.read().lower():
                    return True
        except OSError:
            pass
    return False


def poste_serveur():
    """Vrai sur une machine qui sert les autres appareils (Raspberry Pi, téléphone
    Termux) : réseau local par défaut, et pas de navigateur ouvert sur place."""
    return outils.TERMUX or est_raspberry_pi()


def main(argv=None):
    installer._console_utf8()
    ap = argparse.ArgumentParser(description="Lanceur WEB·SUITE.")
    ap.add_argument("--port", type=int, default=PORT_DEFAUT,
                    help="port du lanceur (défaut : %d)" % PORT_DEFAUT)
    ap.add_argument("--racine", default=outils.RACINE_DEFAUT,
                    help="dossier contenant les outils (défaut : le dossier de WEB_SUITE)")
    ap.add_argument("--installer", action="store_true",
                    help="ouvrir seulement le menu de téléchargement des outils")
    ap.add_argument("--navigateur", action="store_true",
                    help="ouvrir le navigateur au démarrage même sur Raspberry Pi ou Termux")
    ap.add_argument("--sans-navigateur", action="store_true",
                    help="ne pas ouvrir le navigateur au démarrage")
    ecoute = ap.add_mutually_exclusive_group()
    ecoute.add_argument("--reseau", dest="reseau", action="store_true", default=None,
                        help="écouter sur le réseau local : ouvrir WEB·SUITE depuis un autre "
                             "appareil (défaut sur Raspberry Pi et Termux)")
    ecoute.add_argument("--local", dest="reseau", action="store_false",
                        help="n'écouter que sur 127.0.0.1 (défaut ailleurs)")
    ap.add_argument("--sans-maj", dest="verifier_maj", action="store_false", default=True,
                    help="ne pas vérifier les mises à jour GitHub au démarrage")
    args = ap.parse_args(argv)
    racine = os.path.abspath(args.racine)
    serveur_dedie = poste_serveur()
    reseau = serveur_dedie if args.reseau is None else args.reseau

    if args.installer:
        return installer.main(["--racine", racine])

    bilan_suite = os.environ.pop("WEB_SUITE_BILAN_MAJ", "") or (
        "mis à jour, redémarré." if os.environ.get("WEB_SUITE_DEJA_MAJ") == "1" else "")
    if not args.verifier_maj:
        bilan_suite = "mises à jour non vérifiées (--sans-maj)"
    elif os.environ.get("WEB_SUITE_DEJA_MAJ") != "1":
        a_relancer, bilan_suite = verifier_et_appliquer_maj()
        if a_relancer:
            redemarrer(bilan_suite)

    # Premier démarrage sans aucun outil : autant demander tout de suite, en
    # console, lequel télécharger. La page le propose aussi, pour les autres cas.
    aucun = not any(outils.est_installe(o, racine) for o in outils.CATALOGUE)
    if aucun and sys.stdin is not None and sys.stdin.isatty():
        print("\n  Aucun outil installé dans %s." % racine)
        installer.installer(installer.menu(racine), racine)

    if outils.git_disponible():
        print("  Projets : %s" % projets.preparer(racine), flush=True)
        if sys.stdin is not None and sys.stdin.isatty():
            # Téléphone, Pi : se connecter à GitHub ici, une fois, plutôt que de
            # l'apprendre à la tablette au premier envoi (connexion_github.py).
            connexion_github.proposer(racine)

    Lanceur.gestionnaire = outils.Gestionnaire(racine, reseau=reseau)
    Lanceur.jeton = jeton_reseau() if reseau else None
    Lanceur.appairage = Appairage() if reseau else None
    if args.verifier_maj:
        # Pendant l'animation d'intro : les outils lancés ensuite n'ont plus à le faire.
        Lanceur.gestionnaire.verifier_maj_outils()
    try:
        serveur = ouvrir_serveur(args.port, "0.0.0.0" if reseau else "127.0.0.1")
    except OSError as exc:
        print("  [X] %s" % exc)
        return 1
    url = "http://127.0.0.1:%d/" % serveur.server_address[1]
    etats_outils, en_retard = bilans_outils(Lanceur.gestionnaire, racine, args.verifier_maj)
    Lanceur.gestionnaire.lanceur = url.rstrip("/")
    Lanceur.gestionnaire.jeton = Lanceur.jeton
    if outils.git_disponible():
        # Ce qui a été enregistré hors ligne repart dès que GitHub répond.
        Lanceur.gestionnaire.demarrer_envoi_auto()

    print()
    print("  WEB·SUITE")
    print("  " + "-" * 48)
    print("  Adresse   %s" % url)
    ip = adresse_reseau() if reseau else None
    url_reseau = ("http://%s:%d/?jeton=%s" % (ip, serveur.server_address[1], Lanceur.jeton)
                  if ip else None)
    if url_reseau:
        print("  Réseau    http://%s:%d/" % (ip, serveur.server_address[1]))
        print("            tablette, téléphone, autre poste : ouvrir cette adresse,")
        print("            puis saisir le code d'appairage affiché ci-dessous")
        print("            (ou directement, sans code : %s ;" % url_reseau)
        print("             supprimer lanceur/jeton-reseau.txt pour changer de jeton)")
    elif reseau:
        print("  Réseau    écoute sur le réseau, adresse IP du poste introuvable")
        print("            jeton à ajouter à l'adresse : ?jeton=%s" % Lanceur.jeton)
    else:
        print("  Réseau    désactivé (--reseau pour y accéder depuis un autre appareil)")
    print("  Outils    %s" % racine)
    print("  Projets   %s" % projets.dossier(racine))
    print("            %-12s %s" % ("WEB_SUITE", bilan_suite))
    for o, etat in etats_outils:
        print("            %-12s %s" % (o["nom"], etat))
    print()
    print("  Fermer cette fenêtre (ou Ctrl+C) arrête aussi les outils lancés.")
    print()
    sys.stdout.flush()

    ouvrir = not args.sans_navigateur and (args.navigateur or not serveur_dedie)
    if not ouvrir:
        print("  Navigateur non ouvert : aller sur %s"
              % ("http://%s:%d/" % (ip, serveur.server_address[1]) if url_reseau else url))
        if Lanceur.appairage:
            # en dernier, là où l'œil tombe ; un nouveau code s'affiche ici aussi
            Lanceur.appairage.preparer()
        print()
        sys.stdout.flush()
    else:
        threading.Timer(0.6, lambda: webbrowser.open(url)).start()
        if Lanceur.appairage:
            Lanceur.appairage.preparer()
    if os.name != "nt":
        # Fermer le terminal (SIGHUP) ou un kill/systemctl stop (SIGTERM) doit
        # arrêter les outils comme Ctrl+C : ils sont dans leur propre session,
        # rien d'autre ne les arrêterait. Sous Windows, l'objet Job s'en charge.
        import signal

        signaux = (signal.SIGHUP, signal.SIGTERM)

        def arreter(*_):
            for sig in signaux:                        # un second signal n'interrompt pas l'arrêt
                signal.signal(sig, signal.SIG_IGN)
            raise KeyboardInterrupt
        for sig in signaux:
            signal.signal(sig, arreter)
    if en_retard:
        annoncer_plus_tard(Lanceur.gestionnaire, en_retard)
    try:
        serveur.serve_forever()
    except KeyboardInterrupt:
        print("\n  Arrêt.")
    finally:
        Lanceur.gestionnaire.arreter_tout()
        serveur.server_close()
    return 0


if __name__ == "__main__":
    os.chdir(ICI)
    raise SystemExit(main())
