"""Outils MCP du module registre-antenna : lire la page WEB_ANTENNA ouverte et régler sa vue.

Spec : lanceur/docs/SPEC-mcp-registre-antenna.md. Chaque outil valide ses arguments, puis dépose sa
commande à la page « antenna » par le canal de pilotage (mcp_pilote.envoyer, sans écriture) ; la page
(WEB_ANTENNA/js/pilote-antenna.js, lectures dans js/29-lecture-antenna.js) répond en texte. Rien
n'est écrit, rien n'est lancé : les calculs openEMS restent à lancer depuis la page. Ne s'importe
que par mcp_web_suite.
"""
import mcp_pilote as pilote
from mcp_api_outils import facultatif, resumer
from mcp_registre_3d import booleen, choix, refuser
from mcp_registre_cao import entier, texte

QUOI = ("conception", "resultats", "courbe", "balayage", "audit", "reglages", "composants", "composant", "nets", "net")
GRANDEURS = ("s11", "s21", "z", "roe")
ONGLETS = ("bal", "ts", "s11", "s21", "roe", "z", "ff")
TOILES = ("carte", "3d", "courbe")
CONSEIL = "Trop long : précise le filtre, ou demande un seul composant, un seul net ou une seule grandeur."


def envoyer(nom, args, a):
    return pilote.envoyer("web_antenna", ["antenna"], nom, args, False, facultatif(a, "onglet", str, None))


def antenna_lire(a):
    args = {"quoi": choix(a.get("quoi"), "quoi", QUOI)}
    if a.get("grandeur") is not None:
        args["grandeur"] = choix(a["grandeur"], "grandeur", GRANDEURS)
    for cle in ("filtre", "ref", "nom"):
        if a.get(cle) is not None:
            args[cle] = texte(a, cle)
    return resumer(envoyer("antenna_lire", args, a), CONSEIL)


def antenna_vue(a):
    args = {}
    if a.get("vue") is not None:
        args["vue"] = choix(a["vue"], "vue", ("2d", "3d"))
    if a.get("maillage") is not None:
        args["maillage"] = booleen(a, "maillage")
    if booleen(a, "ajuster"):
        args["ajuster"] = True
    # « onglet » désigne l'onglet du navigateur dans tout le canal : l'onglet de résultats s'appelle ici « resultats »
    if a.get("resultats") is not None:
        args["onglet"] = choix(a["resultats"], "resultats", ONGLETS)
    if a.get("point_balayage") is not None:
        args["point_balayage"] = entier(a, "point_balayage", 100000)   # un balayage croisé dépasse vite 1000 points
    if a.get("etape") is not None:
        args["etape"] = entier(a, "etape", 20)
    if a.get("panneau") is not None:
        args["panneau"] = texte(a, "panneau")
    if a.get("toile") is not None:
        args["toile"] = choix(a["toile"], "toile", TOILES)
    capturer = booleen(a, "capturer")
    if not args:
        refuser("Rien à faire : vue, maillage, ajuster, resultats, point_balayage, etape, panneau ou toile.")
    vue = envoyer("antenna_vue", args, a)
    if not capturer or vue.get("etat") != "finie":
        return vue
    # la capture du même onglet, dans la foulée : l'image et la vue en un appel
    image = pilote.pilote_capture({"outil": "web_antenna", "onglet": (vue.get("onglet") or {}).get("id")})
    image["vue"] = vue.get("resultat")
    return image


ONGLET = pilote.ONGLET
T = {"type": "string"}
OUTILS_ANTENNA = {
    "antenna_lire": {
        "description": "Lit la page WEB_ANTENNA ouverte, ce qui n'est pas encore enregistré compris : conception "
                       "(le résumé de l'assistant : carte, empilage, bande, ports, boîte, maillage, motif et cotes, "
                       "chiffrage du serveur), resultats (f0, S11, Z, bande à -10 dB, Dmax, couplages), courbe "
                       "(grandeur s11, s21, z ou roe ; 200 points au plus), balayage (tableau du balayage fini), "
                       "audit (contrôles hors ligne, chacun avec une carte prête pour antenna_proposer), reglages "
                       "(chemins modifiables par antenna_proposer : valeur, unité, bornes, choix), et pour la carte "
                       "importée composants, composant, nets, net. Ne modifie rien, ne lance aucun calcul."
                       + pilote.DONNEES,
        "inputSchema": {"type": "object", "required": ["quoi"], "properties": {
            "quoi": {"type": "string", "enum": list(QUOI)}, "grandeur": {"type": "string", "enum": list(GRANDEURS)},
            "filtre": T, "ref": T, "nom": T, "onglet": ONGLET}},
        "fn": antenna_lire,
    },
    "antenna_vue": {
        "description": "Règle la vue de la page WEB_ANTENNA, sans rien changer au projet ni aux préférences : vue "
                       "(2d, 3d), maillage (montré ou caché), ajuster (2D), resultats (onglet de résultats : bal, ts, "
                       "s11, s21, roe, z, ff), point_balayage (n°), etape (n° de l'étape de l'assistant ; elle part "
                       "avec le prochain enregistrement du projet), panneau (assistant, resultats, couches…), toile "
                       "à capturer (carte ou 3d : la vue affichée ; courbe : ouvre le panneau des résultats s'il est caché). "
                       "Un panneau montré reste si tu changes ensuite la disposition. capturer=true rend l'image dans la foulée. Tout est vérifié "
                       "avant le premier changement. Jamais de calcul lancé." + pilote.DONNEES,
        "inputSchema": {"type": "object", "properties": {
            "vue": {"type": "string", "enum": ["2d", "3d"]}, "maillage": {"type": "boolean"}, "ajuster": {"type": "boolean"},
            "resultats": {"type": "string", "enum": list(ONGLETS)}, "point_balayage": {"type": "integer", "minimum": 1},
            "etape": {"type": "integer", "minimum": 1}, "panneau": T, "toile": {"type": "string", "enum": list(TOILES)},
            "capturer": {"type": "boolean"}, "onglet": ONGLET}},
        "fn": antenna_vue,
    },
}
