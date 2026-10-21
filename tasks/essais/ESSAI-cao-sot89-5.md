# Essai : empreinte SOT-89-5 (Torex) sur une vraie carte

Où : atelier, avec une carte qui porte l'empreinte `SOT-89-5` (le XC6231 soudé ou non) · Durée : 30 min · Source : `web_cao/A-FAIRE.md:288` (rappel l.47)

## Pourquoi
L'empreinte `SOT-89-5` (LIB : `PROJETS/LIB_CAO/lib_empreinte_pcb/SOT-89-5.json`, posée pour le régulateur XC6231A332PR-G, `A-FAIRE.md:287`) est **calculée depuis le contour du boîtier Torex** : Torex ne publie pas de pastilles recommandées dans la datasheet, et son site a refusé le téléchargement du plan de boîtier. Elle fait aussi partie des 28 empreintes qui gardent l'origine de la fiche au lieu du centre (`FP_ORIGINE_FICHE`, `web_cao/editeur-pcb/test/harness.js:23531`). Une pastille trop courte ou mal placée donne des soudures sans ménisque, une languette (dissipation) mal reliée, ou un régulateur qui bascule au refusion.

Je n'ai pas pu lire le fichier d'empreinte (dépôt WEB_SUITE_PROJETS hors de ma session) : l'étape 1 te fait relever ses cotes.

## Ce qu'il te faut
- Une carte fabriquée avec l'empreinte `SOT-89-5`, de préférence une nue et une assemblée.
- Un XC6231 (ou tout SOT-89-5 Torex) en vrac si la carte est nue.
- La Gestion LIB ouverte sur `SOT-89-5`, un pied à coulisse ou un microscope avec réticule, une loupe.

## Étapes
1. Dans la Gestion LIB, ouvre `SOT-89-5` et note pour chaque pastille son numéro, sa largeur × longueur et son centre.
   → à relever : un tableau n° / l × L (mm) / X, Y (mm) ; et l'emplacement de l'origine.
2. Sur la carte nue, mesure deux pastilles de pattes (une de chaque rangée) et la grande pastille de languette.
   → attendu : les mêmes cotes qu'à l'étape 1, à ±0,05 mm (sinon c'est la fabrication, pas l'empreinte).
3. Pose un SOT-89-5 à sec sur l'empreinte, bien centré, et regarde à la loupe.
   → à relever : de combien chaque patte dépasse vers l'extérieur de sa pastille (pointe) ou reste dedans ; si la languette tombe entière sur sa pastille ; si une patte touche la pastille voisine.
4. Sur la carte assemblée, regarde les soudures.
   → à relever : ménisque visible en bout de chaque patte (oui / non), languette soudée sur toute sa longueur, régulateur droit ou tourné.
5. Si tu n'as aucune carte : compare l'étape 1 à une empreinte de référence publique (KiCad `Package_TO_SOT_SMD:SOT-89-5`, ou un plan de pastilles d'un autre fondeur pour le même boîtier) et dis-le dans la réponse — ce n'est alors qu'un recoupement, pas l'essai réel.

## C'est réussi si / La réponse
- [ ] Le tableau des pastilles (étape 1).
- [ ] Patte posée : dépassement de pointe ≥ 0 (patte sur la pastille) et pastille qui dépasse en bout pour le ménisque ; valeur relevée en mm.
- [ ] Languette entièrement sur sa pastille : oui / non.
- [ ] Soudures (si carte assemblée) : bonnes / à reprendre, avec photo.
- [ ] Source de la vérification : carte réelle, ou seulement recoupement (étape 5).

## Après
- Si c'est bon : je coche la moitié SOT-89-5 de `web_cao/A-FAIRE.md:288` et de la mention l.47.
- Sinon : je corrige les pastilles de `PROJETS/LIB_CAO/lib_empreinte_pcb/SOT-89-5.json` (longueur de pointe, entraxe, languette) d'après tes mesures, en gardant l'origine (essai `FP_ORIGINE_FICHE`), et je te dis quelles cartes la portent.
- La moitié LED RVB de la même case est dans `ESSAI-cao-led-rvb.md`.
