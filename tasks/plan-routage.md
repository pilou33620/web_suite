# Plan : routage

Spec : [lanceur/docs/SPEC-mcp-routage.md](../lanceur/docs/SPEC-mcp-routage.md)
(validée le 2026-10-19). Liste des tâches : [todo-routage.md](todo-routage.md).

## Vue d'ensemble

Claude route la carte ouverte avec le routeur push-and-shove de la page, en donnant des
points de passage. Il lance aussi les placements automatiques de la page et lit deux
scores : placement et routage. Les routes, paires et placements vont par lots sur une
carte : une seule étape d'historique, avec un aperçu dessiné avant le clic.

## Graphe des dépendances

```
T1 lectures liaisons et zone (cao_pcb_lire)
 └── T2 socle « simuler puis charger » + cao_router (routes et vias, une carte)
      ├── T3 cao_paire (paires différentielles)
      ├── T4 cao_placement_auto (forces, satellites, blocs)
      └── T5 cao_score (placement, routage)
           └── T6 outils MCP (lanceur/mcp_routage.py)  ← T2…T5
                └── T7 essai réel, puis revue
```

## Décisions d'architecture

- **Simuler, relever, puis charger le résultat.** Le routeur a un garde-temps (250 ms par
  poussée) : le rejouer au clic pourrait donner un autre tracé sur le Pi. `proposer` fait
  donc tout le lot pour de vrai, puis :
  1. relève le document obtenu (`serialize()`) ;
  2. remet l'original par `loadDoc(…, true)`, comme Ctrl+Z ;
  3. remet `S.undo`, `S.redo`, `S.dirty`, la sélection et la vue.

  Au clic, `appliquer` vérifie que le document n'a pas bougé (`S.ver` relevé après la
  remise), fait un seul `push()`, puis charge le document relevé. C'est une étape
  d'historique, et le tracé appliqué est exactement celui de l'aperçu. Le socle
  `ECRITURE_CAO.lot` ne convient pas ici, car il rejoue le lot au clic. Le module a
  donc son propre petit socle, mais reprend du socle le bilan DRC, exporté pour l'occasion.
- **Un fichier de page**, `editeur-pcb/js/37-routage-pilote.js`, avec les commandes
  `cao_router`, `cao_paire`, `cao_placement_auto` et `cao_score`. Les lectures `liaisons`
  et `zone` vont dans `commun/lecture-cao.js` (`LECTURE_CAO.pcb`), à côté des autres.
- **Routes** :
  - départ et arrivée : `REF.pastille` (position monde par `padsWorld`, couche de la
    pastille) ou `{x, y}` en mm dans le repère de l'utilisateur ;
  - points intermédiaires `{x, y, couche?}` : un changement de couche pose une via
    (`routeToLayer`) ;
  - le tracé se fait par `startRoute(x, y, true)` puis, pour chaque point,
    `routeToPoint` et `stepRoute`, et se ferme par `commitRoute`, ou de lui-même s'il
    arrive sur le cuivre du net.
- **Tout ou rien par route** : chaque route part d'un instantané. Si le départ n'a pas de
  net, si `S.route.bad` empêche un pas, si une via est refusée ou si l'arrivée n'est pas
  atteinte, la route est défaite (`cancelRoute`, puis l'instantané). La raison est relevée
  dans l'état du routeur, pas dans `hint()`. Les autres routes du lot restent.
- **Réglages** : pendant le lot, `S.avoid` vaut vrai, les coins sont à 45° et le mode est
  « pousser » ou « contourner ». L'échelle de vue est fixée, pour que les accroches de
  `dpStart` ne dépendent pas du zoom. Tout revient comme avant, sans étape d'historique.
- **Aperçu dessiné** : les pistes et vias neuves du lot sont gardées dans
  `ROUTAGE_APERCU`. Une ligne ajoutée à `draw()` (`03-render.js`) les dessine par-dessus,
  en pointillés, tant que la carte attend. L'aperçu s'efface :
  - au clic ;
  - à la carte suivante ;
  - dès que le document change (`S.ver`).

  Une capture (`pilote_capture`) le montre.
- **Placement automatique** : `autoPlace(iter)` et `placerSatellites()`, ainsi que
  `BLOC_PLACEMENT.calculerAgencement(motif)` pour un motif nommé
  (`BLOC_PLACEMENT.getPatterns()`). Ils passent par le même socle « simuler puis
  charger ». Leurs `push()` internes tombent dans la pile mise de côté, puis remise.
- **Scores** :
  - placement : le document de `PLACEMENT_SCORE` (`construireDocumentPcb`, exporté en
    une ligne) est posté à `/api/pcb/score-placement` par la page, qui rend un résumé ;
  - routage (nouveau) : liaisons restantes (`conn(true)`), erreurs et remarques du DRC,
    nombre de vias, longueur totale, écarts aux contraintes (`cmMesures`, `cmRegleDe`,
    `cmVerifier`), topologie (`topoVerifier`) et groupes de longueur
    (`cmEvaluerGroupe`) ; un tableau et un total pondéré indicatif.
- **MCP** : `lanceur/mcp_routage.py`, avec les mêmes aides que `mcp_ecriture_cao.py`.
  `cao_pcb_lire` gagne les `quoi` `liaisons` et `zone` (`mcp_registre_cao.QUOI`).

## Risques

| Risque | Impact | Parade |
|---|---|---|
| `loadDoc` après la simulation laisse une trace (caches, panneaux, sélection) | Moyen | Test : document, piles, drapeau, sélection et vue identiques après une carte refusée |
| Un pas refusé ne lève rien (`stepRoute` revient en silence) | Élevé | Vérifier `S.route.bad` et `R.done.length` après chaque pas ; échec = route défaite, raison relevée |
| Le garde-temps rend la simulation lente sur le Pi (20 routes) | Moyen | Mesuré à l'essai réel ; lot borné à 20 routes |
| Ajouter `liaisons`/`zone` à `OUTILS_PCB` les donne aussi à Gemini | Faible | Voulu (lectures) ; le relevé de registre-cao ne couvre que les clés existantes |
| `calculerAgencement` dépend des motifs du schéma, absents d'un PCB seul | Faible | Refus clair « aucun motif publié par le schéma » |

## Questions ouvertes

Aucune.
