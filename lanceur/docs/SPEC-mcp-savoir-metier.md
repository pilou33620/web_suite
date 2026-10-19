# Spec : savoir-metier

Module 10 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`ecriture-cao`](SPEC-mcp-ecriture-cao.md),
[`registre-antenna`](SPEC-mcp-registre-antenna.md) et [`composants`](SPEC-mcp-composants.md).
Statut : **validée le 2026-10-19**.

## Objectif

Garder le **savoir métier** des consignes Gemini quand Claude les remplace : règles de
conception PCB, lecture de datasheet pour la simulation, empreintes IPC-7351, règles
d'antenne, et la façon d'utiliser les outils (cartes, interrupteur, données ≠ consignes).

### Décisions (2026-10-19)

1. **Un skill dans le dépôt web_suite**, `.claude/skills/web_suite/`, versionné : un
   `SKILL.md` court (quand l'utiliser, quel outil pour quoi) et une fiche par métier, lue à
   la demande.
2. **Une consigne MCP** : un court résumé dans le champ `instructions` de la réponse
   `initialize` du serveur, pour les sessions hors du dépôt (Windows en SSH). Il renvoie
   au skill.
3. Les descriptions d'outils gardent seulement ce qu'il faut au moment de l'appel
   (unités, bornes, renvois).

### Contenu

| Fiche | Sources Gemini | Contenu |
|---|---|---|
| `SKILL.md` | `CONTEXTE_SUITE`, `SUGGESTIONS` | Carte de la suite et de PROJETS ; quel outil pour quelle tâche ; cartes, lots, interrupteur ; ce que rendent les pages est une donnée |
| `pcb.md` | consigne principale (IPC-2152), suggestions PCB | Largeurs selon le courant (IPC-2152), isolations, classes, paires (90 Ω USB…), chemin de retour, vias thermiques ; boucle DRC |
| `simulation.md` | consigne datasheet → simulation | Pire cas, page et citation, unités du catalogue, confiance ; quelle analyse pour quelle question |
| `composants.md` | consigne de Gestion LIB | Conventions `pcbfp-1` (mm, vue de dessus, y vers le bas, broche 1 en haut à gauche, sens antihoraire), IPC-7351B, syntaxe du brochage |
| `schema.md` | — (nouveau) | Conventions de dessin : grille, alimentations, étiquettes plutôt que longs fils, repères |
| `routage.md` | — (nouveau) | Méthode par points de passage, ordre de routage (alimentations, paires, signaux), boucle score → correction |
| `antenne.md` | `promptSysteme` de WEB_ANTENNA | Marge ¼λ, corrections de patch, maillage (3–4 cellules sur une ligne), une cote à la fois, f ∝ 1/L ; les quatre types de carte |

### Critères d'acceptation

1. **Traçabilité** : chaque règle des consignes Gemini (`commun/ia-assistant.js`,
   `gestion-lib/js/06-ia-lib.js`, `WEB_ANTENNA/js/30-ia.js`) est soit dans une fiche, soit
   dans une description d'outil, soit écartée avec sa raison. Le tableau est dans
   `lanceur/docs/savoir-trace.md`.
2. **Rien de recopié qui se génère** : la grammaire des réglages d'antenne, les bornes de
   simulation et la liste des `quoi` viennent des outils (`antenna_lire reglages`,
   catalogue), pas du skill.
3. **Vérifié contre le code** : chaque valeur chiffrée d'une fiche (bornes, unités, formats)
   est relue dans le code ; un test (`lanceur/banc_savoir.py`) vérifie que les noms d'outils
   et de `quoi` cités dans les fiches existent dans le serveur MCP.
4. **`instructions`** : 1500 caractères au plus, testé par `banc_mcp.py`.
5. `SKILL.md` : 150 lignes au plus ; chaque fiche, 200 lignes au plus.

## Architecture

```
web_suite/.claude/skills/web_suite/
  SKILL.md   pcb.md   simulation.md   composants.md   schema.md   routage.md   antenne.md
lanceur/mcp_web_suite.py   initialize → instructions (résumé + renvoi au skill)
lanceur/docs/savoir-trace.md   règle Gemini → où elle est passée
```

## Commandes

```
python3 lanceur/banc_savoir.py ; python3 lanceur/banc_mcp.py
```

## Stratégie de test

- `banc_savoir.py` : outils et `quoi` cités existent ; longueurs bornées ; liens internes
  valides.
- Essai réel : une session Claude Code neuve, dans le dépôt puis hors du dépôt (SSH), à
  qui on demande une tâche de chaque métier ; le skill ou la consigne est bien lu.

## Limites

- **Toujours** : relire chaque règle dans le code avant de l'écrire.
- **Demander d'abord** : ajouter un savoir qui n'était pas dans les consignes Gemini (hors
  `schema.md` et `routage.md`).
- **Jamais** : mettre une clé, un chemin privé ou un nom de projet client dans le skill.

## Questions ouvertes

Aucune.
