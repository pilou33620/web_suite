"""Banc du skill web-suite (module savoir-metier, lanceur/docs/SPEC-mcp-savoir-metier.md).

Ce que les fiches citent doit exister dans le serveur MCP : chaque outil (`cao_pcb_lire`) et
chaque argument à choix, cité « (quoi=liaisons) » après son outil (ou sous un en-tête de
tableau qui le nomme). Les fiches restent courtes, leurs liens et ancres mènent quelque part,
et rien de privé n'y entre (chemin personnel, clé, nom de projet) : ni dans les fiches, ni dans
la consigne MCP, ni dans le relevé des consignes Gemini, dont chaque source est relue à la
révision figée en tête du relevé.
    python3 lanceur/banc_savoir.py
"""
import os
import re
import subprocess
import sys

ICI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, ICI)
import mcp_web_suite  # noqa: E402

RACINE = os.path.dirname(ICI)
SKILL = os.path.join(RACINE, ".claude", "skills", "web-suite")
TRACE = os.path.join(RACINE, "lanceur", "docs", "savoir-trace.md")
MAX_SKILL, MAX_FICHE, MAX_INSTRUCTIONS = 150, 200, 1500
# toutes les règles ont leur destination : une nouvelle règle relevée en garde une
A_FAIRE_PERMIS = False

OUTIL = re.compile(r"`(etat|(?:cao|lib|antenna|web3d|pilote|pcbparts)_[a-z0-9_]+)`")
# un nom d'outil entre accents graves, ou une citation d'argument « (arg=valeur) »
CITATION = re.compile(r"`([a-z0-9_]+)`|\(([a-z_]+)=([^)\s]+)\)")
LIEN = re.compile(r"\]\(([^)\s]+)\)")
PRIVE = re.compile(r"/home/|/Users/|[A-Za-z]:\\|~/(?!\.claude/)|AIza[0-9A-Za-z_-]{20}|ghp_[0-9A-Za-z]{20}"
                   r"|github_pat_|sk-[a-z]*-?[0-9A-Za-z]{16}|AKIA[0-9A-Z]{16}|xox[bp]-")


def projets():
    """Les noms des projets de PROJETS (quand il est là) : ils n'ont rien à faire dans le skill."""
    noms = []
    for sous in ("CAO", "ANTENNA", "3D"):
        d = os.path.join(RACINE, "PROJETS", sous)
        if os.path.isdir(d):
            noms += [n for n in os.listdir(d) if len(n) > 3 and os.path.isdir(os.path.join(d, n))]
    return noms


def prive(nom, texte, noms_projets):
    m = PRIVE.search(texte)
    if m:
        return ["%s : chemin personnel ou clé : %s" % (nom, m.group(0))]
    return ["%s : nom de projet : %s" % (nom, p) for p in noms_projets if p.lower() in texte.lower()]


def ancres(texte):
    """Les ancres des titres d'une fiche, comme GitHub les fabrique."""
    return {re.sub(r"[^\w\- ]", "", t.strip().lower()).replace(" ", "-")
            for t in re.findall(r"^#+ (.+)$", texte, re.M)}


def enum_de(outils, outil, arg):
    return outils.get(outil, {}).get("inputSchema", {}).get("properties", {}).get(arg, {}).get("enum")


def controler(fiches, outils=None, dossier=SKILL, noms_projets=()):
    """fiches : {nom de fichier: texte}. Rend la liste des erreurs (vide si tout va)."""
    outils = mcp_web_suite.OUTILS_MCP if outils is None else outils
    erreurs = []
    if "SKILL.md" not in fiches:
        erreurs.append("SKILL.md manquant")
    for nom, texte in sorted(fiches.items()):
        lignes = texte.count("\n") + 1
        maxi = MAX_SKILL if nom == "SKILL.md" else MAX_FICHE
        if lignes > maxi:
            erreurs.append("%s : %d lignes (%d au plus)" % (nom, lignes, maxi))
        if nom == "SKILL.md":
            tete = re.match(r"---\nname: (.+)\ndescription: (.+)\n(?:.*\n)*?---\n", texte)
            if not tete or tete.group(1).strip() != "web-suite" or not tete.group(2).strip():
                erreurs.append("SKILL.md : en-tête « name: web-suite » et « description » attendu")
        for o in OUTIL.findall(texte):
            if o not in outils:
                erreurs.append("%s : outil inconnu du serveur MCP : %s" % (nom, o))
        # chaque « (arg=valeur) » vaut pour le dernier outil cité avant lui
        dernier = None
        for code, arg, val in CITATION.findall(texte):
            if code:
                if code in outils:
                    dernier = code
                continue
            enum = enum_de(outils, dernier, arg) if dernier else None
            if not enum or val not in enum:
                erreurs.append("%s : %s n'a pas de %s=%s" % (nom, dernier or "(aucun outil cité avant)", arg, val))
        for cible in LIEN.findall(texte):
            if re.match(r"[a-z]+:", cible):
                continue          # lien externe : pas vérifié ici
            fichier, _, ancre = cible.partition("#")
            if fichier and fichier not in fiches and not os.path.exists(os.path.normpath(os.path.join(dossier, fichier))):
                erreurs.append("%s : lien mort : %s" % (nom, cible))
            elif ancre and ancre not in ancres(fiches.get(fichier or nom, "")):
                erreurs.append("%s : ancre morte : %s" % (nom, cible))
        erreurs += prive(nom, texte, noms_projets)
    return erreurs


