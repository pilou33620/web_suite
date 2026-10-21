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
  `web-suite` n'est pas chargé et passe par `savoir_lire`. Fait le 21/10/2026 sous Linux, outils
  sur le même poste : `claude -p` avec le MCP lit la page WEB_ANTENNA ouverte (onglets,
  `antenna_lire`) ; restent le Pi et Windows par SSH.
- [ ] **WEB_CAO** : `cao_paire`, `cao_placement_auto` et `cao_chemin` dans la vraie page (voir son A-FAIRE).
- [x] **WEB_ANTENNA** (21/10/2026) : vrai calcul openEMS (exemple patch, openEMS 0.0.37 compilé
  sous Linux) lancé depuis la page, puis lu par le MCP : `antenna_lire` resultats (f0 2,45 GHz,
  S11 −23,4 dB, bande −10 dB 2,421–2,479 GHz) et courbe, `antenna_vue` onglets S11 et impédance,
  capture de la courbe juste.
- [x] **WEB_3D** (21/10/2026) : éprouvé dans Chromium par le MCP : un fichier absent, un STEP et un
  STL corrompus rendent « echouee » avec le message de la page (le STL rendait l'erreur brute du
  lecteur : corrigé), un bon STEP s'ouvre ; `claude -p` avec le MCP lit la scène et la mesure.

## Vérifications à faire avec toi

- [ ] **mcp-noyau** : sur le Pi, `/mcp` montre `web_suite` connecté et la réponse de Claude est
  juste ; puis depuis le PC Windows par SSH (`tasks/todo-mcp-noyau.md`).
- [x] **api-outils** (21/10/2026) : passages cités relus, toujours justes (`tasks/todo-api-outils.md`).
- [x] **registre-3d** (21/10/2026) : les deux checkpoints revus (`tasks/todo-registre-3d.md`).
- [ ] **chemin-auto** : choix revus le 21/10/2026 (via 10 mm ; virage et pas gardés) ; restent la
  décision sur les zones de cuivre et l'essai réel sur le Pi (`tasks/todo-chemin-auto.md`).
- [x] **Branche `mcp`** (21/10/2026) : fusionnée dans main, ici et dans WEB_CAO.

## Laissé par les modules, en connaissance de cause

- [x] **ecriture-cao** (21/10/2026, branche `mcp`) : les lots de Claude se fondent entre eux
  quand l'historique est plein, les étapes de l'utilisateur restent ; « affecter » crée une
  classe que le schéma connaît (Horloge, RF, interfaces…).
- [x] **dessin-schema** (21/10/2026, branche `mcp`) : étiquettes et alimentations désignées par
  « #id » (rendu par etiqueter, lecture `etiquettes`) pour déplacer, tourner, supprimer.
- [x] **routage** (21/10/2026, branche `mcp`) : aperçu dessiné de `cao_placement_auto` ; la
  méthode « blocs » a son essai réussi au harnais.
- [x] **registre-antenna** (21/10/2026) : l'étape montrée par `antenna_vue` n'est plus celle
  qu'enregistre le projet, ni un panneau montré celui de la disposition, tant que l'utilisateur
  n'y touche pas (`pilote-antenna.js`, banc-interface section 10).
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
  « Modèle Simulation » de la référence. 4 Mo au plus par fichier.
- [x] **Canal de pilotage élargi** (21/10/2026, sur main dans les quatre dépôts) : une commande
  porte jusqu'à 8 Mo (`pilote.py`, comme une réponse) ; seul le MCP du poste en donne.
  `lib_kicad_convertir` (200 Ko) passe, `lib_simulation_lier` monte à 4 Mo.

## Défauts des outils vus pendant les essais (antérieurs au MCP)

- [ ] Gestion LIB : son tutoriel ne passe pas en Chromium headless sur le Pi (la mémoire au
  chargement, environ 890 Mo, est retirée de la liste le 21/10/2026).
- [x] WEB_ANTENNA : 422 de `/api/openems` pendant le tutoriel (21/10/2026) : la pose de
  l'exemple revérifiait avant que le gabarit ait dessiné son cuivre ; un document sans cuivre
  est maintenant refusé dans la page, sans appel au serveur.
- [x] **Canal de pilotage** (21/10/2026) : une commande porte jusqu'à 8 Mo (`pilote.py`, copies des
  quatre dépôts) ; avant, 64 Ko : un kicad_mod de plus de 64 Ko aurait été refusé.
