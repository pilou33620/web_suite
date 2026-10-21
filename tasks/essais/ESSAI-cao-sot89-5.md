# Essai : empreinte SOT-89-5 (Torex) sur une vraie carte

Qui : les deux (Claude in Chrome : l'empreinte face aux plans publiés ; toi : la carte en main) · Où : PC Windows du boulot, puis l'atelier · Durée : 30 min pour Claude in Chrome, 30 min pour toi · Source : `web_cao/A-FAIRE.md:288` (rappel l.47)

## Pourquoi
L'empreinte `SOT-89-5` (LIB : `PROJETS/LIB_CAO/lib_empreinte_pcb/SOT-89-5.json`, posée pour le régulateur XC6231A332PR-G, `A-FAIRE.md:287`) est **calculée depuis le contour du boîtier Torex** : Torex ne publie pas de pastilles recommandées dans la datasheet, et son site a refusé le téléchargement du plan de boîtier à l'agent qui l'a faite (pas à un vrai navigateur, peut-être). Elle fait aussi partie des 28 empreintes qui gardent l'origine de la fiche au lieu du centre (`FP_ORIGINE_FICHE`, `web_cao/editeur-pcb/test/harness.js:23531`). Une pastille trop courte ou mal placée donne des soudures sans ménisque, une languette (dissipation) mal reliée, ou un régulateur qui bascule au refusion.

Claude in Chrome relève l'empreinte servie par la LIB et la compare au plan Torex (s'il s'ouvre dans le navigateur) et à des empreintes publiques ; c'est un recoupement sur papier. Les mesures, la pose et les soudures restent à toi.

## Ce que tu prépares
- WEB·SUITE lancé sur le PC (WEB_CAO sur `http://127.0.0.1:8000`), Claude in Chrome ouvert.
- Pour ta moitié : une carte avec l'empreinte `SOT-89-5` (une nue et une assemblée si possible), un XC6231 (ou tout SOT-89-5 Torex) en vrac, un pied à coulisse ou un microscope à réticule, une loupe.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-sot89-5.md. But : comparer l'empreinte SOT-89-5 de la LIB aux plans publiés.
Tu ne modifies rien (LIB, projets) ; aucun bouton « 💾 Enregistrer », « Auto-associer ».

1. Onglet neuf, lecture seule :
   http://127.0.0.1:8000/api/lib/fichier?type=pcb&nom=SOT-89-5.json
   Puis exécute :
   (()=>{const f=JSON.parse(document.body.innerText).data;return JSON.stringify({body:f.body,pads:(f.pads||[]).map(q=>({n:q.n,x:q.x,y:q.y,w:q.w,h:q.h,forme:q.shape||"rect",rot:q.rot||0})),holes:f.holes||[]},null,1)})()
   → à relever : un tableau n° / largeur × longueur (mm) / centre X, Y (mm) ; le corps ;
     l'origine (0, 0) : où tombe-t-elle (centre du corps, pastille 1, autre) ? Y est VERS LE BAS,
     vue de dessus.
   → à calculer : entraxe des pattes d'une rangée, distance entre les deux rangées (bord
     extérieur à bord extérieur, et centre à centre), dimensions de la grande pastille de languette.
2. Ouvre http://127.0.0.1:8000/gestion-lib/gestion-lib.html, onglet « 📋 Catalogue Composants
   (CSV) », puis exécute :
   JSON.stringify(LIB_STATE.composants.filter(c=>/XC6231/i.test(JSON.stringify(c))),null,1)
   → à relever : Part Name, Package type, Empreinte PCB, Brochage, Maximun Height.
   Onglet « 📐 Empreintes PCB », filtre « SOT-89-5 » : capture d'écran de la carte de l'empreinte.
   (Le point blanc de l'aperçu est toujours au coin haut-gauche du corps : ne t'en sers pas.)
3. Sur le web, site Torex (torexsemi.com) : datasheet du XC6231 et, s'il existe, le document de
   boîtier « SOT-89-5 » (package information, « reference pattern layout », « reference metal mask »).
   → à relever : URL, page, figure ; cotes du boîtier (corps, longueur et largeur des pattes, pas,
     languette) ; pastilles recommandées si Torex en donne ; affectation des broches du XC6231
     (VIN, VSS, CE, VOUT, NC / languette).
   Si le site refuse un téléchargement ou n'affiche pas le PDF, note le message exact et passe.
