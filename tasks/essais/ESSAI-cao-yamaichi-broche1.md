# Essai : broche 1 du support Yamaichi IC51-0484-806

Où : atelier, avec le support réel en main (et un PC avec la Gestion LIB ouverte) · Durée : 15 min · Source : `web_cao/A-FAIRE.md:198` (rappel l.47)

## Pourquoi
L'empreinte `Yamaichi-IC51-0484-806` (LIB : `PROJETS/LIB_CAO/lib_empreinte_pcb/Yamaichi-IC51-0484-806.json`) a été reprise le 16/10/2026 : trous NPTH revérifiés sur fiche, corps tourné en 32 × 29 mm (`A-FAIRE.md:190`). La broche 1, elle, n'est pas marquée sur le plan 48 broches de Yamaichi ; sa place dans l'empreinte vient d'**une seule source indirecte** (`tasks/todo-points-ouverts.md:75` : « broche 1 en doute écrit »). Si elle est fausse, le QFP-48 posé dans le support est tourné de 90° ou 180° : toutes ses broches sur le mauvais net, carte de test inutilisable.

Je n'ai pas pu lire le fichier d'empreinte (dépôt WEB_SUITE_PROJETS hors de ma session) : la première étape te fait relever ce qu'il dit.

## Ce qu'il te faut
- Le support IC51-0484-806 lui-même (neuf ou monté sur une carte).
- La Gestion LIB ouverte sur l'empreinte `Yamaichi-IC51-0484-806`.
- Une loupe, un pied à coulisse.

## Étapes
1. Dans la Gestion LIB, ouvre l'empreinte `Yamaichi-IC51-0484-806` (vue de dessus, côté composant).
   → à relever : le coin ou le côté où se trouve la pastille « 1 », par rapport au grand côté (32 mm) et au petit côté (29 mm) du corps ; la place des trous NPTH de centrage par rapport à elle.
2. Pose le support dessus devant toi (côté où l'on insère la puce vers le haut). Cherche le repère moulé : triangle, point, chanfrein, ou « 1 » gravé, sur le corps ou sur le capot.
   → à relever : la nature du repère et le coin ou le côté où il est.
3. Retourne le support (vue de dessous, côté picots). Repère le picot sous le repère de l'étape 2 et les plots de centrage.
   → à relever : la place du picot 1 par rapport aux plots de centrage. Attention : la vue de dessous est en miroir de l'empreinte.
4. Mesure au pied à coulisse la distance du picot 1 au plot de centrage le plus proche (X et Y, en mm), pour la comparer à l'empreinte.
   → à relever : X, Y en mm, à ±0,2 mm.
5. Compare : remets-toi en vue de dessus et dis si la pastille 1 de l'empreinte tombe sous le picot repéré.

## C'est réussi si / La réponse
- [ ] Le repère moulé existe : oui / non, et sa forme.
- [ ] Coin du picot 1 en vue de dessus (ex. « coin haut-gauche, grand côté horizontal »).
- [ ] La pastille 1 de l'empreinte est au même endroit : oui / non (sinon : tourne de 90°, 180° ou 270°, dans quel sens).
- [ ] Cote picot 1 → plot de centrage : X = … mm, Y = … mm.
- [ ] Une photo du repère (dessus et dessous), si tu peux.

## Après
- Si c'est juste : je retire le doute écrit de l'empreinte et je coche `web_cao/A-FAIRE.md:198` et la mention l.47.
- Si c'est faux : je renumérote les pastilles de `PROJETS/LIB_CAO/lib_empreinte_pcb/Yamaichi-IC51-0484-806.json` (rotation de la numérotation seulement, trous et corps gardés), je mets à jour la ligne de `LIB_composants.csv`, puis je coche les mêmes cases. Toute carte qui porte déjà ce support est à revoir : je te les liste.
