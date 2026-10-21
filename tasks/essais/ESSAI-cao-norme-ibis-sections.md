# Essai : IBIS, sens des sections d'un boîtier (`Len=`, `Fork`) dans la norme

Qui : Claude in Chrome · Où : PC Windows du boulot (navigateur seul) · Durée : 30 min · Source : `web_cao/A-FAIRE.md:265` (rappel l.54)

## Pourquoi
Dans un `[Define Package Model]`, `[Pin Numbers]` peut décrire chaque broche par des sections (« A1 Len=0 L=1.2n / », puis « Len=1.2 L=2.3n C=4.3p / », `Fork` … `Endfork`). Le code lit ces sections **du die vers la broche** (fil de câblage d'abord) et les retourne pour la cascade :
- `python/ibis.py:477-483` (`_ligne_sections`) : « dans l'ordre du fichier, DU DIE VERS LA BROCHE » ;
- `python/ibis.py:550-559` (`abcd_sections`) : `_cascade(freqs, list(sections)[::-1])`, rendu de la broche vers le die ;
- `ibis` 1.5.0 a inversé l'ordre d'avant, sur deux **sources secondaires** seulement (exemple de la norme repris au sommet IBIS de novembre 2014 par KEI Systems, présentation ICM d'Intel) ; « le texte de la norme n'a pas été relu » (l.558).

Si le sens est faux, un boîtier à sections dissymétriques (fil + piste de substrat) met la capacité du mauvais côté : œil et réflexions faux pour tout modèle IBIS à sections. Sans sections, rien ne change.

La spécification IBIS est publique et gratuite sur ibis.org, en PDF lisible dans le navigateur (la dernière ratifiée est la 8.0, décembre 2025 ; la 7.2 reste en ligne) : Claude in Chrome peut la lire et citer la phrase.

## Ce que tu prépares
Rien : Claude in Chrome n'a besoin que du web. Lis son rapport à la fin.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-norme-ibis-sections.md. But : lire dans la spécification IBIS elle-même dans
quel ordre les sections d'une broche sont écrites sous [Pin Numbers] d'un [Define Package Model],
et comment se branche une dérivation Fork / Endfork. Pas de WEB_CAO dans cet essai.
Règle des sources : la seule source PRIMAIRE est le texte de la spécification sur ibis.org ; cite
la version, le chapitre, la page du PDF et la phrase exacte en anglais. Présentations, forums,
docs d'outils = SECONDAIRES.

1. Ce que dit le code. Ouvre https://raw.githubusercontent.com/pilou33620/WEB_CAO/main/python/ibis.py
   et exécute :
   (()=>{const re=/def _ligne_sections|def abcd_sections|\[::-1\]|DIE VERS|Fork|ouverte au bout|Len = 0/;return document.body.innerText.split("\n").map((l,i)=>(i+1)+": "+l).filter(l=>re.test(l)).join("\n")})()
   → à relever : les lignes qui disent le sens supposé (du die vers la broche), l'inversion
     `[::-1]`, la dérivation « ouverte au bout », la section Len=0 « localisée : C à la broche,
     puis R et L en série ».
2. Ouvre https://www.ibis.org/ver8.0/ et le PDF de la spécification (ver8_0.pdf ou le nom que
   la page donne). Va au chapitre du modèle de boîtier : mot-clé [Define Package Model],
   sous-mot-clé [Pin Numbers].
   → à relever : version, numéro de chapitre ou de section, page du PDF.
3. Lis la description des sections (Len=, L=, R=, C=, Fork, Endfork).
   → attendu (code) : la première section d'une broche est côté DIE (pastille de la puce), la
     dernière côté BROCHE du boîtier.
   → à relever : la ou les phrases exactes, en anglais, qui fixent l'ordre ; si le texte ne le
     dit pas explicitement, écris-le, et ce qu'il dit à la place.
4. Dans l'exemple de la norme sous [Pin Numbers], regarde la première section d'une broche
   (souvent « Len=0 L=… », le fil de câblage).
   → à relever : l'exemple la place-t-il côté die ou côté broche (commentaire, schéma, texte) ?
     Recopie l'extrait de l'exemple.
5. Fork / Endfork : la dérivation part-elle du point où l'on en est, dans ce même sens ?
   Est-elle ouverte au bout (rien ne la referme) ?
   → attendu (code) : branche posée en dérivation au point courant, ouverte au bout.
6. Section Len=0 (éléments localisés) : dans quel ordre la norme place-t-elle R, L et C (C côté
   broche ou côté die ; en tête ou en fin de section) ?
   → attendu (code) : C (et G) à la broche, puis R et L en série.
7. Recoupement : ouvre aussi https://www.ibis.org/ver7.2/ (même chapitre). Le texte a-t-il changé
   entre 7.2 et 8.0 sur ces points ? (oui / non, et quoi).

Rapport, en plus du format commun :
- CODE : les lignes relevées à l'étape 1.
- NORME : version, chapitre, page ; la phrase sur l'ordre des sections (citée) ; l'extrait de
  l'exemple ; Fork ouverte au bout oui / non (phrase) ; ordre R, L, C d'une section Len=0 (phrase).
- VERDICT : sens des sections die → broche ou broche → die ; concordance avec le code point par
  point (ordre, Fork, Len=0).
- SOURCES : URL de chaque PDF lu, PRIMAIRE / SECONDAIRE.
```

## C'est réussi si
- [ ] Version d'IBIS, section, page.
- [ ] Sens des sections : die → broche, ou broche → die, avec la phrase de la norme (ou « la norme ne le dit pas explicitement », et ce que montre son exemple).
- [ ] Dérivation `Fork` : ouverte au bout, oui / non.
- [ ] Ordre R, L, C d'une section `Len=0`.
- [ ] Concordance avec le code, point par point.

## Ce qui reste à toi
Rien d'obligatoire. Si le verdict dit « broche → die » ou si la norme est muette, jette un œil à la phrase citée (page donnée) avant que je retourne le code.

## Après
Pour Claude Code, avec le rapport :
- Die → broche :
  - [ ] remplacer « le texte de la norme n'a pas été relu » par la référence exacte (version, section, page) dans `web_cao/python/ibis.py:477-483` et `:555-558` ;
  - [ ] corriger la dernière phrase de `web_cao/A-FAIRE.md:265` (déjà [x]) et cocher la mention l.54.
- Broche → die :
  - [ ] retirer l'inversion (`[::-1]` de `abcd_sections`, `web_cao/python/ibis.py:559`) ;
  - [ ] corriger les docstrings l.477-483 et l.975-981, les essais `web_cao/python/test/banc-oeil.py:1911-1950` (boîtier par sections, `abcd_sections`) ;
  - [ ] passer `ibis` en 1.6.0, puis mêmes cases.
- Fork ou Len=0 différents du code : corriger `_cascade` (`web_cao/python/ibis.py:562-575`) avec un essai dans `banc-oeil.py`.
