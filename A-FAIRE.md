# À faire — fonctionnalités et bugs

Ce qui reste à coder dans web_suite (lanceur et serveur MCP), au 22/10/2026 : nouvelles
fonctionnalités et bugs, rien d'autre. Les essais sont repoussés à plus tard ;
ce qui est fait, dans [HISTORIQUE.md](HISTORIQUE.md) et `tasks/` (un plan et une todo par
module). Chaque outil web a les deux mêmes fichiers dans son dépôt.

## Bugs

Aucun bug connu ouvert.

## Fonctionnalités

Décidées le 22/10/2026 : les quatre sont à coder. Chacune touche une partie que sa spec met en
« Demander d'abord » : ton choix vaut cet accord.

- [ ] **Mode réseau chiffré (HTTPS)** : en mode réseau, commandes, captures et cookie du jeton
  circulent en clair ; quelqu'un qui écoute le Wi-Fi récupère le jeton et devient un appareil
  appairé. Touche `appairage.py` (copié dans chaque outil) et l'appairage de l'iPad, à refaire
  une fois ([SPEC-mcp-canal-pilote.md](lanceur/docs/SPEC-mcp-canal-pilote.md), « Limites connues »).
  Le plus gros chantier des quatre.
- [x] **Paires différentielles proposées par `cao_chemin`** (22/10/2026) : l'axe de la paire,
  aussi large que ses deux pistes et leur écart, rendu pour `cao_paire` ; une seule couche
  ([SPEC-mcp-chemin-auto.md](lanceur/docs/SPEC-mcp-chemin-auto.md)).
- [ ] **Schéma piloté plus loin** : faits le 22/10/2026, les classes et contraintes du schéma
  (`page: "schema"`) et les fils par coordonnées (`fil`). Reste : dessiner les symboles de la
  LIB (`schsym-1`)
  ([SPEC-mcp-ecriture-cao.md](lanceur/docs/SPEC-mcp-ecriture-cao.md),
  [SPEC-mcp-dessin-schema.md](lanceur/docs/SPEC-mcp-dessin-schema.md)).
- [x] **Modifier un composant existant de la LIB** (22/10/2026) : `lib_composant_modifier`,
  carte toujours cliquée, avant / après par case, refus si la ligne a changé depuis sa lecture
  ([SPEC-mcp-composants.md](lanceur/docs/SPEC-mcp-composants.md)). Le fichier `schsym-1` va
  avec les symboles de la LIB dessinés, ci-dessus.
