# Spec : chemin-auto

Module 10 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`routage`](SPEC-mcp-routage.md).
Statut : **écrite et faite le 2026-10-21** ; choix revus avec toi le 2026-10-21 (via portée à 10 mm).

## Objectif

`cao_router` demande à Claude les points de passage de chaque route. Ce module les
**propose** : un chercheur de chemin (A*) dans la page PCB trouve un premier tracé par
liaison, que Claude relit puis donne à `cao_router`. Le routeur push-and-shove fait le
détail (contournement, poussée) et garde sa carte : rien ne change dans son
fonctionnement.

### Décisions (2026-10-21)

1. **Un outil de lecture**, `cao_chemin` : il ne pose rien et n'a pas de carte. Il rend,
   par liaison, « trouvée » ou la raison, et `routes`, la liste à donner telle quelle à
   `cao_router`.
2. **Grille par couche, A***, pas de 0,25 mm par défaut (0,05 à 2 mm). La grille couvre la
   boîte du contour ; au-delà de 2 millions de cellules (toutes couches), le pas grossit
   de lui-même et la réponse dit celui qui a servi.
3. **Huit directions** (angles de 45°, comme le routeur réglé à 45°). Un virage coûte
   (0,3 mm par 45°), un virage aigu (135°, demi-tour) est interdit. Pas de coin coupé en
   diagonale entre deux cellules bloquées.
