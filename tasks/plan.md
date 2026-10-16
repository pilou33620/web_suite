# Plan d'implémentation : mcp-noyau

Spec : [lanceur/docs/SPEC-mcp-noyau.md](../lanceur/docs/SPEC-mcp-noyau.md) · Carte :
[lanceur/docs/SPEC-mcp.md](../lanceur/docs/SPEC-mcp.md) · Tâches : [todo.md](todo.md)

## Vue d'ensemble

Un serveur MCP stdio en Python standard (`lanceur/mcp_web_suite.py`) que Claude Code lance
lui-même, sur le poste ou par SSH. Il a un seul outil, `etat` (lanceur et trois outils :
lancé ou non, port, adresse). Il fournit `appeler_outil_web()`, la seule porte vers les
outils pour les modules suivants : boucle locale, ports du catalogue, routes interdites
refusées. Un banc (`lanceur/banc_mcp.py`) le vérifie en CI.

## Décisions d'architecture

- **Ports lus, jamais recopiés** : `outils.CATALOGUE` (8000, 8732, 8139) et
  `web_suite.PORT_DEFAUT` (8100). Vérifié : importer `outils` et `web_suite` n'écrit rien
  sur stdout, ce qui est indispensable puisque stdout porte le protocole.
- **Remplacement des ports pour le banc** : une variable d'environnement
  (`WEB_SUITE_MCP_PORTS`, JSON `{"web_cao": 54321, ...}`), même idée que
  `WEB_TOOLS_JETON_FICHIER`.
- **« Lancé » = `GET /` en boucle locale qui répond en HTTP** (n'importe quel statut), avec
  un délai court (1 s), pour que `etat` reste rapide même quand tout est arrêté. Depuis la
  boucle locale, aucun jeton n'est demandé (`Appairage.ce_poste`, `lanceur/appairage.py:223`).
- **Écriture sur `sys.stdout.buffer`** en UTF-8 avec `\n` seul : sous Windows, le mode texte
  ajouterait `\r\n` et un encodage local. Lecture sur `sys.stdin.buffer`, ligne par ligne.
- **Chemin absolu** : `sys.path` reçoit le dossier du script, car Claude Code lance le MCP
  depuis un dossier courant quelconque.
- **Routes interdites dans `appeler_outil_web()`**, pas dans chaque outil MCP : un module
  futur ne peut pas les ouvrir par oubli.

## Graphe de dépendances

```
outils.CATALOGUE, web_suite.PORT_DEFAUT  (existants)
        │
        ▼
Tâche 1 : boucle MCP stdio (protocole, erreurs)  ──▶  banc : protocole
        │
        ▼
Tâche 2 : appeler_outil_web + outil etat          ──▶  banc : faux outil, refus
        │
        ▼
Tâche 3 : CI et documentation
        │
        ▼
Tâche 4 : essai réel (Pi, puis SSH depuis Windows)
```

## Liste des tâches

### Phase 1 : socle
- [x] Tâche 1 : le MCP répond au protocole (initialize, ping, tools/list, erreurs)
- [x] Tâche 2 : l'outil `etat` et la porte `appeler_outil_web()`

### Checkpoint : socle
- [x] `python lanceur/banc_mcp.py` vert
- [x] Les bancs existants restent verts (`banc_serveur.py`, `appairage.py`)

### Phase 2 : livraison
- [x] Tâche 3 : étape CI et section « Claude Code (MCP) » dans la documentation
- [ ] Tâche 4 : essai réel avec Claude Code, sur le Pi puis par SSH depuis Windows

### Checkpoint : terminé
- [ ] Tous les critères de réussite de la spec sont remplis
- [x] `/review` fait avant de déclarer terminé (règle CLAUDE.md)

## Risques et parades

| Risque | Impact | Parade |
|---|---|---|
| Un `.bashrc` du Pi écrit sur stdout lors d'une commande SSH et casse le protocole | Haut | Documenter `ssh -T` ; l'essai de la tâche 4 le révèle ; si besoin, lancer le script avec un chemin absolu vers python3 et sans shell interactif |
| SSH demande un mot de passe : stdin porte le protocole, la saisie est impossible | Haut | Clé SSH obligatoire, à documenter (`ssh-keygen` puis `ssh-copy-id` ou équivalent Windows) |
| Encodage et fins de ligne sous Windows | Moyen | Écriture en octets UTF-8 et `\n` (voir les décisions) ; banc exécuté aussi sous Windows à la main |
| `etat` lent quand rien ne tourne (4 délais d'attente) | Faible | Délai de 1 s par port, sondes en parallèle (threads) si besoin |
| Python 3.8 sur un poste ancien | Faible | Aucune syntaxe après 3.8 ; la CI tourne en 3.12, relecture de la syntaxe dans `/review` |

## Questions ouvertes

Aucune : les trois questions de la spec sont tranchées.
