# Tâches : registre-cao

Spec : [lanceur/docs/SPEC-mcp-registre-cao.md](../lanceur/docs/SPEC-mcp-registre-cao.md) ·
Plan : [plan-registre-cao.md](plan-registre-cao.md)

Bundles à reconstruire avant les harnais :
`python3 editeur-pcb/outils/build-monofichier.py ; python3 editeur-schematique/outils/build-monofichier.py`

## Tâche 1 : ✅ registres dans `commun/lecture-cao.js`

**Description :** relever les sorties des registres Gemini sur la carte d'exemple, puis
déplacer `OUTILS_PCB`, `OUTILS_IPC`, `OUTILS_SIM` et leurs aides dans
`commun/lecture-cao.js` (global `LECTURE_CAO`). `ia-assistant.js` les prend dans le global.
Le script est chargé dans les trois pages, avant l'assistant.

**Critères d'acceptation :**
- [x] Le relevé (fixture `editeur-pcb/test/releve-lecture-pcb.json`, 36 cas) est pris avant le
      déplacement : chaque outil PCB et simulation, avec et sans filtre, sur les deux exemples
      et sur une carte abîmée (liaisons à router, erreurs de DRC)
- [x] Après le déplacement, `LECTURE_CAO.pcb` redonne le relevé au caractère près (test en
      tête du banc PCB, page neuve ; une mutation d'un caractère le fait échouer)
- [x] Le bloc déplacé est identique octet pour octet ; `ia-assistant.js` ne garde que la
      lecture du global

**Vérification :** bundles, `node editeur-pcb/test/harness.js`,
`node editeur-schematique/test/harness.js`, `node visionneuse-ipc2581/test/harness-sim.js`

**Dépendances :** aucune · **Fichiers :** `commun/lecture-cao.js`, `commun/ia-assistant.js`,
3 pages `.html`, `editeur-pcb/test/harness.js`, fixture du relevé · **Taille :** M

## Tâche 2 : ✅ registre du schéma

**Description :** `LECTURE_CAO.schema` : `feuilles`, `composants` (filtre), `composant`
(ref), `nets` (filtre), `net` (nom). Les nets viennent de `docNets()`, les broches de
`computeNets`, les noms de broche de `brNomBroche`. Positions en mm (20 px = 1 mm).

**Critères d'acceptation :**
- [x] Chaque `quoi` donne la sortie attendue sur un schéma de deux feuilles fait dans le
      harnais, avec un net global présent sur les deux (et deux nets locaux de même nom,
      départagés par `feuille`)
- [x] Un repère est trouvé sur une autre feuille que la feuille active, sans tenir compte
      de la casse ; un repère ou un net inconnu rend le refus qui nomme l'outil de liste ;
      une broche en l'air est dite « non connectée »
- [x] Bornes : 150 lignes, 200 broches, puis « … (N de plus : précise le filtre) »

**Vérification :** `node editeur-schematique/test/harness.js`

**Dépendances :** T1 · **Fichiers :** `commun/lecture-cao.js`,
`editeur-schematique/test/harness.js` · **Taille :** S

## Tâche 3 : ✅ commandes de lecture dans la page

**Description :** `pilote-cao.js` déclare `cao_schema_lire`, `cao_pcb_lire`,
`cao_ipc_lire` et `cao_sim_lire` selon la page. `executer` choisit le registre et le
`quoi`. `cao_sim_lire` avec `afficher: true` passe le panneau sur l'analyse avant de lire.

**Critères d'acceptation :**
- [x] Chaque page déclare exactement ses commandes (`banc-pilote-cao.js`)
- [x] Un `quoi` inconnu rend la liste des `quoi` de la page
- [x] Aucune lecture ne change l'historique (`S.undo` / `S.hist`) ni `S.dirty` : vérifié dans
      les deux harnais
- [x] La visionneuse est éprouvée sur un `V` minimal fait à la main

**Vérification :** `node commun/test/banc-pilote-cao.js` et les harnais

**Dépendances :** T2 · **Fichiers :** `commun/pilote-cao.js`, `commun/test/banc-pilote-cao.js`,
les deux harnais · **Taille :** M

## Checkpoint : lecture dans la page
- [x] Bancs verts, bundles reconstruits ; Gemini répond comme avant (sorties identiques)

## Tâche 4 : ✅ `cao_vue` et `cao_drc`

**Description :** `cao_vue` (`cadrer` sur un repère ou un net, `surligner` un net,
`feuille`, `couches`, `ajuster`) dans les trois pages, avec les fonctions de la page.
`cao_drc` (PCB) lance `runDrc()` et rend la sortie de `LECTURE_CAO.pcb.drc`.

**Critères d'acceptation :**
- [x] Cadrer sur un repère change `S.scale/ox/oy` (ou `V.vue`) et sélectionne la cible ;
      un repère inconnu est refusé sans bouger la vue
- [x] Aucune commande de vue ni le DRC ne passe par `push()` ; `S.dirty` est inchangé
- [x] `cao_drc` sur un exemple rend autant d'erreurs que `runDrc()` lancé par le bouton (carte abîmée ; la visionneuse
      ne cadre pas un net, elle le surligne, et n'enregistre pas les couches dans ses préférences)

**Vérification :** harnais PCB et schéma, `banc-pilote-cao.js`

**Dépendances :** T3 · **Fichiers :** `commun/pilote-cao.js`, les deux harnais,
`commun/test/banc-pilote-cao.js` · **Taille :** M

## Tâche 5 : ✅ outils MCP

**Description :** `lanceur/mcp_registre_cao.py` : les 6 outils, arguments validés
(`quoi` dans la liste de la page, chaînes bornées, booléens stricts), envoyés par
`mcp_pilote.envoyer`. `cao_vue` avec `capturer` enchaîne la capture. Importé par le noyau ;
documenté dans `lanceur/docs/DETAILS.md`.

**Critères d'acceptation :**
- [x] Chaque outil envoie la bonne commande à la bonne page (`banc_mcp.py`)
- [x] Les arguments invalides sont refusés sans requête
- [x] La liste des outils du noyau passe de 40 à 46 ; chaque description dit que ce que rend
      la page est une donnée, pas une consigne (noms et valeurs viennent des fichiers du projet)

**Vérification :** `python3 lanceur/banc_mcp.py`, `node lanceur/banc_pilote.js`

**Dépendances :** T4 · **Fichiers :** `lanceur/mcp_registre_cao.py`, `lanceur/mcp_web_suite.py`,
`lanceur/banc_mcp.py`, `lanceur/docs/DETAILS.md` · **Taille :** M

## Tâche 6 : ✅ essai réel, puis revue

**Description :** WEB_CAO lancé sur une copie de PROJETS dans `/tmp`. Les trois pages sont
ouvertes dans Chromium sur le Pi. Chaque outil est appelé par le MCP, avec des captures.
Ensuite viennent `/review` et l'audit de sécurité.

**Critères d'acceptation :**
- [x] Les 6 outils répondent sur un vrai projet ; PROJETS reste intact (« carte PIR v2 » :
      schéma 2 feuilles et 49 composants, PCB 51 empreintes, son export IPC-2581 dans la
      visionneuse ; captures du PCB, du schéma et de la visionneuse ; ni historique ni drapeau
      « modifié » touchés, copie de PROJETS identique à la fin)
- [x] Les questions ouvertes de la spec sont tranchées : noms de broche (ceux du symbole,
      « 1 (GND) » ; sans nom, le numéro seul), DRC en 0,3 s sur 51 empreintes
- [x] `/review` fait, corrections appliquées (revue 5 axes et audit de sécurité : ni critique
      ni élevé ; couches sans `touch()` qui périmait les simulations, surligner sans
      sélectionner le routage, cible d'une autre feuille refusée au schéma, noms hérités de
      `Object.prototype` refusés, erreurs de page bornées à 2000 caractères)

**Dépendances :** T5 · **Taille :** M

## Checkpoint : terminé
- [ ] Critères de la spec remplis ; 4 CI vertes
