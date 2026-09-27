"""Catalogue des outils WEB·SUITE : où ils vivent, comment les installer et les lancer.

Aucune dépendance hors bibliothèque standard. Ce module est partagé par
``web_suite.py`` (le lanceur web) et ``installer.py`` (le menu en console).

Disposition attendue sur le disque :

    WEB_TOOLS/         <- ce dépôt (web_suite.py, installer.py, ...)
    ├── WEB_CAO/       <- un dépôt git par outil, cloné ici
    ├── WEB_ANTENNA/      et ignoré par le .gitignore de WEB_SUITE
    └── WEB_3D/
"""

import io
import os
import re
import shutil
import socket
import subprocess
import sys
import threading
import time
import urllib.request
import zipfile

ICI = os.path.dirname(os.path.abspath(__file__))
RACINE_DEFAUT = ICI
DOSSIER_JOURNAUX = os.path.join(ICI, "journaux")

WINDOWS = os.name == "nt"
# Pas de fenêtre console pour les serveurs lancés : leur sortie va au journal.
SANS_FENETRE = 0x08000000 if WINDOWS else 0          # CREATE_NO_WINDOW

CATALOGUE = [
    {
        "id": "web_cao",
        "nom": "WEB_CAO",
        "sous_titre": "Schéma & routage PCB",
        "couleur": "#ff9a4d",
        "depot": "https://github.com/pilou33620/WEB_CAO.git",
        "script": "web_CAO.py",
        "port": 8000,
        "args": ["--local", "--sans-navigateur", "--sans-pause"],
        # numpy / scipy : solveurs SI-PI, facultatifs, installables par pip.
        "dependances": "pip",
    },
    {
        "id": "web_antenna",
        "nom": "WEB_ANTENNA",
        "sous_titre": "Simulation d’antennes RF",
        "couleur": "#4fe0ff",
        "depot": "https://github.com/pilou33620/WEB_ANTENNA.git",
        "script": "web_antenna.py",
        "port": 8732,
        "args": ["--local", "--sans-navigateur"],
        # openEMS : Python 3.10/3.11 et archives binaires, à la main.
        "dependances": "guide",
    },
    {
        "id": "web_3d",
        "nom": "WEB_3D",
        "sous_titre": "Modélisation & assemblage 3D",
        "couleur": "#8dffb8",
        "depot": "https://github.com/pilou33620/WEB_3D.git",
        "script": "web_3D.py",
        "port": 8139,
        "args": ["--local", "--sans-navigateur", "--sans-pause"],
        "dependances": None,
    },
]
PAR_ID = {o["id"]: o for o in CATALOGUE}

DELAI_DEMARRAGE = 90       # s : la vérification GitHub des outils peut prendre du temps
RE_URL = re.compile(r"https?://([\w.\-]+|\[[0-9a-fA-F:]+\]):(\d{2,5})")


# ---------------------------------------------------------------------------
# Emplacements
# ---------------------------------------------------------------------------
def dossier(outil, racine=RACINE_DEFAUT):
    return os.path.join(racine, outil["nom"])


def est_installe(outil, racine=RACINE_DEFAUT):
    return os.path.isfile(os.path.join(dossier(outil, racine), outil["script"]))


def git_disponible():
    return shutil.which("git") is not None


