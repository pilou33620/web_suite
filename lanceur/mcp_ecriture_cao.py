"""Outils MCP du module ecriture-cao : écrire dans les pages WEB_CAO ouvertes, par cartes en lot.

Spec : lanceur/docs/SPEC-mcp-ecriture-cao.md. Chaque outil vérifie la forme du lot (clés connues,
types, textes d'une ligne, 50 éléments au plus), puis le dépose par le canal de pilotage
(mcp_pilote.envoyer, écriture). La page (WEB_CAO/commun/ecriture-cao.js) vérifie les valeurs avec
ses propres règles, montre la carte et applique au clic, en une étape d'historique (ou une carte
qu'Annuler défait, pour la simulation). Ne s'importe que par mcp_web_suite.
"""
import math
import re

import mcp_pilote as pilote
from mcp_api_outils import facultatif, resumer
from mcp_registre_3d import refuser

LOT_MAX = 50
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
NOMBRE_OU_NUL = (lambda v: v is None or _nombre(v), "nombre fini, ou null pour effacer")
BOOLEEN = (lambda v: isinstance(v, bool), "true ou false")
COUCHES = (lambda v: v is None or (isinstance(v, list) and 1 <= len(v) <= 64 and all(_nom(x) for x in v)),
           "liste de 1 à 64 noms de couches, ou null")
SPECS = (lambda v: isinstance(v, dict) and len(v) <= 30 and all(_nom(k) and (x is None or _texte(x)) for k, x in v.items()),
         "objet de 30 clés au plus, valeurs texte ou null")
LARGEURS = (lambda v: isinstance(v, dict) and len(v) <= 64 and all(_nom(k) and (x is None or _nombre(x)) for k, x in v.items()),
            "objet {couche: largeur en mm ou null}")
VALEUR_SIM = (lambda v: _nombre(v) or _nom(v), "nombre, ou texte pour un paramètre à choix")


def parmi(*choix):
    return (lambda v: v in choix, " ou ".join(choix))


CHAMPS_SCH = {"ref": NOM, "valeur": TEXTE_, "boitier": TEXTE_, "fabricant": TEXTE_, "mpn": TEXTE_,
              "specs": SPECS, "non_monte": BOOLEEN}
CHAMPS_PCB = {"ref": NOM, "valeur": TEXTE_, "boitier": NOM, "face": parmi("dessus", "dessous"),
              "x": NOMBRE, "y": NOMBRE, "rotation": NOMBRE}
PISTE = {"net": NOM, "largeur": NOMBRE, "couche": NOM, "realigner": BOOLEEN}
CLASSE = {"action": parmi("creer", "modifier", "supprimer", "affecter"), "nom": NOM, "net": NOM, "classe": NOM,
          "largeur": NOMBRE, "isolation": NOMBRE, "via": NOMBRE, "percage": NOMBRE, "largeur_couches": LARGEURS}
CONTRAINTE = {"action": parmi("paire", "net"), "nom": NOM, "portee": TEXTE_, "impedance": TEXTE_,
              "largeur_min": NOMBRE, "largeur_pref": NOMBRE, "largeur_max": NOMBRE, "ecart_min": NOMBRE,
              "ecart_pref": NOMBRE, "ecart_max": NOMBRE, "decouple_max": NOMBRE,
              "net": NOM, "z": NOMBRE_OU_NUL, "z_tol": NOMBRE_OU_NUL, "l_min": NOMBRE_OU_NUL, "l_max": NOMBRE_OU_NUL,
              "vias_max": NOMBRE_OU_NUL, "couches": COUCHES}
# géométrie de la carte : la page vérifie les valeurs (bornes, polygone simple, nets et couches connus)
POINTS = (lambda v: isinstance(v, list) and 3 <= len(v) <= 1000 and all(
              isinstance(p, dict) and set(p) == {"x", "y"} and _nombre(p["x"]) and _nombre(p["y"]) for p in v),
          "liste de 3 à 1000 sommets {x, y} en mm")
ID = (lambda v: type(v) is int and v >= 1, "entier, l'id lu par cao_pcb_lire")
FORME = {"x": NOMBRE, "y": NOMBRE, "largeur": NOMBRE, "hauteur": NOMBRE, "rayon": NOMBRE, "points": POINTS}
CONTOUR = dict(FORME, forme=parmi("rectangle", "disque", "polygone"))
DECOUPE = dict(FORME, action=parmi("creer", "supprimer"), forme=parmi("rectangle", "disque", "polygone"),
               numero=(lambda v: type(v) is int and v >= 1, "entier, le numéro lu par cao_pcb_lire carte"))
