# Essai : le canal de pilotage depuis l'iPad appairé

Où : iPad (Safari) pour les pages, Pi pour les outils et Claude Code (terminal du Pi ou SSH
depuis le PC) · Outils : WEB_CAO, WEB_ANTENNA, WEB_3D lancés sur une copie de PROJETS,
Claude Code avec le MCP · Durée : 45 min · Source : `A-FAIRE.md:15`, `tasks/todo.md:247`, `tasks/plan.md:104`

## Pourquoi

Le canal de pilotage laisse Claude agir dans les pages ouvertes, y compris celles d'un
appareil appairé (`lanceur/docs/SPEC-mcp-canal-pilote.md`, décision 4). L'essai prouve que
sur l'iPad, en `http://` non sécurisé, tout marche :
- l'onglet se branche et apparaît avec l'adresse de l'iPad ;
- une écriture arrive en carte et n'est faite qu'au clic ;
- l'interrupteur « Appliquer directement (cet onglet) » saute la carte, et il est éteint
  au rechargement ;
- la capture rend une image ;
- et cela dans les trois outils.

Déjà fait ailleurs : le 18/10/2026 sur le Pi, Chromium piloté par CDP, cinq onglets, trois
outils, essai de durée de 7 min (`tasks/todo.md:230-241`). Le contexte non sécurisé a été
vérifié dans Chromium 155 sur `http://192.168.1.31` (`SPEC-mcp-canal-pilote.md:297-301`).
D'où l'élection du relais par BroadcastChannel, sans `navigator.locks` ni
`crypto.randomUUID`. **Jamais essayé : Safari sur un vrai iPad.**

## Avant de commencer

1. Sur le Pi, mets à jour les quatre dépôts :
   ```
   cd ~/Documents/web_suite
   git pull --ff-only
   for d in WEB_CAO WEB_ANTENNA WEB_3D; do git -C $d pull --ff-only; done
   ```
2. Dans WEB·SUITE, arrête les trois outils s'ils tournent (**■ Arrêter** sur chaque carte) :
   ils vont être relancés à la main sur la copie, à leurs ports du catalogue.
