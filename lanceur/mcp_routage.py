"""Outils MCP du module routage : router la carte ouverte par points de passage, placer, scorer.

Spec : lanceur/docs/SPEC-mcp-routage.md. Chaque outil vérifie la forme de ses arguments, puis les
dépose à la page PCB par le canal de pilotage (WEB_CAO/editeur-pcb/js/37-routage-pilote.js). La page
joue le lot pour de vrai, relève le résultat, remet la carte et montre la carte d'action avec un
aperçu ; au clic, elle charge le résultat relevé en une étape d'historique. cao_score et cao_chemin
(module chemin-auto, SPEC-mcp-chemin-auto.md : des points de passage proposés par A*, 40-chemin-auto.js)
sont des lectures. Ne s'importe que par mcp_web_suite.
"""
import mcp_pilote as pilote
from mcp_api_outils import facultatif, resumer
from mcp_ecriture_cao import NOM, NOMBRE, parmi, valider
from mcp_registre_3d import choix, refuser

ROUTES_MAX, POINTS_MAX, VIAS_MAX, POINTS_LOT = 20, 200, 50, 1000
MODE = parmi("pousser", "contourner")


def _mm(v):
    return NOMBRE[0](v) and -1000 <= v <= 1000


def _point(v):
    return (isinstance(v, str) and NOM[0](v)) or (
        isinstance(v, dict) and set(v) <= {"x", "y", "couche"} and _mm(v.get("x")) and _mm(v.get("y"))
        and (v.get("couche") is None or NOM[0](v["couche"])))


POINT = (_point, "« REF.pastille », ou {x, y, couche?} en mm (±1000)")
POINTS = (lambda v: isinstance(v, list) and len(v) <= POINTS_MAX and all(_point(x) for x in v),
          "liste de %d points au plus" % POINTS_MAX)
ROUTE = {"depart": POINT, "arrivee": POINT, "points": POINTS, "largeur": NOMBRE, "couche": NOM}
VIA = {"x": NOMBRE, "y": NOMBRE, "net": NOM, "de": NOM, "a": NOM}


def envoyer(nom, args, a, ecriture=True):
    return resumer(pilote.envoyer("web_cao", ["pcb"], nom, args, ecriture, facultatif(a, "onglet", str, None)),
                   "Réponse trop longue : relis avec cao_pcb_lire ou cao_score.")


def liste(a, cle, maxi, champs, requis):
    v = a.get(cle)
    if v is None:
        return []
    if not isinstance(v, list) or len(v) > maxi:
        refuser("Argument « %s » invalide : liste de %d éléments au plus." % (cle, maxi))
    for i, e in enumerate(v, 1):
        valider(e, champs, requis, "%s, élément %d" % (cle, i))
    return v


def cao_router(a):
    args = {"routes": liste(a, "routes", ROUTES_MAX, ROUTE, ("depart", "arrivee")),
            "vias": liste(a, "vias", VIAS_MAX, VIA, ("x", "y", "net"))}
    if not args["routes"] and not args["vias"]:
        refuser("Rien à router : « routes » ou « vias ».")
    if sum(len(r.get("points") or []) for r in args["routes"]) > POINTS_LOT:
        refuser("%d points de passage au plus par carte : découpe le lot." % POINTS_LOT)
    if a.get("mode") is not None:
        args["mode"] = choix(a["mode"], "mode", ("pousser", "contourner"))
    return envoyer("cao_router", args, a)


def cao_paire(a):
    champs = dict(ROUTE, paire=NOM, mode=MODE)
    del champs["largeur"]
    corps = {k: v for k, v in a.items() if k != "onglet"}
    valider(corps, champs, ("paire", "depart", "arrivee"), "paire")
    return envoyer("cao_paire", corps, a)


def cao_placement_auto(a):
    args = {"methode": choix(a.get("methode"), "methode", ("forces", "satellites", "blocs"))}
    if a.get("motif") is not None:
        valider({"motif": a["motif"]}, {"motif": NOM}, (), "placement")
        args["motif"] = a["motif"]
    return envoyer("cao_placement_auto", args, a)


