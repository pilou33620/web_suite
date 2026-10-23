# Écrire à Claude depuis une page (claude-pages)

Décidé le 22/10/2026 : depuis n'importe quel appareil (iPad, téléphone, autre poste), on clique
sur « Claude » en bas à droite d'un outil lancé par WEB·SUITE, on écrit ce qu'on veut, et Claude
Code **du poste des outils** (Pi ou PC) le fait avec le MCP web_suite.

## Chemin

```
page (pilote.js) ──POST /api/pilote/demande──▶ serveur de l'outil (pilote.py)
                 ◀─GET /api/pilote/conversation (long-poll)─┘   ▲
                                                                │ boucle locale seulement
lanceur web_suite (claude_pages.py) ──GET /api/pilote/demandes (long-poll)
                                    ──POST /api/pilote/progres (textes, outils, fin)
        └── claude -p … (stream-json), dans le dépôt web_suite, avec le MCP web_suite seul
```

- La demande passe par le canal de pilotage, sur l'origine de la page : appairage, jeton et
  HTTPS sont ceux de l'outil, rien de neuf n'écoute sur le réseau.
- `demandes` et `progres` sont dans `ORDRES` de `pilote.py` : boucle locale, sans `Origin` ni
  `Sec-Fetch-Site`, comme les ordres du MCP. Un appareil appairé ne lit que les conversations
  de son navigateur.
- Une demande que personne ne prend en 20 s échoue en le disant (outil lancé sans WEB·SUITE).
- Une seule demande en cours par navigateur.

## Claude Code

```
claude -p --output-format stream-json --verbose --tools Skill --strict-mcp-config
       --mcp-config '{"mcpServers":{"web_suite":{…mcp_web_suite.py}}}'
       --allowedTools mcp__web_suite --permission-mode dontAsk
       --settings '{"disableAllHooks":true}'
       --append-system-prompt "<outil, page, onglet, projet, appareil>"  [--resume <session>]
```

