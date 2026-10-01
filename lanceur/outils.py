"""Catalogue des outils WEB·SUITE : où ils vivent, comment les installer et les lancer.

Aucune dépendance hors bibliothèque standard. Ce module est partagé par
``web_suite.py`` (le lanceur web) et ``installer.py`` (le menu en console).

Disposition attendue sur le disque :

    WEB_TOOLS/         <- ce dépôt (installer.cmd, demarrer_WEB_SUITE.cmd)
    ├── lanceur/       <- ce module, web_suite.py, installer.py, pages...
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

import projets

ICI = os.path.dirname(os.path.abspath(__file__))       # lanceur/
RACINE_DEFAUT = os.path.dirname(ICI)                   # racine du dépôt, où vivent les outils
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
        # openEMS : archive binaire + venv Python 3.10/3.11, voir installer_openems().
        "dependances": "openems",
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

# Dernière version stable d'openEMS pour Windows. Ses roues Python n'existent
# que pour CPython 3.10 et 3.11 : c'est ce qui impose un venv à part.
OPENEMS_URL = ("https://github.com/thliebig/openEMS-Project/releases/download/"
               "v0.0.36/openEMS_v0.0.36.zip")
OPENEMS_VERIF = (
    "import os,sys\n"
    "d=sys.argv[1]\n"
    "os.add_dll_directory(d)\n"
    "os.environ['PATH']=d+os.pathsep+os.environ.get('PATH','')\n"
    "import CSXCAD, openEMS\n"
    "from importlib.metadata import version\n"
    "print('openEMS', version('openEMS'), '/ Python', sys.version.split()[0])\n"
)
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
        code = _executer(["git", "clone", "--progress", outil["depot"], cible], ecrire)
        if code == 0 and est_installe(outil, racine):
            ecrire("[OK] %s installé." % outil["nom"])
            return True
        ecrire("[X] git clone a échoué (code %s)." % code)
        return False

    url = outil["depot"][:-4] + "/archive/refs/heads/main.zip"
    ecrire("git introuvable : téléchargement de l'archive %s" % url)
    ecrire("(sans git, l'outil ne pourra pas se mettre à jour tout seul)")
    try:
        with urllib.request.urlopen(url, timeout=60) as rep:
            donnees = rep.read()
        with zipfile.ZipFile(io.BytesIO(donnees)) as z:
            _extraire(z, z.namelist()[0].split("/")[0] + "/", cible)
    except Exception as exc:                           # noqa: BLE001
        ecrire("[X] Téléchargement impossible : %s" % exc)
        return False
    ok = est_installe(outil, racine)
    ecrire("[OK] %s installé." % outil["nom"] if ok else "[X] Archive incomplète.")
    return ok


def _executer(cmd, ecrire, cwd=None):
    """Lance une commande et recopie sa sortie ligne à ligne. Renvoie le code de sortie."""
    try:
        proc = subprocess.Popen(
            cmd, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            stdin=subprocess.DEVNULL, creationflags=SANS_FENETRE,
            env=dict(os.environ, PYTHONIOENCODING="utf-8"),
        )
    except OSError as exc:
        ecrire("[X] %s : %s" % (cmd[0], exc))
        return -1
    # git et pip réécrivent leur ligne de progression avec \r : on découpe dessus aussi.
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
    return proc.wait()


def _extraire(z, prefixe, cible):
    """Extrait les entrées de l'archive qui commencent par `prefixe` dans `cible`."""
    base = os.path.normpath(cible)
    for nom in z.namelist():
        if not nom.startswith(prefixe):
            continue
        relatif = nom[len(prefixe):]
        if not relatif:
            continue
        chemin = os.path.normpath(os.path.join(base, relatif))
        if not chemin.startswith(base + os.sep):
            continue                                   # entrée hors du dossier : ignorée
        if nom.endswith("/"):
            os.makedirs(chemin, exist_ok=True)
        else:
            os.makedirs(os.path.dirname(chemin), exist_ok=True)
            with z.open(nom) as src, open(chemin, "wb") as dst:
                shutil.copyfileobj(src, dst)


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
# openEMS (WEB_ANTENNA)
# ---------------------------------------------------------------------------
# Trois morceaux, que WEB_ANTENNA sait retrouver tout seul une fois en place :
#   WEB_ANTENNA/openEMS/   l'archive binaire (DLL, openEMS.exe, roues Python)
#   WEB_ANTENNA/env/       un venv Python 3.10 ou 3.11, où les roues s'installent
#   requirements.txt       numpy, h5py, et CSXCAD/openEMS pris dans openEMS/python
def _dossier_openems(outil, racine):
    return os.path.join(dossier(outil, racine), "openEMS")


