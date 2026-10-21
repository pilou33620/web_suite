# Essai : le MCP depuis un PC Windows, par SSH vers le Pi

Qui : toi · Où : un PC Windows de la maison (PowerShell, Claude Code) et le Pi (les outils
tournent) · Durée : 30 min (10 de plus la première fois, pour la clé SSH) ·
Source : `A-FAIRE.md:19-22`, `tasks/todo-mcp-noyau.md:92`, `tasks/todo-savoir-metier.md:150`

**Pas pour le PC du boulot** : il faut PowerShell, Claude Code et le Pi sur le même réseau.
À faire après `ESSAI-pi-mcp-claude-code.md` : si le MCP marche sur le Pi, une panne ici vient
de SSH ou de Windows.

## Pourquoi

Sous Windows, Claude Code ne tourne pas dans le dépôt : le skill `web-suite` n'est pas chargé.
Le serveur MCP le remplace (`lanceur/docs/SPEC-mcp-savoir-metier.md`, décision 2) par un
résumé dans sa réponse `initialize` (`lanceur/mcp_web_suite.py:252-269`) et par l'outil
`savoir_lire` (`mcp_web_suite.py:203-228`). L'essai prouve : le transport SSH (`/mcp` connecté,
`SPEC-mcp-noyau.md`, critère 2) ; `etat` et la page ouverte lus juste depuis le PC ; Claude
qui va chercher le savoir par `savoir_lire`. Jamais essayé : la session Windows à travers SSH.

## Ce que tu prépares

1. Sur le Pi : outils lancés comme dans `ESSAI-pi-mcp-claude-code.md` (« Ce que tu prépares »,
   points 1, 3 et 4) : WEB_CAO et WEB_ANTENNA lancés, WEB_3D arrêté, WEB_ANTENNA ouvert sur
   **🧪 Patch**. Lecture seule : pas de copie de PROJETS.
2. Sur le PC, PowerShell (`lanceur/docs/DETAILS.md:286-297`) : `ssh -V` (sinon Paramètres →
   Applications → Fonctionnalités facultatives → **Client OpenSSH**) ; clé, une fois :
   `ssh-keygen -t ed25519` puis
   `type $env:USERPROFILE\.ssh\id_ed25519.pub | ssh pilou@<pi> "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"` ;
   `ssh -T -o BatchMode=yes pilou@<pi> echo ok` → `ok`, **sans mot de passe**.
3. `dir $env:USERPROFILE\.claude\skills` ne doit pas contenir `web-suite` (sinon renomme-le le
   temps de l'essai : il masquerait `savoir_lire`).
4. Déclare le MCP, une fois :
   `claude mcp add -s user web_suite -- ssh -T -o BatchMode=yes pilou@<pi> python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py`
   ; `claude mcp get web_suite` montre `ssh` et **tous** ses arguments.
5. Un dossier qui n'est pas web_suite : `mkdir $env:USERPROFILE\essai-mcp; cd $env:USERPROFILE\essai-mcp`.

Claude in Chrome ne peut pas : l'essai est une session Claude Code et son MCP par SSH ; il
n'a ni terminal, ni Claude Code, ni MCP.

## C'est réussi si

- [ ] `/mcp` montre `web_suite` connecté, par SSH, sans mot de passe demandé
- [ ] `etat` décrit juste les outils du Pi (deux lancés, un arrêté)
- [ ] La page WEB_ANTENNA est lue juste depuis le PC
- [ ] Claude passe par `savoir_lire` avant les questions de métier, et répond comme les fiches

## Ce qui reste à toi

1. `claude` dans `essai-mcp` → pas d'erreur MCP.
2. `/mcp` → `web_suite` **connected**, 73 outils.
3. « Quels outils de WEB·SUITE tournent ? » → `etat` : WEB·SUITE, WEB_CAO, WEB_ANTENNA lancés,
   WEB_3D arrêté (8100, 8000, 8732, 8139) : ceux **du Pi**.
4. « Que montre l'onglet WEB_ANTENNA ouvert ? » → `pilote_onglets` puis `antenna_lire` : patch
   2,45 GHz sur FR-4.
5. « Dans une empreinte de la LIB, où est la broche 1 et dans quel sens tourne la
   numérotation ? » → `savoir_lire` (fiche `SKILL`, puis `composants`) **avant** la réponse ;
   vue de dessus, y vers le bas, broche 1 en haut à gauche, antihoraire (`composants.md:47-49`).
6. « Quelle largeur de piste pour 2 A sur une couche externe ? » → `savoir_lire` `pcb`, puis
   renvoi au calcul IPC-2152 (`cao_calcul`, analyse `dc`) plutôt qu'un chiffre de tête.

Si ça rate, note : la sortie de `ssh -T -o BatchMode=yes pilou@<pi> echo ok` ; celle de
`'{"jsonrpc":"2.0","id":1,"method":"ping"}' | ssh -T -o BatchMode=yes pilou@<pi> python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py`
(une seule ligne JSON ; un texte avant vient d'un `echo` du `.bashrc` du Pi, à passer dans
`.profile` : `DETAILS.md:299-301`) ; `claude mcp get web_suite` (arguments de `ssh` tronqués
= le `--` mangé par PowerShell : refais l'ajout depuis `cmd.exe` ou avec `claude.cmd mcp add …`) ;
`claude --version`, la ligne `/mcp` (capture), `claude --debug` ; pour une réponse de métier,
la question, la réponse, et si `savoir_lire` a été appelé.

## Après

Pour Claude Code, en cas de réussite :
- [ ] `A-FAIRE.md:19-22` (Claude Code relancé avec le MCP) et `:35-36` (mcp-noyau) : cocher, si l'essai Pi est fait
- [ ] `tasks/todo-mcp-noyau.md:92` : cocher « Depuis le PC Windows par SSH » ; `:95` et `:105` si le Pi est fait aussi
- [ ] `tasks/plan-mcp-noyau.md:62` (tâche 4) et `:66` (checkpoint)
- [ ] `tasks/todo-savoir-metier.md:150-151` et `tasks/plan-savoir-metier.md:67` : « L'essai Windows en SSH reste… » → « fait le <date> »
- [ ] Une commande corrigée en route : `lanceur/docs/DETAILS.md:280-303` et `README.md` (section « Claude Code (MCP) »)
