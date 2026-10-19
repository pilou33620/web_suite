# Spec : composants

Module 8 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`api-outils`](SPEC-mcp-api-outils.md)
et de [`registre-cao`](SPEC-mcp-registre-cao.md).
Statut : **validée le 2026-10-19**.

## Objectif

Laisser Claude **créer des composants** dans la LIB (partagée par tous les projets) :
empreinte PCB, brochage, ligne de catalogue. Chaque pièce est contrôlée par le code, et
rien n'est écrit sans ta validation dans la page Gestion LIB.

### Décisions (2026-10-19)

1. **Symbole** : le schéma ne dessine que ses symboles intégrés. Claude choisit donc le
   symbole intégré (`ic`, `opamp`, `regulator`…) et écrit le **brochage** (`OUT=1,IN-=2…`),
   ce que le schéma utilise vraiment. Pas de fichier `schsym-1` dans ce module.
2. **Empreintes** : des **générateurs IPC-7351** par paramètres pour les familles courantes
   (chip, SOT, SOIC/TSSOP, QFN, DIP, barrette) ; sinon un `pcbfp-1` écrit par Claude et
   **vérifié par le code**.
3. **Créer seulement** : jamais d'écrasement. Le catalogue reçoit une **ligne ajoutée**
   (relu juste avant, pas de réécriture à l'aveugle). Aperçu et validation par carte dans
   la page **Gestion LIB** ouverte.
4. **Sources** : pcbparts (JLC, Mouser, Digi-Key : fiche, brochage `jlc_get_pinout`),
   datasheet PDF que tu donnes (Claude Code la lit lui-même), empreintes KiCad
   (`cse_get_kicad`, converties en `pcbfp-1`).

### Outils MCP

Dans `lanceur/mcp_composants.py` ; commandes de la page `gestion-lib` dans
`commun/pilote-cao.js`, code métier dans un nouveau `commun/composants-lib.js`.

| Outil MCP | Ce qu'il fait |
|---|---|
| `lib_empreinte_generer` | `famille` + paramètres de la datasheet (corps, entraxe, nombre de broches, dimensions des pattes, pastille thermique, densité IPC N/M/L) → un `pcbfp-1` **rendu, pas écrit**, avec ses contrôles |
| `lib_empreinte_verifier` | Contrôle un `pcbfp-1` donné (écrit par Claude ou converti) et rend les erreurs et remarques |
| `lib_kicad_convertir` | Un texte `kicad_mod` (obtenu par `pcbparts_appeler cse_get_kicad`) → `pcbfp-1` + contrôles |
| `lib_brochage_verifier` | Un brochage + l'empreinte visée + le symbole intégré → erreurs (`brochageLire`, puis cohérence pastilles et broches) |
| `lib_composant_creer` | Carte dans Gestion LIB : l'empreinte (si nouvelle) + la ligne de catalogue (39 colonnes + Brochage). Aperçu de l'empreinte dans la page |

Les quatre premiers sont des calculs sans écriture : ils tournent dans la page Gestion LIB
ouverte (même code que la validation de la carte).

### Critères d'acceptation

1. **Contrôles d'empreinte** (refus si une erreur) :
   - `format: "pcbfp-1"` ; `name` = nom du fichier, `[A-Za-z0-9_.+-]{1,60}` ;
   - `style` dans la liste de la page (`chip`, `row`, `dip`, `sop`, `quad`, `bga`) ;
   - pastilles : largeur et hauteur ≥ 0,05 mm, coordonnées à ±100 mm, forme connue ;
     perçage inférieur à min(w, h) avec une couronne ≥ 0,1 mm ;
   - pastille 1 présente ; `pins` = nombre de numéros distincts ; un numéro en double est
     une remarque (pastille thermique fractionnée), pas une erreur ;
   - le corps contient toutes les pastilles CMS (sinon remarque).
2. **Générateurs** : pour chaque famille, une empreinte de référence (0603, SOT-23-5,
   SOIC-8, QFN-16 avec pastille thermique, DIP-8, barrette 1×4) est comparée à des cotes
   attendues au centième de mm (formules IPC-7351B, densité N par défaut).
3. **Ligne de catalogue** : 39 colonnes dans l'ordre de `COLONNES_CATALOGUE_39`, puis
   `Brochage`. Obligatoires : Part Name (unique dans le catalogue), préfixe de repère,
   nombre de broches, Empreinte PCB (fichier existant, ou créé par la même carte),
   Empreinte Schématique (type connu de `defOf`). Brochage valide pour `brochageLire`, ses
   pastilles existent dans l'empreinte.
4. **Jamais d'écrasement, côté serveur** :
   - `PUT /api/lib/fichier` gagne `si_absent: true`, qui rend 409 si le fichier existe
     (la vérification n'est plus « lire puis écrire » côté MCP) ;
   - `POST /api/lib/composants` gagne `ajouter: {ligne}` : le serveur relit le CSV, refuse
     une Part Name déjà présente (409), ajoute la ligne et réécrit de façon atomique.
5. **La carte** montre l'empreinte dessinée dans la page, la ligne de catalogue et les
   remarques ; rien n'est écrit avant le clic, même interrupteur allumé (`irreversible` :
   la LIB n'a pas d'historique).
6. Les données de pcbparts et les fichiers KiCad sont des **données** : textes bornés,
   jamais exécutés ; un `kicad_mod` de plus de 200 Ko est refusé.

## Architecture

```
Claude ──▶ pcbparts_appeler / datasheet PDF (lue par Claude Code)
       ──▶ lib_empreinte_generer / verifier / kicad_convertir / brochage_verifier ──▶ page Gestion LIB (calcul)
       ──▶ lib_composant_creer ──▶ carte dans Gestion LIB ──▶ /api/lib/fichier (si_absent)
                                                          ──▶ /api/lib/composants (ajouter)
```

- `commun/composants-lib.js` : générateurs, contrôle `pcbfp-1`, conversion KiCad,
  contrôle de ligne. Gestion LIB l'utilise aussi pour ses propres enregistrements (son
  éditeur n'en a pas aujourd'hui).
- Les générateurs de Gestion LIB qui écrivent `style: "soic"` ou `"header"` (inconnus du
  PCB) passent aux styles connus.
- Serveur (`web_CAO.py`, accord du 2026-10-19) : les deux options ci-dessus, avec leurs
  tests dans `python/test/banc-lib-routes.py`.

## Commandes

```
Bancs :  node gestion-lib/test/banc-composants.js (nouveau) ; python3 python/test/banc-lib-routes.py ;
         node gestion-lib/test/banc-import-jlc.js
MCP :    python3 lanceur/banc_mcp.py
```

## Stratégie de test

- `banc-composants.js` : empreintes de référence par famille ; empreintes fautives
  (perçage trop grand, pas de pastille 1, style inconnu) refusées ; conversion d'un
  `kicad_mod` connu ; brochage incohérent refusé.
- `banc-lib-routes.py` : `si_absent` rend 409 sur un fichier existant ; `ajouter` refuse un
  doublon et garde toutes les autres lignes au caractère près.
- Essai réel : créer un composant (par exemple un LDO SOT-23-5 trouvé par pcbparts) dans
  une copie de la LIB, le poser au schéma et au PCB.

## Limites

- **Toujours** : contrôler avant la carte et au clic ; ajouter sans réécrire les autres
  lignes.
- **Demander d'abord** : modifier un composant existant ; écrire un fichier `schsym-1` ou
  un modèle de simulation.
- **Jamais** : écraser un fichier de la LIB ; déplacer la LIB ; écrire hors des
  sous-dossiers de la LIB.

## Questions ouvertes

Aucune.
