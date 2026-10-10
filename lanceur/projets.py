"""Dossier PROJETS : les projets des outils, synchronisés avec GitHub.

    PROJETS/              <- dépôt git WEB_SUITE_PROJETS, ignoré par WEB_SUITE
    ├── projets.txt       <- la liste des projets, réécrite à chaque envoi
    ├── CAO/<projet>/     <- WEB_CAO --projets
    ├── LIB_CAO/          <- WEB_CAO --lib
    ├── ANTENNA/<projet>/ <- WEB_ANTENNA --projets
    └── 3D/               <- WEB_3D --projets (modèles rangés depuis la visionneuse)

Le lanceur tire (pull) au démarrage et avant chaque lancement d'outil, et
envoie (commit + push) quand on arrête un outil. Un seul utilisateur : en cas
de conflit on s'arrête et on prévient, sans rien fusionner tout seul.

WEB_SUITE_PROJETS est privé : sans compte GitHub connecté sur le poste, git
ne peut ni le cloner ni le tirer. On ne crée alors pas de dépôt local (son
historique serait sans lien avec GitHub) : PROJETS reste un simple dossier,
cloné dès que le poste est connecté (connexion_github.py).
"""

import os
import re
import shutil
import subprocess
import threading
import time

DEPOT = "https://github.com/pilou33620/WEB_SUITE_PROJETS.git"
BRANCHE = "main"
SANS_FENETRE = 0x08000000 if os.name == "nt" else 0

# Les dossiers de chaque outil : ce qu'on lui passe, et ce qu'on envoie à l'arrêt.
DOSSIERS = {"web_cao": ("CAO", "LIB_CAO"), "web_antenna": ("ANTENNA",), "web_3d": ("3D",)}
SECTIONS = (("WEB_CAO", "CAO"), ("WEB_ANTENNA", "ANTENNA"), ("WEB_3D", "3D"))
# Les résultats d'openEMS se recalculent et pèsent vite des centaines de Mo.
# Les zip des packs Murata sont déballés à côté (lier_modeles_murata.py), et
# c'est le déballé qui est suivi : un zip ne se compresse pas en différences.
GITIGNORE = "ANTENNA/*/calculs/\nLIB_CAO/**/*.zip\n"
# Fins de ligne normalisées dans le dépôt : un PC Windows (autocrlf) et le Pi
# ne doivent pas se renvoyer des fichiers modifiés de la première à la dernière ligne.
GITATTRIBUTES = "* text=auto\n"
# Sans LC_ALL=C, un git traduit (Pi en fr_FR) ne dit plus « CONFLICT » ;
# GIT_OPTIONAL_LOCKS=0 : le git status de l'affichage ne prend plus index.lock
# pendant le commit d'un envoi.
ENV_GIT = dict(GIT_TERMINAL_PROMPT="0", LC_ALL="C", LANGUAGE="C", GIT_OPTIONAL_LOCKS="0")

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
    if oid == "web_3d":
        return ["--projets", os.path.join(p, "3D")]
    return []


def _git(cwd, *args, timeout=60):
    # GIT_TERMINAL_PROMPT=0 : pas de question en console invisible. Sous Windows,
    # Git Credential Manager ouvre sa propre fenêtre de connexion au premier push.
    try:
        r = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True,
                           encoding="utf-8", errors="replace", timeout=timeout,
                           stdin=subprocess.DEVNULL, creationflags=SANS_FENETRE,
                           env=dict(os.environ, **ENV_GIT))
    except (OSError, subprocess.SubprocessError) as exc:
        return 1, str(exc)
    return r.returncode, (r.stdout + r.stderr).strip()


