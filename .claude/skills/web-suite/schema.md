# Schéma : dessiner et vérifier

## Lire d'abord

`cao_schema_lire` (quoi=feuilles), puis (quoi=composants) et (quoi=nets) : repères, valeurs,
broches et nets existants. On ne pose rien sans savoir ce qui est déjà là.

## Dessiner (`cao_schema_dessiner`)

- Une carte = une liste ordonnée d'opérations (un lot, une étape Ctrl+Z). Un repère posé
  dans le lot sert aux opérations suivantes.
- Positions en mm, accrochées à la grille courante de l'éditeur (0,25 à 5 mm ; 1 mm par
  défaut, le pas des broches). Sans position, la page cherche une place libre à droite du
  dernier posé, par pas de 5 mm : donner les positions quand l'agencement compte.
- Poser depuis le catalogue (« catalogue » : Part Name de `lib_composants`) plutôt qu'un
  symbole nu : boîtier, empreinte et brochage suivent.
- Alimentations et masses : étiquettes de genre vcc et gnd sur la broche, jamais un long fil
  qui traverse la feuille.
- Fils courts entre broches voisines (relier) ; au-delà, une étiquette de net (local dans la
  feuille, global entre feuilles). Un fil qui toucherait d'autres broches est refusé : on
  étiquette alors.
- Une feuille par sous-système (alimentation, microcontrôleur, connecteurs, RF…) ; à
  l'intérieur, un bloc par fonction, les signaux de gauche à droite.
- Renuméroter les repères à la fin (renumeroter, sur tout le document), et **avant** de
  passer au PCB : le PCB garde les anciens repères jusqu'à sa resynchronisation.

## Règles de conception courantes

- **Découplage** : un condensateur de 100 nF par broche d'alimentation de CI, au plus près,
  et un condensateur de réserve (10 µF typiquement) par rail. La datasheet du CI prime.
- **Tirages** : toute sortie à drain ouvert (I2C, reset, interruptions) a sa résistance de
  tirage. I2C : la valeur dépend de la vitesse et de la capacité du bus (de l'ordre de
  4,7 kΩ à 100 kHz, 2,2 kΩ à 400 kHz).
- **Entrées** : pas d'entrée de CMOS en l'air ; une broche inutilisée suit la datasheet
  (à la masse, en tirage, ou laissée libre si elle le permet).
- **Inversion de polarité** : un MOSFET canal P en tête d'alimentation (drain côté entrée,
  source côté charge, grille vers la masse, avec une Zener grille-source si la tension
  d'entrée dépasse la Vgs admise) perd moins qu'une diode série.
- **Entrée analogique d'ADC** : un filtre RC, fc = 1 / (2π·R·C). R reste sous l'impédance
  de source maximale que la datasheet de l'ADC admet, et C (souvent quelques nF) fournit la
  charge de l'échantillonneur.
- Une valeur calculée se propose à l'application : `cao_valeur_composant` pour une valeur,
  `cao_composants_ecrire` pour plusieurs champs ou plusieurs composants.

## Vérifier une portion de schéma

Cohérence électrique (nets nommés, rails reliés), valeurs plausibles, découplages et
résistances de tirage manquants, entrées en l'air. Lire les nets, puis capturer la feuille
(`pilote_capture`) pour juger de la lisibilité.
