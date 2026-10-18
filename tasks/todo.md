# Tâches : api-outils

Plan : [plan.md](plan.md) · Spec : [lanceur/docs/SPEC-mcp-api-outils.md](../lanceur/docs/SPEC-mcp-api-outils.md)

## Tâche 1 : ✅ vérifier l'enregistrement de l'éditeur et d'ANTENNA (lecture seule)

**Description :** répondre à deux questions par la lecture du code, sans rien modifier.
1. Quand l'éditeur de WEB_CAO (schéma ou PCB) enregistre un document qu'il avait ouvert,
   envoie-t-il `modifie` ? Et sur un 409, prévient-il ou écrase-t-il ? Chercher les appels
   `PUT /api/projet/doc` dans le JS.
2. Est-ce que `POST /api/projet/enregistrer` de WEB_ANTENNA (`_pr_enregistrer`) change le
   projet ouvert quand `nom` est un autre projet ?

**Critères d'acceptation :**
- [x] Les deux réponses sont écrites dans la spec, avec `fichier:ligne`
- [x] (sans objet : l'éditeur demande avant d'écraser) Si l'éditeur écrase : `cao_doc_ecrire` passe dans « Ce que ce module ne fait pas »
      et la tâche 4 est réduite en conséquence
- [x] Si l'enregistrement change le projet ouvert : `antenna_projet_enregistrer` est limité
      au projet ouvert dans la spec et dans la tâche 6

**Vérification :**
- [ ] Relecture des passages cités, avec toi (au checkpoint)

**Dépendances :** aucune

**Fichiers :** `lanceur/docs/SPEC-mcp-api-outils.md`, `tasks/todo.md`

**Taille :** XS

## Tâche 2 : ✅ le socle, puis lire un projet CAO

**Description :**
- `appeler_outil_web()` reçoit `methode` (PUT possible, DELETE refusé, le lanceur en GET
  seulement).
- Créer `lanceur/mcp_api_outils.py`, avec `exiger()`, `resumer()` (100 Ko) et les trois
  premiers outils : `cao_projets`, `cao_projet_lire`, et `cao_doc_lire` avec `parties`.
- Le noyau ajoute `OUTILS_API` à `OUTILS_MCP`.
- Le banc reçoit `FauxApi`, un faux outil piloté par une table qui note les requêtes reçues.

**Critères d'acceptation :**
- [x] `tools/list` montre `etat` et les trois outils ; chaque outil atteint sa route en GET
- [x] Un argument manquant (`chemin`) donne un `isError` qui le nomme, sans requête ; un
      `DELETE` est refusé avant toute requête ; un PUT vers le lanceur est refusé
- [x] Un document de 300 Ko donne un résumé de moins de 100 Ko, et `parties` rend
      seulement les clés demandées

**Vérification :**
- [x] `python lanceur/banc_mcp.py` OK
- [x] `python lanceur/banc_serveur.py` et `python lanceur/appairage.py` toujours OK

**Dépendances :** tâche 1

**Fichiers :** `lanceur/mcp_web_suite.py`, `lanceur/mcp_api_outils.py`, `lanceur/banc_mcp.py`

**Taille :** M

## Checkpoint : socle
- [ ] Tous les bancs du lanceur verts
- [ ] Revue avec toi avant la phase 2

## Tâche 3 : ✅ lectures CAO, LIB, WEB_3D

**Description :** ajouter `cao_solveurs` (7 GET d'état en un appel, gabarits de l'œil
réduits à leurs noms), `cao_profils`, `cao_profil_lire`, `lib_fichiers`, `lib_fichier_lire`
(`type` contrôlé), `lib_parasites`, `lib_composants` (filtre, colonnes, `debut`/`nombre`,
50 par défaut) et `web3d_projets`.

**Critères d'acceptation :**
- [x] Chaque outil atteint sa ou ses routes en GET
- [x] `cao_solveurs` rend un résultat même si un des sept GET échoue (marqué en erreur pour
      ce solveur)
- [x] `lib_composants` filtre sans tenir compte de la casse et pagine ; un `type` LIB
      inconnu est refusé sans requête

**Vérification :**
- [x] `python lanceur/banc_mcp.py` OK

**Dépendances :** tâche 2

**Fichiers :** `lanceur/mcp_api_outils.py`, `lanceur/banc_mcp.py`

**Taille :** S

## Tâche 4 : écritures CAO gardées

**Description :** ajouter `cao_doc_ecrire` (PUT, `modifie` obligatoire, un 409 rendu en
« le fichier a changé, relis-le » ; seulement si la tâche 1 le garde), `cao_profil_ecrire`
(PUT, refusé si le profil existe) et `lib_fichier_ecrire` (POST, refusé si le fichier
existe).

