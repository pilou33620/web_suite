# Spec : routage

Module 9 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`registre-cao`](SPEC-mcp-registre-cao.md).
Statut : **validée le 2026-10-19**.

## Objectif

Laisser Claude **router** la carte ouverte avec le routeur push-and-shove existant, sans
souris : pistes, vias, paires différentielles ; lancer le placement automatique de la
page ; rendre un **score** de placement et de routage pour boucler jusqu'à une carte
propre.

### Décisions (2026-10-19)

1. **Claude donne les points de passage** : il lit la carte (liaisons à router,
   obstacles, captures) et donne une suite de points et de couches ; le routeur fait le
   détail (contournement, poussée). Un chercheur de chemin (A*) viendra plus tard, dans un
   module à part (`chemin-auto`, spec plus tard).
2. **Échec** : une route qui bloque n'est pas posée du tout ; la page dit où et pourquoi
   (point, couche, obstacle), Claude réessaie autrement.
3. **Placement** : score de placement (serveur existant), placement automatique de la page
   (par carte), score de routage (nouveau).
4. **Une carte = un lot de routes** : une seule étape d'annulation, taille bornée (20
   routes), aperçu dessiné avant le clic.

### Outils MCP

Dans `lanceur/mcp_routage.py` ; commandes de page dans
`editeur-pcb/js/40-routage-pilote.js` (le routage n'existe qu'au PCB).

| Outil MCP | Ce qu'il fait |
|---|---|
| `cao_pcb_lire` (nouveaux `quoi`) | `liaisons` : liaisons à router, de `REF.pastille` à `REF.pastille`, coordonnées, couche, net, classe (bornées à 150) ; `zone` + `x1,y1,x2,y2` : pistes, vias, pastilles, zones et contour dans une fenêtre (200 objets au plus) |
| `cao_router` | Lot de routes `[{depart, points:[{x,y,couche?}], arrivee, largeur?}]` et de vias seuls `[{x,y,net}]` ; `mode` `pousser` ou `contourner`. `depart`/`arrivee` : `REF.pastille` ou `{x,y}` |
| `cao_paire` | Une paire déclarée : `paire`, `depart`, `points`, `arrivee` (même forme) |
| `cao_placement_auto` | `methode` (`forces`, `satellites`, `blocs`) et `refs` facultatif : carte, une étape d'historique |
| `cao_score` | `quoi` = `placement` (score serveur : longueur des liaisons, congestion, découplage, pires contributeurs) ou `routage` (nouveau, ci-dessous) |

**Score de routage** : liaisons restantes, erreurs et remarques du DRC, nombre de vias,
longueur totale, écarts aux contraintes (`cmVerifier`), topologie (`topoVerifier`),
désappariement des paires et groupes de longueur. Un tableau, plus un total pondéré
indicatif.

### Critères d'acceptation

1. **Simulation avant la carte** : `proposer` joue le lot sur le document (instantané
   `serialize`), relève le résultat, puis remet l'original. La carte dit route par route
   « posée » ou « refusée : raison », et montre l'aperçu dessiné. `appliquer` charge le
   résultat relevé si le document n'a pas changé (`S.ver` identique) ; sinon, la carte
   est refusée (à reproposer). Le résultat ne dépend donc pas de la vitesse du Pi (garde
   de 250 ms du routeur).
2. **Une étape d'historique** pour tout le lot : Ctrl+Z enlève toutes les routes, vias et
   cuivre poussé du lot.
3. **Tout ou rien par route** : une route refusée ne laisse rien ; les autres routes du lot
   restent. Les raisons viennent de l'état du routeur (`S.route.bad`, `net` vide au
   départ, via refusée par `placeVia`, paire croisée), pas des messages `hint()`.
4. **Départ exact** : `startRoute(x, y, true)` ; un départ hors du cuivre d'un net est
   refusé (aujourd'hui la page crée une route sans net). L'échelle de la vue est fixée
   pendant la simulation, pour que les accroches de la paire ne dépendent pas du zoom ; la
   vue de l'utilisateur est remise ensuite.
5. **Réglages** : `S.avoid` forcé à vrai, coins à 45° ; le mode `pousser`/`contourner` vaut
   pour le lot, puis le réglage de l'utilisateur revient (sans étape d'historique).
6. **DRC après application** (comme `ecriture-cao`) : erreurs avant → après, liaisons
   restantes.
7. Bornes : 20 routes, 200 points par route, 50 vias seuls par lot ; coordonnées à
   ±1000 mm ; couches existantes.

## Architecture

- `40-routage-pilote.js` est l'adaptateur mince prévu par l'étude du 2026-10-16
  (~150 lignes) : `startRoute`, `routeToPoint`, `stepRoute`, `routeToLayer`,
  `commitRoute`, `cancelRoute`, `placeVia`, `dpStart…dpCommit`, puis un seul instantané
  pour le lot.
- Placement : `autoPlace`, `placerSatellites`, `BLOC_PLACEMENT` ; score de placement par
  `PLACEMENT_SCORE` → `/api/pcb/score-placement`.
- Score de routage : `conn(true)`, `runDrc`, `cmMesures`/`cmVerifier`, `topoVerifier`,
  `cmEvaluerGroupe`.

## Commandes

```
Bundle :  python3 editeur-pcb/outils/build-monofichier.py
Bancs :   node editeur-pcb/test/harness.js ; node commun/test/banc-pilote-cao.js
MCP :     python3 lanceur/banc_mcp.py
```

## Stratégie de test

- Harnais PCB : sur l'exemple, router trois liaisons par points de passage dont une avec
  changement de couche ; une route bloquée refusée sans trace ; Ctrl+Z enlève le lot ; une
  paire ; un départ hors cuivre refusé ; le résultat appliqué est celui de l'aperçu.
- Score de routage : valeurs attendues sur une carte abîmée.
- Essai réel : Claude route une petite carte d'une copie de PROJETS jusqu'à zéro liaison
  restante, en bouclant sur `cao_score`.

## Limites

- **Toujours** : simuler avant la carte ; un lot = une étape ; remettre la vue et les
  réglages de l'utilisateur.
- **Demander d'abord** : changer le routeur lui-même (`11-pns-*.js` à `13-pns-shove.js`) ;
  un chercheur de chemin (module `chemin-auto`).
- **Jamais** : router sans carte quand l'interrupteur est éteint ; enregistrer le projet.

## Questions ouvertes

Aucune.
