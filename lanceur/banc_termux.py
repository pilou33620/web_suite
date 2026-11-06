"""Banc de l'installation des dépendances sous Termux (Android).

    python lanceur/banc_termux.py

Sous Termux, pip n'a pas de roue numpy/scipy et tente de les compiler (cmake ->
ninja -> numpy), ce qui échoue. L'installateur doit passer par pkg. Ce banc joue
Termux sur n'importe quel poste : un faux `pkg` dans le PATH note ses arguments,
rien n'est installé. Affiche OK si tout passe.
"""
import contextlib
import io
import os
import stat
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import installer  # noqa: E402
import outils  # noqa: E402
import web_suite  # noqa: E402

REQ = """\
# commentaire
numpy>=1.24.0
scipy>=1.10.0   # solveur DC
NumPy==2.0      # doublon : un seul paquet pkg
"""


def main():
    with tempfile.TemporaryDirectory() as tmp:
        # 1. Lecture et répartition : tout part à pkg, rien à pip.
        req = os.path.join(tmp, "requirements.txt")
        with open(req, "w", encoding="utf-8") as f:
            f.write(REQ)
        paquets, reste = outils.repartir_termux(outils.lire_requirements(req))
        assert paquets == ["python-numpy", "python-scipy"], paquets
        assert reste == [], reste

        # Un paquet pur Python inconnu reste à pip.
        with open(req, "a", encoding="utf-8") as f:
            f.write("requests>=2\n")
        paquets, reste = outils.repartir_termux(outils.lire_requirements(req))
        assert reste == ["requests>=2"], reste

        # 2. Installation complète avec un faux pkg et une fausse vérification.
        with open(req, "w", encoding="utf-8") as f:
            f.write(REQ)
        os.makedirs(os.path.join(tmp, "WEB_CAO"))
        os.replace(req, os.path.join(tmp, "WEB_CAO", "requirements.txt"))
        bin_ = os.path.join(tmp, "bin")
        os.makedirs(bin_)
        trace = os.path.join(tmp, "pkg.log")
        # Sous Windows, un script shell ne s'exécute pas (WinError 193) : le
        # faux pkg y est un .cmd, que shutil.which trouve aussi.
        if os.name == "nt":
            faux = os.path.join(bin_, "pkg.cmd")
            contenu = '@echo %%*> "%s"\r\n@echo faux pkg : %%*\r\n' % trace
        else:
            faux = os.path.join(bin_, "pkg")
            contenu = '#!/bin/sh\necho "$@" > "%s"\necho "faux pkg : $@"\n' % trace
        with open(faux, "w") as f:
            f.write(contenu)
        os.chmod(faux, os.stat(faux).st_mode | stat.S_IEXEC)
        os.environ["PATH"] = bin_ + os.pathsep + os.environ.get("PATH", "")

        outils.TERMUX = True
        outils._module_present = lambda nom: True
        lignes = []
        ok = outils.installer_dependances(outils.PAR_ID["web_cao"], tmp, lignes.append)
        assert ok, lignes
        with open(trace) as f:
            assert f.read().split() == ["install", "-y", "python-numpy", "python-scipy"]
        assert not any("pip install" in l for l in lignes), lignes

        # 3. pkg réussit mais numpy reste introuvable : échec signalé, pas de faux OK.
        outils._module_present = lambda nom: nom != "numpy"
        lignes = []
        assert not outils.installer_dependances(outils.PAR_ID["web_cao"], tmp, lignes.append)
        assert any("numpy" in l and "introuvable" in l for l in lignes), lignes

    # 4. Hors Termux : rien à faire si tout est là (pip refuserait la Python de Debian),
    #    et un échec de pip est dit, avec la commande apt.
    outils.TERMUX = False
    with tempfile.TemporaryDirectory() as tmp:
        os.makedirs(os.path.join(tmp, "WEB_CAO"))
        with open(os.path.join(tmp, "WEB_CAO", "requirements.txt"), "w", encoding="utf-8") as f:
            f.write(REQ)
        cao = outils.PAR_ID["web_cao"]
        lancees = []
        outils._executer = lambda cmd, ecrire, cwd=None: lancees.append(cmd) or 1
        outils._module_present = lambda nom: True
        assert outils.modules_manquants(cao, tmp) == []
        assert outils.installer_dependances(cao, tmp, lambda t: None) and not lancees
        outils._module_present = lambda nom: nom != "scipy"
        assert outils.modules_manquants(cao, tmp) == ["scipy"]
        lignes = []
        assert not outils.installer_dependances(cao, tmp, lignes.append) and lancees
        assert any("ne démarrera pas" in l and "scipy" in l for l in lignes), lignes
        if os.name != "nt":
            assert any("apt install python3-scipy" in l for l in lignes), lignes

        # Lancer un outil à qui il manque un paquet : pas de serveur, l'installation part.
        with open(os.path.join(tmp, "WEB_CAO", "web_CAO.py"), "w", encoding="utf-8") as f:
            f.write("")
        outils.git_disponible = lambda: False
        g = outils.Gestionnaire(tmp)
        for evt in g.maj_finie.values():
            evt.set()
        del lancees[:]
        assert g.lancer("web_cao") is False
        for _ in range(100):
            if g.etats["web_cao"]["phase"] != "installation":
                break
            time.sleep(0.05)
        assert lancees and lancees[0][1:4] == ["-m", "pip", "install"], lancees
        assert g.etats["web_cao"]["phase"] == "erreur", g.etats["web_cao"]

    # 5. Hors Windows, openEMS ne s'installe pas tout seul : aucune question à poser.
    outils.installer = lambda o, racine, ecrire: True
    outils.openems_pret = lambda o, racine: None

    def question(_):
        raise AssertionError("question openEMS posée hors Windows")
    installer.oui = question
    with contextlib.redirect_stdout(io.StringIO()) as sortie:
        assert installer.installer(["web_antenna"], "/inexistant")
    assert "que sous Windows" in sortie.getvalue(), sortie.getvalue()

    # 6. Termux : le lanceur sert les autres appareils (réseau, pas de navigateur).
    assert web_suite.poste_serveur()

    print("OK")


if __name__ == "__main__":
    main()
