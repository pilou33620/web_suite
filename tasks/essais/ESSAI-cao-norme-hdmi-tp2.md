# Essai : HDMI 1.4b, masque du puits à TP2 (figure 4-32)

Qui : toi · Où : là où tu as accès à la spécification HDMI 1.4b (accès adoptant) · Durée : 15 min · Source : `web_cao/A-FAIRE.md:264` et l.54

## Pourquoi
Le gabarit « HDMI 1.4 TMDS — récepteur (TP2) » est dans `python/oeil.py:413-446` :
- hexagone (l.439-441) : 0 mV à **±0,35 UI** du centre, **±75 mV de −0,25 à +0,25 UI** ;
- limites ±780 mV (l.442) ;
- marqué « recoupé » (l.417) mais **repris d'une source secondaire** : le masque publié par le logiciel ouvert ngscopeclient (`src/ngscopeclient/masks/hdmi-1.4-sink-tp2.yml`, « HDMI v1.4b figure 4-32 »), la norme n'ayant pas été lue (commentaire l.418-428, essai `python/test/banc-oeil.py:428-430`).

Si ngscopeclient s'est trompé (unités, centre, mV crête ou différentiels), un œil HDMI est jugé à tort.

Claude in Chrome ne peut pas : la spécification HDMI 1.4b et sa procédure de conformité ne sont diffusées qu'aux adoptants, sous accord ; il n'existe pas de copie en ligne qu'on puisse citer comme source légitime, et relire ngscopeclient une deuxième fois n'apporterait rien (c'est déjà la source secondaire du code).

Sans accès adoptant, l'essai ne peut pas se faire : dis-le, la case restera ouverte.

## Ce que tu prépares
- *High-Definition Multimedia Interface Specification, Version 1.4b* : § 4.2.6, figure 4-32 (masque de l'œil du puits à TP2) et son tableau.
- Pour comparaison, la *Compliance Test Specification* HDMI 1.4b si tu l'as.

## C'est réussi si
- [ ] Figure, paragraphe, page.
- [ ] Les six sommets (UI, mV) et l'origine des UI.
- [ ] Limites ±… mV.
- [ ] Débits concernés.
- [ ] Ou : « pas d'accès à la norme ».

## Ce qui reste à toi
1. Ouvre la figure 4-32 (vérifie numéro et paragraphe).
   → à relever : numéro de figure, paragraphe, page.
2. Relève les sommets du masque : position (UI, en précisant l'origine : centre de l'œil ou début de l'UI) et tension (mV, différentielle).
   → attendu (code) : 0 mV à ±0,35 UI du centre ; ±75 mV de −0,25 à +0,25 UI.
3. Relève les limites hautes et basses (zones interdites).
   → attendu (code) : ±780 mV.
4. Note pour quel débit le masque vaut (jusqu'à 3,4 Gb/s ? un masque par débit ?) et la charge de mesure.
   → attendu (code) : débit 3,4 Gb/s, charge 100 Ω différentielle.

## Après
Pour Claude Code, avec ta réponse :
- Concordant :
  - [ ] la source de `web_cao/python/oeil.py:430-434` devient « HDMI 1.4b figure 4-32, relue » (plus « source secondaire »), la note l.435-438 aussi, et le commentaire l.418-428 ;
  - [ ] cocher la mention de `web_cao/A-FAIRE.md:54` et corriger « source secondaire » à la ligne l.264 (déjà [x]).
- Différent :
  - [ ] corriger les points `web_cao/python/oeil.py:440-442`, l'essai `web_cao/python/test/banc-oeil.py:428-430` et `web_cao/docs/simulation-em.md:929`.
- Pas d'accès : laisser la case ouverte, rien à changer.
