# Essai : USB 2.0 Template 2, positions en % UI dans la figure 7-15

Qui : les deux (toi : déposer le PDF officiel, 5 min ; Claude in Chrome : la lecture) · Où : PC Windows du boulot · Durée : 5 min pour toi, 25 min pour Claude in Chrome · Source : `web_cao/A-FAIRE.md:263` (rappel l.52)

## Pourquoi
Le gabarit « USB 2.0 HS — extrémité de câble (Template 2/4) » de l'œil est dans `python/oeil.py:286-320` :
- hexagone (l.315-316), en UI comptés depuis le centre de l'œil : `[-0.375, 0]`, `[-0.15, ±0.175]`, `[0.15, ±0.175]`, `[0.375, 0]` — soit 0 mV à **12,5 et 87,5 % UI**, ±175 mV de **35 à 65 % UI** (position du code = % UI / 100 − 0,5) ;
- limites ±525 mV partout (l.317), alors que la norme donne ±525 mV dans l'UI qui suit une transition et ±475 mV ailleurs (note l.309-313).

Ces positions viennent **de mémoire** : le commentaire (l.291-302) le dit. Le ±175 mV a été retrouvé (brevet), pas les positions (`A-FAIRE.md:263`).

**Contradiction dans les listes, à lever par la lecture** : `A-FAIRE.md:263` (et l.52) tient la case ouverte, mais `A-FAIRE.md:398` dit « Template 2 passe "recoupé" (… six points concordants, de mémoire) … », et le code l'affiche déjà « recoupé » (`oeil.py:290`, `"fiabilite": "corrobore"` ; `docs/simulation-em.md:910` et `:920` ; essai `python/test/banc-oeil.py:424-427`). Une concordance de mémoire n'est pas un recoupement. Si les positions sont fausses, un œil USB HS est déclaré conforme ou non conforme à tort.

**Où lire la norme dans le navigateur.** La spécification USB 2.0 est gratuite, mais usb.org la livre dans une archive zip (« USB 2.0 Specification », `usb_20_<date>.zip`, qui contient `usb_20.pdf`) : Claude in Chrome ne peut pas ouvrir un zip téléchargé. Des copies de `usb_20.pdf` s'ouvrent directement sur le web (par exemple `https://www.pjrc.com/tmp/usb20.pdf`, repérée par recherche, non ouverte ici), mais ce ne sont pas des copies officielles. D'où ton geste de 5 minutes : mettre la copie officielle là où WEB_CAO la sert.

