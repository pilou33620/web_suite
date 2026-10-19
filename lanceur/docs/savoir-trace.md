# Savoir métier : d'où vient chaque règle

Module [savoir-metier](SPEC-mcp-savoir-metier.md). Ce tableau suit chaque règle des
consignes de l'assistant Gemini, jusqu'à l'endroit où le skill `web-suite`
(`.claude/skills/web-suite/`) ou le serveur MCP la reprend. Les sources relevées :

- `WEB_CAO/commun/ia-assistant.js` : CONTEXTE_SUITE, SUGGESTIONS, consigne principale,
  actions, datasheet → simulation, outils de lecture, analyses du clic droit ;
- `WEB_CAO/commun/simulation-rapport.js` : la question qui accompagne un rapport ;
- `WEB_CAO/gestion-lib/js/06-ia-lib.js` : consigne de Gestion LIB ;
- `WEB_ANTENNA/js/30-ia.js` : promptSysteme et outils de lecture.

Les numéros de ligne valent aux révisions `WEB_CAO@c814de83f32d3e11d1c2f2d498efa430631bd586`
et `WEB_ANTENNA@24ba852f80ac21d0efc97a997d08ffe6f8f72c9e`, avant le retrait de Gemini.
`lanceur/banc_savoir.py` relit chaque source à cette révision et vérifie ce tableau. Une destination est l'une des formes suivantes :
- une fiche du skill (`pcb.md`), qui doit exister ;
- `outil : nom`, dont l'outil doit exister ;
- `généré : nom (quoi=x)`, dont l'outil et le `quoi` doivent exister ;
- `écartée : raison` ;
- `à faire`, permis seulement tant que le module n'est pas fini.

