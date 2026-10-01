"""Dossier PROJETS : les projets des outils, synchronisés avec GitHub.

    PROJETS/              <- dépôt git WEB_SUITE_PROJETS, ignoré par WEB_SUITE
    ├── projets.txt       <- la liste des projets, réécrite à chaque envoi
    ├── CAO/<projet>/     <- WEB_CAO --projets
    ├── LIB_CAO/          <- WEB_CAO --lib
    ├── ANTENNA/<projet>/ <- WEB_ANTENNA --projets
    └── 3D/               <- pas encore de projets dans WEB_3D

Le lanceur tire (pull) au démarrage et avant chaque lancement d'outil, et
envoie (commit + push) quand on arrête un outil. Un seul utilisateur : en cas
de conflit on s'arrête et on prévient, sans rien fusionner tout seul.
"""

import os
import subprocess
import threading

DEPOT = "https://github.com/pilou33620/WEB_SUITE_PROJETS.git"
BRANCHE = "main"
SANS_FENETRE = 0x08000000 if os.name == "nt" else 0

# Les dossiers de chaque outil : ce qu'on lui passe, et ce qu'on envoie à l'arrêt.
DOSSIERS = {"web_cao": ("CAO", "LIB_CAO"), "web_antenna": ("ANTENNA",), "web_3d": ("3D",)}
SECTIONS = (("WEB_CAO", "CAO"), ("WEB_ANTENNA", "ANTENNA"), ("WEB_3D", "3D"))
# Les résultats d'openEMS se recalculent et pèsent vite des centaines de Mo.
GITIGNORE = "ANTENNA/*/calculs/\n"

_verrou = threading.Lock()                             # une commande git à la fois


def dossier(racine):
    return os.path.join(racine, "PROJETS")


def arguments(oid, racine, reseau=False):
    """Les options qui pointent l'outil vers ses dossiers de PROJETS.

    reseau : le lanceur écoute sur le réseau (--reseau). WEB_CAO y ferme
    d'office ses routes de projet ; --projets-reseau les rouvre, confinées
    à PROJETS/CAO et à la LIB. WEB_ANTENNA, lui, les sert déjà au réseau."""
    p = dossier(racine)
    if oid == "web_cao":
        return (["--projets", os.path.join(p, "CAO"), "--lib", os.path.join(p, "LIB_CAO")]
                + (["--projets-reseau"] if reseau else []))
    if oid == "web_antenna":
        return ["--projets", os.path.join(p, "ANTENNA")]
    return []


def _git(cwd, *args, timeout=60):
    # GIT_TERMINAL_PROMPT=0 : pas de question en console invisible. Sous Windows,
    # Git Credential Manager ouvre sa propre fenêtre de connexion au premier push.
    try:
        r = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True,
                           encoding="utf-8", errors="replace", timeout=timeout,
                           stdin=subprocess.DEVNULL, creationflags=SANS_FENETRE,
                           env=dict(os.environ, GIT_TERMINAL_PROMPT="0"))
    except (OSError, subprocess.SubprocessError) as exc:
        return 1, str(exc)
    return r.returncode, (r.stdout + r.stderr).strip()


def preparer(racine, depot=DEPOT):
    """Crée ou récupère PROJETS/, puis le met à jour. Renvoie un message lisible."""
    p = dossier(racine)
    with _verrou:
        if not os.path.isdir(os.path.join(p, ".git")):
            vide = not os.path.isdir(p) or not os.listdir(p)
            if not vide or _git(racine, "clone", depot, p, timeout=600)[0] != 0:
                # Dépôt pas encore créé sur GitHub, ou hors ligne : on part en
                # local, le premier envoi réussi le remplira.
                os.makedirs(p, exist_ok=True)
                _git(p, "init")
                _git(p, "remote", "add", "origin", depot)
            if _git(p, "rev-parse", "--verify", "HEAD")[0] != 0:
                # Branche encore vide (init, ou clone d'un dépôt vide) : on la
                # nomme main quel que soit le réglage init.defaultBranch du poste.
                _git(p, "symbolic-ref", "HEAD", "refs/heads/" + BRANCHE)
        # Tirer avant de créer le squelette : un .gitkeep local non suivi
        # bloquerait le pull du même fichier venu de GitHub.
        etat = _tirer(p)
        for nom in ("CAO", "3D", "ANTENNA", "LIB_CAO"):
            os.makedirs(os.path.join(p, nom), exist_ok=True)
            if not os.listdir(os.path.join(p, nom)):
                open(os.path.join(p, nom, ".gitkeep"), "w").close()   # git ignore les dossiers vides
        gi = os.path.join(p, ".gitignore")
        if not os.path.isfile(gi):
            with open(gi, "w", encoding="utf-8") as f:
                f.write(GITIGNORE)
        return etat


