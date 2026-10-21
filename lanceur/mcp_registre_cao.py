"""Outils MCP du module registre-cao : lire les pages WEB_CAO ouvertes, régler leur vue, lancer le DRC.

Spec : lanceur/docs/SPEC-mcp-registre-cao.md. Chaque outil valide ses arguments, puis dépose sa
commande aux pages WEB_CAO par le canal de pilotage (mcp_pilote.envoyer) ; la page
(WEB_CAO/commun/pilote-cao.js, lectures dans commun/lecture-cao.js) répond en texte. Rien n'est
écrit dans le projet. Ne s'importe que par mcp_web_suite.
"""
import re

import mcp_pilote as pilote
from mcp_api_outils import facultatif, resumer
from mcp_registre_3d import booleen, choix, refuser

QUOI = {
    "schema": ("feuilles", "composants", "composant", "etiquettes", "nets", "net"),
    "pcb": ("empilage", "classes", "composants", "composant", "nets", "net", "drc", "liaisons", "zone",
            "carte", "zones", "serigraphie", "interdits"),
    "ipc": ("empilage", "composants", "composant", "nets", "net"),
    "sim": ("simulations", "simulation", "catalogue"),
}
PAGES = {"schema": ["schema"], "pcb": ["pcb"], "ipc": ["visionneuse-ipc"], "sim": ["pcb", "visionneuse-ipc"]}
PAGES_VUE = ["schema", "pcb", "visionneuse-ipc"]
PARTIES = ("chiffres", "diagnostics", "methode", "piste", "bornes", "reglages", "resultats", "texte", "json")
TEXTE = re.compile(r"^[^\x00-\x1f]{1,200}$")     # repère, net, filtre, couche : une ligne, bornée
CONSEIL = "Trop long : précise le filtre, ou demande un seul composant ou un seul net."


def texte(a, nom):
    v = a.get(nom)
    if v is not None and (not isinstance(v, str) or not TEXTE.match(v)):
        refuser("Argument « %s » invalide : texte d'une ligne, 200 caractères au plus." % nom)
    return v


def entier(a, nom, maxi=1000):
    v = a.get(nom)
    if v is not None and (type(v) is not int or not 1 <= v <= maxi):
        refuser("Argument « %s » invalide : entier de 1 à %d." % (nom, maxi))
    return v


def envoyer(pages, nom, args, a):
    return pilote.envoyer("web_cao", pages, nom, args, False, facultatif(a, "onglet", str, None))


def coordonnee(a, nom):
    v = a.get(nom)
    if v is not None and (type(v) not in (int, float) or not -1000 <= v <= 1000 or v != v):
        refuser("Argument « %s » invalide : nombre de mm, à ±1000." % nom)
    return v


def lire(famille, a, cles):
    args = {"quoi": choix(a.get("quoi"), "quoi", QUOI[famille])}
    for cle in cles:
        v = (booleen(a, cle) if cle in ("a_router", "afficher") else entier(a, cle) if cle == "feuille"
             else coordonnee(a, cle) if cle in ("x1", "y1", "x2", "y2") else texte(a, cle))
        if v is not None and v is not False:        # « in (None, False) » écarterait aussi 0 (0 == False)
            args[cle] = v
    if args.get("partie") is not None:
        choix(args["partie"], "partie", PARTIES)
    return resumer(envoyer(PAGES[famille], "cao_%s_lire" % famille, args, a), CONSEIL)


def cao_schema_lire(a):
    return lire("schema", a, ("filtre", "ref", "nom", "feuille"))


def cao_pcb_lire(a):
    return lire("pcb", a, ("filtre", "ref", "nom", "a_router", "x1", "y1", "x2", "y2"))


def cao_ipc_lire(a):
    return lire("ipc", a, ("filtre", "ref", "nom"))


def cao_sim_lire(a):
    return lire("sim", a, ("analyse", "partie", "afficher"))


