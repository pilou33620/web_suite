# Spec : api-outils

Module 2 de la [carte MCP web_suite](SPEC-mcp.md). Dépend de [`mcp-noyau`](SPEC-mcp-noyau.md).
Statut : **validée le 2026-10-18**.

## Objectif

Donner à Claude, par des outils MCP nommés, ce que les serveurs des outils savent déjà faire
**sans la page** : lister et lire les projets, lire la LIB, interroger pcbparts, suivre
openEMS, et écrire projets, LIB et profils avec des garde-fous. Tout passe par
`appeler_outil_web()` du noyau.

Utilisateur : toi, depuis Claude Code, sur le poste ou par SSH.

**Ce que ce module ne fait pas** (décidé le 2026-10-18) :
- **Lancer un calcul WEB_CAO** (EM, DC, crosstalk, RF, œil, analyse-carte, placement,
  motifs) : leurs documents d'entrée sont fabriqués par le JS de la page à partir de
  l'éditeur. Ces calculs passeront par `canal-pilote`. Ici, seulement l'**état des solveurs**.
- **Lancer ou arrêter openEMS** : Claude chiffre, prépare et suit seulement.
- **Ouvrir ou fermer un projet WEB_ANTENNA** : ça change le projet courant de ta session.
- **Brancher pcbparts.dev directement** : on passe par le relais de WEB_CAO (sa liste blanche).

## Outils MCP

Un outil par action, chacun avec un schéma d'entrée précis. Il n'y a pas d'outil générique
« appeler une route » : **la liste ci-dessous est la liste blanche**.

