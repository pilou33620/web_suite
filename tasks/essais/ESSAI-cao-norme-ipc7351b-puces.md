# Essai : IPC-7351B, constantes de soudure des puces (tableaux 3-5 et 3-6)

Où : PC avec la norme IPC-7351B (payante, IPC) · Durée : 20 min · Source : `web_cao/A-FAIRE.md:116` et l.54

## Pourquoi
Le générateur d'empreintes (Gestion LIB et `lib_empreinte_generer`) prend ses constantes dans `commun/composants-lib.js:291-305` :
- puces ≥ 1608 (0603), « tableau 3-5 » (l.298) : pointe Jt M/N/L = 0,55 / 0,35 / 0,15 ; talon Jh = 0 / 0 / 0 ; côté Js = 0,05 / 0 / −0,05 ;
- puces < 1608, « tableau 3-6 » (l.300) : Jt = 0,30 / 0,20 / 0,10 ; Jh = 0 / 0 / 0 ; Js = 0,05 / 0 / −0,05 ;
- F (fabrication) = 0,10, P (placement) = 0,05 (l.291) ; pas d'arrondi au 0,05 (l.289-290).

Elles viennent du générateur d'empreintes de KiCad (`kilibs/config/ipc_configs/ipc7351B_2terminal.yaml`) : « la norme elle-même n'a pas été lue » (l.294-296). Le talon est passé de −0,05 (IPC-7351 A) à 0. Une constante fausse change la longueur de toutes les pastilles de puces générées : ménisque trop court, effet « tombstone » sur les 0201 / 0402.

## Ce qu'il te faut
- IPC-7351B, *Generic Requirements for Surface Mount Design and Land Pattern Standard* (juin 2010).
- Ses tableaux des puces rectangulaires et carrées : 1608 et plus (3-5 d'après le code), moins de 1608 (3-6 d'après le code) ; et le paragraphe des tolérances F et P.

## Étapes
1. Ouvre le tableau des puces de 1608 et plus.
   → à relever : numéro du tableau ; Toe, Heel, Side pour Most / Nominal / Least (mm) ; l'excédent de courtyard et l'arrondi s'ils y sont.
   → attendu (code) : Toe 0,55 / 0,35 / 0,15 ; Heel 0 / 0 / 0 ; Side 0,05 / 0 / −0,05.
2. Même chose pour le tableau des puces de moins de 1608.
   → attendu (code) : Toe 0,30 / 0,20 / 0,10 ; Heel 0 / 0 / 0 ; Side 0,05 / 0 / −0,05.
3. Relève F et P par défaut dans le texte des équations de pastille.
   → attendu (code) : F = 0,10 mm, P = 0,05 mm.
4. Note si la norme impose l'arrondi des cotes au 0,05 mm.

## C'est réussi si / La réponse
- [ ] Tableau ≥ 1608 : numéro, et les 9 valeurs.
- [ ] Tableau < 1608 : numéro, et les 9 valeurs.
- [ ] F et P.
- [ ] Arrondi imposé : oui (pas) / non.
- [ ] Concordance avec le code : oui / non, valeur par valeur.

## Après
- Concordant : je remplace « la norme elle-même n'a pas été lue » par les numéros de tableau dans `commun/composants-lib.js:294-300`, et je coche la mention de `web_cao/A-FAIRE.md:54`.
- Différent : je corrige `SOUDURE.puce` / `SOUDURE.pucePetite` (l.298-300), je rejoue `banc-composants.js` (les écarts aux empreintes KiCad de `A-FAIRE.md:116` changeront et je les réécris), et je te liste les empreintes de puces de la LIB à régénérer.
