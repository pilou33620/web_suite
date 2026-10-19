# Antennes : WEB_ANTENNA et openEMS

WEB_ANTENNA mène une carte (un export IPC-2581 de WEB_CAO, ou un dessin du mode conception)
jusqu'à une simulation FDTD openEMS : S11, impédance d'entrée, diagramme de rayonnement.
Un projet vit dans PROJETS/ANTENNA/<projet>.

La carte importée vient du PCB de WEB_CAO : une modification du cuivre se fait dans
l'éditeur PCB de WEB_CAO, puis la carte se réexporte en IPC-2581 et se réimporte.

## Lire, proposer, ne jamais lancer

- Lire : `antenna_lire` (quoi=conception) pour le résumé (carte, empilage, bande, ports,
  boîte, maillage, motif ouvert, cotes balayables, chiffrage) ; (quoi=resultats) pour f0,
  S11, Z, bande à −10 dB, Dmax ; (quoi=courbe) pour une courbe ; (quoi=audit) pour les
  contrôles, chacun avec une carte prête.
- Proposer : `antenna_proposer`, une carte que l'utilisateur clique.
- openEMS ne se lance jamais par le MCP : `antenna_openems_chiffrer` vérifie et chiffre le modèle,
  `antenna_openems_suivre` en suit un que l'utilisateur a lancé.
- Ne jamais inventer un chiffre du projet : s'il manque, le dire et dire où le lire.
- Ne pas promettre un résultat : une simulation se relance et se vérifie.
- Répondre brièvement, avec des chiffres : une explication sans nombre n'aide pas à régler.

## Les conventions de l'outil

- Fréquences **toujours en hertz** dans une proposition (2.45e9), quelle que soit l'unité
  affichée.
- Longueurs dans l'unité du document (mm sauf mention contraire), sauf les cotes de motif,
  **toujours en mm**.
- Zéro veut dire « au mailleur de décider » pour les marges d'air et les pas de maillage :
  proposer zéro est une décision, pas un oubli.
- L'arrêt se fait sur l'énergie résiduelle (−40 dB d'usine) ; le nombre de pas de temps est
  un garde-fou, il ne doit pas être ce qui arrête.

## Ce qui a été mesuré sur cet outil

- Marge d'air : **¼ λ à la fréquence basse**, PML comprise. En dessous, la PML absorbe du
  champ proche réactif et la résonance dérive sans que rien ne le signale.
- Un motif tombe à ±5 % de sa cible : le premier dessin est un point de départ.
- Le motif patch corrige le modèle de cavité :
  - longueur L·(1 − 0,466·√(h/λ0)) ;
  - résistance de bord : 0,17 fois la conductance de fente sur FR-4, 0,72 sur RO4350B,
    interpolée en tan δ entre les deux ;
  - resserrer les encoches g de 3,5 à 1,5 mm gagne 8 dB : elles ajoutent une capacité que
    le modèle de cavité ignore.
- Maillage : λ/20 dans le diélectrique **ne suffit pas** sur une ligne d'alimentation étroite.
  Il faut 3 à 4 cellules en travers de la ligne, soit un pas au quart de sa largeur.
- Allonger une ligne d'alimentation **n'adapte pas** : une ligne sans perte au Z0 de
  référence fait tourner Γ sans changer son module.
- Résonance trop haute : allonger la cote résonante du même pourcentage environ
  (f ∝ 1/L au premier ordre). **Une seule cote à la fois.**

## Les quatre cartes de `antenna_proposer`

Une carte par idée ; plusieurs cartes si plusieurs idées.

1. `{"type":"reglages","titre":…,"valeurs":{chemin: valeur}}` : bande, boîte, maillage,
   arrêt, ports. **Seuls les chemins** que rend `antenna_lire` (quoi=reglages) sont acceptés
   (avec leurs unités, bornes et choix) ; « port.X » vise le port en cours, « ports.0.X »
   le premier ; les chemins « con.* » n'existent qu'en mode conception.
2. `{"type":"cotes","motif":…,"valeurs":{…},"tracer":false}` : cotes du motif ouvert, en mm,
   identifiants tels que la lecture conception les liste. « tracer » à vrai redessine le
   motif : le dessin en cours est effacé et la bande remise à ±15 % ; à faux, seules la fiche
   et son aperçu changent.
3. `{"type":"balayage","titre":…,"source":…,"min":…,"max":…,"pas":…}` : arme un balayage
   (jamais lancé) sur une des cotes balayables listées par la lecture conception, recopiée à
   la lettre ; s'il n'y en a aucune, pas de balayage.
4. `{"type":"formes",…}` : de la géométrie libre en mode conception.
   - mm, origine au coin de la carte, x de 0 à L, y de 0 à W ;
   - « couche » : haut, bas, ou le nom exact d'une couche ;
   - genres : rect, disque, poly (trois sommets au moins), piste (pts, w), via ; « trou » à
     vrai fait une découpe (rect, disque, poly) ;
   - « remplacer » à vrai efface le dessin ; sans port, rien ne se simule.
   Le nombre de formes et de sommets est plafonné : la page refuse au-delà.

## Motif ou géométrie libre

Toujours un motif quand il en existe un : patch rectangulaire, monopôle imprimé, F inversé
(IFA), F inversé à méandres (MIFA), dipôle imprimé, réseau 3 × 3 de dipôles (et une ligne
50 Ω étalon). Un motif calcule ses cotes, tient une fiche, se repose et se balaye. La
géométrie libre ne sait rien d'elle-même : ne la proposer que pour une antenne qu'aucun
motif ne trace (fente, anneau, patch à coins coupés), et le dire en une phrase.
