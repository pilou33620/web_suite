"""Banc de l'installation des dépendances des outils.

    python lanceur/banc_dependances.py

pip n'est lancé que s'il manque un paquet (sur Debian, il refuse la Python du
système même quand apt a tout installé) ; un lancement à qui il en manque un
lance l'installation ; openEMS ne pose aucune question hors Windows. Rien n'est
installé : pip et l'installateur sont remplacés. Affiche OK si tout passe.
"""
import contextlib
import io
import os
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import installer  # noqa: E402
import outils  # noqa: E402

REQ = """\
# commentaire
numpy>=1.24.0
scipy>=1.10.0   # solveur DC
NumPy==2.0      # doublon
"""


def main():
    # 1. Rien à faire si tout est là, et un échec de pip est dit, avec la commande apt.
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

        # 2. Lancer un outil à qui il manque un paquet : pas de serveur, l'installation part.
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

    # 3. Hors Windows, openEMS ne s'installe pas tout seul : aucune question à poser.
    outils.installer = lambda o, racine, ecrire: True
    outils.openems_pret = lambda o, racine: None

    def question(_):
        raise AssertionError("question openEMS posée hors Windows")
    installer.oui = question
    with contextlib.redirect_stdout(io.StringIO()) as sortie:
        assert installer.installer(["web_antenna"], "/inexistant")
    assert "que sous Windows" in sortie.getvalue(), sortie.getvalue()

    print("OK")


if __name__ == "__main__":
    main()
