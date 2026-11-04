"""Outil MCP cao_schema_dessiner : dessiner dans le schéma ouvert, en une carte.

Spec : lanceur/docs/SPEC-mcp-dessin-schema.md. L'outil vérifie la forme des opérations (clés connues,
types, textes d'une ligne), puis les dépose par le canal de pilotage (mcp_pilote.envoyer, écriture).
La page (WEB_CAO/editeur-schematique/js/28-dessin-pilote.js, sur le socle commun/ecriture-cao.js) les
simule, montre la carte et applique au clic, en une étape d'historique. Toute autre écriture dans une
page passe par pilote_interface (décision du 2026-11-04). Ne s'importe que par mcp_web_suite.
"""
import math
import re

import mcp_pilote as pilote
from mcp_api_outils import facultatif, resumer
from mcp_registre_3d import refuser

TEXTE = re.compile(r"^[^\x00-\x1f\x7f]{0,200}$")


def _texte(v):
    return isinstance(v, str) and TEXTE.match(v) is not None


def _nombre(v):
    return isinstance(v, (int, float)) and not isinstance(v, bool) and math.isfinite(v) and abs(v) <= 1e6


def _nom(v):
    return _texte(v) and v.strip() != ""


# (contrôle, ce qu'on attend) : le message de refus le dit
TEXTE_ = (_texte, "texte d'une ligne, 200 caractères au plus")
NOM = (_nom, "texte d'une ligne non vide, 200 caractères au plus")
NOMBRE = (_nombre, "nombre fini")
BOOLEEN = (lambda v: isinstance(v, bool), "true ou false")


def parmi(*choix):
    return (lambda v: v in choix, " ou ".join(choix))


# dessin-schema : les clés de chaque opération (la page vérifie les valeurs)
DESSIN_MAX = 40
ENTIER = (lambda v: type(v) is int and 1 <= v <= 1000, "entier de 1 à 1000")
ANGLE = (lambda v: type(v) is int and v in (0, 90, 180, 270), "0, 90, 180 ou 270")
F = {"feuille": ENTIER}
OPERATIONS = {
    "poser": dict(F, type=NOM, catalogue=NOM, ref=NOM, valeur=NOM, x=NOMBRE, y=NOMBRE, rotation=ANGLE, miroir=BOOLEEN),
    "deplacer": dict(F, ref=NOM, x=NOMBRE, y=NOMBRE, dx=NOMBRE, dy=NOMBRE),
    "tourner": dict(F, ref=NOM, angle=ANGLE),
    "miroir": dict(F, ref=NOM),
    "supprimer": dict(F, ref=NOM),
    "relier": dict(F, de=NOM, a=NOM),
    "fil": dict(F, points=(lambda v: isinstance(v, list) and 2 <= len(v) <= 50 and all(
        isinstance(p, dict) and set(p) == {"x", "y"} and _nombre(p["x"]) and _nombre(p["y"]) for p in v),
        "liste de 2 à 50 points {x, y} en mm")),
    "supprimer_fil": dict(F, de=NOM, a=NOM),
    "etiqueter": dict(F, broche=NOM, net=NOM, genre=parmi("local", "global", "vcc", "gnd")),
    "feuille": {"creer": NOM, "renommer": ENTIER, "nom": NOM},
    "renumeroter": {},
    "cadre": dict(F, x1=NOMBRE, y1=NOMBRE, x2=NOMBRE, y2=NOMBRE, titre=TEXTE_, zone=BOOLEEN),
}


def valider(element, champs, requis, ou):
    if not isinstance(element, dict):
        refuser("%s : un objet attendu." % ou)
    for k in requis:
        if k not in element:
            refuser("%s : « %s » manquant." % (ou, k))
    for k, v in element.items():
        if k not in champs:
            refuser("%s : clé « %s » inconnue (%s)." % (ou, k, ", ".join(champs)))
        controle, attendu = champs[k]
        if not controle(v):
            refuser("%s : « %s » invalide (%s)." % (ou, k, attendu))


def envoyer(pages, nom, liste, a):
    # la réponse reprend les lignes de la carte (textes du projet) : résumée au-delà de 100 Ko
    return resumer(pilote.envoyer("web_cao", pages, nom, {"lot": liste}, True, facultatif(a, "onglet", str, None)),
                   "Lot trop gros pour la réponse : la carte est dans la page ; relis avec cao_*_lire.")


def cao_schema_dessiner(a):
    ops = a.get("operations")
    if not isinstance(ops, list) or not 1 <= len(ops) <= DESSIN_MAX:
        refuser("Argument « operations » invalide : liste de 1 à %d opérations (une carte)." % DESSIN_MAX)
    for i, o in enumerate(ops, 1):
        if not isinstance(o, dict) or not isinstance(o.get("op"), str) or o["op"] not in OPERATIONS:
            refuser("operations, élément %d : « op » parmi %s." % (i, ", ".join(OPERATIONS)))
        valider(o, dict(OPERATIONS[o["op"]], op=(lambda v: True, "")), (), "operations, élément %d (%s)" % (i, o["op"]))
    return envoyer(["schema"], "cao_schema_dessiner", ops, a)


ONGLET = pilote.ONGLET
OUTILS_ECRITURE_CAO = {
    "cao_schema_dessiner": {
        "description": "Dessine dans le schéma ouvert, en une carte : une liste ordonnée d'opérations (40 au "
                       "plus), simulée par la page avant la carte puis appliquée en une étape d'historique "
                       "(Ctrl+Z la défait). Opérations (« op ») : poser {type (symbole intégré : resistor, "
                       "capacitor, ic, opamp, regulator…) ou catalogue (Part Name de lib_composants), ref, "
                       "valeur, x, y (mm ; sans position, place libre), rotation, miroir} ; deplacer {ref, x, y "
                       "ou dx, dy} ; tourner {ref, angle} ; miroir {ref} ; supprimer {ref} ; relier {de, a : "
                       "« R1.2 » ou « U1.OUT »} ; fil {points : [{x, y}…] en mm, segments horizontaux ou "
                       "verticaux ; il ne relie que par ses deux bouts (broche, fil ou rien)} ; supprimer_fil {de, a} ; etiqueter {broche, net, genre : local, "
                       "global, vcc, gnd} ; feuille {creer} ou {renommer, nom} ; renumeroter (tout le document) ; "
                       "cadre {x1, y1, x2, y2, titre?, zone? : true = zone fonctionnelle, une room au PCB ; sinon "
                       "un cadre de note}. Texte libre : poser {type: annot_text (ou annot_note, encadrée), valeur ; "
                       "« | » sépare les lignes}. "
                       "Les autres opérations peuvent viser une « feuille » (n°), sinon la feuille affichée. Les "
                       "étiquettes et alimentations n'ont pas de repère : « #id » les désigne (rendu par etiqueter, ou "
                       "cao_schema_lire etiquettes) pour deplacer, tourner, miroir, supprimer. Un repère posé sert aux "
                       "opérations suivantes. Un fil qui relierait d'autres broches que celles demandées est "
                       "refusé : étiquette alors. Un seul refus et rien ne change."
                       + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": {"type": "object", "required": ["operations"], "properties": {
            "operations": {"type": "array", "minItems": 1, "maxItems": DESSIN_MAX, "items": {
                "type": "object", "required": ["op"], "properties": {"op": {"type": "string", "enum": list(OPERATIONS)}}}},
            "onglet": ONGLET}},
        "fn": cao_schema_dessiner,
    },
}
