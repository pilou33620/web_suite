# Spec : registre-cao

Module 4 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`canal-pilote`](SPEC-mcp-canal-pilote.md).
Statut : **validée le 2026-10-19**.

## Objectif

Donner à Claude la **lecture** des pages WEB_CAO ouvertes, des **commandes de vue** et le
**DRC**, par le canal de pilotage :
- éditeur schématique (nouveau : Gemini n'avait que le contexte de la feuille) ;
- éditeur PCB, visionneuse IPC-2581 et simulations (registres `OUTILS_PCB`, `OUTILS_IPC`,
  `OUTILS_SIM` de l'assistant Gemini, repris).

Les écritures viennent ensuite, dans [`ecriture-cao`](SPEC-mcp.md) puis `dessin-schema`.
Ce module n'écrit rien dans le projet.

Utilisateur : toi, depuis Claude Code. La page WEB_CAO est ouverte sur le poste ou sur un
appareil appairé.

### Décisions (2026-10-19)

1. **Découpage** : `registre-cao` = lecture, vue, DRC. `ecriture-cao` (propriétés de
   composants, pistes et classes, paramètres de simulation) et `dessin-schema` (poser,
   relier, étiqueter) suivent, chacun avec sa spec.
2. **Un outil MCP par page**, avec un argument `quoi` : 6 outils au lieu d'une vingtaine.
3. **Code partagé** : les registres quittent `commun/ia-assistant.js` pour un fichier
   commun, `commun/lecture-cao.js`, chargé avant lui. Gemini et le canal l'utilisent tous
   les deux jusqu'à `retrait-gemini`. Aucune copie.
4. **Vue et DRC** s'appliquent **directement**, avec une trace dans le panneau « Claude » :
   ils ne touchent pas au projet ni à l'historique.
5. Reprises dans les modules suivants (pour mémoire) : toutes les écritures passeront par
   une carte quand l'interrupteur est éteint ; une carte pourra porter un **lot** (une seule
   étape d'historique, taille bornée) ; les paramètres de simulation seront libres, avec
   l'unité du catalogue vérifiée et la valeur bornée.

### Commandes et outils MCP

Les commandes sont déclarées dans la page par `commun/pilote-cao.js` (déjà branché dans
les trois pages). Côté MCP, elles vivent dans un nouveau fichier,
`lanceur/mcp_registre_cao.py`, importé par le noyau comme `mcp_registre_3d.py`.

| Outil MCP | Page | `quoi` | Ce qu'il rend |
|---|---|---|---|
| `cao_schema_lire` | schéma | `feuilles`, `composants`, `composant`, `nets`, `net` | Feuilles (nom, nombre de composants) ; composants (repère, valeur, type, boîtier, référence fabricant, feuille, position en mm) ; un composant et le net de chaque broche, avec le nom de broche ; nets du document (nom, global, broches, feuilles, conflit de nom) ; un net et ses broches `REF.broche` |
| `cao_pcb_lire` | PCB | `empilage`, `classes`, `composants`, `composant`, `nets`, `net`, `drc` | Les sorties de `OUTILS_PCB`, telles quelles |
| `cao_ipc_lire` | visionneuse | `empilage`, `composants`, `composant`, `nets`, `net` | Les sorties de `OUTILS_IPC`, telles quelles |
| `cao_sim_lire` | PCB, visionneuse | `simulations`, `simulation` | Les sorties de `OUTILS_SIM` ; `afficher: true` passe le panneau sur l'analyse demandée pour lire son rapport complet (commande de vue, tracée) |
| `cao_vue` | les trois | — | `cadrer` sur un repère ou un net, `surligner` un net, `feuille` (schéma), `couches` visibles (PCB, visionneuse), `ajuster` ; `capturer: true` rend l'image dans la foulée |
| `cao_drc` | PCB | — | Lance le DRC (`runDrc`, synchrone) ; rend les erreurs puis les remarques, avec leur position |

Arguments communs : `filtre`, `ref`, `nom`, `a_router` (comme Gemini), `analyse` et
`partie` (simulations), `onglet` (canal).

### Critères d'acceptation

1. Chaque outil appelle sa commande de page et rend **du texte en lignes**, celui des
   registres Gemini, déjà fait pour être lu par un modèle. Les longueurs sont en mm (le
   schéma convertit ses pixels : 20 px = 1 mm).
2. **Mêmes sorties que Gemini** : pour PCB, IPC et simulations, la sortie de l'outil MCP est
   identique, au caractère près, à celle de `OUTILS_*` dans `ia-assistant.js` pour les
   mêmes arguments. L'assistant Gemini marche comme avant.
3. **Bornes reprises** : 150 composants ou nets, 200 broches, 80 lignes de DRC, puis
   « … (N de plus : précise le filtre) ». Le MCP résume au-delà de 100 Ko.
4. **Rien n'est écrit** : aucune lecture ne change `S`, `V` ni `SIM`, ni l'historique ni le
   drapeau « modifié ». `cao_vue` et `cao_drc` laissent une trace dans le panneau et ne
   passent pas par `push()`.
5. Un repère ou un net inconnu rend un refus qui dit quel outil donne la liste. Un `quoi`
   inconnu sur une page rend la liste des `quoi` de cette page. Une page qui n'a pas
   l'outil (par exemple `cao_drc` vers le schéma) rend un refus clair.
6. **Schéma** : les nets sont ceux de `docNets()` (noms globaux réunis sur toutes les
   feuilles), les broches celles de `computeNets`. Un composant est cherché sur toutes les
   feuilles, sans tenir compte de la casse.
7. `cao_drc` sur la carte d'exemple rend le même nombre d'erreurs que le bouton DRC.

## Architecture

```
MCP (lanceur/mcp_registre_cao.py) ──▶ /api/pilote/commande de WEB_CAO ──▶ page : commun/pilote-cao.js
                                                                          └─ commun/lecture-cao.js
                                                                             (LECTURE_CAO.pcb / .ipc / .sim / .schema)
commun/ia-assistant.js (Gemini) ─────────────────────────────────────────────┘
```

- `commun/lecture-cao.js` : script classique (comme les pages), qui pose un global
  `LECTURE_CAO = { pcb, ipc, sim, schema }` et ses aides (`contient`, `borner`, `mm`). Les
  fonctions lisent `S`, `V` et `SIM` **au moment de l'appel** : un même fichier sert aux
  trois pages.
- `ia-assistant.js` garde son comportement : il prend ses outils dans `LECTURE_CAO` au lieu
  de les définir. Le schéma n'y gagne pas d'outils Gemini : son contexte reste celui
  d'aujourd'hui.
- `pilote-cao.js` déclare `cao_schema_lire`, `cao_pcb_lire`, `cao_ipc_lire`,
  `cao_sim_lire`, `cao_vue`, `cao_drc` selon la page.
- Les trois pages chargent `lecture-cao.js` avant `ia-assistant.js` ; les bundles
  `dist/` le reprennent seuls (ils suivent les `<script src>` de la page).

## Pile technique

- Page : scripts classiques, globaux, aucune dépendance nouvelle.
- MCP : Python 3.8+, bibliothèque standard, même style que `mcp_registre_3d.py`.

## Commandes

```
Bundles :      python3 editeur-pcb/outils/build-monofichier.py ; python3 editeur-schematique/outils/build-monofichier.py
Bancs pages :  node editeur-pcb/test/harness.js ; node editeur-schematique/test/harness.js ; node commun/test/banc-pilote-cao.js
Canal, MCP :   node lanceur/banc_pilote.js ; python3 lanceur/banc_mcp.py (web_suite)
```

## Style de code

Celui de `commun/` : scripts classiques, noms en français, commentaires sur le *pourquoi*,
lignes de texte rendues comme les registres Gemini.

```js
/* Un composant du schéma se cherche sur toutes les feuilles : le repère est unique
   dans le document, pas dans la feuille. */
function schTrouver(ref) {
  const cible = String(ref || "").trim().toUpperCase();
  for (const p of S.pages || []) {
    const c = (p.comps || []).find(x => (x.ref || "").trim().toUpperCase() === cible);
    if (c) return { c, page: p };
  }
  return null;
}
```

## Stratégie de test

- **Harnais existants** (bundles, sans navigateur) : `editeur-pcb/test/harness.js` et
  `editeur-schematique/test/harness.js` gagnent des tests pour chaque `quoi`, les refus, et
  l'absence d'écriture (`S.undo`/`S.hist` et `S.dirty` inchangés).
- **Égalité avec Gemini** : le banc compare la sortie de `LECTURE_CAO.pcb` avec un relevé
  pris **avant** le déplacement du code (même carte, mêmes arguments).
- **`banc-pilote-cao.js`** : les commandes déclarées selon la page.
- **`banc_mcp.py`** : chaque outil envoie la bonne commande ; arguments refusés sans
  requête (`quoi` hors liste, types, bornes).
- **Essai réel** dans Chromium sur le Pi : une copie de PROJETS dans `/tmp`, WEB_CAO lancé
  dessus, les trois pages ouvertes ; chaque outil appelé par le MCP, captures à l'appui.

## Limites

- **Toujours** :
  - lire `S`, `V`, `SIM` sans les modifier ;
  - garder les sorties de Gemini identiques tant qu'il existe ;
  - bancs verts avant chaque commit ; bundles reconstruits avant les harnais.
- **Demander d'abord** :
  - changer une sortie de registre (le texte que Gemini lit aussi) ;
  - toucher à `web_CAO.py` ;
  - ajouter un `quoi` qui n'est pas dans cette spec.
- **Jamais** :
  - écrire dans le projet depuis ce module ;
  - exposer `/api/ia/*` ;
  - modifier un fichier de PROJETS pendant l'essai réel (copie dans `/tmp`).

## Questions ouvertes

1. Le schéma n'a qu'une forme de nom de broche fiable (`brNomBroche`) : si une broche n'a
   pas de nom, la sortie dit seulement son numéro. À vérifier à l'essai réel.
2. Durée du DRC sur une grosse carte : non mesurée. Si elle dépasse 2 min, le canal rend
   « en_attente », à suivre par `pilote_suivre`.
