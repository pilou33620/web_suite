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

## Prérequis sur le poste

Claude Code installé et connecté une fois (`claude` dans un terminal). Sur un Raspberry Pi :
système 64 bits.

## Bancs

- `python3 lanceur/pilote.py` : routes, refus, appareil distant, page en HTTPS.
- `python3 lanceur/claude_pages.py` : faux `claude` qui écrit le flux stream-json — arguments
  (outils permis, MCP seul, consigne), messages, reprise, nouvelle conversation, échec, absence.
- `node lanceur/banc_pilote.js` : la zone de texte, la conversation affichée en texte brut,
  une coupure réseau retentée, un refus affiché.
- Essai réel le 22/10/2026 : page PCB de WEB_CAO dans Chromium, vrai Claude Code et vrai MCP
  (`cao_pcb_lire`), réponse dans le panneau en 8 à 10 s.
