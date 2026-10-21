# Essai : rugosité de Huray, a et SR depuis des fiches de feuillard (méthode « cannonball »)

Qui : Claude in Chrome (relecture des calculs par toi) · Où : PC Windows du boulot (navigateur seul) · Durée : 1 h · Source : `web_cao/A-FAIRE.md:266` (rappels l.53, l.371)

## Pourquoi
Les deux jeux de Huray de l'éditeur sont `editeur-pcb/js/01-core.js:720-721` :
- « Huray · ED standard » : a = 0,5 µm, SR = 1,5 ;
- « Huray · HVLP » : a = 0,3 µm, SR = 0,6.

Le commentaire l.709-712 dit seulement qu'ils donnent ~70 % et ~15 % de pertes cuivre en plus à 10 GHz, « ce que les fabricants annoncent ». Aucune fiche ne donne a et SR (elles donnent Rz / Ra) ; la méthode « cannonball » de Simonovich les tire de Rz, mais son texte n'avait pas pu être lu (`A-FAIRE.md:266`). Le modèle du code (`python/ligne_mom.py:2131-2136`) : K = 1 + (3/2)·SR / (1 + δ/a + δ²/(2a²)), avec SR = N·4πa² / A_plat. Si a ou SR est faux, la perte d'une piste rapide est fausse d'autant (jusqu'à ×1,7 sur α_c au-dessus de quelques GHz), et l'œil ou le S21 avec.

Tout est public et se lit dans le navigateur : le papier de Simonovich (une copie est hébergée par Polar Instruments), son brevet, les fiches des fabricants de feuillard. Claude in Chrome lit, recopie les formules, calcule ; tu relis.

## Ce que tu prépares
Rien : Claude in Chrome n'a besoin que du web. Lis son rapport à la fin.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-huray.md. But : tirer le rayon a et le facteur SR du modèle de Huray à partir
de la rugosité Rz de deux feuillards réels (un ED standard, un HVLP), par la méthode
« cannonball » de Bert Simonovich, et comparer aux deux jeux de WEB_CAO (ED standard : a 0,5 µm,
SR 1,5 ; HVLP : a 0,3 µm, SR 0,6). Pas de WEB_CAO dans cet essai : seulement le web.
Règle des sources : cite chaque chiffre avec l'URL exacte, la page, l'équation, la figure ou le
tableau ; dis si la source est PRIMAIRE (l'auteur de la méthode, le fabricant du feuillard) ou
SECONDAIRE (article, blog, application note d'un éditeur de logiciel qui la reprend).

1. Le code, pour savoir ce qu'on compare. Ouvre
   https://raw.githubusercontent.com/pilou33620/WEB_CAO/main/python/ligne_mom.py
   et exécute :
   (()=>{const re=/huray|cannon|\bSR\b/i;return document.body.innerText.split("\n").map((l,i)=>(i+1)+": "+l).filter(l=>re.test(l)).join("\n")})()
   → à relever : la formule de K et la définition de SR telles que le code les écrit (attendu :
     K = 1 + (3/2)·SR / (1 + δ/a + δ²/(2a²)), SR = N·4πa² / A_plat, vers les lignes 2131-2136).
2. Le papier : Lambert (Bert) Simonovich, « Practical Method for Modeling Conductor Surface
   Roughness Using The Cannonball Stack Principle » (Lamsim Enterprises, 2015, issue 2.0).
   Copie hébergée : https://www.polarinstruments.com/support/si/AP8203.pdf (vérifie le titre,
   l'auteur, la date ; une copie chez Polar reste le texte de l'auteur, dis-le). Cherche aussi
   le site de l'auteur (Lamsim Enterprises) et, s'il existe, la version DesignCon.
   → à relever : les formules qui donnent le rayon de sphère r (= a) à partir de Rz, la taille de
     la tuile de base (A_plat), le nombre de sphères N par tuile, et SR = N·4πr²/A_plat (ou la
     forme qu'il emploie) ; numéros d'équation et pages ; l'exemple chiffré s'il y en a un.
   → compare sa définition de SR à celle du code (même chose, ou facteur différent ?).
3. Source primaire de la méthode, en recoupement : le brevet US 8 527 246 « Method for modeling
   conductor surface roughness » (https://patents.google.com/patent/US8527246 ou le PDF de
   l'USPTO). → les mêmes formules ? (oui / non, revendication ou paragraphe).
   L'application note Polar AP8195 (https://www.polarinstruments.com/support/si/AP8195.pdf) est
   SECONDAIRE : lis-la pour vérifier, ne t'en sers pas comme source des formules.
4. Deux fiches de feuillard du fabricant, avec Rz : un ED standard (STD) et un HVLP (par exemple
   Mitsui Kinzoku, Circuit Foil, Furukawa Electric, JX Advanced Metals, Fukuda ; ou la fiche d'un
   stratifié qui nomme son cuivre et donne Rz).
   → à relever pour chacun : fabricant, référence du feuillard, date ou révision de la fiche, URL,
     Rz en µm de la face traitée (« matte / treated side », celle qui touche le diélectrique) et de
     la face tambour (« shiny / drum side ») si elle est donnée, la méthode de mesure si dite
     (profilomètre, ISO 4287, JIS B 0601). Si seule Ra est donnée, note-le et prends un autre feuillard.
5. Applique les formules de l'étape 2 aux Rz de l'étape 4 (face traitée ; et face tambour pour un
   cuivre « reverse treated » si tu en trouves un).
   → à relever : a (µm) et SR pour chaque feuillard, avec le détail du calcul.
6. Si le papier donne un exemple chiffré, refais-le avec tes formules.
   → attendu : le même a et le même SR que le papier (sinon tes formules sont mal recopiées).
7. Compare aux jeux de WEB_CAO : STD (0,5 µm ; 1,5) et HVLP (0,3 µm ; 0,6). Écart en % sur a et
   sur SR. Si tu peux, calcule K à 10 GHz avec la formule du code (δ du cuivre recuit à 10 GHz,
   ≈ 0,66 µm ; dis la valeur prise) pour tes valeurs et pour celles de WEB_CAO.

Rapport, en plus du format commun :
- FORMULES : recopiées, avec référence (document, page, équation), et la définition de SR face à
  celle du code.
- FEUILLARDS : un tableau fabricant / référence / fiche (URL, date) / face / Rz (µm).
- CALCUL : a et SR de chaque feuillard, l'exemple du papier refait, K à 10 GHz.
- ÉCARTS : aux jeux actuels (0,5 / 1,5 et 0,3 / 0,6), en %.
- SOURCES : chacune classée PRIMAIRE / SECONDAIRE.
```

## C'est réussi si
- [ ] Les formules (avec leur référence primaire) et la définition de SR, comparée à celle du code.
- [ ] Rz des deux feuillards, avec la fiche (fabricant, référence, date, URL).
- [ ] a et SR calculés : STD → a = … µm, SR = … ; HVLP → a = … µm, SR = … ; l'exemple du papier retrouvé.
- [ ] Écart aux jeux actuels (0,5 / 1,5 et 0,3 / 0,6).
- [ ] Ta relecture des calculs (ci-dessous).

## Ce qui reste à toi
- Relire le calcul (une erreur de recopie de formule fausse tout) : refais au moins l'exemple du papier.
- Dire si les deux feuillards choisis sont représentatifs de ce que tes fabricants de cartes emploient (sinon donne-moi le nom du feuillard ou du stratifié, et je redonne l'essai).

## Après
Pour Claude Code, avec le rapport et ta relecture :
- [ ] remplacer les valeurs de `web_cao/editeur-pcb/js/01-core.js:720-721` par les a et SR calculés, et citer la fiche et le papier dans le commentaire l.709-712 ;
- [ ] si la définition de SR diffère : corriger `web_cao/python/ligne_mom.py:2131-2136` et son banc `web_cao/python/test/banc-ligne-mom.py` ;
- [ ] recalculer les K à 10 GHz pour la note des jeux ;
- [ ] cocher `web_cao/A-FAIRE.md:266`, la mention l.53 et la ligne l.371.
