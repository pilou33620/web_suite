# Essai : IBIS, sens des sections d'un boîtier (`Len=`, `Fork`) dans la norme

Où : PC avec la spécification IBIS (ibis.org, gratuite) · Durée : 20 min · Source : `web_cao/A-FAIRE.md:265` (rappel l.54)

## Pourquoi
Dans un `[Define Package Model]`, `[Pin Numbers]` peut décrire chaque broche par des sections (« A1 Len=0 L=1.2n / », puis « Len=1.2 L=2.3n C=4.3p / », `Fork` … `Endfork`). Le code lit ces sections **du die vers la broche** (fil de câblage d'abord) et les retourne pour la cascade :
- `python/ibis.py:479-482` (`_ligne_sections`) : « dans l'ordre du fichier, DU DIE VERS LA BROCHE » ;
- `python/ibis.py:551-559` (`abcd_sections`) : `_cascade(freqs, list(sections)[::-1])`, rendu de la broche vers le die ;
- `ibis` 1.5.0 a inversé l'ordre d'avant, sur deux **sources secondaires** seulement (exemple de la norme repris au sommet IBIS de novembre 2014 par KEI Systems, présentation ICM d'Intel) ; « le texte de la norme n'a pas été lu » (l.558).

Si le sens est faux, un boîtier à sections dissymétriques (fil + piste de substrat) met la capacité du mauvais côté : œil et réflexions faux pour tout modèle IBIS à sections. Sans sections, rien ne change.

## Ce qu'il te faut
- IBIS (I/O Buffer Information Specification), la dernière version publiée sur ibis.org (7.x ; note laquelle).
- Le chapitre « Package Modeling » : mot-clé `[Define Package Model]`, sous-mot-clé `[Pin Numbers]` (description des sections `Len=`, `L=`, `R=`, `C=`, `Fork`, `Endfork`) et son exemple.

## Étapes
1. Ouvre la spécification au chapitre du modèle de boîtier, `[Pin Numbers]`.
   → à relever : version d'IBIS, numéro de section et page.
2. Lis la phrase qui dit dans quel ordre les sections d'une broche sont écrites.
   → attendu (code) : la première section est côté **die** (pastille de la puce), la dernière côté **broche** du boîtier.
   → à relever : la phrase exacte, en anglais.
3. Dans l'exemple de la norme, regarde la première section d'une broche (souvent `Len=0 L=…`, le fil de câblage).
   → à relever : la norme la place-t-elle côté die ou côté broche ?
4. Pour `Fork` / `Endfork` : la dérivation part-elle du point où l'on en est dans ce même sens ? Est-elle ouverte au bout ?
   → attendu (code, `ibis.py:573`) : la branche est ouverte au bout.

## C'est réussi si / La réponse
- [ ] Version d'IBIS, section, page.
- [ ] Sens des sections : die → broche, ou broche → die, avec la phrase de la norme.
- [ ] Dérivation `Fork` : ouverte au bout, oui / non.

## Après
- Die → broche : je remplace « le texte de la norme n'a pas été lu » par la référence exacte dans `python/ibis.py:481-482` et `:555-558`, et je coche `web_cao/A-FAIRE.md:265` (déjà [x] : je corrige sa dernière phrase) et la mention l.54.
- Broche → die : je retire l'inversion (`[::-1]` de `abcd_sections`, `ibis.py:559`), je corrige les docstrings l.479-482 et l.978-981, les essais de `python/test/banc-oeil.py:1911-1950` (boîtier par sections, `abcd_sections`), et je passe `ibis` en 1.6.0.