def cao_vue(a):
    pages = [choix(a["page"], "page", PAGES_VUE)] if a.get("page") is not None else PAGES_VUE
    args = {k: texte(a, k) for k in ("cadrer", "surligner") if a.get(k) is not None}
    if a.get("feuille") is not None:
        args["feuille"] = entier(a, "feuille")
    if a.get("couches") is not None:
        c = a["couches"]
        if not isinstance(c, list) or not 1 <= len(c) <= 64 or not all(isinstance(x, str) for x in c):
            refuser("Argument « couches » invalide : liste de 1 à 64 noms de couches.")
        args["couches"] = [texte({"couche": x}, "couche") for x in c]
    if booleen(a, "ajuster"):
        args["ajuster"] = True
    capturer = booleen(a, "capturer")
    if not args:
        refuser("Rien à faire : cadrer, surligner, feuille, couches ou ajuster.")
    vue = envoyer(pages, "cao_vue", args, a)
    if not capturer or vue.get("etat") != "finie":
        return vue
    # La capture du même onglet, dans la foulée : l'image et la vue en un appel
    image = pilote.pilote_capture({"outil": "web_cao", "onglet": (vue.get("onglet") or {}).get("id")})
    image["vue"] = vue.get("resultat")
    return image


def cao_thermique(a):
    return envoyer(["pcb"], "cao_thermique", {}, a)


def cao_drc(a):
    return resumer(envoyer(["pcb"], "cao_drc", {}, a), "Trop d'erreurs : corrige les premières, puis relance.")


ONGLET = pilote.ONGLET
DONNEES = pilote.DONNEES
PROPRIETES = {"filtre": {"type": "string", "description": "Sous-chaîne, sans tenir compte de la casse (repère, valeur, boîtier, nom de net)."},
              "ref": {"type": "string", "description": "Repère d'un composant (quoi = composant)."},
              "nom": {"type": "string", "description": "Nom d'un net (quoi = net)."},
              "onglet": ONGLET}


def schema_lire(famille, extra):
    props = dict(PROPRIETES, quoi={"type": "string", "enum": list(QUOI[famille])}, **extra)
    return {"type": "object", "properties": props, "required": ["quoi"]}


