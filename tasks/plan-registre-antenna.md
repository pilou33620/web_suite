# Plan : registre-antenna

Spec : [lanceur/docs/SPEC-mcp-registre-antenna.md](../lanceur/docs/SPEC-mcp-registre-antenna.md)
(validée le 2026-10-19). Liste des tâches : [todo-registre-antenna.md](todo-registre-antenna.md).

## Vue d'ensemble

Deux outils MCP pour la page WEB_ANTENNA ouverte, sans rien y écrire :
- `antenna_lire` : conception, résultats, courbe, balayage, audit, réglages, et carte importée ;
- `antenna_vue` : 2D/3D, maillage, ajuster, onglet de résultats, point de balayage,
  étape, panneau, et capture de la carte, de la 3D ou des courbes.

D'abord, les lectures de Gemini quittent `js/30-ia.js` pour un fichier commun, sans
changer leur sortie, comme `lecture-cao.js` à registre-cao. Les écritures restent celles
d'`antenna_proposer`, et aucun calcul n'est jamais lancé.

## Graphe des dépendances

```
T1 js/29-lecture-antenna.js (déplacement, sorties identiques)
 └── T2 antenna_lire dans la page (pilote-antenna.js)
      └── T3 antenna_vue et captures (carte, 3D, courbe)
           └── T4 outils MCP (lanceur/mcp_registre_antenna.py)
                └── T5 essai réel, puis revue
```

## Décisions d'architecture

- **Le déplacement d'abord, sans rien changer** (T1). Un relevé des sorties est pris avant
  le déplacement, dans le banc : `iaContexte()` et les outils de lecture, sur les états
  que le banc construit déjà (conception, carte importée, balayage fini). Après le
  déplacement, les sorties doivent être identiques au caractère près.
- **Ce qui part dans `js/29-lecture-antenna.js`** (global `LECTURE_ANTENNA`, chargé avant
  `30-ia.js`) :
  - les aides de formatage `nb`, `nbp`, `fHz`, `uniteDoc` ;
  - `extraireContexteOpenems` ;
  - les outils de lecture (`OUTILS_CARTE`, `OUTIL_BALAYAGE`, `outilsLecture`,
    `tableauBalayage`, `balayageTermine`, `contient`, `borner`) ;
  - `resultatCourant`.

  `30-ia.js` reprend ces aides depuis le global, et ses `window.ia*` restent pareils.
  Le protocole propre à Gemini (`consigneOutils`, `executerOutils`) reste dans
  `30-ia.js`. `IA_CHAMPS`, les cartes et `controles()` aussi : ils partiront à
  `retrait-gemini`, et les lectures `audit` et `reglages` les appellent au moment de
  l'appel (`iaControles`, `IA_CHAMPS`, `iaResoudre`).
- **Lectures nouvelles**, écrites dans `29-lecture-antenna.js` :
  - `resultats` : résumé de `antRes()` ;
  - `courbe` : points `f; valeur`, 200 au plus, sous-échantillonnés au-delà ;
  - `audit` : `iaControles()`, avec la carte de correction de chaque contrôle en JSON,
    prête pour `antenna_proposer` ;
  - `reglages` : chaque chemin d'`IA_CHAMPS`, avec sa valeur actuelle, son unité, ses
    bornes et ses choix, filtrable.
- **Vue** : `antVuePoser`, `antBasculerVueMaillage`, `fit`, `ANT_ONGLET` +
  `antResultatsRendre`, `ANT_BAL_POINT`, `ANT.etape` + `antAssistantRendre`,
  `wsShow`/`wsReveler`. Tout est vérifié avant le premier changement, comme `cao_vue`.
  Rien ne passe par `antReglagesEcrire` (préférences).
- **Captures** : `Pilote.capture(fn)` reçoit une fonction qui rend la toile choisie par la
  dernière vue (`carte` par défaut). Pour la 3D, elle force un rendu (`ant3dDessiner()`)
  juste avant : la toile WebGL n'a pas `preserveDrawingBuffer`, et `pilote.js` la copie
  dans la même tâche. `pilote.js` et `pilote.py` ne changent pas.
- **Jamais lancer** : un test remplace `antLancer`, `antLancerBalayage` et
  `antLancerTableauS` par des pièges, puis appelle toutes les lectures et vues.
- **MCP** : `lanceur/mcp_registre_antenna.py`, page `antenna`, même forme que
  `mcp_registre_cao.py`. `antenna_vue` avec `capturer` enchaîne `pilote_capture` sur le
  même onglet.

## Risques

| Risque | Impact | Parade |
|---|---|---|
| `extraireContexteOpenems` dépend d'aides internes de l'IIFE que le reste de `30-ia.js` utilise aussi (`nb`, `nbp`, `fHz` : plus de 60 usages) | Moyen | Elles partent avec lui et `30-ia.js` les reprend du global ; relevé avant/après |
| Capture 3D noire (WebGL sans `preserveDrawingBuffer`) | Moyen | Rendu forcé dans la fonction de capture ; vérifié à l'essai réel (le banc n'a pas de WebGL) |
| `antResultatsRendre` ou `antVuePoser` écrivent des préférences | Faible | Pièges sur `antReglagesEcrire` et `localStorage.setItem` pendant les tests de vue |
| Le graphe de WEB_ANTENNA est d'avant les derniers commits | Faible | Lu pour s'orienter, faits vérifiés dans le code ; mise à jour proposée à la fin |

## Questions ouvertes

Aucune.
