# WEB·SUITE — détails

Compléments au [README](../../README.md).

## Arborescence

```
WEB_TOOLS/                  <- ce dépôt
├── installer.cmd / .sh     <- télécharge les outils
├── demarrer_WEB_SUITE.cmd / .sh  <- lance WEB·SUITE
├── maj_git.py              <- état git de WEB_SUITE, des outils et de PROJETS
├── lanceur/                <- web_suite.py, installer.py, outils.py, projets.py, pages
├── WEB_CAO/                <- clonés par l'installateur, chacun avec son dépôt git,
├── WEB_ANTENNA/               ignorés par le .gitignore de WEB_SUITE
├── WEB_3D/
└── PROJETS/                <- dépôt WEB_SUITE_PROJETS (cloné au démarrage)
    ├── projets.txt         <- liste des projets par outil, réécrite à chaque envoi
    ├── CAO/<projet>/       <- WEB_CAO --projets
    ├── LIB_CAO/            <- WEB_CAO --lib (catalogue, empreintes, symboles, modèles)
    ├── ANTENNA/<projet>/   <- WEB_ANTENNA --projets (les calculs openEMS ne sont pas envoyés)
    └── 3D/                 <- WEB_3D --projets
```

Au premier démarrage sans aucun outil, `web_suite.py` pose la question de l'installateur
en console. La page propose aussi **Télécharger** sur chaque carte et un bouton pour tout
télécharger d'un coup. Sans git, les outils sont téléchargés en zip.

## Installation selon le système

- **Linux / Raspberry Pi** : `sudo apt install python3 git`.
- **Termux** : l'installateur installe `numpy` / `scipy` par `pkg install python-numpy
  python-scipy` (paquets précompilés) au lieu de pip, qui n'a pas de roue Android et
  échouerait en tentant de les compiler. Lancer WEB_SUITE avec la `python` de Termux, hors
  venv, pour qu'elle voie ces paquets.
- **openEMS (WEB_ANTENNA, Windows)** : l'installateur (ou `python lanceur/installer.py
  antenna`, ou le bouton **Installer openEMS** de la carte) télécharge openEMS v0.0.36
  (~50 Mo) dans `WEB_ANTENNA/openEMS/`, crée `WEB_ANTENNA/env/` et y installe les paquets.
  Les roues openEMS n'existent que pour Python 3.10 / 3.11 64 bits
  (`winget install Python.Python.3.11`).

## Appairage d'un appareil

En mode réseau, le lanceur exige un jeton de tout appareil autre que le serveur.

- L'appareil ouvre `http://<adresse>:8100/` et saisit le **code d'appairage** à 6 chiffres
  affiché dans le terminal du serveur. Le bon code lui donne le jeton, rangé dans un cookie
  valable un an.
- Le code sert une fois et vit 10 minutes ; 5 erreurs l'annulent avec une minute
  d'attente. Chaque nouveau code s'affiche dans le terminal, une série d'essais s'y voit.
- Le cookie vaut pour une adresse : si celle du serveur change (partage de connexion, 4G,
  autre Wi-Fi), on ressaisit un code.
- L'adresse complète `?jeton=…`, toujours affichée, marche aussi sans code.
- Le jeton est conservé dans `lanceur/jeton-reseau.txt` ; le supprimer en crée un neuf
  et oblige à réappairer tous les appareils.
- WEB_CAO y est lancé avec `--projets-reseau` pour que ses projets et la LIB restent
  accessibles depuis l'autre appareil (confinés à `PROJETS/`).

## Mises à jour

Chaque dépôt garde son propre git et se met à jour par `git pull --ff-only`. Si des
fichiers sont modifiés, « des fichiers sont modifiés » s'affiche et rien n'est touché.

- **WEB_SUITE** se vérifie avant d'ouvrir son serveur et redémarre seul s'il vient d'être
  mis à jour.
- **Les outils installés** se vérifient en parallèle pendant l'animation d'intro : la carte
  affiche « Mise à jour… » et le bandeau « Mises à jour · WEB_CAO ✓ … ». L'intro attend
  la fin (30 s au plus). Un outil lancé dans les 10 minutes qui suivent reçoit
  `--sans-maj` ; au-delà, il revérifie lui-même.
- **Terminal** : une ligne pour WEB_SUITE et une par outil, par exemple
  `WEB_CAO  installé · Mis à jour (2 commit(s)).`, `installé · À jour.` ou
  `GitHub injoignable : mise à jour ignorée.`. Un outil qui dépasse 30 s s'affiche plus bas
  dès qu'il a fini.

## Synchronisation de PROJETS

- **Au démarrage**, le lanceur clone `PROJETS/` (ou le crée en local si GitHub est vide ou
  injoignable), puis le met à jour (`pull --rebase`). Un `PROJETS/` déjà rempli mais sans
  git est cloné à côté : ses fichiers absents de GitHub y sont ajoutés, et l'ancien dossier
  reste en sauvegarde (`PROJETS.avant-git-<date>`).
- **Avant chaque lancement d'outil**, il tire à nouveau. L'outil est lancé pointé sur ses
  dossiers de `PROJETS/`.
