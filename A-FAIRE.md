# Ce qui reste à faire — web_suite

Le serveur MCP (`lanceur/mcp_web_suite.py`, 64 outils) et ses modules sont faits jusqu'à
retrait-gemini inclus (carte des modules : [lanceur/docs/SPEC-mcp.md](lanceur/docs/SPEC-mcp.md),
détail de chaque module dans `tasks/`). Ce qui reste, au 19/10/2026 :

## Modules

- [ ] **chemin-auto** : le routage automatique (A*) qui proposerait lui-même les points de
  passage de `cao_router` ; spec à écrire (décision du 19/10/2026 : « Oui, mais spec plus tard »).

## Essais réels en attente

- [ ] **iPad** : le canal de pilotage depuis un appareil appairé (cartes, interrupteur, capture).
- [ ] **Claude Code relancé avec le MCP** : sur le Pi, puis sous Windows par SSH, où le skill
  `web-suite` n'est pas chargé et passe par `savoir_lire`.
- [ ] **WEB_CAO** : `cao_paire` et `cao_placement_auto` dans la vraie page (voir son A-FAIRE).
- [ ] **WEB_ANTENNA** : un vrai calcul openEMS pour les courbes (voir son A-FAIRE).

## Capacités manquantes, relevées en route

- [ ] `cao_sim_ecrire` : condensateurs (ESR, ESL) et bornes de la chute DC ; `cao_calcul` :
  l'analyse « Bus synchrone » (détail dans l'A-FAIRE de WEB_CAO).

## Défauts des outils vus pendant les essais (antérieurs au MCP)

- [ ] Gestion LIB : environ 890 Mo au chargement sur le Pi (OOM possible).
- [ ] Visionneuse et WEB_ANTENNA : clic droit sans carte ouverte → `TypeError`.
- [ ] WEB_ANTENNA : 422 de `/api/openems` pendant le tutoriel sur une page vide.
