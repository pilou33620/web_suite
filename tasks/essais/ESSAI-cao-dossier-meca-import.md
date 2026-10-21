# Essai : télécharger le dossier méca et importer son DXF dans une CAO méca (fentes comprises)

Où : PC avec WEB_CAO dans un vrai navigateur et une CAO méca (FreeCAD, Fusion 360, SolidWorks, Onshape…) · Durée : 30 min · Source : `web_cao/tasks/todo-dossier-meca.md:128` (fentes : `web_cao/A-FAIRE.md:340`)

## Pourquoi
Le dossier méca (`<projet>-MECA.zip` : `-PLAN-MECA.pdf`, `-MECA.json`, `-MECA.dxf`, `commun/dossier-meca.js:444-450`) a été vu dans Chromium sans tête, le PDF relu par pdftoppm, le DXF relu par ezdxf (`todo-dossier-meca.md:128`). Personne n'a encore **téléchargé le zip pour de vrai** ni **ouvert le DXF dans une CAO méca**, qui est son seul usage.

Ce que le DXF doit contenir (`commun/dossier-meca.js:397-426`) : R12 en mm, repère des Gerber, Y vers le haut ; calques `CONTOUR`, `DECOUPES`, `FIXATIONS`, `DEGAGEMENTS`, `REPERES_FIXATIONS`, `CORPS_DESSUS_H<h>` / `CORPS_DESSOUS_H<h>` (ex. `CORPS_DESSUS_H9_50`, `H_INCONNUE` sans hauteur ; non-monté en tirets), `OUVERTURES`, `REPERES_DESSUS` / `REPERES_DESSOUS`, `COTES` (cotes hors tout).

**Fentes, depuis le 21/10/2026** (commit `b11bee2`) : une fente de carte garde `l` et `rot` dans le JSON (l.143), le DXF la dessine en vraie forme — deux arcs et deux droites sur `FIXATIONS`, l'oblong élargi de son dégagement sur `DEGAGEMENTS` (`mecaOblong`, l.33-41 ; `mecaDxf`, l.406-409) — et le plan écrit « Ø d × l » (`mecaDiam`, l.43). Avant, elle sortait en trou rond. L'essai automatique (`editeur-pcb/test/harness.js:26792`) compte les entités ; il ne dit pas si une CAO les referme en profil.

Le dégagement vaut la pastille si elle est plus grande que le trou, sinon Ø + 2 mm (l.142).

## Ce qu'il te faut
- WEB_CAO lancé (avec la LIB, pour les hauteurs), un navigateur normal (pas de mode privé qui bloque les téléchargements).
- Une CAO méca qui importe le DXF (note laquelle et sa version).

## Étapes
1. Éditeur PCB : « Exemples… » → exemple 2 « Interface IoT & Bus Haute Vitesse » (66 × 42 mm, 4 couches) → « Ouvrir cet exemple ».
2. L'exemple n'a pas de trou : outil « Trou NPTH », pose quatre trous Ø 3,2 mm à environ 3,5 mm de chaque coin, puis un cinquième trou Ø 1,2 mm, hors de tout composant (sinon il devient le trou de ce composant), et, dans son panneau, « Longueur (mm) » 3,2, « Angle (°) » 0 (une fente horizontale).
   → à relever : X, Y de chaque trou dans le panneau.
3. Plans : coche la feuille « Plan mécanique », puis « ⬇ Dossier méca .zip ».
   → attendu : un fichier `…-MECA.zip` téléchargé ; la barre d'état annonce « Dossier mécanique … : plan PDF, JSON et DXF 1:1 — 5 fixation(s), … ».
4. Ouvre le zip avec l'explorateur de ton système.
   → attendu : trois fichiers, `-PLAN-MECA.pdf`, `-MECA.json`, `-MECA.dxf`, qui s'ouvrent.
5. PDF : tableau des trous de fixation.
   → attendu : H1 à H4 en Ø 3,2 ; H5 en « 1,2 × 3,2 » (ou « 1,20 × 3,20 ») ; dégagement 5,2 pour tous ; la fente dessinée en oblong dans la vue de dessus.
6. CAO méca : importe `-MECA.dxf` (unités mm, échelle 1:1).
   → attendu : contour 66 × 42 mm (mesure-le), cotes du calque `COTES` lisibles ; les calques de la liste ci-dessus.
7. Mesure les fixations.
   → attendu : cercles Ø 3,2 sur `FIXATIONS`, Ø 5,2 sur `DEGAGEMENTS`, centrés aux X, Y du JSON (`fixations[].x / y`) ; même place relative qu'à l'écran (pas de miroir en Y).
8. La fente : sélectionne ses entités sur `FIXATIONS`.
   → attendu : deux arcs de rayon 0,6 mm et deux segments ; longueur hors tout 3,2 mm le long de X, largeur 1,2 mm ; sur `DEGAGEMENTS` un oblong de 5,2 × 3,2 mm (longueur × largeur).
   → à relever : la CAO en fait-elle un profil fermé (esquisse fermée, extrusion ou enlèvement de matière possible) ? oui / non.
9. Extrude un corps d'un calque `CORPS_DESSUS_H…` à la hauteur écrite dans son nom.
   → à relever : un corps qui s'extrude tel quel (profil fermé) ; un calque `H_INCONNUE` s'il y en a.

## C'est réussi si / La réponse
- [ ] Zip téléchargé et ouvert, trois fichiers : oui / non.
- [ ] CAO utilisée, version ; import sans erreur ni avertissement (sinon, le texte).
- [ ] Contour 66 × 42 mm, trous ronds aux bonnes cotes et aux bons Ø.
- [ ] Fente : deux arcs r 0,6, 3,2 × 1,2 mm, profil fermé ; dégagement 5,2 × 3,2 ; « Ø d × l » au PDF.
- [ ] Un corps extrudé à sa hauteur ; liste des calques vus.
- [ ] Copies d'écran de la CAO (vue de dessus avec calques).

## Après
- Réussi : je coche `web_cao/tasks/todo-dossier-meca.md:128` (le « [~] » passe à « [x] ») et je note dans `editeur-pcb/README.md` la CAO qui a été essayée.
- Profil de fente ou de corps non fermé, arcs mal lus : je corrige l'écriture dans `commun/dossier-meca.js:401-426` (ou `dxfContour` de `editeur-pcb/js/33-draftsman-export.js`) et j'ajoute l'essai au harnais PCB.
- Téléchargement qui échoue dans le vrai navigateur : je regarde `editeur-pcb/js/36-dossier-meca.js:64-70` et `commun/zip.js`.
- Garde ce zip : il sert à `ESSAI-cao-dossier-meca-ia.md`.