def tirer(racine):
    with _verrou:
        return _tirer(dossier(racine))


def _tirer(p):
    code, sortie = _git(p, "pull", "--rebase", "--autostash", "origin", BRANCHE, timeout=120)
    if code == 0:
        return "Projets à jour."
    if "couldn't find remote ref" in sortie or "Repository not found" in sortie:
        return "Dépôt GitHub vide ou pas encore créé : projets gardés en local."
    if "CONFLICT" in sortie or "conflict" in sortie:
        _git(p, "rebase", "--abort")
        return "[X] Conflit avec GitHub : rien n'a été fusionné, à régler à la main dans %s." % p
    return "GitHub injoignable : on travaille en local (%s)." % (sortie.splitlines() or ["?"])[-1]


def en_attente(racine, oid):
    """True si les dossiers de l'outil ont des modifs à envoyer."""
    p = dossier(racine)
    if not os.path.isdir(os.path.join(p, ".git")):
        return False
    code, sortie = _git(p, "status", "--porcelain", "--", *DOSSIERS[oid], timeout=20)
    return code == 0 and bool(sortie)


def ecrire_liste(p):
    lignes = []
    for titre, nom in SECTIONS:
        lignes.append("[%s]" % titre)
        base = os.path.join(p, nom)
        lignes += sorted(n for n in os.listdir(base) if os.path.isdir(os.path.join(base, n))) \
            if os.path.isdir(base) else []
        lignes.append("")
    with open(os.path.join(p, "projets.txt"), "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lignes))


def envoyer(racine, oid, message):
    """commit + pull --rebase + push des dossiers de l'outil. Renvoie (ok, message)."""
    p = dossier(racine)
    with _verrou:
        if not os.path.isdir(os.path.join(p, ".git")):
            return False, "PROJETS n'est pas un dépôt git."
        ecrire_liste(p)
        _git(p, "add", "-A", "--", *DOSSIERS[oid], "projets.txt", ".gitignore")
        if _git(p, "diff", "--cached", "--quiet")[0] != 0:
            code, sortie = _git(p, "commit", "-m", message.strip() or "Session " + oid)
            if code != 0:
                return False, "[X] git commit a échoué : %s" % sortie[-300:]
        tire = _tirer(p)
        if tire.startswith("[X]"):
            return False, tire
        code, sortie = _git(p, "push", "-u", "origin", BRANCHE, timeout=600)
        if code != 0:
            return False, ("[!] Enregistré en local, mais l'envoi sur GitHub a échoué "
                           "(réessayé au prochain envoi) : %s" % (sortie.splitlines() or ["?"])[-1])
        return True, "Envoyé sur GitHub."


if __name__ == "__main__":
    # Vérification : un dépôt nu joue GitHub, deux racines jouent deux PC.
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        nu = os.path.join(tmp, "github.git")
        _git(tmp, "init", "--bare", nu)
        a, b = os.path.join(tmp, "pc_a"), os.path.join(tmp, "pc_b")
        os.makedirs(a), os.makedirs(b)
        print(preparer(a, nu))                         # dépôt vide : départ en local
        os.makedirs(os.path.join(dossier(a), "CAO", "carte_alim"))
        open(os.path.join(dossier(a), "CAO", "carte_alim", "projet.cao.json"), "w").close()
        assert en_attente(a, "web_cao")
        _git(dossier(a), "config", "user.name", "test")
        _git(dossier(a), "config", "user.email", "test@example.com")
        ok, msg = envoyer(a, "web_cao", "nouveau projet")
        assert ok, msg
        assert not en_attente(a, "web_cao")
        print(preparer(b, nu))                         # l'autre PC clone
        assert os.path.isfile(os.path.join(dossier(b), "CAO", "carte_alim", "projet.cao.json"))
        with open(os.path.join(dossier(b), "projets.txt"), encoding="utf-8") as f:
            assert "[WEB_CAO]\ncarte_alim\n" in f.read()
        assert os.path.isdir(os.path.join(dossier(b), "3D"))
    assert "--projets-reseau" in arguments("web_cao", "x", reseau=True)
    assert "--projets-reseau" not in arguments("web_cao", "x")
    print("OK")
