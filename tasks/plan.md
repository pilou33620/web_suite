# Plan d'implémentation : canal-pilote

Spec : [lanceur/docs/SPEC-mcp-canal-pilote.md](../lanceur/docs/SPEC-mcp-canal-pilote.md) · Carte :
[lanceur/docs/SPEC-mcp.md](../lanceur/docs/SPEC-mcp.md) · Tâches : [todo.md](todo.md) ·
Modules précédents : [plan-mcp-noyau.md](plan-mcp-noyau.md), [plan-api-outils.md](plan-api-outils.md)

## Vue d'ensemble

Un canal entre le MCP et les pages ouvertes des trois outils, en trois pièces :
- **`pilote.py`**, côté serveur : file d'attente, long-poll, attente de la réponse ;
- **`pilote.js` et `pilote.css`**, côté page : onglet relais, registre des commandes,
  panneau « Claude » avec ses cartes et l'interrupteur ;
- **`lanceur/mcp_pilote.py`**, côté MCP : les outils nommés.

Il est livré avec ses premières commandes : onglets, capture, calculs WEB_CAO, valeur d'un
composant du schéma, cartes ANTENNA.

## Décisions d'architecture

- **Originaux dans le lanceur, copies dans les outils**, comme `appairage.py` :
  `lanceur/pilote.py`, `lanceur/pilote.js` et `lanceur/pilote.css` sont les originaux,
  recopiés dans WEB_CAO, WEB_ANTENNA et WEB_3D. La CI de web_suite vérifie que les copies
  sont identiques.
- **Élection de l'onglet relais par BroadcastChannel**, et non par `navigator.locks` comme
  l'écrivait la spec. Web Locks et `crypto.randomUUID` n'existent qu'en contexte sécurisé
  (HTTPS ou localhost), alors que l'iPad ouvre `http://<adresse du Pi>`. La tâche 1 le
  confirme, puis la spec est corrigée.
  - Le mécanisme : chaque onglet annonce sa présence toutes les 2 s ; le relais est le plus
    petit id parmi les onglets entendus depuis moins de 5 s. Un seul mécanisme, qui marche
    partout, et qui se teste sous Node (BroadcastChannel existe depuis Node 18).
- **`pilote.py` sans serveur à lui** : une fonction `traiter(handler)` que chaque serveur
  appelle en tête de son aiguillage. Elle rend True quand la route était à elle.
  L'auto-test démarre un petit `ThreadingHTTPServer`.
- **Attente côté serveur** : un `threading.Condition` par commande. `POST /commande` attend
  la réponse de la page jusqu'à `attente` secondes (120 par défaut, 1 dans l'auto-test),
  puis rend `{etat: "en_attente", id}`.
- **Le MCP attend un peu plus que le serveur** : il appelle avec `delai=130`.
  `appeler_outil_web()` a déjà ce paramètre.
- **Captures** : chaque page déclare `Pilote.capture(() => canvas)`. `pilote.js` réduit
  l'image à 1600 px de large, puis fait `toDataURL("image/png")`. Le noyau du MCP apprend à
  rendre `{"type": "image", "data", "mimeType"}`.
- **Calculs** : `cao_calcul` choisit l'analyse (`simAllerFamille` / `simAllerAnalyse`),
  puis fait `await` sur la fonction de lancement (toutes sont `async`, vu à la tâche 1).
  Le résultat rendu est `simRapMarkdown(simRapCollecter())`, qui contient les erreurs
  rangées dans l'état.
- **Annuler** : porté par `pilote.js`. Chaque écriture déclare `etat()`, et la carte
  compare son instantané avant d'annuler. `iaAnnuler` d'ANTENNA ne vérifie rien (tâche 1).

## Graphe de dépendances

```
T1 vérifications ──▶ T2 pilote.py (serveur + auto-test)
                          │
                          ├──▶ T3 MCP : mcp_pilote.py + image dans le noyau + banc_mcp
                          └──▶ T4 pilote.js : relais, long-poll, registre (+ banc Node)
                                    │
                                    └──▶ T5 panneau, cartes, interrupteur, annuler
                                              │
             ┌───────────────┬───────────────┼───────────────┐
             ▼               ▼               ▼               ▼
      T6 WEB_CAO : branchement, captures   T9 WEB_ANTENNA   T10 WEB_3D
             │
       ┌─────┴─────┐
       ▼           ▼
   T7 cao_calcul  T8 cao_valeur_composant
                                     ▼
                     T11 CI des copies + docs ──▶ T12 essai réel (Playwright) + revue
```