def _python_env(outil, racine):
    return os.path.join(dossier(outil, racine), "env", "Scripts", "python.exe")


def openems_pret(outil, racine=RACINE_DEFAUT):
    """Test rapide (sans lancer Python) : binaires en place et roues installées dans env/."""
    if outil.get("dependances") != "openems":
        return None
    paquets = os.path.join(dossier(outil, racine), "env", "Lib", "site-packages")
    return (os.path.isfile(os.path.join(_dossier_openems(outil, racine), "CSXCAD.dll"))
            and os.path.isfile(_python_env(outil, racine))
            and os.path.isdir(os.path.join(paquets, "openEMS"))
            and os.path.isdir(os.path.join(paquets, "CSXCAD")))


def _sortie(cmd, delai=30):
    """Sortie d'une commande courte, ou '' si elle échoue."""
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=delai,
                           stdin=subprocess.DEVNULL, creationflags=SANS_FENETRE)
    except (OSError, subprocess.SubprocessError):
        return ""
    return r.stdout.strip() if r.returncode == 0 else ""


def _version_python(exe):
    """'3.10' pour un Python 64 bits, '' sinon (32 bits, introuvable, cassé)."""
    return _sortie([exe, "-c", "import sys,struct;"
                    "print('%d.%d' % sys.version_info[:2] if struct.calcsize('P') == 8 else '')"])


def _versions_roues(dossier_oe):
    """Versions de Python pour lesquelles l'archive fournit des roues : {'3.10', '3.11'}."""
    versions = set()
    try:
        noms = os.listdir(os.path.join(dossier_oe, "python"))
    except OSError:
        return versions
    for nom in noms:
        m = re.search(r"-cp3(\d+)-", nom)
        if nom.endswith(".whl") and m:
            versions.add("3." + m.group(1))
    return versions


def _trouver_python(versions):
    """Un Python 64 bits de l'une de ces versions, ou None. Le lanceur py d'abord."""
    for v in sorted(versions, reverse=True):
        exe = _sortie(["py", "-" + v, "-c", "import sys;print(sys.executable)"])
        if exe and _version_python(exe) == v:
            return exe
    candidats = [sys.executable] + [shutil.which("python" + v) for v in sorted(versions, reverse=True)]
    for exe in candidats:
        if exe and _version_python(exe) in versions:
            return exe
    return None


def _telecharger_openems(cible, ecrire):
    """Télécharge l'archive openEMS et l'extrait dans `cible`. Renvoie True si CSXCAD.dll y est."""
    ecrire("Téléchargement de %s" % OPENEMS_URL)
    tmp = cible + ".zip.part"
    try:
        with urllib.request.urlopen(OPENEMS_URL, timeout=60) as rep, open(tmp, "wb") as f:
            total = int(rep.headers.get("Content-Length") or 0)
            lu, palier = 0, 0
            while True:
                bloc = rep.read(1 << 20)
                if not bloc:
                    break
                f.write(bloc)
                lu += len(bloc)
                if total and lu * 100 // total >= palier + 10:
                    palier = lu * 100 // total // 10 * 10
                    ecrire("  %d %%  (%.0f / %.0f Mo)" % (palier, lu / 1e6, total / 1e6))
        with zipfile.ZipFile(tmp) as z:
            # L'archive range tout sous un dossier (openEMS/ aujourd'hui) : on
            # prend celui qui contient CSXCAD.dll, pour ne pas finir en openEMS/openEMS/.
            dll = [n for n in z.namelist() if n.lower().endswith("/csxcad.dll") or n.lower() == "csxcad.dll"]
            if not dll:
                ecrire("[X] L'archive ne contient pas CSXCAD.dll : format inattendu.")
                return False
            ecrire("Extraction dans %s" % cible)
            _extraire(z, min(dll, key=len)[:-len("CSXCAD.dll")], cible)
    except Exception as exc:                           # noqa: BLE001
        ecrire("[X] Téléchargement d'openEMS impossible : %s" % exc)
        return False
    finally:
        try:
            os.remove(tmp)
        except OSError:
            pass
    return os.path.isfile(os.path.join(cible, "CSXCAD.dll"))


