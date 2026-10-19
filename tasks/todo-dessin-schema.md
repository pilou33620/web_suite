# Tâches : dessin-schema

Spec : [lanceur/docs/SPEC-mcp-dessin-schema.md](../lanceur/docs/SPEC-mcp-dessin-schema.md) ·
Plan : [plan-dessin-schema.md](plan-dessin-schema.md)

Bundle à reconstruire avant le harnais : `python3 editeur-schematique/outils/build-monofichier.py`

## Tâche 1 : ✅ socle du dessin, poser un type intégré, déplacer, tourner, supprimer

**Description :** `editeur-schematique/js/28-dessin-pilote.js` déclare
`cao_schema_dessiner` sur `ECRITURE_CAO.lot` (plafond de 40). `verifier` simule le lot,
puis remet le document. Opérations de cette tâche :
- `poser` avec `type`, `ref`, `valeur`, `x`, `y`, `rotation`, `miroir` ; sans position,
  une place libre ;
- `deplacer`, `tourner`, `miroir`, `supprimer`.

**Critères d'acceptation :**
- [x] Proposer ne change rien : `serialize()`, `S.hist`, `S.dirty`, la sélection et la
      vue sont identiques après la carte. Appliquer fait une étape, et Ctrl+Z remet tout
- [x] Poser trois composants : repères donnés ou tirés par `nextRef`, positions en mm sur
      la grille, chevauchement refusé ; sans position, la place libre ne chevauche rien ;
      un repère posé sert aux opérations suivantes du lot
- [x] Déplacer, tourner et miroir font suivre les fils, comme à la souris. Repère, type
      ou angle inconnu : refus de tout le lot, rien n'est touché

**Vérification :** `node editeur-schematique/test/harness.js`, `node commun/test/banc-pilote-cao.js`

**Dépendances :** aucune · **Fichiers :** `editeur-schematique/js/28-dessin-pilote.js`,
`commun/ecriture-cao.js`, `commun/pilote-cao.js`, `editeur-schematique/editeur-schematique.html`,
`editeur-schematique/test/harness.js` · **Taille :** M

## Tâche 2 : ✅ poser depuis le catalogue

**Description :** extraire de la page `schTypeDuCatalogue(ligne)` (explorateur) et
`schPoserDepuisCatalogue(el, ligne)` (clic de pose). La souris et l'explorateur passent
par elles, et `poser {catalogue: "Part Name"}` aussi, avec les lignes de
`window.CSV_LIB`.

**Critères d'acceptation :**
- [x] Une même ligne de catalogue posée à la souris et par Claude donne le même
      composant : type, valeur, boîtier, MPN, brochage, specs
- [x] Part Name inconnue : refus qui nomme `lib_composants` ; catalogue pas chargé :
      refus clair ; symbole inconnu de la page : symbole générique, et la carte le dit
- [x] Bancs existants de pose et de brochage verts

**Vérification :** `node editeur-schematique/test/harness.js`

**Dépendances :** T1 · **Fichiers :** `editeur-schematique/js/09-interaction.js`,
`editeur-schematique/js/14-clavier-boutons.js`, `28-dessin-pilote.js`, harnais · **Taille :** M

## Tâche 3 : ✅ relier, supprimer_fil, pas de connexion par accident

**Description :**
- `relier {de, a}` trace le fil en L (`routeL`), puis `touchWires` et `resolveSplits` ;
- `supprimer_fil {de, a}` enlève les segments qui relient directement ces deux broches,
  jusqu'aux jonctions ;
- garde : broche → net relevé avant et après chaque opération ; une fusion non demandée
  refuse le lot.

**Critères d'acceptation :**
- [x] `relier R1.2 → R2.1` : les deux broches sont sur le même net, et la carte le dit
      (« R1.2 — R2.1 sur N$… »)
- [x] Un fil qui passerait sur la broche d'un troisième composant est refusé, et la
      raison nomme cette broche ; rien n'est touché
- [x] `supprimer_fil` défait un `relier` (les deux broches retrouvent leurs nets
      séparés) ; sans fil direct entre elles : refus

**Vérification :** `node editeur-schematique/test/harness.js`

**Dépendances :** T1 · **Fichiers :** `28-dessin-pilote.js`, harnais · **Taille :** M

## Tâche 4 : ✅ étiqueter

**Description :** `etiqueter {broche, net, genre: local|global|vcc|gnd}` pose `port`,
`gport`, `vcc` ou `gnd` au bout d'un court fil (2 mm) qui part vers l'extérieur du
composant. Le symbole est tourné pour que sa broche touche le fil.

**Critères d'acceptation :**
- [x] Les quatre genres, sur des broches qui partent dans les quatre directions : la
      broche rejoint le net nommé (`docNets()`), global ou local selon le genre
- [x] Un nom de net vide ou de plus de 40 caractères est refusé ; une étiquette qui
      toucherait autre chose est refusée (même garde que T3)

