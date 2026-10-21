# Essai : Claude Code relancé avec le MCP, sur le Pi

Où : Pi (terminal du Pi, ou session SSH depuis le PC) · Outils : Claude Code, WEB·SUITE (WEB_CAO, WEB_ANTENNA lancés ; WEB_3D arrêté), un navigateur pour la page WEB_ANTENNA · Durée : 20 min · Source : `A-FAIRE.md:16` et `A-FAIRE.md:32`, `tasks/todo-mcp-noyau.md:91`

## Pourquoi

Le serveur MCP n'a jamais été branché dans une session **interactive** de Claude Code sur le
Pi : `/mcp` qui montre `web_suite` connecté, et une réponse juste à « quels outils
tournent ? » (critères de réussite de `lanceur/docs/SPEC-mcp-noyau.md`, l.129-131).

Déjà fait ailleurs :
- 18/10/2026, sur le Pi, par Claude : les 25 outils d'api-outils appelés en JSON-RPC sur une
  copie de PROJETS (`tasks/todo-api-outils.md:186-193`) ; reste « l'essai par Claude Code
  redémarré, avec toi » ;
- `claude -p` sur le Pi, dans le dépôt : le skill `web-suite` et ses fiches lus
  (`tasks/todo-savoir-metier.md:142-151`) ;
- 21/10/2026, sous Linux, outils sur le même poste : `claude -p` avec le MCP lit la page
  WEB_ANTENNA ouverte (`A-FAIRE.md:16-19`).

Ce qui reste, et que seul toi peux faire : la vraie session, avec le MCP déclaré par
`claude mcp add` et Claude Code relancé.

Le MCP se déclare **par la commande** `claude mcp add` : le dépôt n'a pas de `.mcp.json`
(`lanceur/docs/DETAILS.md:280-284`).

## Avant de commencer

1. Mets à jour les quatre dépôts. Démarrer WEB·SUITE le fait tout seul (étape 3) ; à la main :
   ```
   cd ~/Documents/web_suite
   git pull --ff-only
   for d in WEB_CAO WEB_ANTENNA WEB_3D; do git -C $d pull --ff-only; done
   git merge-base --is-ancestor 979b01a HEAD && echo "web_suite à jour"
   ```
   → attendu : `web_suite à jour`. Adapte le chemin si ton dépôt n'est pas dans
   `~/Documents/web_suite` (le chemin de `DETAILS.md`).
2. Vérifie que le MCP répond à la main :
   ```
   printf '{"jsonrpc":"2.0","id":1,"method":"ping"}\n' | python3 lanceur/mcp_web_suite.py
   ```
   → attendu : `{"jsonrpc": "2.0", "id": 1, "result": {}}`.
3. Lance WEB·SUITE : `./demarrer_WEB_SUITE.sh`. Le terminal donne une ligne par dépôt
   (`installé · À jour.` ou `Mis à jour (n commit(s)).`) et l'adresse **Réseau**.
   Dans la page (`http://<adresse du Pi>:8100/`), **▶ Lancer** sur WEB_CAO et sur WEB_ANTENNA.
   Laisse **WEB_3D arrêté**. Cet essai ne fait que lire : pas besoin de copie de PROJETS.
   Ne demande à Claude aucune écriture.
4. Ouvre WEB_ANTENNA dans un navigateur (Chromium du Pi, ou le PC à l'adresse Réseau,
   `http://<adresse du Pi>:8732/`), puis clique **🧪 Patch**. Garde cet onglet au premier plan.
5. Déclare le MCP, une fois (portée utilisateur) :
   ```
   claude mcp add -s user web_suite -- python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py
   ```
   S'il existe déjà, `claude mcp get web_suite` montre sa commande : elle doit être celle-ci.
   Note la version : `claude --version`.

## Étapes

1. `cd ~/Documents/web_suite` puis `claude` (une session neuve). Si Claude Code demande de
   faire confiance au dossier, accepte.
   → attendu : la session s'ouvre sans erreur MCP.
2. Tape `/mcp`.
   → attendu : `web_suite` **connected**. Ouvert, il liste **73** outils (le banc
   `lanceur/banc_mcp.py:735` en compte 73 ; `A-FAIRE.md:3` dit encore 71).
3. Demande : « Quels outils de WEB·SUITE tournent sur ce poste ? »
   → attendu : un appel à `etat` ; WEB·SUITE lancé (8100), WEB_CAO lancé (8000),
   WEB_ANTENNA lancé (8732), WEB_3D arrêté (8139). Si le lanceur a décalé un port parce
   que le sien était pris, la réponse donne le port décalé, celui de la carte de WEB·SUITE.
4. Demande : « Quels onglets sont ouverts, et que montre l'onglet WEB_ANTENNA ? »
   → attendu : `pilote_onglets` liste l'onglet `antenna`, avec l'appareil (`ce poste`, ou
   l'adresse du PC) ; puis `antenna_lire` : un patch 2,45 GHz sur FR-4. Rien ne change dans
   la page.
5. Demande : « Dans une empreinte de la LIB, où est la broche 1 et dans quel sens tourne la
   numérotation ? »
   → attendu : Claude charge le skill `web-suite` (la session affiche l'appel au skill, ou la
   lecture de `.claude/skills/web-suite/composants.md`) et répond : vue de dessus, y vers le
   bas, broche 1 en haut à gauche, sens **antihoraire** (`composants.md:47-49`).
6. Arrête WEB_ANTENNA (**■ Arrêter** sur sa carte), puis redemande « Quels outils tournent ? ».
   → attendu : WEB_ANTENNA arrêté. Une lecture de la page rend un message clair
   (« WEB_ANTENNA ne tourne pas : lance-le depuis WEB·SUITE »), pas une trace Python.

## C'est réussi si

- [ ] `/mcp` montre `web_suite` connecté
- [ ] La réponse à « quels outils tournent ? » est juste, avec deux outils lancés et un arrêté,
      puis après l'arrêt de WEB_ANTENNA
- [ ] La page WEB_ANTENNA ouverte est lue juste (patch 2,45 GHz), sans rien y changer
- [ ] Le skill `web-suite` est chargé dans le dépôt et la réponse sur la broche 1 est juste

## Si ça rate, note

- La version de Claude Code (`claude --version`) et la sortie de `claude mcp get web_suite`.
- La ligne de `/mcp` pour `web_suite` (état, message d'erreur), en capture.
- Le résultat de la commande `printf … | python3 lanceur/mcp_web_suite.py` de « Avant de
  commencer ».
- Les erreurs du serveur MCP : relance `claude --debug` et copie les lignes qui citent
  `web_suite` (Claude Code les range aussi sous `~/.cache/claude-cli-nodejs/…/mcp-logs-web_suite/`).
- Pour une réponse fausse : la question exacte, la réponse de Claude, et ce que montre la page
  WEB·SUITE (cartes, ports) au même moment.

## Après

En cas de réussite :
- `A-FAIRE.md:16-19` : écrire « fait sur le Pi le <date> » ; la case ne se coche qu'avec
  l'essai Windows (`ESSAI-windows-ssh-mcp.md`)
- `A-FAIRE.md:32-33` (mcp-noyau) : même chose, la partie Pi est faite
- `tasks/todo-mcp-noyau.md:91` : cocher « Sur le Pi : `/mcp` montre `web_suite` connecté »
- `tasks/todo-api-outils.md:192-193` : retirer « reste l'essai par Claude Code redémarré, avec toi »
- `A-FAIRE.md:3` : 71 → 73 outils, si `/mcp` en montre bien 73
