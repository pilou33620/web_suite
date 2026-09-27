# WEB·SUITE

<p align="center">
  <img src="docs/intro.webp" alt="Animation d'intro WEB·SUITE : WEB_CAO, WEB_ANTENNA, WEB_3D" width="800">
</p>

<p align="center"><sub>Intro du lanceur (<code>websuite-intro.html</code>) —
<a href="https://raw.githack.com/pilou33620/web_suite/main/websuite-intro.html">voir la version interactive</a></sub></p>

Lanceur des trois outils web :

| Outil | Rôle | Dépôt |
|---|---|---|
| **WEB_CAO** | Schéma & routage PCB | https://github.com/pilou33620/WEB_CAO |
| **WEB_ANTENNA** | Simulation d'antennes RF (openEMS) | https://github.com/pilou33620/WEB_ANTENNA |
| **WEB_3D** | Modélisation & assemblage 3D | https://github.com/pilou33620/WEB_3D |

Au démarrage, le lanceur joue l'animation d'intro (`websuite-intro.html`), puis affiche une
carte par outil : **Télécharger** s'il manque, **Lancer** s'il est là. Lancer démarre
le serveur de l'outil en arrière-plan et ouvre sa page dans un nouvel onglet.

## Installation

Python 3.8+ suffit pour WEB_SUITE : bibliothèque standard seule (voir `requirements.txt`).

```
WEB_TOOLS/         <- ce dépôt : web_suite.py, installer.py, index.html, websuite-intro.html...
├── WEB_CAO/       <- clonés ici par l'installateur, chacun avec son dépôt git,
├── WEB_ANTENNA/      ignorés par le .gitignore de WEB_SUITE
└── WEB_3D/
```

1. Cloner ce dépôt : `git clone https://github.com/pilou33620/WEB_SUITE.git WEB_TOOLS`.
2. Télécharger les outils : double-clic sur **`installer.cmd`** (ou `python installer.py`).
   Le menu demande lequel installer : `1`, `2`, `3`, plusieurs (`1 3`) ou `4` pour les trois.
   Sans menu : `python installer.py cao 3d`, `python installer.py tous`.
3. Lancer : double-clic sur **`demarrer_WEB_SUITE.cmd`** (ou `python web_suite.py`).

Au premier démarrage sans aucun outil, `web_suite.py` pose la même question en
console. La page propose aussi **Télécharger** sur chaque carte et un bouton
pour tout télécharger d'un coup.

Chaque outil garde son propre dépôt git et se met à jour seul au démarrage.
git est recommandé (https://git-scm.com) : sans lui, l'outil est téléchargé en zip
et ne se met plus à jour.

Dépendances propres aux outils :

- **WEB_CAO** : `numpy`, `scipy` facultatifs (solveurs SI/PI). L'installateur propose de les installer.
- **WEB_ANTENNA** : openEMS pour lancer les simulations, à installer à la main
  (Python 3.10/3.11), voir le guide en tête de `WEB_ANTENNA/requirements.txt`.
- **WEB_3D** : aucune.

## Options

```
python web_suite.py [--port 8100] [--racine DOSSIER] [--installer]
                    [--sans-navigateur] [--sans-maj]
```

- `--racine` : dossier contenant les outils (défaut : le dossier de `web_suite.py`).
- `--installer` : ouvre seulement le menu de téléchargement.
- `--sans-maj` : ne pas vérifier les mises à jour de WEB_SUITE sur GitHub.
- La page accepte `?sans-intro` pour sauter l'animation. Elle ne la rejoue pas
  non plus quand on recharge l'onglet ; le bouton **↻ Intro** la relance.

## Fonctionnement

- Le lanceur n'écoute que sur `127.0.0.1` et refuse les requêtes d'action sans
  l'en-tête `X-WebSuite` : aucune page tierce ne peut lancer ou télécharger un outil.
- Les outils sont démarrés avec `--local --sans-navigateur` sur leur port habituel
  (CAO 8000, ANTENNA 8732, 3D 8139) ou le suivant s'il est pris.
- Leur sortie console va dans `journaux/<outil>.log`, lisible depuis le bouton
  **Journal** de chaque carte.
- Fermer la fenêtre de WEB_SUITE (ou Ctrl+C) arrête les outils lancés depuis elle.
