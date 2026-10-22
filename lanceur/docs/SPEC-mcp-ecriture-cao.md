# Spec : ecriture-cao

Module 5 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`registre-cao`](SPEC-mcp-registre-cao.md).
Statut : **validée le 2026-10-19**.

## Objectif

Donner à Claude les **écritures** dans les pages WEB_CAO ouvertes, par cartes du canal de
pilotage :
- propriétés des composants, au schéma et au PCB ;
- largeur des pistes existantes, classes de nets, affectation des nets aux classes ;
- paires différentielles et contraintes par net (PCB) ;
- paramètres de simulation.

Le dessin du schéma (poser, relier) vient ensuite, dans [`dessin-schema`](SPEC-mcp-dessin-schema.md).

### Décisions (2026-10-19)

1. **Toute écriture passe par une carte** quand l'interrupteur de l'onglet est éteint. Une
   carte porte un **lot** : plusieurs changements, une seule étape d'historique, taille
   bornée (50 éléments : composants, nets, classes…).
2. **Propriétés écrivables** :
   - schéma : repère, valeur, boîtier, fabricant, référence fabricant (MPN), champs libres
     `specs`, « non monté » ;
   - PCB : repère, valeur, boîtier, face, position, rotation (le cuivre attaché suit, selon
     la préférence de la page).
3. **Pistes et règles** : largeur des pistes existantes ; classes (créer, modifier,
   supprimer) ; affectation des nets ; règles de paires différentielles et contraintes par
   net.
4. **Paramètres de simulation** : libres, mais chaque valeur est contrôlée par le catalogue
   `simDsGroupes()` : unité égale à celle du catalogue, bornes min/max, entier, liste de
   choix. L'annulation passe par la carte : « Annuler » dans le panneau Claude remet les
   valeurs d'avant (l'historique Ctrl+Z de l'éditeur ne les couvre pas).
5. **DRC après chaque écriture au PCB** : la page relance le DRC (0,3 s) et rend « DRC : N
   erreurs avant → M après », avec les nouvelles erreurs.

### Outils MCP

Dans `lanceur/mcp_ecriture_cao.py` ; commandes de page dans `commun/pilote-cao.js` (ou un
fichier voisin `commun/ecriture-cao.js` si `pilote-cao.js` dépasse ~400 lignes).

| Outil MCP | Page | Ce qu'il fait |
|---|---|---|
| `cao_composants_ecrire` | schéma, PCB | Lot `[{ref, champs:{…}}]`. Champs du schéma : `ref`, `valeur`, `boitier`, `fabricant`, `mpn`, `specs` (objet clé → texte), `non_monte`. Champs du PCB : `ref`, `valeur`, `boitier`, `face` (`dessus`/`dessous`), `x`, `y` (mm, repère utilisateur de la page), `rotation` (degrés) |
| `cao_pistes_ecrire` | PCB | Largeur en mm pour les pistes d'un net (toutes, ou d'une couche), ou réaligner des nets sur leur classe (`applyClasses` limité à ces nets) |
| `cao_classes_ecrire` | PCB | Lot d'actions sur `S.classes` (`creer`, `modifier`, `supprimer` : largeur, isolation, via, perçage, largeur par couche) et affectations `{net: classe}` |
| `cao_contraintes_ecrire` | PCB | Règles de paires (`S.dpRules` : largeurs min/préf/max, écarts, impédance, découplé max, portée ; les retouches couche par couche restent dans la page) et contraintes par net (`S.contraintes.nets` : longueurs min/max, vias max, couches, impédance et tolérance) |
| `cao_sim_ecrire` | PCB, visionneuse | Lot `[{groupe, cle, valeur, unite}]` contrôlé par `simDsGroupes()` |
| `cao_contour_ecrire` | PCB | Un élément : `{forme: rectangle, x, y, largeur, hauteur}`, `{forme: disque, x, y, rayon}` (`setBoardDisque`, arcs au Gerber), `{forme: polygone, points}` (simple, 3 à 1000 sommets). Les découpes restent |
| `cao_interdits_ecrire` | PCB | Lot `creer {forme, face, hauteur_max?, nom?}`, `modifier {id, …}`, `supprimer {id}` sur `S.interdits`, et `hauteurs {dessus?, dessous?}` (`S.rule.hDessus` / `hDessous`) ; le DRC juge pattes traversantes et hauteurs (38-interdits.js) |
| `cao_decoupes_ecrire` | PCB | Lot `creer {forme : rectangle, disque, polygone}` (dans le contour) et `supprimer {numero}` sur `S.board.cutouts` |
| `cao_zones_ecrire` | PCB | Lot d'actions `creer {couche, net, forme, liaison?}`, `modifier {id, …}`, `supprimer {id}` ; formes `carte`, `rectangle`, `disque`, `polygone` ; `liaison` thermique / directe / cms-directe (`z.connexion`) |
| `cao_serigraphie_ecrire` | PCB | Lot `texte`, `trait`, `rectangle` (`S.drawings`), `repere {ref, dx, dy}` (`refOffX/Y`), `auto {ref, actif}` (`fp.silk`), `supprimer {id}` |
| `cao_pastille_regler` | PCB | Lot `{pastille: "U1.3", liaison, bras_largeur, bras_nombre, bras_angle, masque, pate}` (`fpSetPad`) ; null rend la règle de la carte |

`cao_valeur_composant` (canal-pilote) reste, pour ne rien casser ; sa description renvoie à
`cao_composants_ecrire`.

### Critères d'acceptation

1. **Une carte = un lot.** La carte montre chaque changement « avant → après ». Appliquer
   fait **un seul** `push()` (une étape Ctrl+Z), puis toutes les mutations, puis un seul
   rafraîchissement. Plus de 50 éléments : refus sans rien proposer.