**Vérification :** `node editeur-schematique/test/harness.js`

**Dépendances :** T3 · **Fichiers :** `28-dessin-pilote.js`, harnais · **Taille :** S

## Tâche 5 : ✅ feuilles et renuméroter

**Description :**
- `feuille {creer: nom}` ajoute une feuille, qui devient celle des opérations suivantes ;
- `feuille {renommer: n, nom}` renomme ;
- chaque opération peut viser une `feuille` (n°), avec `gotoPage` ;
- `renumeroter {feuille?}` refait les repères par préfixe, dans l'ordre de lecture.

**Critères d'acceptation :**
- [x] Créer une feuille, y poser et relier deux composants en un lot ; Ctrl+Z enlève la
      feuille et son contenu
- [x] Renuméroter sur deux feuilles : repères uniques, ordre haut → bas puis gauche →
      droite, feuille par feuille ; la carte montre la table ancien → nouveau et prévient
      pour le PCB
- [x] Feuille inconnue : refus

**Vérification :** `node editeur-schematique/test/harness.js`

**Dépendances :** T1 · **Fichiers :** `28-dessin-pilote.js`, harnais · **Taille :** M

## Checkpoint : dessin dans la page
- [x] Bancs verts, bundle reconstruit ; un petit montage dessiné en un lot (régulateur,
      deux condensateurs, VCC, GND) a les nets attendus

## Tâche 6 : ✅ outil MCP

**Description :** `cao_schema_dessiner` dans `lanceur/mcp_ecriture_cao.py` :
- `operations` : 1 à 40, de forme vérifiée par opération (clés connues, types, mm, angles
  0/90/180/270) ;
- envoyé à la page `schema`, en écriture.

Documenté dans `DETAILS.md`.

**Critères d'acceptation :**
- [x] Chaque opération acceptée arrive telle quelle ; une opération mal formée est refusée
      sans requête (`banc_mcp.py`) ; 52 outils
- [x] La description dit : une carte, les unités, les broches par numéro ou par nom, la
      garde contre les connexions par accident, donnée ≠ consigne

**Vérification :** `python3 lanceur/banc_mcp.py`, `node lanceur/banc_pilote.js`

**Dépendances :** T2…T5 · **Fichiers :** `lanceur/mcp_ecriture_cao.py`, `lanceur/banc_mcp.py`,
`lanceur/docs/DETAILS.md` · **Taille :** S

## Tâche 7 : ✅ essai réel, puis revue

**Description :** sur une copie de PROJETS, Claude dessine par le MCP un petit montage
dans une nouvelle feuille de « carte PIR v2 » : une carte cliquée, puis l'interrupteur
allumé. Captures, lecture des nets, Ctrl+Z. Ensuite viennent `/review` et l'audit de
sécurité.

**Critères d'acceptation :**
- [x] Le montage a les nets voulus (`cao_schema_lire nets`) ; Ctrl+Z l'enlève ; PROJETS
      n'a pas bougé
      (« carte PIR v2 », feuille « Essai Claude » : LDO du catalogue REG_LD39050PU25R, deux
      condensateurs, 5V, 2V5, GND, en une carte cliquée puis interrupteur allumé ;
      IN → 5V, OUT → 2V5, GND → GND ; Ctrl+Z retire la feuille ; PROJETS au même md5.
      Trouvé et corrigé :
      - le symbole du catalogue, écrit avec son chemin (« lib/symbole/regulator.json »),
        retombait sur un CI générique, même à la souris ;
      - les étiquettes se posaient en travers des fils déjà tracés : elles cherchent
        maintenant une direction libre.)
- [x] `/review` fait, corrections appliquées (revue 5 axes + audit de sécurité : rien de
      critique à l'audit ; la revue avait vu des trous dans la garde. Corrigé :
      - poser, déplacer, tourner et miroir passent la garde ; poser refuse étiquettes et
        alimentations (etiqueter seul nomme un net) ;
      - la garde compare aussi les noms de nets ;
      - deux noms réunis et deux broches déjà reliées sont refusés ;
      - renuméroter garde les parties d'un boîtier sous un repère et signale les repères
        repris ;
      - la simulation travaille sur un clone, sans restore() : objets, geste en cours et
        sélection gardés ;
      - supprimer_fil essaie chaque branche et refuse un chemin qui passe par une broche ;
      - noms réservés refusés ; le symbole inconnu est dit sur la carte ;
      - la pile de rétablissement survit à un échec, aussi dans la visionneuse ;
      - netlist : l'empreinte sur une ligne ; une pose de catalogue part avec le changement
        de feuille.
      Laissé : les étiquettes n'ont pas de repère, seul Ctrl+Z les retire.)

**Dépendances :** T6 · **Taille :** M

## Checkpoint : terminé
- [x] Critères de la spec remplis ; 4 CI vertes (WEB_CAO ae81829, web_suite 1b532e6)
