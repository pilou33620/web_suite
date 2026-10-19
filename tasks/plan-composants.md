# Plan : composants

Spec : [lanceur/docs/SPEC-mcp-composants.md](../lanceur/docs/SPEC-mcp-composants.md)
(validée le 2026-10-19). Liste des tâches : [todo-composants.md](todo-composants.md).

## Vue d'ensemble

Claude crée des composants dans la LIB : empreinte PCB, ligne de catalogue, brochage. Le
symbole est un symbole intégré du schéma. L'empreinte vient d'un générateur IPC-7351,
d'un `pcbfp-1` écrit par Claude ou d'un fichier KiCad converti. Tout est contrôlé par le
code, montré sur une carte dans Gestion LIB et écrit au clic, sans jamais écraser.

## Graphe des dépendances

```
T1 serveur : PUT /api/lib/fichier « si_absent », POST /api/lib/composants « ajouter »
T2 commun/composants-lib.js : contrôles (empreinte, ligne de catalogue, brochage)
 ├── T3 générateurs IPC-7351 (chip, SOT, SOIC/TSSOP, QFN/DFN, DIP, barrette)
 └── T4 conversion KiCad (kicad_mod → pcbfp-1)
      └── T5 commandes de Gestion LIB (4 calculs + lib_composant_creer, carte)  ← T1, T3
           └── T6 outils MCP (lanceur/mcp_composants.py)
                └── T7 essai réel, puis revue
```

## Décisions d'architecture

- **Serveur d'abord** (T1). Le serveur rend lui-même l'écrasement impossible, au lieu de
  laisser le MCP « lire puis écrire », ce qui laisse une course :
  - `PUT /api/lib/fichier` avec `si_absent: true` rend 409 si le fichier existe ;
  - `POST /api/lib/composants` avec `ajouter: {ligne}` relit le CSV, refuse une Part Name
    déjà présente (409, sans tenir compte de la casse), puis **ajoute la ligne au texte
    existant**, dans l'encodage du fichier (utf-8, latin1, cp1252, comme la lecture) et
    avec ses fins de ligne. Les autres lignes ne changent pas d'un octet. Une colonne
    `Brochage` absente de l'en-tête y est ajoutée, comme le fait déjà la page. Le
    caractère qui ne passe pas dans l'encodage est refusé. Comme l'écriture actuelle,
    l'ajout se recopie dans `WEB_CAO/LIB_composants.csv`.
  - `lib_fichier_ecrire` (MCP) passe à `si_absent`.
- **`commun/composants-lib.js`** (global `COMPOSANTS_LIB`) : un script classique, sans DOM,
  éprouvé par un banc Node. Il contient :
  - les contrôles : `pcbfp-1`, ligne de catalogue, brochage ;
  - les générateurs ;
  - la conversion KiCad.

  Gestion LIB le charge, et son éditeur s'en sert aussi avant d'enregistrer : il ne
  contrôle rien aujourd'hui.
- **Contrôles de l'empreinte** : ceux de la spec. On se cale sur `normFpDef`
  (`editeur-pcb/js/08-empreinte.js`), qui lit les empreintes au PCB. Une empreinte que
  `normFpDef` déformerait est refusée.
- **Symboles intégrés** : Gestion LIB ne charge pas le schéma. `composants-lib.js` porte
  donc la liste des types du schéma, et le banc vérifie qu'elle est égale aux clés de `LIB`
  (`editeur-schematique/js/02-bibliotheque.js`).
- **Générateurs** : formules IPC-7351B, densité N par défaut (M, L au choix) :
  - Z = Lmin + 2·Jt + √(CL² + F² + P²) ;
  - G = Smax − 2·Jh − √(CS² + F² + P²) ;
  - X = Wmin + 2·Js + √(CW² + F² + P²).

  Les constantes de soudure (Jt, Jh, Js) par famille et par densité sont dans une table
  commentée, avec F = 0,10 et P = 0,05 mm.

  Les tests comparent :
  - des valeurs calculées à la main pour chaque famille, au centième ;
  - les empreintes déjà dans la LIB (0603, SOT-23, SOIC-8), au dixième.
- **Conversion KiCad** : un lecteur d'expressions S minimal, pour `pad`, `at`, `size`,
  `drill`, `layers` et `fp_line` (contour). Les formes rect, roundrect, circle et oval
  deviennent rect, circ et oval. Le repère KiCad (y vers le bas) est celui de `pcbfp-1`.
  Ce que le lecteur ne connaît pas est listé dans les remarques, jamais ignoré en silence.
- **Commandes de Gestion LIB**, déclarées par `piloteCao("gestion-lib")` :
  - quatre calculs : `lib_empreinte_generer`, `lib_empreinte_verifier`,
    `lib_kicad_convertir` et `lib_brochage_verifier` ;
  - une carte, `lib_composant_creer`, marquée `irreversible` (la LIB n'a pas d'historique)
    et qui n'est jamais appliquée sans clic. La carte montre l'empreinte, la ligne et les
    remarques, et la page ouvre l'aperçu de l'empreinte. Au clic, la page écrit d'abord
    l'empreinte (`si_absent`), puis ajoute la ligne. Si la ligne est refusée, l'empreinte
    écrite est retirée, pour qu'il ne reste pas d'orphelin.
- **Les générateurs de Gestion LIB** qui écrivent `style: "soic"` ou `"header"` (inconnus
  du PCB) passent à `sop` et `row`.

## Risques

| Risque | Impact | Parade |
|---|---|---|
| Constantes IPC-7351B mal recopiées | Moyen | Table commentée par famille ; comparaison avec les empreintes de la LIB |
| Encodage du CSV (latin1) cassé par un ajout | Élevé | Ajout dans l'encodage détecté ; caractère hors encodage refusé ; test octet pour octet |
| Interrupteur allumé : écriture sans clic dans la LIB partagée | Élevé | `lib_composant_creer` est `irreversible` ET demande toujours le clic (garde dans la commande) |
| Un `kicad_mod` exotique (arcs, pastilles personnalisées) | Moyen | Remarques explicites ; pastille inconnue = refus |
| Gestion LIB pas ouverte | Faible | Refus du canal (« aucun onglet ») avec le conseil d'ouvrir Gestion LIB |

## Questions ouvertes

Aucune.
