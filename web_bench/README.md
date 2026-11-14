# web_bench — banc d'essai visible de WEB_CAO, WEB_ANTENNA et WEB_3D

L'outil choisi s'ouvre en entier à gauche, et reste utilisable à la main. À droite, un panneau
lance les essais et montre les résultats. Les essais se lancent outil par outil, et le menu
**Outil** passe de l'un à l'autre. Chaque outil démarre la première fois qu'on le choisit.

| Volet | Ce qu'il fait |
|---|---|
| **Clic-partout** | Clique un par un tous les boutons, menus, onglets, cases, listes et liens de chaque page, et relève ce qui casse : erreur JavaScript, promesse rejetée, requête en échec, panique du serveur de l'outil (« panicked at »), page bloquée plus de 1,5 s, requêtes qui ne finissent pas. Les éléments qui apparaissent après un clic (menu, fenêtre) sont essayés juste après, en rouvrant le menu si besoin. Une fenêtre déjà vue qui masque la page est refermée par son bouton (Annuler, ×, OK). |
| **Bancs** | Trouve les scripts node `banc-*.js / .mjs` de l'outil, les essais que sa CI lance sous un autre nom (`harness.js` des éditeurs, `harness-sim.js`), et ses bancs Rust : par caisse de son espace de travail cargo, ses essais unitaires (`cargo test -p caisse --lib --bins`) puis une entrée par cible de `tests/` (`cargo test -p caisse --test cible`). Il les lance un par un, **comme la CI** : avec les mêmes options de node, et après les mêmes préparations (assemblage par `build-monofichier` de `rust/outils`, puis `harness.js` avant `banc_ipc2581_export`). Ceux que la CI lance sont cochés d'office. Vert : le banc a rendu 0. Un clic montre sa sortie. |
| **Rapports** | Chaque clic-partout terminé ou arrêté est gardé dans `web_bench/rapports/`, en `.json` (tout) et en `.md` (lisible). Un clic recharge un rapport, et **.md** le télécharge. |
| **Journal** | La sortie du serveur de l'outil, avec les paniques en rouge et les requêtes arrêtées par le banc. |

## Démarrer

Depuis la racine de web_suite, une fois les outils installés (`installer.cmd` / `installer.sh`) :

| Système | Démarrer |
|---|---|
| Windows | `demarrer_web_bench.cmd` |
| Linux, Raspberry Pi | `./demarrer_web_bench.sh` |

web_bench est un exécutable Rust (caisse `web_bench`). Pour le lancer à la main, depuis la racine
de web_suite :

```
cargo run -p web_bench -- --outil 3d
```

Il trouve ses pages (`web_bench/page/`) et les outils à la racine du dépôt : celle que donne
`WEB_SUITE_DEPOT`, sinon le premier dossier contenant `lanceur/index.html` en remontant depuis
l'exécutable, puis depuis le dossier courant.

Le navigateur s'ouvre sur `http://127.0.0.1:8200/__banc/`.

Options :
- `--outil cao|antenna|3d` : l'outil ouvert au démarrage ;
- `--racine DOSSIER` : si les outils ne sont pas à la racine de web_suite ;
- `--port` et `--sans-navigateur`.

Les outils sont lancés par leur `demarrer_<NOM>`, comme le fait le lanceur : rien d'autre à
installer pour le clic-partout. Pour les bancs : Rust (cargo) pour les bancs Rust (sans cargo, ils
ne sont pas listés), Node.js pour les bancs JavaScript, et Chrome, Chromium ou Edge pour ceux qui
ouvrent une page.

## Le rapport, pour les corrections

Le `.md` de chaque rapport est fait pour être donné tel quel à Claude. Il contient :

- **le contexte** : la version essayée de l'outil (branche, commit, date, message, et s'il y a des
  modifications locales), le navigateur et la taille du cadre, les pages, les réglages et la liste
  noire ;
- **les problèmes regroupés** : un même message au même endroit du code ne compte qu'une fois,
  avec son nombre d'apparitions et la liste des boutons qui le déclenchent. Les erreurs viennent
  avant les avertissements. Pour chaque problème :
  - **où dans le code** : le fichier et la ligne JS (`WEB_CAO/editeur-pcb/js/29-draftsman.js:82:13`),
    le fichier Rust d'une panique du serveur, ou la route appelée, avec la ligne JS qui l'a appelée ;
  - **l'élément** : son libellé, son sélecteur CSS et sa balise ouvrante (id, classes) ;
  - **pour reproduire** : la page à ouvrir et les derniers gestes faits avant le clic fautif
    (clics, menus rouverts, fenêtres refermées, Échap) ;
  - **la pile d'appels JS**, sans les lignes du banc, ou **la panique du serveur** (fichier, ligne, message) ;
  - pour une requête : la méthode, l'URL, le statut, le corps envoyé et **la réponse du serveur** ;
- **les bancs** lancés dans la session : ceux qui passent, et pour ceux qui échouent, le code de
  sortie, la durée et la fin de leur sortie ;
