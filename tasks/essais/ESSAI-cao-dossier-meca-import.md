# Essai : télécharger le dossier méca et importer son DXF dans une CAO méca (fentes comprises)

Qui : les deux (Claude in Chrome : préparer la carte, contrôler le dossier dans la page, télécharger le zip ; toi : ouvrir le zip et importer le DXF dans la CAO méca) · Où : PC Windows du boulot (WEB·SUITE et une CAO méca) · Durée : 20 min pour Claude in Chrome, 30 min pour toi · Source : `web_cao/tasks/todo-dossier-meca.md:128` (fentes : `web_cao/A-FAIRE.md:340`)

## Pourquoi
Le dossier méca (`<projet>-MECA.zip` : `-PLAN-MECA.pdf`, `-MECA.json`, `-MECA.dxf`, `commun/dossier-meca.js`, `mecaFichiers`) a été vu dans Chromium sans tête, le PDF relu par pdftoppm, le DXF relu par ezdxf (`todo-dossier-meca.md:128`). Personne n'a encore **téléchargé le zip pour de vrai** ni **ouvert le DXF dans une CAO méca**, qui est son seul usage.

Ce que le DXF doit contenir (`commun/dossier-meca.js`, `mecaDxf`) : R12 en mm, repère des Gerber, Y vers le haut ; calques `CONTOUR`, `DECOUPES`, `FIXATIONS`, `DEGAGEMENTS`, `REPERES_FIXATIONS`, `CORPS_DESSUS_H<h>` / `CORPS_DESSOUS_H<h>` (ex. `CORPS_DESSUS_H9_50`, `H_INCONNUE` sans hauteur ; non-monté en tirets), `OUVERTURES`, `REPERES_DESSUS` / `REPERES_DESSOUS`, `COTES` (cotes hors tout).

**Fentes, depuis le 21/10/2026** (commit `b11bee2`) : une fente de carte garde `l` et `rot` dans le JSON, le DXF la dessine en vraie forme — deux arcs et deux droites sur `FIXATIONS`, l'oblong élargi de son dégagement sur `DEGAGEMENTS` (`mecaOblong`, `mecaDxf`) — et le plan écrit « Ø d × l » (`mecaDiam`). Avant, elle sortait en trou rond. L'essai automatique (`editeur-pcb/test/harness.js:26792`) compte les entités ; il ne dit pas si une CAO les referme en profil.

Le dégagement vaut la pastille si elle est plus grande que le trou, sinon Ø + 2 mm : 5,2 mm pour un trou de 3,2 ; **3,2 mm pour la fente de 1,2** (son oblong de dégagement fait donc 5,2 × 3,2 mm).

Claude in Chrome fait tout ce qui se passe dans la page : la carte d'essai (un exemple, rien n'est enregistré), le contrôle du JSON, du DXF et du plan avant téléchargement, le téléchargement. Il ne peut pas ouvrir le zip téléchargé ni une CAO méca : c'est ta moitié.

