"""Outils MCP du module registre-3d : commander la page WEB_3D ouverte.

Spec : lanceur/docs/SPEC-mcp-registre-3d.md. Chaque outil valide ses arguments, puis
dépose sa commande à la page « 3d » par le canal de pilotage (mcp_pilote.envoyer) ; la
page (WEB_3D/js/14-pilote-3d.js) fait le calcul. Ne s'importe que par mcp_web_suite.
"""
import math
import re

import mcp_pilote as pilote
import mcp_web_suite as noyau
from mcp_api_outils import facultatif, resumer

VUES = ("avant", "arriere", "gauche", "droite", "dessus", "dessous", "iso")
ACTIONS = ("masquer", "afficher", "isoler", "tout_afficher", "selectionner")
QUOI = ("dimensions", "distance", "faces")
REF = re.compile(r"^[^\x00-\x1f]{1,200}$")      # uuid, chemin « 0/3/1 », ou nom unique
MM = 1e6                                          # mm : au-delà, la vue casse (positions infinies)
DEGRES = 3600


def refuser(texte):
    raise noyau.ErreurOutil(texte)


def ref(v, nom="piece"):
    if not isinstance(v, str) or not REF.match(v) or not v.strip():
        refuser("Argument « %s » invalide : uuid, chemin (0/3/1) ou nom unique d'une pièce." % nom)
    return v


def nombre(v, nom, mini=None, maxi=None):
    if type(v) not in (int, float) or not math.isfinite(v) or (mini is not None and v < mini) \
            or (maxi is not None and v > maxi):
        refuser("Argument « %s » invalide : nombre fini%s." % (
            nom, "" if mini is None else " entre %s et %s" % (mini, maxi)))
    return v


def vecteur(v, nom, borne=MM):
    if not isinstance(v, list) or len(v) != 3:
        refuser("Argument « %s » invalide : trois nombres [x, y, z]." % nom)
    return [nombre(x, nom, -borne, borne) for x in v]


def booleen(a, nom):
    """Un vrai booléen JSON : en Python, bool("false") vaut True."""
    v = a.get(nom, False)
    if type(v) is not bool:
        refuser("Argument « %s » invalide : true ou false." % nom)
    return v


def choix(v, nom, possibles):
    if v not in possibles:
        refuser("Argument « %s » invalide : %s." % (nom, ", ".join(possibles)))
    return v


def index(v, nom):
    if type(v) is not int or v < 0:
        refuser("Argument « %s » invalide : entier positif ou nul." % nom)
    return v


def cible(a):
    """{piece} ou {fichier}, pour cadrer ou mesurer."""
    if not isinstance(a, dict):
        refuser("Cible invalide : {\"piece\": …} ou {\"fichier\": n}.")
    if "piece" in a:
        return {"piece": ref(a["piece"])}
    if "fichier" in a:
        return {"fichier": index(a["fichier"], "fichier")}
    return {}


def envoyer(nom, args, ecriture, a):
    return pilote.envoyer("web_3d", ["3d"], nom, args, ecriture, facultatif(a, "onglet", str, None))


def web3d_scene(a):
    nb = index(a.get("nombre", 200), "nombre")
    if nb > 1000:
        refuser("Argument « nombre » invalide : 1000 au plus (paginer avec « debut »).")
    args = {"filtre": facultatif(a, "filtre", str, ""), "debut": index(a.get("debut", 0), "debut"), "nombre": nb}
    return envoyer("web3d_scene", args, False, a)


def web3d_vue(a):
    args = {}
    if a.get("vue") is not None:
        args["vue"] = choix(a["vue"], "vue", VUES)
    if a.get("projection") is not None:
        args["projection"] = choix(a["projection"], "projection", ("ortho", "perspective"))
    if a.get("cadrer") is not None:
        args["cadrer"] = cible(a["cadrer"])
    capturer = booleen(a, "capturer")
    vue = envoyer("web3d_vue", args, False, a)
    if not capturer or vue.get("etat") != "finie":
        return vue
    # La capture du même onglet, dans la foulée : l'image et la vue en un appel
    image = pilote.pilote_capture({"outil": "web_3d", "onglet": vue["onglet"]["id"]})
    image["vue"] = vue.get("resultat")
    return image


def web3d_visibilite(a):
    action = choix(a.get("action"), "action", ACTIONS)
    refs = a.get("pieces", [])
    if not isinstance(refs, list) or len(refs) > 500:
        refuser("Argument « pieces » invalide : liste de références.")
    return envoyer("web3d_visibilite", {"action": action, "pieces": [ref(r) for r in refs]}, False, a)


