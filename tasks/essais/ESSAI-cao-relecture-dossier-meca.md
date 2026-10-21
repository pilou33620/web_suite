# Essai : relecture du dossier mécanique

Qui : Claude in Chrome (ta relecture humaine reste possible en plus, et le verdict est le tien) · Où : PC Windows du boulot (navigateur, github.com) · Durée : 1 h · Source : `web_cao/tasks/todo-dossier-meca.md:129`

## Pourquoi
La case demandait une relecture humaine avant la visionneuse ; le mode `/build auto` l'a reportée « à la fin ». La fin est arrivée (phase 3 faite, `todo-dossier-meca.md:131-178`) et seule la `/review` automatique est passée (l.183). Personne n'a relu le code. C'est un autre geste que l'import dans une CAO méca (`ESSAI-cao-dossier-meca-import.md`) et que l'essai de l'IA (`ESSAI-cao-dossier-meca-ia.md`) : ici on lit, on n'exécute pas.

Le dépôt WEB_CAO est public : Claude in Chrome lit le code sur github.com avec des questions précises et rend des remarques au format fixe. Les choix de métier (dégagement d'une vis, borne de hauteur) et le verdict restent à toi.

**Avant de donner l'essai** : les fentes (commit `b11bee2`, 21/10/2026) doivent être dans `main` de WEB_CAO, sinon la relecture porte sur un code sans elles (l'étape 0 le vérifie).

## Ce que tu prépares
Rien : Claude in Chrome n'a besoin que de github.com. Lis son rapport à la fin et tranche.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-relecture-dossier-meca.md. But : relire sur github.com le code du dossier
mécanique de WEB_CAO (dépôt public pilou33620/WEB_CAO, branche main) et rendre des remarques
précises. Tu ne modifies rien sur GitHub (pas d'issue, de commentaire, de fork, d'édition).

Méthode de lecture : fichiers bruts à https://raw.githubusercontent.com/pilou33620/WEB_CAO/main/<chemin>
Lignes numérotées qui contiennent un motif (remplace MOTIF) :
(()=>{const re=/MOTIF/;return document.body.innerText.split("\n").map((l,i)=>(i+1)+": "+l).filter(l=>re.test(l)).join("\n")})()
Extrait (remplace DEBUT et FIN) :
(()=>{const L=document.body.innerText.split("\n");return L.slice(DEBUT-1,FIN).map((l,k)=>(DEBUT+k)+": "+l).join("\n")})()
Les numéros cités ci-dessous datent du 21/10/2026 : cherche par nom de fonction, donne les
numéros que tu as lus.

0. Commit de main : https://github.com/pilou33620/WEB_CAO/commits/main (7 caractères). Historique
   du fichier commun : https://github.com/pilou33620/WEB_CAO/commits/main/commun/dossier-meca.js
   Les fentes sont-elles dans main ? Dans commun/dossier-meca.js, cherche « mecaOblong ».
   → si absent : écris « main sans les fentes (b11bee2) », relis le reste, saute les questions
     marquées FENTE.
1. La spécification d'abord : SPEC-dossier-meca.md (format JSON vers 110-140, architecture vers
   141-165, « Limites » vers 279-288 : Toujours / Demander d'abord / Jamais, critères vers 290-308).
