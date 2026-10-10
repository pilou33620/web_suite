"""Connexion à GitHub, une fois, dans le terminal du serveur (Termux, Raspberry Pi).

    python lanceur/connexion_github.py              vérification (banc d'essai, sans réseau)
    python lanceur/connexion_github.py --connecter  se connecter puis récupérer PROJETS

Sous Windows, Git Credential Manager ouvre sa fenêtre de connexion au premier
push. Sur un téléphone ou un Pi, il n'y en a pas : le push échouait, et l'on
ne l'apprenait que depuis la tablette, au premier envoi. Le lanceur pose donc
la question au démarrage, là où l'on a un clavier et où l'on voit la réponse :
git a-t-il déjà de quoi s'identifier auprès de github.com ? Sinon, il propose
de s'y connecter avec gh (installé au besoin), puis de brancher git dessus.

Au passage, le nom et l'e-mail des commits viennent du compte GitHub : la
tablette n'a plus à les demander au premier envoi.

WEB_SUITE_PROJETS est privé : sans cette connexion, PROJETS n'est même pas
récupéré. Lancé sans terminal (service, démarrage automatique), le lanceur ne
peut pas poser la question : --connecter la pose à la demande.
"""

import os
import shutil
import subprocess
import sys

import outils
import projets

# « ne plus demander » : dans le .git de PROJETS, jamais suivi ni envoyé
MARQUE = "websuite-sans-github"


def _marque(racine):
    return os.path.join(projets.dossier(racine), ".git", MARQUE)


def depot_github_https(racine):
    """PROJETS pousse-t-il vers github.com en HTTPS ? (en SSH, la clé s'en charge)"""
    p = projets.dossier(racine)
    if not os.path.isdir(os.path.join(p, ".git")):
        # Pas encore cloné (dépôt privé, poste pas connecté) : l'adresse par défaut.
        return projets.DEPOT.lower().startswith("https://github.com/")
    # l'adresse telle qu'écrite dans la config du dépôt (get-url appliquerait les insteadOf)
    code, url = projets._git(p, "config", "--get", "remote.origin.url", timeout=10)
    return code == 0 and url.strip().lower().startswith("https://github.com/")


def github_identifie():
    """git a-t-il des identifiants pour github.com ? Ne demande rien, n'envoie rien :
    `git credential fill` interroge les assistants configurés (gh, store...)."""
    try:
        r = subprocess.run(["git", "credential", "fill"], input="protocol=https\nhost=github.com\n\n",
                           capture_output=True, text=True, timeout=20,
                           env=dict(os.environ, **projets.ENV_GIT))
    except (OSError, subprocess.SubprocessError):
        return False
    return r.returncode == 0 and "\npassword=" in "\n" + r.stdout


def commande_installation_gh():
    if outils.TERMUX:
        return ["pkg", "install", "-y", "gh"]
    if shutil.which("apt-get"):
        return ["sudo", "apt-get", "install", "-y", "gh"]
    return None


def identite_depuis_gh(racine, sortie=None):
    """Nom et e-mail des commits pris au compte GitHub, s'ils manquent encore.
    E-mail privé : l'adresse « noreply » que GitHub attribue à chaque compte."""
    if not projets.identite_manquante(racine):
        return False
    if sortie is None:
        try:
            sortie = subprocess.run(
                ["gh", "api", "user", "--jq", '[.id, .login, .name // "", .email // ""] | map(tostring) | join("\\n")'],
                capture_output=True, text=True, timeout=30).stdout
        except (OSError, subprocess.SubprocessError):
            return False
    champs = (sortie or "").splitlines() + ["", "", "", ""]
    ident, login, nom, email = (c.strip() for c in champs[:4])
    if not login:
        return False
    email = email or "%s+%s@users.noreply.github.com" % (ident, login)
    return projets.definir_identite(racine, nom or login, email)[0]