# ---------------------------------------------------------------------------
# Installation
# ---------------------------------------------------------------------------
def installer(outil, racine=RACINE_DEFAUT, ecrire=print):
    """Clone l'outil dans un sous-dossier de la racine. Renvoie True si l'outil est prêt.

    git d'abord : c'est lui qui permet aux outils de se mettre à jour seuls
    au démarrage. Sans git, on se rabat sur l'archive zip de la branche main.
    """
    cible = dossier(outil, racine)
    if est_installe(outil, racine):
        ecrire("%s est déjà installé dans %s" % (outil["nom"], cible))
        return True
    if os.path.exists(cible) and os.listdir(cible):
        ecrire("[X] %s existe mais ne contient pas %s : dossier laissé intact."
               % (cible, outil["script"]))
        return False
    os.makedirs(racine, exist_ok=True)

    if git_disponible():
        ecrire("git clone %s" % outil["depot"])
        proc = subprocess.Popen(
            ["git", "clone", "--progress", outil["depot"], cible],
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL,
            creationflags=SANS_FENETRE,
        )
        # git réécrit sa ligne de progression avec \r : on découpe dessus aussi.
        tampon = b""
        while True:
            bloc = proc.stdout.read(256)
            if not bloc:
                break
            tampon += bloc
            *lignes, tampon = re.split(rb"[\r\n]", tampon)
            for ligne in lignes:
                if ligne.strip():
                    ecrire(ligne.decode("utf-8", "replace").strip())
        if tampon.strip():
            ecrire(tampon.decode("utf-8", "replace").strip())
        if proc.wait() == 0 and est_installe(outil, racine):
            ecrire("[OK] %s installé." % outil["nom"])
            return True
        ecrire("[X] git clone a échoué (code %s)." % proc.returncode)
        return False

    url = outil["depot"][:-4] + "/archive/refs/heads/main.zip"
    ecrire("git introuvable : téléchargement de l'archive %s" % url)
    ecrire("(sans git, l'outil ne pourra pas se mettre à jour tout seul)")
    try:
        with urllib.request.urlopen(url, timeout=60) as rep:
            donnees = rep.read()
        with zipfile.ZipFile(io.BytesIO(donnees)) as z:
            prefixe = z.namelist()[0].split("/")[0] + "/"
            for nom in z.namelist():
                relatif = nom[len(prefixe):]
                if not relatif:
                    continue
                chemin = os.path.normpath(os.path.join(cible, relatif))
                if not chemin.startswith(os.path.normpath(cible) + os.sep):
                    continue                           # entrée hors du dossier : ignorée
                if nom.endswith("/"):
                    os.makedirs(chemin, exist_ok=True)
                else:
                    os.makedirs(os.path.dirname(chemin), exist_ok=True)
                    with z.open(nom) as src, open(chemin, "wb") as dst:
                        shutil.copyfileobj(src, dst)
    except Exception as exc:                           # noqa: BLE001
        ecrire("[X] Téléchargement impossible : %s" % exc)
        return False
    ok = est_installe(outil, racine)
    ecrire("[OK] %s installé." % outil["nom"] if ok else "[X] Archive incomplète.")
    return ok


def installer_dependances(outil, racine=RACINE_DEFAUT, ecrire=print):
    """pip install -r requirements.txt de l'outil, pour ceux qui s'y prêtent."""
    req = os.path.join(dossier(outil, racine), "requirements.txt")
    if outil["dependances"] != "pip" or not os.path.isfile(req):
        return True
    ecrire("pip install -r %s" % req)
    code = subprocess.call([sys.executable, "-m", "pip", "install", "-r", req])
    ecrire("[OK] Dépendances installées." if code == 0
           else "[!] pip a échoué (code %d) : l'outil démarre quand même, "
                "sans ses solveurs avancés." % code)
    return code == 0


# ---------------------------------------------------------------------------
# Lancement
# ---------------------------------------------------------------------------
def port_libre(depart, essais=20):
    for port in range(depart, depart + essais):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            try:
                s.bind(("127.0.0.1", port))
                return port
            except OSError:
                continue
    return 0


def repond(port, delai=0.3):
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=delai):
            return True
    except OSError:
        return False


