# Spec : registre-3d

Module 4 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`canal-pilote`](SPEC-mcp-canal-pilote.md).
Statut : **validée le 2026-10-18**.

## Objectif

Donner à Claude les commandes de la page WEB_3D ouverte, par le canal de pilotage :
- lire la scène et l'arbre ;
- régler la vue, la coupe et la visibilité ;
- mesurer et chercher les collisions ;
- ouvrir des fichiers de PROJETS/3D ;
- déplacer les fichiers de l'assemblage ;
- ranger le STEP d'une pièce dans PROJETS.

Utilisateur : toi, depuis Claude Code. La page WEB_3D est ouverte sur le poste ou sur un
appareil appairé.

### Décisions (2026-10-18)

1. **Carte ou pas** : une **carte à valider** pour ce qui change le modèle ou le disque
   (ouvrir, déplacer, ranger dans PROJETS). Le reste s'applique **directement**, avec une
   trace dans le panneau, parce que c'est visuel et réversible : vue et caméra, coupe,
   visibilité, isolement et sélection, mesures et cotes affichées, collisions affichées.
2. **Ouvrir** : seulement des fichiers de **PROJETS/3D** (servis par `/projets/`), en
   remplacement ou en ajout.
   - Annuler un remplacement **recharge le modèle d'avant** : la carte garde ses fichiers
     d'origine, mais les déplacements faits entre-temps sont perdus.
   - Annuler un ajout retire le fichier ajouté.
3. **Mesures** :
   - dimensions et masse : encombrement, volume, surface, nombre de pièces ;
   - distance minimale entre deux pièces, avec les deux points les plus proches ;
   - faces d'une pièce : plan ou cylindre, rayon, axe, aire, index stable ;
   - sur demande, la **cote affichée** dans la scène.
4. **Déplacement** : il porte sur un **fichier** de l'assemblage, c'est-à-dire un groupe :
   translation, rotation, retour à l'origine, et contrainte face plane contre face plane ou
   axe contre axe (faces désignées par leur index). Annuler remet les matrices d'avant.
5. **Export** : le **STEP d'une pièce** (ou d'un sous-ensemble), extrait du STEP d'origine
   et rangé dans PROJETS/3D. Le serveur n'écrase jamais : un nom pris devient `x (2).stp`.
   - C'est une écriture **irréversible** pour la page, qui ne sait pas supprimer : la carte
     le dit.
   - `pilote.js` accepte une écriture déclarée `irreversible` seulement si elle porte ce
     message.
   - Les déplacements ne sont pas repris : positions d'origine.

### Identifier une pièce

Les noms ne sont pas uniques : l'exemple livré contient 7 « nut ». Une pièce est rendue
avec son **uuid**, stable pour la session, son **chemin d'indices** sous la scène (par
exemple `0/3/1`) et son **nom**, pour l'affichage. Une commande accepte l'uuid ou le
chemin ; un nom seul n'est accepté que s'il est unique, sinon le refus liste les pièces
qui le portent.

### Commandes et outils MCP

Les commandes sont déclarées dans la page par un nouveau module, `js/14-pilote-3d.js`,
importé par `09-demarrage.js`. Côté MCP, elles vivent dans `lanceur/mcp_registre_3d.py`.

