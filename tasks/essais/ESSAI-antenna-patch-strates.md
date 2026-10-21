# Essai : patch, deux strates de plus pour `patchCorrRbord` (FR-4 haute fréquence, RO4003C 1,524 mm)

> **Plan proposé, à valider avec toi.** `A-FAIRE.md` dit quoi mesurer (l.213-215), pas comment : aucun plan n'est arrêté. Celui-ci est calqué sur la mesure RO4350B déjà faite, allégé de son croisement `L` × `y₀` puisque la loi de longueur tient désormais à 0,2 %.

Où : poste avec openEMS (WEB_ANTENNA lancé par WEB·SUITE) · Durée : 2 × (7 points + 1 confirmation) = 16 calculs × 5 à 8 min (le patch FR-4 1,6 mm tournait à 420 000–430 000 cellules et 5 à 6 min par point) — compte 1 h 30 à 2 h 30, plus 9 points par stratifié si le croisement de secours est nécessaire · Source : `web_antenna/A-FAIRE.md:213` (« À mesurer » ; strates l.205-212 ; limite l.371)

## Pourquoi
L'encastrement du patch sort de la résistance de bord, que le gabarit corrige par un rapport mesuré à la conductance de fente : **0,173 sur FR-4** (tan δ 0,02, trois fréquences, ±13 %) et **0,722 sur RO4350B** (tan δ 0,0037, 0,762 mm). Entre les deux, `patchCorrRbord` **interpole en tan δ** et se borne aux deux valeurs (`js/22-antennes.js:231-238`) : « une hypothèse à deux points » (`A-FAIRE.md:211-212`), que l'assistant signale sur tout autre stratifié (`js/31-cartes-antenna.js:1174-1195`). Deux mesures la trancheraient :
- **FR-4 haute fréquence** (tan δ 0,012, εr 4,20) : le gabarit y prend **0,4424** (milieu de la droite). La mesure dit si la droite tient.
- **RO4003C 1,524 mm** (tan δ 0,0027, borné à 0,722) : même famille de pertes que le RO4350B, **deux fois plus épais**. La mesure sépare les pertes de l'épaisseur.

Ce que le gabarit pose (calculé le 21/10/2026 par le code du dépôt), et où tomberait `y₀` selon la strate :

| stratifié | `L` | `W` | `wf` | `g` | `y₀` posé | `y₀` si 0,173 (FR-4) | si 0,196 | si 0,722 (RO) |
|---|---|---|---|---|---|---|---|---|
| FR-4 HF, âme 1,600 mm | 27,908 | 37,9435 | 3,167 | 1,5202 | **9,3907** | 5,92 | 6,58 | 10,45 |
| RO4003C, âme 1,524 mm | 30,3882 | 40,5634 | 3,409 | 1,6363 | **11,0947** | 5,55 | 6,40 | 11,09 |

