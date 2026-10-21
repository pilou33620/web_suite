# Essai : LED RVB KRTBLSLPS1, sens de l'empreinte et couleurs

Où : atelier, avec une carte qui porte la LED RVB (assemblée) et une alimentation de labo · Durée : 20 min · Source : `web_cao/A-FAIRE.md:288` (rappel l.47 ; empreinte décrite l.287)

## Pourquoi
L'empreinte de la LED RVB KRTBLSLPS1 suit les pastilles OSRAM, mais **la vue (dessus ou dessous) est déduite** du schéma « Component Location on Pad » de la datasheet. Les numéros aussi sont choisis faute d'en avoir : 1–3 = anodes A1–A3, 4–6 = cathodes C1–C3, et le brochage de la LIB range ces pattes en trois parties R, V, B (`A-FAIRE.md:287`). Si la vue est à l'envers, la LED est montée en miroir : couleurs inversées, ou LED qui ne s'allume pas (anode et cathode échangées).

Ce n'est pas le même geste que le SOT-89-5 (`ESSAI-cao-sot89-5.md`) : ici on allume la LED, on ne mesure pas des pastilles.

## Ce qu'il te faut
- Une carte assemblée avec la KRTBLSLPS1 sur cette empreinte (ou la LED en vrac et une carte nue).
- Alimentation de labo limitée à 5 mA (ou 5 V + résistance de 1 kΩ), deux pointes fines.
- La Gestion LIB : ligne KRTBLSLPS1 de `LIB_composants.csv` (colonne `Brochage`) et son empreinte.
- La datasheet OSRAM de la LED (repère de polarité, figure « Component Location on Pad »).

## Étapes
1. Dans la Gestion LIB, recopie la colonne `Brochage` de KRTBLSLPS1 (forme `R:A=…,K=…|V:…|B:…`).
   → à relever : pour chaque couleur, le numéro de pastille d'anode et de cathode.
2. Sur la carte, repère la pastille 1 (sérigraphie) et le repère de polarité de la LED elle-même (chanfrein, coin coupé, marque).
   → à relever : la marque de la LED est-elle du côté de la pastille 1, comme le dessin de la datasheet en vue de dessus ?
3. Pour chaque couleur de l'étape 1 : + sur la pastille d'anode, − sur la pastille de cathode, 2 à 5 mA.
   → à relever : la couleur qui s'allume (rouge / vert / bleu / rien).
4. Si une couleur ne s'allume pas, inverse les pointes sur la même paire.
   → à relever : s'allume-t-elle en inverse ? (ça veut dire anode et cathode échangées, donc vue en miroir.)
5. Si la LED est en vrac, fais les étapes 3 et 4 directement sur ses pattes, posée à l'endroit sur l'empreinte imprimée.

## C'est réussi si / La réponse
- [ ] Chaque paire du brochage allume la couleur annoncée (R, V, B), à l'endroit.
- [ ] Sinon : pour chaque paire, la couleur vue et le sens qui allume.
- [ ] Le repère de polarité de la LED tombe du côté de la pastille 1 : oui / non.
- [ ] Une photo de la LED montée avec la sérigraphie.

## Après
- Si tout s'allume comme prévu : je coche la moitié LED RVB de `web_cao/A-FAIRE.md:288` et de la mention l.47.
- Si la vue est en miroir : je retourne l'empreinte (pastilles au miroir, numérotation gardée) dans la LIB, ou je corrige la colonne `Brochage` si ce n'est qu'un mauvais appariement couleur / pattes ; je te dis quelles cartes la portent.
