"""Vérifie l'état git de plusieurs projets et propose de les mettre à jour.

Usage : python maj_git.py
"""
import subprocess
import sys
from pathlib import Path

ICI = Path(__file__).resolve().parent

# Projets à vérifier (chemins absolus ou relatifs à ce script)
PROJETS = [
    ICI,
    "WEB_CAO",
    "WEB_3D",
    "WEB_ANTENNA",
]


def git(repo, *args):
    r = subprocess.run(["git", *args], cwd=repo, capture_output=True, text=True, encoding="utf-8", errors="replace")
    return r.returncode, (r.stdout + r.stderr).strip()


def etat(repo):
    """Retourne (modifs locales, commits à pousser, commits à tirer, a un upstream)."""
    git(repo, "fetch", "--quiet")
    _, out = git(repo, "status", "--porcelain", "-b")
    lignes = out.splitlines()
    entete, modifs = lignes[0], lignes[1:]
    upstream = "..." in entete
    avance = retard = 0
    if upstream:
        code, cpt = git(repo, "rev-list", "--left-right", "--count", "HEAD...@{u}")
        if code == 0:
            avance, retard = map(int, cpt.split())
    return modifs, avance, retard, upstream


def oui(question):
    return input(f"  {question} [o/N] ").strip().lower() in ("o", "oui", "y")


def mettre_a_jour(repo, modifs, avance, retard, upstream):
    if modifs:
        print("  Modifications locales :")
        for m in modifs:
            print("    " + m)
        if oui("Commiter ces modifications ?"):
            msg = input("  Message de commit : ").strip() or "maj"
            git(repo, "add", "-A")
            code, out = git(repo, "commit", "-m", msg)
            print("  " + out.splitlines()[0] if out else "")
            if code != 0:
                return
            avance += 1
    if not upstream:
        print("  Pas de branche distante suivie, pull/push ignorés.")
        return
    if retard and oui(f"Récupérer {retard} commit(s) distant(s) (pull --rebase) ?"):
        code, out = git(repo, "pull", "--rebase", "--autostash")
        print("  " + out.replace("\n", "\n  "))
        if code != 0:
            print("  !! Pull échoué (conflit ?), à régler à la main dans ce dossier.")
            return
    if avance and oui(f"Pousser {avance} commit(s) local(aux) (push) ?"):
        code, out = git(repo, "push")
        print("  " + out.replace("\n", "\n  "))


def main():
    a_faire = []
    print("=== Vérification des projets ===")
    for p in PROJETS:
        repo = (ICI / p).resolve()
        if not (repo / ".git").exists():
            print(f"[??] {repo} : pas un dépôt git")
            continue
        modifs, avance, retard, upstream = etat(repo)
        infos = []
        if modifs:
            infos.append(f"{len(modifs)} fichier(s) modifié(s)")
        if avance:
            infos.append(f"{avance} commit(s) à pousser")
        if retard:
            infos.append(f"{retard} commit(s) à récupérer")
        if infos:
            print(f"[!!] {repo.name} : " + ", ".join(infos))
            a_faire.append((repo, modifs, avance, retard, upstream))
        else:
            print(f"[OK] {repo.name}" + ("" if upstream else " (pas de distant)"))

    if not a_faire:
        print("\nTout est à jour.")
    else:
        print(f"\n{len(a_faire)} projet(s) à mettre à jour.")
        for repo, *e in a_faire:
            print(f"\n--- {repo.name} ({repo}) ---")
            if oui("Mettre à jour ce projet ?"):
                mettre_a_jour(repo, *e)
    if sys.stdin.isatty():
        input("\nEntrée pour quitter...")


if __name__ == "__main__":
    main()
