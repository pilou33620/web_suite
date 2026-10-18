# Spec : canal-pilote

Module 3 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`mcp-noyau`](SPEC-mcp-noyau.md).
Statut : **validée le 2026-10-18**.

## Objectif

Permettre à Claude d'agir dans les **pages ouvertes** de WEB_CAO, WEB_ANTENNA et WEB_3D,
et pas seulement sur leurs serveurs :
- voir quels onglets sont ouverts ;
- capturer leur canevas ;
- lancer les calculs que seule la page sait préparer ;
- proposer des modifications que tu acceptes d'un clic.

Ce module pose le **canal** (serveur ↔ page), la **carte de proposition**, l'**undo** et
l'**interrupteur**, avec des premières commandes qui l'éprouvent. Les registres complets
viennent ensuite (`registre-3d`, `registre-cao`, `registre-antenna`) sur ce même canal.

Utilisateur : toi, depuis Claude Code (sur le poste ou par SSH). Les pages pilotées peuvent
être ouvertes sur le poste ou sur un appareil appairé (iPad…).

### Décisions (2026-10-18)

1. **Premières commandes** :
   - lister les onglets et capturer leur canevas ;
   - lancer les calculs WEB_CAO depuis la page ;
   - une écriture de démonstration : **la valeur d'un composant du schéma** ;
   - les cartes WEB_ANTENNA existantes : `reglages`, `cotes`, `balayage`, `formes`. Un
     balayage appliqué est seulement **armé**, jamais lancé (`WEB_ANTENNA/js/30-ia.js:1363-1390`).
2. **Ciblage** : par défaut, l'onglet du bon type utilisé le plus récemment (dernier
   focus) ; sinon l'onglet nommé par son id, tel que le rend la liste des onglets.
3. **Attente** : un appel qui propose attend ta décision **jusqu'à 2 min**. Au-delà, il rend
   « en attente (id) », et Claude suit la proposition avec `pilote_suivre`.
4. **Appareils** : tous les onglets se branchent, ceux du poste comme ceux d'un appareil
   appairé. La liste indique l'appareil.