ZONE = dict(FORME, action=parmi("creer", "modifier", "supprimer"), id=ID, couche=NOM, net=TEXTE_,
            liaison=parmi("thermique", "directe", "cms-directe", "aucune"), forme=parmi("carte", "rectangle", "disque", "polygone"))
SERIGRAPHIE = {"action": parmi("texte", "trait", "rectangle", "repere", "auto", "supprimer"),
               "face": parmi("dessus", "dessous"), "texte": TEXTE_, "ref": NOM, "actif": BOOLEEN, "id": ID,
               **{k: NOMBRE for k in ("x", "y", "x1", "y1", "x2", "y2", "hauteur", "rotation", "largeur", "dx", "dy")}}
PASTILLE = {"pastille": NOM, "liaison": parmi("zone", "thermique", "directe", "aucune"),
            "bras_nombre": (lambda v: v is None or (type(v) is int and 1 <= v <= 8), "entier de 1 à 8, ou null"),
            **{k: NOMBRE_OU_NUL for k in ("bras_largeur", "bras_angle", "masque", "pate")}}
# condensateurs : {refs} | {mpn} | {valeur, boitier} ; bornes_dc : {composant} (la page contrôle laquelle)
CIBLE = {"refs": (lambda v: isinstance(v, list) and 1 <= len(v) <= 200 and all(_nom(x) for x in v), "liste de 1 à 200 repères"),
         "mpn": NOM, "valeur": NOM, "boitier": TEXTE_, "composant": NOM}
CIBLE_SIM = (lambda v: isinstance(v, dict) and bool(v) and all(k in CIBLE and CIBLE[k][0](x) for k, x in v.items()),
             "objet : refs (liste de repères), mpn, valeur, boitier (condensateurs) ou composant (bornes_dc)")
SIM = {"groupe": NOM, "cle": NOM, "valeur": VALEUR_SIM, "unite": TEXTE_, "cible": CIBLE_SIM}

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


def lot(a, champs, requis, sous=None):
    liste = a.get("lot")
    if not isinstance(liste, list) or not 1 <= len(liste) <= LOT_MAX:
        refuser("Argument « lot » invalide : liste de 1 à %d éléments (une carte)." % LOT_MAX)
    for i, e in enumerate(liste, 1):
        valider(e, champs, requis, "lot, élément %d" % i)
        if sous:
            sous(e, "lot, élément %d" % i)
    return liste


def envoyer(pages, nom, liste, a):
    # la réponse reprend les lignes de la carte (textes du projet) : résumée au-delà de 100 Ko
    return resumer(pilote.envoyer("web_cao", pages, nom, {"lot": liste}, True, facultatif(a, "onglet", str, None)),
                   "Lot trop gros pour la réponse : la carte est dans la page ; relis avec cao_*_lire.")


def cao_composants_ecrire(a):
    page = a.get("page")
    if page not in ("schema", "pcb"):
        refuser("Argument « page » invalide : schema ou pcb.")
    champs = CHAMPS_SCH if page == "schema" else CHAMPS_PCB

    def sous(e, ou):
        if not isinstance(e["champs"], dict) or not e["champs"]:
            refuser("%s : « champs » doit être un objet non vide." % ou)
        valider(e["champs"], champs, (), ou + ", champs")
    liste = lot(a, {"ref": NOM, "champs": (lambda v: True, "")}, ("ref", "champs"), sous)
    return envoyer([page], "cao_composants_ecrire", liste, a)


def cao_pistes_ecrire(a):
    return envoyer(["pcb"], "cao_pistes_ecrire", lot(a, PISTE, ("net",)), a)


def cao_classes_ecrire(a):
    return envoyer(["pcb"], "cao_classes_ecrire", lot(a, CLASSE, ("action",)), a)


def cao_contraintes_ecrire(a):
    return envoyer(["pcb"], "cao_contraintes_ecrire", lot(a, CONTRAINTE, ("action",)), a)


def cao_contour_ecrire(a):
    liste = lot(a, CONTOUR, ("forme",))
    if len(liste) != 1:
        refuser("Argument « lot » invalide : un seul contour par carte (un élément).")
    return envoyer(["pcb"], "cao_contour_ecrire", liste, a)


def cao_decoupes_ecrire(a):
    return envoyer(["pcb"], "cao_decoupes_ecrire", lot(a, DECOUPE, ("action",)), a)


def cao_zones_ecrire(a):
    return envoyer(["pcb"], "cao_zones_ecrire", lot(a, ZONE, ("action",)), a)