class _Job:
    """Objet Job Windows : les serveurs lancés meurent avec le lanceur.

    Sans lui, fermer la fenêtre de WEB_SUITE laisserait tourner des serveurs
    sans fenêtre, impossibles à arrêter autrement que par le gestionnaire des
    tâches. Les petits-enfants (WEB_ANTENNA qui se relance dans son venv)
    héritent du job.
    """

    def __init__(self):
        self.h = None
        if not WINDOWS:
            return
        try:
            import ctypes
            from ctypes import wintypes

            class IO(ctypes.Structure):
                _fields_ = [(n, ctypes.c_ulonglong) for n in (
                    "a", "b", "c", "d", "e", "f")]

            class BASIC(ctypes.Structure):
                _fields_ = [("PerProcessUserTimeLimit", ctypes.c_int64),
                            ("PerJobUserTimeLimit", ctypes.c_int64),
                            ("LimitFlags", wintypes.DWORD),
                            ("MinimumWorkingSetSize", ctypes.c_size_t),
                            ("MaximumWorkingSetSize", ctypes.c_size_t),
                            ("ActiveProcessLimit", wintypes.DWORD),
                            ("Affinity", ctypes.c_size_t),
                            ("PriorityClass", wintypes.DWORD),
                            ("SchedulingClass", wintypes.DWORD)]

            class EXT(ctypes.Structure):
                _fields_ = [("Basic", BASIC), ("Io", IO),
                            ("ProcessMemoryLimit", ctypes.c_size_t),
                            ("JobMemoryLimit", ctypes.c_size_t),
                            ("PeakProcessMemoryUsed", ctypes.c_size_t),
                            ("PeakJobMemoryUsed", ctypes.c_size_t)]

            k32 = ctypes.WinDLL("kernel32", use_last_error=True)
            k32.CreateJobObjectW.restype = wintypes.HANDLE
            k32.SetInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int,
                                                    ctypes.c_void_p, wintypes.DWORD]
            k32.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
            h = k32.CreateJobObjectW(None, None)
            info = EXT()
            info.Basic.LimitFlags = 0x2000             # KILL_ON_JOB_CLOSE
            if h and k32.SetInformationJobObject(h, 9, ctypes.byref(info), ctypes.sizeof(info)):
                self.h, self.k32 = h, k32
        except Exception:                              # noqa: BLE001
            self.h = None

    def ajouter(self, proc):
        if self.h:
            try:
                self.k32.AssignProcessToJobObject(self.h, int(proc._handle))
            except Exception:                          # noqa: BLE001
                pass