BOUTS_PAIRE = (lambda v: isinstance(v, list) and len(v) == 2 and all(isinstance(x, str) and NOM[0](x) for x in v),
               "[« REF.pastille » de P, « REF.pastille » de N]")


def cao_chemin(a):
    liaisons = a.get("liaisons")
    if not isinstance(liaisons, list) or len(liaisons) > ROUTES_MAX:
        refuser("Argument « liaisons » invalide : liste de %d éléments au plus." % ROUTES_MAX)
    for i, e in enumerate(liaisons, 1):
        if isinstance(e, dict) and "paire" in e:     # une paire : son axe, de deux pastilles à deux pastilles
            valider(e, {"paire": NOM, "depart": BOUTS_PAIRE, "arrivee": BOUTS_PAIRE}, ("paire", "depart", "arrivee"),
                    "liaisons, élément %d" % i)
        else:
            valider(e, {"depart": POINT, "arrivee": POINT, "largeur": NOMBRE}, ("depart", "arrivee"),
                    "liaisons, élément %d" % i)
    args = {"liaisons": liaisons}
    if not args["liaisons"]:
        refuser("Argument « liaisons » manquant : de 1 à %d liaisons {depart, arrivee, largeur?}." % ROUTES_MAX)
    if a.get("pas") is not None:
        if not (NOMBRE[0](a["pas"]) and 0.05 <= a["pas"] <= 2):
            refuser("Argument « pas » invalide : de 0,05 à 2 mm.")
        args["pas"] = a["pas"]
    return envoyer("cao_chemin", args, a, False)


def cao_score(a):
    return envoyer("cao_score", {"quoi": choix(a.get("quoi"), "quoi", ("routage", "placement"))}, a, False)


ONGLET = pilote.ONGLET
T = {"type": "string"}
PT = {"description": "« REF.pastille » (U1.3) ou {x, y, couche?} en mm, repère de cao_pcb_lire"}
CARTE = (" Le lot est joué pour de vrai par la page puis défait ; la carte montre route par route « posée » "
         "ou « refusée : raison », et un aperçu en pointillés (le cuivre déplacé par la poussée en rouge ; "
         "pilote_capture le montre). Au clic, le cuivre vu est chargé en une étape d'historique (Ctrl+Z la "
         "défait), avec le DRC avant → après ; les réglages de l'éditeur ne changent pas. Le lot entier est "
         "borné à 10 s de calcul dans la page.")