3. Copie PROJETS (l'essai écrit dans les pages), et note son empreinte :
   ```
   rm -rf /tmp/essai-PROJETS && cp -a ~/Documents/web_suite/PROJETS /tmp/essai-PROJETS
   find ~/Documents/web_suite/PROJETS -type f -not -path '*/.git/*' -exec md5sum {} + | sort -k2 | md5sum
   ```
4. Lance les trois outils sur la copie, **en mode réseau** (sans `--local`). Le `--port` est
   obligatoire : sans lui, WEB_ANTENNA prendrait le 8000 de WEB_CAO (`web_antenna.py:66`).
   ```
   E=/tmp/essai-PROJETS; W=~/Documents/web_suite
   cd $W/WEB_CAO && nohup python3 web_CAO.py --port 8000 --sans-navigateur --sans-pause --sans-maj --projets $E/CAO --lib $E/LIB_CAO --projets-reseau > /tmp/essai-cao.log 2>&1 &
   cd $W/WEB_ANTENNA && nohup python3 web_antenna.py --port 8732 --sans-navigateur --sans-maj --projets $E/ANTENNA > /tmp/essai-antenna.log 2>&1 &
   cd $W/WEB_3D && nohup python3 web_3D.py --port 8139 --sans-navigateur --sans-pause --sans-maj --projets $E/3D > /tmp/essai-3d.log 2>&1 &
   ```
   → attendu : dans chaque journal (`/tmp/essai-*.log`), l'adresse réseau et une ligne
   `Code d'appairage … : 123 456`.
5. Sur le Pi, ferme tout onglet de ces outils (Chromium du Pi, PC). Sinon, des onglets du
   même type sur deux appareils font refuser les commandes tant que Claude ne nomme pas
   l'onglet (décision 2).
6. Sur l'iPad : même Wi-Fi que le Pi ; Verrouillage auto sur **Jamais** le temps de l'essai.
   Safari ralentit vite un onglet caché ou un écran éteint, et l'onglet passe pour mort
   après 90 s de silence.
7. Claude Code sur le Pi, dans `~/Documents/web_suite`, avec le MCP (`ESSAI-pi-mcp-claude-code.md`).

## Étapes

1. Sur l'iPad, ouvre `http://<adresse du Pi>:8000/`. S'il demande un code, saisis celui de
   `/tmp/essai-cao.log`.
   → attendu : l'accueil WEB_CAO. Un iPad déjà appairé au lanceur n'a pas de code à saisir :
   le jeton est partagé par les web tools du poste (`DETAILS.md:41-61`).
2. Ouvre l'éditeur schématique, **Fichier → Exemples…**, « Interface IoT & Bus Haute Vitesse ». En bas à droite, le
   bouton **Claude**. Ouvre le panneau : la case « Appliquer directement (cet onglet) » est
   **décochée**.
3. À Claude : « Liste les onglets ouverts. »
   → attendu : `pilote_onglets` donne un onglet `schema` de WEB_CAO, avec pour appareil
   **l'adresse IP de l'iPad** (pas « ce poste »), et l'interrupteur éteint.
4. « Capture le schéma de l'onglet ouvert. »
   → attendu : `pilote_capture` rend une image du schéma, 1600 px de large au plus.
5. « Change la valeur d'une résistance du schéma en 4k7 (cao_valeur_composant). »
   → attendu : sur l'iPad, le panneau s'ouvre seul ; une carte titrée du repère (R…), avec
   « <ancienne valeur> → 4k7 », **Appliquer** et **Refuser**. Le schéma ne change pas avant le clic.
6. Touche **Refuser**.
   → attendu : Claude reçoit « refusée », le schéma est inchangé.
7. Redemande la même chose et touche **Appliquer**.
   → attendu : la valeur devient 4k7 ; la carte dit « Pour défaire : Ctrl+Z dans l'éditeur. ».
   **Édition → Annuler** (ou « Annuler » de la roulette tactile) rend l'ancienne valeur, en
   une seule étape.
8. Coche « Appliquer directement (cet onglet) », puis : « Mets 10k sur la même résistance. »
   → attendu : appliqué sans clic ; la carte dit « Appliquée directement (interrupteur
   allumé). » ; `pilote_onglets` montre l'interrupteur allumé.
9. Recharge la page.
   → attendu : la case est de nouveau **décochée**.
10. Une carte laissée sans réponse : demande une valeur, n'y touche pas pendant 2 min.
    → attendu : Claude reçoit « en_attente » avec un id ; touche alors Appliquer, puis
    « Suis cette proposition » → `pilote_suivre` rend « appliquee ».
11. WEB_ANTENNA : sur l'iPad, ouvre `http://<adresse du Pi>:8732/`, puis **🧪 Patch**.
    « Capture la carte d'antenne », puis « Propose une cote du patch 0,5 mm plus longue (antenna_proposer, carte cotes). »
    → attendu : une image de la carte 2D ; une carte avant → après. Appliquer change la
    cote ; le bouton **Annuler** de la carte la remet. Aucun calcul openEMS ne se lance.
12. WEB_3D : sur l'iPad, ouvre `http://<adresse du Pi>:8139/?modele=exemples/assemblage-as1.stp`.
    « Capture la vue 3D », puis « Décale le fichier ouvert de 10 mm en X (web3d_deplacer). »
    → attendu : une image de la scène ; une carte. Appliquer déplace la pièce, **Annuler** la
    remet.
13. Ferme les onglets de l'iPad, puis : « Liste les onglets. »
    → attendu : plus aucun onglet de l'iPad, en moins de 2 min.
14. Arrête les outils de l'essai et vérifie PROJETS :
    ```
    pkill -f "projets /tmp/essai-PROJETS"
    find ~/Documents/web_suite/PROJETS -type f -not -path '*/.git/*' -exec md5sum {} + | sort -k2 | md5sum
    ```
    → attendu : la même empreinte qu'au point 3 d'« Avant de commencer ».

## C'est réussi si

- [ ] Les onglets de l'iPad apparaissent dans `pilote_onglets` avec l'adresse de l'iPad, dans les trois outils
- [ ] Une carte n'écrit rien avant le clic ; Refuser et Appliquer rendent le bon état à Claude
- [ ] Ctrl+Z (Édition → Annuler) défait l'écriture du schéma en une étape ; l'Annuler des cartes ANTENNA et 3D remet l'état d'avant
- [ ] L'interrupteur allumé saute la carte, et il est éteint au rechargement
- [ ] `pilote_capture` rend une image du schéma, de la carte ANTENNA et de la vue 3D
- [ ] Au-delà de 2 min : « en_attente », puis `pilote_suivre` juste
- [ ] PROJETS a la même empreinte avant et après

## Si ça rate, note

- La version d'iPadOS (Réglages → Général → Informations).
- La réponse exacte de Claude (le texte du résultat d'outil) et une capture d'écran de l'iPad
  au même moment (panneau Claude ouvert).
- Les journaux des outils : `/tmp/essai-cao.log`, `/tmp/essai-antenna.log`, `/tmp/essai-3d.log`.
- La console de Safari, si le panneau Claude n'apparaît pas ou que l'onglet ne se branche pas.
  Elle demande un Mac : Réglages de l'iPad → Safari → Avancé → Inspecteur web, puis sur le
  Mac, Safari → Développement → <l'iPad>. Sans Mac, note juste ce qui manque.
- Pour une commande refusée « plusieurs appareils » : la liste de `pilote_onglets`.

## Après

En cas de réussite :
- `A-FAIRE.md:15` : cocher **iPad**, avec la date
- `tasks/todo.md:247` : cocher « Essai sur l'iPad avec toi » ; `:246` : retirer « (sauf l'essai iPad) »
- `tasks/plan.md:104` : cocher ; `:103` : retirer « (sauf l'essai iPad) »
- `WEB_CAO/A-FAIRE.md:141` (« Essais réels restants ») et `:48` : retirer « le canal depuis l'iPad »
- `tasks/plan-registre-3d.md:5` : retirer « essai iPad encore ouvert »
