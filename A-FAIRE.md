# À faire — fonctionnalités et bugs

Ce qui reste à coder dans web_suite (lanceur et serveur MCP), au 22/10/2026 : nouvelles
fonctionnalités et bugs, rien d'autre. Les essais sont dans [TESTS-A-FAIRE.md](TESTS-A-FAIRE.md) ;
ce qui est fait, dans [HISTORIQUE.md](HISTORIQUE.md) et `tasks/` (un plan et une todo par
module). Chaque outil web a les deux mêmes fichiers dans son dépôt.

## Bugs

Aucun bug connu ouvert.

## Fonctionnalités

Rien n'est commencé : ce sont les limites que les specs des modules ont laissées en connaissance
de cause, à reprendre si le besoin vient. Chacune demande ton accord avant d'être commencée
(section « Demander d'abord » de sa spec).

- [ ] **Mode réseau chiffré (HTTPS)** : en mode réseau, commandes, captures et cookie du jeton
  circulent en clair ; quelqu'un qui écoute le Wi-Fi récupère le jeton et devient un appareil
  appairé. Touche `appairage.py` (copié dans chaque outil) et l'appairage de l'iPad
  ([SPEC-mcp-canal-pilote.md](lanceur/docs/SPEC-mcp-canal-pilote.md), « Limites connues »).
- [ ] **Paires différentielles proposées par `cao_chemin`** : aujourd'hui `cao_paire` prend ses
  points donnés à la main ([SPEC-mcp-chemin-auto.md](lanceur/docs/SPEC-mcp-chemin-auto.md)).
- [ ] **Classes et contraintes du schéma écrites par le MCP** (`setNetClassSch`, `schCmPoser`) :
  seules celles du PCB s'écrivent ([SPEC-mcp-ecriture-cao.md](lanceur/docs/SPEC-mcp-ecriture-cao.md)).
- [ ] **Fils du schéma posés par coordonnées**, et **symboles de la LIB (`schsym-1`) dessinés**
  par le MCP ([SPEC-mcp-dessin-schema.md](lanceur/docs/SPEC-mcp-dessin-schema.md)).
- [ ] **Modifier un composant existant de la LIB**, et écrire un fichier `schsym-1` : le MCP ne
  sait qu'ajouter ([SPEC-mcp-composants.md](lanceur/docs/SPEC-mcp-composants.md)).
