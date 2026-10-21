# Essai : patch FR-4 2,45 GHz, balayage fin de l'encastrement `y₀` (le reste réactif au pied)

Où : poste avec openEMS (WEB_ANTENNA lancé par WEB·SUITE) · Durée : 7 points × 5 à 6 min (le même document a tourné à 420 000–430 000 cellules, 46 et 56 min pour 9 points le 02/10/2026), compte 40 min à 1 h · Source : `web_antenna/A-FAIRE.md:339` (plan, point 4 ; origine des 15 Ω l.164-183 ; strate l.205-215)

## Pourquoi
La dixième simulation du 02/10/2026 (gabarit d'alors, `g` 1,49, `y₀` 8,51) laissait au pied du patch **57,9 + 15,5 j Ω** : un reste inductif « qu'un encastrement un peu plus court compenserait » (`A-FAIRE.md:173-183`). Le plan : balayer `y₀` finement sous le défaut corrigé, lire où Im Z au pied s'annule, et reporter le recul sur la strate FR-4 de `patchCorrRbord` (0,173 aujourd'hui, `js/22-antennes.js:231-238`).

**Ce que les relevés disent déjà, et qui change la plage** (à trancher avec toi avant de lancer) :
- Depuis le 03/10/2026, le gabarit corrigé pose **`y₀` = 5,9594 mm** (FR-4 1,6 mm, 2,45 GHz ; `L` 27,5856, `g` 1,4938). Les 15 Ω inductifs étaient ceux de `y₀` = 8,51 : ils sont déjà corrigés, et au-delà.
- Dans `croisement-L-y0.json`, Im Z **au pied** vaut −7,8 j à `y₀` = 6 et +4,0 j à `y₀` = 7 (`L` 27,7) ; −5,0 j et +6,4 j à `L` 27,2. Le point de confirmation (`L` 27,55, `y₀` 6,6) donne **+0,4 j**. Im Z s'annule donc vers **6,6 mm, au-dessus du défaut** : au défaut, le pied doit être **capacitif d'environ 8 Ω**, pas inductif.
- Le plan arrêté (`y₀` de −1,5 à 0 mm sous le défaut, soit 4,4594 → 5,9594) ne contiendrait donc **aucun** zéro : sept points tous capacitifs.
- **Je propose la même finesse, décalée** : de −0,5 à +1,0 mm autour du défaut, **5,4594 → 6,9594 par 0,25** (7 points), qui encadre 6,6. Si tu préfères tenir le plan, les valeurs sont 4,4594 → 5,9594 par 0,25 — la spec marche pareil.
- 6,6 mm correspond à une strate de **0,196** (calcul fait depuis le `G1` du gabarit) : c'est déjà la valeur à 2,45 GHz du tableau (0,151 / 0,196 / 0,173 à 868 MHz / 2,45 / 5,8 GHz, `A-FAIRE.md:205-207`). Ce que l'essai ajoute vraiment : **la mesure au défaut lui-même**, jamais simulé tel quel, et la pente de Im Z en ohms par millimètre autour de lui.

