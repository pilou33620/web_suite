# Simulation : régler depuis une datasheet, lancer, lire

Les simulations vivent dans l'éditeur PCB et la visionneuse IPC-2581 ouverts.

## Quelle analyse pour quelle question

| Question | `cao_calcul` |
|---|---|
| Impédance d'une piste, tronçon par tronçon | (analyse=impedance) |
| Impédance différentielle et mode commun d'une paire | (analyse=diff) |
| Par où revient le courant d'une via, inductance de boucle | (analyse=retour) |
| Bruit qu'une victime reçoit de ses voisines | (analyse=crosstalk) |
| Un lien série rapide vu du récepteur (œil, gabarit USB, PCIe, HDMI, LVDS, MIPI) | (analyse=oeil) |
| Setup / hold d'un bus synchrone (SPI, I²C, parallèle), temps de vol, skews | (analyse=bus) |
| Chute de tension continue (IR drop), échauffement | (analyse=dc) |
| Impédance du réseau d'alimentation Z(ω), découplage, impédance cible | (analyse=pdn) |
| Transmission entre deux ports (puce → connecteur ou antenne) | (analyse=s21) |
| Audit de toute la carte avant fabrication | (analyse=verif) |

Le bus prend les nets désignés dans l'onglet « Bus synchrone » (sinon les nets sélectionnés,
ou l'horloge trouvée par son nom) ; ses paramètres se règlent par le groupe bus.

Chaque calcul prend les réglages du panneau et la sélection de l'éditeur : cadrer ou
surligner d'abord ce qu'il faut (`cao_vue`).

## Régler depuis une datasheet

Claude Code lit le PDF lui-même. Ensuite :

1. Lire le catalogue : `cao_sim_lire` (quoi=catalogue). Il donne chaque paramètre réglable
   (groupe.clé), son unité ou ses choix, la valeur actuelle et où la trouver sur une
   datasheet, et ses bornes : une valeur hors bornes est refusée par `cao_sim_ecrire`.
2. Ne relever que des valeurs **écrites** dans le document. Une valeur absente ne s'invente
   pas et ne se tire pas d'une « valeur usuelle » : la dire manquante (le nombre de sorties
   qui basculent ensemble, par exemple, dépend du firmware).
3. **Pire cas** : courants et temps de propagation (IDD, tco) à leur valeur max ; temps de
   montée à leur valeur min (le front le plus raide) ; tsu / th : le min exigé par le
   récepteur. Prendre les conditions les plus proches du projet (tension, horloge,
   température).
4. Pour chaque valeur : la page et une citation courte et exacte du tableau ou de la phrase,
   et une confiance (haute, moyenne, basse).
5. Convertir dans l'unité **exacte** du catalogue : 12,5 mA pour pdn_fiche.iActifMa
   (0,0125 A serait faux). `cao_sim_ecrire` refuse une autre unité, sans conversion.
6. Viser la bonne cible : la fiche pdn_fiche décrit la charge du rail (le composant que le
   rail alimente) ; une datasheet de régulateur règle pdn.rVrmMOhm et pdn.fVrmKhz, pas la
   fiche.
7. Montrer d'abord un tableau court (paramètre, valeur, unité, page, citation, confiance),
   puis proposer le lot par `cao_sim_ecrire` : une carte, Annuler sur la carte la défait.

Les parasites des condensateurs (ESR, ESL, capacité effective sous la tension du rail) et les
bornes de la chute DC (courant total d'une charge, tension d'une source) s'écrivent aussi par
`cao_sim_ecrire`, avec une « cible » : groupe condensateurs, cible {refs}, {mpn} ou {valeur,
boitier} ; groupe bornes_dc, cible {composant}. Une borne doit déjà être posée dans la page
(Chute DC → ⚡ Du schéma) : sinon le dire à l'utilisateur.

## Lire un résultat

1. `cao_sim_lire` (quoi=simulations) : les analyses, celles qui ont un résultat, leurs
   chiffres clés.
2. `cao_sim_lire` (quoi=simulation) d'une analyse : commencer par (partie=chiffres) puis
   (partie=diagnostics) ; la partie json, volumineuse, en dernier recours.
3. Dire si le résultat est fiable, ce qui ne va pas et pourquoi, puis ce qu'il faut corriger
   sur la carte, par ordre de priorité.