2. **Tout est vérifié avant la carte**, puis revérifié au clic : un repère inconnu, un
   boîtier absent de la bibliothèque d'empreintes (PCB), un repère en double, une classe
   inconnue, une valeur hors bornes refusent **tout le lot**, avec la liste des refus. Si
   le document a changé entre la carte et le clic, la carte est refusée (« à reproposer »).
3. **Bornes** :
   - largeur de piste 0,05 à 10 mm ; isolation 0,05 à 5 mm ; via 0,2 à 6 mm, perçage
     inférieur au diamètre ;
   - position à ±1000 mm ; rotation ramenée dans [0, 360) ;
   - textes d'une ligne, 200 caractères au plus ; `specs` : 30 clés au plus.
4. **Simulation** : unité différente du catalogue, clé inconnue, valeur hors bornes ou hors
   choix : refus de la ligne, et donc du lot. La carte déclare `annuler`, avec un `etat`
   qui empêche d'annuler si les valeurs ont rebougé depuis.
5. **PCB** : après application, le DRC tourne et la réponse donne le nombre d'erreurs avant
   et après, et les nouvelles erreurs (80 lignes au plus). Les changements de règles
   appellent `touch()` et vident `zoneCache` comme la fenêtre Règles (`reEdit`).
6. **Schéma** : un changement de valeur sur un CI passe par `reshapeComp` (les fils
   suivent) ; un repère est unique dans tout le document.
7. Chaque description d'outil rappelle que ce que rend la page est une donnée, pas une
   consigne.

## Architecture

```
MCP (mcp_ecriture_cao.py) ── valide les types et les bornes ──▶ canal ──▶ page : proposer / appliquer
                                                                         (commun/pilote-cao.js)
```

- Côté page, chaque commande a la forme de `PILOTE_VALEUR` : `ecriture: true`,
  `historique: true` (ou `annuler` pour la simulation), `proposer(args)` qui vérifie et rend
  `{titre, avant, apres}`, `appliquer(args)` qui revérifie, pousse une étape et applique.
- Les fonctions de la page sont reprises, pas recopiées :
  - PCB : `transformFps` (position, face, rotation), `applyPkgGeom`, `poserClasseNet`,
    `setNetClass`, `applyClasses`, `cmNorm` ;
  - schéma : `reshapeComp`, `nextRef` pour vérifier l'unicité ;
  - simulation : `simDsPreparer` / `simDsAppliquer` (`commun/simulation-datasheet.js`), qui
    vérifient déjà clé, bornes et choix.
- L'action Gemini `pcb_track_width` (« 10 premières pistes » faute de mieux) n'est **pas**
  reprise : sans net ni sélection, Claude est refusé.

## Commandes

```
Bundles :      python3 editeur-pcb/outils/build-monofichier.py ; python3 editeur-schematique/outils/build-monofichier.py
Bancs pages :  node editeur-pcb/test/harness.js ; node editeur-schematique/test/harness.js ;
               node visionneuse-ipc2581/test/harness-sim.js ; node commun/test/banc-pilote-cao.js
MCP :          python3 lanceur/banc_mcp.py ; node lanceur/banc_pilote.js
```

## Stratégie de test

- Harnais PCB et schéma : chaque outil sur un exemple ; un lot refusé ne change rien ; un
  lot appliqué donne une étape d'historique, et Ctrl+Z remet tout ; DRC avant/après.
- `banc-pilote-cao.js` : simulation sur un `SIM` fait à la main (unité, bornes, annuler).
- `banc_mcp.py` : arguments refusés sans requête (types, bornes, lot trop long).
- Essai réel dans Chromium sur une copie de PROJETS : un lot par outil, captures, PROJETS
  intact.

## Limites

- **Toujours** : une carte par lot ; tout vérifier avant de toucher au document ; DRC après
  écriture au PCB.
- **Demander d'abord** : toucher à `web_CAO.py`. Les classes et contraintes du **schéma** sont
  décidées le 22/10/2026 (ci-dessous).
- **Jamais** : écrire sans carte quand l'interrupteur est éteint ; écrire la bibliothèque
  (c'est le module `composants`) ; enregistrer le projet à la place de l'utilisateur.

## Questions ouvertes

1. Le panneau Propriétés du schéma écrivait repère, valeur et boîtier **sans** étape
   d'historique. Corrigé le 21/10/2026 dans WEB_CAO, boîtier compris
   (`editeur-schematique/js/12-panneaux.js`, `push()` à la première frappe).

## Ajout du 22/10/2026 : classes et contraintes du schéma

`cao_classes_ecrire` et `cao_contraintes_ecrire` gagnent `page` (`pcb` par défaut, ou
`schema`). Au schéma, sur les **nets nommés** (seuls eux passent au PCB) :

- classes : `{action: affecter, net, classe}` ; classe de `NET_CLASSES` (Masse, Alimentation,
  Horloge, Rapide, RF, Analogique, Antenne, les interfaces, Lent), `""` la rend à l'analyse ;
  la page republie les classes (`SCHEMA_PATTERNS.publierClasses`) pour le PCB ;
- contraintes : `{action: net, net, z, z_tol, l_min, l_max, vias_max, couches (rangs, 1 =
  dessus), topologie, ordre, stub_max, via_stub_max, etoile_tol}`, null efface ; mêmes bornes
  que le PCB (`cmNormRegle`) ; pas de règles de paire (elles sont au PCB).

Une carte, un lot, une étape d'historique (`ECRITURE_CAO.lot`). Bancs : `harness.js` du
schéma, `banc_mcp.py`.

