# Tâches : canal-pilote

Plan : [plan.md](plan.md) · Spec : [lanceur/docs/SPEC-mcp-canal-pilote.md](../lanceur/docs/SPEC-mcp-canal-pilote.md)

## Tâche 1 : ✅ vérifications en lecture seule

**Description :** répondre à quatre questions par la lecture du code et une vérification
rapide, puis corriger la spec.
1. `sim*Go()` (`commun/simulation-em.js`) rend-elle une promesse qui se résout à la fin du
   calcul ? Sinon, quel état observer pour savoir que c'est fini ou en erreur ?
2. Schéma : comment trouver un composant par son repère ? Quelles fonctions rafraîchissent
   l'affichage après un changement de valeur ? Ce qu'appelle `schema_values`
   (`ia-assistant.js:1391-1460`) sert de modèle.
3. WEB_ANTENNA : signatures exactes de `iaProposition`, `iaAppliquer` et `iaAnnuler`, et
   leur règle « annuler seulement si rien n'a bougé ».
4. `navigator.locks` et `crypto.randomUUID` sont-ils absents sur une page HTTP servie par
   une adresse IP ? Vérification dans Chromium sur le Pi.

**Critères d'acceptation :**
- [x] Les quatre réponses sont dans la spec (`fichier:ligne`)
- [x] La spec est corrigée pour l'élection du relais (BroadcastChannel) si la réponse 4
      le confirme
- [x] La tâche 7 est précisée selon la réponse 1

**Vérification :** relecture avec toi au checkpoint socle

**Dépendances :** aucune · **Fichiers :** `lanceur/docs/SPEC-mcp-canal-pilote.md`, `tasks/todo.md` · **Taille :** XS

## Tâche 2 : ✅ `lanceur/pilote.py` et son auto-test

**Description :**
- `traiter(handler)` sert les six routes `/api/pilote/*` de la spec ;
- l'état est en mémoire et protégé par un verrou ;
- un onglet sans battement depuis plus de 60 s est oublié ;
- les routes d'ordre n'acceptent que la boucle locale ;
- `python3 pilote.py` lance l'auto-test.

**Critères d'acceptation :**
- [x] Une commande déposée est rendue au long-poll du navigateur de l'onglet visé ; sa
      réponse débloque l'appel `commande` qui attendait
- [x] Sans réponse dans le délai, `commande` rend `en_attente` avec un id, et `suivre` le
      rend ensuite fini quand la réponse arrive
- [x] Par défaut, la cible est l'onglet du bon type au dernier focus ; un onglet inconnu
      ou fermé donne une erreur lisible
- [x] `commande`, `onglets` et `suivre` répondent 403 hors de la boucle locale ; un corps
      illisible ou trop gros répond 400 ou 413

**Vérification :** `python3 lanceur/pilote.py` affiche OK

**Dépendances :** T1 · **Fichiers :** `lanceur/pilote.py` · **Taille :** M

## Tâche 3 : ✅ MCP : `mcp_pilote.py`, image dans le noyau, banc

**Description :**
- Les outils MCP `pilote_onglets`, `pilote_suivre`, `pilote_capture`, `cao_calcul`,
  `cao_valeur_composant` et `antenna_proposer` : arguments validés, appel à
  `/api/pilote/commande` avec `delai=130`.
- Le noyau rend un contenu `image` quand un outil rend une capture.
- `banc_mcp.py` : un faux outil sert `/api/pilote/*`.

**Critères d'acceptation :**
- [x] Chaque outil envoie la bonne commande (nom, page, arguments, `ecriture`)
- [x] `pilote_capture` rend un contenu MCP `image` (PNG en base64), sous 2 Mo
- [x] Arguments refusés sans requête (analyse inconnue, repère vide, type de carte
      ANTENNA inconnu) ; `en_attente` se suit avec `pilote_suivre`

**Vérification :** `python lanceur/banc_mcp.py` OK, ainsi que les autres bancs du lanceur

**Dépendances :** T2 · **Fichiers :** `lanceur/mcp_pilote.py`, `lanceur/mcp_web_suite.py`, `lanceur/banc_mcp.py` · **Taille :** M

## Checkpoint : socle
- [ ] Bancs du lanceur verts
- [ ] Revue avec toi (avec les réponses de la tâche 1)

## Tâche 4 : ✅ `lanceur/pilote.js` : relais, long-poll, registre

**Description :**
- id d'onglet et page déclarée ;
- élection du relais par BroadcastChannel (annonce toutes les 2 s, le plus petit id
  parmi les onglets entendus depuis moins de 5 s) ;
