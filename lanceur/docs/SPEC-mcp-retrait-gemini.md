# Spec : retrait-gemini

Module 11 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`savoir-metier`](SPEC-mcp-savoir-metier.md)
et de [`routage`](SPEC-mcp-routage.md).
Statut : **validée le 2026-10-19**.

## Objectif

Retirer les assistants Gemini/Gemma, maintenant que Claude fait leur travail par le MCP,
sans casser ce que le canal de pilotage réutilise.

### Décisions (2026-10-19)

Tout est retiré :
- WEB_CAO : l'assistant du PCB, du schéma et de la visionneuse (`commun/ia-assistant.js`
  et `.css`, bouton « ✨ IA », panneau, clic droit, « 🤖 Envoyer à l'IA » du rapport de
  simulation, étape du tutoriel) ;
- WEB_CAO : le panneau Gemini de Gestion LIB (`gestion-lib/js/06-ia-lib.js`) ;
- WEB_ANTENNA : le panneau IA (Gemini et modèle local) ;
- `/api/ia/cle` des deux serveurs, et la lecture du fichier `api_key_free_ia_studio.txt`.

### Ce qui reste

- `WEB_CAO/commun/lecture-cao.js` (lectures du MCP) ;
- `WEB_ANTENNA/js/29-lecture-antenna.js` (lectures, `registre-antenna`) ;
- le registre des cartes d'antenne (`IA_CHAMPS`, `iaProposition`, `iaAppliquer`,
  `iaAnnuler`, `iaValider`) et l'audit local `controles()`, utilisés par `antenna_proposer`
  et `antenna_lire audit` : ils passent de `30-ia.js` à `js/31-cartes-antenna.js`, sans
  changer leur comportement, sous des noms sans « ia » ;
- `/api/ia` dans les routes interdites du MCP (défense en profondeur, sans coût) ;
- les lignes `.gitignore` du fichier de clé (une clé oubliée sur un poste reste hors git).

### Critères d'acceptation

1. Plus aucune trace active : une recherche de `generativelanguage`, `gemma`, `gemini`,
   `/api/ia/cle`, `ia-assistant`, `06-ia-lib`, `30-ia.js` ne trouve rien hors de l'historique
   des docs et de la liste des routes interdites du MCP.
2. `GET /api/ia/cle` rend 404 sur les deux serveurs ; les tests de serveur le vérifient.
3. Le registre des cartes d'antenne garde ses tests (`banc-interface.js` §9, renommés) ;
   `antenna_proposer` et `antenna_lire audit` marchent comme avant (bancs et essai réel).
4. Pages chargées dans Chromium sans erreur de console : PCB, schéma, visionneuse,
   Gestion LIB, WEB_ANTENNA.
5. Nettoyage :
   - balises `script`/`link`, bouton et panneau dans les 4 pages WEB_CAO et `index.html` ;
   - `"ia"` dans les listes de `00-espace-config.js` et dans `profils/Pilou.json` ;
   - crochets du clic droit, du tutoriel, du rapport de simulation ;
   - au chargement, une seule fois, les clés `openems_ia_cle` et `openems_ia_local` sont
     effacées du stockage de session.
6. Tests et CI : `banc-ia-markdown.js` et la partie IA de `banc-ia-xss.js` retirés ; le
   contrôle « `ia.css` préfixe de `ia-assistant.css` » de la CI web_suite retiré ; les
   tests de refus de `/api/ia/cle` changés en tests de 404.
7. Docs : README de WEB_CAO et WEB_ANTENNA, `A-FAIRE.md`, `lanceur/docs/DETAILS.md`,
   README racine : l'assistant intégré est remplacé par Claude Code et le MCP (renvoi à
   DETAILS.md) ; les « comme Gemini » des specs restent (historique).

## Architecture

Un commit par dépôt et par assistant, chacun avec ses bancs verts : la suppression se
défait par `git revert` si besoin.

1. WEB_ANTENNA : déplacer le registre des cartes et l'audit vers `31-cartes-antenna.js`
   (bancs verts, sorties identiques), puis retirer le panneau IA.
2. WEB_CAO : retirer Gestion LIB IA, puis l'assistant des trois pages.
3. Serveurs : retirer `/api/ia/cle`.
4. web_suite : CI, docs.

## Commandes

```
WEB_CAO :     bundles, puis node editeur-pcb/test/harness.js ; node editeur-schematique/test/harness.js ;
              node visionneuse-ipc2581/test/harness-sim.js ; node commun/test/banc-pilote-cao.js ;
              python3 python/test/banc-serveur-routes.py ; python3 python/test/banc-lib-routes.py
WEB_ANTENNA : python3 python/test/banc-openems.py ; python3 python/test/banc-serveur.py
web_suite :   python3 lanceur/banc_mcp.py ; node lanceur/banc_pilote.js
```

## Stratégie de test

- Les bancs existants, verts après chaque commit.
- Recherche des traces (critère 1) dans un test de la CI web_suite.
- Essai réel dans Chromium : chaque page sans erreur de console ; `antenna_proposer` de
  bout en bout.

## Limites

- **Toujours** : déplacer avant de supprimer ; un commit par étape.
- **Demander d'abord** : retirer `/api/ia` des routes interdites du MCP ; toucher aux
  cartes d'antenne au-delà du renommage.
- **Jamais** : supprimer `lecture-cao.js`, le registre des cartes ou l'audit d'antenne.

## Questions ouvertes

Aucune.
