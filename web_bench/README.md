# web_bench — banc d'essai visible de WEB_CAO, WEB_ANTENNA et WEB_3D

L'outil choisi s'ouvre en entier à gauche, et reste utilisable à la main. À droite, un panneau
lance les essais et montre les résultats. Les essais se lancent outil par outil, et le menu
**Outil** passe de l'un à l'autre. Chaque outil démarre la première fois qu'on le choisit.

| Volet | Ce qu'il fait |
|---|---|
| **Clic-partout** | Clique un par un tous les boutons, menus, onglets, cases, listes et liens de chaque page, et relève ce qui casse : erreur JavaScript, promesse rejetée, requête en échec, erreur Python côté serveur (Traceback), page bloquée plus de 1,5 s, requêtes qui ne finissent pas. Les éléments qui apparaissent après un clic (menu, fenêtre) sont essayés juste après, en rouvrant le menu si besoin. Une fenêtre déjà vue qui masque la page est refermée par son bouton (Annuler, ×, OK). |
| **Bancs** | Trouve les scripts `banc-*.py / .js / .mjs` de l'outil, plus les essais que sa CI lance sous un autre nom (`harness.js` des éditeurs, `harness-sim.js`, `appairage.py` et `pilote.py` de WEB_3D). Il les lance un par un, **comme la CI** : avec les mêmes options de node, et après les mêmes préparations (assemblage `build-monofichier.py`, puis `harness.js` avant `banc-ipc2581-export.py`). Ceux que la CI lance sont cochés d'office. Vert : le script a rendu 0. Un clic montre sa sortie. |
| **Rapports** | Chaque clic-partout terminé ou arrêté est gardé dans `web_bench/rapports/`, en `.json` (tout) et en `.md` (lisible). Un clic recharge un rapport, et **.md** le télécharge. |
| **Journal** | La sortie du serveur de l'outil, avec les Traceback en rouge et les requêtes arrêtées par le banc. |

## Démarrer

Depuis la racine de web_suite, une fois les outils installés (`installer.cmd` / `installer.sh`) :

| Système | Démarrer |
|---|---|
| Windows | `demarrer_web_bench.cmd` |
| Linux, Raspberry Pi | `./demarrer_web_bench.sh` |

Le navigateur s'ouvre sur `http://127.0.0.1:8200/__banc/`.

Options :
- `--outil cao|antenna|3d` : l'outil ouvert au démarrage ;
- `--racine DOSSIER` : si les outils ne sont pas à la racine de web_suite ;
- `--port` et `--sans-navigateur`.

Prérequis : Python 3.8+. Ajoute Node.js pour les bancs JavaScript, et Chrome, Chromium ou Edge
pour les bancs qui ouvrent une page (ceux de WEB_3D surtout).

## Le rapport, pour les corrections

Le `.md` de chaque rapport est fait pour être donné tel quel à Claude. Il contient :

- **le contexte** : la version essayée de l'outil (branche, commit, date, message, et s'il y a des
  modifications locales), le navigateur et la taille du cadre, les pages, les réglages et la liste
  noire ;
- **les problèmes regroupés** : un même message au même endroit du code ne compte qu'une fois,
  avec son nombre d'apparitions et la liste des boutons qui le déclenchent. Les erreurs viennent
  avant les avertissements. Pour chaque problème :
  - **où dans le code** : le fichier et la ligne JS (`WEB_CAO/editeur-pcb/js/29-draftsman.js:82:13`),
    le fichier Python du Traceback, ou la route appelée, avec la ligne JS qui l'a appelée ;
  - **l'élément** : son libellé, son sélecteur CSS et sa balise ouvrante (id, classes) ;
  - **pour reproduire** : la page à ouvrir et les derniers gestes faits avant le clic fautif
    (clics, menus rouverts, fenêtres refermées, Échap) ;
  - **la pile d'appels JS**, sans les lignes du banc, ou **le Traceback Python** complet ;
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
  coupée, ou une erreur Python dans le serveur.
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
  bancs de WEB_CAO tournent sur ta vraie LIB. Sans elle, le banc fait comme la CI de WEB_CAO :
  `banc-lib-routes.py`, `banc-rf.py` et `banc-catalogue.js` sont sautés (gris, avec la raison), et
  les autres tournent sur une LIB factice minimale (`WEB_CAO_LIB_MINIMALE=1`).
- Quelques bancs « Hors CI » demandent encore une préparation à la main (un dossier `PROJETS`
  rempli, des images de référence). Leur sortie le dit.
- La copie de la LIB lie en dur les gros fichiers (les modèles 3D de `Models/`, 290 Mo) : la copie
  est instantanée. C'est sans risque pour la vraie LIB, car WEB_CAO ne réécrit jamais un fichier
  en place (il passe par un nouveau fichier puis un renommage). Si les deux dossiers sont sur des
  disques différents, les fichiers sont copiés.

## Vérifier web_bench lui-même

```
python web_bench/banc_web_bench.py
```

Ce banc utilise de faux outils. Il vérifie le relais, la sonde, le passage d'un outil à l'autre,
les gardes, les routes arrêtées, les bancs et les rapports. Il est lancé par la CI de web_suite.
