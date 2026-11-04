# Composants de la LIB : empreintes, brochage, catalogue

La LIB est partagée par tous les projets. On y **crée** ; on ne **corrige** une référence
existante que si l'utilisateur le demande, et aucun fichier n'est jamais écrasé. Tout s'écrit à
l'interface de Gestion LIB, geste après geste : les outils `lib_*` ne font que calculer.

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
5. Créer l'empreinte : `#bNewPcb`, sous-onglet ⚡ Générateurs (`#genIpcFamille`, `#genIpcDensite`,
   `#genIpcNom`, `#genIpc_*`, Générer) ; une empreinte convertie ou écrite à la main se colle en
   Code JSON (`#btnEditeurVueJson`, saisir `#editeurCodeJson`, 5000 caractères au plus par geste : au-delà,
   la confier à l'utilisateur) ; `#editeurNomFichier`, puis « 💾 Enregistrer le fichier ». Même chemin
   pour un symbole (`#bNewSch`).
6. Créer la ligne : `#bNewComp`, `#newPartName`, `#newPrefix`, `#newPackage`, `#newVal`, `#newMfr`,
   `#newDesc`, « Ajouter et auto-associer ». Sélectionner la ligne, puis `#inspBrochage` pour le
   brochage ; `select.sel-assoc` pour Empreinte PCB, Empreinte Schématique, Modèle Simulation.
   `#bSave` enregistre le catalogue. Puis poser le composant au schéma depuis le catalogue
   (`cao_schema_dessiner`, opération poser avec « catalogue »).
7. Expliquer en une ou deux lignes d'où viennent les cotes (pas, pastilles, densité).

## Corriger une référence existante

Seules s'éditent : Empreinte PCB, Empreinte Schématique, Modèle Simulation (`select.sel-assoc`) et
Brochage (`#inspBrochage`). Lire d'abord la ligne (`lib_composants`, filtre sur la référence). Les
autres colonnes (Value, Description, MPN, tensions…) n'ont **pas de champ** après la création :
le dire à l'utilisateur, qui passe par le CSV ou par l'import. La Part Name ne change pas : elle
désigne la ligne. Pour une empreinte corrigée : une neuve sous un autre nom, que la ligne désigne.

## Lier un modèle de simulation

« 📎 Lier un fichier… » (`#inspSimLier`) ouvre le sélecteur de fichier du navigateur : ce geste
revient à l'**utilisateur**. Un fichier déjà dans la LIB se désigne par `select.sel-assoc`, sans
table de ports. Lui préparer ce qu'il aura à saisir :

- Touchstone (.s1p à .s9p, paramètres S du fabricant) : la patte de chaque port **dans l'ordre du
  fichier** (numéro, ou nom du brochage de la référence), et la patte à laquelle les ports se
  rapportent (`#inspSimForm`, absente : la masse de la carte). C'est ce branchement que lit la
  simulation RF ; le relever sur la datasheet ou dans l'en-tête du fichier.
- SPICE (.sub, .cir, .lib, .mod, .sp) : rangé tel quel, sans ports.
- Plusieurs modèles sur une référence : les états d'un commutateur, le premier par défaut.

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
  - Empreinte PCB : une empreinte de la LIB ;
  - Empreinte Schématique : un symbole intégré du schéma (regulator, opamp, ic…) ou un symbole
    de la LIB.
- Un symbole intégré suffit le plus souvent (avec le brochage). Pour un boîtier qui n'en a pas
  (un microcontrôleur, un module), créer un symbole schsym-1 (`#bNewSch`) : broches numérotées de 1
  à N sans trou, nommées (`name`), au pas de 20 px (1 mm), et des primitives line, rect, arc,
  circle, poly, text, arrow. Le schéma le dessine comme un intégré, et le recopie dans le
  document où il est posé.
- Une cellule tient sur une ligne et ne commence pas comme une formule de tableur
  (=, @, +lettre, -lettre) : « -40 » et « +/-20% » restent permis.