OUTILS_CAO = {
    "cao_schema_lire": {
        "description": "Lit l'éditeur schématique ouvert (toutes les feuilles) : feuilles, composants (repère, valeur, "
                       "boîtier, référence fabricant, feuille, position en mm), composant (net de chaque broche), "
                       "etiquettes (étiquettes de net et alimentations : #id, genre, net, feuille, position), "
                       "nets (global ou local, broches, feuilles), net (broches REPÈRE.patte). Ne modifie rien." + DONNEES,
        "inputSchema": schema_lire("schema", {"feuille": {"type": "integer", "minimum": 1,
                                                          "description": "Feuille (1, 2…) : départage deux nets locaux de même nom."}}),
        "fn": cao_schema_lire,
    },
    "cao_pcb_lire": {
        "description": "Lit l'éditeur PCB ouvert : empilage, classes et règles, composants, composant (net de chaque "
                       "broche), nets (état du routage), net (longueur et largeurs par couche, vias, zones, paire), "
                       "drc (dernier DRC lancé ; cao_drc le relance), liaisons (ce qui reste à router, de REF.pastille "
                       "à REF.pastille, avec positions et couches), zone (pastilles, pistes, vias, zones et contour "
                       "dans la fenêtre x1, y1, x2, y2 en mm), carte (contour : rectangle, disque ou polygone et ses "
                       "sommets, découpes), zones (id, couche, net, liaison des pastilles, emprise), serigraphie (id "
                       "des dessins, repères déplacés), interdits (zones interdites par face, hauteurs maximales). Positions "
                       "en mm, le repère de tous les outils CAO. "
                       "Ne modifie rien." + DONNEES,
        "inputSchema": schema_lire("pcb", {"a_router": {"type": "boolean", "description": "nets : seulement ceux qui restent à router."},
                                           **{k: {"type": "number", "description": "zone : fenêtre en mm"} for k in ("x1", "y1", "x2", "y2")}}),
        "fn": cao_pcb_lire,
    },
    "cao_ipc_lire": {
        "description": "Lit la visionneuse IPC-2581 ouverte : empilage, composants, composant (net de chaque broche), "
                       "nets (classe, longueur), net (pistes, plans, pastilles, broches). Ne modifie rien." + DONNEES,
        "inputSchema": schema_lire("ipc", {}),
        "fn": cao_ipc_lire,
    },
    "cao_sim_lire": {
        "description": "Lit les simulations de la page PCB ou de la visionneuse : simulations (analyses, celles qui ont un "
                       "résultat, chiffres clés), simulation (une partie du rapport d'une analyse ; commence par chiffres "
                       "et diagnostics), catalogue (paramètres réglables par cao_sim_ecrire : groupe.clé, unité ou "
                       "choix, valeur actuelle, où la lire dans une datasheet). Le rapport complet n'existe que pour "
                       "l'analyse affichée : afficher=true y passe le panneau. cao_calcul lance un calcul." + DONNEES,
        "inputSchema": {"type": "object", "required": ["quoi"], "properties": {
            "quoi": {"type": "string", "enum": list(QUOI["sim"])},
            "analyse": {"type": "string", "description": "Clé de l'analyse (rendue par quoi = simulations)."},
            "partie": {"type": "string", "enum": list(PARTIES)},
            "afficher": {"type": "boolean", "description": "Passer le panneau sur cette analyse avant de lire."},
            "onglet": ONGLET}},
        "fn": cao_sim_lire,
    },
    "cao_vue": {
        "description": "Règle la vue d'une page WEB_CAO, sans rien changer au projet : cadrer sur un repère ou un net "
                       "(sélectionné), surligner sans bouger la vue ni la sélection (au schéma, sur la feuille "
                       "affichée ou donnée), feuille (schéma), couches visibles (PCB, "
                       "visionneuse : les autres sont masquées), ajuster à tout le document. capturer=true rend "
                       "l'image dans la foulée. Un repère ou une couche inconnus ne changent rien. La visibilité des "
                       "couches du PCB part avec le prochain Enregistrer de l'utilisateur." + DONNEES,
        "inputSchema": {"type": "object", "properties": {
            "page": {"type": "string", "enum": PAGES_VUE, "description": "Page visée ; par défaut, la dernière utilisée."},
            "cadrer": {"type": "string"}, "surligner": {"type": "string"},
            "feuille": {"type": "integer", "minimum": 1}, "couches": {"type": "array", "items": {"type": "string"}},
            "ajuster": {"type": "boolean"}, "capturer": {"type": "boolean"}, "onglet": ONGLET}},
        "fn": cao_vue,
    },
    "cao_thermique": {
        "description": "Estime l'échauffement des composants du PCB ouvert qui dissipent (puissance réglée par "
                       "cao_composants_ecrire, champ puissance en W) : le cuivre relié à leurs pastilles (îlots de zone "
                       "touchés par un cuivre réel, pistes de leurs nets) est mesuré par la page, puis le modèle "
                       "d'étalement de la chute DC (dc_solver) rend ΔT, la résistance thermique et ce qui est supposé. "
                       "Un ordre de grandeur (±30 %), pas une simulation. Le DRC signale ensuite ce qui dépasse "
                       "l'échauffement max de la carte, tant qu'elle n'a pas bougé. N'entre pas dans l'historique." + DONNEES,
        "inputSchema": {"type": "object", "properties": {"onglet": ONGLET}},
        "fn": cao_thermique,
    },
    "cao_drc": {
        "description": "Lance le DRC de l'éditeur PCB ouvert (comme le bouton DRC) et rend les erreurs puis les "
                       "remarques, avec leur position en mm. N'entre pas dans l'historique." + DONNEES,
        "inputSchema": {"type": "object", "properties": {"onglet": ONGLET}},
        "fn": cao_drc,
    },
}
