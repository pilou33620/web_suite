# Essai : USB 2.0 Template 2, positions en % UI dans la figure 7-15

Où : PC avec la spécification USB 2.0 (usb.org) · Durée : 20 min · Source : `web_cao/A-FAIRE.md:263` (rappel l.52)

## Pourquoi
Le gabarit « USB 2.0 HS — extrémité de câble (Template 2/4) » de l'œil est dans `python/oeil.py:286-320` :
- hexagone (l.315-316), en UI comptés depuis le centre de l'œil : `[-0.375, 0]`, `[-0.15, ±0.175]`, `[0.15, ±0.175]`, `[0.375, 0]` — soit 0 mV à **12,5 et 87,5 % UI**, ±175 mV de **35 à 65 % UI** (position du code = % UI / 100 − 0,5) ;
- limites ±525 mV partout (l.317), alors que la norme donne ±525 mV dans l'UI qui suit une transition et ±475 mV ailleurs (note l.309-313).

Ces positions viennent **de mémoire** : le commentaire (l.291-302) le dit. Le ±175 mV a été retrouvé (brevet), pas les positions (`A-FAIRE.md:263`).

**Contradiction dans les listes, à lever par ta lecture** : `A-FAIRE.md:263` (et l.52) tient la case ouverte, mais `A-FAIRE.md:398` dit « Template 2 passe "recoupé" (… six points concordants, de mémoire) … Plus aucun gabarit "à vérifier" », et le code l'affiche déjà « recoupé » (`oeil.py:290`, `"fiabilite": "corrobore"` ; `docs/simulation-em.md:910` et `:920` ; essai `python/test/banc-oeil.py:424-427`). Une concordance de mémoire n'est pas un recoupement. Si les positions sont fausses, un œil USB HS est déclaré conforme ou non conforme à tort.

## Ce qu'il te faut
- *Universal Serial Bus Specification, Revision 2.0* (27 avril 2000), dans l'archive « USB 2.0 Specification » de usb.org (fichier `usb_20.pdf`).
- § 7.1.2.2, figure 7-15 (Template 2) et son tableau « Template 2 Transmit Waveform Requirements » ; pour comparaison la figure du Template 1.

## Étapes
1. Ouvre `usb_20.pdf`, § 7.1.2.2. Vérifie que la figure 7-15 est bien le Template 2 (sinon note le bon numéro).
   → à relever : numéro de figure et de tableau, page.
2. Dans le tableau du Template 2, relève chaque point : position (% UI) et tension (mV).
   → attendu (code) : points 1 et 4 à 12,5 % et 87,5 % UI, 0 mV ; points 2, 3, 5, 6 à 35 % et 65 % UI, ±175 mV.
3. Relève les niveaux 1 et 2 (limites hautes et basses) et dans quel cas chacun s'applique.
   → attendu (code) : ±525 mV (niveau 1, UI après transition), ±475 mV (niveau 2).
4. Note à quel point de mesure il s'applique.
   → attendu (code) : TP3 pour un hub, TP2 pour un périphérique à câble captif.
5. Coup d'œil au Template 4 (sensibilité du récepteur, ±150 mV, même forme d'après le code) : même positions ? oui / non.

## C'est réussi si / La réponse
- [ ] Les six points du tableau : % UI et mV, avec page et numéro de figure / tableau.
- [ ] Niveaux 1 et 2 en mV.
- [ ] Points de mesure (TP2 / TP3).
- [ ] Concordance avec le code : oui / non, point par point.

## Après
- Concordant : la source de `python/oeil.py:291-312` passe de « de mémoire » à « relu dans la figure 7-15, page … », je coche `web_cao/A-FAIRE.md:263` et l.52, et je corrige `A-FAIRE.md:398` et `docs/simulation-em.md:920` pour que les trois disent la même chose.
- Différent : je corrige les points de `oeil.py:315-316` (et l'essai `python/test/banc-oeil.py:425-427`), puis mêmes cases.
- Dans les deux cas, je te propose de séparer ±525 / ±475 mV si la norme le confirme (la note l.309-313 dit que le contrôle est indulgent).