- le relais fait les battements et le long-poll, et transmet les commandes à l'onglet
  visé ;
- `Pilote.commande(nom, def)` ;
- une écriture sans `annuler` ni historique déclaré est refusée.

Banc Node : `lanceur/banc_pilote.js`, avec un faux `fetch` et plusieurs « onglets » sur un
vrai BroadcastChannel de Node.

**Critères d'acceptation :**
- [x] Trois onglets donnent un seul relais et un seul long-poll ; fermer le relais en fait
      élire un autre en moins de 5 s, et la commande en attente aboutit
- [x] Une commande inconnue de la page est rendue en erreur lisible ; une commande de
      lecture (capture) s'exécute sans carte
- [x] La déclaration d'une écriture sans moyen d'annuler est refusée

**Vérification :** `node lanceur/banc_pilote.js` OK

**Dépendances :** T2 · **Fichiers :** `lanceur/pilote.js`, `lanceur/banc_pilote.js` · **Taille :** M

## Tâche 5 : ✅ panneau « Claude », cartes, interrupteur, annuler

**Description :**
- `pilote.css` et le panneau, replié par défaut, qui s'ouvre à l'arrivée d'une carte ;
- la carte affiche avant → après, avec Appliquer / Refuser ;
- après application, « Annuler » n'agit que si l'état n'a pas bougé (comparaison par
  instantané) ; sinon, la carte renvoie au Ctrl+Z ;
- l'interrupteur est propre à l'onglet, éteint au chargement ;
- tout le texte passe par `textContent`.

**Critères d'acceptation :**
- [x] Rien n'est appliqué avant le clic, et la réponse rendue est « refusée » sur un refus
- [x] Interrupteur allumé : la commande s'applique sans carte, mais la trace reste dans le
      panneau
- [x] Un texte `<img onerror=…>` envoyé par Claude s'affiche tel quel, et rien n'est
      exécuté

**Vérification :** `node lanceur/banc_pilote.js` OK

**Dépendances :** T4 · **Fichiers :** `lanceur/pilote.js`, `lanceur/pilote.css`, `lanceur/banc_pilote.js` · **Taille :** M

## Checkpoint : canal complet
- [ ] `banc_pilote.js` et `banc_mcp.py` verts
- [ ] Revue avec toi

## Tâche 6 : ✅ WEB_CAO : branchement, pages, captures

**Description :**
- copier `pilote.py`, `commun/pilote.js` et `commun/pilote.css` ;
- `pilote.traiter(self)` dans l'aiguillage de `web_CAO.py` ;
- balises dans les pages schéma, PCB, gestion-lib et visionneuse IPC ;
- `Pilote.capture` sur les canevas `#sheet`, `#board` et `#carte` (rendu hors écran,
  comme `exportPng`) ;
- reconstruire les monofichiers.

**Critères d'acceptation :**
- [x] `pilote_onglets` voit les quatre pages, avec leur projet
- [x] `pilote_capture` rend le schéma, le PCB et la visionneuse ; gestion-lib répond
      « pas de canevas »
- [x] Les bancs WEB_CAO restent verts (`harness.js` PCB et schéma, routes du serveur) et
      les monofichiers sont reproductibles

**Vérification :** bancs WEB_CAO, plus un essai rapide avec Chromium

**Dépendances :** T3, T5 · **Fichiers :** WEB_CAO : `pilote.py`, `commun/pilote.*`, `web_CAO.py`, 4 pages, `dist/` · **Taille :** M

## Tâche 7 : ✅ WEB_CAO : `cao_calcul`

**Description :** commande `cao_calcul` dans les pages PCB et visionneuse IPC :
- choisir l'analyse (`simAllerFamille` / `simAllerAnalyse`), puis faire `await` sur son
  lanceur, selon la correspondance de la spec (question tranchée 1) ;
- signaler `SIM.occupe` et `!simCalculable()` au lieu de ne rien faire ;
- rendre `simRapMarkdown()` ;
- afficher la trace « Claude a lancé : … » dans le panneau.

**Critères d'acceptation :**
- [x] Les 9 analyses de la spec sont acceptées ; une analyse impossible (rien de
      sélectionné) rend le message de la page
- [x] Un calcul qui finit après 2 min se suit avec `pilote_suivre`

**Vérification :** banc Node avec un faux `SIM`, puis un vrai calcul d'impédance avec
Chromium

