# Essai : Claude Code relancé avec le MCP, sur le Pi

Qui : toi · Où : Pi, à la maison (terminal du Pi, ou SSH depuis un PC de la maison) · Durée : 20 min ·
Source : `A-FAIRE.md:19-22`, `A-FAIRE.md:35-36`, `tasks/todo-mcp-noyau.md:91`

**Pas pour le PC du boulot** : l'essai porte sur Claude Code et le serveur MCP, dans un
terminal. Gardé pour le Pi ; à faire en premier des essais MCP (les autres s'en servent).

## Pourquoi

Le serveur MCP n'a jamais été branché dans une session **interactive** de Claude Code sur le
Pi : `/mcp` qui montre `web_suite` connecté, et une réponse juste à « quels outils
tournent ? » (critères de `lanceur/docs/SPEC-mcp-noyau.md`, l.129-131).

Déjà fait : les 25 outils d'api-outils appelés en JSON-RPC sur le Pi
(`tasks/todo-api-outils.md:186-193`) ; `claude -p` dans le dépôt lit le skill `web-suite`
(`tasks/todo-savoir-metier.md:142-151`) ; `claude -p` avec le MCP lit la page WEB_ANTENNA
sous Linux (`A-FAIRE.md:19-22`). Reste la vraie session, MCP déclaré par `claude mcp add`
(le dépôt n'a pas de `.mcp.json` : `lanceur/docs/DETAILS.md:280-284`).

## Ce que tu prépares

1. Démarre WEB·SUITE sur le Pi : `cd ~/Documents/web_suite && ./demarrer_WEB_SUITE.sh`. Il met
   les quatre dépôts à jour (une ligne par dépôt : `installé · À jour.` ou `Mis à jour (n
   commit(s)).`) et donne l'adresse **Réseau**. Contrôle :
   `git merge-base --is-ancestor 979b01a HEAD && echo "web_suite à jour"`.
2. Le MCP répond à la main :
   `printf '{"jsonrpc":"2.0","id":1,"method":"ping"}\n' | python3 lanceur/mcp_web_suite.py`
   → `{"jsonrpc": "2.0", "id": 1, "result": {}}`.
3. Dans la page WEB·SUITE, **▶ Lancer** sur WEB_CAO et sur WEB_ANTENNA ; laisse **WEB_3D
   arrêté**. L'essai ne fait que lire : pas de copie de PROJETS, aucune écriture demandée.
4. Ouvre WEB_ANTENNA dans un navigateur (`http://<adresse du Pi>:8732/`), clique **🧪 Patch**,
   garde l'onglet au premier plan.
5. Déclare le MCP, une fois :
   `claude mcp add -s user web_suite -- python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py`
   (s'il existe, `claude mcp get web_suite` doit montrer cette commande). Note `claude --version`.

Claude in Chrome ne peut pas : il n'a ni terminal, ni Claude Code, ni MCP ; tout l'essai est là.

## C'est réussi si

- [ ] `/mcp` montre `web_suite` connecté
- [ ] « Quels outils tournent ? » est juste (deux lancés, un arrêté), puis après l'arrêt de WEB_ANTENNA
- [ ] La page WEB_ANTENNA ouverte est lue juste (patch 2,45 GHz), sans rien y changer
- [ ] Le skill `web-suite` est chargé et la réponse sur la broche 1 est juste

## Ce qui reste à toi

1. `cd ~/Documents/web_suite` puis `claude` (session neuve ; accepte la confiance au dossier).
   → attendu : pas d'erreur MCP.
2. `/mcp` → `web_suite` **connected**, **73** outils (`lanceur/banc_mcp.py:735` ; `A-FAIRE.md:3` dit 73).
3. « Quels outils de WEB·SUITE tournent sur ce poste ? » → `etat` : WEB·SUITE (8100), WEB_CAO
   (8000) et WEB_ANTENNA (8732) lancés, WEB_3D (8139) arrêté (ou le port décalé de sa carte).
4. « Quels onglets sont ouverts, et que montre l'onglet WEB_ANTENNA ? » → `pilote_onglets`
   (onglet `antenna`, appareil `ce poste` ou l'adresse du PC), puis `antenna_lire` : patch
   2,45 GHz sur FR-4. Rien ne change dans la page.
5. « Dans une empreinte de la LIB, où est la broche 1 et dans quel sens tourne la
   numérotation ? » → le skill `web-suite` chargé (appel au skill, ou lecture de
   `.claude/skills/web-suite/composants.md`) ; vue de dessus, y vers le bas, broche 1 en haut
   à gauche, sens **antihoraire** (`composants.md:47-49`).
6. **■ Arrêter** sur WEB_ANTENNA, puis « Quels outils tournent ? » → WEB_ANTENNA arrêté ; une
   lecture de la page rend « WEB_ANTENNA ne tourne pas : lance-le depuis WEB·SUITE », pas une
   trace Python.

Si ça rate, note : `claude --version`, `claude mcp get web_suite`, la ligne `/mcp` de
`web_suite` (capture), la sortie du `ping` du point 2, les lignes de `claude --debug` qui
citent `web_suite` (aussi sous `~/.cache/claude-cli-nodejs/…/mcp-logs-web_suite/`) ; pour une
réponse fausse, la question, la réponse et les cartes de WEB·SUITE au même moment.

## Après

Pour Claude Code, en cas de réussite :
- [ ] `A-FAIRE.md:19-22` : « fait sur le Pi le <date> » (la case attend aussi `ESSAI-windows-ssh-mcp.md`)
- [ ] `A-FAIRE.md:35-36` (mcp-noyau) : même chose
- [ ] `tasks/todo-mcp-noyau.md:91` : cocher « Sur le Pi : `/mcp` montre `web_suite` connecté »
- [ ] `tasks/todo-api-outils.md:192-193` : retirer « reste l'essai par Claude Code redémarré, avec toi »
- [ ] `A-FAIRE.md:3` : le nombre d'outils, si `/mcp` en montre un autre que 73
