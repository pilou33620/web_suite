"""Outils MCP du module api-outils : ce que les serveurs des outils savent faire sans la page.

Spec : lanceur/docs/SPEC-mcp-api-outils.md. Chaque outil valide ses arguments, puis passe
par appeler_outil_web() du noyau : il n'y a pas d'outil « appeler une route », la table
OUTILS_API est la liste blanche.
"""
import json
import urllib.parse

import mcp_web_suite as noyau

PLAFOND = 100 * 1024   # octets de JSON rendus à Claude ; au-delà, un résumé
DOCS_CAO = ("schema", "pcb", "ipc2581")


def exiger(args, **types):
    """Les arguments nommés, présents et du bon type, dans l'ordre demandé ; sinon une
    erreur qui nomme le fautif. Un seul argument : rendu seul."""
    valeurs = [facultatif(args, nom, type_, None) for nom, type_ in types.items()]
    for nom, v in zip(types, valeurs):
        if v is None:
            raise noyau.ErreurOutil("Argument « %s » manquant." % nom)
    return valeurs[0] if len(valeurs) == 1 else valeurs


def facultatif(args, nom, type_, defaut):
    v = args.get(nom)
    if v is None:
        return defaut
    if not isinstance(v, type_) or (type_ is int and isinstance(v, bool)):
        raise noyau.ErreurOutil("Argument « %s » invalide : %s attendu." % (nom, type_.__name__))
    return v


def resumer(valeur, conseil):
    """Au-delà du plafond, la forme du JSON au lieu du contenu : Claude redemande une partie."""
    brut = json.dumps(valeur, ensure_ascii=False)
    if len(brut.encode("utf-8")) <= PLAFOND:
        return valeur

    def decrire(v):
        if len(json.dumps(v, ensure_ascii=False)) <= 200:
            return v
        if isinstance(v, (dict, list, str)):
            return {"type": type(v).__name__, "longueur": len(v)}
        return {"type": type(v).__name__}

    resume = {"resume": True, "taille_octets": len(brut.encode("utf-8")), "conseil": conseil}
    if isinstance(valeur, dict):
        resume["cles_total"] = len(valeur)
        resume["forme"] = {k: decrire(valeur[k]) for k in list(valeur)[:200]}   # 200 x 200 o au plus
    else:
        resume["forme"] = decrire(valeur)
    resume["extrait"] = brut[:2000]
    return resume


def route(base, **params):
    """Route plus paramètres encodés ; les valeurs None sont omises."""
    params = {k: v for k, v in params.items() if v is not None}
    return base + ("?" + urllib.parse.urlencode(params) if params else "")


def cao_projets(_args):
    return resumer(noyau.appeler_outil_web("web_cao", "/api/projets"), "Trop de projets : demande-les par racine.")


def cao_projet_lire(args):
    chemin = exiger(args, chemin=str)
    racine = facultatif(args, "racine", str, None)
    return resumer(noyau.appeler_outil_web("web_cao", route("/api/projet", chemin=chemin, racine=racine)),
                   "Projet trop gros : lis ses documents avec cao_doc_lire.")


def cao_doc_lire(args):
    chemin, doc = exiger(args, chemin=str, doc=str)
    racine = facultatif(args, "racine", str, None)
    parties = facultatif(args, "parties", list, None)
    if doc not in DOCS_CAO:
        raise noyau.ErreurOutil("Argument « doc » invalide : %s attendu." % ", ".join(DOCS_CAO))
    if parties is not None and not all(isinstance(p, str) for p in parties):
        raise noyau.ErreurOutil("Argument « parties » invalide : liste de noms de clés attendue.")
    rep = noyau.appeler_outil_web("web_cao", route("/api/projet/doc", chemin=chemin, racine=racine, doc=doc))
    document = rep.get("document") if isinstance(rep, dict) else None
    if not isinstance(document, dict):
        return resumer(rep, "Document trop gros.")
    if parties is not None:
        absentes = [p for p in parties if p not in document]
        document = {p: document[p] for p in parties if p in document}
        if absentes:
            rep["parties_absentes"] = absentes
    rep["document"] = resumer(document, "Document trop gros : redemande avec « parties » (clés de premier "
                                        "niveau listées dans « forme »).")
    return rep


CHEMIN = {"type": "string", "description": "Chemin du projet, tel que rendu par cao_projets."}
RACINE = {"type": "string", "description": "Racine du projet (champ « racine » de cao_projets) ; facultatif."}

OUTILS_API = {
    "cao_projets": {
        "description": "Liste les projets WEB_CAO (nom, chemin, racine, date). Point de départ avant "
                       "de lire un projet.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": cao_projets,
    },
    "cao_projet_lire": {
        "description": "Ouvre un projet WEB_CAO en lecture : son projet.cao.json et l'état de ses "
                       "documents (schéma, PCB, IPC-2581 : présent, date de modification).",
        "inputSchema": {"type": "object", "properties": {"chemin": CHEMIN, "racine": RACINE},
                        "required": ["chemin"]},
        "fn": cao_projet_lire,
    },
    "cao_doc_lire": {
        "description": "Lit un document d'un projet WEB_CAO (schema, pcb ou ipc2581). Rend aussi "
                       "« modifie », à renvoyer pour écrire. Au-delà de 100 Ko, rend un résumé : "
                       "redemande alors seulement les clés utiles avec « parties ».",
        "inputSchema": {"type": "object", "properties": {
            "chemin": CHEMIN, "racine": RACINE,
            "doc": {"type": "string", "enum": list(DOCS_CAO)},
            "parties": {"type": "array", "items": {"type": "string"},
                        "description": "Clés de premier niveau du document à rendre ; facultatif."}},
            "required": ["chemin", "doc"]},
        "fn": cao_doc_lire,
    },
}
