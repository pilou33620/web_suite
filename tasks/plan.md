# Plan d'implémentation : api-outils

Spec : [lanceur/docs/SPEC-mcp-api-outils.md](../lanceur/docs/SPEC-mcp-api-outils.md) · Carte :
[lanceur/docs/SPEC-mcp.md](../lanceur/docs/SPEC-mcp.md) · Tâches : [todo.md](todo.md) ·
Module précédent : [plan-mcp-noyau.md](plan-mcp-noyau.md) (sa tâche 4, l'essai réel, reste ouverte)

## Vue d'ensemble

On ajoute 23 outils MCP nommés au serveur `lanceur/mcp_web_suite.py`. Ils lisent les
projets, la LIB, les profils, pcbparts et openEMS, et écrivent projets, LIB et profils avec
des garde-fous. Tous passent par `appeler_outil_web()`. Le code est dans un fichier voisin,
`lanceur/mcp_api_outils.py`, et les tests vont dans `lanceur/banc_mcp.py`.

## Décisions d'architecture

- **Un fichier voisin** : `mcp_api_outils.py` définit `OUTILS_API` (même format que
  `OUTILS_MCP`), et `mcp_web_suite.py` l'ajoute à `OUTILS_MCP`. L'import se fait dans ce sens
  seulement : `mcp_api_outils` importe `appeler_outil_web` et `ErreurOutil` du noyau, en
  import tardif dans les fonctions pour éviter l'import circulaire, ou en recevant le noyau
  en paramètre. On tranche à la tâche 2 en prenant le plus court.
- **Deux aides seulement** :
  - `exiger(args, **types)` : présence et type des arguments, avec une erreur qui nomme
    l'argument fautif ;
  - `resumer(valeur)` : au-delà de 100 Ko de JSON, rend la taille, les clés avec leur type
    et la longueur des listes, un extrait et l'indication pour demander une partie.
  `appeler()` du noyau transforme déjà toute exception en résultat `isError` : il n'y a pas
  de nouvelle gestion d'erreur.
- **`methode` dans `appeler_outil_web()`** : `GET` par défaut, `POST` quand il y a un corps
  (comme aujourd'hui), `PUT` sur demande, `DELETE` refusé. Le lanceur reste en GET seulement.
- **Garde-fous dans l'outil MCP, avant l'écriture** : ils lisent d'abord, par
  `/api/lib/fichiers`, `/api/profils`, `/api/projet` ou `/api/tools`, puis refusent ou
  écrivent. Il reste une fenêtre de course entre la lecture et l'écriture ; elle est
  acceptée, car c'est un seul utilisateur sur son poste.
- **Le banc grandit sans changer de forme** : un faux outil `FauxApi` répond aux routes de
  la table avec une réponse prévue et note `(méthode, chemin, corps)` dans une liste. Le banc
  vérifie cette liste après chaque appel.

## Graphe de dépendances

```
appeler_outil_web() du noyau (existe)
        │
        ▼
Tâche 2 : méthode PUT + exiger() + resumer() + 1re lecture CAO   ◀── Tâche 1 : vérifications (lecture)
        │
        ├──▶ Tâche 3 : autres lectures CAO, LIB, 3D
        │         │
        │         └──▶ Tâche 4 : écritures CAO gardées (doc, profil, LIB)
        ├──▶ Tâche 5 : pcbparts
        ├──▶ Tâche 6 : projets ANTENNA
        └──▶ Tâche 7 : openEMS (chiffrer, suivre)
                  │
                  ▼
        Tâche 8 : doc + essai réel
```

Les tâches 3, 5, 6 et 7 sont indépendantes une fois la tâche 2 faite. Elles touchent
toutes les deux mêmes fichiers : elles restent donc séquentielles.

## Liste des tâches

### Phase 1 : risques levés, socle

- [ ] Tâche 1 : vérifier comment l'éditeur enregistre et ce que fait `_pr_enregistrer` (lecture seule)
- [ ] Tâche 2 : `methode`, `exiger()`, `resumer()`, puis `cao_projets`, `cao_projet_lire` et `cao_doc_lire`

### Checkpoint : socle
- [ ] Tous les bancs du lanceur verts ; revue avec toi

### Phase 2 : outils

- [ ] Tâche 3 : lectures CAO, LIB, WEB_3D
- [ ] Tâche 4 : écritures CAO gardées
- [ ] Tâche 5 : pcbparts par le relais WEB_CAO

### Checkpoint : WEB_CAO complet
- [ ] Bancs verts ; revue avec toi

- [ ] Tâche 6 : projets WEB_ANTENNA
- [ ] Tâche 7 : openEMS, chiffrer et suivre

### Phase 3 : fin

- [ ] Tâche 8 : documentation, puis essai réel avec Claude Code

### Checkpoint : terminé
- [ ] Critères de réussite de la spec remplis ; `/review` fait

## Risques et parades

| Risque | Impact | Parade |
|---|---|---|
| L'éditeur écrase le fichier si Claude l'a écrit pendant que la page l'avait ouvert | Élevé : perte de travail | Tâche 1 en tête. Si la page écrase, `cao_doc_ecrire` sort du module et attend `canal-pilote`. |
| `banc_mcp.py` devient long (271 lignes aujourd'hui) | Faible | Un faux outil générique piloté par une table de réponses, pas une classe par route. |
| Import circulaire noyau ↔ `mcp_api_outils` | Faible | Tranché à la tâche 2 : import tardif, ou dict construit dans le noyau. |
| Résumé encore trop gros (liste de 60 000 nœuds) | Moyen | Pour une liste, `resumer` ne rend que la longueur et les deux premiers éléments ; il est vérifié sur une réponse de 300 Ko. |
| Le catalogue LIB entier est relu à chaque `lib_composants` | Faible | Accepté : lecture locale. `shortcut:` à poser si c'est lent sur le Pi. |

## Questions ouvertes

Aucune : les quatre questions de la spec sont tranchées. La tâche 1 peut retirer
`cao_doc_ecrire` du module, et limiter `antenna_projet_enregistrer` au projet ouvert.