def preparer(racine, depot=DEPOT):
    """Crée ou récupère PROJETS/, puis le met à jour. Renvoie un message lisible."""
    p = dossier(racine)
    note = ""
    with _verrou:
        if not os.path.isdir(os.path.join(p, ".git")):
            if not _vide(p):
                ok, note = _adopter(racine, p, depot)
                if not ok:
                    return note
            else:
                shutil.rmtree(p, ignore_errors=True)   # squelette laissé par un essai précédent
                code, sortie = _git(racine, "clone", depot, p, timeout=600)
                if code != 0 and refus_auth(sortie):
                    _squelette(p)
                    return ACCES_REFUSE
                if code != 0:
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
        _squelette(p, gitkeep=True)
        for nom, texte in ((".gitignore", GITIGNORE), (".gitattributes", GITATTRIBUTES)):
            if not os.path.isfile(os.path.join(p, nom)):
                with open(os.path.join(p, nom), "w", encoding="utf-8", newline="\n") as f:
                    f.write(texte)
        return (note + " " + etat).strip()


def _squelette(p, gitkeep=False):
    """Les dossiers des outils. .gitkeep : git ignore les dossiers vides."""
    for nom in ("CAO", "3D", "ANTENNA", "LIB_CAO"):
        os.makedirs(os.path.join(p, nom), exist_ok=True)
        if gitkeep and not os.listdir(os.path.join(p, nom)):
            open(os.path.join(p, nom, ".gitkeep"), "w").close()


def _vide(p):
    """Rien d'autre que des dossiers vides et des .gitkeep : le squelette d'un
    démarrage où GitHub a refusé le clone. On peut cloner à sa place."""
    return all(f == ".gitkeep" for _, _, fichiers in os.walk(p) for f in fichiers)


def _adopter(racine, p, depot):
    """PROJETS/ rempli mais sans .git : on clone à côté plutôt que git init.

    Un init donnait un historique sans lien avec GitHub, donc un conflit
    garanti au premier envoi. On clone dans PROJETS.clone, on y ajoute les
    fichiers locaux que GitHub n'a pas (GitHub gagne quand les deux existent),
    et l'ancien dossier reste intact à côté, en sauvegarde. Renvoie (ok, message)."""
    neuf = p + ".clone"
    shutil.rmtree(neuf, ignore_errors=True)
    code, sortie = _git(racine, "clone", depot, neuf, timeout=600)
    if code != 0:
        shutil.rmtree(neuf, ignore_errors=True)
        if refus_auth(sortie):
            return False, ACCES_REFUSE + " Vos fichiers locaux restent dans %s." % p
        return False, ("[!] %s existe sans git et GitHub est injoignable : rien n'est "
                       "synchronisé, nouvel essai au prochain démarrage." % p)
    shutil.copytree(p, neuf, dirs_exist_ok=True,
                    copy_function=lambda src, dst: os.path.exists(dst) or shutil.copy2(src, dst))
    sauvegarde = p + time.strftime(".avant-git-%Y%m%d-%H%M%S")
    try:
        os.rename(p, sauvegarde)
    except OSError as exc:
        shutil.rmtree(neuf, ignore_errors=True)
        return False, "[!] %s n'a pas pu être mis de côté (fichier ouvert ?) : %s" % (p, exc)
    os.rename(neuf, p)
    return True, "PROJETS relié à GitHub ; l'ancien dossier est gardé dans %s." % sauvegarde


def tirer(racine):
    with _verrou:
        return _tirer(dossier(racine))


def _tirer(p):
    if not os.path.isdir(os.path.join(p, ".git")):
        # Sans ce test, git remonterait au dépôt parent (WEB_SUITE cloné avec git).
        return ACCES_REFUSE
    code, sortie = _git(p, "pull", "--rebase", "--autostash", "origin", BRANCHE, timeout=120)
    if code == 0:
        return "Projets à jour."
    if "couldn't find remote ref" in sortie:
        return "Dépôt GitHub vide : projets gardés en local."
    if refus_auth(sortie):
        return ACCES_REFUSE
    # Le dossier de rebase plutôt que le texte : il reste aussi quand git a été
    # coupé par le délai, et ne laisse jamais PROJETS en plein rebase.
    en_rebase = any(os.path.isdir(os.path.join(p, ".git", d)) for d in ("rebase-merge", "rebase-apply"))
    if en_rebase or "CONFLICT" in sortie:
        _git(p, "rebase", "--abort")
        return "[X] Conflit avec GitHub : rien n'a été fusionné, à régler à la main dans %s." % p
    return "GitHub injoignable : on travaille en local (%s)." % (sortie.splitlines() or ["?"])[-1]


