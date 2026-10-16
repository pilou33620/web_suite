# Carte des modules : MCP web_suite

Un serveur MCP unique qui permet à Claude Code de commander WEB_CAO, WEB_ANTENNA et
WEB_3D : calculs, projets, pilotage en direct des éditeurs, génération de composants,
routage. Carte validée le 2026-10-16. Chaque module a sa spec : `SPEC-mcp-<id>.md`
dans ce dossier. Les ids ne changent plus.

## Décisions de départ

- MCP **stdio**, Python 3.8+ **bibliothèque standard seulement** (pas de SDK `mcp`), dans
  `lanceur/mcp_web_suite.py`. À distance : le même MCP lancé **par SSH**, aucun port ouvert.
- Les outils doivent **déjà tourner** ; le MCP ne les démarre pas.
- Écriture dans un éditeur ouvert : **proposer** (carte d'action) par défaut, un
  interrupteur dans la page autorise l'application directe ; une commande = une étape
  d'undo.
- Claude **remplace** l'assistant Gemini/Gemma ; ses registres d'outils et d'actions
  servent de base.
- Fichiers communs aux outils recopiés dans chaque dépôt, copies identiques vérifiées
  par la CI (même règle que `appairage.py`).
- Jamais exposés : `/api/github/*`, `/api/ia/*`, `/api/envoyer` et `/api/identite` (le lanceur
  publie sur GitHub par là), `/api/appairer` ; le lanceur est en lecture seule ; aucune suppression sans
  confirmation.

## Modules

| Id du module | Rôle | Dépend de |
|---|---|---|
| `mcp-noyau` | Serveur MCP stdio : protocole, découverte des outils lancés, appels en boucle locale, routes interdites, lancement par SSH | — |
| `api-outils` | Couche 1 : les API HTTP existantes en outils MCP (SI/PI/RF, œil, crosstalk, openEMS asynchrone, projets, LIB, pcbparts) | `mcp-noyau` |
| `canal-pilote` | Serveur ↔ page en long-poll, onglets et onglet actif, proposer / appliquer, undo, captures du canevas | `mcp-noyau` |
| `registre-3d` | Commandes WEB_3D : ouvrir, arbre, mesure, coupe, collisions, déplacement, export | `canal-pilote` |
| `registre-cao` | Commandes WEB_CAO (schéma, PCB, IPC-2581, simulations) : `OUTILS_PCB/IPC/SIM` repris, écriture ajoutée | `canal-pilote` |
| `registre-antenna` | Commandes WEB_ANTENNA : lecture et actions `reglages/cotes/balayage/formes` reprises | `canal-pilote` |
| `composants` | Symbole, empreinte, brochage, ligne de catalogue (39 colonnes), validés par le code, écrits dans la LIB | `api-outils`, `registre-cao` |
| `routage` | Le routeur PNS piloté sans souris : placer, router, paire, via ; boucle DRC et score | `registre-cao` |
| `savoir-metier` | Le savoir des consignes Gemini déplacé dans un skill `web_suite` et dans les descriptions d'outils | `registre-cao`, `registre-antenna`, `composants` |
| `retrait-gemini` | Retrait de l'assistant Gemini/Gemma et de `/api/ia/cle` | `savoir-metier`, `routage` |

## Ordre de construction

0. **Essai sans souris du PNS** (première tâche de `routage`, faite en tête : elle décide
   si le routage par Claude est possible tel quel).
   *Étude du 2026-10-16 : pilotable avec un adaptateur mince (~80-150 lignes). Le banc
   `WEB_CAO/editeur-pcb/test/harness.js` route déjà sans souris (`startRoute(x,y,true)`,
   `routeToPoint`, `stepRoute`, `commitRoute`, `dpStart…dpCommit`, `placeVia`, `runDrc`) ;
   une route = une étape d'undo. Limites : routeur local seulement (pas de recherche de
   chemin globale, il faut fournir des points intermédiaires) ; séquence synchrone
   obligatoire ; fixer `S.avoid`, `S.rule.route`, `S.rule.corner` ; la paire différentielle
   accroche à la grille (pas de point exact).*
1. `mcp-noyau`
2. `api-outils`, `canal-pilote`
3. `registre-3d`
4. `registre-cao`, `registre-antenna`
5. `composants`, `routage`
6. `savoir-metier`
7. `retrait-gemini`

## Specs écrites

- [x] `mcp-noyau` : [SPEC-mcp-noyau.md](SPEC-mcp-noyau.md)
