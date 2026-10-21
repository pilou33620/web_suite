# Essai : le canal de pilotage depuis l'iPad appairé

Qui : toi · Où : iPad (Safari) pour les pages, Pi pour les outils et Claude Code, à la maison ·
Durée : 45 min · Source : `A-FAIRE.md:18`, `tasks/todo.md:247`, `tasks/plan.md:104`

**Pas pour le PC du boulot** : il faut l'iPad, et les commandes viennent de Claude Code par le
MCP. À faire après `ESSAI-pi-mcp-claude-code.md`.

## Pourquoi

Le canal de pilotage laisse Claude agir dans les pages ouvertes, y compris sur un appareil
appairé (`lanceur/docs/SPEC-mcp-canal-pilote.md`, décision 4). L'essai prouve que sur l'iPad,
en `http://` non sécurisé, dans les trois outils : l'onglet se branche avec l'adresse de
l'iPad ; une écriture arrive en carte et n'est faite qu'au clic ; l'interrupteur « Appliquer
directement (cet onglet) » saute la carte ; la capture rend une image.

Déjà fait le 18/10/2026 : Chromium du Pi piloté par Playwright, cinq onglets, trois outils, 7 min
(`tasks/todo.md:230-241`) ; contexte non sécurisé vérifié dans Chromium 155
(`SPEC-mcp-canal-pilote.md:297-301`), d'où le relais élu par BroadcastChannel, sans
`navigator.locks` ni `crypto.randomUUID`. **Jamais essayé : Safari sur un vrai iPad.**

## Ce que tu prépares

1. Sur le Pi, mets à jour (ou démarre WEB·SUITE, qui le fait) :
   `cd ~/Documents/web_suite && git pull --ff-only && for d in WEB_CAO WEB_ANTENNA WEB_3D; do git -C $d pull --ff-only; done`
