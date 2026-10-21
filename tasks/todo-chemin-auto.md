# Tâches : chemin-auto

Spec : [lanceur/docs/SPEC-mcp-chemin-auto.md](../lanceur/docs/SPEC-mcp-chemin-auto.md) ·
Carte : [lanceur/docs/SPEC-mcp.md](../lanceur/docs/SPEC-mcp.md)

## Tâche 1 : ✅ chercheur A* dans la page PCB

`WEB_CAO/editeur-pcb/js/40-chemin-auto.js` : grille par couche, obstacles gonflés (`clrK`),
bord, découpes, trous, vias libres sur toutes les couches traversées ; points de passage au
format de `cao_router`. Commande `cao_chemin` (lecture) avec celles du routage.

- [x] Exemple 1 dérouté : quatre chemins, posés par `cao_router`
- [x] Mur sur le dessus : deux vias de part et d'autre ; mur partout : « aucun chemin »
- [x] Rien n'est écrit ; refus par liaison et refus de l'appel

**Vérification :** `node editeur-pcb/test/harness.js`

## Tâche 2 : ✅ outil MCP `cao_chemin`

`lanceur/mcp_routage.py`, banc `lanceur/banc_mcp.py` (71 outils), skill (`routage.md`), DETAILS.

## Checkpoint

- [x] Bancs verts (harnais PCB, banc-pilote-cao, banc_mcp, banc_savoir)
- [ ] Essai réel : Claude route une petite carte d'une copie de PROJETS en partant de
      `cao_chemin`, sur le Pi (durée d'un appel de 20 liaisons)
- [ ] Revue avec toi des choix pris d'office (coût d'une via, d'un virage, pas par défaut)
