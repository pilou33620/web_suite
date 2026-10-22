# Historique — l'ancien `A-FAIRE.md` (jusqu'au 22/10/2026)

> Le 22/10/2026, `A-FAIRE.md` a été séparé en deux : [A-FAIRE.md](A-FAIRE.md) (fonctionnalités et
> bugs encore ouverts) et [TESTS-A-FAIRE.md](TESTS-A-FAIRE.md) (essais à faire). Le texte d'avant
> est gardé ici tel quel : c'est lui que visent les renvois « A-FAIRE.md:ligne » des specs
> d'essai (`tasks/essais/`). Ses cases encore vides ont été reprises dans les deux nouveaux
> fichiers.
>
> Le 22/10/2026, les essais à faire (`TESTS-A-FAIRE.md` et les specs de `tasks/essais/`) ont été retirés : on ne s'occupe pour
> l'instant que des fonctionnalités, les essais se feront plus tard et autrement. Ils restent
> dans l'historique git, au commit `2154478`.

## Ce qui restait au 21/10/2026

Le serveur MCP (`lanceur/mcp_web_suite.py`, 73 outils) et ses modules sont faits jusqu'à
retrait-gemini inclus, plus chemin-auto (21/10/2026) (carte des modules : [lanceur/docs/SPEC-mcp.md](lanceur/docs/SPEC-mcp.md),
détail de chaque module dans `tasks/`). Ce qui reste, au 21/10/2026 :

## Modules

- [x] **chemin-auto** (21/10/2026) : `cao_chemin` propose les points de passage
  de `cao_router` par A* sur grille ([SPEC-mcp-chemin-auto.md](lanceur/docs/SPEC-mcp-chemin-auto.md)).
  Reste : l'essai réel sur le Pi (`tasks/todo-chemin-auto.md`).

## Essais réels en attente

Chaque essai a sa spec, un fichier par essai, la plupart à donner à Claude in Chrome sur le PC du boulot : [tasks/essais/](tasks/essais/README.md)
(Pi, Windows, iPad, calculs openEMS de WEB_ANTENNA, pièces, cartes et normes de WEB_CAO).

- [ ] **iPad** : le canal de pilotage depuis un appareil appairé (cartes, interrupteur, capture).
- [ ] **Claude Code relancé avec le MCP** : sur le Pi, puis sous Windows par SSH, où le skill
  `web-suite` n'est pas chargé et passe par `savoir_lire`. Fait le 21/10/2026 sous Linux, outils
  sur le même poste : `claude -p` avec le MCP lit la page WEB_ANTENNA ouverte (onglets,
  `antenna_lire`) ; restent le Pi et Windows par SSH.
- [x] **WEB_CAO** (21/10/2026) : `cao_paire`, `cao_placement_auto` et `cao_chemin` dans la vraie page sous Linux (Chromium, serveur WEB_CAO sur un PROJETS vide, exemple 2, vrai serveur MCP en JSON-RPC, cartes cliquées) : `cao_chemin` 20 liaisons en 0,5 s (19 trouvées), `cao_router` pose les 19 ; `cao_paire` USB posée de J1 à U1 sans erreur DRC ; `cao_placement_auto` satellites et forces appliqués ; aucune erreur JavaScript. Trouvé puis corrigé (21/10/2026, avec ton accord) : la route U3.1 → U3.3 (+5V) entrait dans la pastille U3.2 (GND). La faute était dans le chanfrein posé au dépôt (`mitreAt`, `05-tools.js` de WEB_CAO), pas dans `11-pns-*` à `13-pns-shove.js` : il vérifie maintenant l'isolation et recule s'il le faut.
  Reste la durée sur le Pi.
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
- [x] **chemin-auto** (21/10/2026) : choix revus (via 10 mm ; virage, pas et zones ignorées gardés) ;
  l'essai réel sur le Pi reste dans « Essais réels en attente » (`tasks/todo-chemin-auto.md`).
- [x] **Branche `mcp`** (21/10/2026) : fusionnée dans main, ici et dans WEB_CAO.

## Laissé par les modules, en connaissance de cause

- [x] **ecriture-cao** (21/10/2026) : les lots de Claude se fondent entre eux
  quand l'historique est plein, les étapes de l'utilisateur restent ; « affecter » crée une
  classe que le schéma connaît (Horloge, RF, interfaces…).
- [x] **dessin-schema** (21/10/2026) : étiquettes et alimentations désignées par
  « #id » (rendu par etiqueter, lecture `etiquettes`) pour déplacer, tourner, supprimer.
- [x] **routage** (21/10/2026) : aperçu dessiné de `cao_placement_auto` ; la
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
- [x] **Outil MCP pour lier un modèle de simulation** (21/10/2026) :
  `lib_simulation_lier`, carte toujours cliquée ; le serveur WEB_CAO ne réécrit que la case
  « Modèle Simulation » de la référence. 4 Mo au plus par fichier.
- [x] **Canal de pilotage élargi** (21/10/2026, sur main dans les quatre dépôts) : une commande
  porte jusqu'à 8 Mo (`pilote.py`, comme une réponse ; avant, 64 Ko) ; seul le MCP du poste en donne.
  `lib_kicad_convertir` (200 Ko) passe, `lib_simulation_lier` monte à 4 Mo.
- [x] **HTTPS en IPv6** (22/10/2026, sur main dans les quatre dépôts) : `appairage.py` met au
  certificat les IPv6 privées du poste (Tailscale `fd7a:…`, ULA `fc00::/7`, pas le lien local
  `fe80::`) et les ajoute aux contraintes de la CA ; une CA d'avant (sans `ca.json`) est refaite
  une fois, à réinstaller sur chaque appareil. Le lanceur et WEB_3D écoutent en double pile,
  IPv4 seule si IPv6 manque ([SPEC-mcp-canal-pilote.md](lanceur/docs/SPEC-mcp-canal-pilote.md)).

## Défauts des outils vus pendant les essais (antérieurs au MCP)

- [x] Gestion LIB (corrigé le 21/10/2026, WEB_CAO) : environ 890 Mo au chargement sur le Pi.
  Chaque ligne du catalogue (250) portait trois listes avec toute la LIB : 1500 empreintes et
  1500 symboles faisaient 5 millions de nœuds ; une liste se remplit maintenant à l'ouverture
  (23 000 nœuds). Le tutoriel visait des boutons hors de l'écran (étapes 7 à 9) : il fait
  défiler la barre. Reste à le confirmer sur le Pi (`tasks/essais/ESSAI-pi-tutoriel-gestion-lib.md`).
- [x] WEB_ANTENNA : 422 de `/api/openems` pendant le tutoriel (21/10/2026) : la pose de
  l'exemple revérifiait avant que le gabarit ait dessiné son cuivre ; un document sans cuivre
  est maintenant refusé dans la page, sans appel au serveur.
