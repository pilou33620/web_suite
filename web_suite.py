"""WEB·SUITE — lanceur des outils WEB_CAO, WEB_ANTENNA et WEB_3D.

    python web_suite.py                 ouvre le lanceur dans le navigateur
    python web_suite.py --installer     menu de téléchargement des outils seulement

Le lanceur joue l'animation d'intro, puis propose les trois outils : il les
télécharge s'ils manquent (git clone dans un sous-dossier), démarre leur serveur
et ouvre leur page. Fermer cette fenêtre arrête les outils lancés depuis elle.

Bibliothèque standard seule. Le serveur n'écoute que sur 127.0.0.1 : il lance
des programmes, il n'a rien à faire sur le réseau.
"""

import argparse
import json
import os
import subprocess
import sys
import threading
import webbrowser
from http import HTTPStatus
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

import installer
import outils

ICI = outils.ICI
PORT_DEFAUT = 8100
FICHIERS_SERVIS = {"/": "index.html", "/index.html": "index.html", "/websuite-intro.html": "websuite-intro.html"}


# ---------------------------------------------------------------------------
# Mise à jour de WEB_SUITE elle-même, comme les outils le font au démarrage
# ---------------------------------------------------------------------------
def _git(*args, timeout=10):
    return subprocess.run(["git", *args], cwd=ICI, capture_output=True, text=True,
                          timeout=timeout, creationflags=outils.SANS_FENETRE,
                          env=dict(os.environ, GIT_TERMINAL_PROMPT="0"))


def verifier_et_appliquer_maj():
    """git pull si le dépôt a une branche distante en avance et rien de modifié."""
    if not os.path.isdir(os.path.join(ICI, ".git")) or not outils.git_disponible():
        return False
    try:
        if _git("fetch", "--quiet", "origin").returncode != 0:
            return False
        if _git("rev-parse", "--verify", "@{u}").returncode != 0:
            return False                               # pas encore de dépôt distant suivi
        retard = _git("rev-list", "HEAD..@{u}", "--count").stdout.strip()
        if not retard.isdigit() or int(retard) == 0:
            return False
        if _git("status", "--porcelain", "-uno").stdout.strip():
            print("  [!] Mise à jour disponible, mais des fichiers sont modifiés : ignorée.")
            return False
        print("  [*] %s mise(s) à jour de WEB_SUITE : téléchargement..." % retard)
        return _git("pull", "--ff-only", timeout=60).returncode == 0
    except (OSError, subprocess.SubprocessError):
        return False


def redemarrer():
    env = dict(os.environ, WEB_SUITE_DEJA_MAJ="1")
    code = subprocess.call([sys.executable, os.path.abspath(__file__)] + sys.argv[1:], env=env)
    os._exit(code)


# ---------------------------------------------------------------------------
# Serveur HTTP
# ---------------------------------------------------------------------------
class Lanceur(SimpleHTTPRequestHandler):
    gestionnaire = None                                # outils.Gestionnaire

    def log_message(self, fmt, *args):                 # la console reste lisible
        pass

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
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        self.wfile.write(corps)

    def do_GET(self):
        url = urlparse(self.path)
        g = self.gestionnaire
        if url.path in FICHIERS_SERVIS:
            return self._fichier(FICHIERS_SERVIS[url.path])
        if url.path == "/api/etat":
            return self._json({"racine": g.racine, "git": outils.git_disponible(),
                               "outils": g.etat()})
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
            corps = json.loads(self.rfile.read(longueur) or b"{}")
        except ValueError:
            return self._json({"erreur": "JSON invalide"}, HTTPStatus.BAD_REQUEST)
        oid = corps.get("id")
        g = self.gestionnaire
        actions = {"/api/installer": g.installer, "/api/lancer": g.lancer,
                   "/api/arreter": g.arreter}
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


def ouvrir_serveur(port):
    for essai in range(port, port + 10):
        try:
            return ThreadingHTTPServer(("127.0.0.1", essai), Lanceur)
        except OSError:
            continue
    raise OSError("aucun port libre entre %d et %d" % (port, port + 9))


def main(argv=None):
    installer._console_utf8()
    ap = argparse.ArgumentParser(description="Lanceur WEB·SUITE.")
    ap.add_argument("--port", type=int, default=PORT_DEFAUT,
                    help="port du lanceur (défaut : %d)" % PORT_DEFAUT)
    ap.add_argument("--racine", default=outils.RACINE_DEFAUT,
                    help="dossier contenant les outils (défaut : le dossier de WEB_SUITE)")
    ap.add_argument("--installer", action="store_true",
                    help="ouvrir seulement le menu de téléchargement des outils")
    ap.add_argument("--sans-navigateur", action="store_true",
                    help="ne pas ouvrir le navigateur au démarrage")
    ap.add_argument("--sans-maj", dest="verifier_maj", action="store_false", default=True,
                    help="ne pas vérifier les mises à jour GitHub au démarrage")
    args = ap.parse_args(argv)
    racine = os.path.abspath(args.racine)

    if args.installer:
        return installer.main(["--racine", racine])

    if args.verifier_maj and os.environ.get("WEB_SUITE_DEJA_MAJ") != "1":
        if verifier_et_appliquer_maj():
            redemarrer()

    # Premier démarrage sans aucun outil : autant demander tout de suite, en
    # console, lequel télécharger. La page le propose aussi, pour les autres cas.
    aucun = not any(outils.est_installe(o, racine) for o in outils.CATALOGUE)
    if aucun and sys.stdin is not None and sys.stdin.isatty():
        print("\n  Aucun outil installé dans %s." % racine)
        installer.installer(installer.menu(racine), racine)

    Lanceur.gestionnaire = outils.Gestionnaire(racine)
    try:
        serveur = ouvrir_serveur(args.port)
    except OSError as exc:
        print("  [X] %s" % exc)
        return 1
    url = "http://127.0.0.1:%d/" % serveur.server_address[1]

    print()
    print("  WEB·SUITE")
    print("  " + "-" * 48)
    print("  Adresse   %s" % url)
    print("  Outils    %s" % racine)
    for o in outils.CATALOGUE:
        print("            %-12s %s" % (o["nom"], "installé" if outils.est_installe(o, racine)
                                         else "non installé"))
    print()
    print("  Fermer cette fenêtre (ou Ctrl+C) arrête aussi les outils lancés.")
    print()
    sys.stdout.flush()

    if not args.sans_navigateur:
        threading.Timer(0.6, lambda: webbrowser.open(url)).start()
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
