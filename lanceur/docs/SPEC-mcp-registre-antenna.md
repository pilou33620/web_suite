# Spec : registre-antenna

Module 7 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`canal-pilote`](SPEC-mcp-canal-pilote.md).
Statut : **validée le 2026-10-19**.

## Objectif

Donner à Claude la **lecture** de la page WEB_ANTENNA ouverte (ce qui n'est pas encore
enregistré) et des **commandes de vue**. Les écritures existent déjà : `antenna_proposer`
(cartes `reglages`, `cotes`, `balayage`, `formes`, canal-pilote).

### Décisions (2026-10-19)

1. **Lectures** : état de la conception, résultats, audit local, grammaire des réglages.
2. **Jamais lancer** openEMS, même depuis la page ouverte : Claude prépare, chiffre et
   suit ; tu lances depuis la page (décision d'api-outils confirmée).
3. **Vue** : 2D / 3D, maillage, ajuster ; onglet de résultats et point de balayage ; étape
   de l'assistant et panneaux ; **captures** de la vue 3D et des courbes, en plus de la
   carte 2D.
4. **Code partagé**, comme pour WEB_CAO : les lectures quittent `js/30-ia.js` pour un
   fichier `js/29-lecture-antenna.js`, chargé avant lui, sans changer leur sortie. Gemini
   et le canal les utilisent tous les deux jusqu'à `retrait-gemini`.

### Outils MCP

Dans `lanceur/mcp_registre_antenna.py`, page `antenna`.

| Outil MCP | `quoi` / arguments | Ce qu'il rend |
|---|---|---|
| `antenna_lire` | `conception` | Le résumé que lisait Gemini (`extraireContexteOpenems`) : origine de la carte, empilage, cuivre gardé, bande, ports, boîte d'air, maillage, arrêt, chiffrage du serveur, motif et cotes, balayage armé |
| | `resultats` | f0, S11 min, Z, bande à -10 dB, Z au pied, Dmax, rendement, couplages, écart à la cible (`antRes()`) |
| | `courbe` + `grandeur` (`s11`, `s21`, `z`, `roe`) | La courbe en points `f; valeur`, 200 points au plus (sous-échantillonnée au-delà) |
| | `balayage` | Le tableau du balayage fini (`tableauBalayage`) |
| | `audit` | Les contrôles hors ligne (`controles()`) : rang, titre, texte, et la carte de correction prête pour `antenna_proposer` |
| | `reglages` + `filtre` | Les chemins modifiables d'`IA_CHAMPS`, avec valeur actuelle, unité, bornes, choix |
| | `composants`, `composant`, `nets`, `net` | Les lectures de la carte importée (outils ```outil``` de Gemini) |
| `antenna_vue` | `vue` (`2d`/`3d`), `maillage`, `ajuster`, `onglet` (`s11`, `s21`, `roe`, `z`, `ff`, `bal`, `ts`), `point_balayage`, `etape` (1–7), `panneau`, `capturer`, `toile` (`carte`, `3d`, `courbe`) | Change la vue sans toucher au projet ; `capturer` rend l'image de la toile demandée |

### Critères d'acceptation

1. **Mêmes sorties que Gemini** : `conception`, `audit` et les lectures de la carte importée
   sont identiques au caractère près à celles de `30-ia.js`, vérifié par un relevé pris
   avant le déplacement du code.
2. **Rien n'est écrit** : aucune lecture ni commande de vue ne change les réglages d'`ANT`,
   `CON`, `PRJ.modifie`, l'historique de conception (`CON_HIST`) ni les préférences
   (`antReglagesEcrire`). La vue ne change que l'état de vue (`ANT.vue`, `ANT.vueMaillage`,
   `ANT.etape`, comme un clic ; depuis le 21/10/2026, le projet enregistre l'étape de
   l'utilisateur et la disposition un panneau montré tel qu'il était, tant que
   l'utilisateur n'y a pas touché : `pilote-antenna.js`) et
   n'enregistre pas de préférence. Le texte rendu commence par un en-tête qui le dit donnée,
   et les noms venus d'un fichier y restent chacun sur sa ligne.
3. **Bornes** : 150 lignes par liste, 200 points par courbe, puis « … (N de plus) ». Le
   MCP résume au-delà de 100 Ko.
4. **Captures** : `toile: "3d"` rend la vue 3D (WebGL, lue après un rendu), `courbe` le
   graphe de résultats ; une toile cachée est d'abord montrée par la commande, sinon
   refus clair.
5. Pas de résultat, pas de balayage, pas de carte importée : refus qui dit quoi faire
   (« lance un calcul depuis la page »).
6. Rien ne lance un calcul : aucun appel à `antLancer`, `antLancerBalayage`,
   `antLancerTableauS` depuis ce module (vérifié par un test qui les remplace par des
   pièges).

## Architecture

```
MCP (mcp_registre_antenna.py) ──▶ /api/pilote/commande ──▶ page antenna : js/pilote-antenna.js
                                                              └─ js/29-lecture-antenna.js
js/30-ia.js (Gemini) ─────────────────────────────────────────┘
```

- `pilote-antenna.js` déclare `antenna_lire` et `antenna_vue` à côté d'`antenna_proposer`.
- La capture : `Pilote.capture(fn)` reçoit une fonction qui rend la toile choisie par la
  dernière commande de vue (`carte` par défaut). `pilote.js` et `pilote.py` (copies
  vérifiées par la CI) ne changent pas.
- Vue : `antVuePoser`, `antBasculerVueMaillage`, `fit`, `ANT_ONGLET` + `antResultatsRendre`,
  `ANT_BAL_POINT`, `ANT.etape` + `antAssistantRendre`, `wsShow`.

## Commandes

```
Bancs :  node WEB_ANTENNA/test/banc-interface.js ; python3 WEB_ANTENNA/python/test/banc-openems.py
MCP :    python3 lanceur/banc_mcp.py ; node lanceur/banc_pilote.js
```

## Stratégie de test

- `banc-interface.js` : relevé des sorties avant le déplacement, puis égalité ; chaque
  `quoi` ; bornes ; aucune écriture (instantané d'`ANT`/`CON`/`PRJ` avant et après) ;
  pièges sur les fonctions de lancement.
- `banc_mcp.py` : outils, pages, arguments refusés.
- Essai réel dans Chromium sur une copie de PROJETS : chaque lecture, les trois captures.

## Limites

- **Toujours** : lire sans écrire ; garder les sorties des registres identiques.
- **Demander d'abord** : changer `pilote.js` / `pilote.py` (copies dans trois dépôts).
- **Jamais** : lancer un calcul openEMS ; enregistrer le projet ; changer les préférences.

## Questions ouvertes

Aucune.