def installer_openems(outil, racine=RACINE_DEFAUT, ecrire=print):
    """Installe openEMS pour WEB_ANTENNA : archive binaire, venv env/, pip. Renvoie True si prêt.

    Chaque étape déjà faite est sautée : on peut relancer après avoir corrigé
    ce qui manquait (typiquement : installer Python 3.10 ou 3.11).
    """
    if outil.get("dependances") != "openems":
        return True
    if not WINDOWS:
        ecrire("[!] Installation automatique d'openEMS : Windows seulement. "
               "Voir le guide en tête de %s." % os.path.join(dossier(outil, racine), "requirements.txt"))
        return False
    if not est_installe(outil, racine):
        ecrire("[X] %s n'est pas installé." % outil["nom"])
        return False
    # Chemin long et réel : venv échoue sur un nom court 8.3 (PIERRE~1.REN).
    racine = os.path.realpath(racine)
    base = dossier(outil, racine)
    if len(base) > 140:
        # pip écrit dans env\Lib\site-packages des chemins de ~120 caractères de plus.
        ecrire("[!] Dossier très profond (%d caractères) : Windows limite les chemins à 260," % len(base))
        ecrire("    pip risque d'échouer (« WinError 206 »). Rapprochez WEB_SUITE de la racine du disque.")

    # 1. Les binaires
    oe = _dossier_openems(outil, racine)
    if os.path.isfile(os.path.join(oe, "CSXCAD.dll")):
        ecrire("[OK] Binaires openEMS déjà présents : %s" % oe)
    elif os.path.isdir(oe) and os.listdir(oe):
        ecrire("[X] %s existe mais ne contient pas CSXCAD.dll : dossier laissé intact." % oe)
        ecrire("    Videz-le ou supprimez-le, puis relancez l'installation d'openEMS.")
        return False
    elif not _telecharger_openems(oe, ecrire):
        return False
    else:
        ecrire("[OK] Binaires openEMS installés.")

    versions = _versions_roues(oe) or {"3.10", "3.11"}
    liste = " ou ".join(sorted(versions))

    # 2. Le venv
    py_env = _python_env(outil, racine)
    if os.path.isfile(py_env):
        v = _version_python(py_env)
        if v not in versions:
            ecrire("[X] %s existe déjà avec Python %s : openEMS demande Python %s (64 bits)."
                   % (os.path.dirname(os.path.dirname(py_env)), v or "?", liste))
            ecrire("    Supprimez ce dossier env\\ puis relancez l'installation d'openEMS.")
            return False
        ecrire("[OK] Environnement Python %s déjà présent : env\\" % v)
        if not _sortie([py_env, "-m", "pip", "--version"]):
            ecrire("pip manque dans env\\ : réinstallation par ensurepip")
            if _executer([py_env, "-m", "ensurepip", "--upgrade", "--default-pip"], ecrire) != 0:
                ecrire("[X] Impossible de réparer pip : supprimez env\\ et relancez.")
                return False
    else:
        exe = _trouver_python(versions)
        if not exe:
            ecrire("[X] Aucun Python %s 64 bits sur ce poste, et les roues openEMS n'existent" % liste)
            ecrire("    que pour ces versions. Installez Python 3.11 64 bits :")
            ecrire("      https://www.python.org/downloads/windows/   ou   winget install Python.Python.3.11")
            ecrire("    puis relancez l'installation d'openEMS (les binaires sont gardés).")
            return False
        ecrire("Création de l'environnement env\\ avec %s" % exe)
        dossier_env = os.path.join(base, "env")
        if _executer([exe, "-m", "venv", dossier_env], ecrire) != 0 or not os.path.isfile(py_env):
            # Un env\ à moitié créé (sans pip) serait pris pour bon à la relance.
            shutil.rmtree(dossier_env, ignore_errors=True)
            ecrire("[X] La création de env\\ a échoué.")
            return False

    # 3. Les paquets. cwd = l'outil : requirements.txt cherche les roues dans openEMS/python.
    ecrire("pip install -r requirements.txt  (numpy, h5py, CSXCAD, openEMS)")
    code = _executer([py_env, "-m", "pip", "install", "--disable-pip-version-check",
                      "-r", "requirements.txt"], ecrire, cwd=base)
    if code != 0:
        ecrire("[X] pip a échoué (code %d) : voir les lignes ci-dessus." % code)
        return False

    # 4. La preuve : import CSXCAD, openEMS avec les DLL de l'archive.
    rapport = _sortie([py_env, "-c", OPENEMS_VERIF, oe], delai=60)
    if not rapport:
        ecrire("[X] openEMS est installé mais « import openEMS » échoue encore.")
        return False
    ecrire("[OK] %s — les simulations de %s sont disponibles." % (rapport, outil["nom"]))
    return True


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

    def __init__(self, racine=RACINE_DEFAUT, reseau=False):
        self.racine = racine
        # reseau : les outils écoutent sur le réseau local (sans --local) pour
        # qu'une tablette ou un autre poste puisse les ouvrir.
        self.reseau = reseau
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
            a_envoyer = projets.en_attente(self.racine, o["id"])    # git status, hors verrou
            with self.verrou:
                e = self.etats[o["id"]]
                self._actualiser(o, e)
                sortie.append({
                    "id": o["id"], "nom": o["nom"], "sous_titre": o["sous_titre"],
                    "couleur": o["couleur"], "dependances": o["dependances"],
                    "installe": est_installe(o, self.racine),
                    "openems": openems_pret(o, self.racine),
                    "dossier": dossier(o, self.racine),
                    "phase": e["phase"], "message": e["message"],
                    "url": e["url"], "port": e["port"], "a_envoyer": a_envoyer,
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
                # Le lanceur tourne sur la même machine : 127.0.0.1 répond
                # toujours. La page remplace l'hôte par celui qu'elle utilise.
                return "http://127.0.0.1:%d/" % port
        return None

    # -- actions -------------------------------------------------------------
    def installer(self, oid):
        return self._installer_en_fond(oid, installer, "Installé.")

    def installer_openems(self, oid):
        o = PAR_ID[oid]
        if o["dependances"] != "openems" or not est_installe(o, self.racine):
            return False
        with self.verrou:
            if self.etats[oid]["phase"] in ("demarrage", "actif"):
                self.etats[oid]["message"] = "Arrêtez l'outil avant d'installer openEMS."
                return False
        return self._installer_en_fond(oid, installer_openems,
                                       "openEMS installé : les simulations sont disponibles.")

    def _installer_en_fond(self, oid, fonction, message_ok):
        o = PAR_ID[oid]
        with self.verrou:
            if self.etats[oid]["phase"] == "installation":
                return False
            self.etats[oid].update(phase="installation", message="Préparation…")

        def tache():
            try:
                ok = fonction(o, self.racine, lambda t: self._noter(oid, t))
            except Exception as exc:                   # noqa: BLE001
                self._noter(oid, "[X] Erreur inattendue : %s" % exc)
                ok = False
            with self.verrou:
                self.etats[oid]["phase"] = None if ok else "erreur"
                if ok:
                    self.etats[oid]["message"] = message_ok
        threading.Thread(target=tache, daemon=True).start()
        return True

    def lancer(self, oid):
        o = PAR_ID[oid]
        if self.etats[oid]["phase"] not in ("demarrage", "actif") and git_disponible():
            # La lib ou un projet a pu changer sur un autre PC depuis le démarrage.
            self._noter(oid, projets.tirer(self.racine))
        with self.verrou:
            e = self.etats[oid]
            self._actualiser(o, e)
            if e["phase"] in ("demarrage", "actif"):
                return True
            if not est_installe(o, self.racine):
                e.update(phase="erreur", message="Outil non installé.")
                return False
            port = port_libre(o["port"])
            args = [x for x in o["args"] if not (self.reseau and x == "--local")]
            cmd = ([sys.executable, "-u", o["script"], "--port", str(port)] + args
                   + projets.arguments(oid, self.racine, self.reseau))
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

    def envoyer(self, oid, message):
        ok, texte = projets.envoyer(self.racine, oid, message)
        self._noter(oid, texte)
        return ok, texte

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