## Avant de commencer
- WEB_ANTENNA à jour (spec écrite sur `c95f9b6` du 21/10/2026). Dans WEB_ANTENNA, `python python/test/banc-openems.py` → **1053 vérifications, toutes passées**.
- Ce que je propose de garder du RO4350B (`croisement-patch-2026-10.json`, cas 3) : le gabarit à ses cotes (`g` = 0,48 × `wf`), un **balayage de `y₀` au pas de 0,8 mm** à `L` fixé, la lecture de Im Z au pied. Ce que j'enlève : le croisement `L` × `y₀` 3 × 3 du début — il servait à régler `L`, que `patchCorrLongueur` règle maintenant à 0,2 % près sur quatre cas. Il revient en secours si la résonance tombe à plus de 1 % de 2,45 GHz.
- Plage : 7 points (le RO4350B en avait 5, prolongés), qui couvrent **les deux bornes** de la strate pour le FR-4 HF, et ±2,4 mm autour du posé pour le RO4003C.
- Documents de départ :
  - **FR-4 HF** : **✏️ Concevoir** → **« 2 couches · FR-4 haute fréquence 1,6 mm »**, **âme retouchée de 1,530 à 1,600 mm** (comme les relevés FR-4 : seul le stratifié change, pas l'épaisseur) → **2,45 GHz** → **« Patch rectangulaire »**, cotes par défaut → **Dessiner ce motif**.
  - **RO4003C** : **« 2 couches · RO4003C 1,524 mm (60 mil) »** tel quel → **2,45 GHz** → **« Patch rectangulaire »**, cotes par défaut → **Dessiner ce motif**.
  - Vérifie les cotes du tableau ci-dessus avant de lancer.
- Réglages des croisements du patch : bande du motif **2,0825 à 2,8175 GHz, 401 points**, boîte et maillage calculés (0), PML 8, règle des tiers, arrêt **−40 dB**, pas maximum 0, diagramme coché, port localisé 50 Ω, ligne déclarée par le motif.
- Un projet par lancement : changer le nom dans **« 📁 Projet »** puis **Ctrl+S** fait une copie.

## Étapes
1. FR-4 HF : projet `essai-strate-fr4hf`, **Ctrl+S**. Étape « Le calcul » → **Balayer une cote** → **« Patch rectangulaire — encastrement »** → De **5,8** à **10,6** par pas de **0,8** → **∑ Chiffrer la plage** → **▶ Lancer le balayage** → **Ctrl+S**.
   → attendu : 7 points (5,8 · 6,6 · 7,4 · 8,2 · 9,0 · 9,8 · 10,6), résonances vers 2,44 GHz, un S₁₁ qui passe par un minimum, Im Z au pied qui change de signe.
2. FR-4 HF, confirmation : projet `essai-strate-fr4hf-conf`. Mets `y₀` au zéro de Im Z au pied interpolé à l'étape 1 (arrondi à 0,05 mm), décoche le balayage → **▶ Lancer openEMS** → **Ctrl+S**.
   → attendu : S₁₁ sous −25 dB à la résonance. Si la résonance est à plus de 1 % de 2,45 GHz (hors 2,4255–2,4745), fais le secours : croisement `L` (posé ± 2,6 %, 3 valeurs) × `y₀` (confirmé ± 1,5 mm, 3 valeurs), comme le RO4350B, dans le projet `essai-strate-fr4hf-LY`.
3. RO4003C : projet `essai-strate-ro4003`, même geste, **De 8,7 à 13,5 par 0,8**.
   → attendu : 7 points (8,7 · 9,5 · 10,3 · 11,1 · 11,9 · 12,7 · 13,5).
4. RO4003C, confirmation : projet `essai-strate-ro4003-conf`, même geste qu'à l'étape 2.
5. Enregistrer le relevé sous `balayage-patch-strates.json` à la racine de WEB_ANTENNA (aucun nom n'est arrêté ; format des précédents, une liste de cas) — depuis le dossier de WEB·SUITE :
   ```
   python tasks/essais/ESSAI-antenna-releve.py WEB_ANTENNA/balayage-patch-strates.json "Strates de patchCorrRbord. Gabarit patch corrige, 2,45 GHz, balayage y0 au pas 0,8 puis confirmation. FR-4 HF ame 1,600 mm ; RO4003C 1,524 mm. Bande +-15 %, 401 pts, arret -40 dB" "FR-4 HF 1,6 mm, y0=PROJETS/ANTENNA/essai-strate-fr4hf/resultats.json" "FR-4 HF 1,6 mm, confirmation=PROJETS/ANTENNA/essai-strate-fr4hf-conf/resultats.json" "RO4003C 1,524 mm, y0=PROJETS/ANTENNA/essai-strate-ro4003/resultats.json" "RO4003C 1,524 mm, confirmation=PROJETS/ANTENNA/essai-strate-ro4003-conf/resultats.json"
   ```
   → attendu : « … : 4 cas, 16 points » (ajoute les croisements de secours s'il y en a eu).

## C'est réussi si
- [ ] Les deux balayages ont 7 points calculés, convergés à −40 dB.
- [ ] Sur chacun, Im Z au pied change de signe entre deux points (sinon l'optimum est au bord : prolonge de 2 points du côté qui manque, comme le RO4350B).
- [ ] Les deux confirmations tombent à moins de 1 % de 2,45 GHz, ou le croisement de secours est fait.
- [ ] `WEB_ANTENNA/balayage-patch-strates.json` existe.

## Ce qu'on en tirera
- Pour chaque stratifié, le rapport mesuré `k = 2·G1·50 / cos²(π·y₀/L)` au `y₀` réglé (je le calcule), à placer sur le graphe k(tan δ) à côté de FR-4 (0,02 ; 0,173) et RO4350B (0,0037 ; 0,722).
- **FR-4 HF près de 0,44** : la droite en tan δ tient, `patchCorrRbord` reste tel quel et son `ponytail:` passe à trois points. **Près de 0,17–0,20** : ce n'est pas la perte qui fait la strate mais la famille de résine (ou l'εr) ; la loi devient un tableau par matériau. **Près de 0,72** : la strate tombe vite dès qu'on quitte le FR-4 standard ; la droite devient une courbe.
- **RO4003C près de 0,72** : l'épaisseur ne compte pas, la strate est une affaire de pertes. **Nettement ailleurs** : `k` dépend aussi de h/λ₀, et la loi prend une seconde variable — sur deux points, je ne la poserai pas sans un troisième.
- La loi de longueur se vérifie en passant : la résonance des confirmations doit tomber à 0,2 % près si `patchCorrLongueur` tient sur ces deux stratifiés.

## Si ça rate, note
- Le refus de « Chiffrer » (il nomme le point), ou la ligne du journal du point raté.
- Le journal : panneau **« Journal du solveur »** de la page (pas écrit sur disque) — les lignes du point, diagnostic d'énergie compris.
- Le dossier du point : `PROJETS/ANTENNA/<projet>/calculs/<identifiant>/`.
- Si l'assistant affiche « L'encastrement du patch est interpolé, pas mesuré » : c'est normal sur ces deux stratifiés, c'est précisément ce qu'on mesure.

## Après
Donne-moi le JSON : je remplis le tableau dans `A-FAIRE.md`, je corrige `patchCorrRbord` si la mesure le dit, et j'adapte l'avertissement de l'assistant. Cases concernées :
- `web_antenna/A-FAIRE.md:205-215` (les strates), le tableau de la correction l.192-197, la limite l.371.
- `web_antenna/js/22-antennes.js:214-238` (`patchCorrRbord`, son commentaire et son `ponytail:`).
- `web_antenna/js/31-cartes-antenna.js:1174-1195` (l'avertissement « interpolé, pas mesuré » et sa tolérance sur tan δ).
- `web_antenna/README.md:2029-2037` (la limite « La correction du patch n'est mesurée que sur deux stratifiés »).