## Ce que tu prépares
- WEB·SUITE lancé (WEB_CAO sur `http://127.0.0.1:8000`, avec la LIB pour les hauteurs). WEB_CAO à jour : la fente demande le commit `b11bee2` (le rapport le vérifie à l'étape 1).
- Une CAO méca qui importe le DXF (FreeCAD, Fusion 360, SolidWorks, Onshape…) : note laquelle et sa version.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-dossier-meca-import.md. But : préparer une carte d'essai (l'exemple 2, rien
n'est enregistré), contrôler son dossier mécanique dans la page, puis télécharger le zip.
Le bouton « ⬇ Dossier méca .zip » est PERMIS ici : c'est l'objet de l'essai. Tout autre
enregistrement est interdit (pas de Ctrl+S, pas de « ☁ Enregistrer »).

1. Onglet neuf : http://127.0.0.1:8000/editeur-pcb/editeur-pcb.html, puis :
   [typeof loadDoc, typeof exemple2, typeof mecaDossier, typeof mecaFichiersEditeur, typeof mecaOblong]
   → attendu : cinq fois "function". Si `mecaOblong` est "undefined", WEB_CAO n'a pas encore les
     fentes (commit b11bee2) : note-le, fais quand même l'essai, la fente sortira ronde.
2. Charge l'exemple 2 (66 × 42 mm, 4 couches) sans boîte de confirmation :
   push(); loadDoc(exemple2()); fit(); [S.fps.length, S.board.w, S.board.h, S.holes.length]
   → attendu : [11, 66, 42, 0]. Capture d'écran.
3. Quatre trous de fixation Ø 3,2 à 3,5 mm de chaque coin :
   push(); for(const [x,y] of [[3.5,3.5],[62.5,3.5],[3.5,38.5],[62.5,38.5]]) S.holes.push(mkHole(x,y,3.2)); touch(); draw(); S.holes.map(h=>[h.x,h.y,h.d])
   → attendu : quatre trous, visibles aux coins.
4. La fente, à la main (c'est aussi l'essai du panneau) : menu « Placer » → « Trou NPTH », puis
   UN clic sur la carte dans la bande libre en haut, vers le milieu. Le panneau « Propriétés »
   montre « Trou NPTH ». Saisis, en validant chaque champ par Entrée :
   « Diamètre de perçage (mm) » 1.2 ; « Longueur (mm) » 3.2 ; « Angle (°) » 0 ; « X (mm) » 33 ;
   « Y (mm) » 3.5. Puis touche Échap (retour en mode Sélection).
   Contrôle : S.holes.map(h=>[h.x,h.y,h.d,h.w||0,h.rot||0])
   → attendu : cinq trous, le dernier [33, 3.5, 1.2, 3.2, 0]. Un trou de trop : Ctrl+Z.
   Si la saisie échoue deux fois, note ce qui s'est passé (écart) et pose la fente ainsi :
   (()=>{push();const h=mkHole(33,3.5,1.2);h.w=3.2;S.holes.push(h);touch();draw();return S.holes.length;})()
   Capture d'écran (la fente doit être dessinée en oblong horizontal).
   Repère : X, Y du panneau sont ceux de l'éditeur (Y vers le BAS) ; le dossier est dans celui
   des Gerber (Y vers le HAUT) : y_dossier = 42 − Y_panneau.
5. Le JSON du dossier, sans télécharger :
   (()=>{const J=mecaDossier();return JSON.stringify({format:J.format,enc:J.carte.encombrement,ep:J.carte.epaisseur,hmax:J.hauteur_max,fix:J.fixations,ouv:J.composants.filter(c=>c.ouverture).map(c=>[c.ref,c.ouverture]),hauteurs:J.composants.map(c=>c.ref+"="+(c.hauteur==null?"?":c.hauteur)+" ("+c.source_hauteur+")"),avert:J.avertissements},null,1)})()
   → attendu : format "web_cao-meca-1" ; enc {x1:0, y1:0, x2:66, y2:42} ; ep 1.6 ;
     fix : H1 (3.5, 38.5), H2 (62.5, 38.5), H3 (3.5, 3.5), H4 (62.5, 3.5), Ø 3.2, d_degagement 5.2 ;
     H5 (33, 38.5), d 1.2, l 3.2, rot 0, d_degagement 3.2 ; tous metallise false.
     ouv : J1, J2, J3 en {type:"face", face:"dessus"} (leurs corps sont à plus de 2 mm du bord).
   → à relever : les hauteurs et leur source (lib / inconnue), hmax, les avertissements.
6. Le DXF, lu dans la page :
   (()=>{const t=new TextDecoder("latin1").decode(mecaFichiersEditeur()[2].data).split("\r\n").map(s=>s.trim());const E=[];let e=null;for(let i=0;i+1<t.length;i+=2){const c=t[i],v=t[i+1];if(c==="0"){e={type:v};E.push(e);}else if(e&&!(c in e))e[c]=v;}const n={};E.forEach(e=>{if(e["8"]){const k=e["8"]+":"+e.type;n[k]=(n[k]||0)+1;}});const arcs=E.filter(e=>e.type==="ARC"&&/FIXATIONS|DEGAGEMENTS/.test(e["8"])).map(e=>[e["8"],+e["10"],+e["20"],+e["40"],+e["50"],+e["51"]]);return JSON.stringify({entites:n,arcs},null,1)})()
   → attendu : CONTOUR:LINE 4 ; FIXATIONS:CIRCLE 4 et DEGAGEMENTS:CIRCLE 4 (les trous ronds) ;
     FIXATIONS:ARC 2 + FIXATIONS:LINE 2 et DEGAGEMENTS:ARC 2 + DEGAGEMENTS:LINE 2 (la fente) ;
     arcs FIXATIONS de rayon 0.6 centrés en (32, 38.5) et (34, 38.5), arcs DEGAGEMENTS de rayon
     1.6 aux mêmes centres ; REPERES_FIXATIONS:TEXT 5 ; un calque CORPS_DESSUS_H… par hauteur ;
     OUVERTURES ; COTES.
7. Menu « Fichier » → « Plans (Draftsman)… ». Coche la feuille « Plan mécanique ». Va sur ses
   onglets de feuille dans l'aperçu.
   → attendu : vue de dessus avec la fente en oblong ; tableau « Trous de fixation » : H1 à H4
     Ø « 3,20 », Ø dégagement « 5,20 » ; H5 Ø « 1,20 × 3,20 », Ø dégagement « 3,20 ».
     Captures de chaque feuille du plan mécanique.
   → à relever : le tableau « Dossier méca : hauteurs (mm) et ouvertures » de la colonne de
     gauche (repère, hauteur grisée et sa source, case Ouv.). N'y saisis rien.
8. Clique « ⬇ Dossier méca .zip » (dans la barre de la fenêtre des plans).
   → attendu : la barre d'état dit « Dossier mécanique ….zip : plan PDF, JSON et DXF 1:1 —
     5 fixation(s), 3 ouverture(s)… ». Recopie le message et le nom du fichier.
   Ferme la fenêtre des plans (✕). Ne ferme pas l'onglet (le navigateur demanderait « Quitter le
   site ? », qui te bloquerait).