4. Empreinte publique de référence : KiCad, fichier
   https://gitlab.com/kicad/libraries/kicad-footprints/-/raw/master/Package_TO_SOT_SMD.pretty/SOT-89-5.kicad_mod
   (s'il manque, cherche « SOT-89-5 » dans le dossier Package_TO_SOT_SMD.pretty du même dépôt).
   → à relever : chaque « (pad … smd … (at x y) (size w h)) » ; KiCad a l'origine au centre et
     Y vers le bas, comme la LIB, mais l'origine de la LIB peut différer : compare des entraxes
     et des tailles, pas des coordonnées brutes.
   Si tu trouves le plan de pastilles d'un autre fondeur pour le même boîtier SOT-89-5, relève-le
   aussi (source secondaire pour Torex).
5. Compare pastille par pastille (LIB contre Torex, puis LIB contre KiCad) :
   → écarts en mm sur la longueur, la largeur, l'entraxe, la distance entre rangées, la languette ;
   → la pastille dépasse-t-elle la patte en bout (pointe, de quoi faire le ménisque) ? de combien ?
   → la numérotation 1 à 5 de la LIB suit-elle celle de Torex (même patte n° 1, même sens) ?

Rapport, en plus du format commun :
- TABLEAU LIB (étape 1) et calculs.
- SOURCES : URL, page, figure, PRIMAIRE (Torex) / SECONDAIRE (KiCad, autre fondeur), et les cotes lues.
- ÉCARTS : un tableau pastille / LIB / Torex / KiCad / écart (mm).
- VERDICT DOCUMENTAIRE : cohérente / à reprendre (quoi) / plan Torex inaccessible (message).
```

## C'est réussi si
- [ ] Le tableau des pastilles de la LIB, et l'origine.
- [ ] Au moins une source comparée (Torex primaire si elle s'ouvre, sinon KiCad et dit comme tel) ; écarts chiffrés.
- [ ] Ta vérification sur la carte (ci-dessous) : cotes mesurées, patte posée, languette, soudures.
- [ ] Source de la vérification annoncée : carte réelle, ou seulement recoupement documentaire.

## Ce qui reste à toi
Avec le tableau du rapport (étape 1) :
1. Sur la carte nue, mesure deux pastilles de pattes (une de chaque rangée) et la grande pastille de languette.
   → attendu : les cotes du tableau, à ±0,05 mm (sinon c'est la fabrication, pas l'empreinte).
2. Pose un SOT-89-5 à sec sur l'empreinte, bien centré, et regarde à la loupe.
   → à relever : de combien chaque patte dépasse ou reste dans sa pastille ; si la languette tombe entière sur sa pastille ; si une patte touche la pastille voisine.
3. Sur la carte assemblée, regarde les soudures.
   → à relever : ménisque en bout de chaque patte (oui / non), languette soudée sur toute sa longueur, régulateur droit ou tourné ; une photo.
4. Sans aucune carte : dis-le ; le rapport de Claude in Chrome n'est alors qu'un recoupement.

## Après
Pour Claude Code, avec le rapport et ta réponse :
- Cohérente et carte bonne :
  - [ ] cocher la moitié SOT-89-5 de `web_cao/A-FAIRE.md:288` et de la mention `web_cao/A-FAIRE.md:47` ;
  - [ ] si le plan Torex s'est ouvert : noter son URL dans la ligne l.287 (la phrase « son site refuse le téléchargement » devient fausse).
- À reprendre :
  - [ ] corriger les pastilles de `PROJETS/LIB_CAO/lib_empreinte_pcb/SOT-89-5.json` (longueur de pointe, entraxe, languette) d'après les mesures et le plan, en gardant l'origine (essai `FP_ORIGINE_FICHE`, `web_cao/editeur-pcb/test/harness.js:23531`) ;
  - [ ] lister les cartes qui la portent.
- La moitié LED RVB de la même case est dans `ESSAI-cao-led-rvb.md`.