- **ce qui n'a pas été essayé** : la liste noire, les liens externes, les éléments jamais atteints
  et la limite de clics.

## Ce que le banc ne touche pas

- **Tes projets et ta LIB.** L'outil est lancé à part avec `--local --sans-maj`, un dossier de
  projets vide et, pour WEB_CAO, une **copie** de ta LIB (`PROJETS/LIB_CAO`, `WEB_CAO/LIB` ou
  `WEB_CAO_LIB`). Tout est rangé dans un dossier temporaire effacé à l'arrêt. **Redémarrer
  l'outil** repart de dossiers neufs.
- **Tes réglages.** Les pages sont servies à l'adresse du banc. Leurs réglages, profils et projets
  récents sont donc séparés de ceux de ton usage normal.
- **Ce qui sortirait de l'outil.** Les `alert`, `confirm` et `prompt` reçoivent une réponse
  automatique : par défaut « non » aux confirmations, et une case permet de répondre « oui ».
  L'impression, les nouvelles fenêtres, les téléchargements et les sélecteurs de fichiers sont
  bloqués, mais notés. La **liste noire**, propre à chaque outil et modifiable, écarte GitHub, la
  publication, la mise à jour, le redémarrage et « Demander à Claude ». Les liens externes ne
  sont jamais suivis.
- **Les requêtes à risque, quel que soit le bouton.** Le relais arrête les envois vers GitHub
  (`/api/github/envoyer`), les demandes à Claude Code (`/api/pilote/demande`) et, dans
  WEB_ANTENNA, toute requête qui lancerait openEMS (`/api/openems/…lancer`). Le bouton « Lancer »
  est donc bien cliqué, et son code essayé jusqu'à la requête, mais aucune simulation ne démarre.

Le banc écoute **seulement sur 127.0.0.1**. Il parle aux outils depuis la boucle locale, donc
l'ouvrir au réseau contournerait l'appairage.

## Lire les résultats

- **Rouge (erreur)** : une erreur JavaScript, une promesse rejetée, une réponse 5xx ou une requête
  coupée, ou une panique du serveur de l'outil.
- **Orange (avertissement)** : une réponse 4xx, un `console.error`, une ressource manquante, un
  clic lent.
- **Gris (ignoré)** : un élément de la liste noire, un lien externe, un sélecteur de fichier,
  des éléments jamais atteints, ou la limite de clics atteinte.

Un clic sur une ligne montre le détail et entoure l'élément dans l'outil, quand il est sur la page
affichée.

## Limites

- Le clic-partout trouve ce qui **plante**. Il ne vérifie pas que le **résultat** est juste :
  c'est le rôle des bancs.
- Il ne tape pas de texte et ne dessine pas sur les canevas (PCB, schéma, vue 3D).
- Il n'y a pas de capture d'écran dans le rapport : le sélecteur et la balise désignent l'élément.
- L'outil tourne dans la même page que le banc. Une vraie boucle infinie dans l'outil fige donc
  aussi le banc. Un clic qui bloque la page est relevé dès qu'elle reprend la main.
- Les trois outils passent par la même adresse, `127.0.0.1:8200` : ils partagent le stockage du
  navigateur pendant les essais.
- **La LIB privée** : avec `PROJETS/LIB_CAO` (le dépôt WEB_SUITE_PROJETS cloné dans `PROJETS/`), tous les
  bancs de WEB_CAO tournent sur ta vraie LIB. Sans elle, `banc-catalogue.js` est sauté (gris, avec la raison),
  et les bancs Rust sautent d'eux-mêmes les cas qui la demandent (les autres tournent sur une LIB factice).
- Quelques bancs « Hors CI » demandent encore une préparation à la main (un dossier `PROJETS`
  rempli, des images de référence). Leur sortie le dit.
- La copie de la LIB lie en dur les gros fichiers (les modèles 3D de `Models/`, 290 Mo) : la copie
  est instantanée. C'est sans risque pour la vraie LIB, car WEB_CAO ne réécrit jamais un fichier
  en place (il passe par un nouveau fichier puis un renommage). Si les deux dossiers sont sur des
  disques différents, les fichiers sont copiés.

## Vérifier web_bench lui-même

```
cargo test -p web_bench
```

Deux choses sont lancées. Les tests unitaires éprouvent les fonctions internes. Le banc
`tests/banc_web_bench.rs` pilote l'exécutable de l'extérieur, avec de faux outils : il vérifie le
relais, la sonde, le passage d'un outil à l'autre, les gardes, les routes arrêtées, les bancs, les
rapports et l'arrêt. Les faux outils ont leur `demarrer_<NOM>`, qui lance un petit serveur node
écrit à la volée dans un dossier temporaire, et le faux WEB_CAO une petite caisse Rust (ses bancs
cargo, son outil de préparation) : sans `node` sur le poste, le banc ne vérifie que les options
refusées et saute le reste. La CI de
web_suite lance les deux.