## Ce que tu prépares
1. Sur usb.org, « Document Library » (`https://www.usb.org/documents`), cherche « USB 2.0 Specification » et télécharge le zip.
2. Ouvre-le avec l'explorateur Windows et copie `usb_20.pdf` dans `<dossier de WEB·SUITE>\WEB_CAO\IPC2581_Exemple\` (le dossier `WEB_CAO` est à côté de `demarrer_WEB_SUITE.cmd` ; crée `IPC2581_Exemple` s'il manque ; il est ignoré par git, rien ne part sur GitHub).
3. Vérifie que `http://127.0.0.1:8000/IPC2581_Exemple/usb_20.pdf` s'ouvre dans Chrome (WEB·SUITE lancé). Note le nom du zip (sa date) pour le rapport.
4. Si tu sautes ce geste, Claude in Chrome lira une copie trouvée sur le web et le dira : la lecture vaudra alors « à confirmer sur la copie officielle ».

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-norme-usb2-template2.md. But : relever dans la spécification USB 2.0 les six
points et les deux niveaux du Template 2 (œil à l'extrémité du câble) et les comparer au code.
Règle des sources : cite l'URL, la page du PDF, le numéro de figure et de tableau ; la source
PRIMAIRE est usb_20.pdf de usb.org.

1. Ce que dit le code. Ouvre https://raw.githubusercontent.com/pilou33620/WEB_CAO/main/python/oeil.py
   et exécute :
   (()=>{const L=document.body.innerText.split("\n");const i=L.findIndex(l=>/usb2-hs-recepteur/.test(l));return L.slice(i,i+36).map((l,k)=>(i+k+1)+": "+l).join("\n")})()
   → à relever : les six points du masque (attendu [-0.375,0], [-0.15,0.175], [0.15,0.175],
     [0.375,0], [0.15,-0.175], [-0.15,-0.175], en UI depuis le centre de l'œil, en volts) et
     v_max / v_min (attendu ±0.525). Position du code = % UI / 100 − 0,5.
2. Ouvre http://127.0.0.1:8000/IPC2581_Exemple/usb_20.pdf (copie officielle déposée par
   l'utilisateur). Si la page répond 404 : cherche sur le web une copie de usb_20.pdf qui s'ouvre
   dans le navigateur (titre « Universal Serial Bus Specification Revision 2.0 », 27 avril 2000),
   note son URL et écris « COPIE NON OFFICIELLE » dans le rapport.
   → à relever : la page de titre (révision, date), le nombre de pages.
3. Va au § 7.1.2.2 (modèles d'œil, « eye pattern templates »). Vérifie que la figure 7-15 est
   bien le Template 2 (sinon note le bon numéro).
   → à relever : numéro de figure et de tableau, page.
4. Dans le tableau du Template 2 (« Template 2 Transmit Waveform Requirements » ou proche),
   relève chaque point : position (% UI) et tension (mV).
   → attendu (code) : points 1 et 4 à 12,5 % et 87,5 % UI, 0 mV ; points 2, 3, 5, 6 à 35 % et
     65 % UI, ±175 mV.
5. Relève les niveaux 1 et 2 (limites hautes et basses) et dans quel cas chacun s'applique.
   → attendu (code) : ±525 mV (niveau 1, UI qui suit une transition), ±475 mV (niveau 2).
6. Note à quel point de mesure il s'applique.
   → attendu (code) : TP3 pour un hub, TP2 pour un périphérique à câble captif.
7. Template 4 (sensibilité du récepteur, ±150 mV, même forme d'après le code) : mêmes positions ?
   oui / non (figure, tableau, page).

Rapport, en plus du format commun :
- SOURCE : URL lue, officielle (déposée) ou COPIE NON OFFICIELLE, révision et date de la page de titre.
- CODE : les six points et les limites relevés à l'étape 1.
- NORME : figure, tableau, page ; les six points (% UI, mV) ; niveaux 1 et 2 et leur cas ; points
  de mesure ; Template 4.
- CONCORDANCE : point par point, oui / non (et l'écart).
```

## C'est réussi si
- [ ] Les six points du tableau : % UI et mV, avec page et numéro de figure / tableau.
- [ ] Niveaux 1 et 2 en mV, et leur cas.
- [ ] Points de mesure (TP2 / TP3).
- [ ] Concordance avec le code : oui / non, point par point.
- [ ] Lu sur la copie officielle (sinon, la case reste « à confirmer »).

## Ce qui reste à toi
- Déposer `usb_20.pdf` (ci-dessus) : c'est ce qui fait de la lecture une lecture de la source primaire.
- Si Claude in Chrome a dû lire une copie non officielle : ouvre ta copie officielle à la page qu'il cite et confirme les six points d'un coup d'œil.

## Après
Pour Claude Code, avec le rapport :
- Concordant (sur la copie officielle) :
  - [ ] la source de `web_cao/python/oeil.py:291-312` passe de « de mémoire » à « relu dans la figure 7-15, page … » ;
  - [ ] cocher `web_cao/A-FAIRE.md:263` et l.52 ;
  - [ ] corriger `web_cao/A-FAIRE.md:398` et `web_cao/docs/simulation-em.md:920` pour que les trois disent la même chose.
- Différent :
  - [ ] corriger les points de `web_cao/python/oeil.py:315-316` et l'essai `web_cao/python/test/banc-oeil.py:425-427`, puis mêmes cases.
- Dans les deux cas : proposer de séparer ±525 / ±475 mV si la norme le confirme (la note `oeil.py:309-313` dit que le contrôle est indulgent).
