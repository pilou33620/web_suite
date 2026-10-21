# Essai : critère 6 du dossier méca, une IA lit le JSON et dessine le boîtier

Qui : Claude in Chrome (contrôle final par toi) · Où : PC Windows du boulot (WEB·SUITE et claude.ai dans le navigateur) · Durée : 25 min · Source : `web_cao/tasks/todo-dossier-meca.md:181` (critère 6 : `web_cao/SPEC-dossier-meca.md:306-308`)

## Pourquoi
Le JSON `-MECA.json` (format `web_cao-meca-1`, `commun/dossier-meca.js:23`, décrit `SPEC-dossier-meca.md:110-140`) est fait pour qu'une IA conçoive le boîtier **sans rien d'autre**. Le critère 6 dit : à partir du seul JSON, elle en déduit les dimensions intérieures minimales du boîtier et la position des entretoises ; on contrôle sa réponse à la main. Les critères 1 à 5 sont tenus par les essais ; le 6 ne peut l'être que par un vrai essai. S'il échoue, le JSON n'est pas assez explicite (repère, unités, hauteurs inconnues) et une IA dessinera un boîtier faux sans le dire.

Claude in Chrome prépare la même carte que `ESSAI-cao-dossier-meca-import.md` (exemple 2, quatre trous Ø 3,2 et une fente, rien n'est enregistré), tire le JSON de la page, le donne à une **nouvelle** discussion claude.ai avec une question fixe, puis compare la réponse aux valeurs du JSON. L'IA interrogée n'a vu que le JSON ; Claude in Chrome, lui, n'est que le messager et le correcteur.

## Ce que tu prépares
- WEB·SUITE lancé (WEB_CAO sur `http://127.0.0.1:8000`, avec la LIB) ; WEB_CAO à jour avec les fentes (commit `b11bee2`).
- claude.ai ouvert et connecté dans le même Chrome. Si ton compte a la mémoire des discussions ou des projets, prévois une discussion hors projet (incognito si proposé) : l'IA ne doit rien savoir de WEB_CAO.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-dossier-meca-ia.md. But : donner le JSON du dossier mécanique d'une carte
d'essai à une NOUVELLE discussion claude.ai, avec une question fixe, et corriger sa réponse
contre le JSON. Rien n'est enregistré dans WEB_CAO (pas de Ctrl+S, pas de « ☁ Enregistrer »,
pas de téléchargement). Dans claude.ai tu n'ajoutes rien au message, tu ne relances pas, tu ne
corriges pas l'IA : une seule question, une seule réponse.

