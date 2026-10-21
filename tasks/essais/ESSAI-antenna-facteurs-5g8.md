# Essai : les facteurs du dipôle, du monopôle et de l'IFA tiennent-ils hors de leur cas ?

Où : poste avec openEMS (WEB_ANTENNA lancé par WEB·SUITE) · Durée : 6 simulations seules × 5 à 15 min (repère : le patch FR-4 à 2,45 GHz, 420 000 à 490 000 cellules ; à 5,8 GHz la boîte est plus petite mais la maille plus fine), le bilan de chaque calcul fait foi — compte 1 h à 1 h 30 · Source : `web_antenna/A-FAIRE.md:334` (plan, point 3 ; constat l.299-302)

## Pourquoi
Les trois facteurs mesurés le 03/10/2026 viennent d'**un seul cas chacun**, FR-4 1,53 mm à 2,45 GHz (`A-FAIRE.md:299-302`) :

| motif | facteur | confirmé à 2,45 GHz, FR-4 1,53 mm | code |
|---|---|---|---|
| dipôle | bras × 1,165 du quart d'onde | 2,461 GHz · −34,3 dB · 48 Ω | `js/22-antennes.js:1046-1052` |
| monopôle | brin × 1,29, masse à 0,8 × brin | 2,435 GHz · −16,3 dB · +16j | `js/22-antennes.js:415-425` |
| IFA | bras × 1,018, écart × 1,8 | 2,48 GHz · bande 2,405–2,575 GHz | `js/22-antennes.js:613-620` |

Deux confirmations par motif, **aux cotes par défaut, sans croisement** : 5,8 GHz sur le même FR-4 (la fréquence change), 2,45 GHz sur RO4003C 0,813 mm (le substrat change). Le résultat décide, motif par motif : facteur confirmé, ou croisement à mener à cette fréquence.

