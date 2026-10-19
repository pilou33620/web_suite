# Spec : dessin-schema

Module 6 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`ecriture-cao`](SPEC-mcp-ecriture-cao.md).
Statut : **validée le 2026-10-19**.

## Objectif

Laisser Claude **dessiner** le schéma ouvert : poser des composants, les relier, poser des
étiquettes de net, déplacer, tourner, supprimer, gérer les feuilles, renuméroter les
repères. Chaque dessin est un lot sur une carte, comme les écritures d'`ecriture-cao`.

### Décisions (2026-10-19)

1. **Poser** : par ligne du catalogue LIB (« Part Name » : valeur, boîtier, MPN, brochage
   repris comme le clic dans l'explorateur de bibliothèque), ou par type intégré
   (`resistor`, `opamp`, `gnd`, `port`…).
2. **Relier** : broche à broche (« R1.2 → U1.3 », la page trace le fil en L comme la
   souris) et par étiquettes de net (port local, global, VCC, GND posés sur une broche).
   Pas de fils par coordonnées.
3. **Placement** : position donnée par Claude (mm, calée sur la grille) ; sans position, la
   page cherche une place libre à côté du dernier composant posé. Un chevauchement est
   refusé.
4. **Opérations** : déplacer, tourner, miroir, supprimer, feuilles (créer, renommer),
   renuméroter les repères (fonction nouvelle).

### Outil MCP

Un seul outil, `cao_schema_dessiner`, dans `lanceur/mcp_ecriture_cao.py`. Il prend une
liste ordonnée d'opérations ; c'est **une carte, une étape d'historique**, 40 opérations
au plus.

| Opération | Arguments | Effet |
|---|---|---|
| `poser` | `catalogue` (Part Name) **ou** `type` ; `ref`, `valeur`, `x`, `y`, `rotation` (0/90/180/270), `miroir`, `feuille` facultatifs | `addComp` + champs du catalogue ; repère par `nextRef` si absent |
| `relier` | `de` « REF.broche », `a` « REF.broche » | Fil en L (`routeL`), puis `touchWires`, `resolveSplits` |
| `etiqueter` | `broche` « REF.broche », `net`, `genre` (`local`, `global`, `vcc`, `gnd`) | `port` / `gport` / `vcc` / `gnd` posé au bout d'un court fil |
| `deplacer` | `ref`, puis `x`,`y` ou `dx`,`dy` (mm) | Les fils suivent, comme à la souris |
| `tourner`, `miroir` | `ref`, `angle` (90/180/270) | `transformSel` sur ce seul composant |
| `supprimer` | `ref` | Le composant, comme `delSel` (les fils restent, comme à la souris) |
| `supprimer_fil` | `de` « REF.broche », `a` « REF.broche » | Les segments du fil qui relie ces deux broches, jusqu'aux jonctions ; refus s'il n'y a pas de fil direct entre elles |
| `feuille` | `creer` (nom) ou `renommer` (n° + nom) | `addPage` / renommage |
| `renumeroter` | — | Tout le document, par préfixe, feuille par feuille, ordre de lecture (haut → bas, gauche → droite) ; les parties d'un même boîtier gardent un seul repère ; la carte montre la table ancien → nouveau et les repères repris par d'autres composants |

Les broches se nomment par numéro ou par nom de broche (`U1.OUT`), comme dans les lectures
de `registre-cao`.

### Critères d'acceptation

1. **Une carte = un lot** : un seul `push()`, Ctrl+Z défait tout le lot.
2. **Tout est simulé avant la carte** : le lot est joué sur une copie du document
   (`serialize` / `restore`), puis la page remet l'original. La carte liste les opérations
   et les connexions obtenues (« R1.2 — U1.3 sur le net N$4 »). Une opération impossible
   refuse tout le lot, avec sa raison.
3. **Pas de connexion par accident** : un fil en L qui passe sur la broche ou le bout de fil
   d'un autre net est refusé (la connectivité de la page relie tout point posé sur un
   segment). La raison propose une étiquette à la place.
4. **Chevauchement** : un composant posé ou déplacé dont la boîte recouvre un autre est
   refusé ; sans position, la place libre est cherchée vers la droite puis vers le bas,
   sur la grille.
5. **Catalogue** : une Part Name inconnue est refusée avec l'outil qui donne la liste
   (`lib_composants`) ; un symbole de catalogue inconnu de la page retombe sur le symbole
   générique, comme l'explorateur, et la carte le dit.
6. **Renuméroter** : les repères restent uniques dans tout le document ; la carte prévient
   que le PCB garde les anciens repères tant qu'il n'est pas resynchronisé.
7. Positions en mm (20 px = 1 mm), calées sur `S.grid` ; bornes ±1000 mm.

## Architecture

- Commande de page `cao_schema_dessiner` dans un nouveau fichier
  `editeur-schematique/js/2x-dessin-pilote.js` (le dessin n'existe qu'au schéma),
  enregistrée par `piloteCao("schema")`.
- Reprises de la page : `addComp`, `nextRef`, `routeL`, `touchWires`, `resolveSplits`,
  `transformSel`, `moveSelBy`, `deleteComps`, `addPage`, `allPins`, `brNomBroche`, la
  correspondance type ↔ catalogue de `schOuvrirExplorateurLib`.
- Nouveau : `renumeroter` et le test « fil sur un autre net », écrits pour ce module et
  testés à part.

## Commandes

```
Bundle :  python3 editeur-schematique/outils/build-monofichier.py
Bancs :   node editeur-schematique/test/harness.js ; node commun/test/banc-pilote-cao.js
MCP :     python3 lanceur/banc_mcp.py
```

## Stratégie de test

- Harnais schéma : dessiner un petit montage (régulateur, deux condensateurs, GND, VCC) en
  un lot ; vérifier les nets par `docNets()` ; un fil qui croiserait une broche est refusé
  sans rien changer ; Ctrl+Z remet le document d'avant ; renuméroter sur deux feuilles.
- Essai réel : Claude dessine un montage dans un schéma vide d'une copie de PROJETS,
  capture à l'appui.

## Limites

- **Toujours** : simuler le lot avant la carte ; refuser plutôt que relier par accident.
- **Demander d'abord** : fils par coordonnées ; symboles de la LIB (`schsym-1`) dessinés
  par le schéma.
- **Jamais** : enregistrer le projet ; toucher au PCB depuis ce module.

## Questions tranchées (2026-10-19)

1. `supprimer_fil` est ajoutée : sans elle, Claude ne pourrait défaire un fil d'un lot appliqué
   qu'en annulant tout le lot.
