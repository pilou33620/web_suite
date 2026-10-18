# Tâches : registre-3d

Plan : [plan-registre-3d.md](plan-registre-3d.md) · Spec : [lanceur/docs/SPEC-mcp-registre-3d.md](../lanceur/docs/SPEC-mcp-registre-3d.md)

## Tâche 1 : ✅ canal, `appliquer` asynchrone et écriture irréversible

**Description :** dans `lanceur/pilote.js` :
- `appliquer` et `annuler` sont attendus (`await`), et une exception asynchrone remonte en
  « echouee » ;
- `irreversible: "message"` peut remplacer `annuler` (et `etat`) : la carte affiche le
  message, sans bouton Annuler ;
- recopier dans WEB_CAO, WEB_ANTENNA et WEB_3D.

**Critères d'acceptation :**
- [x] Une écriture dont `appliquer` est une promesse qui rejette rend « echouee » avec son
      message ; une qui se résout rend « appliquee » avec son résultat
- [x] Une écriture `irreversible` se déclare sans `annuler` et affiche son message ; une
      écriture sans `historique`, `annuler` ni `irreversible` reste refusée
- [x] Les copies sont identiques dans les trois outils

**Vérification :** `node lanceur/banc_pilote.js` ; les bancs des trois outils (pages qui chargent `pilote.js`)

**Dépendances :** aucune · **Fichiers :** `lanceur/pilote.js`, `lanceur/banc_pilote.js`, les 3 copies · **Taille :** S

## Tâche 2 : ✅ `14-pilote-3d.js` : identification et scène

**Description :** créer `WEB_3D/js/14-pilote-3d.js` avec `trouverPiece(modele, ref)` (uuid,
chemin `0/3/1`, nom unique) et `decrireScene(modele, {filtre, debut, nombre})` (fichiers et
pièces, sérialisés).

Créer `test/banc-pilote-3d.mjs` sur la **vraie hiérarchie** de `assemblage-as1.stp`. Si
c'est possible, extraire de `02-import.js` la construction de scène à partir d'un résultat
OCCT, en fonction pure exportée.

**Critères d'acceptation :**
- [x] 18 pièces décrites (uuid, chemin, nom, parent, visible, triangles), et un fichier
      avec position et rotation
- [x] Le chemin et l'uuid retrouvent la même pièce ; « nut » est refusé avec la liste des 8
      candidates ; une référence inconnue est refusée
- [x] La sortie est du JSON pur (aller-retour `JSON.parse(JSON.stringify(x))` identique) ;
      `filtre` et pagination marchent

**Vérification :** `node test/banc-pilote-3d.mjs` ; `node test/banc-exemples.mjs` toujours vert

**Dépendances :** T1 (pour la suite) · **Fichiers :** `js/14-pilote-3d.js`, `test/banc-pilote-3d.mjs`, peut-être `js/02-import.js` · **Taille :** M

## Checkpoint : socle
- [ ] Bancs verts (lanceur, WEB_3D)
- [ ] Revue avec toi

## Tâche 3 : ✅ mesures et cote

**Description :**
- `dimensions(objet)` : boîte, volume, surface, nombre de pièces ;
- `distance(A, B)` : minimum et les deux points, via `mesurerEntites` ;
- `faces(piece)` : index, type, rayon, axe ou normale, centre, aire, via `analyser` et
  `faceSous` sans clic ;
- `coteClaude(p1, p2, texte)` : un groupe three.js à poser dans `vue.annotations`.

**Critères d'acceptation :**
- [x] Dimensions de l'assemblage = boîte attendue par `banc-exemples` (200 × 150 × 84) ;
      volume et surface positifs pour une pièce
- [x] La distance entre deux pièces vaut celle de `mesurerEntites` ; elle est nulle pour
      deux pièces en contact
- [x] `faces` d'une vis rend au moins un cylindre, avec son rayon ; un index se retrouve
      d'un appel à l'autre

**Vérification :** `node test/banc-pilote-3d.mjs`