LIGNE_TRACE = re.compile(r"^\| ([A-Z]\d+) \| ([^|]+?):(\d+) \| (.+) \| (.+?) \|$")
REVISION = re.compile(r"`(WEB_[A-Z0-9]+)@([0-9a-f]{7,40})`")


def depot(outil):
    """Le dépôt d'un outil : dans web_suite sur le poste, à côté de lui en CI (cloné dans ../)."""
    for base in (RACINE, os.path.dirname(RACINE)):
        if os.path.isdir(os.path.join(base, outil, ".git")):
            return os.path.join(base, outil)
    return None


def source_figee(chemin, revisions):
    """Le texte d'une source à la révision figée du relevé ; None si l'outil n'est pas cloné.
    Un clone superficiel (CI) va chercher la révision au besoin."""
    outil, _, reste = chemin.partition("/")
    d = depot(outil)
    if not d:
        return None
    rev = revisions.get(outil)
    if not rev:
        raise ValueError("pas de révision figée pour %s en tête du relevé" % outil)
    for essai in range(2):
        r = subprocess.run(["git", "-C", d, "show", "%s:%s" % (rev, reste)], capture_output=True, text=True)
        if r.returncode == 0:
            return r.stdout
        if essai == 0:
            subprocess.run(["git", "-C", d, "fetch", "--quiet", "--depth", "1", "origin", rev], capture_output=True)
    raise ValueError("%s introuvable à la révision %s" % (chemin, rev))


def controler_trace(texte, fiches, outils=None, a_faire_permis=A_FAIRE_PERMIS, lire=None, noms_projets=()):
    """Le relevé des consignes Gemini : chaque ligne a une source qui existe et une destination valable."""
    outils = mcp_web_suite.OUTILS_MCP if outils is None else outils
    revisions = dict(REVISION.findall(texte))
    lire = lire or (lambda chemin: source_figee(chemin, revisions))
    erreurs, ids, a_faire, sources = [], set(), 0, {}
    lignes = [l for l in texte.split("\n") if re.match(r"^\|\s*[A-Z]+\d+", l)]
    if not lignes:
        erreurs.append("relevé vide")
    for l in lignes:
        m = LIGNE_TRACE.match(l)
        if not m:
            erreurs.append("relevé : ligne illisible : " + l[:60])
            continue
        ident, fichier, num, _regle, dest = m.groups()
        if ident in ids:
            erreurs.append("relevé : %s en double" % ident)
        ids.add(ident)
        try:
            if fichier not in sources:
                sources[fichier] = lire(fichier)
            src = sources[fichier]
            if src is not None and not 0 < int(num) <= src.count("\n") + 1:
                erreurs.append("relevé %s : %s n'a pas de ligne %s" % (ident, fichier, num))
            elif src is not None and not src.split("\n")[int(num) - 1].strip():
                erreurs.append("relevé %s : %s ligne %s est vide" % (ident, fichier, num))
        except ValueError as exc:
            erreurs.append("relevé %s : %s" % (ident, exc))
            sources[fichier] = None
        for o in OUTIL.findall(dest):
            if o not in outils:
                erreurs.append("relevé %s : outil inconnu : %s" % (ident, o))
        if dest == "à faire":
            a_faire += 1
            if not a_faire_permis:
                erreurs.append("relevé %s : encore « à faire »" % ident)
        elif dest.startswith("écartée : ") and len(dest) > 12:
            pass
        elif dest.startswith("outil : "):
            if dest[8:].strip() not in outils:
                erreurs.append("relevé %s : outil inconnu : %s" % (ident, dest[8:]))
        elif dest.startswith("généré : "):
            m2 = re.match(r"généré : ([a-z0-9_]+) \(([a-z_]+)=([a-z_]+)\)$", dest)
            if not m2 or m2.group(3) not in (enum_de(outils, m2.group(1), m2.group(2)) or ()):
                erreurs.append("relevé %s : lecture inconnue : %s" % (ident, dest))
        else:
            for f in re.split(r",\s*", dest):
                if f not in fiches:
                    erreurs.append("relevé %s : destination inconnue : %s" % (ident, f))
    erreurs += prive("savoir-trace.md", texte, noms_projets)
    return erreurs, a_faire


