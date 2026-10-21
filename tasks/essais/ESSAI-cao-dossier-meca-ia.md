# Essai : critère 6 du dossier méca, une IA lit le JSON et dessine le boîtier

Où : PC avec le zip de l'essai d'import et une IA de conversation (Claude, ou une autre) · Durée : 20 min · Source : `web_cao/tasks/todo-dossier-meca.md:181` (critère 6 : `web_cao/SPEC-dossier-meca.md:306-308`)

## Pourquoi
Le JSON `-MECA.json` (format `web_cao-meca-1`, `commun/dossier-meca.js:23`, décrit `SPEC-dossier-meca.md:110-140`) est fait pour qu'une IA conçoive le boîtier **sans rien d'autre**. Le critère 6 dit : à partir du seul JSON, elle en déduit les dimensions intérieures minimales du boîtier et la position des entretoises ; on contrôle sa réponse à la main. Les critères 1 à 5 sont tenus par les essais ; le 6 ne peut l'être que par un vrai essai. S'il échoue, le JSON n'est pas assez explicite (repère, unités, hauteurs inconnues) et une IA dessinera un boîtier faux sans le dire.

## Ce qu'il te faut
- Le `…-MECA.json` du zip de `ESSAI-cao-dossier-meca-import.md` (exemple 2 avec ses quatre trous Ø 3,2 et sa fente), ou refais les étapes 1 à 3 de cet essai.
- Une IA qui n'a pas vu cette conversation (nouvelle discussion), et le JSON ouvert à côté.

## Étapes
1. Ouvre le JSON et note les valeurs de contrôle.
   → à relever : `carte.encombrement` (x1, y1, x2, y2 ; attendu 66 × 42 mm) ; `carte.epaisseur` ; `hauteur_max.dessus`, `.dessous`, `.total` ; chaque `fixations[]` (id, x, y, d, `d_degagement`, `l` et `rot` pour la fente) ; chaque `ouverture` des composants (J1 : `face` ou `bord`) ; `avertissements`.
2. Nouvelle discussion avec l'IA. Colle le JSON entier, puis ce texte, sans rien ajouter :
   « Voici le dossier mécanique d'une carte électronique. Donne : (1) les dimensions intérieures minimales d'un boîtier qui la contient (X, Y, Z, en mm, en disant le jeu que tu prends) ; (2) la position de chaque entretoise (X, Y dans le repère du fichier) et son diamètre maximal ; (3) les ouvertures à prévoir dans le boîtier ; (4) ce que tu ne peux pas savoir avec ce fichier. »
3. Compare la réponse (1) à tes valeurs.
   → attendu : X ≥ 66, Y ≥ 42 (plus le jeu annoncé) ; Z ≥ `hauteur_max.total` (dessous + épaisseur + dessus), sans oublier la face dessous.
4. Compare (2).
   → attendu : une entretoise par fixation, aux X, Y du JSON (pas de miroir, pas de décalage d'origine) ; diamètre ≤ `d_degagement` (5,2 mm) ; la fente reconnue comme telle (vis ou pion, pas une entretoise ronde centrée au hasard).
5. Compare (3).
   → attendu : l'ouverture de J1 (USB) sur la bonne face ou le bon bord, à sa plage (`debut` / `fin`) et sa hauteur (`z_min` / `z_max`).
6. Compare (4).
   → attendu : l'IA cite les composants sans hauteur (`avertissements`, `source_hauteur: "inconnue"`) au lieu de leur inventer une hauteur.

## C'est réussi si / La réponse
- [ ] (1) X, Y, Z intérieurs justes (ou l'écart et sa cause).
- [ ] (2) Entretoises aux bonnes cotes, bon Ø max, fente comprise.
- [ ] (3) Ouverture USB juste.
- [ ] (4) Hauteurs inconnues signalées, rien d'inventé.
- [ ] L'IA utilisée (nom, version) et sa réponse entière collée.

## Après
- Les quatre justes : je coche le critère 6 et `web_cao/tasks/todo-dossier-meca.md:181`.
- Une erreur qui vient du JSON (repère mal compris, Z de la face dessous, fente prise pour un rond) : je précise le champ `repere` (`commun/dossier-meca.js:200-202`) ou les `avertissements` du JSON (et `SPEC-dossier-meca.md:110-140`), j'ajoute l'essai au harnais PCB, et je te redemande l'essai.
- Une erreur de l'IA seule (le JSON était clair) : je la note dans la case, et l'on décide ensemble si le critère est tenu.