def proposer(racine, demander=input, lancer=subprocess.call, ecrire=print,
             identifie=github_identifie, quel=shutil.which, nommer=identite_depuis_gh):
    """Au démarrage, terminal du serveur. Renvoie ce qui s'est passé :
    "deja", "ignore", "plus-tard", "jamais", "connecte", "echec"."""
    if os.name == "nt" or not depot_github_https(racine) or os.path.exists(_marque(racine)):
        return "ignore"
    if identifie():
        if quel("gh"):
            nommer(racine)
        return "deja"
    ecrire("")
    ecrire("  GitHub : ce serveur n'est pas encore connecté à votre compte.")
    ecrire("  Le dépôt des projets (WEB_SUITE_PROJETS) est privé : sans cela, ils ne sont")
    ecrire("  ni récupérés ni envoyés sur GitHub.")
    try:
        rep = demander("  Se connecter maintenant ? [O/n, j = ne plus demander] : ").strip().lower()
    except EOFError:
        return "plus-tard"
    if rep in ("j", "jamais"):
        if not os.path.isdir(os.path.dirname(_marque(racine))):
            ecrire("  PROJETS n'est pas encore récupéré : la question reviendra au prochain démarrage.")
            return "plus-tard"
        open(_marque(racine), "w").close()
        ecrire("  D'accord, plus de question. (Supprimer %s pour la retrouver.)" % _marque(racine))
        return "jamais"
    if rep not in ("", "o", "oui", "y", "yes"):
        ecrire("  Plus tard : la question reviendra au prochain démarrage.")
        return "plus-tard"
    if not quel("gh"):
        cmd = commande_installation_gh()
        if not cmd:
            ecrire("  [X] Installez gh (https://cli.github.com), puis relancez le lanceur.")
            return "echec"
        ecrire("  Installation de l'outil GitHub (gh) : %s" % " ".join(cmd))
        lancer(cmd)
        if not quel("gh"):
            ecrire("  [X] gh n'a pas pu être installé (pas d'Internet ?). Nouvel essai au prochain démarrage.")
            return "echec"
    ecrire("")
    ecrire("  gh va afficher un code à 8 caractères : saisissez-le sur")
    ecrire("  https://github.com/login/device (sur ce téléphone, la tablette ou un PC),")
    ecrire("  validez, et revenez ici.")
    env = None
    if not outils.TERMUX:
        # Sur un Pi sans bureau, gh ouvrirait un navigateur texte (lynx, w3m) dans ce
        # terminal et bloquerait tout : on lui donne un navigateur qui ne fait rien.
        # Sous Termux, il ouvre celui du téléphone, qui est justement le bon appareil.
        env = dict(os.environ, GH_BROWSER="true", BROWSER="true")
        ecrire("  Quand gh dit « Press Enter », appuyez sur Entrée : aucun navigateur ne")
        ecrire("  s'ouvre ici, gh attend simplement que le code soit validé.")
    ecrire("")
    # Pas de --skip-ssh-key : le gh de Raspberry Pi OS / Debian (2.23) ne le connaît
    # pas et s'arrête aussitôt ; en HTTPS, gh ne propose de toute façon aucune clé SSH.
    lancer(["gh", "auth", "login", "--hostname", "github.com", "--git-protocol", "https",
            "--web"], env=env)
    lancer(["gh", "auth", "setup-git", "--hostname", "github.com"])
    if not identifie():
        ecrire("  [X] Connexion à GitHub non aboutie. Nouvel essai au prochain démarrage.")
        return "echec"
    nommer(racine)
    ecrire("  [OK] Ce serveur est connecté à GitHub : les envois partiront.")
    return "connecte"


def connecter(racine):
    """--connecter : se connecter (même après « ne plus demander »), puis
    récupérer ou mettre à jour PROJETS."""
    if os.path.exists(_marque(racine)):
        os.remove(_marque(racine))
    r = proposer(racine)
    print("  Projets : %s" % projets.preparer(racine), flush=True)
    return r


