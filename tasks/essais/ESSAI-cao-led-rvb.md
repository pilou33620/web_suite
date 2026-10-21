# Essai : LED RVB KRTBLSLPS1, sens de l'empreinte et couleurs

Qui : les deux (Claude in Chrome : la datasheet face à la LIB ; toi : allumer la LED) · Où : PC Windows du boulot, puis l'atelier · Durée : 20 min pour Claude in Chrome, 20 min pour toi · Source : `web_cao/A-FAIRE.md:288` (rappel l.47 ; empreinte décrite l.287)

## Pourquoi
L'empreinte de la LED RVB KRTBLSLPS1 suit les pastilles OSRAM, mais **la vue (dessus ou dessous) est déduite** du schéma « Component Location on Pad » de la datasheet. Les numéros aussi sont choisis faute d'en avoir : 1–3 = anodes A1–A3, 4–6 = cathodes C1–C3, et le brochage de la LIB range ces pattes en trois parties R, V, B (`A-FAIRE.md:287`). Si la vue est à l'envers, la LED est montée en miroir : couleurs inversées, ou LED qui ne s'allume pas (anode et cathode échangées).

Claude in Chrome relit la datasheet et la confronte à l'empreinte et au brochage de la LIB : il dit si la déduction tient sur le papier. L'allumage reste à toi : c'est la seule preuve.

Ce n'est pas le même geste que le SOT-89-5 (`ESSAI-cao-sot89-5.md`) : ici on allume la LED, on ne mesure pas des pastilles.

## Ce que tu prépares
- WEB·SUITE lancé sur le PC (WEB_CAO sur `http://127.0.0.1:8000`), Claude in Chrome ouvert.
- Pour ta moitié : une carte assemblée avec la KRTBLSLPS1 sur cette empreinte (ou la LED en vrac et une carte nue), une alimentation de labo limitée à 5 mA (ou 5 V + 1 kΩ), deux pointes fines.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-led-rvb.md. But : dire si l'empreinte et le brochage de la LED RVB KRTBLSLPS1
dans la LIB suivent la datasheet (vue de dessus, numéros, couleurs). Tu ne modifies rien.

1. Ouvre http://127.0.0.1:8000/gestion-lib/gestion-lib.html, onglet « 📋 Catalogue Composants
   (CSV) », puis exécute :
   JSON.stringify(LIB_STATE.composants.filter(c=>/KRTBLSLPS1/i.test(JSON.stringify(c))),null,1)
   → à relever : Part Name, Manufacturer, la référence fabricant exacte, Empreinte PCB, et la
     colonne Brochage (forme R:A=…,K=…|V:…|B:…) : pour chaque couleur, le n° de pastille d'anode
     et de cathode.
   Tape KRTBLSLPS1 dans la recherche du catalogue et clique UNE fois la ligne (un double-clic
   ouvrirait l'éditeur d'empreinte) : capture d'écran de l'inspecteur à droite (aperçu du
   brochage et de l'empreinte).
2. Onglet neuf, lecture seule, avec le nom de fichier de la colonne Empreinte PCB (ex. NOM.json) :
   http://127.0.0.1:8000/api/lib/fichier?type=pcb&nom=NOM.json
   puis :
   (()=>{const f=JSON.parse(document.body.innerText).data;return JSON.stringify({body:f.body,pads:(f.pads||[]).map(q=>({n:q.n,x:q.x,y:q.y,w:q.w,h:q.h,rot:q.rot||0})),lines:(f.lines||[]).length},null,1)})()
   → à relever : la place de chaque pastille 1 à 6 (X, Y en mm, Y VERS LE BAS, vue de dessus) ;
     un dessin en texte de la disposition (ex. « 1 2 3 en haut de gauche à droite, 6 5 4 en bas »).
3. Sur le web, trouve la datasheet du fabricant pour cette référence exacte (la colonne
   Manufacturer dit lequel ; A-FAIRE parle d'OSRAM / ams OSRAM) et lis-la dans le navigateur.
   → à relever, avec page et figure : l'affectation des pattes (quelle patte est l'anode ou la
     cathode de quelle couleur) ; le repère de polarité du boîtier (chanfrein, coin coupé, marque) ;
     la figure « Component Location on Pad » ou « Recommended Solder Pad » : est-elle dessinée vue
     de DESSUS ou de DESSOUS (le texte, une légende « top view » / « bottom view », la place de la
     marque de polarité le disent) ; les cotes des pastilles recommandées.
   Source PRIMAIRE = le PDF du fabricant ; tout le reste (distributeur, empreinte tierce) SECONDAIRE.
4. Compare :
   → les pastilles de la LIB ont-elles la disposition et les cotes de la figure, vue de dessus ?
     Sinon : miroir gauche-droite, haut-bas, ou rotation ?
   → pour chaque couleur, la paire anode / cathode du Brochage est-elle celle de la datasheet ?
   → le repère de polarité du boîtier tombe-t-il du côté de la pastille 1 ?

Rapport, en plus du format commun :
- LIB : la ligne du catalogue, le Brochage, le tableau des 6 pastilles et le dessin en texte.
- DATASHEET : URL, référence et révision du document, page et figure de chaque chose relevée.
- VERDICT DOCUMENTAIRE : vue de dessus confirmée / miroir (lequel) / la datasheet ne permet pas de
  trancher (pourquoi) ; paires couleur par couleur : justes / à corriger (lesquelles).
```

## C'est réussi si
- [ ] Le Brochage et les 6 pastilles de la LIB relevés.
- [ ] La datasheet lue (URL, page, figure) : vue de la figure des pastilles, affectation des pattes, repère de polarité.
- [ ] Verdict documentaire.
- [ ] Ton allumage (ci-dessous) : chaque paire du brochage allume la couleur annoncée, à l'endroit (sinon la couleur vue et le sens qui allume).
- [ ] Une photo de la LED montée avec la sérigraphie.

## Ce qui reste à toi
Avec le Brochage du rapport (pour chaque couleur, pastille d'anode et de cathode) :
1. Sur la carte, repère la pastille 1 (sérigraphie) et la marque de polarité de la LED.
   → à relever : la marque est-elle du côté de la pastille 1 ?
2. Pour chaque couleur : + sur la pastille d'anode, − sur la pastille de cathode, 2 à 5 mA.
   → à relever : la couleur qui s'allume (rouge / vert / bleu / rien).
3. Si une couleur ne s'allume pas, inverse les pointes sur la même paire.
   → à relever : s'allume-t-elle en inverse ? (anode et cathode échangées : vue en miroir.)
4. LED en vrac : mêmes gestes sur ses pattes, posée à l'endroit sur l'empreinte imprimée.

## Après
Pour Claude Code, avec le rapport et ta réponse :
- Tout s'allume comme prévu :
  - [ ] cocher la moitié LED RVB de `web_cao/A-FAIRE.md:288` et de la mention `web_cao/A-FAIRE.md:47` ;
  - [ ] citer la figure de la datasheet dans la ligne l.287 (la vue n'est plus « déduite »).
- Vue en miroir :
  - [ ] retourner l'empreinte de la LIB (pastilles au miroir, numérotation gardée) ;
  - [ ] lister les cartes qui la portent.
- Mauvais appariement couleur / pattes seulement :
  - [ ] corriger la colonne `Brochage` de la ligne KRTBLSLPS1 de `LIB_composants.csv`.
