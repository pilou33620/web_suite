# Plan : dessin-schema

Spec : [lanceur/docs/SPEC-mcp-dessin-schema.md](../lanceur/docs/SPEC-mcp-dessin-schema.md)
(validée le 2026-10-19). Liste des tâches : [todo-dessin-schema.md](todo-dessin-schema.md).

## Vue d'ensemble

Un outil, `cao_schema_dessiner` : une liste ordonnée d'opérations en une carte.
- Opérations : poser, relier, étiqueter, déplacer, tourner, miroir, supprimer,
  supprimer_fil, feuille, renuméroter.
- La carte compte pour une étape d'historique.

Le lot est simulé avant la carte, et la connectivité obtenue est comparée à celle
demandée.

## Graphe des dépendances

```
T1 socle du dessin : simulation du lot, poser (type intégré), déplacer / tourner / miroir / supprimer
 ├── T2 poser depuis le catalogue (fonctions extraites de la page)
 ├── T3 relier, supprimer_fil, garde « pas de connexion par accident »
 │    └── T4 étiqueter (port, port global, VCC, GND)
 └── T5 feuilles (créer, renommer, feuille par opération) et renuméroter
           └── T6 outil MCP  ← T2…T5
                └── T7 essai réel, puis revue
```

## Décisions d'architecture

- **Un fichier de page**, `editeur-schematique/js/28-dessin-pilote.js`, avec la commande
  `cao_schema_dessiner`. Il est construit sur le socle `ECRITURE_CAO.lot`
  (`commun/ecriture-cao.js`), qui apporte :
  - la carte, et la revérification au clic par les lignes ;
  - un seul `push()` ;
  - le retour en arrière si une opération échoue en route.

  Le socle gagne un plafond par commande (`max`, 40 ici) : c'est son seul changement.
- **Simulation** : `verifier` joue le lot sur le document vivant, après `serialize()`.
  - Il relève les lignes de la carte (opérations faites, repères posés, connexions
    obtenues), puis remet le document par `restore()`.
  - `S.dirty`, `S.hist`, `S.redo`, la sélection et la vue sont remis tels quels :
    proposer ne doit rien laisser.
  - Le jeu est déterministe (repères par `nextRef`, positions sur la grille) : au clic,
    le même jeu redonne les mêmes lignes, puis s'applique pour de bon.
- **Unités** : mm côté Claude, ×20 px côté page, puis `snap()` sur `S.grid`.
- **Broches** : `REF.numéro` ou `REF.nom`, résolues comme dans les lectures de
  registre-cao (`allPins`, `brNomBroche`).
- **Un repère posé dans le lot** peut servir aux opérations qui suivent dans le même lot
  (poser R5 puis relier R5.1) : c'est ce qui permet de dessiner un montage en une carte.
- **Pas de connexion par accident** : avant et après chaque `relier` ou `etiqueter`, la
  page relève broche → net sur la feuille (`nets()`). Seule la fusion demandée est
  admise (les deux broches, ou la broche et le net de l'étiquette) ; toute autre fusion
  refuse le lot et nomme les broches touchées.
- **Fonctions de la page, reprises ou extraites** :
  - reprises : `addComp`, `nextRef`, `routeL`, `touchWires`, `resolveSplits`,
    `transformSel` (tourner, miroir ; les fils suivent), `moveSelBy` (déplacer),
    `deleteComps`, `newPage`, `gotoPage`, `bbox` ;
  - extraites, comme `fpReposerBoitier` à ecriture-cao : la pose depuis le catalogue
    (`schTypeDuCatalogue`, tirée du rappel de l'explorateur de bibliothèque, et
    `schPoserDepuisCatalogue`, tirée du clic de pose de `09-interaction.js`). La souris,
    l'explorateur et Claude passent par elles.
- **Nouveau** : `renumeroter` (ordre de lecture, feuille par feuille), et la recherche
  d'une place libre (vers la droite, puis la rangée suivante, sur la grille).

## Risques

| Risque | Impact | Parade |
|---|---|---|
| `restore()` après la simulation laisse une trace (drapeau, onglets, analyse des motifs) | Moyen | État relevé et remis champ par champ ; test « proposer ne change rien » sur `serialize()`, `S.hist`, `S.dirty` |
| Un fil en L passe sur une broche voisine et la relie (la connectivité relie tout point posé sur un segment) | Élevé | Comparaison broche → net avant/après ; refus nommé ; Claude peut étiqueter à la place |
| Extraire la pose depuis le catalogue change le geste à la souris | Moyen | Fonctions extraites telles quelles ; les bancs existants (pose, brochage depuis la LIB) restent verts ; test qui compare souris et Claude sur une même ligne de catalogue |
| Renuméroter casse le lien avec le PCB | Moyen | La carte le dit ; rien n'est touché au PCB |

## Questions ouvertes

Aucune.