| Id | Source | Règle | Destination |
|---|---|---|---|
| C1 | WEB_CAO/commun/ia-assistant.js:281 | La suite : WEB_CAO (schéma, PCB push & shove, DRC, classes, paires, LIB, visionneuse IPC-2581, simulations SI/PI/RF), WEB_ANTENNA (openEMS), WEB_3D (STEP, mesure, assemblage) | SKILL.md |
| C2 | WEB_CAO/commun/ia-assistant.js:285 | Un projet = schéma -SCH, PCB -PCB, carte -IPC ; dans PROJETS/CAO/<projet>, synchronisé sur GitHub ; LIB commune dans PROJETS/LIB_CAO | SKILL.md |
| C3 | WEB_CAO/commun/ia-assistant.js:286 | Une question qui dépasse l'outil ouvert : dire quel outil de la suite y répond | SKILL.md |
| C4 | WEB_CAO/commun/ia-assistant.js:291 | Schéma : pull-up I2C, découplage (100 nF / 10 µF), protection contre l'inversion de polarité (MOSFET P), filtre RC d'entrée ADC | schema.md |
| C5 | WEB_CAO/commun/ia-assistant.js:297 | PCB : largeur selon le courant (IPC-2152, échauffement, cuivre 35 µm), paire USB 2.0 90 Ω, chemin de retour et fente dans le plan, vias thermiques sous une pastille de puissance QFN | pcb.md |
| C6 | WEB_CAO/commun/ia-assistant.js:303 | IPC-2581 : structure du fichier, Rev B / Rev C, tolérances de perçage, isolation et masque | écartée : suggestions de questions générales, sans règle propre à la suite (savoir général de Claude) ; la carte se lit par `cao_ipc_lire` |
| C7 | WEB_CAO/commun/ia-assistant.js:1774 | Rôle : « ingénieur électronicien d'élite » | écartée : rôle de Gemini, sans objet pour Claude Code |
| C8 | WEB_CAO/commun/ia-assistant.js:1776 | Répondre exclusivement en français, sans méta-commentaire anglais | écartée : propre à Gemma ; Claude Code suit la langue de l'utilisateur |
| C9 | WEB_CAO/commun/ia-assistant.js:1778 | Valeurs calculées (composants, largeur de piste) : toujours proposer de les appliquer | pcb.md |
| C10 | WEB_CAO/commun/ia-assistant.js:1779 | Blocs ```action schema_values et pcb_track_width | écartée : remplacés par les outils `cao_valeur_composant`, `cao_composants_ecrire` et `cao_pistes_ecrire` |
| C11 | WEB_CAO/commun/ia-assistant.js:1798 | Datasheet → simulation : ne relever que des valeurs écrites dans le document | simulation.md |
| C12 | WEB_CAO/commun/ia-assistant.js:1799 | Pire cas : courants et tco au max, temps de montée au min, tsu/th au min exigé ; conditions les plus proches du projet | simulation.md |
| C13 | WEB_CAO/commun/ia-assistant.js:1800 | Pour chaque valeur : page et citation exacte | simulation.md |
| C14 | WEB_CAO/commun/ia-assistant.js:1801 | Valeur absente : ne pas l'inventer ni la tirer d'une « valeur usuelle », dire qu'elle manque | simulation.md |
| C15 | WEB_CAO/commun/ia-assistant.js:1802 | Convertir dans l'unité exacte du catalogue (12,5 mA pour pdn_fiche.iActifMa) | simulation.md |
| C16 | WEB_CAO/commun/ia-assistant.js:1803 | Bonne cible : pdn_fiche = charge du rail ; une datasheet de régulateur règle pdn.rVrmMOhm et pdn.fVrmKhz | simulation.md |
| C17 | WEB_CAO/commun/ia-assistant.js:1804 | Répondre d'abord par un tableau court (paramètre, valeur, page, citation), avec une confiance | simulation.md |
| C18 | WEB_CAO/commun/ia-assistant.js:1806 | Bloc sim_params (valeurs, condensateurs ESR/ESL, bornes_dc) | écartée : remplacé par l'outil `cao_sim_ecrire` pour les groupes pdn_fiche, pdn, si, bus, dc ; condensateurs et bornes_dc n'ont pas encore d'écriture MCP (simulation.md le dit : valeurs données à l'utilisateur) |
| C19 | WEB_CAO/commun/ia-assistant.js:1811 | Le catalogue des paramètres réglables (clé, unité, bornes, valeur actuelle, où la lire) joint à la consigne | généré : cao_sim_lire (quoi=catalogue) |
| C20 | WEB_CAO/commun/ia-assistant.js:253 | Le contexte n'est qu'un résumé : demander le détail avant de répondre, du plus ciblé au plus large ; ne jamais inventer une donnée qu'on peut lire | SKILL.md |
| C21 | WEB_CAO/commun/ia-assistant.js:538 | Bus de pistes : skew acceptable à 100 MHz et plus, règle des 3W, diaphonie | pcb.md |
| C22 | WEB_CAO/commun/ia-assistant.js:545 | Empreintes posées : boîtiers, pastilles, recommandations de routage | écartée : question ouverte sans règle ; se lit par `cao_pcb_lire` |
| C23 | WEB_CAO/commun/ia-assistant.js:554 | Portion de schéma : cohérence électrique, valeurs, découplages et résistances de tirage manquants | schema.md |
| C24 | WEB_CAO/commun/ia-assistant.js:564 | Net IPC-2581 : continuité, impédance, pertes, transitions | écartée : question ouverte sans règle ; se lit par `cao_ipc_lire` |
| C25 | WEB_CAO/commun/ia-assistant.js:574 | Composant IPC-2581 : vérifications d'assemblage | écartée : question ouverte sans règle ; se lit par `cao_ipc_lire` |
| R1 | WEB_CAO/commun/simulation-rapport.js:898 | Rapport de simulation : lire chiffres clés et diagnostics d'abord ; dire si le résultat est fiable, ce qui ne va pas et pourquoi, quoi corriger par priorité | simulation.md |
| L1 | WEB_CAO/gestion-lib/js/06-ia-lib.js:192 | Empreintes selon IPC-7351B | composants.md |
| L10 | WEB_CAO/gestion-lib/js/06-ia-lib.js:192 | Symboles selon IEC 60617 / IEEE 315 | écartée : le schéma ne dessine que ses symboles intégrés (module composants), aucun symbole n'est écrit |
| L2 | WEB_CAO/gestion-lib/js/06-ia-lib.js:194 | Répondre en français | écartée : propre à Gemma ; Claude Code suit la langue de l'utilisateur |
| L3 | WEB_CAO/gestion-lib/js/06-ia-lib.js:195 | pcbfp-1 : mm, vue de dessus, origine au centre du corps, y vers le bas, broche 1 en haut à gauche, sens antihoraire | composants.md |
| L4 | WEB_CAO/gestion-lib/js/06-ia-lib.js:195 | Trous NPTH « holes » {x, y, d} (oblong : w, rot) ; perçage oblong « slot » ; les garder en corrigeant une empreinte | composants.md |
| L5 | WEB_CAO/gestion-lib/js/06-ia-lib.js:196 | Bloc ```action:pcb | écartée : remplacé par les outils `lib_empreinte_verifier` et `lib_composant_creer` |
| L6 | WEB_CAO/gestion-lib/js/06-ia-lib.js:213 | Symbole schsym-1, bloc ```action:sch | écartée : décision du module composants (symbole intégré + brochage, pas de schsym-1) |
| L7 | WEB_CAO/gestion-lib/js/06-ia-lib.js:235 | Ligne de catalogue, bloc ```action:comp | écartée : remplacé par l'outil `lib_composant_creer` |
| L8 | WEB_CAO/gestion-lib/js/06-ia-lib.js:249 | Brochage : NOM=patte, virgules, jamais « ; » ; noms des broches des symboles ; parties A:…\|B:…, « *: » ; NC=5/6 ; OUT=2/4 ; exemples AMS1117, MCP6001 | composants.md |
| L9 | WEB_CAO/gestion-lib/js/06-ia-lib.js:251 | Être direct ; expliquer brièvement les cotes choisies (pas, pastilles) | composants.md |
| A1 | WEB_ANTENNA/js/30-ia.js:1581 | Rôle : ingénieur antennes de l'outil | écartée : rôle de Gemini, sans objet pour Claude Code |
| A2 | WEB_ANTENNA/js/30-ia.js:1586 | Carte importée = export IPC-2581 de WEB_CAO : modifier le cuivre dans le PCB de WEB_CAO puis réexporter ; projet dans PROJETS/ANTENNA/<projet> | antenne.md |
| A3 | WEB_ANTENNA/js/30-ia.js:1595 | Répondre exclusivement en français | écartée : propre à Gemma ; Claude Code suit la langue de l'utilisateur |
| A4 | WEB_ANTENNA/js/30-ia.js:1598 | Réponses brèves et chiffrées | antenne.md |
| A5 | WEB_ANTENNA/js/30-ia.js:1602 | Ne rien écrire soi-même : proposer, l'utilisateur applique | antenne.md |
| A6 | WEB_ANTENNA/js/30-ia.js:1603 | N'inventer aucun chiffre du projet : dire qu'il manque et où le lire | antenne.md |
| A7 | WEB_ANTENNA/js/30-ia.js:1605 | Ne pas promettre un résultat : relancer et vérifier | antenne.md |
| A8 | WEB_ANTENNA/js/30-ia.js:1608 | Fréquences toujours en hertz dans les propositions | antenne.md |
| A9 | WEB_ANTENNA/js/30-ia.js:1610 | Longueurs dans l'unité du document, cotes de motif toujours en mm | antenne.md |
| A10 | WEB_ANTENNA/js/30-ia.js:1612 | Zéro = « au mailleur de décider » (marges d'air, pas de maillage) | antenne.md |
| A11 | WEB_ANTENNA/js/30-ia.js:1614 | Arrêt sur l'énergie résiduelle (−40 dB) ; le nombre de pas n'est qu'un garde-fou | antenne.md |
| A12 | WEB_ANTENNA/js/30-ia.js:1618 | Marge d'air : ¼ λ à la fréquence basse, PML comprise | antenne.md |
| A13 | WEB_ANTENNA/js/30-ia.js:1621 | Un gabarit tombe à ±5 % : point de départ, pas une antenne finie | antenne.md |
| A14 | WEB_ANTENNA/js/30-ia.js:1623 | Patch : longueur corrigée L·(1 − 0,466·√(h/λ0)) ; résistance de bord 0,17 (FR-4) à 0,72 (RO4350B) fois la conductance de fente | antenne.md |
| A15 | WEB_ANTENNA/js/30-ia.js:1628 | Patch : resserrer les encoches g de 3,5 à 1,5 mm gagne 8 dB | antenne.md |
| A16 | WEB_ANTENNA/js/30-ia.js:1630 | Maillage : λ/20 ne suffit pas sur une ligne étroite ; 3 à 4 cellules en travers (pas = ¼ de la largeur) | antenne.md |
| A17 | WEB_ANTENNA/js/30-ia.js:1633 | Allonger une ligne d'alimentation n'adapte pas : elle fait tourner Γ sans changer son module | antenne.md |
| A18 | WEB_ANTENNA/js/30-ia.js:1635 | Résonance trop haute : allonger la cote résonante du même pourcentage (f ∝ 1/L) ; une cote à la fois | antenne.md |
| A19 | WEB_ANTENNA/js/30-ia.js:1639 | Quatre cartes (reglages, cotes, balayage, formes) ; plusieurs, une par idée | antenne.md |
| A20 | WEB_ANTENNA/js/30-ia.js:1650 | Cotes : « tracer » à vrai redessine le motif, efface le dessin et remet la bande à ±15 % | antenne.md |
| A21 | WEB_ANTENNA/js/30-ia.js:1662 | Formes : mm, origine au coin de la carte ; couche haut, bas ou nom exact ; « remplacer » efface ; sans port rien ne se simule | antenne.md |
| A22 | WEB_ANTENNA/js/30-ia.js:1671 | Genres admis (rect, disque, poly, piste, via), « trou » = découpe ; formes et sommets plafonnés | antenne.md |
| A23 | WEB_ANTENNA/js/30-ia.js:1678 | Préférer un motif à la géométrie libre ; le dire quand on ne le fait pas | antenne.md |
| A24 | WEB_ANTENNA/js/30-ia.js:1684 | Seuls chemins acceptés pour « reglages » : la grammaire de la page | généré : antenna_lire (quoi=reglages) |
| A25 | WEB_ANTENNA/js/30-ia.js:1688 | port.X / ports.0.X ; con.* en conception seulement ; cotes = motif ouvert ; balayage = cotes balayables ; garde-fou du nombre de points | antenne.md |
| A26 | WEB_ANTENNA/js/30-ia.js:146 | Outils de lecture : le contexte est un résumé, demander avant de répondre, ne rien inventer | SKILL.md |

## Savoir ajouté hors des consignes Gemini

`schema.md` et `routage.md` sont nouvelles par décision de la spec. Ailleurs, ce qui ne vient
pas d'une consigne Gemini vient du code, sauf deux précisions validées par l'utilisateur (2026-10-19) :

| Fiche | Ajout | Source |
|---|---|---|
| pcb.md | IPC-2152 a remplacé la charte d'IPC-2221 ; l'étalement compte ; un échauffement en °C vaut des K | code : `WEB_CAO/commun/simulation-em.js` (vers 269, 906) |
| pcb.md | Classes d'usine, classes d'interface et leur Z différentielle, règle de paire d'usine | code : `editeur-pcb/js/01-core.js` (SCH_CLASSES, DP_FALLBACK), `commun/classes-interface.js` |
| pcb.md | Via de masse près d'une via de signal rapide ; grille de vias thermiques prise dans la datasheet | savoir général, précisant les suggestions C5 : validé par l'utilisateur |
| antenne.md | Fente, anneau : antennes qu'aucun motif ne trace | code : `WEB_ANTENNA/js/30-ia.js` (vers 509) |
| simulation.md | Quelle analyse pour quelle question | code : SIM_ANALYSES (`commun/simulation-em.js`) |
