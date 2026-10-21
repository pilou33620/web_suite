# Essai : le MCP depuis le PC Windows, par SSH vers le Pi

Où : PC Windows (PowerShell, Claude Code) et Pi (les outils tournent) · Outils : Claude Code
sous Windows, client OpenSSH, WEB·SUITE sur le Pi · Durée : 30 min (10 de plus la première fois,
pour la clé SSH) · Source : `A-FAIRE.md:16-17`, `tasks/todo-mcp-noyau.md:92`, `tasks/todo-savoir-metier.md:150`

## Pourquoi

Sous Windows, Claude Code ne tourne pas dans le dépôt web_suite : le skill `web-suite`
n'est pas chargé. Le serveur MCP donne alors deux choses à la place
(`lanceur/docs/SPEC-mcp-savoir-metier.md`, décision 2) :
- un résumé dans sa réponse `initialize` (`instructions`, `lanceur/mcp_web_suite.py:252-269`),
  qui renvoie à `savoir_lire` ;
- les fiches elles-mêmes, par l'outil `savoir_lire` (`mcp_web_suite.py:203-228`).

L'essai prouve trois choses :
- le transport SSH marche : `/mcp` montre `web_suite` connecté (`SPEC-mcp-noyau.md`, critère 2) ;
- `etat` et la page ouverte sont lus juste depuis le PC ;
- Claude va chercher le savoir par `savoir_lire`.

Déjà fait ailleurs : `claude -p` lancé depuis `/tmp` sur le Pi suit la consigne MCP
(`tasks/todo-savoir-metier.md:147-149`), et `claude -p` sous Linux lit la page WEB_ANTENNA
(`A-FAIRE.md:17-19`). Jamais essayé : la session depuis Windows, à travers SSH.

À faire de préférence après `ESSAI-pi-mcp-claude-code.md` : si le MCP marche sur le Pi, une
panne ici vient de SSH ou de Windows.

## Avant de commencer

1. Sur le Pi, mets à jour et lance les outils, comme dans `ESSAI-pi-mcp-claude-code.md`
   (« Avant de commencer », points 1 à 4) : WEB_CAO et WEB_ANTENNA lancés depuis WEB·SUITE,
   WEB_3D arrêté, la page WEB_ANTENNA ouverte avec **🧪 Patch**. L'essai ne fait que lire :
   pas de copie de PROJETS, et ne demande aucune écriture.
2. Sur le PC, dans PowerShell (`lanceur/docs/DETAILS.md:286-297`) :
   - `ssh -V` → une version d'OpenSSH. Sinon : Paramètres → Applications → Fonctionnalités
     facultatives → **Client OpenSSH** ;
   - clé SSH, une fois : `ssh-keygen -t ed25519`, puis
     `type $env:USERPROFILE\.ssh\id_ed25519.pub | ssh pilou@<pi> "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"` ;
   - `ssh -T -o BatchMode=yes pilou@<pi> echo ok` → `ok`, **sans mot de passe demandé**.
3. Sur le PC, vérifie que le skill n'est pas installé à côté :
   `dir $env:USERPROFILE\.claude\skills` ne doit pas contenir `web-suite`. S'il y est, Claude
   le chargerait et l'essai ne prouverait rien sur `savoir_lire` : renomme-le le temps de l'essai.