2. LE CALCUL COMMUN, commun/dossier-meca.js (≈ 450 lignes ; lis-le en entier) :
   - `mecaDonnees` : fixations (trous seuls d'un composant H…/MOUNT, trous libres numérotés H1…),
     dégagement = pastille si plus grande que le trou, sinon Ø + 2 mm ; hauteurs saisie → fichier
     → LIB ; ouvertures (bord à moins de MECA_BORD = 2 mm, sinon face ; z_min / z_max) ;
     avertissements ; `repere`.
   - `mecaHauteur` et `MECA_H_MAX` (100 mm) ; la colonne LIB lue sous le nom « Maximun Height »
     (faute d'orthographe du CSV, gardée exprès).
   - `mecaDxf`, `mecaFichiers`, `mecaJsonTexte` ; FENTE : `mecaOblong`, `mecaDiam`, `mecaDemiDeg`.
   Questions : le JSON produit est-il celui que décrit la spec (noms, unités, repère, Z) ? Un
   composant de hauteur inconnue peut-il recevoir une hauteur inventée quelque part ? Le Z d'une
   ouverture de la face dessous est-il juste (de −ép−h à −ép) ? Une entrée vide ou absurde (contour
   vide, épaisseur nulle, hauteur négative, « 1e9 ») casse-t-elle quelque chose ? FENTE : le
   dégagement d'une fente est-il bien l'oblong élargi de (d_degagement − d)/2, et l'angle `rot`
   tourne-t-il dans le bon sens (Y vers le haut) ?
3. L'ÉDITEUR PCB : editeur-pcb/js/36-dossier-meca.js (≈ 134 lignes : `fenteMeca`,
   `mecaEntreeEditeur`, `mecaDossier`, `mecaFichiersEditeur`, `mecaExporter`, `mecaRegler`,
   `mecaRendreReglages`, `mecaCadre`) ; la feuille « Plan mécanique » dans
   editeur-pcb/js/29-draftsman.js (cherche `meca`) ; commun/zip.js.
   → attendu : rien n'écrit dans la LIB (« Jamais » de la spec) ; une saisie hors bornes est
     ignorée sans casser le document ; le passage du repère de l'éditeur (Y vers le bas) à celui
     des Gerber (Y vers le haut) est fait une fois, au bon endroit.
4. LA VISIONNEUSE ET L'IPC-2581 : visionneuse-ipc2581/js/08-dossier-meca.js (≈ 192 lignes :
   `mecaEntreeIpc`, corps du fichier ou estimé, NPTH rattaché à son boîtier, fente) ; le parseur
   python/ipc2581_parser.py (`_process_package`, vers 2157-2172 : `height`, `<Outline><Polygon>`) ;
   l'export de la hauteur editeur-pcb/js/35-ipc2581-export.js (vers 747-772, `<Package height>`).
   Questions : l'unité de `height` est-elle convertie (fichier en INCH ou MICRON) ? Un `height` à
   0 ou absent donne-t-il bien « pas de hauteur » ? L'éditeur écrit-il `height` dans l'unité que
   la visionneuse relit ? Les deux outils donnent-ils le même dossier pour la même carte (même
   entrée neutre) ?
5. Pour chaque problème, une remarque au format ci-dessous. Pas de remarque de style.

Format de chaque remarque :
  <fichier>:<ligne> (main @ <commit>) — BLOQUANT | À CORRIGER | REMARQUE — <constat en une
  phrase> — <ce qu'il faudrait à la place>

Rapport, en plus du format commun :
- COMMIT lu ; fentes présentes oui / non.
- Réponses aux questions des étapes 2 à 4, puis les remarques au format, rangées par partie
  (calcul commun, éditeur, visionneuse et IPC-2581).
- À TRANCHER PAR L'UTILISATEUR : la règle « Ø + 2 mm » pour le dégagement d'une vis, la borne
  100 mm, et tout choix de métier relevé.
- PROPOSITION DE VERDICT : « rien de bloquant » ou « à reprendre ».
```

## C'est réussi si
- [ ] Calcul commun : réponses et remarques (fichier:ligne, gravité) ou « rien à redire ».
- [ ] Éditeur : même chose.
- [ ] Visionneuse et IPC-2581 : même chose.
- [ ] Ton verdict : « relu, accepté » ou « relu, à reprendre ».

## Ce qui reste à toi
- Trancher les choix de métier : la règle « Ø + 2 mm » pour le dégagement d'une vis ; la borne de 100 mm (`MECA_H_MAX`) ; garder la colonne LIB « Maximun Height » sous ce nom.
- Donner le verdict ; relire toi-même les endroits marqués BLOQUANT si tu veux.

## Après
Pour Claude Code, avec le rapport et ton verdict :
- Si l'étape 0 dit « main sans les fentes » : pousser et fusionner `b11bee2` dans `main` de WEB_CAO, puis redonner la partie FENTE.
- Accepté :
  - [ ] cocher `web_cao/tasks/todo-dossier-meca.md:129`.
- Remarques :
  - [ ] les traiter une par une (commit et essai dans `web_cao/editeur-pcb/test/harness.js` ou `web_cao/visionneuse-ipc2581/test/harness-sim.js`), puis cocher.
- Le point de contrôle final (`web_cao/tasks/todo-dossier-meca.md:181`) attend aussi `ESSAI-cao-dossier-meca-ia.md`.