class Gestionnaire:
    """État des outils (installation, serveur lancé) partagé par le lanceur web."""

    def __init__(self, racine=RACINE_DEFAUT):
        self.racine = racine
        self.verrou = threading.Lock()
        self.job = _Job()
        self.etats = {o["id"]: {"phase": None, "message": "", "url": None,
                                "port": None, "proc": None, "debut": 0.0}
                      for o in CATALOGUE}
        os.makedirs(DOSSIER_JOURNAUX, exist_ok=True)

    def journal(self, oid):
        return os.path.join(DOSSIER_JOURNAUX, oid + ".log")

    def _noter(self, oid, texte):
        with self.verrou:
            self.etats[oid]["message"] = texte
        with open(self.journal(oid), "a", encoding="utf-8") as f:
            f.write(texte + "\n")

    # -- état lisible par la page ------------------------------------------
    def etat(self):
        sortie = []
        for o in CATALOGUE:
            with self.verrou:
                e = self.etats[o["id"]]
                self._actualiser(o, e)
                sortie.append({
                    "id": o["id"], "nom": o["nom"], "sous_titre": o["sous_titre"],
                    "couleur": o["couleur"], "dependances": o["dependances"],
                    "installe": est_installe(o, self.racine),
                    "dossier": dossier(o, self.racine),
                    "phase": e["phase"], "message": e["message"],
                    "url": e["url"], "port": e["port"],
                })
        return sortie

    def _actualiser(self, o, e):
        """Fait avancer 'demarrage' -> 'actif', et repère les serveurs tombés."""
        proc = e["proc"]
        if e["phase"] in ("demarrage", "actif") and proc is not None and proc.poll() is not None:
            e.update(phase="erreur", url=None, proc=None,
                     message="Le serveur s'est arrêté (code %s) : voir le journal." % proc.returncode)
            return
        if e["phase"] != "demarrage":
            return
        url = self._url_du_journal(o["id"])
        port = e["port"]
        if url is None and repond(port):
            url = "http://127.0.0.1:%d/" % port
        if url:
            e.update(phase="actif", url=url, message="En cours sur %s" % url)
        elif time.time() - e["debut"] > DELAI_DEMARRAGE:
            e.update(message="Toujours pas de réponse : voir le journal.")

    def _url_du_journal(self, oid):
        try:
            with open(self.journal(oid), encoding="utf-8", errors="replace") as f:
                texte = f.read()
        except OSError:
            return None
        texte = texte[texte.rfind("=== lancement"):]
        for m in RE_URL.finditer(texte):
            port = int(m.group(2))
            if repond(port):
                # --local : le serveur n'écoute que sur 127.0.0.1, quelle que
                # soit l'adresse réseau qu'il affiche.
                return "http://127.0.0.1:%d/" % port
        return None

    # -- actions -------------------------------------------------------------
    def installer(self, oid):
        o = PAR_ID[oid]
        with self.verrou:
            if self.etats[oid]["phase"] == "installation":
                return False
            self.etats[oid].update(phase="installation", message="Préparation…")

        def tache():
            ok = installer(o, self.racine, lambda t: self._noter(oid, t))
            with self.verrou:
                self.etats[oid]["phase"] = None if ok else "erreur"
                if ok:
                    self.etats[oid]["message"] = "Installé."
        threading.Thread(target=tache, daemon=True).start()
        return True

    def lancer(self, oid):
        o = PAR_ID[oid]
        with self.verrou:
            e = self.etats[oid]
            self._actualiser(o, e)
            if e["phase"] in ("demarrage", "actif"):
                return True
            if not est_installe(o, self.racine):
                e.update(phase="erreur", message="Outil non installé.")
                return False
            port = port_libre(o["port"])
            cmd = [sys.executable, "-u", o["script"], "--port", str(port)] + o["args"]
            env = dict(os.environ, PYTHONIOENCODING="utf-8", PYTHONUNBUFFERED="1")
            with open(self.journal(oid), "a", encoding="utf-8") as f:
                f.write("\n=== lancement %s : %s\n" % (time.strftime("%Y-%m-%d %H:%M:%S"),
                                                        " ".join(cmd)))
            sortie = open(self.journal(oid), "ab")
            try:
                proc = subprocess.Popen(
                    cmd, cwd=dossier(o, self.racine), env=env,
                    stdin=subprocess.DEVNULL, stdout=sortie, stderr=subprocess.STDOUT,
                    creationflags=SANS_FENETRE, start_new_session=not WINDOWS,
                )
            except OSError as exc:
                e.update(phase="erreur", message="Lancement impossible : %s" % exc)
                return False
            finally:
                sortie.close()
            self.job.ajouter(proc)
            e.update(phase="demarrage", proc=proc, port=port, url=None, debut=time.time(),
                     message="Démarrage (vérification des mises à jour)…")
            return True

    def arreter(self, oid):
        with self.verrou:
            e = self.etats[oid]
            proc = e["proc"]
            e.update(phase=None, proc=None, url=None, message="Arrêté.")
        if proc is not None and proc.poll() is None:
            _tuer(proc)
        return True

    def arreter_tout(self):
        for o in CATALOGUE:
            self.arreter(o["id"])


def _tuer(proc):
    """Arrête le serveur et ses enfants (le venv de WEB_ANTENNA en est un)."""
    if WINDOWS:
        subprocess.call(["taskkill", "/PID", str(proc.pid), "/T", "/F"],
                        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                        creationflags=SANS_FENETRE)
    else:
        import signal
        try:
            os.killpg(proc.pid, signal.SIGTERM)
        except OSError:
            proc.terminate()
    try:
        proc.wait(5)
    except subprocess.TimeoutExpired:
        proc.kill()