if __name__ == "__main__" and "--connecter" in sys.argv:
    connecter(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    sys.exit(0)

if __name__ == "__main__":
    # Vérification sans réseau ni vrai gh : un dépôt PROJETS factice, des réponses
    # et des commandes simulées.
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        vide = os.path.join(tmp, "gitconfig-vide")
        open(vide, "w").close()
        os.environ.update(GIT_CONFIG_GLOBAL=vide, GIT_CONFIG_NOSYSTEM="1")
        p = projets.dossier(tmp)
        os.makedirs(p)
        # PROJETS pas encore cloné (dépôt privé) : la question est posée quand même,
        # et « ne plus demander » n'a nulle part où s'écrire.
        assert depot_github_https(tmp)
        assert proposer(tmp, demander=lambda q: "j", ecrire=lambda *a: None,
                        identifie=lambda: False) == "plus-tard"
        projets._git(p, "init")
        projets._git(p, "remote", "add", "origin", "https://github.com/x/WEB_SUITE_PROJETS.git")
        muet = lambda *a: None                                   # noqa: E731

        derniere_env = {}

        def essai(reponses, identifie_avant, identifie_apres=True, gh=True, gh_apres=True):
            etat = {"gh": gh, "connecte": False, "cmds": []}

            def lancer(cmd, env=None):
                etat["cmds"].append(cmd)
                if cmd[:3] in (["pkg", "install", "-y"], ["sudo", "apt-get", "install"]):
                    etat["gh"] = gh_apres
                if cmd[:3] == ["gh", "auth", "login"]:
                    etat["connecte"] = identifie_apres
                    derniere_env["env"] = env
                return 0
            r = proposer(tmp, demander=lambda q: reponses.pop(0), lancer=lancer, ecrire=muet,
                         identifie=lambda: identifie_avant or etat["connecte"],
                         quel=lambda n: "/bin/gh" if etat["gh"] else None,
                         nommer=lambda r: False)
            return r, etat["cmds"]

        assert github_identifie() is False                      # aucun assistant configuré
        assert essai([], True)[0] == "deja"                     # déjà connecté : aucune question
        r, cmds = essai([""], False)                            # Entrée = oui
        assert r == "connecte", r
        assert ["gh", "auth", "setup-git", "--hostname", "github.com"] in cmds
        assert any(c[:3] == ["gh", "auth", "login"] and "--web" in c for c in cmds)
        assert outils.TERMUX or derniere_env["env"]["GH_BROWSER"] == "true"   # pas de lynx qui bloque
        assert essai(["n"], False)[0] == "plus-tard"
        r, cmds = essai(["o"], False, gh=False)                 # gh absent : installé d'abord
        assert r == "connecte" and cmds[0][-1] == "gh", cmds
        assert essai(["o"], False, gh=False, gh_apres=False)[0] == "echec"
        assert essai(["o"], False, identifie_apres=False)[0] == "echec"
        assert essai(["j"], False)[0] == "jamais"
        assert essai([], False)[0] == "ignore"                  # plus jamais demandé
        os.remove(_marque(tmp))
        projets._git(p, "remote", "set-url", "origin", "git@github.com:x/y.git")
        assert essai([], False)[0] == "ignore"                  # SSH : la clé s'en charge
        # Nom et e-mail tirés du compte GitHub (e-mail privé : adresse noreply).
        assert projets.identite_manquante(tmp)
        assert identite_depuis_gh(tmp, sortie="123\npilou33620\n\n\n")
        assert projets._git(p, "config", "user.name")[1] == "pilou33620"
        assert projets._git(p, "config", "user.email")[1] == "123+pilou33620@users.noreply.github.com"
        assert not identite_depuis_gh(tmp, sortie="9\nautre\nAutre\na@b.fr\n")  # déjà fixée : on n'y touche pas
    print("OK")
