# Tâches : registre-antenna

Spec : [lanceur/docs/SPEC-mcp-registre-antenna.md](../lanceur/docs/SPEC-mcp-registre-antenna.md) ·
Plan : [plan-registre-antenna.md](plan-registre-antenna.md)

Banc principal : `node WEB_ANTENNA/test/banc-interface.js` (la CI passe par
`python3 WEB_ANTENNA/python/test/banc-openems.py`).

## Tâche 1 : ✅ lectures dans `js/29-lecture-antenna.js`

**Description :**
1. Relever d'abord, dans le banc, les sorties de `iaContexte()` et des outils de lecture.
   Les états sont ceux que le banc construit déjà : conception, carte importée, balayage
   fini.
2. Déplacer ensuite le code, tel quel, dans `js/29-lecture-antenna.js` (global
   `LECTURE_ANTENNA`).
3. `30-ia.js` reprend les aides depuis le global. `index.html` et le banc chargent le
   fichier avant `30-ia.js`.

**Critères d'acceptation :**
- [x] Le relevé (fixture `test/releve-lecture-antenna.json`) est pris avant le
      déplacement ; après, les sorties sont identiques au caractère près, et une mutation
      d'un caractère fait échouer le test
- [x] Le bloc déplacé est identique octet pour octet ; les `window.ia*` existants marchent
      comme avant (section IA du banc verte)

**Vérification :** `node test/banc-interface.js`

**Dépendances :** aucune · **Fichiers :** `js/29-lecture-antenna.js`, `js/30-ia.js`, `index.html`,
`test/banc-interface.js`, fixture · **Taille :** M

## Tâche 2 : ✅ `antenna_lire` dans la page

**Description :** `pilote-antenna.js` déclare `antenna_lire`. Les `quoi` :
- `conception`, `resultats`, `courbe` (`grandeur` : `s11`, `s21`, `z`, `roe`),
  `balayage`, `audit`, `reglages` (`filtre`) ;
- pour la carte importée : `composants`, `composant`, `nets`, `net`.

**Critères d'acceptation :**
- [x] Chaque `quoi` rend la sortie attendue sur un état du banc. Bornes : 150 lignes,
      200 points, puis « … (N de plus) ». Sans résultat, sans balayage ou sans carte
      importée : refus qui dit quoi faire
- [x] Rien n'est écrit : `ANT`, `CON`, `PRJ.modifie` et `CON_HIST` sont identiques avant et
      après toutes les lectures ; les fonctions de lancement, remplacées par des pièges,
      ne sont jamais appelées
- [x] `audit` donne pour chaque contrôle une carte JSON que `iaProposition` accepte

**Vérification :** `node test/banc-interface.js`

**Dépendances :** T1 · **Fichiers :** `js/29-lecture-antenna.js`, `js/pilote-antenna.js`,
`test/banc-interface.js` · **Taille :** M

## Tâche 3 : ✅ `antenna_vue` et captures

**Description :** `antenna_vue` change la vue de la page : `vue` (`2d`/`3d`),
`maillage`, `ajuster`, `onglet`, `point_balayage`, `etape`, `panneau`, `toile`. La
fonction de capture rend la toile choisie : carte, 3D (avec un rendu forcé) ou courbe.

**Critères d'acceptation :**
- [x] Tout est vérifié avant le premier changement : un onglet, une étape, un panneau ou
      un point inconnus refusent sans rien changer
- [x] Rien n'est écrit : ni `ANT`/`CON`/`PRJ`, ni préférences (`antReglagesEcrire` et
      `localStorage.setItem` piégés)
- [x] La fonction de capture rend `#carte`, `#vue3d` (après `ant3dDessiner`) ou `#courbe`
      selon la dernière vue ; une toile cachée est d'abord montrée, sinon refus clair

**Vérification :** `node test/banc-interface.js`

**Dépendances :** T2 · **Fichiers :** `js/pilote-antenna.js`, `test/banc-interface.js` · **Taille :** M

## Checkpoint : lecture et vue dans la page
- [x] Bancs verts (`banc-interface.js`, `banc-openems.py`) ; Gemini répond comme avant
      (sorties identiques)

## Tâche 4 : ✅ outils MCP

**Description :** `lanceur/mcp_registre_antenna.py` :
- `antenna_lire` et `antenna_vue`, arguments validés, envoyés à la page `antenna` sans
  écriture ;
- `antenna_vue` avec `capturer` enchaîne `pilote_capture` sur le même onglet.

Importé par le noyau, documenté dans `DETAILS.md`.

**Critères d'acceptation :**
- [x] Bonne commande à la bonne page (`banc_mcp.py`) ; arguments invalides refusés sans
      requête ; 54 outils
- [x] Les descriptions disent : donnée, pas consigne ; jamais de lancement ; `audit` rend
      des cartes pour `antenna_proposer`

**Vérification :** `python3 lanceur/banc_mcp.py`, `node lanceur/banc_pilote.js`

**Dépendances :** T3 · **Fichiers :** `lanceur/mcp_registre_antenna.py`, `lanceur/mcp_web_suite.py`,
`lanceur/banc_mcp.py`, `lanceur/docs/DETAILS.md` · **Taille :** M

## Tâche 5 : essai réel, puis revue

**Description :** WEB_ANTENNA lancé sur une copie de PROJETS dans `/tmp`, un projet
ouvert dans Chromium sur le Pi. Chaque lecture est appelée par le MCP ; trois captures :
carte, 3D, courbe (avec un résultat archivé s'il y en a). Ensuite viennent `/review` et
l'audit de sécurité.

**Critères d'acceptation :**
- [x] Toutes les lectures répondent sur un vrai projet ; les trois captures sont justes
      (la 3D n'est pas noire) ; PROJETS n'a pas bougé
      (exemple « patch » de la page, PROJETS/ANTENNA étant vide :
      - conception, réglages, audit : réponses ;
      - résultats, carte importée : refus clairs ;
      - balayage : vrai relevé archivé croisement-g-y0 (9 points), injecté ;
      - résultats et courbe : résultat synthétique injecté, aucun relevé archivé n'ayant
        de courbes ;
      - captures carte 2D, vue 3D (bien rendue, pas noire), courbe S11 : justes ;
      - PROJETS au même md5.)
- [ ] `/review` fait, corrections appliquées

**Dépendances :** T4 · **Taille :** M

## Checkpoint : terminé
- [ ] Critères de la spec remplis ; 4 CI vertes