def cao_serigraphie_ecrire(a):
    return envoyer(["pcb"], "cao_serigraphie_ecrire", lot(a, SERIGRAPHIE, ("action",)), a)


def cao_pastille_regler(a):
    return envoyer(["pcb"], "cao_pastille_regler", lot(a, PASTILLE, ("pastille",)), a)


def cao_schema_dessiner(a):
    ops = a.get("operations")
    if not isinstance(ops, list) or not 1 <= len(ops) <= DESSIN_MAX:
        refuser("Argument « operations » invalide : liste de 1 à %d opérations (une carte)." % DESSIN_MAX)
    for i, o in enumerate(ops, 1):
        if not isinstance(o, dict) or not isinstance(o.get("op"), str) or o["op"] not in OPERATIONS:
            refuser("operations, élément %d : « op » parmi %s." % (i, ", ".join(OPERATIONS)))
        valider(o, dict(OPERATIONS[o["op"]], op=(lambda v: True, "")), (), "operations, élément %d (%s)" % (i, o["op"]))
    return envoyer(["schema"], "cao_schema_dessiner", ops, a)


def cao_sim_ecrire(a):
    return envoyer(["pcb", "visionneuse-ipc"], "cao_sim_ecrire", lot(a, SIM, ("groupe", "cle", "valeur")), a)


ONGLET = pilote.ONGLET
CARTE = (" Un lot = une carte (50 éléments au plus) : la page vérifie tout, un seul refus et rien n'est "
         "changé ; appliqué, le lot fait une seule étape d'historique (Ctrl+Z le défait).")
DRC = " Au PCB, la réponse donne le DRC avant → après et les nouvelles erreurs."


def schema_lot(element, requis, **autres):
    return {"type": "object", "required": list(autres) + ["lot"], "properties": dict(autres, **{
        "lot": {"type": "array", "minItems": 1, "maxItems": LOT_MAX,
                "items": {"type": "object", "required": list(requis), "properties": element}},
        "onglet": ONGLET})}


