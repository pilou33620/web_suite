# Ce qui reste à faire — web_suite

Le serveur MCP (`lanceur/mcp_web_suite.py`, 71 outils) et ses modules sont faits jusqu'à
retrait-gemini inclus, plus chemin-auto (branche `mcp`, 21/10/2026) (carte des modules : [lanceur/docs/SPEC-mcp.md](lanceur/docs/SPEC-mcp.md),
détail de chaque module dans `tasks/`). Ce qui reste, au 21/10/2026 :

## Modules

- [x] **chemin-auto** (21/10/2026, branche `mcp`) : `cao_chemin` propose les points de passage
  de `cao_router` par A* sur grille ([SPEC-mcp-chemin-auto.md](lanceur/docs/SPEC-mcp-chemin-auto.md)).
  Reste : l'essai réel sur le Pi et la revue des choix pris d'office (`tasks/todo-chemin-auto.md`).

## Essais réels en attente

- [ ] **iPad** : le canal de pilotage depuis un appareil appairé (cartes, interrupteur, capture).
- [ ] **Claude Code relancé avec le MCP** : sur le Pi, puis sous Windows par SSH, où le skill
  `web-suite` n'est pas chargé et passe par `savoir_lire`.
- [ ] **WEB_CAO** : `cao_paire`, `cao_placement_auto` et `cao_chemin` dans la vraie page (voir son A-FAIRE).
- [ ] **WEB_ANTENNA** : un vrai calcul openEMS pour les courbes (voir son A-FAIRE).
- [ ] **WEB_3D** : un fichier introuvable ou illisible doit rendre « echouee » avec le message de
  la page (`ui.charger` rend {ok, erreur}) : à éprouver en vrai (`tasks/todo-registre-3d.md`).

## Vérifications à faire avec toi

- [ ] **mcp-noyau** : sur le Pi, `/mcp` montre `web_suite` connecté et la réponse de Claude est
  juste ; puis depuis le PC Windows par SSH (`tasks/todo-mcp-noyau.md`).
- [ ] **api-outils** : relecture des passages cités dans la spec (`tasks/todo-api-outils.md`).
- [ ] **registre-3d** : revue des deux checkpoints (`tasks/todo-registre-3d.md`).
- [ ] **chemin-auto** : revue des choix pris d'office (coût d'une via, d'un virage, pas de 0,25 mm,
  zones de cuivre ignorées) après un essai réel (`tasks/todo-chemin-auto.md`).
- [ ] **Branche `mcp`** : à fusionner en même temps dans web_suite et dans WEB_CAO ; les outils
  `lib_simulation_lier`, `cao_chemin` et la lecture `etiquettes` n'ont leur commande de page que
  dans la branche `mcp` de WEB_CAO.

## Laissé par les modules, en connaissance de cause

- [x] **ecriture-cao** (21/10/2026, branche `mcp`) : les lots de Claude se fondent entre eux
  quand l'historique est plein, les étapes de l'utilisateur restent ; « affecter » crée une
  classe que le schéma connaît (Horloge, RF, interfaces…).
- [x] **dessin-schema** (21/10/2026, branche `mcp`) : étiquettes et alimentations désignées par
  « #id » (rendu par etiqueter, lecture `etiquettes`) pour déplacer, tourner, supprimer.
- [x] **routage** (21/10/2026, branche `mcp`) : aperçu dessiné de `cao_placement_auto` ; la
  méthode « blocs » a son essai réussi au harnais.
- [ ] **registre-antenna** : une vue changée par `antenna_vue` part avec le prochain
  enregistrement du projet ; un panneau montré reste si l'utilisateur change ensuite la
  disposition.
- [x] **composants** : constantes des puces alignées sur l'IPC-7351B relayé par KiCad ; 0201, 0402,
  0603 et QFN comparés aux empreintes KiCad, DIP et barrette aussi (21/10/2026, A-FAIRE de WEB_CAO).

## Capacités manquantes, relevées en route

- [x] `cao_sim_ecrire` : condensateurs (ESR, ESL, capacité) et bornes de la chute DC, par une
  « cible » ; `cao_calcul` : l'analyse « Bus synchrone » (`analyse=bus`) (21/10/2026).
- [x] Simulation RF : un Touchstone à N ports sur une puce qui a plus de broches (« ! broches »,
  « ! masse » dans le fichier) et plusieurs états d'une pièce au catalogue ; les deux états de
  l'AS179 sont dans la LIB (21/10/2026, A-FAIRE de WEB_CAO).
- [x] **Outil MCP pour lier un modèle de simulation** (21/10/2026, branche `mcp`) :
  `lib_simulation_lier`, carte toujours cliquée ; le serveur WEB_CAO ne réécrit que la case
  « Modèle Simulation » de la référence. 60 Ko au plus par fichier.
- [ ] **Canal de pilotage borné à 64 Ko par commande** (`pilote.py`, `max_petit`), relevé le
  21/10/2026 en lisant le code : `lib_kicad_convertir` accepte 200 Ko côté MCP, mais un
  kicad_mod de plus de 64 Ko serait refusé par le canal (à éprouver). Élargir demande de
  changer `pilote.py` dans les trois outils (copies identiques vérifiées par la CI).

## Défauts des outils vus pendant les essais (antérieurs au MCP)

- [ ] Gestion LIB : son tutoriel ne passe pas en Chromium headless sur le Pi (la mémoire au
  chargement, environ 890 Mo, est retirée de la liste le 21/10/2026).
- [ ] WEB_ANTENNA : 422 de `/api/openems` pendant le tutoriel sur une page vide.
