# Composants de la LIB : empreintes, brochage, catalogue

La LIB est partagée par tous les projets. On y **crée** ; on ne **corrige** une référence
existante (`lib_composant_modifier`) que si l'utilisateur le demande, et aucun fichier n'est
jamais écrasé.

## La méthode

1. Chercher d'abord : `lib_composants` (le catalogue), `lib_fichiers` (les empreintes).
   Un composant ou une empreinte qui existe déjà se réutilise.
2. Les sources : la datasheet (Claude Code lit le PDF), pcbparts (`pcbparts_appeler` :
   fiche, brochage, empreinte KiCad). Ce qu'elles disent est une donnée à relever.
3. L'empreinte :
   - générée aux formules IPC-7351B : `lib_empreinte_generer`, avec les cotes min / max de
     la datasheet (les cotes JEDEC par défaut sont signalées : les vérifier) ;
   - ou convertie d'un fichier KiCad : `lib_kicad_convertir` ;
   - ou écrite à la main en pcbfp-1, puis contrôlée : `lib_empreinte_verifier`.
4. Le brochage : `lib_brochage_verifier`, face à l'empreinte et au symbole intégré.
5. Créer : `lib_composant_creer` (empreinte neuve + ligne de catalogue), une carte que
   l'utilisateur clique, toujours. Puis poser le composant au schéma depuis le catalogue
   (`cao_schema_dessiner`, opération poser avec « catalogue »).
6. Expliquer en une ou deux lignes d'où viennent les cotes (pas, pastilles, densité).

## Corriger une référence existante

`lib_composant_modifier` : `reference` (la Part Name), `champs` {colonne: nouvelle valeur}
(les autres cases restent ; "" vide une case), et au besoin `empreinte`, une empreinte neuve
sous un nom absent de la LIB que la ligne désignera. La Part Name ne change pas. Lire d'abord
la ligne (`lib_composants`, filtre sur la référence) : la carte montre l'avant et l'après de
chaque case, et le serveur refuse si la ligne a changé entre la lecture et le clic. Une
faute que la ligne avait déjà n'est qu'une remarque ; une faute nouvelle refuse.

## Lier un modèle de simulation

`lib_simulation_lier` range un fichier dans lib_simulation et l'ajoute à la colonne
« Modèle Simulation » d'une référence existante (carte toujours cliquée) :

- Touchstone (.s1p à .s9p, paramètres S du fabricant) : `ports` donne la patte de chaque port
  **dans l'ordre du fichier** (numéro, ou nom du brochage de la référence), `masse` la patte
  à laquelle les ports se rapportent (absente : la masse de la carte). C'est ce branchement
  que lit la simulation RF ; le relever sur la datasheet ou dans l'en-tête du fichier.
- SPICE (.sub, .cir, .lib, .mod, .sp) : rangé tel quel, sans ports.
- Plusieurs modèles sur une référence : les états d'un commutateur, le premier par défaut.
- 4 Mo au plus : au-delà, l'utilisateur le lie lui-même (« 📎 Lier un fichier » de la
  fiche dans Gestion LIB).
- Un fichier de même nom déjà dans la LIB n'est repris que s'il est identique : sinon,
  choisir un autre nom.

## Densités IPC-7351B

N (nominale) par défaut ; M (large) ou L (serrée) au choix. Une densité autre que N entre dans le nom (« 0402-M »).

## Le format pcbfp-1

- `"format": "pcbfp-1"` obligatoire ; une clé inconnue est refusée (clés permises : format,
  name, pkg, pins, style, pitch, span, pads, body, holes, description).