## Avant de commencer
- **Écart à ta consigne, je suis le plan** : tu m'as écrit « 5,8 GHz sur FR-4 et RO4003C » ; `A-FAIRE.md:335-336` dit **5,8 GHz sur FR-4 1,53 mm** et **2,45 GHz sur RO4003C 0,813 mm**. C'est ce que fait cette spec — dis-le-moi si tu voulais le RO4003C à 5,8 GHz.
- WEB_ANTENNA à jour (spec écrite sur `c95f9b6` du 21/10/2026). Dans WEB_ANTENNA, `python python/test/banc-openems.py` → **1053 vérifications, toutes passées**.
- Six projets, un par calcul (`resultats.json` ne garde que le dernier) : changer le nom dans **« 📁 Projet »** puis **Ctrl+S** fait une copie sous ce nom ; un calcul lancé avant le nom est déplacé du TEMP au projet à l'enregistrement.
- Les deux empilages :
  - **A** : **« 2 couches · FR-4 1,6 mm · 35 µm »** tel quel (âme 1,530 mm, εr 4,30, tan δ 0,020), fréquence visée **5,8 GHz**.
  - **B** : **« 2 couches · RO4003C 1,524 mm (60 mil) »**, **âme retouchée à 0,813 mm** (aucun modèle d'usine ne la donne ; εr 3,55, tan δ 0,0027 restent), fréquence visée **2,45 GHz**.
- Réglages identiques aux mesures du 03/10/2026 : la bande que pose le motif (±15 % : 4,93 à 6,67 GHz pour A, 2,0825 à 2,8175 GHz pour B, 401 points), boîte et maillage calculés (0), PML 8, règle des tiers, arrêt **−40 dB**, pas maximum 0, diagramme de rayonnement coché, port localisé 50 Ω.
- Les cotes que tu dois voir au motif (calculées le 21/10/2026 par le code du dépôt ; si elles diffèrent, arrête-toi et dis-le-moi) :

| point | motif | cotes par défaut | résonance annoncée par la fiche |
|---|---|---|---|
| A1 | Dipôle imprimé | `La` 9,2477 · `wd` 1 · `ov` 1,53 · marge 6,12 | 5,8000 GHz |
| A2 | Monopôle imprimé | `Lm` 10,24 · `wr` 5,952 · `Lg` **10** · `wf` 2,976 · `Lb` 24,312 | 5,8000 GHz |
| A3 | F inversé (IFA) | `La` 8,877 · `ha` 2,58 · `wb` 0,6 · `d` 2,322 · `Lg` 25 · `Lb` 30 | 5,7981 GHz |
| B1 | Dipôle imprimé | `La` 23,6282 · `wd` 1,6901 · `ov` 0,813 | 2,4500 GHz |
| B2 | Monopôle imprimé | `Lm` 26,1634 · `wr` 3,638 · `Lg` 20,9307 · `wf` 1,819 · `Lb` 20 | 2,4500 GHz |
| B3 | F inversé (IFA) | `La` 22,009 · `ha` 6,12 · `wb` 1,22 · `d` 5,508 · `Lg` 30,59 · `Lb` 30 | 2,4496 GHz |

## Étapes
1. **✏️ Concevoir** → empilage A → **5,8 GHz** → motif **« Dipôle imprimé »**, cotes par défaut → « 📁 Projet » : `essai-facteurs-dipole-5g8`, **Ctrl+S** → **▶ Dessiner et simuler**.
   → attendu : un calcul, convergé à −40 dB ; note les cellules et la durée du bilan. **Ctrl+S** à la fin.
2. Même empilage : motif **« Monopôle imprimé »** → projet `essai-facteurs-monopole-5g8` → **▶ Dessiner et simuler** → **Ctrl+S**.
3. Motif **« F inversé (IFA) »** → projet `essai-facteurs-ifa-5g8` → **▶ Dessiner et simuler** → **Ctrl+S**.
4. Empilage B (RO4003C, âme à **0,813 mm**) → **2,45 GHz** → dipôle → `essai-facteurs-dipole-ro4003` → **▶ Dessiner et simuler** → **Ctrl+S**.
5. Monopôle → `essai-facteurs-monopole-ro4003` → idem.
6. IFA → `essai-facteurs-ifa-ro4003` → idem.
   → attendu, pour chacun des six : une résonance lisible (S₁₁ minimal sous −6 dB), loin du bord de bande.
7. Enregistrer le relevé sous `validation-facteurs.json` à la racine de WEB_ANTENNA (format des précédents, une liste de cas comme `croisement-patch-2026-10.json`) — depuis le dossier de WEB·SUITE :
   ```
   python tasks/essais/ESSAI-antenna-releve.py WEB_ANTENNA/validation-facteurs.json "Facteurs dipole/monopole/IFA aux cotes par defaut, sans croisement. A = FR-4 1,53 mm 5,8 GHz, B = RO4003C 0,813 mm 2,45 GHz. Bande +-15 %, 401 pts, arret -40 dB" "dipole FR-4 1,53 mm 5,8 GHz=PROJETS/ANTENNA/essai-facteurs-dipole-5g8/resultats.json" "monopole FR-4 1,53 mm 5,8 GHz=PROJETS/ANTENNA/essai-facteurs-monopole-5g8/resultats.json" "IFA FR-4 1,53 mm 5,8 GHz=PROJETS/ANTENNA/essai-facteurs-ifa-5g8/resultats.json" "dipole RO4003C 0,813 mm 2,45 GHz=PROJETS/ANTENNA/essai-facteurs-dipole-ro4003/resultats.json" "monopole RO4003C 0,813 mm 2,45 GHz=PROJETS/ANTENNA/essai-facteurs-monopole-ro4003/resultats.json" "IFA RO4003C 0,813 mm 2,45 GHz=PROJETS/ANTENNA/essai-facteurs-ifa-ro4003/resultats.json"
   ```
   → attendu : « … : 6 cas, 6 points ».

## C'est réussi si
- [ ] Les six calculs ont rendu une résonance, convergés à −40 dB.
- [ ] Aucune résonance collée au bord de bande (si oui, c'est déjà un écart de plus de 15 % : note-le, ne relance pas).
- [ ] Les cotes vues au motif sont celles du tableau.
- [ ] `WEB_ANTENNA/validation-facteurs.json` existe, 6 cas.

## Ce qu'on en tirera
- Écart de résonance à la cible, motif par motif : **sous 2 %** (±116 MHz à 5,8 GHz, ±49 MHz à 2,45 GHz), le facteur est confirmé et son `ponytail:` « UN SEUL cas mesuré » devient « trois cas » ; **au-delà**, on mène le croisement « cote de résonance × cote d'adaptation » de ce motif à cette fréquence, comme le 03/10/2026 (pour le monopôle, c'était brin × masse, `Lm` × `Lg`).
- **Une prévision à vérifier sur A2** : à 5,8 GHz, la masse du monopôle n'est pas à 0,8 × brin (8,19 mm) mais au plancher de **10 mm** (`js/22-antennes.js:425`). Si la loi mesurée `Lm + 0,43·Lg ≈ cste` suit la longueur d'onde, l'antenne est électriquement 5,6 % trop longue et résonnera vers **5,49 GHz** (−5,4 %). Si c'est ce qu'on voit, ce n'est pas le facteur 1,29 qui est en cause mais le plancher : la correction portera sur `Lm` en fonction de `Lg`, pas sur le 1,29.
- Une dérive qui suit le **substrat** (B) et pas la fréquence (A) dira que le facteur dépend de l'εr, donc de `eeffAir = (εr+1)/2` : c'est elle qu'il faudrait corriger, pour les quatre motifs à la fois.
- L'adaptation (S₁₁, Z) se lit en second : le plan ne juge que la résonance.

## Si ça rate, note
- Le refus de l'assistant avant le lancement (texte exact), ou l'alerte « Ce calcul prendra un moment » avec la durée annoncée.
- Le journal : panneau **« Journal du solveur »** de la page (il n'est pas écrit sur disque) — copie la fin, diagnostic d'énergie compris.
- Le dossier du calcul : `PROJETS/ANTENNA/<projet>/calculs/<identifiant>/` (`antenne.py`, `resultats.json`).
- Pour l'IFA : s'il ne rend aucune résonance et un rendement autour de 25 %, c'est le court-circuit d'alimentation du 03/10/2026 qui revient ; dis-le-moi avec une capture de la vue 2D autour du port.

## Après
Donne-moi le JSON : je remplis le tableau dans `A-FAIRE.md`, je corrige les facteurs qui tombent et j'ouvre les croisements nécessaires. Cases concernées :
- `web_antenna/A-FAIRE.md:334-338` (plan, point 3), `:299-302` (« des cas uniques »), le tableau l.292-297 et la limite l.370.
- `web_antenna/js/22-antennes.js:1046-1052` (dipôle), `:415-425` (monopôle), `:613-620` (IFA), chacun avec son `ponytail:`.