Rapport, en plus du format commun :
- FENTES : présentes dans WEB_CAO (étape 1) oui / non.
- JSON : le résultat brut de l'étape 5 et la liste des écarts à l'attendu.
- DXF : le résultat brut de l'étape 6 et les écarts.
- PLAN : le tableau des trous tel qu'il s'affiche, et le tableau des hauteurs.
- ZIP : nom du fichier téléchargé et message de la barre d'état.
```

## C'est réussi si
- [ ] Dans la page : JSON, DXF et plan conformes à l'attendu (étapes 5 à 7), fente comprise.
- [ ] Zip téléchargé ; ouvert par toi, trois fichiers qui s'ouvrent.
- [ ] CAO utilisée, version ; import sans erreur ni avertissement (sinon, le texte).
- [ ] Contour 66 × 42 mm, trous ronds aux bonnes cotes et aux bons Ø.
- [ ] Fente : deux arcs r 0,6, 3,2 × 1,2 mm, profil fermé ; dégagement 5,2 × 3,2 ; « Ø d × l » au PDF.
- [ ] Un corps extrudé à sa hauteur ; liste des calques vus ; copies d'écran de la CAO (vue de dessus avec calques).

## Ce qui reste à toi
Claude in Chrome ne peut pas ouvrir un fichier téléchargé hors du navigateur ni piloter une CAO méca. Avec son rapport (valeurs attendues du JSON) :
1. Ouvre le zip (dossier Téléchargements) avec l'explorateur Windows.
   → attendu : trois fichiers, `-PLAN-MECA.pdf`, `-MECA.json`, `-MECA.dxf`, qui s'ouvrent.
2. PDF : tableau des trous de fixation.
   → attendu : ce que Claude in Chrome a lu dans l'aperçu (H5 « 1,20 × 3,20 », dégagement 3,20) ; la fente en oblong dans la vue de dessus.
3. CAO méca : importe `-MECA.dxf` (unités mm, échelle 1:1).
   → attendu : contour 66 × 42 mm (mesure-le), cotes du calque `COTES` lisibles ; les calques listés plus haut.
4. Mesure les fixations.
   → attendu : cercles Ø 3,2 sur `FIXATIONS`, Ø 5,2 sur `DEGAGEMENTS`, centrés aux X, Y du JSON ; même place relative qu'à l'écran (pas de miroir en Y).
5. La fente : sélectionne ses entités sur `FIXATIONS`.
   → attendu : deux arcs de rayon 0,6 mm et deux segments ; longueur hors tout 3,2 mm le long de X, largeur 1,2 mm ; sur `DEGAGEMENTS` un oblong de 5,2 × 3,2 mm.
   → à relever : la CAO en fait-elle un profil fermé (esquisse fermée, extrusion ou enlèvement de matière possible) ? oui / non.
6. Extrude un corps d'un calque `CORPS_DESSUS_H…` à la hauteur écrite dans son nom.
   → à relever : un corps qui s'extrude tel quel (profil fermé) ; un calque `H_INCONNUE` s'il y en a.
7. Ferme l'onglet du PCB sans enregistrer (« Quitter »). Garde le zip : il peut servir à `ESSAI-cao-dossier-meca-ia.md`.

## Après
Pour Claude Code, avec le rapport et ta réponse :
- Réussi :
  - [ ] cocher `web_cao/tasks/todo-dossier-meca.md:128` (le « [~] » passe à « [x] ») ;
  - [ ] noter dans `web_cao/editeur-pcb/README.md` la CAO qui a été essayée.
- Écart dans la page (JSON, DXF ou plan, étapes 5 à 7) :
  - [ ] corriger `web_cao/commun/dossier-meca.js` (`mecaDonnees`, `mecaFeuilles`, `mecaDxf`) et ajouter le cas au harnais PCB (`web_cao/editeur-pcb/test/harness.js`).
- Profil de fente ou de corps non fermé, arcs mal lus par la CAO :
  - [ ] corriger l'écriture dans `web_cao/commun/dossier-meca.js` (`mecaDxf`, `mecaOblong`) ou `dxfContour` / `dxfSegments` de `web_cao/editeur-pcb/js/33-draftsman-export.js`, et ajouter l'essai au harnais PCB.
- Téléchargement qui échoue dans le vrai navigateur :
  - [ ] regarder `web_cao/editeur-pcb/js/36-dossier-meca.js` (`mecaExporter`) et `web_cao/commun/zip.js`.
- `mecaOblong` absent (étape 1) : pousser et fusionner `b11bee2` dans `main` de WEB_CAO, puis redonner l'essai.