4. Déclare le MCP sur le PC (une fois) :
   ```
   claude mcp add -s user web_suite -- ssh -T -o BatchMode=yes pilou@<pi> python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py
   ```
   Contrôle : `claude mcp get web_suite` doit montrer `ssh` suivi de **tous** ces arguments
   (`-T`, `-o BatchMode=yes`, l'hôte, `python3`, le chemin).
5. Ouvre un dossier qui n'est pas web_suite : `mkdir $env:USERPROFILE\essai-mcp; cd $env:USERPROFILE\essai-mcp`.

## Étapes

1. `claude`, dans `essai-mcp`.
   → attendu : la session s'ouvre sans erreur MCP.
2. `/mcp`.
   → attendu : `web_suite` **connected**, 73 outils.
3. « Quels outils de WEB·SUITE tournent ? »
   → attendu : `etat` ; WEB·SUITE, WEB_CAO et WEB_ANTENNA lancés, WEB_3D arrêté, avec leurs
   ports (8100, 8000, 8732, 8139). Ce sont les outils **du Pi** : le MCP tourne là-bas.
4. « Que montre l'onglet WEB_ANTENNA ouvert ? »
   → attendu : `pilote_onglets` puis `antenna_lire` : patch 2,45 GHz sur FR-4.
5. « Dans une empreinte de la LIB, où est la broche 1 et dans quel sens tourne la
   numérotation ? »
   → attendu : un appel `savoir_lire` (fiche `SKILL`, puis `composants`) **avant** la réponse,
   visible dans la session comme un appel d'outil `web_suite`. Réponse : vue de dessus, y vers
   le bas, broche 1 en haut à gauche, sens antihoraire (`composants.md:47-49`). Pas d'appel au
   skill `web-suite` : il n'existe pas ici.
6. « Quelle largeur de piste pour 2 A sur une couche externe ? »
   → attendu : `savoir_lire` `pcb` ; Claude renvoie au calcul IPC-2152 (`cao_calcul`, analyse
   `dc`) plutôt que de donner un chiffre de tête.

## C'est réussi si

- [ ] `/mcp` montre `web_suite` connecté, par SSH, sans mot de passe demandé
- [ ] `etat` décrit juste les outils du Pi (deux lancés, un arrêté)
- [ ] La page WEB_ANTENNA est lue juste depuis le PC
- [ ] Claude passe par `savoir_lire` avant de répondre aux questions de métier, et ses réponses
      sont celles des fiches

## Si ça rate, note

- La sortie de `ssh -T -o BatchMode=yes pilou@<pi> echo ok`, telle quelle.
- La sortie de ce test du transport (une ligne JSON attendue, rien d'autre) :
  `'{"jsonrpc":"2.0","id":1,"method":"ping"}' | ssh -T -o BatchMode=yes pilou@<pi> python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py`.
  Un texte avant la ligne JSON vient d'un `.bashrc` du Pi qui écrit (un `echo`) : à déplacer
  dans `.profile` (`DETAILS.md:299-301`).
- `claude mcp get web_suite`. Si les arguments de `ssh` y sont tronqués (le `--` mangé par
  PowerShell quand `claude` est un script `.ps1`), refais l'ajout depuis `cmd.exe`, ou avec
  `claude.cmd mcp add …`, et note-le.
- `claude --version`, la ligne `/mcp` de `web_suite` (capture), et `claude --debug` si la
  connexion échoue.
- Pour une réponse de métier : la question, la réponse, et si `savoir_lire` a été appelé ou non.

## Après

En cas de réussite :
- `A-FAIRE.md:16-19` (Claude Code relancé avec le MCP) : cocher, si l'essai sur le Pi est fait
- `A-FAIRE.md:32-33` (mcp-noyau) : cocher, si l'essai sur le Pi est fait
- `tasks/todo-mcp-noyau.md:92` : cocher « Depuis le PC Windows par SSH : même résultat » ;
  `:95` (vérification à la main) et `:105` (checkpoint) si le Pi est fait aussi
- `tasks/plan-mcp-noyau.md:62` (tâche 4) et `:66` (checkpoint terminé)
- `tasks/todo-savoir-metier.md:150-151` : « L'essai Windows en SSH reste… » → « fait le <date> »
- `tasks/plan-savoir-metier.md:67` : même chose
- Une correction de commande trouvée en route va dans `lanceur/docs/DETAILS.md:280-303` et `README.md` (section « Claude Code (MCP) »)
