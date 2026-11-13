"""Banc de l'installation des dépendances des outils.

    python lanceur/banc_dependances.py

pip n'est lancé que s'il manque un paquet (sur Debian, il refuse la Python du
système même quand apt a tout installé) ; un lancement à qui il en manque un
lance l'installation ; openEMS ne pose aucune question hors Windows. Rien n'est
installé : les exécutables Rust (installer, web_suite) sont pilotés en boîte
noire, et la Python qu'ils lancent (WEB_SUITE_PYTHON) est une fausse, qui note
ses arguments, dit quels modules manquent et fait échouer pip. Affiche OK si
tout passe.
"""
import json
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from banc_lanceur import Poste  # noqa: E402

REQ = """\
# commentaire
numpy>=1.24.0
scipy>=1.10.0   # solveur DC
NumPy==2.0      # doublon
"""

FAUSSE_PYTHON = """\
import json, os, sys
a = sys.argv[1:]
with open(os.environ["FAUSSE_PYTHON_JOURNAL"], "a", encoding="utf-8") as f:
    f.write(json.dumps(a) + "\\n")
if a[:1] == ["-c"] and a[1].startswith("import "):
    sys.exit(1 if a[1].split()[1] in os.environ.get("FAUSSE_PYTHON_MANQUANTS", "").split(",") else 0)
sys.exit(1 if a[:3] == ["-m", "pip", "install"] else 0)
"""


def fausse_python(tmp):
    """-> (l'exécutable à donner dans WEB_SUITE_PYTHON, son journal)."""
    script = os.path.join(tmp, "fausse_python.py")
    with open(script, "w", encoding="utf-8") as f:
        f.write(FAUSSE_PYTHON)
    if os.name == "nt":
        exe = os.path.join(tmp, "fausse-python.cmd")
        with open(exe, "w", encoding="utf-8") as f:
            f.write('@"%s" "%s" %%*\n' % (sys.executable, script))
    else:
        exe = os.path.join(tmp, "fausse-python")
        with open(exe, "w", encoding="utf-8") as f:
            f.write('#!/bin/sh\nexec "%s" "%s" "$@"\n' % (sys.executable, script))
        os.chmod(exe, 0o755)
    return exe, os.path.join(tmp, "fausse_python.log")


def lancees(journal):
    """Les commandes reçues par la fausse Python, puis le journal est vidé."""
    try:
        with open(journal, encoding="utf-8") as f:
            cmds = [json.loads(ligne) for ligne in f]
    except OSError:
        return []
    os.remove(journal)
    return cmds


def main():
    with Poste() as poste:
        tmp = poste.racine
        exe, journal = fausse_python(poste.tmp)
        env = dict(WEB_SUITE_PYTHON=exe, FAUSSE_PYTHON_JOURNAL=journal)

        # 1. Rien à faire si tout est là, et un échec de pip est dit, avec la commande apt.
        os.makedirs(os.path.join(tmp, "WEB_CAO"))
        with open(os.path.join(tmp, "WEB_CAO", "requirements.txt"), "w", encoding="utf-8") as f:
            f.write(REQ)
        with open(os.path.join(tmp, "WEB_CAO", "web_CAO.py"), "w", encoding="utf-8") as f:
            f.write("")
        code, sortie = poste.lancer("installer", "cao", "--racine", tmp, **env)
        cmds = lancees(journal)
        assert code == 0 and "Dépendances déjà présentes" in sortie, sortie
        assert cmds == [["-c", "import numpy"], ["-c", "import scipy"]], cmds   # doublon écarté, pip pas lancé
        code, sortie = poste.lancer("installer", "cao", "--racine", tmp, FAUSSE_PYTHON_MANQUANTS="scipy", **env)
        cmds = lancees(journal)
        lignes = sortie.splitlines()
        assert code == 1 and any(c[:3] == ["-m", "pip", "install"] for c in cmds), (code, cmds)
        assert any("ne démarrera pas" in l and "scipy" in l and "numpy" not in l for l in lignes), lignes
        if os.name != "nt":
            assert any("apt install python3-scipy" in l for l in lignes), lignes
        code, sortie = poste.lancer("installer", "cao", "--racine", tmp, "--sans-dependances",
                                    FAUSSE_PYTHON_MANQUANTS="scipy", **env)
        assert code == 0 and lancees(journal) == [], sortie

        # 2. Lancer un outil à qui il manque un paquet : pas de serveur, l'installation part.
        with poste.lanceur("--local", FAUSSE_PYTHON_MANQUANTS="scipy", **env) as l:
            def cao():
                return json.loads(l.req("/api/etat").corps)["outils"][0]
            r = l.req("/api/lancer", "POST", entetes={"X-WebSuite": "1"}, corps=b'{"id":"web_cao"}')
            assert r.status == 200, r.corps
            for _ in range(200):
                if cao()["phase"] not in ("installation", "maj"):
                    break
                time.sleep(0.05)
            cmds = [c for c in lancees(journal) if c[:1] != ["-c"]]
            assert cmds and cmds[0][:3] == ["-m", "pip", "install"], cmds
            e = cao()
            assert e["id"] == "web_cao" and e["phase"] == "erreur" and e["url"] is None, e

        # 3. Hors Windows, openEMS ne s'installe pas tout seul : aucune question à poser.
        if os.name != "nt":
            os.makedirs(os.path.join(tmp, "WEB_ANTENNA"))
            with open(os.path.join(tmp, "WEB_ANTENNA", "web_antenna.py"), "w", encoding="utf-8") as f:
                f.write("")
            code, sortie = poste.lancer("installer", "antenna", "--racine", tmp, **env)
            assert code == 0 and "que sous Windows" in sortie, sortie
            assert "Installer openEMS maintenant" not in sortie, "question openEMS posée hors Windows"

    print("OK")


if __name__ == "__main__":
    main()