| Outil MCP | Carte | Ce qu'il fait |
|---|---|---|
| `web3d_scene` | non | Fichiers ouverts (nom, format, chemin PROJETS, nombre de pièces, position et rotation) et pièces (uuid, chemin, nom, parent, visible, couleur, triangles), avec `filtre` et pagination ; sélection, coupe et vue courantes |
| `web3d_vue` | non | Vue standard (`avant`, `arriere`, `gauche`, `droite`, `dessus`, `dessous`, `iso`), projection, cadrage sur une pièce ou un fichier ; sans animation (onglet caché) ; `capturer: true` rend l'image dans la foulée |
| `web3d_visibilite` | non | `masquer`, `afficher`, `isoler`, `tout_afficher` et `selectionner`, sur des pièces |
| `web3d_coupe` | non | Activer ou couper, axe `x`/`y`/`z` ou face désignée, décalage en mm (borné), sens |
| `web3d_mesurer` | non | `dimensions`, `distance` (deux pièces), `faces` (une pièce) ; `afficher: true` pose la cote dans la scène ; nombres en mm, repère du monde |
| `web3d_collisions` | non | Interférences entre pièces (tolérance en mm, défaut 0,1) : couples (uuid, nom) et longueur des courbes ; `afficher` les dessine |
| `web3d_ouvrir` | **oui** | Fichiers de PROJETS/3D (chemins de `web3d_projets`), `ajout` ou remplacement ; un `.obj` emmène son `.mtl` |
| `web3d_deplacer` | **oui** | Fichier visé, `translation` [mm], `rotation` [deg, autour du centre de sa boîte], `origine`, ou `contrainte` (`plan`/`axe`, face mobile et face fixe, décalage) |
| `web3d_ranger_step` | **oui, irréversible** | Pièces visées, nom du fichier : STEP extrait, rangé dans PROJETS/3D |

### Critères d'acceptation

1. Chaque outil appelle sa commande de page et rend des **nombres et des identifiants**
   sérialisables : jamais de Vector3, de fonction ni de maillage. Les unités sont le mm et
   le degré.
2. **Lectures** : `web3d_scene`, `web3d_mesurer` et `web3d_collisions` ne changent rien.
   Les commandes directes (vue, visibilité, coupe, mesure affichée, collisions affichées)
   laissent une trace dans le panneau.
3. **Écritures** : `web3d_ouvrir` et `web3d_deplacer` montrent une carte (avant → après,
   par exemple « gear.stp : x 0 → 25 mm »), et rien ne change avant le clic, sauf
   interrupteur allumé. Annuler remet l'état d'avant **si rien n'a bougé depuis**.
4. `web3d_ranger_step` montre une carte « crée PROJETS/3D/<nom> ; non annulable ici » ;
   appliquée, elle rend le chemin réellement créé.
5. Une pièce désignée par un nom en double est refusée, avec la liste des candidates. Une
   face désignée par un index hors limites est refusée.
6. Les préférences de la page (`localStorage` : tolérance, précision, projection…) ne sont
   **jamais** modifiées par une commande.
7. Une ouverture plus longue que 2 min rend « en_attente », à suivre par `pilote_suivre`.
   Une erreur de lecture du fichier remonte en échec lisible.
8. Les valeurs suivantes viennent du banc (exemple `assemblage-as1.stp`) :
   - 18 pièces ;
   - 2 collisions bolt ↔ l-bracket ;
   - une distance pièce-pièce identique à `mesurerEntites` ;
   - un déplacement puis Annuler qui redonne les matrices d'origine au bit près ;
   - un STEP de pièce relu par OpenCascade, qui donne une pièce.

## Changements dans le canal (`pilote.js`, copie identique)

- `appliquer` et `annuler` peuvent être **asynchrones** (`await`) : ouvrir lit des fichiers.
  Une exception asynchrone remonte en « échouée ».
- `irreversible: "message"` remplace `annuler` pour une écriture qui ne se défait pas. La
  carte affiche le message, sans bouton Annuler.
- Les **bancs** (`banc_pilote.js`) couvrent les deux.

## Architecture

```
MCP (lanceur/mcp_registre_3d.py) ──▶ /api/pilote/commande de WEB_3D ──▶ page : js/14-pilote-3d.js
                                                                        ├─ lit W3D (vue, arbre, mesure, ui, coupe, nav)
                                                                        └─ importe 06-topologie, 12-collisions,
                                                                           13-deplacement (contraindre), 10-export-step
```