1. Onglet neuf : http://127.0.0.1:8000/editeur-pcb/editeur-pcb.html, puis (exemple 2, quatre trous
   Ø 3,2 aux coins, une fente Ø 1,2 × 3,2 horizontale) :
   (()=>{push();loadDoc(exemple2());for(const [x,y] of [[3.5,3.5],[62.5,3.5],[3.5,38.5],[62.5,38.5]])S.holes.push(mkHole(x,y,3.2));const f=mkHole(33,3.5,1.2);f.w=3.2;S.holes.push(f);touch();fit();return [S.fps.length,S.holes.length,typeof mecaOblong];})()
   → attendu : [11, 5, "function"]. Si "undefined" : WEB_CAO n'a pas les fentes ; arrête-toi et
     dis-le (l'essai n'a de sens qu'avec elles).
2. Valeurs de contrôle :
   (()=>{const J=mecaDossier();return JSON.stringify({enc:J.carte.encombrement,ep:J.carte.epaisseur,hmax:J.hauteur_max,fix:J.fixations,ouv:J.composants.filter(c=>c.ouverture).map(c=>[c.ref,c.ouverture]),sansHauteur:J.composants.filter(c=>c.hauteur==null).map(c=>c.ref),avert:J.avertissements,taille:mecaJsonTexte(J).length},null,1)})()
   → attendu : enc {x1:0, y1:0, x2:66, y2:42} ; ep 1.6 ; H1 (3.5, 38.5), H2 (62.5, 38.5),
     H3 (3.5, 3.5), H4 (62.5, 3.5) Ø 3.2, d_degagement 5.2 ; H5 (33, 38.5) d 1.2, l 3.2, rot 0,
     d_degagement 3.2 ; ouvertures J1, J2, J3 de type « face », face « dessus ».
   → à relever : hmax (dessus, dessous, total), les composants sans hauteur, les avertissements.
     Garde ce résultat : c'est ton corrigé.
3. Copie le JSON entier dans le presse-papiers :
   navigator.clipboard.writeText(mecaJsonTexte(mecaDossier())).then(()=>"copié",e=>"refusé : "+e.message)
   Si c'est refusé : ouvre-le en texte dans un onglet,
   window.open(URL.createObjectURL(new Blob([mecaJsonTexte(mecaDossier())],{type:"text/plain"})))
   et sélectionne-le tout (Ctrl+A, Ctrl+C) ; si ça échoue aussi, demande à l'utilisateur de faire
   Ctrl+A, Ctrl+C dans cet onglet.
   Ne ferme pas l'onglet du PCB à la fin (le navigateur demanderait « Quitter le site ? »).
4. Nouvel onglet : https://claude.ai/new (discussion neuve, hors projet ; incognito si le choix
   existe). Note le modèle affiché. Colle le JSON (Ctrl+V ; il peut devenir une pièce jointe
   « texte collé », c'est bien), puis tape à la suite ce texte, mot pour mot, et envoie :
   « Voici le dossier mécanique d'une carte électronique. Donne : (1) les dimensions intérieures minimales d'un boîtier qui la contient (X, Y, Z, en mm, en disant le jeu que tu prends) ; (2) la position de chaque entretoise (X, Y dans le repère du fichier) et son diamètre maximal ; (3) les ouvertures à prévoir dans le boîtier ; (4) ce que tu ne peux pas savoir avec ce fichier. »
   Attends la réponse complète. Recopie-la entière.
5. Corrige (1) contre ton corrigé de l'étape 2.
   → attendu : X ≥ 66, Y ≥ 42 (plus le jeu annoncé) ; Z ≥ hmax.total (dessous + épaisseur +
     dessus), sans oublier la face dessous.
6. Corrige (2).
   → attendu : une entretoise à chacun de H1 à H4, aux X, Y du JSON (pas de miroir, pas de
     décalage d'origine) ; diamètre ≤ 5,2 mm (d_degagement) ; H5 reconnue comme une FENTE de
     1,2 × 3,2 mm (vis, pion ou détrompeur), pas comme une entretoise ronde.
7. Corrige (3).
   → attendu : des ouvertures en face dessus pour J1, J2, J3 (ce que dit le JSON : type
     « face ») ; s'il y avait une ouverture de type « bord », sa plage `debut` / `fin` et sa
     hauteur `z_min` / `z_max`. Une ouverture inventée (sur un bord que le JSON ne cite pas) est
     une faute.
8. Corrige (4).
   → attendu : l'IA cite les composants sans hauteur (`source_hauteur: "inconnue"`, les
     avertissements) au lieu de leur inventer une hauteur ; si elle prend une hauteur par défaut,
     elle le dit.

Rapport, en plus du format commun :
- IA : nom et modèle affichés par claude.ai, date et heure.
- CORRIGÉ : le résultat brut de l'étape 2.
- RÉPONSE : la réponse entière de l'IA, recopiée.
- NOTES : (1) à (4), chacune « juste » / « faux » avec la valeur attendue et la valeur donnée ;
  pour chaque faute, ta lecture de la cause : JSON ambigu (quel champ) ou erreur de l'IA alors
  que le JSON était clair.
```

## C'est réussi si
- [ ] (1) X, Y, Z intérieurs justes (ou l'écart et sa cause).
- [ ] (2) Entretoises aux bonnes cotes, bon Ø max, fente comprise.
- [ ] (3) Ouvertures justes (J1, J2, J3 en face dessus, rien d'inventé).
- [ ] (4) Hauteurs inconnues signalées, rien d'inventé.
- [ ] L'IA utilisée (nom, modèle) et sa réponse entière dans le rapport.
- [ ] Ton contrôle final de la correction (ci-dessous).

## Ce qui reste à toi
- Relire la correction de Claude in Chrome contre la réponse recopiée : c'est toi qui dis si le critère est tenu (une IA qui en corrige une autre peut se tromper dans le même sens).
- À la fin, fermer l'onglet du PCB sans enregistrer (« Quitter »).

## Après
Pour Claude Code, avec le rapport et ton verdict :
- Les quatre justes :
  - [ ] cocher le critère 6 (`web_cao/SPEC-dossier-meca.md:306-308`) et `web_cao/tasks/todo-dossier-meca.md:181`.
- Une erreur qui vient du JSON (repère mal compris, Z de la face dessous, fente prise pour un rond) :
  - [ ] préciser le champ `repere` ou les `avertissements` dans `web_cao/commun/dossier-meca.js` (`mecaDonnees`), et `web_cao/SPEC-dossier-meca.md:110-140` ;
  - [ ] ajouter l'essai au harnais PCB (`web_cao/editeur-pcb/test/harness.js`), puis redonner cet essai.
- Une erreur de l'IA seule (le JSON était clair) : la noter dans la case, et décider avec l'utilisateur si le critère est tenu.