**Critères d'acceptation :**
- [ ] Une écriture permise atteint sa route avec la bonne méthode et le bon corps
- [ ] Un `modifie` absent, un profil existant ou un fichier LIB existant donnent `isError`,
      et **aucune** écriture n'arrive au faux outil (au plus la lecture de contrôle)
- [ ] Un 409 du faux outil revient sous forme de message lisible

**Vérification :**
- [ ] `python lanceur/banc_mcp.py` OK

**Dépendances :** tâches 1 et 3

**Fichiers :** `lanceur/mcp_api_outils.py`, `lanceur/banc_mcp.py`

**Taille :** S

## Tâche 5 : pcbparts par le relais WEB_CAO

**Description :** ajouter `pcbparts_outils` (GET `/api/tools`) et `pcbparts_appeler` (POST
`/api/tool`, `name` contrôlé contre `allowed`).

**Critères d'acceptation :**
- [ ] Un outil de la liste `allowed` est appelé avec `{name, arguments}`
- [ ] Un outil hors liste est refusé, et le faux outil ne reçoit que le GET `/api/tools`

**Vérification :**
- [ ] `python lanceur/banc_mcp.py` OK

**Dépendances :** tâche 2

**Fichiers :** `lanceur/mcp_api_outils.py`, `lanceur/banc_mcp.py`

**Taille :** XS

## Checkpoint : WEB_CAO complet
- [ ] Bancs verts
- [ ] Revue avec toi

## Tâche 6 : projets WEB_ANTENNA

**Description :** ajouter `antenna_projets`, `antenna_projet_lire` (seulement le projet
ouvert, vérifié par `/api/projet`), `antenna_projet_enregistrer` (POST, toute valeur `null`
au premier niveau du corps refusée ; limité au projet ouvert ;
champs d'en-tête absents repris du projet ouvert, car le serveur les remet à vide) et
`antenna_calculs`.

**Critères d'acceptation :**
- [ ] Lire un projet qui n'est pas ouvert : refusé, et aucun GET `/api/projet/ouvrir` envoyé
- [ ] Un enregistrement qui contient `"carte": null`, ou qui vise un autre projet que
      l'ouvert, est refusé sans POST
- [ ] Un enregistrement partiel (seulement `antenne`) arrive avec `titre`, `dessin`, etc.
      repris du projet ouvert, et sans `carte` ni `resultats`
- [ ] `antenna_projets` et `antenna_calculs` atteignent leur route

**Vérification :**
- [ ] `python lanceur/banc_mcp.py` OK

**Dépendances :** tâches 1 et 2

**Fichiers :** `lanceur/mcp_api_outils.py`, `lanceur/banc_mcp.py`

**Taille :** S

## Tâche 7 : openEMS, chiffrer et suivre

**Description :** ajouter `antenna_openems_etat`, `antenna_openems_chiffrer` (`quoi` ∈
`modele` → `/api/openems`, `balayage`, `tableau_s` → `/api/openems/tableau-s`, `script`),
`antenna_openems_suivre` (`id`, `depuis`) et `antenna_openems_champs`.

**Critères d'acceptation :**
- [ ] Chaque valeur de `quoi` atteint la bonne route en POST ; une autre valeur (dont
      `lancer`) est refusée sans requête
- [ ] Aucune route en `…/lancer` ou `…/arreter` n'est jamais atteinte (le banc parcourt
      les requêtes notées)
- [ ] `suivre` passe `id` et `depuis` dans l'URL, encodés

**Vérification :**
- [ ] `python lanceur/banc_mcp.py` OK

**Dépendances :** tâche 2

**Fichiers :** `lanceur/mcp_api_outils.py`, `lanceur/banc_mcp.py`

**Taille :** S

## Tâche 8 : documentation, puis essai réel

**Description :** lister les outils MCP dans `lanceur/docs/DETAILS.md` (section « Claude
Code (MCP) »), en une ligne chacun, et dire ce qui reste exclu. Puis faire l'essai réel avec
les trois outils lancés depuis WEB·SUITE : lister les projets CAO, lire un PCB (le résumé,
puis une partie), chercher un composant dans la LIB et sur pcbparts, chiffrer un modèle
openEMS.

**Critères d'acceptation :**
- [ ] DETAILS.md à jour, ainsi que la carte `SPEC-mcp.md` si besoin
- [ ] L'essai réel donne des réponses justes ; aucune réponse ne dépasse 100 Ko

**Vérification :**
- [ ] CI verte après le push
- [ ] Essai réel avec toi, sur le Pi

**Dépendances :** tâches 1 à 7

**Fichiers :** `lanceur/docs/DETAILS.md`, `lanceur/docs/SPEC-mcp.md`

**Taille :** XS

## Checkpoint : terminé
- [ ] Critères de réussite de la spec remplis
- [ ] `/review` fait