- Ce qui se calcule (identification, sérialisation, mesures, collisions, déplacement et
  annulation, extraction STEP) est écrit en **fonctions pures** sur une scène three.js, donc
  testable sous Node sans WebGL.
- La partie qui touche la page (arbre, interface, coupe) reste mince.
- Ce qui manque à `13-deplacement.js` et `11-projets.js` (`apres()`, `ouvrir()`, fonctions
  internes) est soit **exporté** (préféré, une ligne chacune), soit recopié avec un
  commentaire qui dit d'où.

## Pile technique

- Page : modules ES, three.js r186. Aucune dépendance nouvelle.
- MCP : Python 3.8+, bibliothèque standard, même style que `mcp_pilote.py`.
- Le ranger passe par `POST /api/projets` du serveur WEB_3D (`X-Web3D: 1`, jamais
  d'écrasement), appelé depuis la page.

## Commandes

```
Banc 3D (Node) :     node test/banc-pilote-3d.mjs       (WEB_3D, à côté de banc-exemples.mjs)
Bancs existants :    node test/banc-exemples.mjs ; python test/banc-serveur.py
Canal :              node lanceur/banc_pilote.js ; python lanceur/banc_mcp.py (web_suite)
```

## Style de code

Celui de WEB_3D : modules ES, noms en français, commentaires sur le *pourquoi*.

```js
// Une pièce se désigne par uuid ou par chemin d'indices : les noms se répètent (7 « nut »)
export function trouverPiece(modele, ref){
  if(/^\d+(\/\d+)*$/.test(ref)) return parChemin(modele, ref.split("/").map(Number));
  return modele.getObjectByProperty("uuid", ref) || refuser(`Pièce « ${ref} » inconnue.`);
}
```

## Stratégie de test

- **`test/banc-pilote-3d.mjs`** (Node, sans WebGL) : charge `assemblage-as1.stp` comme
  `banc-exemples.mjs`, puis éprouve les fonctions pures : identification (uuid, chemin, nom
  en double refusé), sérialisation de la scène, dimensions, volume, distance, faces,
  collisions, déplacement et contrainte puis annulation, extraction STEP relue.
- **`banc_pilote.js`** (web_suite) : `appliquer` asynchrone, écriture `irreversible`.
- **`banc_mcp.py`** : chaque outil envoie la bonne commande, arguments refusés sans
  requête.
- **CI** : `banc-pilote-3d.mjs` ajouté à la CI de WEB_3D.
- **Essai réel** dans Chromium sur le Pi (SwiftShader) : ouvrir l'exemple depuis PROJETS,
  scène, vue et capture, coupe, mesure affichée, collisions, déplacer puis Annuler, ranger
  un STEP. PROJETS est une copie dans `/tmp`.

## Limites

- **Toujours** :
  - une carte pour ouvrir, déplacer et ranger ;
  - désigner les pièces par uuid ou chemin ;
  - jamais d'animation pour les commandes ;
  - les préférences de la page restent intactes ;
  - bancs verts avant chaque commit.
- **Demander d'abord** :
  - changer le comportement de l'interface pour la souris ;
  - ajouter un format d'export ;
  - toucher à `web_3D.py` (le ranger y existe déjà : rien à changer côté serveur).
- **Jamais** :
  - écraser un fichier de PROJETS ;
  - ouvrir un fichier hors de PROJETS/3D ;
  - modifier un fichier source ;
  - lancer une commande d'écriture sans carte quand l'interrupteur est éteint.

## Questions tranchées (2026-10-18)

1. **Onglet caché** : le ralentissement est mesuré à l'essai réel. S'il est lent, le
   résultat le dit (« onglet caché, calcul ralenti »), et l'appel bascule en « en_attente »
   au-delà de 2 min.
2. **Cote affichée** : l'adaptateur pose **sa propre cote** (ligne et étiquette dans
   `vue.annotations`), effacée au prochain `web3d_mesurer` ou par `effacer: true`. L'outil
   Mesure de la page n'est pas touché.
