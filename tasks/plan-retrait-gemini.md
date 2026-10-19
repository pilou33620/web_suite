# Plan : retrait-gemini

Spec : [lanceur/docs/SPEC-mcp-retrait-gemini.md](../lanceur/docs/SPEC-mcp-retrait-gemini.md)
(validée le 2026-10-19). Liste des tâches : [todo-retrait-gemini.md](todo-retrait-gemini.md).

## Vue d'ensemble

Claude fait par le MCP ce que faisaient les assistants Gemini / Gemma. On les retire
entièrement : trois pages CAO, Gestion LIB, WEB_ANTENNA, `/api/ia/cle` des deux serveurs.
Le canal de pilotage, lui, réutilise une partie de ces assistants, qui reste en place :
- les lectures (`lecture-cao.js`, `29-lecture-antenna.js`) ;
- le registre des cartes d'antenne et son audit.

Chaque étape fait un commit, avec ses bancs verts, et se défait par `git revert`.

## Graphe des dépendances

```
T1 WEB_ANTENNA : registre des cartes + audit → js/31-cartes-antenna.js (CARTES_ANTENNA)
 └── T2 WEB_ANTENNA : retirer le panneau IA (30-ia.js, ia.css, page, clés de session)
T3 WEB_CAO : retirer l'assistant de Gestion LIB (06-ia-lib.js)
T4 WEB_CAO : retirer l'assistant des trois pages (ia-assistant.js / .css)
T5 Serveurs : /api/ia/cle → 404 (web_CAO.py, web_antenna.py)        ← T2, T3, T4
 └── T6 web_suite : CI (contrôle ia.css retiré, recherche des traces), docs
      └── T7 essai réel (pages sans erreur de console, antenna_proposer), revue
```

## Décisions d'architecture

- **Déplacer avant de supprimer (T1).**
  - Le registre des cartes et l'audit passent de `30-ia.js` à `js/31-cartes-antenna.js`,
    sous un objet global `CARTES_ANTENNA` : `CHAMPS`, `resoudre`, `valider`, `proposition`,
    `appliquer`, `annuler`, `controles`, `grammaire`. On y met aussi tout ce dont ces
    fonctions ont besoin (validation des formes, `rafraichir`, `lireBalayage`), copié tel
    quel.
  - `30-ia.js` les reprend de `CARTES_ANTENNA` le temps de T1, puis disparaît en T2.
  - `pilote-antenna.js`, `29-lecture-antenna.js` et `banc-interface.js` §9 passent aux
    nouveaux noms.
  - Comportement inchangé, deux preuves :
    - le relevé `test/releve-lecture-antenna.json` reste identique ;
    - les tests §9 passent sans autre changement que les noms.
- **Un commit par assistant** :
  - T2 : WEB_ANTENNA ;
  - T3 : Gestion LIB ;
  - T4 : les trois pages CAO, ensemble, car ils partagent `ia-assistant.js`.

  Chaque retrait emporte ses crochets :
  - le clic droit (`iaAfficherMenuContextuel` dans `05-tools.js`, `09-interaction.js` et
    `04-interaction.js`) ;
  - le tutoriel (`#bModeIaLib`, `#bIaAssistant` dans `commun/tuto.js`) ;
  - le bouton du rapport de simulation (`bSimRapIa`, `simRapVersIa`) ;
  - le panneau « ia » des espaces (`00-espace-config.js`, `profils/Pilou.json`) ;
  - les balises des pages.

  Les commentaires qui citent l'assistant (`lecture-cao.js`, `pilote-cao.js`,
  `simulation-rapport.js`) sont réécrits pour le MCP.
- **Les clés de session d'ANTENNA** (`openems_ia_cle`, `openems_ia_local`) sont effacées une
  fois au chargement, par une ligne dans le code qui reste (`31-cartes-antenna.js` ou
  l'initialisation de la page).
- **Serveurs (T5)** : `GET /api/ia/cle` et sa lecture de `api_key_free_ia_studio.txt`
  disparaissent, donc la route rend 404. Restent en place :
  - les lignes `.gitignore` du fichier de clé ;
  - la liste des fichiers que la mise à jour préserve (`web_CAO.py`), où le nom du fichier
    figure : une clé oubliée sur un poste reste protégée.

  Les tests qui vérifiaient le refus de la route vérifient désormais un 404.
- **Recherche des traces (T6)** : un petit banc de web_suite (`lanceur/banc_traces_ia.py`),
  lancé par la CI après le clone des outils. Il cherche `generativelanguage`, `gemma`,
  `gemini`, `/api/ia/cle`, `ia-assistant`, `06-ia-lib` et `30-ia.js` dans les trois dépôts.
  - Ce qui reste permis, et où :
    - l'historique des docs : `lanceur/docs/` et `tasks/` ;
    - la liste des routes interdites du MCP ;
    - le relevé du savoir et son banc (`savoir-trace.md`, `banc_savoir.py`, qui citent les
      consignes à une révision figée) ;
    - les `.gitignore`.
  - Le banc s'éprouve lui-même sur un texte factice.
- **Docs** : les README de WEB_CAO, de WEB_ANTENNA et de la racine, `A-FAIRE.md` et
  `DETAILS.md` remplacent l'assistant intégré par Claude Code et le MCP, avec un renvoi à
  DETAILS.md. Les « comme Gemini » des specs restent : c'est l'historique.

## Risques

| Risque | Impact | Parade |
|---|---|---|
| Une dépendance cachée de `30-ia.js` (une fonction du registre appelle un utilitaire du panneau) | Élevé | T1 seul, panneau encore présent ; relevé et §9 identiques avant de supprimer |
| Un crochet oublié : la page lève une erreur au clic droit ou au tutoriel | Moyen | Recherche de `ia[A-Z]` après chaque retrait ; essai Chromium, console vide, clic droit et tutoriel joués |
| Le bundle PCB ou schéma garde l'assistant | Moyen | Bundles reconstruits, harnais verts, recherche dans `dist/` |
| Mise en page des espaces cassée par le panneau « ia » retiré d'un profil enregistré | Moyen | Les profils inconnus de panneau ignorent la clé (vérifié) ; `Pilou.json` nettoyé |
| La CI web_suite casse quand ia-assistant.css disparaît | Certain | Le contrôle ia.css est retiré dans le même mouvement (T6, poussé juste après les outils) |

## Questions ouvertes

Aucune.
