# Tâches : routage

Spec : [lanceur/docs/SPEC-mcp-routage.md](../lanceur/docs/SPEC-mcp-routage.md) ·
Plan : [plan-routage.md](plan-routage.md)

Bundle à reconstruire avant le harnais : `python3 editeur-pcb/outils/build-monofichier.py`

## Tâche 1 : lectures `liaisons` et `zone`

**Description :** `LECTURE_CAO.pcb` gagne deux lectures :
- `liaisons` : les liaisons à router, de `REF.pastille` à `REF.pastille`, avec
  coordonnées en mm, couche, net et classe, 150 au plus ;
- `zone {x1, y1, x2, y2}` : les pistes, vias, pastilles, zones et le contour présents dans
  la fenêtre, 200 objets au plus.

**Critères d'acceptation :**
- [ ] Sur l'exemple 1, `liaisons` donne autant de lignes que `conn(true).rats` (moins les
      liaisons de zone), chacune avec ses deux pastilles ; `filtre` sur le net
- [ ] `zone` sur une fenêtre autour de U1 liste ses pastilles et les pistes qui la
      traversent ; fenêtre vide ou inversée : message clair ; bornes respectées
- [ ] Le relevé de registre-cao (sorties de Gemini) reste identique

**Vérification :** `node editeur-pcb/test/harness.js`

**Dépendances :** aucune · **Fichiers :** `commun/lecture-cao.js`, `commun/pilote-cao.js`,
`editeur-pcb/test/harness.js` · **Taille :** S

## Tâche 2 : socle « simuler puis charger » et `cao_router`

**Description :** `editeur-pcb/js/37-routage-pilote.js`. Le socle simule le lot, relève le
document, remet l'original, garde l'aperçu, puis au clic charge le relevé en une étape.
`cao_router` : routes (départ, points, arrivée, largeur) et vias seules ; mode `pousser`
ou `contourner`. Le DRC avant → après est donné dans la réponse.

**Critères d'acceptation :**
- [ ] Sur l'exemple 1, trois liaisons routées par points de passage, dont une qui change
      de couche (via) ; la carte dit « posée » pour chacune ; une seule étape `S.undo` ;
      Ctrl+Z enlève tout ; le document appliqué est celui de l'aperçu (`serialize()` égal)
- [ ] Une route bloquée ou un départ hors cuivre : « refusée : raison », rien d'elle ne
      reste, les autres routes du lot sont posées
- [ ] Proposer seul ne laisse rien : document, piles d'historique, drapeau, sélection,
      vue, réglages du routeur ; le document qui bouge entre la carte et le clic fait
      refuser la carte

**Vérification :** `node editeur-pcb/test/harness.js`

**Dépendances :** T1 · **Fichiers :** `37-routage-pilote.js`, `03-render.js` (une ligne),
`commun/ecriture-cao.js` (export du bilan DRC), `editeur-pcb/editeur-pcb.html`,
`commun/pilote-cao.js`, harnais · **Taille :** L

## Tâche 3 : `cao_paire`

**Description :** une paire déclarée, routée avec `dpStart`, puis pour chaque point
`dpUpdate` et `dpStep`, puis `dpCommit`. Elle passe par le même socle.

**Critères d'acceptation :**
- [ ] Une paire de l'exemple routée en un lot ; une étape ; Ctrl+Z l'enlève
- [ ] Paire inconnue, départ hors des deux nets, jambes croisées : refus avec la raison

**Vérification :** `node editeur-pcb/test/harness.js`

**Dépendances :** T2 · **Fichiers :** `37-routage-pilote.js`, harnais · **Taille :** M

## Tâche 4 : `cao_placement_auto`

**Description :** `methode` vaut `forces` (`autoPlace`), `satellites`
(`placerSatellites`) ou `blocs` (`calculerAgencement` d'un motif nommé). Il passe par le
même socle : une carte, une étape.

**Critères d'acceptation :**
- [ ] Chaque méthode, sur l'exemple : une étape, Ctrl+Z remet les positions ; la carte dit
      combien d'empreintes bougent
- [ ] `blocs` sans motif publié : refus clair ; méthode inconnue : refus

**Vérification :** `node editeur-pcb/test/harness.js`

**Dépendances :** T2 · **Fichiers :** `37-routage-pilote.js`, harnais · **Taille :** S

## Tâche 5 : `cao_score`

**Description :**
- `routage` : le tableau de la spec (liaisons restantes, DRC, vias, longueur,
  contraintes, topologie, groupes) et un total indicatif ;
- `placement` : le score du serveur, résumé.

**Critères d'acceptation :**
- [ ] `routage` sur l'exemple, puis sur une carte abîmée : les valeurs attendues
      (liaisons, erreurs, vias, longueur au centième)
- [ ] `placement` : le document envoyé est celui de `PLACEMENT_SCORE` ; une réponse en
      erreur du serveur devient un refus clair
- [ ] Aucune écriture (lecture)

**Vérification :** `node editeur-pcb/test/harness.js`

**Dépendances :** T2 · **Fichiers :** `37-routage-pilote.js`, `20-placement-score.js` (une
ligne d'export), harnais · **Taille :** M

## Checkpoint : routage dans la page
- [ ] Bancs verts, bundle reconstruit

## Tâche 6 : outils MCP

**Description :** `lanceur/mcp_routage.py` contient `cao_router`, `cao_paire`,
`cao_placement_auto` et `cao_score`, avec la forme des arguments vérifiée. `cao_pcb_lire`
gagne `liaisons` et `zone`.

**Critères d'acceptation :**
- [ ] Bonne commande, bonne page, écriture ou lecture selon l'outil (`banc_mcp.py`) ;
      arguments invalides refusés sans requête ; 58 outils
- [ ] Les descriptions disent la méthode : points de passage, un lot = une carte, tout ou
      rien par route, et l'aperçu qu'on peut capturer

**Vérification :** `python3 lanceur/banc_mcp.py`, `node lanceur/banc_pilote.js`

**Dépendances :** T2…T5 · **Fichiers :** `lanceur/mcp_routage.py`, `lanceur/mcp_registre_cao.py`,
`lanceur/mcp_web_suite.py`, `lanceur/banc_mcp.py`, `lanceur/docs/DETAILS.md` · **Taille :** M

## Tâche 7 : essai réel, puis revue

**Description :** sur une copie de « carte PIR v2 », Claude route quelques liaisons par le
MCP, en bouclant sur `cao_score` et sur l'aperçu capturé. Une carte est cliquée, puis
l'interrupteur est allumé. On mesure la durée d'une carte de 20 routes sur le Pi. Ensuite
viennent `/review` et l'audit de sécurité.

**Critères d'acceptation :**
- [ ] Des liaisons routées pour de vrai ; le score baisse ; Ctrl+Z ; PROJETS n'a pas bougé
- [ ] `/review` fait, corrections appliquées

**Dépendances :** T6 · **Taille :** M

## Checkpoint : terminé
- [ ] Critères de la spec remplis ; 4 CI vertes