N = {"type": "number"}
T = {"type": "string"}
OUTILS_ECRITURE_CAO = {
    "cao_composants_ecrire": {
        "description": "Change des propriétés de composants. Schéma : ref, valeur, boitier, fabricant, mpn, "
                       "specs (objet clé → texte, null efface), non_monte. PCB : ref, valeur, boitier (doit "
                       "exister dans les empreintes), face (dessus/dessous), x, y (mm, même repère que "
                       "cao_pcb_lire), rotation (degrés) ; le cuivre accroché suit selon la préférence de "
                       "la page. Un repère est unique dans le document, à chaque étape du lot : pour échanger R1 et R2, "
                       "passe par un repère libre dans deux cartes."
                       + CARTE + DRC + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"ref": T, "champs": {"type": "object"}}, ("ref", "champs"),
                                  page={"type": "string", "enum": ["schema", "pcb"]}),
        "fn": cao_composants_ecrire,
    },
    "cao_pistes_ecrire": {
        "description": "Largeur des pistes déjà posées : {net, largeur (mm, 0,05 à 10), couche?} pour les "
                       "pistes du net (d'une couche) ; {net, realigner: true} remet pistes et vias du net sur "
                       "sa classe." + CARTE + DRC + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"net": T, "largeur": N, "couche": T, "realigner": {"type": "boolean"}}, ("net",)),
        "fn": cao_pistes_ecrire,
    },
    "cao_classes_ecrire": {
        "description": "Classes de nets, actions jouées dans l'ordre : creer / modifier {nom, largeur, "
                       "isolation, via, percage (mm), largeur_couches {couche: mm}}, supprimer {nom} (pas la "
                       "classe par défaut, ni une classe qui garde des nets), affecter {net, classe}. Le "
                       "routage posé ne bouge pas : cao_pistes_ecrire realigner le fait."
                       + CARTE + DRC + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"action": {"type": "string", "enum": ["creer", "modifier", "supprimer", "affecter"]},
                                   "nom": T, "net": T, "classe": T, "largeur": N, "isolation": N, "via": N,
                                   "percage": N, "largeur_couches": {"type": "object"}}, ("action",)),
        "fn": cao_classes_ecrire,
    },
    "cao_contraintes_ecrire": {
        "description": "Règles de paires différentielles {action: paire, nom, largeur_min/pref/max, "
                       "ecart_min/pref/max, decouple_max (mm), portee (nom de paire, vide = toutes), "
                       "impedance (profil : D90, D100, D85, D120)} et contraintes de net {action: net, net, "
                       "z (Ω), z_tol (%), l_min, l_max (mm), vias_max, couches ; null efface}. Les "
                       "contraintes saisies au schéma ne sont pas touchées."
                       + CARTE + DRC + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"action": {"type": "string", "enum": ["paire", "net"]}, "nom": T, "portee": T,
                                   "impedance": T, "net": T, "couches": {"type": ["array", "null"], "items": T}},
                                  ("action",)),
        "fn": cao_contraintes_ecrire,
    },
    "cao_contour_ecrire": {
        "description": "Contour de la carte, un seul élément : {forme: rectangle, x, y (coin), largeur, hauteur} ; "
                       "{forme: disque, x, y (centre), rayon} — écrit en arcs au Gerber, au DXF et à l'IPC-2581 ; "
                       "{forme: polygone, points: [{x, y}…]} (3 à 1000 sommets, côtés qui ne se coupent pas). "
                       "En mm, côtés de 5 mm au moins. Les découpes intérieures restent ; les plans de couche "
                       "suivent le contour. Relire : cao_pcb_lire carte." + CARTE + DRC + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"forme": {"type": "string", "enum": ["rectangle", "disque", "polygone"]},
                                   "x": N, "y": N, "largeur": N, "hauteur": N, "rayon": N,
                                   "points": {"type": "array", "items": {"type": "object", "properties": {"x": N, "y": N}}}},
                                  ("forme",)),
        "fn": cao_contour_ecrire,
    },
    "cao_decoupes_ecrire": {
        "description": "Découpes intérieures de la carte (fenêtres fraisées), actions dans l'ordre : creer {forme : "
                       "rectangle {x, y, largeur, hauteur}, disque {x, y, rayon}, polygone {points}} — dans le "
                       "contour ; supprimer {numero : 1 = la première, cao_pcb_lire carte ; numéros d'avant le lot}. "
                       "Le cuivre s'en tient à la marge de bord, comme du contour." + CARTE + DRC + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"action": {"type": "string", "enum": ["creer", "supprimer"]}, "numero": {"type": "integer"},
                                   "forme": {"type": "string", "enum": ["rectangle", "disque", "polygone"]},
                                   "x": N, "y": N, "largeur": N, "hauteur": N, "rayon": N,
                                   "points": {"type": "array", "items": {"type": "object", "properties": {"x": N, "y": N}}}},
                                  ("action",)),
        "fn": cao_decoupes_ecrire,
    },
    "cao_zones_ecrire": {
        "description": "Zones de cuivre, actions jouées dans l'ordre : creer {couche, net (vide = cuivre isolé), forme "
                       "(carte : la carte entière ; rectangle {x, y, largeur, hauteur} ; disque {x, y, rayon} ; polygone "
                       "{points}), liaison?} ; modifier {id, et ce qui change : couche, net, forme…, liaison} ; "
                       "supprimer {id}. liaison des pastilles du net : thermique (bras, défaut), directe, cms-directe "
                       "(directe pour les CMS, bras pour les traversantes : plot thermique de LED, QFN, régulateur), "
                       "aucune (pastilles dégagées, non reliées). Un bras qui toucherait un autre net n'est pas tracé. "
                       "Les id viennent de cao_pcb_lire zones ; la réponse donne ceux des zones créées."
                       + CARTE + DRC + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"action": {"type": "string", "enum": ["creer", "modifier", "supprimer"]}, "id": {"type": "integer"},
                                   "couche": T, "net": T, "liaison": {"type": "string", "enum": ["thermique", "directe", "cms-directe", "aucune"]},
                                   "forme": {"type": "string", "enum": ["carte", "rectangle", "disque", "polygone"]},
                                   "x": N, "y": N, "largeur": N, "hauteur": N, "rayon": N,
                                   "points": {"type": "array", "items": {"type": "object", "properties": {"x": N, "y": N}}}},
                                  ("action",)),
        "fn": cao_zones_ecrire,
    },
    "cao_serigraphie_ecrire": {
        "description": "Sérigraphie, actions dans l'ordre : texte {face (dessus/dessous), texte, x, y (centre), hauteur "
                       "(mm, 1 par défaut), rotation, largeur (trait, 0,15)} ; trait / rectangle {face, x1, y1, x2, y2, "
                       "largeur} ; repere {ref, dx, dy : décalage du repère depuis sa place d'office au-dessus du "
                       "boîtier, en mm} ; auto {ref, actif : sérigraphie automatique du composant} ; supprimer {id "
                       "d'un dessin, cao_pcb_lire serigraphie}. Ce qui est posé est ce que le Gerber imprime ; les "
                       "valeurs des composants ne sont pas imprimées." + CARTE + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"action": {"type": "string", "enum": ["texte", "trait", "rectangle", "repere", "auto", "supprimer"]},
                                   "face": {"type": "string", "enum": ["dessus", "dessous"]}, "texte": T, "ref": T,
                                   "actif": {"type": "boolean"}, "id": {"type": "integer"},
                                   **{k: N for k in ("x", "y", "x1", "y1", "x2", "y2", "hauteur", "rotation", "largeur", "dx", "dy")}},
                                  ("action",)),
        "fn": cao_serigraphie_ecrire,
    },
    "cao_pastille_regler": {
        "description": "Réglages d'une pastille {pastille: « U1.3 » (toutes celles de ce numéro), liaison (zone : celle "
                       "de la zone ; thermique ; directe ; aucune), bras_largeur (mm), bras_nombre (1 à 8), bras_angle (°), "
                       "masque (ouverture, mm), pate (retrait, mm)} ; null rend la règle de la carte. Une empreinte "
                       "calculée est figée en pastilles explicites. Relire : cao_pcb_lire composant."
                       + CARTE + DRC + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"pastille": T, "liaison": {"type": "string", "enum": ["zone", "thermique", "directe", "aucune"]},
                                   "bras_nombre": {"type": ["integer", "null"]},
                                   **{k: {"type": ["number", "null"]} for k in ("bras_largeur", "bras_angle", "masque", "pate")}},
                                  ("pastille",)),
        "fn": cao_pastille_regler,
    },
    "cao_sim_ecrire": {
        "description": "Paramètres de simulation de la page PCB ou de la visionneuse : {groupe (pdn_fiche, "
                       "pdn, si, bus, dc), cle, valeur, unite}. L'unité doit être celle du catalogue de la "
                       "page (pas de conversion) ; bornes et choix sont ceux du catalogue. Parasites des "
                       "condensateurs : {groupe: condensateurs, cle (esr_mohm, esl_nh, cap_nf), valeur, unite "
                       "(mΩ, nH, nF), cible: {refs: [repères]} ou {mpn} ou {valeur: 100nF, boitier?}}. Bornes "
                       "de la chute DC déjà posées : {groupe: bornes_dc, cle (courant_ma : courant TOTAL de la "
                       "charge, réparti sur ses broches ; tension_v : la source), valeur, unite (mA, V), cible: "
                       "{composant: repère}}. Hors historique : "
                       "Annuler sur la carte remet les valeurs d'avant. La réponse donne aussi ce que la page "
                       "en dit (calcul à relancer, assistant ΔI)." + CARTE.replace(
                           " ; appliqué, le lot fait une seule étape d'historique (Ctrl+Z le défait).", ".")
                       + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": schema_lot({"groupe": T, "cle": T, "valeur": {"type": ["number", "string"]}, "unite": T,
                                   "cible": {"type": "object"}},
                                  ("groupe", "cle", "valeur")),
        "fn": cao_sim_ecrire,
    },
    "cao_schema_dessiner": {
        "description": "Dessine dans le schéma ouvert, en une carte : une liste ordonnée d'opérations (40 au "
                       "plus), simulée par la page avant la carte puis appliquée en une étape d'historique "
                       "(Ctrl+Z la défait). Opérations (« op ») : poser {type (symbole intégré : resistor, "
                       "capacitor, ic, opamp, regulator…) ou catalogue (Part Name de lib_composants), ref, "
                       "valeur, x, y (mm ; sans position, place libre), rotation, miroir} ; deplacer {ref, x, y "
                       "ou dx, dy} ; tourner {ref, angle} ; miroir {ref} ; supprimer {ref} ; relier {de, a : "
                       "« R1.2 » ou « U1.OUT »} ; supprimer_fil {de, a} ; etiqueter {broche, net, genre : local, "
                       "global, vcc, gnd} ; feuille {creer} ou {renommer, nom} ; renumeroter (tout le document) ; "
                       "cadre {x1, y1, x2, y2, titre?, zone? : true = zone fonctionnelle, une room au PCB ; sinon "
                       "un cadre de note}. Texte libre : poser {type: annot_text (ou annot_note, encadrée), valeur ; "
                       "« | » sépare les lignes}. "
                       "Les autres opérations peuvent viser une « feuille » (n°), sinon la feuille affichée. Les "
                       "étiquettes et alimentations n'ont pas de repère : seul Ctrl+Z les retire. Un repère posé sert aux "
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
