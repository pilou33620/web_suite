# Tests à faire

Les essais encore à faire pour web_suite (lanceur, serveur MCP, canal de pilotage), au
22/10/2026 : ce que les bancs automatiques (`lanceur/banc_*.py`, CI) ne peuvent pas vérifier
seuls. Chaque essai a sa spec dans [tasks/essais/](tasks/essais/README.md) : qui le fait,
comment, et la consigne à donner à Claude in Chrome. Le code à écrire est dans
[A-FAIRE.md](A-FAIRE.md).

## Claude Code avec le MCP

- [ ] **Sur le Pi** : Claude Code relancé, `/mcp` montre `web_suite` connecté et la réponse de
  Claude est juste (fait le 21/10/2026 sous Linux, outils sur le même poste) —
  [ESSAI-pi-mcp-claude-code.md](tasks/essais/ESSAI-pi-mcp-claude-code.md)
- [ ] **Depuis le PC Windows par SSH** : même résultat ; le skill `web-suite` n'y est pas chargé,
  Claude passe par `savoir_lire` — [ESSAI-windows-ssh-mcp.md](tasks/essais/ESSAI-windows-ssh-mcp.md)

## Canal de pilotage

- [ ] **Depuis l'iPad** : un appareil appairé (cartes, interrupteur, capture) —
  [ESSAI-ipad-canal-pilotage.md](tasks/essais/ESSAI-ipad-canal-pilotage.md)

## Les tests des outils web

Chaque outil a son propre `TESTS-A-FAIRE.md` :
[WEB_CAO](https://github.com/pilou33620/WEB_CAO/blob/main/TESTS-A-FAIRE.md),
[WEB_ANTENNA](https://github.com/pilou33620/WEB_ANTENNA/blob/main/TESTS-A-FAIRE.md),
[WEB_3D](https://github.com/pilou33620/WEB_3D/blob/main/TESTS-A-FAIRE.md).
L'index de tous les essais, rangé par qui les fait, est [tasks/essais/README.md](tasks/essais/README.md).
