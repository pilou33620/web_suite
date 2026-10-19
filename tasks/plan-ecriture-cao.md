# Plan : ecriture-cao

Spec : [lanceur/docs/SPEC-mcp-ecriture-cao.md](../lanceur/docs/SPEC-mcp-ecriture-cao.md)
(validée le 2026-10-19). Liste des tâches : [todo-ecriture-cao.md](todo-ecriture-cao.md).

## Vue d'ensemble

Cinq écritures par cartes en lot dans les pages WEB_CAO :
- composants au schéma et au PCB ;
- pistes, classes, contraintes ;
- paramètres de simulation.

D'abord un socle commun, la carte en lot. Il est éprouvé sur le schéma, puis chaque
écriture du PCB et la simulation viennent dessus. Les outils MCP ferment la marche, comme
pour registre-cao.

## Graphe des dépendances

```
T1 socle « carte en lot » (commun/ecriture-cao.js) + composants du schéma
 ├── T2 composants du PCB (+ DRC avant → après)
 │    ├── T3 pistes
 │    ├── T4 classes et affectations
 │    └── T5 paires et contraintes
 └── T6 paramètres de simulation (annuler par la carte)
           └── T7 outils MCP (lanceur/mcp_ecriture_cao.py)  ← T2…T6
                └── T8 essai réel, puis revue
```

## Décisions d'architecture

- **Un fichier neuf, `commun/ecriture-cao.js`**, chargé après `lecture-cao.js` et avant
  `pilote-cao.js` dans les trois pages. Pour le schéma et le PCB, `pilote-cao.js` ne
  gagne que l'enregistrement des commandes. La simulation passe aussi par la visionneuse.
- **Le socle `ecLot(def)`** fabrique une commande d'écriture pilote-compatible :
  - `{ecriture, historique, proposer, appliquer}` ;
  - pour la simulation : `{ecriture, annuler, etat, proposer, appliquer}`.

  Une écriture déclare ce qu'elle sait faire, et le socle fait le reste :
  - `verifier(args)` rend `{lignes:[{quoi, avant, apres}], refus:[…]}` sans rien toucher ;
  - `muter(args)` applique, sans `push()`.

  Le socle donne :
  - le refus au-delà de 50 changements ;
  - le refus de tout le lot au premier refus, avec la liste des refus ;
  - la carte « avant → après » ;
  - au clic, une seconde vérification, puis un seul `push()`, `muter`, un seul
    rafraîchissement.
- **Document changé entre la carte et le clic** : `proposer` garde une empreinte du
  document, un hachage court de `JSON.stringify(serialize())`. `appliquer` refuse si
  elle a changé (« le document a changé : reproposer »), comme le fait déjà
  `antenna_proposer` avec `attendu`.
- **DRC au PCB** : `proposer` compte les erreurs (`runDrc()` est synchrone, 0,3 s).
  Après `muter`, `appliquer` relance `runDrc()` et ajoute à `prop.apres` la ligne
  « DRC : N erreurs avant → M après » et les nouvelles erreurs (80 lignes au plus).
  `pilote.js` renvoie `prop.apres` dans sa réponse « appliquee » : la réponse arrive à
  Claude **sans changer `pilote.js`**, dont les copies sont vérifiées par la CI dans
  trois dépôts.
- **Validation par les fonctions de la page** : on normalise, et si la normalisation
  change une valeur donnée par Claude, c'est un refus. Les normaliseurs repris :
  - `normClass` (classes) ;
  - `normDpRule` (règles de paire) ;
  - `cmNormRegle` (contraintes) ;
  - `pkgGeom` (un boîtier hors table est refusé, alors que la page l'ignore en silence) ;
  - `simDsPreparer` (simulation).
- **Fonctions de la page reprises** :
  - PCB : `transformFps` (face, position par `wxu`/`wyu`, rotation), `applyPkgGeom`,
    `setNetClass`, `classWidth` ;
  - schéma : `reshapeComp` (valeur d'un CI) ;
  - simulation : `simDsEcrire` / `simDsLire` pour l'annulation.
- **Simulation** :
  - `appliquer` passe par `simDsAppliquer(prep, ids)`, qui garde ses effets (assistant
    ΔI, oubli des résultats périmés) ;
  - `etat` = les valeurs des champs touchés ;
  - `annuler` les remet par `simDsEcrire`.

  L'unité donnée par Claude doit être celle du catalogue (`ch.unite`) : pas de
  conversion.
- **MCP** : `lanceur/mcp_ecriture_cao.py`, sur le modèle de `mcp_registre_cao.py`
  (`refuser`, `choix`, `booleen`, `texte`, `entier`, `envoyer`), avec
  `ecriture: True`.

## Risques

| Risque | Impact | Parade |
|---|---|---|
| Renommer un repère au PCB casse le lien avec le schéma (cross-probe, synchronisation) | Moyen | La carte le dit ; refus d'un repère déjà pris ; essai réel sur le lien |
| Changer le boîtier au PCB refait les pastilles ; des pistes restent en l'air | Moyen | DRC avant → après dans la réponse ; `fpReshape` garde les pistes par numéro |
| `transformFps` déplace le cuivre selon `S.moveEtch`, une préférence de l'utilisateur | Faible | Dit dans la description de l'outil ; pas de changement de la préférence |
| `simDsAppliquer` a des effets de bord (assistant ΔI allumé) que `annuler` ne défait pas | Moyen | `etat` et `annuler` couvrent les champs ; l'effet ΔI est écrit sur la carte ; test dédié |
| L'empreinte du document (`serialize`) coûte sur une grosse carte | Faible | Une fois à la carte, une fois au clic ; mesuré à l'essai réel (carte PIR v2) |

## Questions ouvertes

Aucune pour le plan. La question de la spec (le panneau Propriétés du schéma écrit sans
étape d'historique) reste notée, hors de ce module.
