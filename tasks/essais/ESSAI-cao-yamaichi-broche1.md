# Essai : broche 1 du support Yamaichi IC51-0484-806

Qui : les deux (Claude in Chrome : la fiche et la LIB ; toi : le support en main) · Où : PC Windows du boulot, puis l'atelier · Durée : 20 min pour Claude in Chrome, 15 min pour toi · Source : `web_cao/A-FAIRE.md:198` (rappel l.47)

## Pourquoi
L'empreinte `Yamaichi-IC51-0484-806` (LIB : `PROJETS/LIB_CAO/lib_empreinte_pcb/Yamaichi-IC51-0484-806.json`) a été reprise le 16/10/2026 : trous NPTH revérifiés sur fiche, corps tourné en 32 × 29 mm (`A-FAIRE.md:190`). La broche 1, elle, n'est pas marquée sur le plan 48 broches de Yamaichi ; sa place dans l'empreinte vient d'**une seule source indirecte** (`tasks/todo-points-ouverts.md:75` : « broche 1 en doute écrit »). Si elle est fausse, le QFP-48 posé dans le support est tourné de 90° ou 180° : toutes ses broches sur le mauvais net, carte de test inutilisable.

Deux moitiés : Claude in Chrome relit l'empreinte telle que la LIB la sert et cherche sur le web une deuxième source (plan, photo du fabricant) ; toi seul peux regarder le repère moulé du vrai support.

## Ce que tu prépares
- WEB·SUITE lancé sur le PC (WEB_CAO sur `http://127.0.0.1:8000`), Claude in Chrome ouvert.
- Pour ta moitié : le support IC51-0484-806 (neuf ou monté), une loupe, un pied à coulisse.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-yamaichi-broche1.md. But : dire, d'après les documents, si la pastille 1 de
l'empreinte Yamaichi-IC51-0484-806 de la LIB est au bon coin. Tu ne modifies rien : ni la LIB,
ni un projet ; aucun bouton « 💾 Enregistrer », « Auto-associer », « Exporter CSV ».

1. Ouvre dans un onglet neuf (lecture seule, la LIB telle que WEB_CAO la sert) :
   http://127.0.0.1:8000/api/lib/fichier?type=pcb&nom=Yamaichi-IC51-0484-806.json
   Puis exécute dans cet onglet :
   (()=>{const f=JSON.parse(document.body.innerText).data;const p=f.pads||[];const b=f.body||{};const un=p.find(q=>String(q.n)==="1");const coin=q=>q?((q.x<(b.x1+b.x2)/2?"gauche":"droite")+"-"+(q.y<(b.y1+b.y2)/2?"haut":"bas")):"?";return JSON.stringify({body:b,nPads:p.length,p1:un,coinP1:coin(un),p2:p.find(q=>String(q.n)==="2"),p12:p.find(q=>String(q.n)==="12"),p13:p.find(q=>String(q.n)==="13"),dernier:p[p.length-1],holes:f.holes||[]},null,1)})()
   → à relever : le corps (x1, y1, x2, y2 ; lequel des deux côtés fait 32 mm), le nombre de
     pastilles (attendu 48), la pastille 1 (x, y, w, h) et son coin, les pastilles 2, 12, 13 et
     la dernière (sens de la numérotation : horaire ou antihoraire vu de dessus), les trous
     `holes` (x, y, d). Repère : mm, origine de l'empreinte, X vers la droite, Y VERS LE BAS,
     vue de dessus côté composant.
   Si la page répond 404 : ouvre http://127.0.0.1:8000/gestion-lib/gestion-lib.html, onglet
   « 📐 Empreintes PCB », filtre « Yamaichi », note le nom exact du fichier et refais l'étape avec.
2. Ouvre http://127.0.0.1:8000/gestion-lib/gestion-lib.html, onglet « 📐 Empreintes PCB »,
   filtre « IC51 ». Capture d'écran de la carte de l'empreinte.
   Attention : le point blanc de sérigraphie de cet aperçu est TOUJOURS dessiné au coin haut-gauche
   du corps (gestion-lib/js/02-rendu-pcb.js, _drawSilkscreen) ; il ne dit rien de la broche 1.
   Seule la pastille numérotée « 1 » compte.
3. Même page, onglet « 📋 Catalogue Composants (CSV) ». Exécute :
   JSON.stringify(LIB_STATE.composants.filter(c=>/IC51/i.test(JSON.stringify(c))),null,1)
   → à relever : Part Name, Empreinte PCB, Brochage, Manufacturer et toute colonne qui cite le
     fabricant ou sa référence.
