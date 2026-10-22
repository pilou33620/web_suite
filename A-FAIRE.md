# À faire — fonctionnalités et bugs

Ce qui reste à coder dans web_suite (lanceur et serveur MCP), au 22/10/2026 : nouvelles
fonctionnalités et bugs, rien d'autre. Les essais sont repoussés à plus tard ;
ce qui est fait, dans [HISTORIQUE.md](HISTORIQUE.md) et `tasks/` (un plan et une todo par
module). Chaque outil web a les deux mêmes fichiers dans son dépôt.

## Bugs

Aucun bug connu ouvert. Corrigé le 22/10/2026 : `pilote.py` refusait (403) les pages servies en
HTTPS, donc toute page d'un autre appareil en mode réseau chiffré ; il accepte maintenant
`https://<hôte>` comme `http://<hôte>`.

## Fonctionnalités

Décidées le 22/10/2026 : les quatre sont à coder, et sont faites. Chacune touche une partie que sa spec met en
« Demander d'abord » : ton choix vaut cet accord.

- [x] **Mode réseau chiffré (HTTPS)** (22/10/2026) : `appairage.py` (copie identique dans
  chaque outil) crée une fois une CA propre au poste (`~/.web_tools/tls/`, commande `openssl`,
  limitée aux adresses privées et noms locaux) et sert HTTPS sur le port de chaque outil et du
  lanceur ; la boucle locale garde HTTP, un autre appareil en HTTP est renvoyé vers https://, le
  cookie du jeton devient `Secure`. Chaque appareil installe la CA une fois
  (`/appairage/ca.pem`, lien sur la page du code) ; le cookie déjà posé reste valable
  ([SPEC-mcp-canal-pilote.md](lanceur/docs/SPEC-mcp-canal-pilote.md), « Limites connues »).
- [x] **Paires différentielles proposées par `cao_chemin`** (22/10/2026) : l'axe de la paire,
  aussi large que ses deux pistes et leur écart, rendu pour `cao_paire` ;
  sur plusieurs couches depuis le même jour
  (ci-dessous, [SPEC-mcp-chemin-auto.md](lanceur/docs/SPEC-mcp-chemin-auto.md)).
- [x] **Schéma piloté plus loin** (22/10/2026) : les classes et contraintes du schéma
  (`page: "schema"`), les fils par coordonnées (`fil`), et les symboles de la LIB (`schsym-1`)
  dessinés par le schéma et écrits par `lib_composant_creer` / `lib_composant_modifier`
  ([SPEC-mcp-ecriture-cao.md](lanceur/docs/SPEC-mcp-ecriture-cao.md),
  [SPEC-mcp-dessin-schema.md](lanceur/docs/SPEC-mcp-dessin-schema.md)).
- [x] **Modifier un composant existant de la LIB** (22/10/2026) : `lib_composant_modifier`,
  carte toujours cliquée, avant / après par case, refus si la ligne a changé depuis sa lecture
  ([SPEC-mcp-composants.md](lanceur/docs/SPEC-mcp-composants.md)). Le fichier `schsym-1` va
  avec les symboles de la LIB dessinés, ci-dessus.

- [x] **Écrire à Claude depuis la page** (22/10/2026) : zone « Demander à Claude » dans le
  panneau de chaque outil (`pilote.js`), demandes gardées par `pilote.py` et passées par le
  lanceur à Claude Code du poste (`claude_pages.py`), avec le MCP web_suite seul ; conversation
  continue, réponses dans le panneau, écritures en cartes
  ([SPEC-claude-pages.md](lanceur/docs/SPEC-claude-pages.md)).
- [x] **Pièces jointes dans le panneau « Claude »** (22/10/2026) : images, captures, PDF, fichiers
  texte, par trombone, coller, glisser-déposer et « Vue » ; lues par Claude, rien d'écrit sur
  disque par les outils.

### Limites connues

Relevées le 22/10/2026 sur les fonctionnalités ci-dessus ; décidées et faites le même jour.

- [x] **Paires différentielles de `cao_chemin` sur plusieurs couches** (22/10/2026) : l'axe
  change de couche par une via de paire (deux vias côte à côte au pas de la paire), que
  `cao_paire` pose ; pas de via de paire contre les pastilles des bouts
  ([SPEC-mcp-chemin-auto.md](lanceur/docs/SPEC-mcp-chemin-auto.md)).
- [x] **HTTPS en IPv6** (22/10/2026) : le certificat et les contraintes de la CA couvrent
  Tailscale `fd7a:…` et les ULA `fc00::/7` (pas le lien local `fe80::`) ; le lanceur et WEB_3D
  écoutent en double pile. La CA est refaite une fois : chaque appareil la réinstalle
  ([SPEC-mcp-canal-pilote.md](lanceur/docs/SPEC-mcp-canal-pilote.md), « Limites connues »).
