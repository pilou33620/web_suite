# Essai : MIFA, croisement du développé visé `kd` × nombre de replis `n`

Où : poste avec openEMS (WEB_ANTENNA lancé par WEB·SUITE) · Durée : 12 points × 5 à 13 min (repère : le patch FR-4, 420 000 à 490 000 cellules par point) ; les points à `n` = 3 seront plus longs (brins à 0,16 mm), « ∑ Chiffrer la plage » fait foi — compte 1 h 30 à 3 h · Source : `web_antenna/A-FAIRE.md:321` (plan, point 1 ; constat l.304-311 ; limite README l.2038-2045)

## Pourquoi
Le MIFA résonne **au-dessus de 3,5 GHz pour 2,45 visés** (FR-4 1,53 mm, `A-FAIRE.md:297`). Son tracé pose un développé égal au quart d'onde, mais deux brins repliés trop proches se couplent et raccourcissent l'antenne électrique. Allonger l'empreinte efface le méandre (2,415 GHz à 17 mm, mais c'est un IFA droit). La bonne mesure : à empreinte fixe, allonger le **développé** (`kd`, en % du quart d'onde, 100 par défaut, `js/22-antennes.js:872` et `:902`), et voir combien il en faut selon l'écartement des brins.

Aux cotes par défaut (FR-4 1,53 mm, 2,45 GHz), d'après le tracé :

| `n` | pas du méandre | écartement des brins | écartement / largeur de bras (0,940 mm) | hauteur des dents à `kd` = 100 / 160 |
|---|---|---|---|---|
| 1 | 6,577 mm | 2,349 mm | 2,50 | 4,016 / 9,654 mm |
| 2 *(défaut)* | 3,289 mm | 0,705 mm | 0,75 | 2,008 / 4,827 mm |
| 3 | 2,192 mm | 0,157 mm | 0,17 | 1,339 / 3,218 mm |

`n` = 4 est exclu : écartement −0,117 mm (vérifié).

Le résultat décide du **défaut de `kd`** : une loi `kd(écartement/largeur)` à la place de 100.

## Avant de commencer
- WEB_ANTENNA au moins au commit `0decae2` (la cote `kd`) ; spec écrite sur `c95f9b6` du 21/10/2026. Dans WEB_ANTENNA, `python python/test/banc-openems.py` → **1053 vérifications, toutes passées**.
- Un projet à lui, pour que les calculs aillent dans `PROJETS/ANTENNA/essai-mifa-kd-n/calculs/` et pas dans le TEMP : panneau **« 📁 Projet »**, nom `essai-mifa-kd-n`, **Ctrl+S** avant de lancer.
- Document de départ : **✏️ Concevoir** → empilage **« 2 couches · FR-4 1,6 mm · 35 µm »** (âme 1,530 mm, εr 4,30, tan δ 0,020 : ne la retouche pas, le plan dit FR-4 1,53 mm) → fréquence visée **2,45 GHz** → motif **« F inversé à méandres (MIFA) »**, toutes cotes par défaut (`Lx` 6,5772, `n` 2, `wb` 0,9396, `kd` 100…) → **Dessiner ce motif**.
- **La bande, et c'est le seul écart aux relevés précédents** : le motif pose ±15 % (2,0825 à 2,8175 GHz), or le MIFA résonne au-dessus de 3,5 GHz et `kd` = 160 le fera descendre vers 1,5–2 GHz. Étape « La bande » : **1,5 à 4,5 GHz, 601 points** (pas de 5 MHz), cible 2,45 GHz. Sans cela, la moitié des points rendrait un minimum collé au bord de bande.
- Le reste aux valeurs par défaut, comme les croisements du patch : boîte (marges 0 = calculées, PML 8), maillage (pas air et diélectrique à 0 = calculés, règle des tiers cochée), arrêt à **−40 dB**, pas de temps maximum **0** (calculé), diagramme de rayonnement coché (le motif le coche), port localisé 50 Ω.
- Le pas de maillage du balayage se fige sur le document **tel qu'il est au lancement** (`python/openems_modele.py:5563`) : lance depuis les cotes par défaut, pas depuis un point retouché.

## Étapes
1. Étape « Le calcul » → coche **Balayer une cote** → Cote : **« F inversé à méandres (MIFA) — développé visé »** (%) → De **100** à **160** par pas de **20**.
   → attendu : 4 valeurs.
2. Coche **Croiser avec une seconde cote** → Seconde cote : **« F inversé à méandres (MIFA) — nombre de replis »** → De **1** à **3** par pas de **1**.
   → attendu : « 4 × 3 = 12 points, 40 au plus ».
3. **∑ Chiffrer la plage**.
   → attendu : aucun refus (tous les points sont vérifiés avant le premier). Note le nombre de cellules du pire point, la durée annoncée et la plus petite cellule (elle sera à `n` = 3, dans l'écart de 0,157 mm entre brins — c'est attendu).
4. **▶ Lancer le balayage**.
   → attendu : 12 lignes dans le journal, une par point : « → … = 100 × 1 % : résonance x,xxxx GHz, S11 −xx,xx dB ». Le sens se voit dès les premiers points : à `n` fixé, la résonance doit **descendre** quand `kd` monte.
5. Une fois fini, regarde le tableau à deux entrées : pour chaque `n`, encadre-t-on 2,45 GHz entre deux valeurs de `kd` ?
   → si à `n` = 3 même `kd` = 160 reste au-dessus de 2,45 GHz : relance une ligne de plus, `kd` 180 à 220 par 20 à `n` = 3 seul (balayage simple, 3 points), même projet, et garde les deux relevés.
6. **Ctrl+S** (enregistre le projet, résultats compris).
7. Enregistrer le relevé sous `croisement-mifa-kd-n.json` à la racine de WEB_ANTENNA (format des précédents) — depuis le dossier de WEB·SUITE :
   ```
   python tasks/essais/ESSAI-antenna-releve.py WEB_ANTENNA/croisement-mifa-kd-n.json "MIFA kd x n, FR-4 1,53 mm, 2,45 GHz, bande 1,5-4,5 GHz 601 pts, reglages par defaut, arret -40 dB" "kd x n=PROJETS/ANTENNA/essai-mifa-kd-n/resultats.json"
   ```
   → attendu : « … : 1 cas, 12 points ». Si tu as fait le prolongement de l'étape 5, renomme d'abord le projet `essai-mifa-kd-n3` dans « 📁 Projet » avant de le lancer (le nom changé fait une copie, le premier `resultats.json` reste intact), puis ajoute `"kd n=3 prolonge=PROJETS/ANTENNA/essai-mifa-kd-n3/resultats.json"` à la même commande.

## C'est réussi si
- [ ] Les 12 points sont calculés, aucun « aucune grandeur exploitable » dans le journal.
- [ ] Chaque point a convergé à −40 dB (pas de « coupure prématurée » dans le diagnostic).
- [ ] Aucune résonance collée au bord de bande (colonne bande sans « bord »).
- [ ] Pour au moins `n` = 1 et `n` = 2, 2,45 GHz est encadré par deux `kd`.
- [ ] `WEB_ANTENNA/croisement-mifa-kd-n.json` existe, 12 points.

## Ce qu'on en tirera
- Pour chaque `n`, une interpolation de `kd` où la résonance vaut 2,45 GHz : trois couples (écartement/largeur ; `kd`) = (2,50 ; ?), (0,75 ; ?), (0,17 ; ?).
- **Si `kd` croît quand l'écartement se resserre** (ce qu'on attend : plus de couplage, plus de développé à rattraper), la loi qui passe par ces trois points devient le défaut de `kd`, calculé depuis l'écartement — le défaut (`n` = 2) prend la valeur de 0,75.
- **Si les trois `kd` sont à peu près égaux**, ce n'est pas le couplage entre dents qui manque mais un facteur global (comme le bras × 1,018 de l'IFA) : `kd` devient une constante.
- Le rendement dira si un méandre serré (`n` = 3) vaut encore la peine : s'il tombe nettement sous celui de `n` = 1, la fiche du motif le dira.
- On ne lit pas la résonance d'un point dont le S₁₁ minimal reste au-dessus de −3 dB : ce n'est pas une résonance, je regarderai sa courbe.

