"""Relevé d'un essai WEB_ANTENNA, au format des croisement-*.json de la racine.

Les essais se font sur le PC Windows par Claude in Chrome, qui recopie le relevé
dans son rapport (commande J3 des specs, lue dans la page). Ce script sert à
Claude Code à CONTRÔLER cette recopie quand les projets essai-claude-* sont
arrivés jusqu'à lui (envoyés sur GitHub par l'utilisateur : PROJETS = dépôt
WEB_SUITE_PROJETS ; les calculs/ n'y vont pas, resultats.json si) :
    python tasks/essais/ESSAI-antenna-releve.py SORTIE.json "note" "nom du cas=<PROJETS>/ANTENNA/<projet>/resultats.json" [...]

Lit le `resultat` de chaque resultats.json (balayage, croisement ou simulation seule),
garde les nombres (f0, S11 min, Z au port et au pied, bande, MSLPort, rendement et gain)
et laisse les courbes et le diagramme, qui restent dans le projet. Un seul cas : le
fichier a la forme de croisement-L-y0.json ; plusieurs : une liste "cas", comme
croisement-patch-2026-10.json. Specs : tasks/essais/ESSAI-antenna-*.md.
"""
import json, sys
GARDE = ("f0", "s11_min_db", "z0_re", "z0_im", "z0_pied_re", "z0_pied_im",
         "bp_f1", "bp_f2", "bp_bord", "msl", "ligne")
def court(r):
    o = {k: r.get(k) for k in GARDE if k in r}
    nf = r.get("nf2ff") or {}
    o["nf2ff"] = {k: v for k, v in nf.items() if not isinstance(v, (list, dict))}
    return o
def releve(r):
    if not r.get("balayage"):
        return court(r)
    return {"balayage": {k: r.get(k) for k in ("nom", "unite", "nom2", "unite2", "croise")},
            "lignes": r["lignes"],
            "points": [dict(etiquette=p["etiquette"], valeur=p["valeur"], valeur2=p.get("valeur2"),
                            **court(p["resultat"] or {})) for p in r["points"]]}
sortie, note, *cas = sys.argv[1:]
liste = []
for c in cas:
    nom, chemin = c.split("=", 1)
    liste.append(dict(cas=nom, **releve(json.load(open(chemin, encoding="utf-8"))["resultat"])))
out = dict(_note=note, **liste[0]) if len(liste) == 1 else {"_note": note, "cas": liste}
json.dump(out, open(sortie, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(sortie, ":", len(liste), "cas,", sum(len(c.get("points", [1])) for c in liste), "points")
