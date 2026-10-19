# Tâches : retrait-gemini

Spec : [lanceur/docs/SPEC-mcp-retrait-gemini.md](../lanceur/docs/SPEC-mcp-retrait-gemini.md) ·
Plan : [plan-retrait-gemini.md](plan-retrait-gemini.md)

## Tâche 1 : WEB_ANTENNA, le registre des cartes déplacé

**Description :** `js/31-cartes-antenna.js` (`CARTES_ANTENNA`) reçoit, sans rien y changer :
- IA_CHAMPS ;
- resoudre, valider, proposition et ses validateurs ;
- appliquer, annuler, controles et grammaire ;
- les utilitaires dont ils dépendent.

`30-ia.js` les reprend de là. `pilote-antenna.js`, `29-lecture-antenna.js` et
`banc-interface.js` §9 passent aux nouveaux noms.

**Critères d'acceptation :**
- [ ] Relevé `test/releve-lecture-antenna.json` identique ; §9 vert avec les seuls noms changés
- [ ] Panneau IA encore fonctionnel (il lit CARTES_ANTENNA) ; bancs WEB_ANTENNA verts

**Vérification :** `node test/banc-interface.js`, `python3 python/test/banc-openems.py`,
`python3 python/test/banc-serveur.py`

**Dépendances :** aucune · **Fichiers :** `js/31-cartes-antenna.js`, `js/30-ia.js`,
`js/pilote-antenna.js`, `js/29-lecture-antenna.js`, `index.html`, `test/banc-interface.js` · **Taille :** M

## Tâche 2 : WEB_ANTENNA, le panneau IA retiré

**Description :**
- retirer `js/30-ia.js`, `css/ia.css`, leurs balises, le bouton et les crochets (menu
  contextuel, raccourcis) ;
- effacer une fois `openems_ia_cle` et `openems_ia_local` du stockage de session ;
- mettre à jour README et A-FAIRE.

**Critères d'acceptation :**
- [ ] Aucune trace (`gemini`, `gemma`, `generativelanguage`, `30-ia.js`, `iaOuvrir`…) dans le
      code ; bancs verts ; les cartes et l'audit marchent (banc §9, banc du canal)

**Vérification :** bancs de T1

**Dépendances :** T1 · **Taille :** M

## Tâche 3 : WEB_CAO, l'assistant de Gestion LIB retiré

**Description :**
- retirer `gestion-lib/js/06-ia-lib.js`, son panneau et ses boutons (`iaActionRapide`,
  `iaLib*`), l'étape `#bModeIaLib` du tutoriel et la partie IA de `banc-ia-xss.js` ;
- garder ce que `banc-ia-xss.js` éprouve d'autre (le banc est renommé s'il ne reste rien
  d'IA).

**Critères d'acceptation :**
- [ ] Gestion LIB sans trace ; bancs de Gestion LIB verts

**Vérification :** `node gestion-lib/test/*.js`, `node commun/test/banc-pilote-cao.js`

**Dépendances :** aucune · **Taille :** S

## Tâche 4 : WEB_CAO, l'assistant des trois pages retiré

**Description :**
- retirer `commun/ia-assistant.js` et `.css` ;
- retirer panneaux et balises (PCB, schéma, visionneuse), le clic droit (`05-tools.js`,
  `09-interaction.js`, `04-interaction.js`), l'étape `#bIaAssistant` du tutoriel et le
  bouton « Envoyer à l'IA » du rapport ;
- retirer « ia » des `00-espace-config.js` et de `profils/Pilou.json`, et
  `banc-ia-markdown.js` ;
- réécrire pour le MCP les commentaires qui citent l'assistant ;
- mettre à jour README et A-FAIRE.

**Critères d'acceptation :**
- [ ] Aucune trace dans le code ni dans les bundles reconstruits ; harnais PCB, schéma,
      visionneuse et banc du canal verts

**Vérification :** bundles, puis les trois harnais et `banc-pilote-cao.js`

**Dépendances :** T3 (même dépôt, tutoriel commun) · **Taille :** L

## Checkpoint : les pages
- [ ] Bancs verts dans les deux outils

## Tâche 5 : serveurs, `/api/ia/cle` retiré

**Description :**
- `web_CAO.py` et `web_antenna.py` : la route, sa lecture du fichier de clé, son entrée
  CORS ;
- les tests de refus deviennent des tests de 404 ;
- restent : `.gitignore` et la liste des fichiers préservés.

**Critères d'acceptation :**
- [ ] `GET /api/ia/cle` rend 404 sur les deux serveurs (bancs)

**Vérification :** `python3 python/test/banc-serveur-routes.py`, `python3 python/test/banc-lib-routes.py`
(WEB_CAO), `python3 python/test/banc-serveur.py` (WEB_ANTENNA)

**Dépendances :** T2, T4 · **Taille :** S

## Tâche 6 : web_suite, CI et docs

**Description :**
- `lanceur/banc_traces_ia.py` (recherche des traces, avec ses exceptions, éprouvé sur un
  factice) dans la CI ;
- le contrôle « ia.css préfixe d'ia-assistant.css » retiré ;
- README racine et DETAILS.md mis à jour.

**Critères d'acceptation :**
- [ ] Le banc trouve une trace plantée dans un fichier factice et rien dans les dépôts ; CI à jour

**Vérification :** `python3 lanceur/banc_traces_ia.py`, `python3 lanceur/banc_mcp.py`

**Dépendances :** T5 · **Taille :** S

## Tâche 7 : essai réel, puis revue

**Description :** dans Chromium (copie de PROJETS), charger chaque page :
- PCB, schéma, visionneuse, Gestion LIB, WEB_ANTENNA ;
- console sans erreur ; clic droit et tutoriel joués ;
- `antenna_proposer` de bout en bout et `antenna_lire` (quoi=audit).

Puis `/review`.

**Critères d'acceptation :**
- [ ] Cinq pages sans erreur de console ; carte d'antenne proposée, cliquée, annulée
- [ ] `/review` fait, corrections appliquées

**Dépendances :** T6 · **Taille :** M

## Checkpoint : terminé
- [ ] Critères de la spec remplis ; 4 CI vertes
