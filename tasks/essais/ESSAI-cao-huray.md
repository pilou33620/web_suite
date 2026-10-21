# Essai : rugosité de Huray, a et SR depuis des fiches de feuillard (méthode « cannonball »)

Où : PC avec le papier de Simonovich et des fiches de feuillard ou de stratifié · Durée : 1 h · Source : `web_cao/A-FAIRE.md:266` (rappels l.53, l.371)

## Pourquoi
Les deux jeux de Huray de l'éditeur sont `editeur-pcb/js/01-core.js:720-721` :
- « Huray · ED standard » : a = 0,5 µm, SR = 1,5 ;
- « Huray · HVLP » : a = 0,3 µm, SR = 0,6.

Le commentaire l.709-712 dit seulement qu'ils donnent ~70 % et ~15 % de pertes cuivre en plus à 10 GHz, « ce que les fabricants annoncent ». Aucune fiche ne donne a et SR (elles donnent Rz / Ra) ; la méthode « cannonball » de Simonovich les tire de Rz, mais son texte n'a pas pu être lu (`A-FAIRE.md:266`). Le modèle du code (`python/ligne_mom.py:2131-2136`) : K = 1 + (3/2)·SR / (1 + δ/a + δ²/(2a²)), avec SR = N·4πa² / A_plat. Si a ou SR est faux, la perte d'une piste rapide est fausse d'autant (jusqu'à ×1,7 sur α_c au-dessus de quelques GHz), et l'œil ou le S21 avec.

## Ce qu'il te faut
- Bert Simonovich, « Practical Method for Modeling Conductor Surface Roughness Using Close Packing of Equal Spheres » (DesignCon 2015 ; titre à confirmer — aussi repris sur son site Lamsim Enterprises sous le nom « Cannonball-Huray »).
- Deux fiches de feuillard avec Rz : un ED standard (STD) et un HVLP (par exemple Mitsui Kinzoku, Circuit Foil, Furukawa, JX ; ou la fiche du stratifié qui nomme son cuivre). Côté traité (« matte side ») et côté tambour si les deux sont donnés.

## Étapes
1. Dans le papier, recopie les formules qui donnent le rayon a et SR à partir de Rz (et la taille de la tuile de base).
   → à relever : les formules exactes et le numéro d'équation ; la définition de SR (vérifie qu'elle est la même que N·4πa² / A_plat du code).
2. Dans chaque fiche, relève Rz (µm) des deux faces, et Ra si c'est tout ce qu'il y a.
   → à relever : feuillard, face, Rz en µm, méthode de mesure si dite (profilomètre, ISO 4287).
3. Applique les formules aux Rz du STD et du HVLP (face traitée, celle qui touche le diélectrique ; et face tambour pour un cuivre « traité inversé »).
   → à relever : a (µm) et SR pour chaque feuillard.
4. Si le papier donne un exemple chiffré, refais-le avec tes formules.
   → attendu : le même a et le même SR que le papier.

## C'est réussi si / La réponse
- [ ] Les formules (avec leur référence) et la définition de SR.
- [ ] Rz des deux feuillards, avec la fiche (fabricant, référence, date).
- [ ] a et SR calculés : STD → a = … µm, SR = … ; HVLP → a = … µm, SR = ….
- [ ] Écart aux jeux actuels (0,5 / 1,5 et 0,3 / 0,6).

## Après
- Je remplace les valeurs de `editeur-pcb/js/01-core.js:720-721` par tes a et SR, et je mets la fiche et le papier dans le commentaire l.709-712 ; si la définition de SR diffère, je corrige aussi `python/ligne_mom.py:2131-2136` et son banc `python/test/banc-ligne-mom.py`.
- Je recalcule les K à 10 GHz pour la note des jeux, et je coche `web_cao/A-FAIRE.md:266`, l.53 et l.371.
