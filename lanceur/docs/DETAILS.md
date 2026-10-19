# WEB·SUITE — détails

Compléments au [README](../../README.md).

## Arborescence

```
WEB_TOOLS/                  <- ce dépôt
├── installer.cmd / .sh     <- télécharge les outils
├── demarrer_WEB_SUITE.cmd / .sh  <- lance WEB·SUITE
├── maj_git.py              <- état git de WEB_SUITE, des outils et de PROJETS
├── lanceur/                <- web_suite.py, installer.py, outils.py, projets.py, pages,
│                              mcp_web_suite.py (Claude Code)
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
- Le jeton est **partagé par tous les web tools du poste** (`~/.web_tools/jeton-reseau.txt`,
  module `lanceur/appairage.py`, copie identique dans WEB_CAO, WEB_ANTENNA, WEB_3D, web_ferme) et le
  cookie porte le même nom partout : un appareil appairé au lanceur l'est aussi pour les
  outils (un navigateur envoie le cookie à tous les ports de l'adresse), et inversement.
  L'ancien `lanceur/jeton-reseau.txt` est repris une fois. Supprimer le fichier partagé en
  crée un neuf au prochain démarrage et oblige à réappairer tous les appareils.
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
- **Dépôt privé** : WEB_SUITE_PROJETS est privé, git doit donc être connecté au compte
  GitHub pour le cloner, le tirer et l'envoyer. Sous Windows, Git Credential Manager ouvre
  sa fenêtre de connexion. Sur Termux ou un Pi, le lanceur le propose dans son terminal au
  démarrage (`gh auth login` + `gh auth setup-git`), ou à tout moment avec
  `python lanceur/connexion_github.py --connecter`. Tant que le poste n'est pas
  connecté, `PROJETS/` reste un simple dossier (pas de dépôt local sans lien avec GitHub) et
  la page l'indique ; il est cloné dès la connexion, ce qui a été créé entre-temps est gardé.
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
3. dans l'éditeur, **Fichier → ☁ Sauvegarder le projet → GitHub** (`Ctrl+Maj+S`, ou
   « Projet → GitHub » ajouté à la roulette tactile) : le document ouvert est enregistré,
   puis tout le projet part en un seul commit.

C'est le lanceur qui envoie (`CAO/` et `LIB_CAO/`) : WEB_CAO lui relaie la demande. Le
relai exige le même jeton que le lanceur (porté par le cookie d'appairage) ; un appareil
non appairé ne peut donc pas pousser avec les identifiants du serveur. **Enregistrer**
seul (Ctrl+S, ou 💾 de la roulette) et l'export du schéma vers le PCB (**⇉ PCB**) écrivent
sur le serveur sans rien envoyer : une retouche de dernière minute ne fait pas de commit.

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

## Claude Code (MCP)

`lanceur/mcp_web_suite.py` est un serveur MCP stdio (Python standard, sans dépendance) :
Claude Code le lance lui-même et lui parle sur stdin/stdout. Il appelle les outils sur
`127.0.0.1`, à leur port du catalogue (CAO 8000, ANTENNA 8732, 3D 8139, lanceur 8100),
ou au port décalé que le lanceur annonce dans `/api/etat` quand le premier était pris ;
depuis la boucle locale, aucun jeton n'est demandé. La suite est décrite dans
[SPEC-mcp.md](SPEC-mcp.md).

Outils MCP (`etat` dans `mcp_web_suite.py`, les autres dans `mcp_api_outils.py`, voir
[SPEC-mcp-api-outils.md](SPEC-mcp-api-outils.md)) :

- `etat` : qui tourne, sur quel port.
- WEB_CAO, lecture : `cao_projets`, `cao_projet_lire`, `cao_doc_lire` (schéma, PCB,
  IPC-2581 ; `parties` pour n'en lire qu'un morceau), `cao_solveurs` (état des solveurs),
  `cao_profils`, `cao_profil_lire`.
- LIB, lecture : `lib_composants` (recherche et pagination dans le catalogue),
  `lib_fichiers`, `lib_fichier_lire`, `lib_parasites`.
- Écriture gardée : `cao_doc_ecrire` (remplace seulement les clés données ; exige le
  `modifie` lu : refus si le fichier a changé depuis), `cao_profil_ecrire` et
  `lib_fichier_ecrire` (refus si le nom existe déjà : jamais d'écrasement).
- pcbparts.dev, par le relais de WEB_CAO : `pcbparts_outils`, `pcbparts_appeler` (liste
  blanche de WEB_CAO).
- WEB_ANTENNA : `antenna_projets`, `antenna_calculs`, `antenna_projet_lire` et
  `antenna_projet_enregistrer` (le projet ouvert seulement ; `null` refusé, en-tête absent
  repris du projet ouvert), `antenna_openems_etat`, `antenna_openems_chiffrer` (modèle,
  balayage, tableau S, script), `antenna_openems_suivre`, `antenna_openems_champs`.
- WEB_3D : `web3d_projets`.
- Pages ouvertes (canal de pilotage, `mcp_pilote.py`, voir
  [SPEC-mcp-canal-pilote.md](SPEC-mcp-canal-pilote.md)) : `pilote_onglets` (onglets ouverts sur
  ce poste et les appareils appairés), `pilote_capture` (image du canevas), `pilote_suivre`,
  `cao_calcul` (lance une analyse dans la page PCB ou la visionneuse et rend son rapport),
  `cao_valeur_composant` (schéma), `antenna_proposer` (cartes de l'assistant ANTENNA).
- Page WEB_3D (`mcp_registre_3d.py`, voir [SPEC-mcp-registre-3d.md](SPEC-mcp-registre-3d.md)) :
  `web3d_scene` (fichiers et pièces : uuid, chemin, nom), `web3d_vue` (vue standard, cadrage,
  capture), `web3d_visibilite`, `web3d_coupe`, `web3d_mesurer` (dimensions, distance avec cote,
  faces), `web3d_collisions` ; en carte : `web3d_ouvrir` (fichiers de PROJETS/3D, annulable),
  `web3d_deplacer` (translation, rotation, contraintes, annulable), `web3d_ranger_step` (STEP
  d'une pièce rangé dans PROJETS/3D, jamais d'écrasement, non annulable).
- Pages WEB_CAO (`mcp_registre_cao.py`, voir [SPEC-mcp-registre-cao.md](SPEC-mcp-registre-cao.md)) :
  `cao_schema_lire` (feuilles, composants, composant, nets, net : tout le document),
  `cao_pcb_lire` (empilage, classes, composants, composant, nets, net, drc), `cao_ipc_lire`
  (visionneuse), `cao_sim_lire` (simulations, une partie d'un rapport), `cao_vue` (cadrer,
  surligner, feuille, couches, ajuster, capture), `cao_drc` (lance le DRC). Lectures et vue
  seulement : rien n'est écrit dans le projet. Les lectures sont celles de l'assistant de la
  page (`WEB_CAO/commun/lecture-cao.js`). Seule exception : la visibilité des couches du PCB
  fait partie du fichier, comme l'œil du panneau ; elle part avec le prochain Enregistrer.
  Surligner un net le met en avant sans sélectionner son routage.
- Écritures WEB_CAO (`mcp_ecriture_cao.py`, voir [SPEC-mcp-ecriture-cao.md](SPEC-mcp-ecriture-cao.md)) :
  `cao_composants_ecrire` (schéma ou PCB : repère, valeur, boîtier, fabricant, MPN, specs,
  non monté ; face, position, rotation au PCB), `cao_pistes_ecrire` (largeur, réalignement
  sur la classe), `cao_classes_ecrire` (créer, modifier, supprimer, rattacher des nets),
  `cao_contraintes_ecrire` (règles de paires, contraintes de net), `cao_sim_ecrire`
  (paramètres de simulation, unité du catalogue). Un lot de 50 éléments au plus = une carte
  = une étape Ctrl+Z (la simulation : Annuler sur la carte). Un seul refus et rien ne
  change ; au clic, la page revérifie que rien n'a bougé sous la carte. Au PCB, la réponse
  donne le DRC avant → après. Code de page : `WEB_CAO/commun/ecriture-cao.js`.

**Le panneau « Claude ».** Chaque page des outils a, en bas à droite, un bouton « Claude ».
Une écriture proposée par Claude s'y affiche en carte, avant → après, avec Appliquer et
Refuser : rien ne change avant le clic. La case « Appliquer directement (cet onglet) »
saute la carte ; elle est éteinte à chaque chargement de la page. Une écriture appliquée
se défait par Ctrl+Z dans l'éditeur qui a un historique (schéma), sinon par le bouton
Annuler de la carte, qui refuse si quelque chose a changé depuis. Un seul onglet par outil
et par navigateur tient la connexion au serveur et relaie les autres.

Au-delà de 100 Ko, une réponse devient un résumé (clés, tailles, extrait). Le MCP ne
supprime rien, ne lance ni n'arrête aucun calcul (les calculs WEB_CAO passeront par la
page, avec le module `canal-pilote`), n'ouvre ni ne ferme de projet ANTENNA, ne touche
ni aux racines de projets ni au catalogue LIB entier.

**Sur le poste des outils** (Linux, Pi) :

```
claude mcp add -s user web_suite -- python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py
```

Windows, outils sur le même PC : `claude mcp add -s user web_suite -- py -3 C:\chemin\WEB_TOOLS\lanceur\mcp_web_suite.py`.

**Depuis Windows vers le Pi, par SSH** : le MCP tourne sur le Pi, SSH ne fait que
transporter stdin/stdout. Aucun port à ouvrir, et le trafic est chiffré.

1. Client OpenSSH présent ? `ssh -V` dans PowerShell. Sinon : Paramètres → Applications →
   Fonctionnalités facultatives → **Client OpenSSH**.
2. Clé SSH (une fois) : `ssh-keygen -t ed25519`, puis copier la clé publique sur le Pi :
   `type $env:USERPROFILE\.ssh\id_ed25519.pub | ssh pilou@<pi> "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"`.
3. Vérifier : `ssh -T -o BatchMode=yes pilou@<pi> echo ok` doit afficher `ok` **sans
   demander de mot de passe**. Le MCP passe par stdin : une invite de mot de passe le
   bloquerait, `BatchMode=yes` le fait échouer net à la place.
4. Ajouter :
   `claude mcp add -s user web_suite -- ssh -T -o BatchMode=yes pilou@<pi> python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py`

**Dépannage** : `/mcp` affiche l'état de la connexion. Si elle échoue par SSH, un
`.bashrc` du Pi qui écrit quelque chose (un `echo`) pollue stdout : le déplacer dans
`.profile` ou le limiter aux shells interactifs. Essai à la main :
`printf '{"jsonrpc":"2.0","id":1,"method":"ping"}\n' | python3 lanceur/mcp_web_suite.py`
doit rendre `{"jsonrpc": "2.0", "id": 1, "result": {}}`.

## Tests

- `python lanceur/projets.py` : deux postes et un faux GitHub (dépôt nu) — création,
  identité git manquante, envoi, clonage sur l'autre poste, `projets.txt`.
- `python lanceur/banc_serveur.py` : `Host` étranger, `X-WebSuite`, taille des requêtes,
  jeton du mode réseau (refus, cookie, poste local exempté), code d'appairage (usage
  unique, expiration, cinq erreurs puis pause).
- `python lanceur/banc_termux.py` : faux `pkg` — `numpy` / `scipy` partent à
  `pkg install`, rien à pip, un paquet introuvable après coup est signalé.
- `python lanceur/banc_mcp.py` : serveur MCP en sous-processus — protocole (initialize,
  ping, erreurs), outil `etat` face à un faux WEB_CAO, routes `/api/github`, `/api/envoyer`,
  `/api/identite`, `/api/ia`, `/api/appairer` refusées et lanceur en lecture seule, sans
  qu'aucune requête n'atteigne l'outil, outil arrêté signalé ; outils d'`api-outils` face
  à un faux outil qui note chaque requête (bonne route et bonne méthode, garde-fous qui
  refusent sans écrire, résumé sous 100 Ko, jamais `…/lancer`) ; outils du canal de pilotage
  face à un faux outil (commande déposée, en attente, refusée, capture en image).
- `python lanceur/pilote.py` : le serveur du canal de pilotage (file, long-poll, ciblage par
  dernier focus, redistribution, ordres réservés à ce poste, autre origine refusée).
- `node lanceur/banc_pilote.js` : la page (un seul relais, changement de relais sans perte,
  cartes, interrupteur, Annuler si rien n'a bougé, aucun texte de Claude lu comme du HTML).

Chacun affiche `OK` si tout passe.
