# Placement et routage du PCB

Claude planifie ; le routeur push-and-shove de la page trace le détail entre les points de
passage donnés. Pour les règles électriques (largeurs, paires, retour), voir [pcb.md](pcb.md).

## Préparer

- `cao_pcb_lire` (quoi=empilage) et (quoi=classes) : couches, largeurs, paires déclarées.
- `cao_pcb_lire` (quoi=liaisons) : les liaisons à router, de pastille à pastille, avec leurs
  coordonnées, couche, net et classe.
- `cao_pcb_lire` (quoi=zone) : ce qu'il y a dans une fenêtre (pistes, vias, pastilles,
  zones, contour) avant d'y faire passer une route.
- Toutes les positions sont dans le repère des lectures (mm), pas l'origine de
  l'utilisateur.

## Placer d'abord

1. `cao_placement_auto` (methode=forces) dégrossit tout, découplages contre leur broche.
2. (methode=satellites) range découplages et composants série contre leur circuit ;
   (methode=blocs) agence un motif publié par le schéma.
3. `cao_score` (quoi=placement) : longueur des liaisons, congestion, découplage, pires
   contributeurs. Corriger les pires d'abord.

## Ordre de routage

1. Alimentations et masses : larges pistes ou zones, selon leur classe.
2. Paires différentielles : `cao_chemin` avec `{paire, depart: [P, N], arrivee: [P, N]}`
   propose l'axe (vias de paire comprises), puis `cao_paire` avec l'élément de « paires » rendu ;
   ou `cao_paire` d'un bout libre de la paire (ou du milieu de ses deux pastilles) à la main.
3. Signaux critiques : horloges, RF, liens rapides, au plus court, sur un plan de référence
   continu.
4. Le reste.

## Router par points de passage (`cao_router`)

- Une route : départ « REF.pastille » (U1.3) ou {x, y}, points aux coudes (angles à 45°),
  arrivée. Un point portant « couche » change de couche par une via.
- Mode pousser (défaut) ou contourner pour ne pas déranger un routage fini.
- Un lot de routes = une carte (20 routes au plus). Tout ou rien par route : une route
  refusée ne laisse rien et la carte dit pourquoi. Ajouter des points, changer de couche ou
  de mode, puis reproposer cette route seule.
- Avant de cliquer, l'aperçu en pointillés se voit par `pilote_capture` ; le cuivre poussé
  apparaît en rouge.

## Points proposés (`cao_chemin`)

- Pour des liaisons sans évidence, `cao_chemin` propose les points : un chercheur A* sur
  une grille (0,25 mm par défaut), angles de 45°, vias là où toutes les couches sont libres,
  le cuivre des autres nets évité à l'isolation de leur classe. Rien n'est écrit.
- Donner les liaisons d'une même zone dans l'ordre voulu (les plus critiques d'abord) :
  chaque chemin trouvé gêne les suivants. 20 au plus par appel.
- Relire « routes » (une via de trop, un détour) avant de le donner tel quel à
  `cao_router`. Une liaison « sans chemin » : pas plus fin (pastilles à pas fin), autre
  ordre, ou points à la main.
- Les zones de cuivre ne gênent pas le chercheur. Une paire : « paires » va à `cao_paire`, une
  carte par paire ; elle change de couche par une via de paire (deux vias côte à côte), posée
  au point qui précède celui qui porte « couche ».

## La boucle

1. Après le clic : `cao_score` (quoi=routage) donne les liaisons restantes, le DRC, les vias,
   la longueur et un total indicatif à faire baisser.
2. `cao_drc` pour le détail des erreurs, `cao_pcb_lire` (quoi=drc) pour les relire.
3. Corriger une cause à la fois, puis recommencer.
