"""Installation des outils WEB·SUITE : demande lequel télécharger (ou les trois).

    python lanceur/installer.py      menu interactif
    python lanceur/installer.py cao 3d   installe directement WEB_CAO et WEB_3D
    python lanceur/installer.py tous     installe les trois

Chaque outil est cloné depuis GitHub à la racine de ce dépôt
(WEB_CAO/, WEB_ANTENNA/, WEB_3D/, ignorés par git), et garde son propre dépôt
git : il se met à jour seul à
chaque démarrage.
"""

import argparse
import sys

import outils

ALIAS = {
    "1": "web_cao", "cao": "web_cao", "web_cao": "web_cao",
    "2": "web_antenna", "antenna": "web_antenna", "antenne": "web_antenna",
    "web_antenna": "web_antenna",
    "3": "web_3d", "3d": "web_3d", "web_3d": "web_3d",
}
TOUS = {"4", "t", "tous", "tout", "all", "les3", "*"}


def _console_utf8():
    for flux in (sys.stdout, sys.stderr):
        try:
            flux.reconfigure(errors="replace")
        except (AttributeError, ValueError):
            pass


def interpreter(reponse):
    """'1,3' / '1 3' / 'tous' -> liste d'ids, ou None si la saisie est invalide."""
    jetons = [j for j in reponse.lower().replace(",", " ").replace(";", " ").split() if j]
    if not jetons or jetons == ["0"] or jetons == ["q"]:
        return []
    if any(j in TOUS for j in jetons):
        return [o["id"] for o in outils.CATALOGUE]
    ids = []
    for j in jetons:
        if j not in ALIAS:
            return None
        if ALIAS[j] not in ids:
            ids.append(ALIAS[j])
    return ids


def menu(racine):
    print()
    print("  WEB·SUITE — installation des outils")
    print("  " + "-" * 56)
    print("  Dossier : %s" % racine)
    if not outils.git_disponible():
        print("  [!] git introuvable : les outils seront téléchargés en zip et")
        print("      ne pourront pas se mettre à jour seuls. %s"
              % ("pkg install git" if outils.TERMUX else "https://git-scm.com"))
    print()
    for i, o in enumerate(outils.CATALOGUE, 1):
        etat = "installé" if outils.est_installe(o, racine) else "à télécharger"
        print("   %d  %-12s  %-30s [%s]" % (i, o["nom"], o["sous_titre"], etat))
    print("   4  Les trois")
    print("   0  Rien, quitter")
    print()
    while True:
        try:
            reponse = input("  Quel(s) outil(s) télécharger ? (ex. 1, 1 3 ou 4) : ")
        except (EOFError, KeyboardInterrupt):
            print()
            return []
        ids = interpreter(reponse)
        if ids is not None:
            return ids
        print("  Réponse non comprise : tapez 1, 2, 3, 4, ou plusieurs numéros.")


def oui(question):
    try:
        return input("  %s [o/N] : " % question).strip().lower() in ("o", "oui", "y", "yes")
    except (EOFError, KeyboardInterrupt):
        print()
        return False


def installer(ids, racine, demander_dependances=True):
    installes = []
    for oid in ids:
        o = outils.PAR_ID[oid]
        print()
        print("  == %s ==" % o["nom"])
        if outils.installer(o, racine, lambda t: print("  " + t)):
            installes.append(o)

    for o in installes:
        if o["dependances"] == "pip" and demander_dependances:
            print()
            via = " par pkg (Termux)" if outils.TERMUX else ""
            if oui("Installer les dépendances Python facultatives de %s "
                   "(numpy, scipy : solveurs avancés)%s ?" % (o["nom"], via)):
                outils.installer_dependances(o, racine, lambda t: print("  " + t))
        elif o["dependances"] == "openems" and not outils.openems_pret(o, racine):
            print()
            print("  %s démarre sans rien d'autre, mais ses simulations demandent openEMS." % o["nom"])
            if demander_dependances and oui(
                    "Installer openEMS maintenant (~50 Mo, Python 3.10 ou 3.11 requis) ?"):
                outils.installer_openems(o, racine, lambda t: print("  " + t))
            else:
                print("  Plus tard : python lanceur/installer.py antenna")

    echecs = len(ids) - len(installes)
    print()
    if ids:
        print("  Terminé : %d outil(s) prêt(s)%s." % (
            len(installes), ", %d échec(s)" % echecs if echecs else ""))
    return echecs == 0


def main(argv=None):
    _console_utf8()
    ap = argparse.ArgumentParser(description="Télécharge un, plusieurs ou les trois outils WEB·SUITE.")
    ap.add_argument("outils", nargs="*",
                    help="cao, antenna, 3d, ou tous (sans argument : menu interactif)")
    ap.add_argument("--racine", default=outils.RACINE_DEFAUT,
                    help="dossier où cloner les outils (défaut : le dossier de WEB_SUITE)")
    ap.add_argument("--sans-dependances", action="store_true",
                    help="ne pas proposer l'installation des dépendances (pip, openEMS)")
    args = ap.parse_args(argv)

    if args.outils:
        ids = interpreter(" ".join(args.outils))
        if ids is None:
            ap.error("outil inconnu : choisir parmi cao, antenna, 3d, tous")
    else:
        ids = menu(args.racine)
    ok = installer(ids, args.racine, not args.sans_dependances)
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