## Si ça rate, note
- Le message de refus de « Chiffrer » (il dit quel point), ou la ligne du journal du point en cause.
- Le journal du solveur : panneau **« Journal du solveur »** de la page (il n'est pas écrit sur disque) — copie les lignes du point raté, diagnostic d'énergie compris (« divergence », « coupure prématurée », « énergie éteinte sans résultat »).
- Le dossier du point : `PROJETS/ANTENNA/essai-mifa-kd-n/calculs/<identifiant>/` (son `antenne.py` et son `resultats.json`).
- Si `n` = 3 diverge ou rend un court-circuit : c'est l'écart de 0,157 mm, dis-le-moi avec la plus petite cellule annoncée.

## Après
Donne-moi le JSON (ou dis-moi qu'il est à la racine) : je remplis le tableau dans `A-FAIRE.md`, je corrige le gabarit et je mets le banc à jour. Cases concernées :
- `web_antenna/A-FAIRE.md:321-328` (plan, point 1) et `:304-311` (« Reste ouvert — le MIFA ») ; la ligne MIFA du tableau l.297 ; la limite l.372.
- `web_antenna/README.md:2038-2045` (« Un méandre serré sort du modèle »).
- `web_antenna/js/22-antennes.js:902` (`kd:100`) et `:872` (aide de la cote).
- `web_antenna/test/banc-interface.js:482-487` (le banc de `kd`).
