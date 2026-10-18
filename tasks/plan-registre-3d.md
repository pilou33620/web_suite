# Plan d'implémentation : registre-3d

Spec : [lanceur/docs/SPEC-mcp-registre-3d.md](../lanceur/docs/SPEC-mcp-registre-3d.md) · Carte :
[lanceur/docs/SPEC-mcp.md](../lanceur/docs/SPEC-mcp.md) · Tâches : [todo-registre-3d.md](todo-registre-3d.md) ·
Module précédent : [plan.md](plan.md) (canal-pilote, essai iPad encore ouvert)

## Vue d'ensemble

Neuf commandes WEB_3D sur le canal de pilotage :
- lectures et réglages directs : scène, vue, visibilité, coupe, mesures, collisions ;
- écritures à carte : ouvrir, déplacer, ranger un STEP (irréversible).

Le calcul est écrit en fonctions pures dans `WEB_3D/js/14-pilote-3d.js`, testées sous
Node sur l'assemblage d'exemple. Le branchement dans la page reste mince, et les outils
MCP vivent dans `lanceur/mcp_registre_3d.py`.

## Décisions d'architecture

- **`js/14-pilote-3d.js`** a deux parties :
  - des **fonctions pures exportées**, testables sous Node : `trouverPiece`, `decrireScene`,
    `dimensions`, `distance`, `faces`, `collisions`, `deplacer`, `etatMatrices`,
    `remettreMatrices`, `extraireStep` ;
  - **`brancherPilote3d(W3D, Pilote)`**, qui déclare les commandes et fait le lien avec
    `ui`, `arbre`, `coupe` et `nav`. Il est appelé par `09-demarrage.js`, à la place de la
    déclaration actuelle.
- **Fonctions internes rendues réutilisables, sans les recopier** :
  - `apres()` de `13-deplacement.js` devient une fonction exportée
    `apresDeplacement(W3D, g)`, que le panneau Déplacer appelle aussi ;
  - `ouvrir()` et `compagnons()` de `11-projets.js` sont exportées.
  - La logique reste à un seul endroit.
- **Le canal d'abord** : `appliquer` et `annuler` asynchrones, et l'écriture
  `irreversible`, dans `lanceur/pilote.js`, recopié dans les trois outils, pour que la CI
  des copies reste verte.
- **Identification** : uuid, ou chemin d'indices sous `vue.modele`. Un nom n'est accepté
  que s'il est unique. Le chemin est calculé à la volée, jamais stocké.
- **Sorties** : uniquement des nombres, des chaînes et des tableaux, arrondis au µm ou au
  millième de degré. Le plafond de 100 Ko du MCP s'applique (`resumer`).
- **La cote de Claude** est un petit groupe (ligne et étiquette) dans `vue.annotations`,
  nommé `cote-claude`. Il est remplacé à chaque `web3d_mesurer`, et indépendant de l'outil
  Mesure.
- **Préférences intactes** : aucune commande n'appelle de réglage qui écrit dans
  `localStorage`. La tolérance de collision est passée en argument, et la projection est
  posée sur la vue, pas dans les préférences.

## Graphe de dépendances

```
T1 canal : appliquer async + irreversible (4 dépôts)
        │
T2 14-pilote-3d.js : trouverPiece, decrireScene + banc-pilote-3d.mjs (scène réelle de l'exemple)
        ├──▶ T3 mesures (dimensions, distance, faces) + cote
        ├──▶ T4 collisions
        ├──▶ T5 déplacement (translation, rotation, origine, contrainte) + matrices ; apresDeplacement exporté
        └──▶ T7 extraction STEP (relue par OpenCascade)
T6 ouvrir (11-projets exporté, annuler = recharger / retirer)  ◀── T1
        ▼
T8 brancherPilote3d : les 9 commandes dans la page (cartes, traces, vue, visibilité, coupe)
        ▼
T9 MCP mcp_registre_3d.py + banc_mcp ──▶ T10 CI WEB_3D + docs ──▶ T11 essai réel + /review
```

## Liste des tâches

### Phase 1 : canal et socle
- [ ] T1 : `pilote.js` : `appliquer`/`annuler` asynchrones, écriture `irreversible` (4 dépôts)
- [ ] T2 : `14-pilote-3d.js` : identification et scène, banc Node sur la vraie hiérarchie

### Checkpoint : socle
- [ ] Bancs verts (lanceur, WEB_3D) ; revue avec toi

### Phase 2 : le calcul (fonctions pures)
- [ ] T3 : mesures et cote
- [ ] T4 : collisions
- [ ] T5 : déplacement, contrainte, matrices ; `apresDeplacement` exporté
- [ ] T6 : ouvrir depuis PROJETS/3D, annulation
- [ ] T7 : extraction STEP

### Checkpoint : calcul
- [ ] Banc 3D vert ; revue avec toi

### Phase 3 : branchement et fin
- [ ] T8 : `brancherPilote3d` : les 9 commandes dans la page
- [ ] T9 : `lanceur/mcp_registre_3d.py`, banc MCP
- [ ] T10 : CI WEB_3D, documentation
- [ ] T11 : essai réel dans Chromium, puis `/review`

### Checkpoint : terminé
- [ ] Critères de la spec remplis ; CI verte dans les dépôts touchés

## Risques et parades

| Risque | Impact | Parade |
|---|---|---|
| La hiérarchie réelle (groupes repliés, noms OCCT) ne se reconstruit pas sous Node sans le Worker | Moyen | T2 : extraire de `02-import.js` la construction de scène à partir d'un résultat OCCT (fonction pure exportée) et la réutiliser dans le banc ; sinon, tester l'identification sur une scène construite à la main |
| `charger` ne rejette jamais : on ne sait pas si l'ouverture a échoué | Moyen | T6 : lire `#errDepot` et le nombre de groupes après `await` ; ou faire rendre un résultat à `charger` (petit changement, à faire si c'est plus propre) |
| Annuler un remplacement recharge les fichiers : lent, positions perdues | Faible (voulu) | La carte le dit ; l'état comparé est la liste des fichiers et leurs matrices |
| Collisions ralenties dans un onglet caché | Moyen | Mesurer à T11 ; le résultat le signale ; au-delà de 2 min, « en_attente » |
| `trouverProduit` du STEP choisit la mauvaise pièce quand des noms se répètent | Moyen | T7 : extraire par l'index de produit associé au maillage quand c'est possible ; sinon, refuser les noms en double avec un message clair |
| Le changement de `pilote.js` doit partir dans les 4 dépôts en même temps | Moyen (CI rouge entre deux) | Pousser les outils avant web_suite, comme à canal-pilote |

## Questions ouvertes

Aucune.