**Dépendances :** T6 · **Fichiers :** WEB_CAO : `commun/simulation-em.js` ou un fichier voisin, `dist/` · **Taille :** S

## Tâche 8 : ✅ WEB_CAO : `cao_valeur_composant`

**Description :** commande d'écriture dans la page schéma :
- proposer : avant et après, par repère ;
- appliquer : `push()`, puis la valeur, puis rafraîchir l'affichage ;
- l'undo passe par l'historique de l'éditeur.

**Critères d'acceptation :**
- [x] Carte puis clic : la valeur change, et un seul Ctrl+Z la défait
- [x] Repère inconnu : erreur lisible, rien n'est poussé dans l'historique

**Vérification :** `editeur-schematique/test/harness.js`, enrichi de ce cas

**Dépendances :** T6 · **Fichiers :** WEB_CAO : un fichier de l'éditeur schématique, `harness.js`, `dist/` · **Taille :** S

## Tâche 9 : ✅ WEB_ANTENNA : branchement, capture, `antenna_proposer`

**Description :**
- copier les trois fichiers et brancher `web_antenna.py` ;
- capture du canevas `#carte` ;
- `antenna_proposer`, qui passe par `iaProposition`, `iaAppliquer` et `iaAnnuler` ;
- `etat()` déclaré pour la règle « annuler seulement si rien n'a bougé », et annulation
  du balayage par l'adaptateur, qui garde et remet `ANT.balayage`.

**Critères d'acceptation :**
- [x] Les cartes `reglages`, `cotes`, `balayage` et `formes` passent par le panneau ;
      Annuler défait l'application
- [x] Un balayage appliqué est armé, jamais lancé
- [x] `python/test/banc-serveur.py` reste vert

**Vérification :** banc serveur, plus un essai avec Chromium

**Dépendances :** T3, T5 · **Fichiers :** WEB_ANTENNA : `pilote.py`, `js/pilote.js`, `css/pilote.css`, `web_antenna.py`, `index.html`, `js/30-ia.js` · **Taille :** M

## Tâche 10 : ✅ WEB_3D : branchement, capture

**Description :** copier les trois fichiers, brancher `web_3D.py` (dans `do_GET` et
`do_POST`, avant les fichiers statiques), et faire déclarer `Pilote.capture` par
`09-demarrage.js` à partir de `Vue3D.capture()`.

**Critères d'acceptation :**
- [x] `pilote_capture` rend la vue 3D ; WEB_3D apparaît dans `pilote_onglets`

**Vérification :** essai avec Chromium

**Dépendances :** T3, T5 · **Fichiers :** WEB_3D : `pilote.py`, `js/pilote.js`, `css/pilote.css`, `web_3D.py`, `index.html`, `js/09-demarrage.js` · **Taille :** S

## Checkpoint : outils branchés
- [ ] Bancs de chaque dépôt verts
- [ ] Revue avec toi

## Tâche 11 : ✅ CI des copies et documentation

**Description :**
- la CI de web_suite vérifie que `pilote.py`, `pilote.js` et `pilote.css` sont identiques
  dans les trois outils, et lance `banc_pilote.js` et l'auto-test ;
- DETAILS.md : le panneau, l'interrupteur, les nouveaux outils MCP ;
- la carte `SPEC-mcp.md`.

**Critères d'acceptation :**
- [ ] CI de web_suite verte après le push des quatre dépôts
- [x] Documentation à jour

**Dépendances :** T6 à T10 · **Fichiers :** `.github/workflows/ci.yml`, `lanceur/docs/DETAILS.md`, `README.md`, `lanceur/docs/SPEC-mcp.md` · **Taille :** S

## Tâche 12 : essai réel, puis revue

**Description :** sur le Pi, lancer WEB_CAO, WEB_ANTENNA et WEB_3D sur une copie de
PROJETS, avec Chromium piloté par Playwright : deux onglets WEB_CAO, un ANTENNA, un 3D.
Dérouler les essais de la spec, dont la fermeture du relais pendant une attente. Puis
`/review` et l'audit de sécurité.

**Critères d'acceptation :**
- [ ] Tous les essais de la spec passent ; PROJETS reste intact
- [ ] `/review` fait, corrections appliquées

**Vérification :** essai Playwright ; essai iPad avec toi

**Dépendances :** T11 · **Fichiers :** aucun (corrections éventuelles) · **Taille :** M

## Checkpoint : terminé
- [ ] Critères de réussite de la spec remplis
- [ ] Essai sur l'iPad avec toi
