# Essai : monopôle, croisement largeur du brin `wr` × longueur `Lm` (la réactance de +16j)

Où : poste avec openEMS (WEB_ANTENNA lancé par WEB·SUITE) · Durée : 12 points (un croisement de 9 + un balayage de 3) × 5 à 13 min (repère : le patch FR-4, 420 000 à 490 000 cellules par point), « ∑ Chiffrer la plage » fait foi — compte 1 h à 2 h 30 · Source : `web_antenna/A-FAIRE.md:329` (plan, point 2 ; constat l.294 et l.310-311)

## Pourquoi
Corrigé le 03/10/2026 (brin × 1,29, masse à 0,8 × brin), le monopôle résonne à **2,435 GHz, −16,3 dB, mais avec une réactance de +16j qui reste** (`A-FAIRE.md:294`). Élargir le brin réduit l'inductance — et abaisse aussi la résonance, d'où le croisement avec `Lm`. Le résultat décide du **défaut de `wr`** : le rapport au défaut actuel où Im Z ≈ 0 à la résonance.

Défauts sur FR-4 1,53 mm à 2,45 GHz (`js/22-antennes.js:411-428`) : `Lm` 24,2416 · `wr` 5,952 (= 2 × `wf`) · `Lg` 19,3933 · `wf` 2,976 · `Lb` 24,312 · marge 4,59 mm. Les valeurs du plan :

| | ×0,6 | ×1 | ×1,5 | ×2 |
|---|---|---|---|---|
| `wr` | 3,5712 | 5,952 | 8,928 | 11,904 mm |

| | −3 % | 0 | +3 % |
|---|---|---|---|
| `Lm` | 23,5144 | 24,2416 | 24,9688 mm |

## Avant de commencer
- WEB_ANTENNA à jour (spec écrite sur `c95f9b6` du 21/10/2026). Dans WEB_ANTENNA, `python python/test/banc-openems.py` → **1053 vérifications, toutes passées**.
- **Pourquoi deux lancements** : le balayage prend « de … à … par pas de … », et 0,6 / 1 / 1,5 / 2 ne sont pas à pas constant. On fait donc un croisement `wr` = ×1 / ×1,5 / ×2 (pas de 2,976 mm) × `Lm` (9 points), puis `wr` = ×0,6 posé à la main et un balayage de `Lm` seul (3 points). Le pas de maillage se fige sur le document au lancement (`python/openems_modele.py:5563`) ; le cuivre le plus étroit reste la ligne (2,976 mm) même à `wr` = 3,5712 : le pas est le même dans les deux lancements.
- Deux projets, un par lancement (`resultats.json` ne garde que le dernier calcul) : `essai-monopole-wr-lm` puis `essai-monopole-wr06`. Dans le panneau **« 📁 Projet »**, changer le nom puis **Ctrl+S** fait une copie sous ce nom : c'est ainsi qu'on passe de l'un à l'autre.
- Document de départ : **✏️ Concevoir** → **« 2 couches · FR-4 1,6 mm · 35 µm »** (âme 1,530 mm, εr 4,30, tan δ 0,020, non retouchée) → **2,45 GHz** → motif **« Monopôle imprimé »**, cotes par défaut → **Dessiner ce motif**.
- Bande posée par le motif : **2,0825 à 2,8175 GHz** (±15 %), 401 points — `wr` × 2 et `Lm` + 3 % resteront dedans. Le reste aux valeurs par défaut, comme les croisements du patch : boîte et maillage calculés (0), PML 8, règle des tiers, arrêt **−40 dB**, pas maximum 0, diagramme de rayonnement coché, port localisé 50 Ω.
- `Lb` (largeur de carte) reste à 24,312 mm sur tous les points : à `wr` = 11,904 il reste 6,2 mm de substrat de chaque côté du brin. C'est voulu (seule la cote balayée change), ne la retouche pas.

## Étapes
1. « 📁 Projet » : nom `essai-monopole-wr-lm`, **Ctrl+S**. Étape « Le calcul » → **Balayer une cote** → **« Monopôle imprimé — largeur du brin »** → De **5,952** à **11,904** par pas de **2,976**.
   → attendu : 3 valeurs (5,952 · 8,928 · 11,904).
2. **Croiser avec une seconde cote** → **« Monopôle imprimé — longueur du brin »** → De **23,5144** à **24,9688** par pas de **0,7272**.
   → attendu : « 3 × 3 = 9 points ». (Le champ peut se colorer pour un pas hors de sa grille de 0,05 : la valeur est quand même prise.)
