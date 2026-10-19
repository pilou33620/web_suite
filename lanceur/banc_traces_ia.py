"""Banc du module retrait-gemini (lanceur/docs/SPEC-mcp-retrait-gemini.md) : plus aucune trace
active de l'ancien assistant IA (Gemini / Gemma) dans web_suite ni dans ses trois outils.

Restent permis :
- l'historique : les docs de lanceur/docs (specs, relevé du savoir) et les tasks/ ;
- ce qui cite l'assistant pour s'en garder : les routes interdites du MCP (mcp_web_suite.py) et
  leurs tests (banc_mcp.py), les tests de 404 des serveurs (python/test/), les .gitignore du
  fichier de clé ;
- le banc du savoir, qui relit les anciennes consignes à une révision figée, et ce banc-ci.
    python3 lanceur/banc_traces_ia.py
"""
import os
import re
import subprocess
import sys

ICI = os.path.dirname(os.path.abspath(__file__))
RACINE = os.path.dirname(ICI)
TRACE = re.compile(r"generativelanguage|gemma|gemini|/api/ia/cle|ia-assistant|06-ia-lib|30-ia\.js", re.I)
PERMIS = {
    "web_suite": (r"lanceur/docs/", r"tasks/", r"lanceur/mcp_web_suite\.py$", r"lanceur/banc_mcp\.py$",
                  r"lanceur/banc_savoir\.py$", r"lanceur/banc_traces_ia\.py$", r"\.gitignore$"),
    "outil": (r"tasks/", r"python/test/", r"\.gitignore$"),
}


def chercher(fichiers, permis):
    """fichiers : {chemin: texte}. Rend les traces hors des chemins permis : [(chemin, n°, ligne)]."""
    traces = []
    for chemin, texte in sorted(fichiers.items()):
        if any(re.match(p, chemin) or re.search(p, chemin) for p in permis):
            continue
        for n, ligne in enumerate(texte.split("\n"), 1):
            if TRACE.search(ligne):
                traces.append((chemin, n, ligne.strip()[:120]))
    return traces


def fichiers_suivis(depot):
    noms = subprocess.run(["git", "-C", depot, "ls-files"], capture_output=True, text=True, check=True).stdout.split("\n")
    out = {}
    for nom in filter(None, noms):
        try:
            with open(os.path.join(depot, nom), encoding="utf-8") as f:
                out[nom] = f.read()
        except (UnicodeDecodeError, OSError):
            continue          # binaire (wasm, image) ou fichier absent de la copie de travail
    return out


def depot(outil):
    """Dans web_suite sur le poste, à côté de lui en CI (cloné dans ../)."""
    for base in (RACINE, os.path.dirname(RACINE)):
        if os.path.isdir(os.path.join(base, outil, ".git")):
            return os.path.join(base, outil)
    return None


def essais():
    faux = {"js/page.js": "fetch('https://generativelanguage.googleapis.com')",
            "tasks/todo.md": "retirer ia-assistant.js",
            "python/test/banc.py": "GET /api/ia/cle -> 404",
            "css/x.css": "rien à signaler"}
    t = chercher(faux, PERMIS["outil"])
    assert [c for c, _n, _l in t] == ["js/page.js"], t
    assert chercher({"lanceur/docs/SPEC.md": "comme Gemini"}, PERMIS["web_suite"]) == []
    assert chercher({"lanceur/outil.py": "Gemma 4"}, PERMIS["web_suite"])


if __name__ == "__main__":
    essais()
    traces, vus = [], []
    for nom, d, permis in [("web_suite", RACINE, PERMIS["web_suite"])] + \
                          [(o, depot(o), PERMIS["outil"]) for o in ("WEB_CAO", "WEB_ANTENNA", "WEB_3D")]:
        if not d:
            print("(%s non cloné : pas vérifié)" % nom)
            continue
        vus.append(nom)
        traces += [(nom,) + t for t in chercher(fichiers_suivis(d), permis)]
    for nom, chemin, n, ligne in traces:
        print("TRACE : %s/%s:%d : %s" % (nom, chemin, n, ligne))
    if traces:
        sys.exit(1)
    print("OK : aucune trace de l'ancien assistant IA dans %s" % ", ".join(vus))