- **À l'arrêt d'un outil**, s'il a des modifications, la page demande un message de commit
  puis envoie (commit + pull + push) ses seuls dossiers. Annuler remet à plus tard : le
  bouton **⇧ Envoyer sur GitHub** reste tant qu'une modification ou un commit attend.
- **Fins de ligne et zip** : `.gitattributes` (`* text=auto`) normalise le dépôt entre
  systèmes ; les `.zip` de `LIB_CAO` ne sont pas envoyés (leur contenu déballé, si).
- **Identité git** : si git ne connaît ni nom ni e-mail, la page les demande au premier
  envoi et les enregistre dans la config du dépôt `PROJETS` seulement.
- **Conflit** : le lanceur annule le rebase et indique le dossier où régler la situation
  à la main. git y tourne en `LC_ALL=C` (un git traduit ne trompe pas la détection), et
  PROJETS n'est jamais laissé en plein rebase.
- **Hors ligne** : le commit est fait sur le poste et rien n'est perdu. Le lanceur renvoie
  ce qui attend dès son démarrage, puis toutes les minutes (jusqu'à 5 minutes d'intervalle
  si la connexion tarde). La page affiche « ⏳ n enregistrement(s) pas encore sur
  GitHub », puis confirme l'envoi. Seuls les enregistrements déjà commités partent ainsi :
  des modifications jamais envoyées attendent le prochain envoi.

## Enregistrer depuis un autre appareil

Cas typique : le serveur est un téléphone (Termux), un Raspberry Pi ou un PC, et l'on
travaille depuis une tablette, un téléphone ou un autre poste.

**Une fois, sur le serveur** : connecter git à GitHub. Sous Windows, Git Credential Manager
s'en charge au premier envoi. Sur Termux et le Pi, le lanceur le propose au démarrage :

```
  GitHub : ce serveur n'est pas encore connecté à votre compte.
  Se connecter maintenant ? [O/n, j = ne plus demander] :
```

Entrée : il installe `gh` s'il manque (`pkg install gh` ou `sudo apt-get install gh`),
lance `gh auth login` (code à 8 caractères à saisir sur https://github.com/login/device
depuis n'importe quel appareil), puis `gh auth setup-git`. Le nom et l'e-mail des commits
sont pris au compte GitHub. `n` remet la question au prochain démarrage, `j` ne la pose
plus (supprimer `PROJETS/.git/websuite-sans-github` pour la retrouver). Le terminal peut
être une session SSH. À la main : `gh auth login` puis `gh auth setup-git`.

**Ensuite, depuis l'appareil** :

1. ouvrir l'adresse **Réseau** du lanceur et, si besoin, saisir le code d'appairage ;
   lancer WEB_CAO ;
2. ouvrir ou créer le projet : il vit sur le serveur, dans `PROJETS/CAO/<projet>/` ;
3. dans l'éditeur, **Fichier → Enregistrer + GitHub**, ou **💾 Sauver** dans la barre
   tactile. Un message de commit est demandé (Annuler = enregistrer sans envoyer).

C'est le lanceur qui envoie (`CAO/` et `LIB_CAO/`) : WEB_CAO lui relaie la demande. Le
relai exige le même jeton que le lanceur (porté par le cookie d'appairage) ; un appareil
non appairé ne peut donc pas pousser avec les identifiants du serveur. **Enregistrer**
seul (Ctrl+S) écrit sur le serveur sans rien envoyer.

## Fonctionnement du lanceur

- Il n'écoute que sur `127.0.0.1` (sauf `--reseau`, défaut sur Pi et Termux) et refuse les
  requêtes d'action sans l'en-tête `X-WebSuite`. Il refuse aussi tout en-tête `Host` qui
  n'est ni une adresse IP, ni `localhost`, ni le nom du poste (protection contre le DNS
  rebinding).
- Les outils sont démarrés avec `--local --sans-navigateur` (sans `--local` en mode
  réseau) sur leur port habituel (CAO 8000, ANTENNA 8732, 3D 8139) ou le suivant s'il est
  pris, avec `--projets` / `--lib` vers `PROJETS/`.
- La page ne rejoue pas l'intro quand on recharge l'onglet ; le bouton **↻ Intro** la relance.
- Fermer la fenêtre de WEB_SUITE (ou Ctrl+C) arrête les outils lancés depuis elle. Sous
  Linux, fermer le terminal (SIGHUP) ou `kill` (SIGTERM) aussi.

## Tests

- `python lanceur/projets.py` : deux postes et un faux GitHub (dépôt nu) — création,
  identité git manquante, envoi, clonage sur l'autre poste, `projets.txt`.
- `python lanceur/banc_serveur.py` : `Host` étranger, `X-WebSuite`, taille des requêtes,
  jeton du mode réseau (refus, cookie, poste local exempté), code d'appairage (usage
  unique, expiration, cinq erreurs puis pause).
- `python lanceur/banc_termux.py` : faux `pkg` — `numpy` / `scipy` partent à
  `pkg install`, rien à pip, un paquet introuvable après coup est signalé.

Chacun affiche `OK` si tout passe.