def en_attente(racine, oid):
    """True si les dossiers de l'outil ont des modifs à envoyer, ou des commits
    pas encore poussés (envoi précédent fait hors ligne)."""
    p = dossier(racine)
    if not os.path.isdir(os.path.join(p, ".git")):
        return False
    code, sortie = _git(p, "status", "--porcelain", "--", *DOSSIERS[oid], timeout=20)
    if code == 0 and sortie:
        return True
    # --not --remotes=origin : marche aussi avant le tout premier push (pas d'@{u}).
    code, sortie = _git(p, "rev-list", "--count", "HEAD", "--not", "--remotes=origin",
                        "--", *DOSSIERS[oid], timeout=20)
    return code == 0 and sortie.isdigit() and int(sortie) > 0


def ecrire_liste(p):
    lignes = []
    for titre, nom in SECTIONS:
        lignes.append("[%s]" % titre)
        base = os.path.join(p, nom)
        # Un dossier par projet (CAO, ANTENNA), un fichier par modèle (3D).
        lignes += sorted(n for n in os.listdir(base) if not n.startswith(".")) \
            if os.path.isdir(base) else []
        lignes.append("")
    with open(os.path.join(p, "projets.txt"), "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lignes))


def identite_manquante(racine):
    """True si git ne sait pas qui signe les commits de PROJETS.

    Cas d'un poste neuf, typiquement le Raspberry Pi : sans user.name et
    user.email, le commit échoue. La page les demande alors une fois."""
    p = dossier(racine)
    return any(_git(p, "config", cle, timeout=10)[0] != 0
               for cle in ("user.name", "user.email"))


def definir_identite(racine, nom, email):
    """Enregistre nom et e-mail dans la config du dépôt PROJETS (pas en global :
    on ne règle que ce que le lanceur utilise). Renvoie (ok, message)."""
    nom = " ".join(str(nom or "").split())
    email = str(email or "").strip()
    if not nom or len(nom) > 100:
        return False, "Nom invalide."
    if len(email) > 200 or not re.fullmatch(r"[^\s@]+@[^\s@]+\.[^\s@]+", email):
        return False, "Adresse e-mail invalide."
    with _verrou:
        p = dossier(racine)
        for cle, valeur in (("user.name", nom), ("user.email", email)):
            if _git(p, "config", cle, valeur, timeout=10)[0] != 0:
                return False, "git config %s a échoué." % cle
    return True, "Identité git enregistrée pour PROJETS."


# Ce que git répond quand GitHub ne sait pas qui pousse. Sous Windows, Git
# Credential Manager ouvre sa fenêtre ; sur un Raspberry Pi ou sous Termux, il
# n'y en a pas, et GIT_TERMINAL_PROMPT=0 interdit la question en console.
# Dépôt privé : sans compte autorisé, GitHub répond « Repository not found ».
REFUS_AUTH = ("could not read username", "could not read password",
              "terminal prompts disabled", "authentication failed",
              "invalid username or password", "permission denied",
              "the requested url returned error: 403", "error: 403",
              "repository not found")
CONNEXION = ("relancez le lanceur dans son terminal : il propose de s'y connecter, une "
             "seule fois (depuis web_launcher : action « Connecter ce serveur à GitHub » ; "
             "à la main : « gh auth login » puis « gh auth setup-git »).")
ACCES_REFUSE = ("[!] GitHub refuse l'accès à WEB_SUITE_PROJETS (dépôt privé) : ce poste n'est "
                "pas encore connecté à votre compte GitHub, les projets ne sont pas "
                "synchronisés. Pour le connecter, " + CONNEXION)


def refus_auth(sortie):
    return any(m in sortie.lower() for m in REFUS_AUTH)


def echec_push(sortie):
    """Le message d'un push refusé : si c'est l'authentification, dire quoi faire."""
    if refus_auth(sortie):
        return ("[!] Enregistré sur ce serveur, mais GitHub refuse l'envoi : le serveur "
                "n'est pas encore connecté à votre compte GitHub. Pour le connecter, "
                + CONNEXION + " Rien n'est perdu : l'envoi repartira tout seul une fois connecté.")
    return ("[!] Enregistré sur ce poste ; GitHub injoignable pour l'instant (pas "
            "d'Internet ?). Rien n'est perdu : l'envoi repart tout seul dès que la "
            "connexion revient, et au prochain démarrage. (%s)" % _cause(sortie))


def _cause(sortie):
    """La ligne de git qui dit pourquoi : « fatal: ... », sinon la dernière."""
    lignes = [l.strip() for l in sortie.splitlines() if l.strip()] or ["?"]
    return next((l for l in lignes if l.startswith("fatal:")), lignes[-1])


def commits_a_pousser(racine):
    """Commits de PROJETS pas encore sur GitHub : les envois faits hors ligne.

    --not --remotes=origin : marche aussi avant le tout premier push (pas d'@{u})."""
    p = dossier(racine)
    if not os.path.isdir(os.path.join(p, ".git")):
        return 0
    code, sortie = _git(p, "rev-list", "--count", "HEAD", "--not", "--remotes=origin", timeout=20)
    return int(sortie) if code == 0 and sortie.isdigit() else 0


def pousser_en_attente(racine, delai=120):
    """Renvoie à GitHub ce qui a été enregistré hors ligne (pull --rebase + push).

    Rien n'est commité ici : on ne pousse que ce qu'un envoi a déjà enregistré,
    avec son message. Renvoie None si rien n'attend, sinon (ok, message).
    Délai plus court qu'un envoi demandé : un Wi-Fi sans Internet peut laisser
    une connexion pendre, et l'essai suivant viendra de toute façon."""
    p = dossier(racine)
    with _verrou:
        n = commits_a_pousser(racine)
        if not n:
            return None
        tire = _tirer(p)
        if tire.startswith("[X]"):
            return False, tire
        code, sortie = _git(p, "push", "-u", "origin", BRANCHE, timeout=delai)
        if code != 0:
            return False, echec_push(sortie)
        return True, ("Revenu en ligne : %d enregistrement(s) fait(s) hors ligne envoyé(s) "
                      "sur GitHub." % n)


def envoyer(racine, oid, message):
    """commit + pull --rebase + push des dossiers de l'outil. Renvoie (ok, message)."""
    p = dossier(racine)
    with _verrou:
        if not os.path.isdir(os.path.join(p, ".git")):
            return False, "PROJETS n'est pas un dépôt git."
        ecrire_liste(p)
        # Un chemin absent ferait échouer tout l'add (« pathspec did not match »).
        racine_fichiers = [f for f in ("projets.txt", ".gitignore", ".gitattributes")
                           if os.path.exists(os.path.join(p, f))]
        _git(p, "add", "-A", "--", *DOSSIERS[oid], *racine_fichiers)
        if _git(p, "diff", "--cached", "--quiet")[0] != 0:
            code, sortie = _git(p, "commit", "-m", message.strip() or "Session " + oid)
            if code != 0:
                return False, "[X] git commit a échoué : %s" % sortie[-300:]
        tire = _tirer(p)
        if tire.startswith("[X]"):
            return False, tire
        code, sortie = _git(p, "push", "-u", "origin", BRANCHE, timeout=600)
        if code != 0:
            return False, echec_push(sortie)
        return True, "Envoyé sur GitHub."


if __name__ == "__main__":
    # Vérification : un dépôt nu joue GitHub, deux racines jouent deux PC.
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        # Poste neuf : aucune identité git, ni globale ni système.
        vide = os.path.join(tmp, "gitconfig-vide")
        open(vide, "w").close()
        os.environ.update(GIT_CONFIG_GLOBAL=vide, GIT_CONFIG_NOSYSTEM="1")
        nu = os.path.join(tmp, "github.git")
        _git(tmp, "init", "--bare", nu)
        a, b = os.path.join(tmp, "pc_a"), os.path.join(tmp, "pc_b")
        os.makedirs(a), os.makedirs(b)
        print(preparer(a, nu))                         # dépôt vide : départ en local
        os.makedirs(os.path.join(dossier(a), "CAO", "carte_alim"))
        open(os.path.join(dossier(a), "CAO", "carte_alim", "projet.cao.json"), "w").close()
        assert en_attente(a, "web_cao")
        assert identite_manquante(a)
        assert not envoyer(a, "web_cao", "sans identité")[0]
        assert not definir_identite(a, "test", "pas-un-mail")[0]
        assert definir_identite(a, " test ", "test@example.com")[0]
        assert not identite_manquante(a)
        ok, msg = envoyer(a, "web_cao", "nouveau projet")
        assert ok, msg
        assert not en_attente(a, "web_cao")
        print(preparer(b, nu))                         # l'autre PC clone
        assert os.path.isfile(os.path.join(dossier(b), "CAO", "carte_alim", "projet.cao.json"))
        with open(os.path.join(dossier(b), "projets.txt"), encoding="utf-8") as f:
            assert "[WEB_CAO]\ncarte_alim\n" in f.read()
        assert os.path.isdir(os.path.join(dossier(b), "3D"))
        assert os.path.isfile(os.path.join(dossier(a), ".gitattributes"))
        # Les zip de la LIB ne partent pas, leur contenu déballé si.
        os.makedirs(os.path.join(dossier(a), "LIB_CAO", "pack"))
        for nom in ("pack.zip", "pack/m.mod"):
            open(os.path.join(dossier(a), "LIB_CAO", nom), "w").close()
        assert envoyer(a, "web_cao", "pack")[0]
        suivis = _git(dossier(a), "ls-files", "LIB_CAO")[1]
        assert "pack/m.mod" in suivis and ".zip" not in suivis, suivis
        # Envoi hors ligne : rien à commiter ensuite, mais un commit à pousser.
        fa = os.path.join(dossier(a), "CAO", "carte_alim", "projet.cao.json")
        fb = os.path.join(dossier(b), "CAO", "carte_alim", "projet.cao.json")
        assert definir_identite(b, "b", "b@example.com")[0]
        _git(dossier(b), "remote", "set-url", "origin", os.path.join(tmp, "absent.git"))
        with open(fb, "w") as f:
            f.write("b")
        ok, msg = envoyer(b, "web_cao", "hors ligne")
        assert not ok and msg.startswith("[!]"), msg
        assert not _git(dossier(b), "status", "--porcelain", "--", "CAO", "LIB_CAO")[1]
        assert en_attente(b, "web_cao")
        _git(dossier(b), "remote", "set-url", "origin", nu)
        # Conflit : détecté, et PROJETS n'est pas laissé en plein rebase.
        with open(fa, "w") as f:
            f.write("a")
        assert envoyer(a, "web_cao", "a")[0]
        ok, msg = envoyer(b, "web_cao", "conflit")
        assert not ok and msg.startswith("[X] Conflit"), msg
        assert not os.path.isdir(os.path.join(dossier(b), ".git", "rebase-merge"))
        # PROJETS rempli sans .git : cloné à côté, le local ajouté, l'ancien gardé.
        c = os.path.join(tmp, "pc_c")
        os.makedirs(os.path.join(dossier(c), "CAO", "perso"))
        open(os.path.join(dossier(c), "CAO", "perso", "p.json"), "w").close()
        print(preparer(c, nu))
        assert os.path.isdir(os.path.join(dossier(c), ".git"))
        assert os.path.isfile(os.path.join(dossier(c), "CAO", "perso", "p.json"))
        with open(os.path.join(dossier(c), "CAO", "carte_alim", "projet.cao.json")) as f:
            assert f.read() == "a"
        assert any(n.startswith("PROJETS.avant-git-") for n in os.listdir(c))
        # Hors ligne puis retour d'Internet : ce qui attend repart tout seul.
        assert definir_identite(c, "c", "c@example.com")[0]
        assert pousser_en_attente(c) is None                 # rien n'attend
        with open(os.path.join(dossier(c), "CAO", "perso", "p.json"), "w") as f:
            f.write("hors ligne")
        _git(dossier(c), "remote", "set-url", "origin", os.path.join(tmp, "absent.git"))
        ok, msg = envoyer(c, "web_cao", "dans le train")
        assert not ok and "repart tout seul" in msg, msg
        assert commits_a_pousser(c) >= 1
        ok, msg = pousser_en_attente(c)                      # toujours hors ligne
        assert not ok and commits_a_pousser(c) >= 1, msg
        _git(dossier(c), "remote", "set-url", "origin", nu)   # Internet revient
        ok, msg = pousser_en_attente(c)
        assert ok and "hors ligne" in msg, msg
        assert commits_a_pousser(c) == 0 and not en_attente(c, "web_cao")
        assert "dans le train" in _git(nu, "log", "-1", "--format=%s", BRANCHE)[1]
        assert pousser_en_attente(c) is None
        # Dépôt privé, poste pas connecté à GitHub : pas de dépôt local sans lien
        # avec GitHub, juste les dossiers ; le clone se fait une fois connecté.
        vrai_git = _git

        def git_refuse(cwd, *args, **kw):
            if args[0] in ("clone", "pull", "push"):
                return 128, "fatal: could not read Username for 'https://github.com': terminal prompts disabled"
            return vrai_git(cwd, *args, **kw)
        globals()["_git"] = git_refuse
        d = os.path.join(tmp, "pc_d")
        assert preparer(d, nu) == ACCES_REFUSE
        assert not os.path.isdir(os.path.join(dossier(d), ".git"))
        assert os.path.isdir(os.path.join(dossier(d), "CAO")) and _vide(dossier(d))
        assert tirer(d) == ACCES_REFUSE                      # et pas un pull dans un dépôt parent
        assert not en_attente(d, "web_cao") and commits_a_pousser(d) == 0
        assert not envoyer(d, "web_cao", "x")[0]
        assert preparer(d, nu) == ACCES_REFUSE               # redémarrage, toujours pas connecté
        assert tirer(a) == ACCES_REFUSE                      # PROJETS déjà cloné avant le passage en privé
        ok, msg = envoyer(a, "web_cao", "refusé")
        assert not ok and "gh auth login" in msg, msg
        globals()["_git"] = vrai_git                         # le poste est connecté
        print(preparer(d, nu))
        assert os.path.isfile(os.path.join(dossier(d), "CAO", "carte_alim", "projet.cao.json"))
        assert not any(n.startswith("PROJETS.avant-git-") for n in os.listdir(d))
    # Push refusé faute d'identifiants (Pi, Termux) : le message dit quoi faire.
    assert "gh auth login" in echec_push(
        "fatal: could not read Username for 'https://github.com': terminal prompts disabled")
    assert "gh auth login" in echec_push("remote: Permission to x.git denied.\n"
                                         "fatal: unable to access '...': The requested URL returned error: 403")
    assert "gh auth login" in echec_push("remote: Repository not found.\n"
                                         "fatal: repository 'https://github.com/x/y.git/' not found")
    assert "gh auth" not in echec_push("fatal: unable to access '...': Could not resolve host: github.com")
    assert "Could not resolve host" in echec_push(
        "fatal: unable to access 'https://github.com/x.git/': Could not resolve host: github.com\n")
    assert "fatal: '/x' does not appear" in echec_push(
        "fatal: '/x' does not appear to be a git repository\nfatal: Could not read from remote "
        "repository.\n\nPlease make sure you have the correct access rights\nand the repository exists.")
    assert "--projets-reseau" in arguments("web_cao", "x", reseau=True)
    assert "--projets-reseau" not in arguments("web_cao", "x")
    print("OK")