**Dépendances :** T2 · **Fichiers :** `js/14-pilote-3d.js`, `test/banc-pilote-3d.mjs` · **Taille :** S

## Tâche 4 : ✅ collisions

**Description :** `collisions(modele, {tolerance})` sur `chercherCollisions`, résultat
sérialisé : couples (uuid, nom), longueur des courbes, durée. Les préférences ne sont pas
touchées.

**Critères d'acceptation :**
- [x] L'exemple donne les collisions de `chercherCollisions` (22 : vis et tiges dans leurs perçages ; le « 2 » de l'exploration n'était pas mesuré sur la vraie scène)
- [x] Les pièces masquées sont ignorées

**Vérification :** `node test/banc-pilote-3d.mjs`

**Dépendances :** T2 · **Fichiers :** `js/14-pilote-3d.js`, `test/banc-pilote-3d.mjs` · **Taille :** S

## Tâche 5 : ✅ déplacement, contrainte, matrices

**Description :**
- `deplacer(g, {translation, rotation, origine})`, avec la rotation autour du centre de la
  boîte (comme le panneau Déplacer) ;
- `contrainte` par `contraindre` (faces désignées par leur index) ;
- `etatMatrices(modele)` et `remettreMatrices(modele, etat)` ;
- `apres()` de `13-deplacement.js` exporté (`apresDeplacement`) et utilisé par le panneau.

**Critères d'acceptation :**
- [x] Translation, rotation et origine sont exactes au µm près ; une rotation garde le
      centre de la boîte
- [x] Une contrainte plan contre plan plaque les faces (distance nulle) ; des faces
      incompatibles rendent le message de `contraindre`
- [x] Déplacer puis remettre l'état redonne les matrices d'origine au bit près

**Vérification :** `node test/banc-pilote-3d.mjs` ; `node test/banc-exemples.mjs`

**Dépendances :** T2, T3 (faces) · **Fichiers :** `js/14-pilote-3d.js`, `js/13-deplacement.js`, `test/banc-pilote-3d.mjs` · **Taille :** M

## Tâche 6 : ✅ ouvrir depuis PROJETS/3D

**Description :** exporter `ouvrir` et `compagnons` de `11-projets.js`, en leur faisant
rendre un résultat (succès, ou échec et message). La commande `web3d_ouvrir` :
- `proposer` : liste des fichiers, remplacement ou ajout ;
- `appliquer` : async, garde les fichiers d'avant ;
- `annuler` : recharge le modèle d'avant (remplacement) ou retire le fichier ajouté ;
- `etat` : liste des fichiers et leurs matrices.

**Critères d'acceptation :**
- [x] Un `.obj` emmène son `.mtl` ; un chemin hors de PROJETS (`..`, absolu) est refusé
      avant tout `fetch`
- [ ] Un fichier introuvable ou illisible rend « echouee » avec le message de la page (ui.charger rend maintenant {ok, erreur} ; à éprouver à l'essai réel, T11)

**Vérification :** banc Node pour `compagnons` et la validation des chemins ; le reste à
l'essai réel (T11)

**Dépendances :** T1 · **Fichiers :** `js/11-projets.js`, `js/14-pilote-3d.js`, `test/banc-pilote-3d.mjs` · **Taille :** S

## Tâche 7 : ✅ extraction STEP

**Description :** `extraireStep(groupe, pieces)` via `StepExtractor`. Les pièces sont
retrouvées sans ambiguïté ; sinon, refus clair. La carte irréversible de `web3d_ranger_step`
appelle `POST /api/projets` (`X-Web3D: 1`) depuis la page, puis rend le chemin créé.

**Critères d'acceptation :**
- [x] Le STEP extrait d'une pièce de l'exemple est relu par OpenCascade, et donne une
      pièce de même boîte
- [x] Noms exacts seulement ; une pièce à la fois (un sous-ensemble choisi pièce par pièce emporte ses voisines : refusé avec la raison)
- [x] Aucun déplacement n'est appliqué (positions d'origine, dit sur la carte)

