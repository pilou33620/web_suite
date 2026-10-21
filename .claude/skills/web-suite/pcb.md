# PCB : règles de conception et méthode

Toujours lire avant de proposer : `cao_pcb_lire` (quoi=empilage) pour les couches et les
épaisseurs, (quoi=classes) pour les règles, (quoi=net) pour un net, (quoi=drc) pour les
erreurs en cours. Ne jamais supposer une épaisseur de cuivre ou un empilage : ils se lisent.

## Largeur de piste selon le courant (IPC-2152)

- IPC-2152 a remplacé la vieille charte d'IPC-2221 : l'échauffement d'une piste dépend du
  courant, de l'épaisseur de cuivre (35 µm, 70 µm…), de la couche (interne ou externe) et
  surtout de l'**étalement** de la chaleur dans le stratifié et les plans voisins.
- La largeur se donne pour un échauffement admis, en °C (un écart : °C et K, c'est pareil),
  à comparer à une température absolue : Tg du stratifié, limite d'un boîtier.
- Ne pas recopier une valeur de charte comme un fait. Calculer avec la page :
  `cao_calcul` (analyse=dc) résout la chute de tension et l'échauffement par étalement.
- Appliquer : `cao_pistes_ecrire` (largeur d'un net), ou mieux une classe de nets
  (`cao_classes_ecrire`), pour que les pistes futures la suivent. Changer une classe ne
  touche pas les pistes déjà posées : les réaligner ensuite (`cao_pistes_ecrire`, realigner).
- Un calcul de largeur se termine par une proposition d'application (une carte), jamais par
  un chiffre laissé à reporter à la main.

## Classes et isolations

Le schéma propose des classes d'usine (créées au besoin, jamais imposées) :

| Classe | Largeur | Isolation | Via / perçage |
|---|---|---|---|
| Alimentation, Masse | 0,6 mm | 0,25 mm | 0,9 / 0,45 mm |
| Horloge | 0,25 mm | 0,5 mm = 2 largeurs, soit 3W d'axe à axe | 0,8 / 0,4 mm |
| Rapide | 0,25 mm | 0,25 mm | 0,8 / 0,4 mm |
| RF | 50 Ω sur la couche du dessus, d'après l'empilage | 0,5 mm | 0,8 / 0,4 mm |
| Analogique | 0,3 mm | 0,35 mm | 0,8 / 0,4 mm |
| Antenne | 0,3 mm | 0,5 mm | 0,8 / 0,4 mm |

Les classes d'interface (USB FS et HS, ETH 100 et 1000, LVDS, CAN, RS-485) prennent les
règles de leur famille (Rapide ou Lent) et portent l'impédance différentielle de leur
norme : 90 Ω (USB), 100 Ω (Ethernet, LVDS), 120 Ω (CAN, RS-485).

Ce sont des points de départ : la classe réelle se lit (`cao_pcb_lire` (quoi=classes)) et se
règle par `cao_classes_ecrire` (créer, modifier, rattacher des nets).

## Paires différentielles

- USB 2.0 : 90 Ω différentiel. D'autres liens ont la leur (LVDS, Ethernet 100 Ω…) : la
  datasheet ou la norme du lien la donne.
- La règle de paire d'usine ne fixe pas d'impédance : largeur 0,15 / 0,2 / 0,4 mm
  (min / préférée / max), écart 0,13 / 0,15 / 0,4 mm, 12,7 mm au plus de longueur non
  couplée. Pour une impédance visée : `cao_contraintes_ecrire` (action paire, impedance :
  profil D90 pour l'USB, D100, D85 ou D120).
- Vérifier l'impédance obtenue avec `cao_calcul` (analyse=diff), router la paire avec
  `cao_paire`.

## Chemin de retour

- Tout signal rapide court au-dessus d'un plan de référence continu. Une fente ou une
  coupure du plan sous la piste allonge la boucle de retour : rayonnement et diaphonie.
- Un changement de couche d'un signal rapide change aussi de plan de référence : poser une
  via de masse près de la via de signal.
- `cao_calcul` (analyse=retour) montre le chemin de retour d'un net.

## Diaphonie et bus

- Règle des 3W : écart d'axe à axe d'au moins trois largeurs (isolation = deux largeurs)
  entre une horloge ou un signal rapide et ses voisins.
- Le décalage de longueur (skew) d'un bus se juge sur le budget de temps de l'interface
  (tsu / th de la datasheet), pas sur un chiffre fixe : l'analyse « Bus synchrone » de la page
  calcule temps de vol, skews et marge setup / hold (voir [simulation.md](simulation.md)).
- `cao_calcul` (analyse=crosstalk) chiffre la diaphonie, (analyse=oeil) un lien rapide.

## Vias thermiques

Sous la pastille thermique d'un QFN ou d'un composant de puissance : une grille de vias vers
le plan de masse interne. La grille recommandée (diamètre, pas, vias bouchées ou non) est
dans la datasheet du boîtier : la lire plutôt que l'inventer. `cao_calcul` (analyse=dc)
chiffre l'échauffement obtenu.

## La boucle de vérification

1. Proposer le lot (une carte, une étape Ctrl+Z).
2. Après le clic : `cao_score` (quoi=routage), puis `cao_drc` pour le détail des erreurs
   (comme dans [routage.md](routage.md)).
3. `cao_calcul` (analyse=verif) pour l'audit de la carte avant fabrication.
4. Corriger ce qui reste, une cause à la fois.

## Contour, zones, sérigraphie, pastilles

Ne jamais modifier l'état de la page (`S`) depuis la console : il n'entre pas dans
l'historique, et Ctrl+Z se comporte mal ensuite. Chaque geste a sa commande, une carte et
une étape d'historique :

- Contour : `cao_contour_ecrire` (rectangle, disque — écrit en arcs au Gerber —, polygone).
  Relire : `cao_pcb_lire` (quoi=carte). Fenêtres fraisées dans la carte : `cao_decoupes_ecrire`.
- Zones de cuivre : `cao_zones_ecrire` (creer, modifier, supprimer ; forme « carte » pour un
  plan pleine carte). Relire : `cao_pcb_lire` (quoi=zones), qui donne les id.
- Plot thermique d'une LED de puissance, d'un QFN, d'un régulateur : liaison **directe** à
  la zone, pas des bras thermiques. Soit la zone entière (`liaison` : cms-directe = directe
  pour les CMS, bras pour les traversantes), soit la pastille seule
  (`cao_pastille_regler`, liaison directe). Les bras restent l'usage pour une pastille
  qu'on brase à la main ou une traversante.
- Sérigraphie : `cao_serigraphie_ecrire` (texte, trait, rectangle ; repere pour éloigner un
  repère d'une pastille ; auto pour couper la sérigraphie d'office). Les valeurs ne sont pas
  imprimées. Relire : `cao_pcb_lire` (quoi=serigraphie).
- Une via seule (de couture, de plot thermique) : `cao_router` avec `vias` et sans `routes`.
- Intégration mécanique (carte dans un boîtier, un bouchon, une face contre une LED) :
  `cao_interdits_ecrire` — zones interdites par face (aucune patte traversante qui en
  ressorte, composants sous une hauteur permise) et hauteur maximale par face. Une patte
  traversante ressort sur la face opposée à son composant. Relire : `cao_pcb_lire`
  (quoi=interdits) ; le DRC dit ce qui déborde.
- Chaleur : donner à chaque composant qui dissipe sa puissance (`cao_composants_ecrire`, champ
  puissance en W), puis `cao_thermique`. C'est un ordre de grandeur (±30 %) : pour faire
  baisser ΔT, relier plus de cuivre (liaison directe du plot, zone, vias vers un plan).