## Liste des tâches

### Phase 1 : risques et socle serveur
- [x] T1 : vérifications en lecture seule (fin des calculs, API du schéma, API des cartes
      ANTENNA, contexte non sécurisé)
- [x] T2 : `lanceur/pilote.py` et son auto-test
- [x] T3 : `lanceur/mcp_pilote.py`, contenu image dans le noyau, `banc_mcp.py`

### Checkpoint : socle
- [x] Bancs du lanceur verts ; revue avec toi

### Phase 2 : la page
- [x] T4 : `pilote.js` : id d'onglet, élection du relais, long-poll, registre, réponses
- [x] T5 : panneau « Claude », cartes, interrupteur, annuler si rien n'a bougé

### Checkpoint : canal complet
- [x] Banc Node vert ; revue avec toi

### Phase 3 : les outils
- [x] T6 : WEB_CAO : copies, route, pages (schéma, PCB, gestion-lib, visionneuse), captures, monofichiers
- [x] T7 : WEB_CAO : `cao_calcul`
- [x] T8 : WEB_CAO : `cao_valeur_composant`
- [x] T9 : WEB_ANTENNA : copies, route, capture 2D, `antenna_proposer`
- [x] T10 : WEB_3D : copies, route, capture

### Checkpoint : outils branchés
- [x] Bancs de chaque dépôt verts ; revue avec toi

### Phase 4 : fin
- [x] T11 : CI de web_suite (copies de `pilote.*`), documentation
- [x] T12 : essai réel sur le Pi (Chromium et Playwright), puis `/review`

### Checkpoint : terminé
- [x] Critères de réussite de la spec remplis (sauf l'essai iPad)
- [ ] Essai iPad avec toi (suivi dans A-FAIRE.md, Essais réels en attente)

## Risques et parades

| Risque | Impact | Parade |
|---|---|---|
| `navigator.locks` et `crypto.randomUUID` absents sur l'iPad (HTTP non sécurisé) | Élevé : pas de relais | Élection par BroadcastChannel et id par `Math.random`, confirmés à T1 |
| `sim*Go()` sans promesse : on ne sait pas quand le calcul finit | Moyen | T1 ; sinon, observer `SIM.res` et ses voisins, avec un délai |
| Les monofichiers `dist/` changent dès qu'on ajoute un script | Moyen : CI WEB_CAO rouge | Reconstruire dans la même tâche (T6) et vérifier la reproductibilité comme la CI |
| Relais fermé pendant une attente | Moyen | La commande reste dans la file du serveur ; le nouveau relais la reprend. Testé en T4, puis en réel en T12 |
| `innerHTML` avec un texte venu de Claude dans les cartes | Élevé : XSS | Cartes construites par `textContent` et `createElement` seulement ; vérifié par le banc Node |
| Quatre dépôts touchés, commits croisés | Moyen | Un commit par dépôt et par tâche ; le lanceur d'abord, les outils ensuite ; la CI des copies en dernier (T11) pour ne pas rester rouge entre deux tâches |
| Deux origines différentes pour un même outil (`localhost` et l'adresse IP) | Faible | Chacune a son relais : deux navigateurs logiques. C'est accepté |
| Chromium headless se bloquait sur le Pi | Résolu | La cause est D-Bus : lancer Chromium avec `DBUS_SESSION_BUS_ADDRESS=disabled:`. Pour T12, piloter Chromium par `--remote-debugging-port` (CDP) depuis Node 20 (`node --experimental-websocket`) ; Playwright n'est pas nécessaire |

## Questions ouvertes

Aucune. La tâche 1 a tranché l'attente de `cao_calcul` et l'élection du relais
(BroadcastChannel) ; la spec est corrigée (`tasks/todo.md`, tâches 1 et 7).