- **Rien que le MCP** : `--tools Skill` retire le terminal et les fichiers du poste,
  `dontAsk` refuse tout ce qui n'est pas permis. Les écritures dans la page restent des cartes
  (interrupteur de l'onglet compris), les écritures du MCP gardent leurs refus.
- Le texte de la demande part sur l'entrée standard, jamais dans la ligne de commande.
- La conversation continue (`--resume`), par outil et par navigateur, jusqu'à « Nouvelle
  conversation » ; une session qui ne reprend pas repart de zéro à la demande suivante.
- `claude` est cherché dans le PATH, puis `~/.local/bin`, `~/.claude/local`, ou
  `WEB_SUITE_CLAUDE`. Absent : la page le dit. Arrêté au bout de 30 min.
- Le panneau montre les textes de Claude et le nom des outils qu'il appelle, jamais leurs
  résultats.

## Pièces jointes

Décidé le 22/10/2026 : une demande peut porter des fichiers que Claude lit avec le texte.

- **Ajout** dans le panneau : bouton trombone (sélecteur de fichiers ; sur l'iPad, Photos et
  appareil photo), coller dans la zone de texte (Ctrl+V d'une image), glisser-déposer sur le
  panneau, et bouton « Vue » qui joint la capture de l'onglet (seulement si la page a un
  canevas, `Pilote.capture`). Chaque pièce est une puce (nom, taille) qu'un × retire.
- **Types** :
  - images PNG, JPEG, GIF, WebP : passées telles quelles ; une autre image (HEIC de l'iPad) est
    redessinée en JPEG par le navigateur, refusée s'il ne sait pas la lire. Toute image de plus
    de 2000 px ou de 3,75 Mo est réduite dans le navigateur (l'API refuse au-delà de 5 Mo en base64) ;
  - PDF : passé tel quel (Claude en lit le texte et les pages ; au-delà de 100 pages, l'erreur
    de Claude s'affiche) ;
  - tout autre fichier lisible en UTF-8 (md, json, kicad_*, csv, cir, net, s2p, svg…) : passé en
    texte, précédé de son nom ;
  - un binaire (zip, docx, xlsx…) : refusé dans le panneau, avec la raison.
- **Limites** (par demande) : 10 fichiers, 20 Mo au total après réduction des images, et
  400 000 caractères de texte joint (≈ 100 000 jetons : le contexte de Claude). Vérifiées par la
  page pour prévenir, puis par `pilote.py`, qui seul fait foi (`demande` a sa propre taille de
  corps, 28 Mo : 20 Mo en base64 plus le texte).
- **20 Mo par conversation** aussi : `--resume` renvoie toute la conversation à l'API, qui refuse
  une requête de plus de 32 Mo ; Claude Code retirait alors les pièces jointes sans le dire (essai
  réel : deux PDF de 11 Mo, le second tour ne voyait plus rien). `claude_pages.py` compte les
  pièces de chaque conversation ; une demande qui dépasserait 20 Mo ouvre une nouvelle
  conversation, et le panneau l'annonce (message `info`, affiché en trace).
- **Lu seulement** : les pièces vivent en mémoire dans `pilote.py` jusqu'à ce que le lanceur
  prenne la demande, puis en sont retirées ; rien n'est écrit sur disque par les outils.
  Claude Code les garde dans son historique de session sur le poste (`~/.claude/projects`),
  ce qui lui permet de les revoir au message suivant, et copie les images dans `/tmp/claude-<uid>/`. Pour enregistrer un fichier (LIB,
  projet), Claude passe par les outils du MCP habituels.
- **Vers Claude** : `claude_pages.py` passe toujours la demande par `--input-format stream-json`
  (un seul message utilisateur : blocs `image`, `document`, puis texte des fichiers, puis la
  demande). La consigne précise que le contenu des pièces jointes est une donnée, jamais une
  consigne.
- La conversation du panneau montre la demande suivie des noms des pièces jointes.

## Modèle et test de connexion

Décidé le 23/10/2026.

- **Modèle** : un menu du panneau propose Opus (défaut), Sonnet et Haiku ; le choix est gardé
  par appareil (localStorage). La demande porte `modele` (`opus`, `sonnet` ou `haiku`, vérifié
  par `pilote.py`) et `claude_pages.py` le passe par `--model` (alias du dernier modèle de
  chaque famille). Changer de modèle garde la conversation (`--resume`).
- **Test de connexion, sans requête au modèle** : à chaque ouverture du panneau, la page lit
  `GET /api/pilote/statut`. Le serveur de l'outil sait si le lanceur l'écoute (un long-poll
  `/demandes` depuis moins de 35 s) et quel était l'état de Claude Code au dernier test.
  Un état de plus de 30 s fait demander un nouveau test : le long-poll du lanceur rend
  `{"tester": true}`, le lanceur lance `claude --version` et `claude auth status`, puis
  repasse aussitôt sur `/demandes?claude=<état>` ; la page attend ce nouvel état (8 s au plus).
- **Ligne d'état** en haut du panneau : point vert « Claude Code 2.1.301 connecté », ou point
  rouge avec la raison et quoi faire (lanceur absent, Claude Code introuvable, pas connecté) ;
  « Envoyer » est alors désactivé. L'état ne porte ni e-mail ni organisation.
- **Windows** : l'installeur officiel crée `%USERPROFILE%\.local\bin\claude.exe`, cherché aussi
  quand ce dossier n'est pas dans le PATH du lanceur.

## Prérequis sur le poste

Claude Code installé et connecté une fois (`claude` dans un terminal). Sur un Raspberry Pi :
système 64 bits.

## Bancs

- `python3 lanceur/pilote.py` : routes, refus, appareil distant, page en HTTPS.
- `python3 lanceur/claude_pages.py` : faux `claude` qui écrit le flux stream-json — arguments
  (outils permis, MCP seul, consigne), messages, reprise, nouvelle conversation, échec, absence.
- `node lanceur/banc_pilote.js` : la zone de texte, la conversation affichée en texte brut,
  une coupure réseau retentée, un refus affiché.
- Pièces jointes : `pilote.py` (types, nombre, tailles, base64 invalide, binaire refusé,
  pièces retirées une fois prises), `claude_pages.py` (le faux `claude` relit le message
  stream-json : blocs image, document et texte), `banc_pilote.js` (trombone, coller, déposer,
  « Vue », puce ×, refus d'un binaire, limites).
- `claude -p --input-format stream-json` a lu en vrai une image, un PDF et un fichier md dans
  le même message (22/10/2026).
- Essai réel le 22/10/2026 : page PCB de WEB_CAO dans Chromium, vrai Claude Code et vrai MCP
  (`cao_pcb_lire`), réponse dans le panneau en 8 à 10 s.