def web3d_coupe(a):
    args = {}
    if "actif" in a:
        args["actif"] = booleen(a, "actif")
    if a.get("axe") is not None:
        args["axe"] = choix(a["axe"], "axe", ("x", "y", "z"))
    if a.get("face") is not None:
        f = a["face"]
        if not isinstance(f, dict):
            refuser("Argument « face » invalide : {\"piece\": …, \"index\": n}.")
        args["face"] = {"piece": ref(f.get("piece")), "index": index(f.get("index"), "face.index")}
    if a.get("decalage_mm") is not None:
        args["decalage_mm"] = nombre(a["decalage_mm"], "decalage_mm", -MM, MM)
    if "inverse" in a:
        args["inverse"] = booleen(a, "inverse")
    return envoyer("web3d_coupe", args, False, a)


def web3d_mesurer(a):
    if booleen(a, "effacer"):
        return envoyer("web3d_mesurer", {"effacer": True}, False, a)
    quoi = choix(a.get("quoi", "dimensions"), "quoi", QUOI)
    args = {"quoi": quoi, "afficher": booleen(a, "afficher")}
    if quoi == "distance":
        refs = a.get("pieces")
        if not isinstance(refs, list) or len(refs) != 2:
            refuser("distance : deux pièces dans « pieces ».")
        args["pieces"] = [ref(r) for r in refs]
    elif quoi == "faces":
        args["piece"] = ref(a.get("piece"))
    else:
        args.update(cible({k: a[k] for k in ("piece", "fichier") if k in a}))
    return resumer(envoyer("web3d_mesurer", args, False, a), "Trop de faces : demande une autre pièce.")


def web3d_collisions(a):
    args = {"tolerance": nombre(a.get("tolerance", 0.1), "tolerance", 1e-6, 10), "afficher": booleen(a, "afficher")}
    return envoyer("web3d_collisions", args, False, a)


def web3d_ouvrir(a):
    chemins = a.get("chemins")
    if not isinstance(chemins, list) or not 1 <= len(chemins) <= 20:
        refuser("Argument « chemins » invalide : 1 à 20 fichiers de PROJETS/3D (voir web3d_projets).")
    for c in chemins:
        if (not isinstance(c, str) or not c or c.startswith("/") or "\\" in c or ":" in c
                or any(b in ("", ".", "..") for b in c.split("/"))):
            refuser("Chemin invalide : %r (relatif à PROJETS/3D, tel que le rend web3d_projets)." % (c,))
    return envoyer("web3d_ouvrir", {"chemins": chemins, "ajout": booleen(a, "ajout")}, True, a)


def web3d_deplacer(a):
    args = {"fichier": index(a.get("fichier"), "fichier")}
    for k in ("position", "translation", "rotation_deg"):
        if a.get(k) is not None:
            args[k] = vecteur(a[k], k, DEGRES if k == "rotation_deg" else MM)
    if booleen(a, "origine"):
        args["origine"] = True
    if a.get("contrainte") is not None:
        c = a["contrainte"]
        if not isinstance(c, dict) or not isinstance(c.get("mobile"), dict) or not isinstance(c.get("fixe"), dict):
            refuser("contrainte : {\"mobile\": {piece, face}, \"fixe\": {piece, face}, \"decalage\": mm}.")
        args["contrainte"] = {k: {"piece": ref(c[k].get("piece")), "face": index(c[k].get("face"), k + ".face")}
                              for k in ("mobile", "fixe")}
        args["contrainte"]["decalage"] = nombre(c.get("decalage", 0), "decalage", -MM, MM)
    if len(args) == 1:
        refuser("Rien à faire : position, translation, rotation_deg, origine ou contrainte.")
    return envoyer("web3d_deplacer", args, True, a)


def web3d_ranger_step(a):
    args = {"piece": ref(a.get("piece"))}
    if a.get("nom") is not None:
        nom = a["nom"]
        if not isinstance(nom, str) or not re.match(r"^[\w .()+-]{1,100}$", nom):
            refuser("Argument « nom » invalide : un nom de fichier simple (sans dossier).")
        args["nom"] = nom
    return envoyer("web3d_ranger_step", args, True, a)


ONGLET = pilote.ONGLET
REF_DOC = {"type": "string", "description": "Pièce : uuid ou chemin (0/3/1) rendus par web3d_scene ; un nom seulement s'il est unique."}
VEC = {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3}
CARTE = (" Carte dans la page : rien ne change avant le clic de l'utilisateur (sauf interrupteur allumé) ; "
         "l'appel attend jusqu'à 2 min, puis rend « en_attente » (pilote_suivre).")