OUTILS_ROUTAGE = {
    "cao_router": {
        "description": "Route des liaisons de la carte PCB ouverte par points de passage : le routeur de la page "
                       "(push-and-shove) fait le détail entre les points. routes : [{depart, points?, arrivee, "
                       "largeur?, couche?}] (20 au plus ; un point avec « couche » change de couche par une via) ; "
                       "vias : [{x, y, net, de?, a?}] seules. mode : pousser (défaut) ou contourner. Tout ou rien par "
                       "route : une route bloquée ne laisse rien, les autres restent. Lis d'abord cao_pcb_lire "
                       "liaisons et zone." + CARTE + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": {"type": "object", "properties": {
            "routes": {"type": "array", "maxItems": ROUTES_MAX, "items": {"type": "object", "required": ["depart", "arrivee"],
                       "properties": {"depart": PT, "arrivee": PT, "points": {"type": "array", "items": PT},
                                      "largeur": {"type": "number"}, "couche": T}}},
            "vias": {"type": "array", "maxItems": VIAS_MAX, "items": {"type": "object", "required": ["x", "y", "net"]}},
            "mode": {"type": "string", "enum": ["pousser", "contourner"]}, "onglet": ONGLET}},
        "fn": cao_router,
    },
    "cao_paire": {
        "description": "Route une paire différentielle déclarée (cao_pcb_lire classes les liste) : depart (un bout "
                       "libre de ses deux nets, ou le milieu de ses deux pastilles), points? de passage, arrivee ; "
                       "les deux pistes restent au pas de la règle de paire." + CARTE + pilote.ATTENTE_DOC
                       + pilote.DONNEES,
        "inputSchema": {"type": "object", "required": ["paire", "depart", "arrivee"], "properties": {
            "paire": T, "depart": PT, "arrivee": PT, "points": {"type": "array", "items": PT}, "couche": T,
            "mode": {"type": "string", "enum": ["pousser", "contourner"]}, "onglet": ONGLET}},
        "fn": cao_paire,
    },
    "cao_placement_auto": {
        "description": "Lance un placement automatique de la page sur la carte PCB : forces (dégrossit tout, "
                       "découplages contre leur broche), satellites (découplages et composants série contre leur "
                       "circuit), blocs (agence un motif ou une zone publiés par le schéma : « motif »)."
                       + " Le placement est joué par la page puis défait ; la carte dit combien d'empreintes bougent "
                         "et lesquelles, et l'aperçu dessine chacune à sa nouvelle place en pointillés, avec son "
                         "trajet depuis l'ancienne (pilote_capture le montre). Au clic, une étape d'historique "
                         "(Ctrl+Z la défait)."
                       + pilote.ATTENTE_DOC + pilote.DONNEES,
        "inputSchema": {"type": "object", "required": ["methode"], "properties": {
            "methode": {"type": "string", "enum": ["forces", "satellites", "blocs"]}, "motif": T, "onglet": ONGLET}},
        "fn": cao_placement_auto,
    },
    "cao_score": {
        "description": "Score de la carte PCB ouverte. routage : liaisons restantes, erreurs et remarques du DRC "
                       "(dont contraintes, groupes de longueur, topologie), vias, longueur, et un total indicatif "
                       "à faire baisser. placement : le score du serveur (longueur des liaisons, congestion, "
                       "découplage, pires contributeurs). Ne modifie rien." + pilote.DONNEES,
        "inputSchema": {"type": "object", "required": ["quoi"], "properties": {
            "quoi": {"type": "string", "enum": ["routage", "placement"]}, "onglet": ONGLET}},
        "fn": cao_score,
    },
    "cao_chemin": {
        "description": "Propose des points de passage pour des liaisons de la carte PCB ouverte (module "
                       "chemin-auto) : un chercheur de chemin A* sur une grille par couche (pas de 0,25 mm par "
                       "défaut), angles de 45°, vias là où toutes les couches traversées sont libres. Il évite le "
                       "cuivre des autres nets à l'isolation de leurs classes, le bord de carte, les découpes et "
                       "les trous ; les zones de cuivre ne gênent pas (elles se recoulent). Les liaisons se "
                       "cherchent dans l'ordre, chaque chemin trouvé gênant les suivantes. Rend, par liaison, "
                       "trouvée ou la raison, et « routes » à donner tel quel à cao_router (qui fait le détail et "
                       "montre sa carte). Une PAIRE différentielle déclarée (cao_pcb_lire classes) : {paire, "
                       "depart: [pastille de P, pastille de N], arrivee: [idem]} ; son axe est cherché aussi large "
                       "que ses deux pistes et leur écart (règle de paire) ; il change de couche par une via de "
                       "paire (deux vias côte à côte, au point qui précède le point portant « couche »), jamais à "
                       "portée de ses pastilles, et « paires » rend, pour chacune, les arguments de cao_paire (une carte par paire, "
                       "qui fait les éventails jusqu'aux pastilles). Ne modifie rien. 20 liaisons et 8 s au plus "
                       "par appel." + pilote.DONNEES,
        "inputSchema": {"type": "object", "required": ["liaisons"], "properties": {
            "liaisons": {"type": "array", "minItems": 1, "maxItems": ROUTES_MAX, "items": {
                "type": "object", "required": ["depart", "arrivee"], "properties": {
                    "paire": {"type": "string", "description": "nom d'une paire déclarée : depart et arrivee sont alors "
                                                               "deux « REF.pastille », celle de P et celle de N"},
                    "depart": {"description": PT["description"] + " ; pour une paire, [P, N]"},
                    "arrivee": {"description": PT["description"] + " ; pour une paire, [P, N]"},
                    "largeur": {"type": "number", "description": "mm ; absente : la classe du net (pas pour une paire)"}}}},
            "pas": {"type": "number", "description": "pas de la grille en mm (0,05 à 2), 0,25 par défaut"},
            "onglet": ONGLET}},
        "fn": cao_chemin,
    },
}
