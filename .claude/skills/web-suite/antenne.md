# Antennes : WEB_ANTENNA et openEMS

WEB_ANTENNA mène une carte (un export IPC-2581 de WEB_CAO, ou un dessin du mode conception)
jusqu'à une simulation FDTD openEMS : S11, impédance d'entrée, diagramme de rayonnement.
Un projet vit dans PROJETS/ANTENNA/<projet>.

La carte importée vient du PCB de WEB_CAO : une modification du cuivre se fait dans
l'éditeur PCB de WEB_CAO, puis la carte se réexporte en IPC-2581 et se réimporte.

## Lire, modifier, ne jamais lancer

- Lire : `antenna_lire` (quoi=conception) pour le résumé (carte, empilage, bande, ports,
  boîte, maillage, motif ouvert, cotes balayables, chiffrage) ; (quoi=resultats) pour f0,
  S11, Z, bande à −10 dB, Dmax ; (quoi=courbe) pour une courbe ; (quoi=audit) pour les
  contrôles, chacun avec sa correction proposée ; (quoi=reglages) pour les réglages, leurs unités,
  bornes et choix.
- Modifier : par les gestes de l'assistant (ci-dessous), une carte par geste.
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

## Les quatre gestes de l'assistant

Une idée à la fois.

1. Réglages (`#bAssistant`) : bande `#antF1`, `#antF2`, `#antUF`, `#antFc`, `#antN` ; boîte `#antMx`,
   `#antMy`, `#antMzh`, `#antMzb`, `#antPml` ; maillage `#antRa`, `#antRd`, `#antTiers` ; arrêt `#antEn`,
   `#antNmax` ; ports `#bPortPlus`, `#antPtype`, `#antPx`, `#antPy`, `#antPde`, `#antPa`, `#antPdir`,
   `#antPw`, `#antPl`, `#antPR`. Les valeurs, unités, bornes et choix sont ceux que rend `antenna_lire`
   (quoi=reglages) ; « port.X » y vise le port en cours, « ports.0.X » le premier ; les chemins
   « con.* » n'existent qu'en mode conception.
2. Cotes du motif (`#bConcevoir`) : choisir le motif `[data-con-gabarit]`, saisir `[data-con-cote]`
   (en mm, identifiants tels que la lecture conception les liste). « Dessiner ce motif » le
   redessine : le dessin en cours est effacé et la bande remise à ±15 % ; sans ce bouton, seules
   la fiche et son aperçu changent.
3. Balayage (étape « Le calcul ») : `#antBalOn` arme (jamais lancé), `#antBalSrc` choisit une des cotes
   balayables de la lecture conception, `#antBalMin`, `#antBalMax`, `#antBalPas` ; s'il n'y en a aucune,
   pas de balayage. 40 points au plus. `#bBalLancer` reste à l'utilisateur.
4. Géométrie libre, en mode conception : outils `[data-con-outil]` (Rectangle, Piste, Polygone,
   Disque, Via), à la souris sur le canevas (pixels de `pilote_capture`) ; une forme posée se
   corrige par `[data-con-el]`, se retire par `[data-con-el-suppr]`.
   - mm, origine au coin de la carte, x de 0 à L, y de 0 à W ;
   - rectangle (x1, y1, x2, y2), disque (cx, cy, r), polygone (trois sommets au moins), piste
     (points, largeur), via (x, y, d) ; une forme peut être une découpe ;
   - sans port rien ne se simule ;
   - une forme entièrement hors de la carte n'a pas de sens ; le nombre de formes et de sommets
     est plafonné.

## Motif ou géométrie libre

Toujours un motif quand il en existe un : patch rectangulaire, monopôle imprimé, F inversé
(IFA), F inversé à méandres (MIFA), dipôle imprimé, réseau 3 × 3 de dipôles (et une ligne
50 Ω étalon). Un motif calcule ses cotes, tient une fiche, se repose et se balaye. La
géométrie libre ne sait rien d'elle-même : ne la proposer que pour une antenne qu'aucun
motif ne trace (fente, anneau, patch à coins coupés : les exemples de la page), et le dire en une phrase.
