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
| [**WEB_3D**](https://github.com/pilou33620/WEB_3D) | Visionneuse 3D (STEP, IGES, STL…) avec mesure |

Une carte par outil : **Télécharger** s'il manque, **Lancer** s'il est là. Les projets sont
dans `PROJETS/`, synchronisé avec GitHub. Détails complets : [lanceur/docs/DETAILS.md](lanceur/docs/DETAILS.md).

## Installation

Prérequis : Python 3.8+ et git (sans git, pas de mises à jour ni de synchronisation).

```
git clone https://github.com/pilou33620/WEB_SUITE.git WEB_TOOLS
```

| Système | Installer les outils | Démarrer |
|---|---|---|
| Windows | `installer.cmd` | `demarrer_WEB_SUITE.cmd` |
| Linux, Raspberry Pi | `./installer.sh` | `./demarrer_WEB_SUITE.sh` |
| Android (Termux) | `pkg install python git` puis `./installer.sh` | `./demarrer_WEB_SUITE.sh` |

L'installateur demande quels outils télécharger (`1`, `1 3`, `4` = tous), ou directement :
`python lanceur/installer.py cao 3d`.

Dépendances des outils, proposées par l'installateur :

- **WEB_CAO** : `numpy`, `scipy` (facultatifs). Sous Termux, installés par `pkg`.
- **WEB_ANTENNA** : openEMS, installé automatiquement sous Windows seulement
  (nécessite Python 3.10 ou 3.11 64 bits). Ailleurs, voir `WEB_ANTENNA/requirements.txt`.
- **WEB_3D** : aucune.

## Utiliser depuis un autre appareil

Sur Raspberry Pi et Termux, le lanceur écoute sur le réseau local et n'ouvre pas de
navigateur (ailleurs : `--reseau`). Depuis la tablette, le téléphone ou le PC :

1. ouvrir l'adresse **Réseau** affichée dans le terminal (`http://<adresse>:8100/`) ;
2. saisir le **code d'appairage** à 6 chiffres affiché juste en dessous (une fois par
   appareil et par adresse du serveur).

Sous Termux, garder `termux-wake-lock` actif.

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
  Sous Windows, une fenêtre de connexion s'ouvre toute seule. Sur Termux ou un Raspberry Pi,
  le lanceur le propose dans son terminal au démarrage ; depuis web_launcher, choisir l'action
  **Connecter ce serveur à GitHub (projets privés)**.
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
Sur Pi et Termux, le lanceur propose au démarrage de se connecter via `gh` : un code est à
saisir sur https://github.com/login/device, depuis n'importe quel appareil.

## Mises à jour

WEB_SUITE et les outils se mettent à jour au démarrage (`git pull --ff-only`), jamais
par-dessus des fichiers modifiés. Le bilan s'affiche dans la page et dans le terminal.
`--sans-maj` désactive tout.

## Options

```
python lanceur/web_suite.py [--port 8100] [--racine DOSSIER] [--installer]
                    [--navigateur | --sans-navigateur] [--reseau | --local] [--sans-maj]
```

- `--reseau` / `--local` : écouter sur le réseau local ou seulement sur ce poste.
- `--navigateur` / `--sans-navigateur` : forcer ou empêcher l'ouverture du navigateur.
- `--racine` : dossier des outils. `--installer` : menu de téléchargement seul.
- `?sans-intro` dans l'adresse saute l'animation.

## Sécurité

- En mode réseau, tout autre appareil doit être appairé (jeton stocké dans un cookie).
  Le même appairage vaut pour les outils (WEB_CAO, WEB_ANTENNA, WEB_3D) : jeton et cookie
  sont partagés par les web tools du poste. Supprimer `~/.web_tools/jeton-reseau.txt`
  révoque tous les appareils.
- Le lanceur refuse les requêtes venant d'autres pages (en-tête `X-WebSuite`, contrôle du `Host`).

## Divers

- Journaux des outils : `lanceur/journaux/<outil>.log` (bouton **Journal**).
- `python maj_git.py` : état git de WEB_SUITE, des outils et de `PROJETS/`, avec
  commit / pull / push guidés.
- Tests : `python lanceur/projets.py`, `python lanceur/banc_serveur.py`,
  `python lanceur/banc_termux.py` (affichent `OK`).
