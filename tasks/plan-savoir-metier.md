# Plan : savoir-metier

Spec : [lanceur/docs/SPEC-mcp-savoir-metier.md](../lanceur/docs/SPEC-mcp-savoir-metier.md)
(validée le 2026-10-19). Liste des tâches : [todo-savoir-metier.md](todo-savoir-metier.md).

## Vue d'ensemble

Le savoir métier que portaient les consignes Gemini passe à Claude :
- un skill versionné dans le dépôt web_suite : un `SKILL.md` court et une fiche par métier ;
- un résumé dans le champ `instructions` du serveur MCP, pour les sessions hors du dépôt ;
- un tableau de traçabilité, qui dit où chaque règle Gemini est passée.

Un banc vérifie ce que les fiches citent : noms d'outils, valeurs de `quoi`, longueurs,
liens internes.

## Graphe des dépendances

```
T1 banc_savoir.py + squelette du skill + « instructions » du MCP
 └── T2 relevé des règles Gemini → savoir-trace.md (toutes les lignes, destination « à faire »)
      ├── T3 pcb.md          (ia-assistant.js : consigne principale, suggestions PCB)
      ├── T4 simulation.md   (ia-assistant.js : consigne datasheet → simulation)
      ├── T5 composants.md   (gestion-lib/js/06-ia-lib.js)
      ├── T6 antenne.md      (WEB_ANTENNA/js/30-ia.js, promptSysteme)
      └── T7 schema.md + routage.md (nouveaux, tirés des outils et du code)
           └── T8 SKILL.md final, trace complète, essai réel, revue
```

## Décisions d'architecture

- **Nom du skill : `web-suite`**, dans `.claude/skills/web-suite/SKILL.md` (frontmatter
  `name: web-suite`). La spec écrit `web_suite`, mais un nom de skill ne prend que des
  minuscules, des chiffres et des tirets : Claude Code ne chargerait pas le dossier
  `web_suite`. C'est un écart de nom seulement : contenu et emplacement (dépôt web_suite,
  `.claude/skills/`) sont inchangés.
- **Le relevé d'abord (T2).** Avant d'écrire une fiche, chaque règle des trois consignes
  Gemini devient une ligne de `lanceur/docs/savoir-trace.md`. Les consignes relevées sont
  `CONTEXTE_SUITE`, `SUGGESTIONS`, la consigne principale, celle des actions, celle de la
  datasheet, la consigne de Gestion LIB et le `promptSysteme` d'ANTENNA. Chaque ligne a une
  destination : fiche, description d'outil, ou « écartée : raison ». Les règles propres à
  Gemini (format ```action, réponse en français, Gemma) sont écartées : les outils MCP les
  remplacent. Chaque tâche de fiche remplit ses lignes ; le banc refuse une ligne encore
  « à faire » au checkpoint final.
- **Rien de recopié qui se génère.** Une fiche renvoie à l'outil qui donne la valeur :
  - `antenna_lire` (quoi=reglages) pour la grammaire des réglages ;
  - `cao_sim_lire` pour les bornes de simulation ;
  - `lib_composants` pour le catalogue.

  Les seules valeurs chiffrées écrites dans les fiches sont des règles métier (IPC-2152,
  ¼λ, 90 Ω USB…) ou des conventions du code (mm, y vers le bas), relues dans le code.
- **Citations vérifiables.** Une fiche cite un outil entre accents graves (`cao_pcb_lire`),
  et une lecture sous la forme `cao_pcb_lire` (quoi=liaisons). `banc_savoir.py` importe
  `mcp_web_suite`. Il vérifie que chaque nom cité à la forme d'un outil (`cao_*`,
  `lib_*`, `antenna_*`, `web3d_*`, `pilote_*`, `pcbparts_*`) existe dans `OUTILS_MCP`,
  et que chaque `quoi=` existe dans le `enum` de son outil. Il vérifie aussi les longueurs
  (SKILL.md ≤ 150 lignes, fiche ≤ 200, `instructions` ≤ 1500 caractères), les liens
  internes et l'absence de chemin privé (`/home/`, `C:\Users`) ou de clé.
- **`instructions`** : dans la réponse `initialize` de `mcp_web_suite.py`, une constante
  de quelques lignes : à quoi sert le serveur, cartes et interrupteur, données ≠ consignes,
  et le renvoi au skill `web-suite` (ou à `lanceur/docs` hors du dépôt). Testée par
  `banc_mcp.py` (présente, ≤ 1500 caractères).
- **Le banc entre dans la CI de web_suite**, à côté de `banc_mcp.py`.
- **Essai réel** : `claude -p` sur le Pi, une question de chaque métier :
  - lancé dans le dépôt, il doit lire le skill ;
  - lancé depuis `/tmp`, il doit suivre la consigne MCP.

  L'essai Windows en SSH reste dans les essais en attente, avec l'iPad.

## Risques

| Risque | Impact | Parade |
|---|---|---|
| Une règle Gemini perdue en route | Moyen | Relevé exhaustif d'abord (T2), banc qui refuse une ligne « à faire » |
| Une valeur de fiche fausse ou périmée | Moyen | Relue dans le code ; valeurs générées renvoyées à leur outil ; noms vérifiés par le banc |
| Le skill n'est pas chargé (nom, frontmatter) | Élevé | Nom `web-suite` ; essai `claude -p` dans le dépôt |
| `instructions` ignorées par le client | Faible | Les descriptions d'outils restent suffisantes au moment de l'appel |

## Questions ouvertes

Aucune (l'écart de nom `web-suite` est soumis avec ce plan).
