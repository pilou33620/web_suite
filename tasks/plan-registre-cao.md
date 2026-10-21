# Plan : registre-cao

Spec : [lanceur/docs/SPEC-mcp-registre-cao.md](../lanceur/docs/SPEC-mcp-registre-cao.md)
(validée le 2026-10-19). Liste des tâches : [todo-registre-cao.md](todo-registre-cao.md).

## Vue d'ensemble

Six outils MCP de lecture, de vue et de DRC pour les pages WEB_CAO. Les registres
`OUTILS_PCB`, `OUTILS_IPC` et `OUTILS_SIM` sortent de l'assistant Gemini pour un fichier
commun, sans changer leur sortie. Le schéma gagne son registre. `pilote-cao.js` déclare
les commandes. Enfin, `lanceur/mcp_registre_cao.py` les expose.

## Graphe des dépendances

```
T1 commun/lecture-cao.js (déplacement, sorties identiques)
 ├── T2 registre du schéma (LECTURE_CAO.schema)
 │    └── T3 commandes de lecture dans pilote-cao.js (4 pages)
 │         └── T4 cao_vue et cao_drc dans la page
 │              └── T5 outils MCP (lanceur/mcp_registre_cao.py)
 │                   └── T6 essai réel, puis revue
```

## Décisions d'architecture

- **Le déplacement d'abord, sans rien changer** (T1). Un relevé des sorties est pris
  **avant** le déplacement, par un script jetable qui extrait le bloc des registres de
  `ia-assistant.js`. Il est rangé comme fixture du banc PCB. Après le déplacement,
  `LECTURE_CAO.pcb` doit redonner ce relevé au caractère près.
- **Un seul global**, `LECTURE_CAO = { pcb, ipc, sim, schema }`, posé par un script
  classique chargé avant `ia-assistant.js` dans les trois pages. Les bundles `dist/`
  suivent les `<script src>` : rien à changer dans `monofichier.py`.
- **Les commandes de page sont minces** : `executer(args)` choisit `LECTURE_CAO[page][quoi]`,
  appelle `fn(args)` et rend `lignes.join("\n")`. La trace « Claude a lancé : … » vient déjà
  de `pilote.js`.
- **La vue réutilise les fonctions de la page** :
  - schéma : `schCibleAller`, `selectNet`, `gotoPage`, `fit` ;
  - PCB : `rpCadrer` + `fpBBox`/`rpNetBox`, `S.hlNet`, `S.cuL[i].vis`, `fit` ;
  - visionneuse : `choisirComp`, `choisirNet`, couches `visible`, `fit`.
  
  Aucune ne passe par `push()`.
- **MCP** : même forme que `mcp_registre_3d.py` (`refuser`, `choix`, `booleen`), envoi par
  `mcp_pilote.envoyer` vers les pages `schema`, `pcb`, `visionneuse-ipc`. `capturer` enchaîne
  `pilote.pilote_capture`, comme `web3d_vue`.
- **CI** : rien à ajouter. La CI de WEB_CAO lance déjà les deux harnais, `harness-sim.js` et
  `banc-pilote-cao.js` ; celle de web_suite lance `banc_mcp.py` et `banc_pilote.js`.

## Risques

| Risque | Impact | Parade |
|---|---|---|
| Les registres lisent des aides internes de l'IIFE (`contient`, `borner`, `mm`, `nomCouchePcb`, `simEtat`…) | Moyen | Les aides partent avec eux dans `lecture-cao.js` ; `ia-assistant.js` les reprend du global s'il s'en sert ailleurs |
| La visionneuse n'a pas de harnais qui construit `V` depuis un fichier | Moyen | Banc Node sur un `V` minimal fait à la main (T3), plus la vraie carte à l'essai réel (T6) |
| Un nom de global déjà pris dans une page (`LECTURE_CAO`, `schTrouver`) | Faible | `grep` sur les trois pages avant d'écrire ; préfixe `lc` pour les aides |
| DRC long sur une grosse carte | Faible | Le canal rend « en_attente » après 2 min ; mesuré à l'essai réel |

## Questions ouvertes

Aucune. Les deux questions de la spec (noms de broches, durée du DRC) sont tranchées à
l'essai réel du 2026-10-19 (`lanceur/docs/SPEC-mcp-registre-cao.md`, « Questions tranchées »).
