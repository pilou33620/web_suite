# Essai : un IPC-2581 d'eCADSTAR porte-t-il `<Package><Outline>` et `height` ?

Où : PC avec eCADSTAR et une carte réelle, puis WEB_CAO (visionneuse) · Durée : 20 min · Source : `web_cao/A-FAIRE.md:338` (rappel l.47 ; risque noté `tasks/plan-dossier-meca.md:81`)

## Pourquoi
Le dossier mécanique de la visionneuse tire le corps et la hauteur de chaque composant du fichier IPC-2581 :
- `python/ipc2581_parser.py:2167-2172` lit l'attribut `height` du `<Package>` (dans les unités du fichier) et le premier `<Polygon>` **directement sous** `<Package><Outline>` ;
- `visionneuse-ipc2581/js/08-dossier-meca.js:30-38` : sans ce polygone (au moins 3 points), le corps devient la boîte des pastilles, marqué `corps_estime` ; l.58 : sans `height`, la hauteur vient de la LIB (« Maximun Height ») ou de ta saisie ;
- `commun/dossier-meca.js:192` écrit alors l'avertissement « Encombrement estimé d'après les pastilles (pas de contour de boîtier dans le fichier) ».

On ne sait pas ce qu'eCADSTAR écrit. S'il met le contour ailleurs (`<AssemblyDrawing>`, `<SilkScreen>`, un `<Polyline>`) ou la hauteur dans un attribut non standard, tous les corps sont estimés trop petits et le boîtier dessiné d'après le dossier peut toucher des composants.

## Ce qu'il te faut
- eCADSTAR (PCB Editor) et une carte réelle avec des composants de hauteurs variées (connecteur, électrolytique, CI).
- Un éditeur de texte qui ouvre un gros XML (VS Code, Notepad++) ou un terminal.
- WEB_CAO lancé (visionneuse IPC-2581).

## Étapes
1. Dans eCADSTAR, exporte la carte en IPC-2581 : la révision la plus haute proposée (C, sinon B), avec tout le contenu (composants, assemblage, packages). Si l'export propose une option de hauteur ou de contour d'assemblage / de placement, coche-la et note son nom.
   → à relever : version d'eCADSTAR, révision IPC-2581, options cochées (copie d'écran de la fenêtre d'export).
2. Ouvre le XML. Vérifie l'outil : cherche `SoftwarePackage`.
   → attendu : `name` ou `vendor` qui dit eCADSTAR / Zuken.
3. Cherche `<CadHeader` et note `units`.
   → à relever : `MILLIMETER`, `INCH` ou `MICRON`.
4. Cherche `<Package ` et regarde le premier bloc entier, puis un connecteur.
   → à relever : présence de l'attribut `height="…"` (valeur, et si elle est plausible pour ce boîtier dans les unités de l'étape 3) ; liste des enfants du `<Package>` (`Outline`, `PinOne`, `AssemblyDrawing`, `SilkScreen`, `Pin`…).
5. Si `<Outline>` existe : regarde son premier enfant.
   → attendu pour que le code le lise : `<Polygon>` avec `<PolyBegin>` et des `<PolyStepSegment>` / `<PolyStepCurve>`. Note s'il contient autre chose.
6. Si `height` manque : cherche sans tenir compte de la casse `height`, `HEIGHT`, `Height` dans tout le fichier (ex. `<NonstandardAttribute name="HEIGHT" …>` sur `<Component>` ou `<Package>`).
   → à relever : où et sous quel nom la hauteur apparaît, ou « nulle part ».
7. Ouvre le fichier dans la visionneuse, bouton « Dossier méca ».
   → à relever : l'avertissement « Encombrement estimé d'après les pastilles … N composant(s) » (N), et dans le tableau combien de hauteurs portent « (fichier) ».

## C'est réussi si / La réponse
- [ ] `<Package height>` : présent / absent / ailleurs (nom exact et élément).
- [ ] `<Package><Outline><Polygon>` : présent / absent / contour ailleurs (élément exact).
- [ ] Unités du fichier.
- [ ] Visionneuse : N corps estimés sur M composants, K hauteurs « (fichier) ».
- [ ] Un extrait XML d'un `<Package>` complet (un connecteur), collé dans la réponse ou en fichier.

## Après
- Les deux présents et lus (étape 7 sans corps estimé) : je coche `web_cao/A-FAIRE.md:338` et la mention l.47.
- Contour ou hauteur ailleurs : j'ajoute ce chemin au parseur (`python/ipc2581_parser.py:2157-2172`, puis `ipc2581_json.py`, recopie dans WEB_ANTENNA) avec ton extrait comme essai dans `visionneuse-ipc2581/test/banc-essai.py`, puis je coche.
- Absents : le repli actuel est le bon ; je note dans `A-FAIRE.md:338` qu'eCADSTAR ne les écrit pas (avec la version et les options essayées) et je coche.