**Vérification :** `node test/banc-pilote-3d.mjs`

**Dépendances :** T2 · **Fichiers :** `js/14-pilote-3d.js`, `test/banc-pilote-3d.mjs`, peut-être `js/10-export-step.js` · **Taille :** S

## Checkpoint : calcul
- [ ] `banc-pilote-3d.mjs` et `banc-exemples.mjs` verts
- [ ] Revue avec toi

## Tâche 8 : les 9 commandes dans la page

**Description :** `brancherPilote3d(W3D, Pilote)`, appelé par `09-demarrage.js` :
- lectures : `web3d_scene`, `web3d_mesurer`, `web3d_collisions` ;
- directes, avec trace : `web3d_vue` (sans animation, capture en option),
  `web3d_visibilite`, `web3d_coupe` ;
- cartes : `web3d_ouvrir`, `web3d_deplacer`, `web3d_ranger_step` ;
- la capture déjà déclarée reste.

**Critères d'acceptation :**
- [ ] Chaque commande est déclarée avec le bon type (lecture, directe ou carte) ; aucune
      n'écrit dans `localStorage`
- [ ] Banc Node : un faux `Pilote` reçoit les 9 déclarations, et les cartes ont
      `annuler` + `etat` ou `irreversible`

**Vérification :** `node test/banc-pilote-3d.mjs` ; l'essai réel en T11

**Dépendances :** T3 à T7 · **Fichiers :** `js/14-pilote-3d.js`, `js/09-demarrage.js`, `test/banc-pilote-3d.mjs` · **Taille :** M

## Tâche 9 : outils MCP

**Description :** `lanceur/mcp_registre_3d.py` : les 9 outils, arguments validés
(références, axes, vues, nombres finis, chemins PROJETS relatifs), envoyés sur
`/api/pilote/commande` de WEB_3D. Importé par le noyau.

**Critères d'acceptation :**
- [ ] Chaque outil dépose la bonne commande (page `3d`, nom, arguments, écriture) ;
      `web3d_vue` avec `capturer` rend une image
- [ ] Les arguments invalides sont refusés sans requête

**Vérification :** `python lanceur/banc_mcp.py`

**Dépendances :** T8 (noms des commandes) · **Fichiers :** `lanceur/mcp_registre_3d.py`, `lanceur/mcp_web_suite.py`, `lanceur/banc_mcp.py` · **Taille :** S

## Tâche 10 : CI et documentation

**Description :** `node test/banc-pilote-3d.mjs` dans la CI de WEB_3D ; DETAILS.md (outils
web3d_*) ; la carte `SPEC-mcp.md`.

**Critères d'acceptation :**
- [ ] CI verte dans les dépôts touchés (web_suite, WEB_CAO, WEB_ANTENNA, WEB_3D)
- [ ] Documentation à jour

**Dépendances :** T9 · **Fichiers :** `WEB_3D/.github/workflows/ci.yml`, `lanceur/docs/DETAILS.md`, `lanceur/docs/SPEC-mcp.md` · **Taille :** XS

## Tâche 11 : essai réel, puis revue

**Description :** WEB_3D lancé sur une copie de PROJETS contenant l'exemple, page ouverte
dans Chromium (SwiftShader) sur le Pi. Dérouler, par le MCP :
- ouvrir ;
- scène ;
- vue avec capture ;
- coupe ;
- mesurer avec cote ;
- collisions, en onglet caché aussi, pour mesurer le ralentissement ;
- déplacer, puis Annuler ;
- ranger un STEP.

Puis `/review` et l'audit de sécurité.

**Critères d'acceptation :**
- [ ] Tous les essais passent ; PROJETS reste intact ; le ralentissement en onglet caché
      est mesuré et, s'il le faut, signalé
- [ ] `/review` fait, corrections appliquées

**Dépendances :** T10 · **Taille :** M

## Checkpoint : terminé
- [ ] Critères de la spec remplis