3. **∑ Chiffrer la plage**, note cellules, durée, plus petite cellule. **▶ Lancer le balayage**.
   → attendu : 9 lignes dans le journal ; à `Lm` fixé, la résonance descend quand `wr` monte.
4. **Ctrl+S**.
5. « 📁 Projet » : renomme en `essai-monopole-wr06`, **Ctrl+S** (copie). Dans les cotes du motif, mets **`wr` = 3,5712** (le reste inchangé) → **Dessiner ce motif**. Étape « Le calcul » → **Balayer une cote** → **« Monopôle imprimé — longueur du brin »** → De **23,5144** à **24,9688** par pas de **0,7272**, croisement **décoché**.
   → attendu : 3 points. **∑ Chiffrer**, **▶ Lancer le balayage**, puis **Ctrl+S**.
6. Enregistrer le relevé sous `croisement-monopole-wr-Lm.json` à la racine de WEB_ANTENNA (format des précédents) — depuis le dossier de WEB·SUITE :
   ```
   python tasks/essais/ESSAI-antenna-releve.py WEB_ANTENNA/croisement-monopole-wr-Lm.json "Monopole wr x Lm, FR-4 1,53 mm, 2,45 GHz, reglages par defaut, arret -40 dB. wr x1/x1,5/x2 croise, wr x0,6 balaye seul" "wr x1 a x2 croise Lm=PROJETS/ANTENNA/essai-monopole-wr-lm/resultats.json" "wr x0,6 (3,5712 mm), Lm seul=PROJETS/ANTENNA/essai-monopole-wr06/resultats.json"
   ```
   → attendu : « … : 2 cas, 12 points ».

## C'est réussi si
- [ ] Les 12 points sont calculés (9 + 3), aucun « aucune grandeur exploitable ».
- [ ] Convergence à −40 dB partout.
- [ ] Le point `wr` 5,952 × `Lm` 24,2416 retombe près du relevé du 03/10/2026 (2,435 GHz, +16j) : c'est le témoin du document.
- [ ] Im Z change de signe entre deux `wr` sur au moins une ligne de `Lm` (sinon la plage ne l'encadre pas).
- [ ] `WEB_ANTENNA/croisement-monopole-wr-Lm.json` existe, 2 cas.

## Ce qu'on en tirera
- Im Z **au port** à la résonance, où le +16j a été lu ; le relevé garde aussi Z au pied, je lirai les deux.
- **Si Im Z passe par zéro entre ×1 et ×2** : on retient le rapport interpolé (par exemple ×1,4), et `wr` par défaut passe de `2·wf` à ce rapport × `2·wf`. Le `Lm` qui garde 2,45 GHz à ce `wr` se lit sur la même ligne, et donne la nouvelle correction de longueur (1,29 descendra, puisque le brin large résonne plus bas).
- **Si Im Z reste positif même à ×2** : la largeur ne suffit pas, l'inductance vient d'ailleurs (la ligne, le raccord brin-masse) ; le défaut de `wr` ne bouge pas, et c'est `Lg` ou la marge qu'il faudrait croiser.
- **Si ×0,6 est déjà le meilleur** : le sens est l'inverse de celui attendu, et je relirai les courbes avant de toucher au gabarit.

## Si ça rate, note
- Le refus de « Chiffrer » (il nomme le point), ou la ligne du journal du point raté.
- Le journal : panneau **« Journal du solveur »** de la page (il n'est pas écrit sur disque) — copie les lignes du point, diagnostic d'énergie compris.
- Le dossier du point : `PROJETS/ANTENNA/<projet>/calculs/<identifiant>/` (`antenne.py`, `resultats.json`).
- Si la liste des cotes ne propose plus « Monopôle imprimé — … » : le motif a été retouché à la main (un via, une forme du motif déplacée) ; redessine-le depuis le panneau.

## Après
Donne-moi le JSON : je remplis le tableau dans `A-FAIRE.md`, je corrige le gabarit et le banc. Cases concernées :
- `web_antenna/A-FAIRE.md:329-333` (plan, point 2), `:294` (ligne monopôle du tableau), `:310-311`.
- `web_antenna/js/22-antennes.js:413` (`wr`) et `:415-422` (correction de `Lm`, `ponytail:`).
- `web_antenna/test/banc-interface.js` (section 4, les motifs) si le défaut change.