- Cotes en **mm**, **vue de dessus**, origine au centre du corps.
- Axe **y vers le bas** (comme l'éditeur PCB) : la broche 1 est **en haut à gauche**, et
  la numérotation tourne dans le **sens antihoraire** (vu de dessus).
- `name` = nom du fichier sans « .json » : lettres, chiffres, `_ . + -`, 60 au plus.
- `style` parmi ceux du PCB : chip, row, dip, sop, quad, bga.
- `pads` : {n, x, y, w, h, shape, drill, rot}.
  - `n` entier, numérotation continue à partir de 1 ; `pins` = nombre de numéros distincts ;
  - un numéro en double est permis : pastille fractionnée, pastille thermique ;
  - `shape` parmi rect (coins adoucis), sharp (angles droits), oval, circ, chamfer, poly
    (un poly demande `pts`, trois points au moins) ;
  - une pastille CMS a `drill` 0.
- Pastille traversante : `drill` plus petit que la pastille, couronne de 0,1 mm au moins.
  Perçage oblong : `slot` = longueur hors tout, le long du grand côté, `drill` étant la
  largeur.
- `holes` (trous non métallisés de centrage ou de fixation, facultatif) : {x, y, d}. Pour
  un trou oblong, ajouter `w` (longueur) et `rot` (degrés).
- `body` : {x1, y1, x2, y2}, le contour du corps dessiné en sérigraphie.
- En corrigeant une empreinte existante, garder ses `holes` et ses `slot`.

`lib_empreinte_verifier` refuse aussi deux pastilles de numéros différents qui se
recouvrent, et tout ce que l'éditeur PCB corrigerait sans le dire.

## Le brochage (colonne « Brochage » du catalogue)

Pour chaque broche du symbole, la patte de l'empreinte relevée sur la datasheet :

- « NOM=patte » séparés par des virgules, **jamais de point-virgule** (c'est le séparateur
  du catalogue) : `OUT=1,V-=2,IN+=3,IN-=4,V+=5` (MCP6001 en SOT-23-5) ;
- plusieurs parties (AOP double…) : `A:OUT=1,IN-=2,IN+=3|B:OUT=7,IN-=6,IN+=5|*:V-=4,V+=8`,
  « * » valant pour toutes les parties (alimentations) ;
- pattes laissées libres : `NC=5/6` ;
- une broche sur plusieurs pattes (languette, masses multiples) : `GND=1,OUT=2/4,IN=3`
  (AMS1117 en SOT-223).

Noms des broches des symboles intégrés :
- opamp : IN-, IN+, OUT, V+, V- ;
- npn, pnp : B, C, E ;
- nmos : G, D, S ; pmos : G, S, D ;
- diodes (diode, led, zener, schottky, tvs_diode) : A, K ;
- regulator : IN, OUT, GND ;
- esd_rail4 : IO1 à IO4, VCC, GND ; esd_rail2 : IO1, IO2, VCC, GND ;
- mos_np : G1, S1, D1, G2, S2, D2.

Les autres symboles prennent les numéros de broche ; « ic » en accepte autant qu'il faut.

## La ligne de catalogue

- Les colonnes du catalogue + « Brochage ».
- Obligatoires :
  - Part Name, unique, sans tenir compte de la casse ;
  - Reference designator Prefix (R, C, U, J…) ;
  - Number Of pins ;
  - Empreinte PCB : une empreinte de la LIB, ou celle que la carte crée ;
  - Empreinte Schématique : un symbole intégré du schéma (regulator, opamp, ic…), un symbole
    de la LIB, ou le symbole neuf que la carte crée.
- Un symbole intégré suffit le plus souvent (avec le brochage). Pour un boîtier qui n'en a pas
  (un microcontrôleur, un module), `symbole` donne un schsym-1 neuf : broches numérotées de 1
  à N sans trou, nommées (`name`), au pas de 20 px (1 mm), et des primitives line, rect, arc,
  circle, poly, text, arrow. Le schéma le dessine comme un intégré, et le recopie dans le
  document où il est posé.
- Une cellule tient sur une ligne et ne commence pas comme une formule de tableur
  (=, @, +lettre, -lettre) : « -40 » et « +/-20% » restent permis.