4. Sur le web, cherche les documents du fabricant pour IC51-0484-806 (site Yamaichi Electronics :
   fiche produit, plan « outline drawing », plan de perçage recommandé « PCB layout ») et lis-les
   dans le navigateur.
   → à relever pour chaque document : URL exacte, page, figure ; le repère de la broche 1 sur le
     plan (triangle, chanfrein, « 1 », point) ou « aucun » ; la place des plots / trous de
     centrage par rapport à la broche 1 ; les cotes plot → première broche si elles y sont.
   Puis cherche une deuxième source : photo du fabricant montrant le repère moulé, plan du même
   support chez un distributeur (Digi-Key, Mouser) ou une empreinte publiée (SnapEDA, Ultra
   Librarian). Classe chaque source : PRIMAIRE (document Yamaichi) ou SECONDAIRE (tout le reste).
   N'ouvre que ce qui s'affiche dans le navigateur ; ne télécharge pas de zip ni de CAO.
5. Compare, vue de dessus, dans le repère de l'empreinte :
   → la pastille 1 de l'empreinte est-elle au coin / côté que les documents donnent à la broche 1 ?
   → les trous `holes` de l'empreinte tombent-ils où le plan met les plots de centrage, par
     rapport à la broche 1 (même côté, mêmes cotes à ±0,1 mm) ?
   → si un document donne la broche 1 vue de DESSOUS (côté picots), dis-le : elle est alors en
     miroir.

Rapport, en plus du format commun :
- EMPREINTE : le résultat brut de l'étape 1, et le coin de la pastille 1 en une phrase
  (ex. « pastille 1 en haut à gauche, sur le côté de 32 mm, numérotation antihoraire »).
- SOURCES : une ligne par source (URL, page, figure, PRIMAIRE / SECONDAIRE, ce qu'elle dit de la
  broche 1).
- VERDICT DOCUMENTAIRE : concordant / discordant (rotation de 90°, 180° ou 270°, sens) / documents
  muets sur la broche 1.
```

## C'est réussi si
- [ ] L'empreinte servie par la LIB est relevée : corps, 48 pastilles, coin et sens de la numérotation, trous de centrage.
- [ ] Au moins une source primaire Yamaichi lue (URL, page, figure), et ce qu'elle dit de la broche 1 (ou qu'elle est muette).
- [ ] Verdict documentaire : concordant / discordant / muet.
- [ ] Ta vérification sur le support (ci-dessous) : repère moulé, coin du picot 1, cote picot 1 → plot de centrage.
- [ ] Les deux disent la même chose. Si les documents sont muets, ta vérification seule tranche.

## Ce qui reste à toi
Le geste physique, avec le rapport de Claude in Chrome sous les yeux (coin de la pastille 1 et place des trous de centrage dans l'empreinte) :
1. Pose le support devant toi, côté où l'on insère la puce vers le haut. Cherche le repère moulé (triangle, point, chanfrein, « 1 » gravé), sur le corps ou sur le capot.
   → à relever : la nature du repère et son coin ou son côté.
2. Retourne-le (vue de dessous, côté picots). Repère le picot sous le repère et les plots de centrage. Attention : la vue de dessous est en miroir de l'empreinte.
3. Mesure au pied à coulisse la distance du picot 1 au plot de centrage le plus proche.
   → à relever : X, Y en mm, à ±0,2 mm.
4. Remets-toi en vue de dessus et dis si la pastille 1 de l'empreinte (rapport, étape 1) tombe sous le picot repéré. Une photo du repère (dessus et dessous) si tu peux.

## Après
Pour Claude Code, avec le rapport de Claude in Chrome et ta réponse :
- Concordant (documents et support, ou support seul si les documents sont muets) :
  - [ ] retirer le doute écrit de l'empreinte et citer la source trouvée ;
  - [ ] cocher `web_cao/A-FAIRE.md:198` et la mention `web_cao/A-FAIRE.md:47` ;
  - [ ] mettre à jour `web_cao/tasks/todo-points-ouverts.md:75` (« deux sources » au lieu d'une).
- Discordant :
  - [ ] renuméroter les pastilles de `PROJETS/LIB_CAO/lib_empreinte_pcb/Yamaichi-IC51-0484-806.json` (rotation de la numérotation seulement, trous et corps gardés) ;
  - [ ] mettre à jour la ligne de `LIB_composants.csv` ;
  - [ ] lister les cartes qui portent déjà ce support, puis cocher les mêmes cases.
