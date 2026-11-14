# WEB·SUITE

<p align="center">
  <img src="lanceur/docs/intro.webp" alt="Animation d'intro WEB·SUITE : WEB_CAO, WEB_ANTENNA, WEB_3D" width="800">
</p>

<p align="center"><sub><a href="https://raw.githack.com/pilou33620/web_suite/main/lanceur/websuite-intro.html">Voir l'intro interactive</a></sub></p>

Lanceur de trois outils web, utilisables depuis n'importe quel appareil avec un navigateur
(PC, tablette, téléphone) :

| Outil | Rôle |
|---|---|
| [**WEB_CAO**](https://github.com/pilou33620/WEB_CAO) | Schéma & routage PCB, bibliothèque, IPC-2581, simulation SI / PI / RF |
| [**WEB_ANTENNA**](https://github.com/pilou33620/WEB_ANTENNA) | Simulation d'antennes RF (openEMS) |
| [**WEB_3D**](https://github.com/pilou33620/WEB_3D) | Visionneuse 3D (STEP, IGES, STL…) avec mesure, et conception de pièces |

Une carte par outil : **Télécharger** s'il manque, **Lancer** s'il est là. Les projets sont
dans `PROJETS/`, synchronisé avec GitHub. Détails complets : [lanceur/docs/DETAILS.md](lanceur/docs/DETAILS.md).

## Installation

Prérequis : git (sans git, pas de mises à jour ni de synchronisation). Le lanceur est en Rust :
les scripts ci-dessous prennent ses exécutables dans `bin/` — téléchargés depuis la publication
« binaires » de GitHub (Windows x86_64, Linux x86_64 et aarch64, macOS aarch64), ou compilés sur
place si `cargo` est installé. Les trois outils sont livrés de la même façon : rien d'autre à installer.

```
git clone https://github.com/pilou33620/WEB_SUITE.git WEB_TOOLS
```

| Système | Installer les outils | Démarrer |
|---|---|---|
| Windows | `installer.cmd` | `demarrer_WEB_SUITE.cmd` |
| Linux, Raspberry Pi | `./installer.sh` | `./demarrer_WEB_SUITE.sh` |

Sans git ni sources : la publication « binaires » de GitHub porte aussi une archive de production
par système (`web_suite-prod-<système>`), faite par la CI — les exécutables, ces scripts de
démarrage et les pages. On la décompresse et on lance les mêmes scripts ; elle ne se met pas à
jour seule (on reprend l'archive).

L'installateur demande quels outils télécharger (`1`, `1 3`, `4` = tous), ou directement :
`installer.cmd cao 3d` (`./installer.sh cao 3d`).

Dépendances des outils :

- **WEB_CAO**, **WEB_3D** : aucune.
- **WEB_ANTENNA** : openEMS pour lancer des simulations, c'est-à-dire ses deux exécutables
  `openEMS` et `nf2ff`. Sous Windows, l'installateur (ou le bouton **Installer openEMS**) pose
  l'archive binaire dans `WEB_ANTENNA/openEMS/`. Ailleurs, installe openEMS de sorte que ces
  deux exécutables soient dans le `PATH`.

## Utiliser depuis un autre appareil

Sur Raspberry Pi, le lanceur écoute sur le réseau local et n'ouvre pas de
navigateur (ailleurs : `--reseau`). Depuis la tablette, le téléphone ou le PC :

1. ouvrir l'adresse **Réseau** affichée dans le terminal (`https://<adresse>:8100/`) ;
2. saisir le **code d'appairage** à 6 chiffres affiché juste en dessous (une fois par
   appareil et par adresse du serveur).

En mode réseau, la connexion est **chiffrée (HTTPS)** quand la commande `openssl` est présente sur le poste (Raspberry Pi, Linux, macOS ; sous Windows, celle de Git) : une autorité propre au poste, créée une fois dans `~/.web_tools/tls/`, signe le certificat du serveur. Chaque appareil l'installe une fois, depuis le lien de la page du code (`/appairage/ca.pem`) ; sur iPad : Réglages › Profil téléchargé › Installer, puis Général › Informations › Réglages des certificats › activer. Comparer son empreinte SHA-256 avec celle du terminal. Sans `openssl` (ou avec `WEB_TOOLS_HTTP=1`), tout reste en HTTP et le terminal le dit.

## Projets et GitHub

`PROJETS/` est un dépôt git à part
([WEB_SUITE_PROJETS](https://github.com/pilou33620/WEB_SUITE_PROJETS)) :

```
PROJETS/
├── CAO/<projet>/
├── LIB_CAO/
├── ANTENNA/<projet>/
└── 3D/
```

- Le dépôt est **privé** : chaque poste doit être connecté une fois à votre compte GitHub.
  Sous Windows, une fenêtre de connexion s'ouvre toute seule. Sur un Raspberry Pi,
  le lanceur le propose dans son terminal au démarrage (ou, à tout moment :
  `bin/connexion_github --connecter`).
- Il est récupéré au démarrage et avant chaque lancement d'outil.
- À l'arrêt d'un outil, la page propose d'envoyer les modifications sur GitHub (message de
  commit demandé). Le bouton **⇧ Envoyer sur GitHub** reste tant que quelque chose attend.
- Dans WEB_CAO, **Enregistrer** (`Ctrl+S`) et **⇉ PCB** (exporter le schéma vers le PCB)
  écrivent seulement en local, dans `PROJETS/CAO/<projet>`. **Fichier → ☁ Sauvegarder le
  projet → GitHub** (`Ctrl+Maj+S`) enregistre puis envoie tout le projet en un seul commit,
  depuis n'importe quel appareil connecté.
- **Hors ligne**, l'envoi est enregistré localement et repart tout seul au retour de la connexion.
- **Conflit** : rien n'est fusionné automatiquement, le lanceur indique le dossier à régler.

**Connexion à GitHub** : sous Windows, Git Credential Manager s'en charge au premier envoi.
Sur Pi, le lanceur propose au démarrage de se connecter via `gh` : un code est à
saisir sur https://github.com/login/device, depuis n'importe quel appareil.

## Mises à jour

WEB_SUITE et les outils se mettent à jour au démarrage (`git pull --ff-only`). Le bilan
s'affiche dans la page et dans le terminal. Si des fichiers sont modifiés, le lanceur n'y touche
pas ; un outil lancé ensuite refait alors sa propre vérification : il met ses modifications de
côté (`git stash`), se met à jour, puis les reprend (`git stash pop`). Il la refait aussi quand la
dernière vérification date de plus de 10 minutes.
`--sans-maj` : le lanceur ne vérifie ni lui-même ni les outils au démarrage, mais un outil lancé
ensuite fait quand même sa propre vérification.

## Options

```
bin/web_suite [--port 8100] [--racine DOSSIER] [--installer]
                    [--navigateur | --sans-navigateur] [--reseau | --local] [--sans-maj]
```

- `--port` : port du lanceur (8100 par défaut, le suivant s'il est pris).
- `--reseau` / `--local` : écouter sur le réseau local ou seulement sur ce poste.
- `--navigateur` / `--sans-navigateur` : forcer ou empêcher l'ouverture du navigateur.
- `--racine` : dossier des outils et de `PROJETS/`. `--installer` : menu de téléchargement seul.
- `?sans-intro` dans l'adresse saute l'animation.

## Claude Code (MCP)

L'ancien assistant IA intégré aux outils est retiré : Claude, par ce serveur MCP, fait son travail.

`bin/mcp_web_suite` branche Claude Code sur les outils du poste (serveur MCP, sans
dépendance). Les outils doivent déjà tourner. Sur le poste des outils :

```
claude mcp add -s user web_suite -- /chemin/vers/WEB_TOOLS/bin/mcp_web_suite
```

Depuis une autre machine, par SSH (clé SSH obligatoire, aucun port ouvert en plus) :

```
claude mcp add -s user web_suite -- ssh -T -o BatchMode=yes <utilisateur>@<poste> /chemin/vers/WEB_TOOLS/bin/mcp_web_suite
```

Puis `/mcp` dans Claude Code. Windows, clé SSH, dépannage : [DETAILS](lanceur/docs/DETAILS.md#claude-code-mcp).

**Depuis la page, sur n'importe quel appareil** : dans un outil lancé par WEB·SUITE, le bouton
« Claude » en bas à droite ouvre une zone « Demander à Claude ». Ce qu'on y écrit part au lanceur,
qui le passe à Claude Code **de ce poste**, avec le MCP web_suite seul (ni terminal ni fichiers) ;
la réponse s'affiche dans le panneau et les modifications arrivent en cartes. On peut joindre des
fichiers (bouton « Joindre », coller, glisser-déposer, bouton « Vue » pour l'image de l'onglet) : images,
PDF, fichiers texte (md, json, KiCad, csv…), 10 fichiers et 20 Mo par demande. Un menu choisit le
modèle (Opus, Sonnet, Haiku) et, à chaque ouverture, une ligne d'état dit si le lanceur et Claude
Code sont prêts (sans requête au modèle). Il suffit que Claude
Code soit installé sur le poste et connecté une fois (`claude` dans un terminal ; sur un Pi,
système 64 bits) : pas besoin de `claude mcp add` pour cela.
[SPEC-claude-pages](lanceur/docs/SPEC-claude-pages.md).

## Sécurité

- En mode réseau, tout autre appareil doit être appairé (jeton stocké dans un cookie).
  Le même appairage vaut pour les outils (WEB_CAO, WEB_ANTENNA, WEB_3D) : jeton et cookie
  sont partagés par les web tools du poste. Supprimer `~/.web_tools/jeton-reseau.txt`
  révoque tous les appareils.
- Le lanceur refuse les requêtes venant d'autres pages (en-tête `X-WebSuite`, contrôle du `Host`).
- Le serveur MCP n'appelle que `127.0.0.1` et refuse `/api/github`, `/api/envoyer`,
  `/api/identite`, `/api/ia` et `/api/appairer`, et ne fait que lire le lanceur : Claude ne
  publie rien sur GitHub, ne touche pas à la clé d'IA et ne lance ni n'arrête rien.
  Il ne fait que ce que tu peux faire à l'interface : il écrit dans une page par des gestes
  (clic, saisie, touche, souris), et ne lance aucun calcul openEMS. Seuls le routage, les paires,
  le dessin du schéma et l'ouverture d'un fichier 3D restent des outils qui écrivent.
- Dans les pages ouvertes, chaque écriture de Claude (une carte par geste) s'affiche dans le
  panneau « Claude » et n'est appliquée qu'à ton clic (sauf case « Appliquer directement » cochée
  dans l'onglet, décochée à chaque chargement, utile pour enchaîner). Seul le MCP de ce poste peut donner des
  ordres aux pages ; une page d'un autre site est refusée.

## Banc d'essai (web_bench)

`demarrer_web_bench.cmd` (Windows) ou `./demarrer_web_bench.sh` ouvre un banc d'essai visible.
Il fonctionne pour les trois outils : l'outil choisi s'ouvre en entier dans la page, un
**clic-partout** clique sur tous ses boutons et relève les erreurs JS, réseau et serveur, et
l'onglet **Bancs** lance ses bancs (scripts node `banc-*`, `cargo test`). L'outil tourne sur des dossiers temporaires :
tes projets et ta LIB ne sont pas touchés, et rien ne part sur GitHub ni dans openEMS. Chaque
rapport est gardé dans `web_bench/rapports/`, avec un `.md` détaillé à donner à Claude pour les
corrections. Détails : [web_bench/README.md](web_bench/README.md).

## Divers

- Journaux des outils : `lanceur/journaux/<outil>.log` (bouton **Journal**).
- `bin/maj_git` : état git de WEB_SUITE, des outils et de `PROJETS/`, avec
  commit / pull / push guidés.
- Tests : `cargo test --workspace` lance tout ce qui est en Rust — les tests unitaires (commun,
  ipc2581, lanceur, web_bench) et les bancs, qui pilotent les exécutables en boîte noire :
  `lanceur/tests/` (`banc_serveur`, `banc_dependances`, `banc_mcp`, `banc_savoir`, `banc_traces_ia` ;
  les deux derniers lisent aussi les dépôts des outils, quand ils sont clonés) et
  `web_bench/tests/banc_web_bench.rs`. Un seul banc : `cargo test -p web_suite_lanceur --test banc_mcp`.
  Reste `node lanceur/banc_pilote.js` (la page). La CI lance le tout.