def lire_skill(dossier=SKILL):
    fiches = {}
    for f in sorted(os.listdir(dossier)):
        if f.endswith(".md"):
            with open(os.path.join(dossier, f), encoding="utf-8") as fd:
                fiches[f] = fd.read()
    return fiches


def essais():
    """Le banc lui-même : chaque faute connue doit être vue."""
    bon = ("---\nname: web-suite\ndescription: x\n---\n# Titre\nVoir `cao_pcb_lire` (quoi=liaisons) puis "
           "(quoi=zone), et [pcb](pcb.md#règles), et `etat`.\n| Question | `cao_calcul` |\n|---|---|\n| x | (analyse=dc) |\n")
    fiches = {"SKILL.md": bon, "pcb.md": "# Règles\nok"}
    assert controler(fiches) == [], controler(fiches)
    for faute, motif in ((bon + "`cao_inventer`", "outil inconnu"),
                         (bon + "`cao_pcb_lire` (quoi=tout)", "quoi=tout"),
                         (bon + "`cao_pcb_lire` x (quoi=Reglages)", "quoi=Reglages"),
                         (bon + "| y | (analyse=thermique) |", "analyse=thermique"),
                         ("---\nname: web-suite\ndescription: x\n---\n(quoi=nets)", "aucun outil cité"),
                         (bon + "x\n" * 160, "lignes"),
                         (bon + "[x](absent.md)", "lien mort"),
                         (bon + "[x](pcb.md#rien)", "ancre morte"),
                         (bon + "dans /home/pilou/x", "chemin personnel"),
                         (bon + "C:\\Users\\pilou\\x", "chemin personnel"),
                         (bon + "~/Documents", "chemin personnel"),
                         (bon + "github_pat_11ABC", "chemin personnel"),
                         (bon + "le projet Carte Secrète", "nom de projet"),
                         (bon.replace("web-suite", "web_suite"), "en-tête")):
        e = controler({"SKILL.md": faute, "pcb.md": "# Règles\nok"}, noms_projets=["carte secrète"])
        assert any(motif in x for x in e), (motif, e)
    assert controler({"SKILL.md": bon + "~/.claude/skills", "pcb.md": "# Règles"}) == []
    assert any("SKILL.md manquant" in x for x in controler({"pcb.md": "ok"}))

    src = lambda chemin: "ligne 1\n\nligne 3\n"
    l = "| C1 | WEB_CAO/x.js:1 | une règle | %s |"
    for dest in ("pcb.md", "à faire", "écartée : propre à Gemini", "outil : cao_drc",
                 "généré : cao_pcb_lire (quoi=liaisons)", "pcb.md, SKILL.md"):
        e = controler_trace(l % dest, fiches, a_faire_permis=True, lire=src)[0]
        assert e == [], (dest, e)
    for dest, motif in (("antenne.md", "destination inconnue"), ("outil : cao_rien", "outil inconnu"),
                        ("généré : cao_pcb_lire (quoi=tout)", "lecture inconnue"), ("écartée : ", "destination inconnue")):
        e = controler_trace(l % dest, fiches, lire=src)[0]
        assert any(motif in x for x in e), (dest, e)
    for ligne, motif in (("| C1 | WEB_CAO/x.js:9 | r | pcb.md |", "pas de ligne 9"),
                         ("| C1 | WEB_CAO/x.js:2 | r | pcb.md |", "ligne 2 est vide"),
                         ((l % "pcb.md") + "\n" + (l % "pcb.md"), "en double"),
                         ("|C26| WEB_CAO/x.js:1 | r | pcb.md |", "ligne illisible"),
                         (l % "à faire", "encore")):
        e = controler_trace(ligne, fiches, lire=src)[0]
        assert any(motif in x for x in e), (motif, e)
    if depot("WEB_CAO"):     # outil cloné : une source sans révision figée est une erreur
        assert any("révision" in x for x in controler_trace(l % "pcb.md", fiches)[0])


if __name__ == "__main__":
    essais()
    noms = projets()
    fiches = lire_skill()
    erreurs = controler(fiches, noms_projets=noms)
    with open(TRACE, encoding="utf-8") as fd:
        err_trace, a_faire = controler_trace(fd.read(), fiches, noms_projets=noms)
    erreurs += err_trace
    consigne = mcp_web_suite.INSTRUCTIONS
    if len(consigne) > MAX_INSTRUCTIONS or "web-suite" not in consigne:
        erreurs.append("consigne MCP : %d caractères au plus, avec le renvoi au skill web-suite" % MAX_INSTRUCTIONS)
    erreurs += prive("consigne MCP (INSTRUCTIONS)", consigne, noms)
    for e in erreurs:
        print("ÉCHEC :", e)
    if erreurs:
        sys.exit(1)
    print("OK : %d fiche(s), %d règle(s) encore « à faire » dans le relevé%s" %
          (len(fiches), a_faire, "" if depot("WEB_CAO") else " (sources non clonées : non relues)"))
