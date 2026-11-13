"""Un lanceur WEB·SUITE (exécutable Rust) en sous-processus, pour les bancs d'essai.

    with Poste() as poste:                       # dossier temporaire, faux GitHub
        with poste.lanceur("--reseau", WEB_TOOLS_BANC_DISTANT="1") as l:
            r = l.req("/api/etat")

Rien ne part sur le réseau : un dépôt nu local joue GitHub (WEB_SUITE_BANC_PROJETS),
le jeton et la CA vont dans le dossier temporaire (WEB_TOOLS_JETON_FICHIER), git
ne voit aucune identité du poste.
"""
import http.client
import os
import re
import subprocess
import tempfile
import threading
import time

from banc_bin import binaire

RE_CODE = re.compile(r"Code d'appairage[^:\n]*:\s*(\d{3} \d{3})")


class Poste:
    def __init__(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = self._tmp.name
        self.racine = os.path.join(self.tmp, "racine")
        os.makedirs(self.racine)
        vide = os.path.join(self.tmp, "gitconfig-vide")
        open(vide, "w").close()
        nu = os.path.join(self.tmp, "github.git")
        self.fichier_jeton = os.path.join(self.tmp, "partage", "jeton.txt")
        self.env = dict(os.environ, GIT_CONFIG_GLOBAL=vide, GIT_CONFIG_NOSYSTEM="1",
                        WEB_SUITE_BANC_PROJETS=nu, WEB_TOOLS_JETON_FICHIER=self.fichier_jeton)
        for cle in ("WEB_TOOLS_BANC_DISTANT", "WEB_TOOLS_HTTP", "WEB_SUITE_RELANCEUR", "WEB_SUITE_DEJA_MAJ"):
            self.env.pop(cle, None)
        subprocess.run(["git", "init", "--quiet", "--bare", nu], check=False, env=self.env,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self._tmp.cleanup()

    def jeton(self):
        with open(self.fichier_jeton, encoding="utf-8") as f:
            return f.read().strip()

    def lanceur(self, *options, **env):
        return Lanceur(self, options, env)

    def lancer(self, nom, *args, **env):
        """Un exécutable court (installer…) : -> (code de sortie, sortie)."""
        r = subprocess.run([binaire(nom), *args], env=dict(self.env, **env), stdin=subprocess.DEVNULL,
                           stdout=subprocess.PIPE, stderr=subprocess.STDOUT, encoding="utf-8",
                           errors="replace", timeout=120)
        return r.returncode, r.stdout


class Lanceur:
    def __init__(self, poste, options, env):
        self.lignes = []
        maj = [] if env.pop("maj", False) else ["--sans-maj"]   # maj=True : WEB_SUITE cherche sa mise à jour
        self.proc = subprocess.Popen(
            [binaire("web_suite"), "--port", "0", "--racine", poste.racine, *maj,
             "--sans-navigateur", *options],
            env=dict(poste.env, **env), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, encoding="utf-8", errors="replace")
        threading.Thread(target=self._lire, daemon=True).start()
        # la dernière ligne du démarrage : le serveur écoute déjà
        self.attendre("Navigateur non ouvert")
        if "--reseau" in options:
            self.attendre("Code d'appairage")
        self.port = int(re.search(r"Adresse   http://127\.0\.0\.1:(\d+)/", self.sortie()).group(1))

    def _lire(self):
        for ligne in self.proc.stdout:
            self.lignes.append(ligne)

    def sortie(self):
        return "".join(self.lignes)

    def codes(self):
        """Les codes d'appairage affichés dans le terminal du lanceur, sans l'espace."""
        return [c.replace(" ", "") for c in RE_CODE.findall(self.sortie())]

    def attendre(self, texte, delai=30):
        fin = time.time() + delai
        while texte not in self.sortie():
            assert self.proc.poll() is None, "le lanceur s'est arrêté :\n" + self.sortie()
            assert time.time() < fin, "« %s » attendu dans :\n%s" % (texte, self.sortie())
            time.sleep(0.02)

    def req(self, chemin, methode="GET", hote=None, entetes=None, corps=b"", contexte=None):
        if contexte:
            c = http.client.HTTPSConnection("127.0.0.1", self.port, timeout=10, context=contexte)
        else:
            c = http.client.HTTPConnection("127.0.0.1", self.port, timeout=10)
        c.putrequest(methode, chemin, skip_host=True)
        c.putheader("Host", hote or "127.0.0.1:%d" % self.port)
        for k, v in (entetes or {}).items():
            c.putheader(k, v)
        if methode == "POST":
            c.putheader("Content-Length", (entetes or {}).get("Content-Length") or str(len(corps)))
        c.endheaders(corps if methode == "POST" and corps else None)
        r = c.getresponse()
        r.corps = r.read()
        c.close()
        return r

    def arreter(self):
        """SIGTERM (kill, systemctl stop) : -> code de sortie."""
        self.proc.terminate()
        try:
            return self.proc.wait(20)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            raise

    def __enter__(self):
        return self

    def __exit__(self, *_):
        if self.proc.poll() is None:
            self.proc.kill()
            self.proc.wait()
