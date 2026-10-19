"""Banc du skill web-suite (module savoir-metier, lanceur/docs/SPEC-mcp-savoir-metier.md).

Ce que les fiches citent doit exister dans le serveur MCP : chaque outil (`cao_pcb_lire`) et
chaque lecture (`cao_pcb_lire` (quoi=liaisons)). Les fiches restent courtes, leurs liens
internes mènent quelque part, et rien de privé n'y entre (chemin personnel, clé).
    python3 lanceur/banc_savoir.py
"""
import os
import re
import sys

ICI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, ICI)
import mcp_web_suite  # noqa: E402

RACINE = os.path.dirname(ICI)
SKILL = os.path.join(RACINE, ".claude", "skills", "web-suite")
MAX_SKILL, MAX_FICHE = 150, 200

OUTIL = re.compile(r"`((?:cao|lib|antenna|web3d|pilote|pcbparts)_[a-z0-9_]+)`")
LECTURE = re.compile(r"`([a-z0-9_]+)`\s*\(quoi=([a-z_]+)\)")
LIEN = re.compile(r"\]\(([^)\s]+)\)")
PRIVE = re.compile(r"/home/|/Users/|[A-Za-z]:\\\\|AIza[0-9A-Za-z_-]{20}|ghp_[0-9A-Za-z]{20}|sk-ant-")


def controler(fiches, outils=None, dossier=SKILL):
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
        for o, quoi in LECTURE.findall(texte):
            enum = (outils.get(o, {}).get("inputSchema", {}).get("properties", {}).get("quoi", {}).get("enum"))
            if not enum or quoi not in enum:
                erreurs.append("%s : %s n'a pas de quoi=%s" % (nom, o, quoi))
        for cible in LIEN.findall(texte):
            if re.match(r"[a-z]+:", cible):
                continue          # lien externe : pas vérifié ici
            chemin = os.path.normpath(os.path.join(dossier, cible.split("#")[0]))
            if cible.split("#")[0] and cible.split("#")[0] not in fiches and not os.path.exists(chemin):
                erreurs.append("%s : lien mort : %s" % (nom, cible))
        if PRIVE.search(texte):
            erreurs.append("%s : chemin personnel ou clé : %s" % (nom, PRIVE.search(texte).group(0)))
    return erreurs


def lire_skill(dossier=SKILL):
    return {f: open(os.path.join(dossier, f), encoding="utf-8").read()
            for f in sorted(os.listdir(dossier)) if f.endswith(".md")}


def essais():
    """Le banc lui-même : chaque faute connue doit être vue."""
    bon = "---\nname: web-suite\ndescription: x\n---\nVoir `cao_pcb_lire` (quoi=liaisons) et [pcb](pcb.md).\n"
    assert controler({"SKILL.md": bon, "pcb.md": "ok"}) == [], controler({"SKILL.md": bon, "pcb.md": "ok"})
    for faute, motif in ((bon + "`cao_inventer`", "outil inconnu"),
                         (bon + "`cao_pcb_lire` (quoi=tout)", "quoi=tout"),
                         (bon + "x\n" * 160, "lignes"),
                         (bon + "[x](absent.md)", "lien mort"),
                         (bon + "dans /home/pilou/x", "chemin personnel"),
                         (bon.replace("web-suite", "web_suite"), "en-tête")):
        e = controler({"SKILL.md": faute, "pcb.md": "ok"})
        assert any(motif in x for x in e), (motif, e)
    assert any("SKILL.md manquant" in x for x in controler({"pcb.md": "ok"}))


if __name__ == "__main__":
    essais()
    erreurs = controler(lire_skill())
    if len(mcp_web_suite.INSTRUCTIONS) > 1500 or "web-suite" not in mcp_web_suite.INSTRUCTIONS:
        erreurs.append("consigne MCP : 1500 caractères au plus, avec le renvoi au skill web-suite")
    for e in erreurs:
        print("ÉCHEC :", e)
    if erreurs:
        sys.exit(1)
    print("OK : %d fiche(s)" % len(lire_skill()))