| Outil MCP | Outil web | Route | Garde-fou du MCP |
|---|---|---|---|
| `cao_solveurs` | WEB_CAO | GET `/api/simulation`, `/api/simulation-dc`, `/api/crosstalk`, `/api/simulation-rf`, `/api/oeil`, `/api/analyse-carte`, `/api/ipc2581` | un seul appel MCP ; les gabarits de l'œil sont réduits à leurs noms |
| `cao_projets` | WEB_CAO | GET `/api/projets` | — |
| `cao_projet_lire` | WEB_CAO | GET `/api/projet?chemin=` | — |
| `cao_doc_lire` | WEB_CAO | GET `/api/projet/doc?chemin=&doc=` | `doc` ∈ `schema`, `pcb`, `ipc2581` ; `parties` (clés de premier niveau) au choix |
| `cao_doc_ecrire` | WEB_CAO | GET puis PUT `/api/projet/doc?chemin=&doc=&modifie=` | **fusion** : les clés de premier niveau données remplacent celles du document relu, le reste demeure ; `modifie` **obligatoire et ≥ 0** (WEB_CAO ne contrôle le conflit que pour un entier positif) ; résumé recopié refusé ; un 409 revient sous la forme « le fichier a changé, relis-le » |
| `cao_profils` / `cao_profil_lire` | WEB_CAO | GET `/api/profils`, `/api/profil?nom=` | — |
| `cao_profil_ecrire` | WEB_CAO | PUT `/api/profil?nom=` | **refus si le profil existe déjà** : `GET /api/profil?nom=` doit rendre 404 |
| `lib_composants` | WEB_CAO | GET `/api/lib/composants` | filtré et paginé **par le MCP** : `filtre` (texte), `colonnes`, `debut`, `nombre` (50 par défaut) |
| `lib_fichiers` | WEB_CAO | GET `/api/lib/fichiers` | — |
| `lib_fichier_lire` | WEB_CAO | GET `/api/lib/fichier?type=&nom=` | `type` ∈ `pcb`, `schematique`, `simulation` |
| `lib_fichier_ecrire` | WEB_CAO | POST `/api/lib/fichier` | **refus si le fichier existe déjà** : `GET /api/lib/fichier?type=&nom=` doit rendre 404 (le serveur applique sa propre normalisation du nom) |
| `lib_parasites` | WEB_CAO | GET `/api/lib/parasites` | — |
| `pcbparts_outils` | WEB_CAO | GET `/api/tools` | — |
| `pcbparts_appeler` | WEB_CAO | POST `/api/tool` | `name` doit figurer dans la liste `allowed` rendue par `/api/tools` ; arguments de 2 Ko au plus |
| `antenna_projets` | WEB_ANTENNA | GET `/api/projet` | — |
| `antenna_projet_lire` | WEB_ANTENNA | GET `/api/projet/ouvrir?nom=` | **seulement si `nom` est le projet déjà ouvert**, vérifié par `/api/projet` avant l'appel |
| `antenna_projet_enregistrer` | WEB_ANTENNA | POST `/api/projet/enregistrer` | **seulement le projet ouvert** (revérifié avant l'envoi) ; **refus de toute valeur `null`**, et pour `carte`, `resultats`, `dessin`, `antenne`, `affichage` de tout ce qui n'est pas un objet non vide ; les champs d'en-tête absents sont repris du projet ouvert |
| `antenna_calculs` | WEB_ANTENNA | GET `/api/projet/calculs` | — |
| `antenna_openems_etat` | WEB_ANTENNA | GET `/api/openems` | — |
| `antenna_openems_chiffrer` | WEB_ANTENNA | POST `/api/openems`, `/api/openems/balayage`, `/api/openems/tableau-s`, `/api/openems/script` | `quoi` ∈ `modele`, `balayage`, `tableau_s`, `script` ; jamais `…/lancer` |
| `antenna_openems_suivre` | WEB_ANTENNA | GET `/api/openems/journal?id=&depuis=` | — |
| `antenna_openems_champs` | WEB_ANTENNA | GET `/api/openems/champs?id=` | inventaire seulement, pas les coupes de champ (plusieurs Mo) |
| `web3d_projets` | WEB_3D | GET `/api/projets` | — |

**Grosses réponses.** Au-delà de **100 Ko** de JSON (le texte rendu à Claude, sans indentation, pour tous les outils), le MCP ne rend pas le contenu brut. Il
rend un **résumé** : la taille, les clés de premier niveau avec leur type et la longueur de
chaque liste, et un extrait. Il y ajoute l'indication de l'argument à passer pour obtenir une
partie (`parties` pour un document, `debut`/`nombre` pour le catalogue).

**Critères d'acceptation**

1. `tools/list` montre les outils ci-dessus, avec des descriptions en français qui disent
   quand s'en servir. `etat` reste en place.
2. Chaque outil appelle **exactement** sa route et sa méthode, en boucle locale, par
   `appeler_outil_web()`.
3. Les garde-fous de la colonne de droite refusent **avant** la moindre requête
   d'écriture : pas de `modifie`, fichier LIB ou profil existant, `null` dans un enregistrement
   ANTENNA, projet ANTENNA non ouvert, outil pcbparts hors de la liste blanche. Chaque refus
   rend un résultat `isError` lisible.
4. Arguments manquants ou du mauvais type : résultat `isError`, qui nomme l'argument fautif.
5. Une réponse de plus de 100 Ko devient un résumé de moins de 100 Ko.
6. Un outil web arrêté donne le message du noyau (« WEB_CAO ne tourne pas : lance-le depuis
   WEB·SUITE »).

## Changement dans le noyau

`appeler_outil_web()` ne fait que GET et POST. WEB_CAO écrit les projets et les profils en
**PUT**. Il faut donc ajouter un argument `methode` (`GET`, `POST` ou `PUT` ; `DELETE`
jamais), toujours refusé pour le lanceur quand il n'est pas en GET. Après la revue du
2026-10-18 : les routes « Jamais » ci-dessous sont aussi ajoutées à `ROUTES_INTERDITES`, et
une réponse de plus de 64 Mo est refusée (mémoire du Pi).

## Pile technique

Celle du noyau : Python 3.8+, bibliothèque standard seulement. Pas de jeton : la boucle
locale passe déjà. Le lanceur démarre WEB_CAO avec `--local`, ou avec `--projets-reseau` en
mode réseau (`lanceur/projets.py:arguments`), donc ses routes de projet et de LIB sont
ouvertes. Un WEB_CAO lancé à la main en écoute réseau sans `--projets-reseau` répond 403 :
le message est rendu tel quel.

## Commandes

```
Banc :              python lanceur/banc_mcp.py
Tous les bancs :    python lanceur/banc_serveur.py && python lanceur/appairage.py && python lanceur/banc_mcp.py
```

## Structure

```
lanceur/mcp_api_outils.py  → les outils de ce module (dict ajouté à OUTILS_MCP), validation des arguments, résumé
lanceur/mcp_web_suite.py   → importe mcp_api_outils ; appeler_outil_web() reçoit « methode »
lanceur/banc_mcp.py        → le faux outil sert les routes ci-dessus et note chaque requête reçue
lanceur/docs/SPEC-mcp.md   → « Specs écrites » mise à jour
```

## Style de code

Celui du noyau : noms en français, docstrings courtes, commentaires sur le *pourquoi*, une
entrée de dict par outil.

```python
def cao_doc_ecrire(args):
    chemin, doc, document = exiger(args, chemin=str, doc=str, document=dict)
    modifie = exiger(args, modifie=int)   # sans lui, WEB_CAO écrase sans contrôle de conflit
    route = "/api/projet/doc?" + urlencode({"chemin": chemin, "doc": doc, "modifie": modifie})
    return appeler_outil_web("web_cao", route, document, methode="PUT")
```

## Stratégie de test

Il n'y a pas de nouveau banc : `lanceur/banc_mcp.py` grandit. Son faux outil HTTP répond aux
routes de la table avec de petits JSON et **note chaque requête** (méthode, chemin, corps).
Le banc vérifie :

- un appel par outil : la bonne méthode et la bonne route atteignent le faux outil ;
- chaque garde-fou : refus `isError`, et **aucune** requête d'écriture reçue ;
- un argument manquant donne une erreur qui le nomme ;
- une réponse de 300 Ko donne un résumé de moins de 100 Ko ;
- `lib_composants` filtre et pagine ;
- un 409 sur `cao_doc_ecrire` revient sous forme lisible.

Essai à la main, une fois, avec les vrais outils lancés : demander à Claude de lister les
projets CAO, de lire un PCB (résumé puis une partie), de chercher un composant dans la LIB et
sur pcbparts, et de chiffrer un modèle openEMS.

## Limites

- **Toujours** : passer par `appeler_outil_web()` ; valider les arguments avant tout appel ;
  bibliothèque standard seulement ; banc vert avant chaque commit.
- **Demander d'abord** : ajouter un outil ou une route qui ne figure pas dans la table ;
  toucher aux serveurs des outils (WEB_CAO, WEB_ANTENNA, WEB_3D).
- **Jamais** : DELETE ; lancer ou arrêter openEMS ; ouvrir, fermer, archiver ou importer un
  projet ANTENNA ; écrire `projet.cao.json` (aucun contrôle de conflit côté serveur) ; changer une racine de projets ou la LIB (`/api/lib/config` en écriture,
  `/api/projet/racine`) ; remplacer le catalogue LIB entier ; ouvrir l'explorateur de
  fichiers (`…/dossier`) ; télécharger une datasheet ; importer un IPC-2581 ; envoyer un
  fichier à WEB_3D ; lancer un calcul WEB_CAO (il passera par `canal-pilote`) ; tout ce que
  le noyau interdit déjà.

## Critères de réussite

- `python lanceur/banc_mcp.py` vert en local et en CI ; les autres bancs restent verts.
- Les essais à la main ci-dessus donnent des réponses justes, sans qu'aucune réponse ne
  dépasse 100 Ko.

## Questions tranchées (2026-10-18)

1. **Page ouverte pendant une écriture de Claude.** Vérifié le 2026-10-18 : l'éditeur
   envoie le `modifie` qu'il a lu, et sur un 409 il **demande avant d'écraser** (« Ce
   document a été modifié ailleurs… Écraser quand même ? »), dans
   `WEB_CAO/commun/projet-disque.js:513-528`. `cao_doc_ecrire` reste donc dans le module.
   Reste à ta charge : répondre « non » à cette question si c'est Claude qui vient d'écrire.
2. **Profils CAO** : l'écrasement est refusé, comme pour la LIB.
3. **`projet.cao.json`** : `cao_projet_ecrire` est retiré ; les documents suffisent.
4. **Enregistrer un projet ANTENNA qui n'est pas ouvert.** Vérifié le 2026-10-18 :
   `projet.enregistrer` crée le dossier si besoin (`WEB_ANTENNA/python/projet.py:420`) et
   **ouvre le projet enregistré** (`projet.py:462`, puis `_pr_suivre` redirige les calculs).
   L'enregistrement est donc limité au projet ouvert. Autre piège : `titre`, `outil`,
   `source`, `fichier`, `dessin`, `antenne` et `affichage` absents sont **remis à vide**
   (`projet.py:438-446`), alors qu'une `carte` ou des `resultats` absents restent
   inchangés. Le MCP reprend donc les champs d'en-tête absents depuis le projet ouvert
   avant d'envoyer.
