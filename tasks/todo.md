# Tâches : mcp-noyau

Plan : [plan.md](plan.md) · Spec : [lanceur/docs/SPEC-mcp-noyau.md](../lanceur/docs/SPEC-mcp-noyau.md)

## Tâche 1 : ✅ le MCP répond au protocole

**Description :** `lanceur/mcp_web_suite.py` lit des messages JSON-RPC 2.0 sur stdin (une
ligne par message) et répond sur stdout. Il traite `initialize` (version du client si
connue, sinon `2025-06-18` ; `serverInfo` ; capacité `tools`), `notifications/initialized`
(sans réponse), `ping`, `tools/list` (à partir de `OUTILS_MCP`) et `tools/call` (outil
inconnu : résultat `isError`). Le banc `lanceur/banc_mcp.py` lance le script en
sous-processus et vérifie tout ça.

**Critères d'acceptation :**
- [x] `initialize` puis `tools/list` répondent avec le bon `id` ; une notification ne reçoit
      aucune réponse
- [x] Une ligne illisible donne `-32700`, une méthode inconnue `-32601`, et le serveur
      répond encore au message suivant
- [x] stdout ne contient que des lignes JSON-RPC (journaux sur stderr) ; fin de stdin =
      sortie propre, code 0

**Vérification :**
- [x] `python lanceur/banc_mcp.py` affiche OK
- [x] À la main : `printf '{"jsonrpc":"2.0","id":1,"method":"ping"}\n' | python3 lanceur/mcp_web_suite.py`

**Dépendances :** aucune

**Fichiers :** `lanceur/mcp_web_suite.py`, `lanceur/banc_mcp.py`

**Taille :** S

## Tâche 2 : ✅ l'outil `etat` et la porte `appeler_outil_web()`

**Description :** ajouter `appeler_outil_web(id_outil, route, corps=None)` : boucle locale
seulement, port lu dans `outils.CATALOGUE` ou `web_suite.PORT_DEFAUT` (remplaçables par
`WEB_SUITE_MCP_PORTS` pour le banc), refus de `/api/github` et `/api/ia` **avant** toute
requête, erreur lisible si l'outil ne tourne pas. Ajouter l'outil MCP `etat` (lanceur,
WEB_CAO, WEB_ANTENNA, WEB_3D : lancé, port, adresse). Le banc démarre un faux outil HTTP qui
compte les requêtes reçues.

**Critères d'acceptation :**
- [x] `etat` montre le faux outil lancé et les trois autres arrêtés, avec leurs ports
- [x] `/api/github/envoyer` et `/api/ia/cle` sont refusés et le faux outil reçoit **zéro**
      requête ; une route permise l'atteint bien
- [x] Un outil arrêté donne un message clair (« WEB_CAO ne tourne pas : lance-le depuis
      WEB·SUITE »), pas de trace Python

**Vérification :**
- [x] `python lanceur/banc_mcp.py` affiche OK
- [x] `python lanceur/banc_serveur.py` et `python lanceur/appairage.py` toujours OK

**Dépendances :** tâche 1

**Fichiers :** `lanceur/mcp_web_suite.py`, `lanceur/banc_mcp.py`

**Taille :** S

## Checkpoint : socle
- [ ] Tous les bancs du lanceur verts
- [ ] Revue avec toi avant la phase 2

## Tâche 3 : CI et documentation

**Description :** ajouter l'étape `python lanceur/banc_mcp.py` à `.github/workflows/ci.yml`.
Écrire la section « Claude Code (MCP) » : commandes `claude mcp add` pour le poste (Linux,
Pi), pour Windows en local (`py -3`) et pour Windows vers le Pi par SSH (`ssh -T`, clé SSH
obligatoire, vérifier qu'OpenSSH client est installé). Mettre la carte `SPEC-mcp.md` à jour.

**Critères d'acceptation :**
- [ ] La CI exécute le banc MCP
- [ ] `README.md` donne la commande courte et renvoie à `lanceur/docs/DETAILS.md` pour le
      détail (SSH, clé, Windows)

**Vérification :**
- [ ] CI verte après le push
- [ ] Relecture des commandes copiées-collées telles quelles

**Dépendances :** tâche 2

**Fichiers :** `.github/workflows/ci.yml`, `README.md`, `lanceur/docs/DETAILS.md`,
`lanceur/docs/SPEC-mcp.md`

**Taille :** S

## Tâche 4 : essai réel avec Claude Code

**Description :** brancher le MCP pour de vrai et demander à Claude « quels outils
tournent ? », avec un outil arrêté et deux lancés.

**Critères d'acceptation :**
- [ ] Sur le Pi : `/mcp` montre `web_suite` connecté ; la réponse de Claude est juste
- [ ] Depuis le PC Windows par SSH : même résultat

**Vérification :**
- [ ] À la main, par toi (la partie Windows ne peut pas être faite d'ici)

**Dépendances :** tâche 3

**Fichiers :** aucun (corrections éventuelles dans la doc)

**Taille :** XS

## Checkpoint : terminé
- [ ] Critères de réussite de la spec remplis
- [ ] `/review` fait