4. **Obstacles** : le cuivre des autres nets (pastilles, pistes, vias), gonflé de
   l'isolation de la paire de classes (`clrK`, celle du DRC et du routeur) et de la
   demi-largeur de la piste ; la marge de bord (`S.rule.edge`) le long du contour et des
   découpes ; les trous NPTH. Le cuivre du même net gêne aussi (le routeur s'arrête dès
   qu'il le touche), sauf les pastilles des deux bouts. Les **zones de cuivre ne gênent
   pas** : elles se recoulent autour de la piste (gardé le 2026-10-21 : avec la via à 10 mm,
   aucune piste ne traverse le plan d'un autre net sur les exemples 1 et 2).
5. **Vias** : là où le centre d'une via (diamètre de la classe) est libre sur toutes les
   couches qu'elle traverse ; jamais dans la pastille CMS d'un bout. Une via coûte 10 mm
   (revu le 2026-10-21 : à 2 mm, l'exemple 1 dérouté prenait 6 vias et `cao_router` refusait
   une route sur 16 ; à 10 mm, 2 vias, 16 routes posées sur 16, 11 % de longueur en plus).
6. **Plusieurs liaisons** (20 au plus) se cherchent dans l'ordre : chaque chemin trouvé
   devient un obstacle pour les suivantes. Une liaison sans chemin n'arrête pas les autres.
7. **Bornes** : 8 s par appel pour toute la recherche (le Pi reste utilisable).

### Outil MCP

Dans `lanceur/mcp_routage.py` ; code de page dans `WEB_CAO/editeur-pcb/js/40-chemin-auto.js`,
commande déclarée avec celles du routage (`routagePiloteCommandes`).

| Argument | Contenu |
|---|---|
| `liaisons` | 1 à 20 `{depart, arrivee, largeur?}` ; `depart` / `arrivee` : `REF.pastille` ou `{x, y, couche?}` (un bout au moins est une pastille : elle donne le net) ; `largeur` absente : celle de la classe du net, couche par couche |
| `pas` | pas de la grille en mm, 0,25 par défaut |

Réponse : `{pas_mm, resultats: [{liaison, trouve, longueur_mm, vias, points} ou {liaison, trouve: false, raison}], routes: [{depart, couche, points, arrivee, largeur?}], conseil}`.
Dans `points`, une via se pose au point qui précède celui qui porte `couche`, comme
`cao_router` les lit.

### Critères d'acceptation

1. Rien n'est écrit : document, historique et drapeau identiques après l'appel.
2. Sur l'exemple 1 dérouté, quatre liaisons de pastille à pastille : quatre chemins,
   segments à 45°, points dans le contour ; donnés à `cao_router`, les quatre routes sont
   « posées » et autant de liaisons disparaissent.
3. Un mur de cuivre d'un autre net sur le dessus, d'un bord à l'autre : le chemin passe
   dessous par deux vias, de part et d'autre du mur, et `cao_router` le pose. Le mur aussi
   dessous : « aucun chemin », avec le pas, l'isolation et la largeur.
4. Pastille inconnue, deux nets différents : refus de cette liaison seulement, les autres
   continuent ; aucune liaison ou plus de 20, pas hors bornes : refus de l'appel.

## Commandes

```
Bundle :  python3 editeur-pcb/outils/build-monofichier.py
Bancs :   node editeur-pcb/test/harness.js ; node commun/test/banc-pilote-cao.js
MCP :     python3 lanceur/banc_mcp.py
```

## Limites

- **Toujours** : proposer seulement ; la pose passe par `cao_router` et sa carte.
- **Demander d'abord** : changer le routeur lui-même (`11-pns-*.js` à `13-pns-shove.js`).
  Les paires différentielles sont décidées le 22/10/2026 (ci-dessous).
- **Jamais** : écrire dans le document.
- Laissé : une pastille fine (pas de 0,5 mm et moins) peut demander un pas plus fin que
  0,25 mm (« pas » le règle) ; rooms et zones interdites ne sont pas lues, et n'ont pas à
  l'être : les unes sont un repère de placement, les autres mécaniques (hauteurs, pattes),
  aucune n'interdit le cuivre (revu le 21/10/2026) ; le chemin est le plus
  court sur la grille, pas le plus joli : le routeur le lisse.

## Ajout du 22/10/2026 : les paires différentielles

Une liaison `{paire, depart: [P, N], arrivee: [P, N]}` (deux « REF.pastille » de chaque côté,
celles des deux nets de la paire déclarée) :

- l'axe se cherche du milieu des deux pastilles de départ au milieu de celles d'arrivée, aussi
  large que les deux pistes et leur écart (`dpGeom` : 2 w + écart), les deux nets « à soi » ;
- elle change de couche par une via de paire (ajout du 22/10/2026, ci-dessous) ; avant, une
  seule couche, celle que les quatre pastilles avaient en commun ;
- les points qui passeraient à portée d'accroche d'un couple de pastilles de la paire (1,25 mm,
  `dpPrimPair`, le critère de `cao_paire`) sont retirés : sinon `cao_paire` terminerait la
  paire avant l'arrivée ;
- la réponse gagne `paires` : pour chacune, les arguments de `cao_paire` (une carte par paire),
  qui fait les éventails jusqu'aux pastilles. `largeur` est refusée pour une paire (sa règle).
- Banc : `harness.js`, « chemin-auto : une paire déclarée… » (ligne droite, puis un mur à
  contourner) ; `banc_mcp.py` (arguments).

## Ajout du 22/10/2026 : les vias de paire

- L'axe d'une paire change de couche comme une liaison simple, mais sa via est une via de paire :
  les deux vias côte à côte que `cao_paire` pose (`dpVia`, écartés de `dpViaSpread`, avancés par
  l'éventail de ce qu'il rattrape sur le pas des pistes). Sur la grille, elle tient dans un disque
  qui couvre les deux vias dans tous les sens (`shortcut:` dans le code : pas deux vias orientés).
- Pas de via de paire à portée d'accroche de ses pastilles (disque + 1,25 mm) : `dpVia` veut un
  segment avant elle pour savoir où ouvrir l'éventail, et `cao_paire` s'accrocherait aux
  pastilles. Départ et arrivée gardent chacun leurs couches : plus besoin d'une couche commune.
- Le point qui précède la via n'est jamais repris deux fois (un demi-tour sur place croiserait les
  deux pistes de `cao_paire`). Le filtre d'accroche juge chaque point sur sa couche ; le « couche »
  d'un point retiré passe au suivant, ou à l'arrivée : `cao_paire` pose la via là où il en est.
- `resultats` donne aussi `vias` (le nombre de vias de paire) pour une paire.
- Banc : `harness.js`, « chemin-auto : une paire passe sous un mur par deux vias de paire… »
  (deux 0603 sur 2 couches, mur sur le dessus : 4 vias posées par `cao_paire`, aucune erreur DRC
  nouvelle, la paire reliée).

## Questions ouvertes

Aucune. Les choix ci-dessus ont été revus avec toi le 2026-10-21 : via portée à 10 mm
(décision 5) ; virage 0,3 mm, pas 0,25 mm et zones de cuivre ignorées gardés. Reste
l'essai réel sur le Pi (`tasks/todo-chemin-auto.md`).