## Avant de commencer
- WEB_ANTENNA à jour (spec écrite sur `c95f9b6` du 21/10/2026). Dans WEB_ANTENNA, `python python/test/banc-openems.py` → **1053 vérifications, toutes passées**.
- « 📁 Projet » : nom `essai-patch-y0-fin`, **Ctrl+S** avant de lancer.
- Document de départ, **le même que `croisement-patch-2026-10.json`** : **✏️ Concevoir** → **« 2 couches · FR-4 1,6 mm · 35 µm »**, **âme retouchée de 1,530 à 1,600 mm** (εr 4,30, tan δ 0,020 — c'est le « FR-4 1,6 mm d'âme » des relevés ; sans cette retouche les tableaux ne se lisent pas ensemble, `A-FAIRE.md:95-97`) → **2,45 GHz** → motif **« Patch rectangulaire »**, cotes par défaut → **Dessiner ce motif**.
- Cotes que tu dois voir : `L` 27,5856 · `W` 37,5839 · `y₀` **5,9594** · `wf` 3,112 · `g` 1,4938 · `Lf` 6,4 · marge 4,8 mm. Sinon, arrête-toi et dis-le-moi.
- Réglages des croisements du patch : bande du motif **2,0825 à 2,8175 GHz, 401 points**, boîte et maillage calculés (0), PML 8, règle des tiers, arrêt **−40 dB**, pas maximum 0, diagramme de rayonnement coché, port localisé 50 Ω avec la ligne déclarée par le motif (6,4 mm × 3,112 mm : c'est elle qui donne Z au pied).

## Étapes
1. Étape « Le calcul » → **Balayer une cote** → **« Patch rectangulaire — encastrement »** → De **5,4594** à **6,9594** par pas de **0,25** (ou 4,4594 → 5,9594 si tu tiens le plan). Croisement décoché.
   → attendu : 7 valeurs.
2. **∑ Chiffrer la plage**.
   → attendu : 7 points, environ 420 000 cellules chacun, pas de refus.
3. **▶ Lancer le balayage**.
   → attendu : 7 lignes au journal ; la résonance bouge à peine (quelques MHz : c'est `L` qui fait la fréquence), le S₁₁ passe par un minimum, et Im Z au pied croît avec `y₀`.
4. **Ctrl+S**.
5. Enregistrer le relevé sous `balayage-patch-y0-fin.json` à la racine de WEB_ANTENNA (format des précédents) — depuis le dossier de WEB·SUITE :
   ```
   python tasks/essais/ESSAI-antenna-releve.py WEB_ANTENNA/balayage-patch-y0-fin.json "Patch gabarit corrige, FR-4 1,6 mm d'ame, 2,45 GHz, y0 de 5,4594 a 6,9594 par 0,25 (ou plage du plan), memes reglages que croisement-patch-2026-10.json" "y0 fin=PROJETS/ANTENNA/essai-patch-y0-fin/resultats.json"
   ```
   → attendu : « … : 1 cas, 7 points ».

## C'est réussi si
- [ ] Les 7 points sont calculés, convergés à −40 dB.
- [ ] Le point `y₀` = 5,9594 (le défaut) est dans la plage, et Im Z au pied y est lu.
- [ ] Avec la plage proposée : Im Z au pied change de signe entre deux points.
- [ ] Le point le plus proche de 6,6 mm retombe près de la confirmation du 02/10/2026 (2,450 GHz, −39,9 dB, 50,9 + 0,4 j au pied) : c'est le témoin du document.
- [ ] `WEB_ANTENNA/balayage-patch-y0-fin.json` existe, 7 points.

## Ce qu'on en tirera
- Le `y₀` où Im Z au pied s'annule, interpolé entre deux points, et ce qu'il donne de strate : `k = 2·G1·50 / cos²(π·y₀/L)` (je le calcule).
- **Si le zéro tombe vers 6,6 mm** (k ≈ 0,196) : la mesure confirme que la strate FR-4 moyenne (0,173) laisse ~8 Ω capacitifs au pied à 2,45 GHz. Monter la strate à 0,196 corrigerait 2,45 GHz mais dégraderait 868 MHz (0,151) : la décision est soit de garder la moyenne et d'écrire l'écart dans l'aide de la cote, soit de faire dépendre la strate de h/λ₀. Je ne corrige pas la formule sur ce seul relevé.
- **Si le zéro tombe ailleurs** (le défaut n'est pas où le croisement l'interpolait) : c'est `g` ou `L` du gabarit corrigé qui a déplacé l'adaptation, et je le noterai à côté de la loi de longueur.
- La pente de Im Z (Ω/mm) dit quelle tolérance tenir sur `y₀` en fabrication.

## Si ça rate, note
- Le refus de « Chiffrer » (il nomme le point), ou la ligne du journal du point raté.
- Le journal : panneau **« Journal du solveur »** de la page (pas écrit sur disque) — les lignes du point, diagnostic d'énergie compris.
- Le dossier du point : `PROJETS/ANTENNA/essai-patch-y0-fin/calculs/<identifiant>/`.
- Si Z au pied manque dans le tableau : la ligne n'est pas déclarée au port (étape « Les ports », longueur 6,4 et largeur 3,112) — redessine le motif.

## Après
Donne-moi le JSON : je remplis le tableau dans `A-FAIRE.md` et je décide de la strate avec toi. Cases concernées :
- `web_antenna/A-FAIRE.md:339-343` (plan, point 4 ; la plage s'y corrige si tu valides la proposée), `:164-183` (les 15 Ω), `:205-215` (les strates), la limite l.371.
- `web_antenna/js/22-antennes.js:214-238` (`patchCorrRbord` et son commentaire) et l'aide de `y₀` l.251-255.
