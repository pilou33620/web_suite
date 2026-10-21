# Essai : HDMI 1.4b, masque du puits à TP2 (figure 4-32)

Où : PC avec la spécification HDMI 1.4b (diffusée aux adoptants, sous accord) · Durée : 15 min · Source : `web_cao/A-FAIRE.md:264` et l.54

## Pourquoi
Le gabarit « HDMI 1.4 TMDS — récepteur (TP2) » est dans `python/oeil.py:413-446` :
- hexagone (l.439-441) : 0 mV à **±0,35 UI** du centre, **±75 mV de −0,25 à +0,25 UI** ;
- limites ±780 mV (l.442) ;
- marqué « recoupé » (l.417) mais **repris d'une source secondaire** : le masque publié par le logiciel ouvert ngscopeclient (`src/ngscopeclient/masks/hdmi-1.4-sink-tp2.yml`, « HDMI v1.4b figure 4-32 »), la norme n'ayant pas été lue (commentaire l.418-428, essai `python/test/banc-oeil.py:428-430`).

Si ngscopeclient s'est trompé (unités, centre, mV crête ou différentiels), un œil HDMI est jugé à tort.

Sans accès adoptant à la norme, cet essai ne peut pas se faire : dis-le, la case restera ouverte.

## Ce qu'il te faut
- *High-Definition Multimedia Interface Specification, Version 1.4b* : § 4.2.6, figure 4-32 (masque de l'œil du puits à TP2) et son tableau.
- Pour comparaison, la *Compliance Test Specification* HDMI 1.4b si tu l'as.

## Étapes
1. Ouvre la figure 4-32 (vérifie numéro et paragraphe).
   → à relever : numéro de figure, paragraphe, page.
2. Relève les sommets du masque : position (UI, en précisant l'origine : centre de l'œil ou début de l'UI) et tension (mV, différentielle).
   → attendu (code) : 0 mV à ±0,35 UI du centre ; ±75 mV de −0,25 à +0,25 UI.
3. Relève les limites hautes et basses (zones interdites).
   → attendu (code) : ±780 mV.
4. Note pour quel débit le masque vaut (jusqu'à 3,4 Gb/s ? un masque par débit ?) et la charge de mesure.
   → attendu (code) : débit 3,4 Gb/s, charge 100 Ω différentielle.

## C'est réussi si / La réponse
- [ ] Figure, paragraphe, page.
- [ ] Les six sommets (UI, mV) et l'origine des UI.
- [ ] Limites ±… mV.
- [ ] Débits concernés.
- [ ] Ou : « pas d'accès à la norme ».

## Après
- Concordant : la source de `python/oeil.py:430-434` devient « HDMI 1.4b figure 4-32, relue » (plus « source secondaire »), la note l.435-438 aussi, et je coche la mention de `web_cao/A-FAIRE.md:54` (la ligne l.264 est déjà [x], je corrige « source secondaire »).
- Différent : je corrige les points l.440-442 et l'essai `python/test/banc-oeil.py:428-430`, et `docs/simulation-em.md:929`.
