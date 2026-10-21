# Ce qui reste à faire — web_suite

Le serveur MCP (`lanceur/mcp_web_suite.py`, 64 outils) et ses modules sont faits jusqu'à
retrait-gemini inclus (carte des modules : [lanceur/docs/SPEC-mcp.md](lanceur/docs/SPEC-mcp.md),
détail de chaque module dans `tasks/`). Ce qui reste, au 21/10/2026 :

## Modules

- [ ] **chemin-auto** : le routage automatique (A*) qui proposerait lui-même les points de
  passage de `cao_router` ; spec à écrire (décision du 19/10/2026 : « Oui, mais spec plus tard »).

## Essais réels en attente

- [ ] **iPad** : le canal de pilotage depuis un appareil appairé (cartes, interrupteur, capture).
- [ ] **Claude Code relancé avec le MCP** : sur le Pi, puis sous Windows par SSH, où le skill
  `web-suite` n'est pas chargé et passe par `savoir_lire`.
- [ ] **WEB_CAO** : `cao_paire` et `cao_placement_auto` dans la vraie page (voir son A-FAIRE).
- [ ] **WEB_ANTENNA** : un vrai calcul openEMS pour les courbes (voir son A-FAIRE).
- [ ] **WEB_3D** : un fichier introuvable ou illisible doit rendre « echouee » avec le message de
  la page (`ui.charger` rend {ok, erreur}) : à éprouver en vrai (`tasks/todo-registre-3d.md`).

## Vérifications à faire avec toi

- [ ] **mcp-noyau** : sur le Pi, `/mcp` montre `web_suite` connecté et la réponse de Claude est
  juste ; puis depuis le PC Windows par SSH (`tasks/todo-mcp-noyau.md`).
- [ ] **api-outils** : relecture des passages cités dans la spec (`tasks/todo-api-outils.md`).
- [ ] **registre-3d** : revue des deux checkpoints (`tasks/todo-registre-3d.md`).

## Laissé par les modules, en connaissance de cause

- [ ] **ecriture-cao** : un flot de cartes appliquées directement (interrupteur allumé) peut
  vider l'historique de l'éditeur (80 étapes) ; « affecter » une classe ne connaît pas les
  classes publiées par le schéma.
- [ ] **dessin-schema** : les étiquettes et alimentations posées n'ont pas de repère ; seul
  Ctrl+Z les retire.
- [ ] **routage** : `cao_placement_auto` n'a pas d'aperçu dessiné (la carte dit ce qui bouge) ;
  la méthode « blocs » n'a pas d'essai positif au banc.
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
- [ ] **Outil MCP pour lier un modèle de simulation** à une référence de la LIB (demandé le
  21/10/2026) : le pendant de « 📎 Lier un fichier » de la Gestion LIB. Claude donne la référence,
  le fichier (.sNp ou SPICE) et, pour un Touchstone, la patte de chaque port et la masse (numéro
  ou nom du brochage) ; carte de confirmation, puis le fichier rangé dans `lib_simulation` et
  ajouté à la colonne « Modèle Simulation ». Réutiliser `COMPOSANTS_LIB.brancherTouchstone`
  (`commun/composants-lib.js` de WEB_CAO), comme `lib_composant_creer` réutilise ses contrôles.

## Défauts des outils vus pendant les essais (antérieurs au MCP)

- [ ] Gestion LIB : son tutoriel ne passe pas en Chromium headless sur le Pi (la mémoire au
  chargement, environ 890 Mo, est retirée de la liste le 21/10/2026).
- [ ] WEB_ANTENNA : 422 de `/api/openems` pendant le tutoriel sur une page vide.