5. **Interface** : un panneau « Claude » commun aux trois outils (`pilote.js` et
   `pilote.css`, recopiés à l'identique). Il affiche les cartes avant → après avec les
   boutons Appliquer / Refuser / Annuler, plus l'interrupteur.
6. **Interrupteur « appliquer directement »** : **un par onglet**, toujours **éteint** au
   chargement de la page.
7. **Undo** : une commande d'écriture = une étape annulable.
   - Si l'éditeur a un historique, la commande y pousse **une** étape.
   - Sinon, elle garde l'état d'avant, et la carte offre « Annuler ».
   - Une écriture qui n'a ni l'un ni l'autre est **refusée** à l'enregistrement de la
     commande.
8. **Connexions** : par outil et par navigateur, **un seul onglet relais** tient le
   long-poll et relaie aux autres par BroadcastChannel. Il est élu **par BroadcastChannel**
   (corrigé à la tâche 1, voir plus bas) : chaque onglet s'annonce toutes les 2 s, et le
   relais est le plus petit id parmi les onglets entendus depuis moins de 5 s.

### Critères d'acceptation

1. **`pilote_onglets`** rend, pour chaque outil qui tourne, ses onglets branchés : id,
   page (schema, pcb, gestion-lib, visionneuse-ipc, accueil, antenna, 3d), projet, appareil
   (ce poste ou l'adresse), dernier focus, interrupteur. Un onglet fermé disparaît en
   moins de 60 s.
2. **`pilote_capture`** rend une **image** (contenu MCP `image`, PNG) du canevas de
   l'onglet visé :
   - largeur réduite à 1600 px au plus ;
   - pages couvertes : schéma, PCB, visionneuse IPC, WEB_ANTENNA 2D, WEB_3D ;
   - une page sans canevas répond par une erreur lisible.
3. **`cao_calcul`** lance dans l'onglet PCB (ou visionneuse IPC) une analyse parmi
   `impedance`, `diff`, `retour`, `crosstalk`, `oeil`, `dc`, `pdn`, `s21`, `verif` :
   - avec les réglages du panneau et la sélection de l'éditeur ;
   - il rend le rapport de la simulation (`simRapMarkdown`), résumé au-delà de 100 Ko ;
   - au-delà de 2 min, il rend « en cours (id) » ;
   - lancer un calcul ne modifie pas le document : **pas de carte à valider**, mais le
     panneau affiche « Claude a lancé : … ».
4. **`cao_valeur_composant`** (schéma, par repère, par exemple `R12` → `4k7`) :
   - affiche une carte avant → après, et n'écrit **rien** avant ton clic, sauf si
     l'interrupteur de l'onglet est allumé ;
   - appliquée, elle fait **une** étape d'undo (Ctrl+Z la défait) ;
   - refusée, elle rend « refusée » à Claude.
5. **`antenna_proposer`** (`reglages`, `cotes`, `balayage`, `formes`, au format des cartes
   actuelles) passe par `iaProposition` puis `iaAppliquer`, et l'Annuler de la carte par
   `iaAnnuler`. Mêmes règles d'attente et d'interrupteur.
6. **`pilote_suivre`** rend l'état d'une commande : en attente, appliquée, refusée, finie
   (avec son résultat), échouée, ou expirée.
7. **Un seul long-poll par outil et par navigateur**, quel que soit le nombre d'onglets.
   Fermer l'onglet relais fait élire un autre onglet en moins de 5 s, sans perdre de
   commande en attente.
8. **Robustesse** : outil arrêté, aucun onglet branché, onglet visé fermé, page qui ne
   connaît pas la commande, délai dépassé — chaque cas donne un message lisible à Claude,
   jamais une exception.
9. **Sécurité** :
   - les routes qui donnent des ordres (`commande`, `onglets`, `suivre`) n'acceptent que la
     **boucle locale**, c'est-à-dire le MCP sur le poste ;
   - une page ne peut pas envoyer d'ordre à une autre page ;
   - les commandes sont des **noms d'un registre** de la page : jamais de code exécuté
     (`eval`, `Function`) ;
   - l'id d'une commande est aléatoire (`secrets.token_hex`).

## Architecture

```
Claude Code ──stdio── MCP (lanceur/mcp_pilote.py) ──HTTP 127.0.0.1──▶ serveur de l'outil (pilote.py)
                                                                              ▲   │ long-poll
                                                       onglet relais (pilote.js) ◀──┘
                                                              │ BroadcastChannel
                                                       autres onglets du même outil
```

**Serveur : `pilote.py`**, recopié à l'identique dans WEB_CAO, WEB_ANTENNA et WEB_3D, comme
`appairage.py`. L'état est en mémoire seulement : onglets, file d'attente par navigateur,
commandes en cours. Ses routes :

| Route | Qui | Rôle |
|---|---|---|
| `POST /api/pilote/battement` | page (relais) | « je suis le relais de ce navigateur, voici mes onglets » (id, page, projet, focus, interrupteur) ; toutes les 20 s et à chaque changement |
| `GET /api/pilote/attendre?navigateur=` | page (relais) | long-poll de 25 s : rend les commandes destinées aux onglets de ce navigateur |
| `POST /api/pilote/reponse` | page | état et résultat d'une commande (par son id) |
| `POST /api/pilote/commande` | MCP, **boucle locale** | dépose une commande `{onglet?, page, nom, args, ecriture}` et attend la réponse (jusqu'à 2 min) |
| `GET /api/pilote/onglets` | MCP, **boucle locale** | la liste des onglets branchés |
| `GET /api/pilote/suivre?id=` | MCP, **boucle locale** | l'état d'une commande |

Chaque serveur branche ces routes en quelques lignes (`pilote.traiter(handler)`), avant
ses propres routes. WEB_3D les traite dans `do_GET` et `do_POST`, avant `super().do_GET()`.

**Page : `pilote.js` et `pilote.css`**, recopiés à l'identique, en script classique.
`pilote.js` s'occupe de :
- l'id d'onglet ;
- l'élection du relais (`navigator.locks`) ;
- le relais par BroadcastChannel ;
- le panneau « Claude » (cartes et interrupteur) ;
- le registre des commandes : `Pilote.commande(nom, {ecriture, proposer, appliquer, annuler, executer})`.

Chaque page déclare ses commandes et sa capture. WEB_3D le fait depuis `09-demarrage.js`,
puisque ses modules n'exposent rien sur `window`.

**MCP : `lanceur/mcp_pilote.py`**, à côté de `mcp_api_outils.py`, avec le même style :
outils nommés, arguments validés, tout passe par `appeler_outil_web()`. Le noyau apprend à
rendre un contenu MCP `image`.

## Changements hors du lanceur (dépôts des outils)

| Dépôt | Fichiers |
|---|---|
| WEB_CAO | `pilote.py`, `commun/pilote.js`, `commun/pilote.css` ; branchement dans `web_CAO.py` ; balises `<script>` dans les pages schéma, PCB, gestion-lib et visionneuse IPC ; commandes `cao_calcul` (PCB, visionneuse) et `cao_valeur_composant` (schéma) ; captures via les `exportPng` existants ; monofichiers `dist/` reconstruits |
| WEB_ANTENNA | `pilote.py`, `js/pilote.js`, `css/pilote.css` ; branchement dans `web_antenna.py` ; commande `antenna_proposer` sur `iaProposition` / `iaAppliquer` / `iaAnnuler` ; capture du canevas 2D `#carte` (la vue 3D n'a pas `preserveDrawingBuffer` : non couverte) |
| WEB_3D | `pilote.py`, `js/pilote.js`, `css/pilote.css` ; branchement dans `web_3D.py` ; capture via `Vue3D.capture()` dans `09-demarrage.js` |
| web_suite | `lanceur/mcp_pilote.py`, `lanceur/mcp_web_suite.py` (contenu `image`), `lanceur/banc_mcp.py`, la CI (copies de `pilote.py`, `pilote.js` et `pilote.css` identiques dans les trois outils), la doc |

## Pile technique

- Python 3.8+, bibliothèque standard seulement : `threading.Condition` pour l'attente,
  `secrets` pour les ids.
- JS sans dépendance, en script classique : `BroadcastChannel` et `fetch`. **Ni
  `navigator.locks` ni `crypto.randomUUID`** : ils sont réservés aux contextes sécurisés,
  alors que l'iPad ouvre `http://<adresse du Pi>`. Les ids viennent de
  `crypto.getRandomValues`, qui existe partout.
- Le long-poll tient sur les serveurs actuels : `ThreadingTCPServer` avec
  `daemon_threads`, sans délai côté serveur.

## Commandes

```
Banc MCP :          python lanceur/banc_mcp.py
Banc du canal :     python3 pilote.py           (auto-test du module, dans chaque outil)
Banc page (Node) :  node commun/test/banc-pilote.js   (WEB_CAO : registre, relais simulé, cartes)
Monofichiers :      python3 editeur-pcb/outils/build-monofichier.py (et schéma) après ajout du script
```

## Style de code

Celui des outils : noms en français, commentaires sur le *pourquoi*, aucune dépendance.
Côté page, une commande se déclare ainsi :

```js
Pilote.commande("cao_valeur_composant", {
  ecriture: true,
  proposer: a => { const c = trouverComp(a.ref); return {avant: c.value, apres: a.valeur, titre: a.ref}; },
  appliquer: a => { push(); trouverComp(a.ref).value = a.valeur; refreshPanels(); draw(); },
  // pas d'« annuler » : l'éditeur a un historique, Ctrl+Z défait l'étape poussée
});
```

## Stratégie de test

- **`pilote.py`** : auto-test lancé par `python3 pilote.py`, dans le style des bancs du
  lanceur. Il vérifie :
  - la file d'attente et le long-poll (une commande déposée est rendue au bon navigateur) ;
  - l'attente de 2 min (raccourcie pour le banc) ;
  - l'oubli des onglets muets ;
  - le refus des routes d'ordre hors boucle locale ;
  - un id inconnu.
- **`lanceur/banc_mcp.py`** grandit. Un faux outil sert les routes `/api/pilote/*`, et le
  banc vérifie :
  - que chaque outil MCP envoie la bonne commande ;
  - les arguments refusés ;
  - le contenu `image` ;
  - « en attente (id) », puis `pilote_suivre`.
- **WEB_CAO, banc Node** (`commun/test/dom-stub.js` existe déjà) : le registre (une
  écriture sans undo est refusée), la carte (rien n'est appliqué avant le clic, ni avec
  l'interrupteur éteint), une seule étape d'undo pour `cao_valeur_composant`, le relais
  simulé.
- **CI de web_suite** : les copies de `pilote.py`, `pilote.js` et `pilote.css` sont
  identiques dans les trois outils.
- **Essai réel**, sur le Pi, avec Chromium (déjà installé) piloté par Playwright :
  - deux onglets WEB_CAO (schéma et PCB) et un onglet WEB_3D ;
  - Claude liste les onglets, capture le PCB, lance une impédance, propose `R1` → `4k7`,
    puis l'applique (clic simulé) et vérifie que Ctrl+Z la défait ;
  - on ferme l'onglet relais pendant une attente, et la commande aboutit quand même.
  - Puis un essai à la main avec toi, sur l'iPad.

## Limites

- **Toujours** :
  - proposer par défaut, n'appliquer directement que si l'interrupteur de l'onglet est
    allumé ;
  - une écriture = une étape annulable ;
  - les commandes se font par nom d'un registre ;
  - les routes d'ordre en boucle locale seulement ;
  - les copies identiques sont vérifiées par la CI ;
  - les bancs sont verts avant chaque commit ;
  - les monofichiers sont reconstruits.
- **Demander d'abord** :
  - ajouter une commande d'écriture hors de la liste ci-dessus ;
  - toucher à `appairage.py` ;
  - changer le comportement de l'assistant Gemini (son retrait est le module `retrait-gemini`).
- **Jamais** :
  - exécuter du code reçu (`eval`, `Function`, `innerHTML` avec une valeur venant de
    Claude sans échappement) ;
  - appliquer une écriture sans carte quand l'interrupteur est éteint ;
  - lancer ou arrêter openEMS ;
  - laisser une page envoyer des ordres à une autre ;
  - garder l'interrupteur allumé d'un chargement à l'autre.

## Critères de réussite

- Les bancs sont verts : `banc_mcp.py`, l'auto-test de `pilote.py` dans les trois outils,
  le banc Node de WEB_CAO, et la CI de web_suite et de WEB_CAO (monofichiers
  reproductibles).
- L'essai réel avec Playwright, sur le Pi, passe ; l'essai à la main sur l'iPad, avec
  toi, aussi.

## Questions tranchées (2026-10-18)

1. **Fin d'un calcul** (vérifié le 2026-10-18).
   - Les lanceurs sont `async`, et leur promesse se résout quand le calcul est fini :
     `simGo` (`commun/simulation-em.js:19054`), `simXtGo` (:9217), `simDCLancer` (:2468),
     `simRfGo` (:16330), `simOeilGo` (:17420) et `simCarteGo` (:6592). `simCalculerPDN`
     (:13626) est local et synchrone.
   - Une erreur ne lève pas d'exception : elle est rangée dans l'état (`SIM.err`,
     `SIM_PDN.erreur`…), puis lue dans le rapport.
   - Correspondance : `impedance`, `diff`, `retour` → `simGo` (comme
     `simRafraichir`, :19470-19474) ; `crosstalk` → `simXtGo` ; `oeil` → `simOeilGo` ;
     `dc` → `simDCLancer` ; `pdn` → `simCalculerPDN` ; `s21` → `simRfGo` ; `verif` →
     `simCarteGo`.
   - Avant le lancement, on choisit l'analyse avec `simAllerFamille` / `simAllerAnalyse`
     (:18270, :18282). `simGo` ne fait rien si `SIM.occupe` ou si `!simCalculable()` :
     ce cas est signalé à Claude.
2. **Annuler par la carte** : seulement si l'état n'a pas bougé depuis la commande. Sinon,
   la carte renvoie au Ctrl+Z de l'éditeur, et n'écrase jamais ton travail.
   - Vérifié le 2026-10-18 : `iaAnnuler` d'ANTENNA (`js/30-ia.js:1437-1467`) remet l'état
     d'avant **sans** vérifier qu'il n'a pas bougé, et ne sait pas annuler un `balayage`.
   - C'est donc `pilote.js` qui porte la règle : chaque commande d'écriture déclare un
     `etat()`, la carte en garde l'instantané juste après l'application, et Annuler
     n'agit que si `etat()` vaut toujours cet instantané.
   - Pour le balayage, l'adaptateur garde et remet lui-même `ANT.balayage`.
3. **Panneau « Claude »** : replié dans un coin, il s'ouvre de lui-même quand une carte
   arrive, puis se replie. Il cohabite avec l'assistant Gemini jusqu'au module
   `retrait-gemini`.
4. **Schéma** (vérifié le 2026-10-18) : `cao_valeur_composant` reprend l'action
   `schema_values` (`commun/ia-assistant.js:1411-1450`). Elle fait `push()`, cherche le
   repère sans tenir compte de la casse dans `S.comps` puis dans `S.pages[].comps`, passe
   par `reshapeComp` quand la définition a des broches calculées, puis appelle
   `buildList`, `refreshPanels` et `draw`.
5. **Contexte non sécurisé** (vérifié le 2026-10-18, Chromium 155 sur le Pi). Sur
   `http://192.168.1.31`, la page n'est pas en contexte sécurisé : `navigator.locks` et
   `crypto.randomUUID` valent `undefined`. `BroadcastChannel`, `crypto.getRandomValues` et
   `localStorage` sont présents. Sur `http://127.0.0.1`, tout est présent. D'où
   l'élection par BroadcastChannel (décision 8).
