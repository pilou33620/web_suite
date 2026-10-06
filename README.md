# WEB·SUITE

<p align="center">
  <img src="lanceur/docs/intro.webp" alt="Animation d'intro WEB·SUITE : WEB_CAO, WEB_ANTENNA, WEB_3D" width="800">
</p>

<p align="center"><sub>Intro du lanceur (<code>lanceur/websuite-intro.html</code>) —
<a href="https://raw.githack.com/pilou33620/web_suite/main/lanceur/websuite-intro.html">voir la version interactive</a></sub></p>

Lanceur des trois outils web :

| Outil | Rôle | Dépôt |
|---|---|---|
| **WEB_CAO** | Schéma & routage PCB, Gestion LIB, visionneuse IPC-2581, simulation SI / PI / RF, vérification de la carte | https://github.com/pilou33620/WEB_CAO |
| **WEB_ANTENNA** | Simulation d'antennes RF (openEMS) : import de carte, mode conception, balayage, champs | https://github.com/pilou33620/WEB_ANTENNA |
| **WEB_3D** | Visionneuse 3D (STEP / IGES / BREP / 3MF / OBJ / STL), mesure façon Fusion 360 | https://github.com/pilou33620/WEB_3D |

Le détail de chaque outil est dans son propre README.

Au démarrage, le lanceur joue l'animation d'intro (`websuite-intro.html`) pendant qu'il cherche
les mises à jour des outils, puis affiche une carte par outil : **Télécharger** s'il manque,
**Lancer** s'il est là. Lancer démarre le serveur de l'outil en arrière-plan et ouvre sa page
dans un nouvel onglet. Les projets de tous les outils vivent dans `PROJETS/`, synchronisé
avec GitHub (voir [Projets](#projets)).

## Installation

Python 3.8+ suffit pour WEB_SUITE : bibliothèque standard seule (voir `lanceur/requirements.txt`).

```
WEB_TOOLS/                  <- ce dépôt
├── installer.cmd / .sh     <- télécharge les outils (Windows / Linux, Raspberry Pi)
├── demarrer_WEB_SUITE.cmd / .sh  <- lance WEB·SUITE
├── maj_git.py              <- état git de WEB_SUITE, des outils et de PROJETS (voir plus bas)
├── lanceur/                <- code du lanceur : web_suite.py, installer.py, outils.py, projets.py, pages
├── WEB_CAO/                <- clonés ici par l'installateur, chacun avec son dépôt git,
├── WEB_ANTENNA/               ignorés par le .gitignore de WEB_SUITE
├── WEB_3D/
└── PROJETS/                <- projets + LIB_CAO, dépôt WEB_SUITE_PROJETS (cloné au démarrage)
```

1. Cloner ce dépôt : `git clone https://github.com/pilou33620/WEB_SUITE.git WEB_TOOLS`.
2. Télécharger les outils : double-clic sur **`installer.cmd`** (ou `python lanceur/installer.py`).
   Le menu demande lequel installer : `1`, `2`, `3`, plusieurs (`1 3`) ou `4` pour les trois.
   Sans menu : `python lanceur/installer.py cao 3d`, `python lanceur/installer.py tous`.
3. Lancer : double-clic sur **`demarrer_WEB_SUITE.cmd`** (ou `python lanceur/web_suite.py`).

Sous Linux / Raspberry Pi, mêmes étapes avec `./installer.sh` et `./demarrer_WEB_SUITE.sh`
(`python3` et `git` : `sudo apt install python3 git`). Sur Raspberry Pi, le lanceur
n'ouvre pas le navigateur et écoute sur le réseau local : ouvrir depuis n'importe quel appareil
l'adresse **Réseau** affichée (`http://<ip-du-pi>:8100/`), ou lancer avec `--navigateur` ;
l'installation automatique d'openEMS reste réservée à Windows.

Sous **Termux** (Android) : `pkg install python git`, puis `./installer.sh`. L'installateur
détecte Termux et installe `numpy` / `scipy` de WEB_CAO par `pkg install python-numpy
python-scipy` (paquets précompilés) au lieu de pip, qui n'a pas de roue Android et
échouerait en tentant de les compiler (cmake, ninja). Lancer WEB_SUITE avec la `python` de
Termux, hors venv, pour qu'elle voie ces paquets. Comme sur Raspberry Pi, `./demarrer_WEB_SUITE.sh`
y démarre directement en mode réseau, sans ouvrir de navigateur : ouvrir l'adresse **Réseau**
affichée depuis l'autre appareil (garder `termux-wake-lock` actif).

Au premier démarrage sans aucun outil, `web_suite.py` pose la même question en
console. La page propose aussi **Télécharger** sur chaque carte et un bouton
pour tout télécharger d'un coup.

git est recommandé (https://git-scm.com) : sans lui, les outils sont téléchargés en zip,
ne se mettent plus à jour, et les projets ne sont pas synchronisés.

Dépendances propres aux outils :

- **WEB_CAO** : `numpy`, `scipy` facultatifs (solveurs SI/PI). L'installateur propose de les installer
  (par pip, ou par `pkg` sous Termux).
- **WEB_ANTENNA** : openEMS pour lancer les simulations. L'installateur propose de
  l'installer (ou `python lanceur/installer.py antenna`, ou le bouton **Installer openEMS** de
  la carte) : il télécharge l'archive openEMS v0.0.36 (~50 Mo) dans `WEB_ANTENNA/openEMS/`,
  crée `WEB_ANTENNA/env/` et y installe les paquets. Seul prérequis à installer soi-même :
  **Python 3.10 ou 3.11 en 64 bits** (les roues openEMS n'existent que pour ces versions),
  par exemple `winget install Python.Python.3.11`. Windows seulement ; ailleurs, voir le
  guide en tête de `WEB_ANTENNA/requirements.txt`.
- **WEB_3D** : aucune.

## Mises à jour

Chaque dépôt garde son propre git et se met à jour par `git pull --ff-only`, jamais
par-dessus des fichiers modifiés (« des fichiers sont modifiés » s'affiche alors et rien
n'est touché) :

- **WEB_SUITE** se vérifie avant d'ouvrir son serveur et redémarre seul s'il vient d'être mis à jour.
- **Les outils installés** se vérifient tous en parallèle pendant l'animation d'intro :
  la carte affiche « Mise à jour… » et le bandeau de l'intro « Mises à jour · WEB_CAO ✓ … ».
  L'intro attend la fin (30 s au plus) avant de rendre la main. Un outil lancé dans les
  10 minutes qui suivent reçoit `--sans-maj` et démarre sans refaire la vérification ;
  au-delà, il revérifie lui-même.
- `--sans-maj` coupe les deux.

## Projets

Les projets et la bibliothèque de composants de WEB_CAO sont dans `PROJETS/`, un dépôt git
à part (https://github.com/pilou33620/WEB_SUITE_PROJETS), ignoré par WEB_SUITE :

```
PROJETS/
├── projets.txt       <- liste des projets par outil, réécrite à chaque envoi
├── CAO/<projet>/     <- WEB_CAO --projets
├── LIB_CAO/          <- WEB_CAO --lib (catalogue, empreintes, symboles, modèles)
├── ANTENNA/<projet>/ <- WEB_ANTENNA --projets (les calculs/ openEMS ne sont pas envoyés)
└── 3D/               <- WEB_3D --projets (modèles ouverts et rangés depuis la visionneuse)
```

- **Au démarrage**, le lanceur le clone (ou le crée en local si GitHub est vide ou
  injoignable), puis le met à jour (`pull --rebase`). Un `PROJETS/` déjà rempli mais sans
  git est cloné à côté : ses fichiers absents de GitHub y sont ajoutés, et l'ancien dossier
  reste en sauvegarde (`PROJETS.avant-git-<date>`).
- **Fins de ligne et zip** : `.gitattributes` (`* text=auto`) normalise le dépôt entre
  Windows et le Pi ; les `.zip` de `LIB_CAO` ne sont pas envoyés (leur contenu déballé, si).
- **Avant chaque lancement d'outil**, il tire à nouveau : un projet ou la LIB modifiés sur
  un autre poste sont donc à jour. L'outil est lancé pointé sur ses dossiers de `PROJETS/`.
- **À l'arrêt d'un outil**, s'il a des modifications, la page demande un message de commit
  puis envoie (commit + pull + push) ses seuls dossiers. Annuler remet à plus tard : le bouton
  **⇧ Envoyer sur GitHub** reste sur la carte tant que quelque chose attend, modification
  à commiter ou commit pas encore poussé.
- **Poste neuf** (Raspberry Pi…) : si git ne connaît ni nom ni e-mail, la page les demande
  au premier envoi et les enregistre dans la config du dépôt `PROJETS` seulement (pas en global).
  Au premier push, Git Credential Manager ouvre sa fenêtre de connexion GitHub.
- **Conflit** : rien n'est fusionné automatiquement ; le lanceur annule le rebase et indique
  le dossier où régler la situation à la main (git y tourne en `LC_ALL=C` : un git traduit
  ne trompe pas la détection, et PROJETS n'est jamais laissé en plein rebase). Hors ligne, tout reste en local et part au
  prochain envoi réussi.

### Sauvegarder depuis une tablette

Cas typique : WEB_SUITE tourne sur un téléphone (Termux) ou un Raspberry Pi, et l'on
dessine sur l'iPad.

**Une seule fois, sur le serveur** (téléphone ou Pi), connecter git à votre compte GitHub.
Sous Windows, Git Credential Manager s'en charge au premier envoi ; ni Termux ni le Pi
n'en ont, et l'envoi serait refusé (le lanceur le dit alors en clair). Dans un terminal
du serveur :

```
pkg install gh            # Termux      (Raspberry Pi : sudo apt install gh)
gh auth login             # GitHub.com → HTTPS → « Login with a web browser »
gh auth setup-git
```

`gh auth login` affiche un code à 8 caractères : l'ouvrir sur l'iPad, à l'adresse
https://github.com/login/device, saisir le code, valider. `gh auth setup-git` dit ensuite
à git d'utiliser cette connexion ; elle reste valable d'un démarrage à l'autre.
Sur le Pi, le terminal peut être une session SSH depuis l'iPad.

**Ensuite, à chaque fois, depuis l'iPad** :

1. ouvrir l'adresse **Réseau** du lanceur (celle qui se termine par `?jeton=…`, à garder
   en favori) et lancer WEB_CAO ;
2. sur la page d'accueil de WEB_CAO, ouvrir ou créer le projet : il vit sur le serveur,
   dans `PROJETS/CAO/<projet>/` ;
3. dans l'éditeur (schéma ou PCB), **Fichier → Enregistrer + GitHub**, ou le bouton
   **💾 Sauver** de la barre tactile. Le document est écrit sur le serveur, puis envoyé :
   un message de commit est demandé (Annuler = enregistrer sans envoyer), et au tout
   premier envoi, votre nom et votre e-mail pour git.

C'est le lanceur qui envoie (commit + pull + push de `CAO/` et `LIB_CAO/`), exactement comme
le bouton **⇧ Envoyer sur GitHub** de sa page : WEB_CAO lui relaie la demande. Le relai
exige le même jeton que le lanceur — l'iPad le porte déjà, dans le cookie posé par
l'adresse **Réseau** — et un autre appareil du réseau ne peut donc pas pousser sur
GitHub avec les identifiants du serveur. **Enregistrer** seul (Ctrl+S) écrit sur le
serveur sans rien envoyer ; arrêter l'outil depuis le lanceur propose toujours d'envoyer
ce qui attend.

## Options

```
python lanceur/web_suite.py [--port 8100] [--racine DOSSIER] [--installer]
                    [--navigateur | --sans-navigateur] [--reseau | --local] [--sans-maj]
```

- `--reseau` : le lanceur et les outils écoutent sur le réseau local ; l'adresse
  **Réseau** affichée s'ouvre depuis une tablette, un téléphone ou un autre poste.
  C'est le défaut sur Raspberry Pi et sous Termux. WEB_CAO y est lancé avec `--projets-reseau` pour que
  ses projets et la LIB restent ouverts depuis l'autre appareil (confinés à `PROJETS/`).
  L'adresse **Réseau** se termine par `?jeton=…` : le lanceur exige ce jeton de tout
  autre appareil (le premier passage le range dans un cookie, l'adresse se garde en favori).
  Il est conservé dans `lanceur/jeton-reseau.txt` ; supprimer ce fichier en crée un neuf.
  Les outils, eux, restent sans authentification : réseau de confiance uniquement.
  `--local` force l'écoute sur `127.0.0.1` seulement (défaut ailleurs).

- Le navigateur s'ouvre automatiquement, sauf sur Raspberry Pi et sous Termux.
  `--navigateur` force l'ouverture, `--sans-navigateur` l'empêche partout.

- `--racine` : dossier contenant les outils (défaut : la racine du dépôt, parent de `lanceur/`).
- `--installer` : ouvre seulement le menu de téléchargement.
- `--sans-maj` : ne vérifier les mises à jour ni de WEB_SUITE ni des outils.
- La page accepte `?sans-intro` pour sauter l'animation. Elle ne la rejoue pas
  non plus quand on recharge l'onglet ; le bouton **↻ Intro** la relance.

## Fonctionnement

- Le lanceur n'écoute que sur `127.0.0.1` (sauf `--reseau`) et refuse les requêtes d'action sans
  l'en-tête `X-WebSuite` : aucune page tierce ne peut lancer ou télécharger un outil.
  Il refuse aussi tout en-tête `Host` qui n'est ni une adresse IP, ni `localhost`, ni le nom
  du poste : une page piégée par DNS rebinding ne peut pas se faire passer pour lui.
- Les outils sont démarrés avec `--local --sans-navigateur` (sans `--local` en mode réseau) sur leur port habituel
  (CAO 8000, ANTENNA 8732, 3D 8139) ou le suivant s'il est pris, avec `--projets` / `--lib`
  vers `PROJETS/`, et `--sans-maj` si le lanceur vient de les vérifier.
- Leur sortie console va dans `lanceur/journaux/<outil>.log`, lisible depuis le bouton
  **Journal** de chaque carte.
- Fermer la fenêtre de WEB_SUITE (ou Ctrl+C) arrête les outils lancés depuis elle. Sous Linux,
  fermer le terminal (SIGHUP) ou `kill` (SIGTERM) aussi.

## maj_git.py

`python maj_git.py` fait le tour de WEB_SUITE, des trois outils et de `PROJETS/` :
modifications locales, commits à pousser ou à récupérer. Pour chaque dépôt en retard, il
propose de commiter, de tirer (`pull --rebase`) puis de pousser, en demandant à chaque étape.

## Vérifications

```
python lanceur/projets.py
```

Joue deux postes et un faux GitHub (dépôt nu) : création, identité git manquante, envoi,
clonage sur l'autre poste, `projets.txt`. Affiche `OK` si tout passe.

```
python lanceur/banc_serveur.py
```

Serveur du lanceur avec un gestionnaire factice : `Host` étranger, `X-WebSuite`, taille
des requêtes, jeton du mode réseau (refus, cookie, ce poste exempté). Affiche `OK`.

```
python lanceur/banc_termux.py
```

Joue Termux avec un faux `pkg` : `numpy` / `scipy` partent à `pkg install`, rien à pip, et un
paquet toujours introuvable après coup est signalé. Affiche `OK`.