2. Dans WEB·SUITE, **■ Arrêter** les trois outils : ils sont relancés à la main sur une copie.
3. Copie PROJETS (l'essai écrit dans les pages) et note son empreinte :
   ```
   rm -rf /tmp/essai-PROJETS && cp -a ~/Documents/web_suite/PROJETS /tmp/essai-PROJETS
   find ~/Documents/web_suite/PROJETS -type f -not -path '*/.git/*' -exec md5sum {} + | sort -k2 | md5sum
   ```
4. Lance les trois outils sur la copie, **en mode réseau** (le `--port` est obligatoire : sans
   lui, WEB_ANTENNA prendrait le 8000, `web_antenna.py:66`) :
   ```
   E=/tmp/essai-PROJETS; W=~/Documents/web_suite
   cd $W/WEB_CAO && nohup python3 web_CAO.py --port 8000 --sans-navigateur --sans-pause --sans-maj --projets $E/CAO --lib $E/LIB_CAO --projets-reseau > /tmp/essai-cao.log 2>&1 &
   cd $W/WEB_ANTENNA && nohup python3 web_antenna.py --port 8732 --sans-navigateur --sans-maj --projets $E/ANTENNA > /tmp/essai-antenna.log 2>&1 &
   cd $W/WEB_3D && nohup python3 web_3D.py --port 8139 --sans-navigateur --sans-pause --sans-maj --projets $E/3D > /tmp/essai-3d.log 2>&1 &
   ```
   → attendu : dans chaque `/tmp/essai-*.log`, l'adresse réseau et `Code d'appairage … : 123 456`.
5. Ferme tout autre onglet de ces outils (Chromium du Pi, PC) : deux appareils avec le même type
   d'onglet font refuser les commandes tant que Claude ne nomme pas l'onglet (décision 2).
6. iPad : même Wi-Fi que le Pi ; Verrouillage auto sur **Jamais** (un onglet caché ou un écran
   éteint passe pour mort après 90 s).
7. Claude Code sur le Pi, dans `~/Documents/web_suite`, avec le MCP.

Claude in Chrome ne peut pas : les commandes viennent de Claude Code par le MCP, et la question
est Safari sur l'iPad. Dans Chrome sur le PC, sans MCP, il ne verrait que le panneau vide et
une case décochée en dur à chaque chargement (`lanceur/pilote.js:349`) : rien que le banc et
l'essai Chromium n'aient déjà montré.

## C'est réussi si

- [ ] Les onglets de l'iPad apparaissent dans `pilote_onglets` avec l'adresse de l'iPad, dans les trois outils
- [ ] Une carte n'écrit rien avant le clic ; Refuser et Appliquer rendent le bon état à Claude
- [ ] Édition → Annuler défait l'écriture du schéma en une étape ; l'Annuler des cartes ANTENNA et 3D remet l'état d'avant
- [ ] L'interrupteur allumé saute la carte, et il est éteint au rechargement
- [ ] `pilote_capture` rend une image du schéma, de la carte ANTENNA et de la vue 3D
- [ ] Au-delà de 2 min : « en_attente », puis `pilote_suivre` juste
- [ ] PROJETS a la même empreinte avant et après

## Ce qui reste à toi

1. iPad : `http://<adresse du Pi>:8000/` (code de `/tmp/essai-cao.log` s'il est demandé ; un
   iPad déjà appairé au lanceur n'en a pas : `DETAILS.md:41-61`) → l'accueil WEB_CAO.
2. Éditeur schématique, **Fichier → Exemples…**, « Interface IoT & Bus Haute Vitesse ». Ouvre le
   panneau **Claude** (en bas à droite) : la case « Appliquer directement (cet onglet) » est **décochée**.
3. À Claude : « Liste les onglets ouverts. » → `pilote_onglets` : un onglet `schema`, appareil
   **l'adresse IP de l'iPad** (pas « ce poste »), interrupteur éteint.
4. « Capture le schéma de l'onglet ouvert. » → `pilote_capture` : une image, 1600 px de large au plus.
5. « Change la valeur d'une résistance du schéma en 4k7 (cao_valeur_composant). » → le panneau
   s'ouvre seul ; une carte titrée du repère, « <ancienne valeur> → 4k7 », **Appliquer** /
   **Refuser**. Le schéma ne change pas avant le clic.
6. **Refuser** → Claude reçoit « refusée », schéma inchangé.
7. Même demande, **Appliquer** → 4k7 ; la carte dit « Pour défaire : Ctrl+Z dans l'éditeur. » ;
   **Édition → Annuler** (ou la roulette tactile) rend l'ancienne valeur en une étape.
8. Coche la case, puis « Mets 10k sur la même résistance. » → appliqué sans clic, « Appliquée
   directement (interrupteur allumé). » ; `pilote_onglets` montre l'interrupteur allumé.
9. Recharge la page → la case est **décochée**.
10. Une demande de valeur laissée 2 min sans réponse → « en_attente » avec un id ; touche
    Appliquer, puis « Suis cette proposition » → `pilote_suivre` rend « appliquee ».
11. WEB_ANTENNA : `http://<adresse du Pi>:8732/`, **🧪 Patch**. « Capture la carte d'antenne »,
    puis « Propose une cote du patch 0,5 mm plus longue (antenna_proposer, carte cotes). » →
    une image, une carte avant → après ; Appliquer change la cote, **Annuler** la remet ; aucun
    calcul openEMS.
12. WEB_3D : `http://<adresse du Pi>:8139/?modele=exemples/assemblage-as1.stp`. « Capture la vue
    3D », puis « Décale le fichier ouvert de 10 mm en X (web3d_deplacer). » → une image, une
    carte ; Appliquer déplace, **Annuler** remet.
13. Ferme les onglets de l'iPad, puis « Liste les onglets. » → plus d'onglet de l'iPad en moins de 2 min.
14. `pkill -f "projets /tmp/essai-PROJETS"`, puis l'empreinte de PROJETS (point 3) → la même.

Si ça rate, note : la version d'iPadOS ; le texte du résultat d'outil et une capture de l'iPad
(panneau Claude ouvert) ; `/tmp/essai-cao.log`, `/tmp/essai-antenna.log`, `/tmp/essai-3d.log` ;
la console de Safari (iPad : Réglages → Safari → Avancé → Inspecteur web, puis Safari du Mac →
Développement ; sans Mac, ce qui manque) ; pour « plusieurs appareils », la liste de `pilote_onglets`.

## Après

Pour Claude Code, en cas de réussite :
- [ ] `A-FAIRE.md:18` : cocher **iPad**, avec la date
- [ ] `tasks/todo.md:247` : cocher ; `:246` : retirer « (sauf l'essai iPad) »
- [ ] `tasks/plan.md:104` : cocher ; `:103` : retirer « (sauf l'essai iPad) »
- [ ] `WEB_CAO/A-FAIRE.md:141` et `:48` : retirer « le canal depuis l'iPad »
- [ ] `tasks/plan-registre-3d.md:5` : retirer « essai iPad encore ouvert »