OUTILS_3D = {
    "web3d_scene": {
        "description": "Scène WEB_3D ouverte : fichiers (index, position, rotation) et pièces (uuid, chemin, nom, "
                       "visible, couleur), avec filtre sur le nom et pagination ; sélection, coupe, projection.",
        "inputSchema": {"type": "object", "properties": {"filtre": {"type": "string"}, "debut": {"type": "integer"},
                                                         "nombre": {"type": "integer"}, "onglet": ONGLET}},
        "fn": web3d_scene,
    },
    "web3d_vue": {
        "description": "Règle la vue 3D (avant, arriere, gauche, droite, dessus, dessous, iso), la projection, et "
                       "cadre une pièce ou un fichier ; capturer: true rend l'image dans la foulée.",
        "inputSchema": {"type": "object", "properties": {
            "vue": {"type": "string", "enum": list(VUES)}, "projection": {"type": "string", "enum": ["ortho", "perspective"]},
            "cadrer": {"type": "object", "description": "{\"piece\": ref} ou {\"fichier\": n}"},
            "capturer": {"type": "boolean"}, "onglet": ONGLET}},
        "fn": web3d_vue,
    },
    "web3d_visibilite": {
        "description": "Masquer, afficher, isoler, tout afficher ou sélectionner des pièces dans la vue 3D.",
        "inputSchema": {"type": "object", "properties": {
            "action": {"type": "string", "enum": list(ACTIONS)}, "pieces": {"type": "array", "items": REF_DOC},
            "onglet": ONGLET}, "required": ["action"]},
        "fn": web3d_visibilite,
    },
    "web3d_coupe": {
        "description": "Plan de coupe : actif, axe x/y/z ou face plane d'une pièce (index de web3d_mesurer faces), "
                       "décalage en mm (borné par la page), sens inversé.",
        "inputSchema": {"type": "object", "properties": {
            "actif": {"type": "boolean"}, "axe": {"type": "string", "enum": ["x", "y", "z"]},
            "face": {"type": "object", "description": "{\"piece\": ref, \"index\": n}"},
            "decalage_mm": {"type": "number"}, "inverse": {"type": "boolean"}, "onglet": ONGLET}},
        "fn": web3d_coupe,
    },
    "web3d_mesurer": {
        "description": "Mesures en mm (repère du monde) : dimensions (encombrement, volume, surface) d'une pièce, d'un "
                       "fichier ou de la scène ; distance minimale entre deux pièces (afficher: true pose la cote dans "
                       "la vue) ; faces d'une pièce (plan, cylindre : rayon, axe, aire, index). effacer: true retire la cote.",
        "inputSchema": {"type": "object", "properties": {
            "quoi": {"type": "string", "enum": list(QUOI)}, "piece": REF_DOC, "fichier": {"type": "integer"},
            "pieces": {"type": "array", "items": REF_DOC}, "afficher": {"type": "boolean"},
            "effacer": {"type": "boolean"}, "onglet": ONGLET}},
        "fn": web3d_mesurer,
    },
    "web3d_collisions": {
        "description": "Interférences entre pièces visibles (surfaces qui se croisent ; un contact ne compte pas) : "
                       "couples et longueur des courbes. afficher: true les dessine dans la vue.",
        "inputSchema": {"type": "object", "properties": {
            "tolerance": {"type": "number", "description": "mm, 0,1 par défaut"}, "afficher": {"type": "boolean"},
            "onglet": ONGLET}},
        "fn": web3d_collisions,
    },
    "web3d_ouvrir": {
        "description": "Ouvre des fichiers de PROJETS/3D (chemins de web3d_projets) dans la page, en remplacement ou en "
                       "ajout ; un .obj emmène son .mtl. Annuler recharge le modèle d'avant (ou retire l'ajout)." + CARTE,
        "inputSchema": {"type": "object", "properties": {
            "chemins": {"type": "array", "items": {"type": "string"}}, "ajout": {"type": "boolean"}, "onglet": ONGLET},
            "required": ["chemins"]},
        "fn": web3d_ouvrir,
    },
    "web3d_deplacer": {
        "description": "Déplace un fichier de l'assemblage (index de web3d_scene) : position absolue, translation, "
                       "rotation_deg absolue (autour du centre de sa boîte), origine, ou contrainte face plane contre "
                       "face plane (décalage) / axe contre axe, faces désignées par leur index. Annulable." + CARTE,
        "inputSchema": {"type": "object", "properties": {
            "fichier": {"type": "integer"}, "position": VEC, "translation": VEC, "rotation_deg": VEC,
            "origine": {"type": "boolean"},
            "contrainte": {"type": "object", "description": "{\"mobile\": {\"piece\", \"face\"}, \"fixe\": {\"piece\", \"face\"}, \"decalage\": mm}"},
            "onglet": ONGLET}, "required": ["fichier"]},
        "fn": web3d_deplacer,
    },
    "web3d_ranger_step": {
        "description": "Extrait le STEP d'une pièce (du STEP d'origine, positions d'origine) et le range dans "
                       "PROJETS/3D sans jamais écraser (un nom pris devient « x (2) ») ; non annulable par la page."
                       + CARTE,
        "inputSchema": {"type": "object", "properties": {
            "piece": REF_DOC, "nom": {"type": "string", "description": "Nom du fichier, sans dossier"},
            "onglet": ONGLET}, "required": ["piece"]},
        "fn": web3d_ranger_step,
    },
}

for _nom, _outil in OUTILS_3D.items():
    _outil["description"] += pilote.DONNEES
    if _nom != "web3d_vue":       # la vue avec capture garde son image (rendue à part)
        _outil["fn"] = (lambda fn: lambda a: resumer(fn(a), "Réponse trop grosse."))(_outil["fn"])
