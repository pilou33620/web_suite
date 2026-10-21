# Essai : navigation tactile de WEB_3D sur l'iPad

Où : iPad (Safari), WEB_3D sur le Pi · Outils : WEB·SUITE (WEB_3D lancé) · Durée : 15 min ·
Source : `WEB_3D/README.md:52-55`

## Pourquoi

Le README le dit : la navigation tactile « est codée, mais pas encore essayée sur une vraie
tablette ». Le code (`WEB_3D/js/03-navigation.js`) :
- un doigt tourne (l.198-202, puis `orbiter`, l.235-237) ;
- deux doigts déplacent et pincent pour zoomer (l.191-196, puis `pincer`, l.293-302) ;
- un appui court sans bouger sélectionne (l.258-268) ;
- le canevas a `touch-action:none` (`css/visionneuse-3d.css:131`), pour que Safari ne
  fasse pas défiler la page.

Rien de cela n'a tourné sous un vrai doigt. Un point à surveiller : le tour à un doigt lit
`ev.movementX` / `movementY` (l.229-230). Si Safari les donne à 0 pour le toucher, le
modèle ne tournera pas, alors que le pincement, qui lit les positions, marchera.

Cet essai ne fait que regarder : rien n'est écrit, pas besoin de copie de PROJETS.

## Avant de commencer

1. Sur le Pi : `cd ~/Documents/web_suite && git -C WEB_3D pull --ff-only` (ou démarre
   WEB·SUITE, qui met les outils à jour pendant l'intro) ; WEB_3D doit contenir le commit
   `3f6757e` : `git -C WEB_3D merge-base --is-ancestor 3f6757e HEAD && echo ok`.
2. Démarre WEB·SUITE (`./demarrer_WEB_SUITE.sh`). Sur l'iPad, ouvre l'adresse **Réseau**
   (`http://<adresse du Pi>:8100/`), saisis le code d'appairage si demandé, puis **▶ Lancer**
   sur WEB_3D.
3. Sur l'iPad, ouvre `http://<adresse du Pi>:8139/?modele=exemples/assemblage-as1.stp`
   (le port est celui de la carte WEB_3D, 8139 sauf s'il était pris).
   → attendu : l'assemblage s'affiche.

## Étapes

1. Un doigt posé sur la vue, glissé lentement de gauche à droite, puis de haut en bas.
   → attendu : le modèle tourne autour de son centre et suit le doigt. La page ne défile pas,
   Safari ne zoome pas.
2. Un doigt, glissé vite puis lâché.
   → attendu : le tour continue un instant puis s'arrête (inertie, allumée par défaut :
   `js/00-config.js:135`). Pas de saut.
3. Deux doigts posés, écartés puis rapprochés, sans les déplacer.
   → attendu : zoom avant puis arrière, centré entre les doigts. Pas de zoom de la page Safari.
4. Deux doigts posés, glissés ensemble dans la même direction.
   → attendu : le modèle se déplace avec les doigts (panoramique), sans tourner.
5. Pincer et glisser en même temps, puis lever un doigt et continuer avec l'autre.
   → attendu : zoom et déplacement ensemble ; une fois le doigt levé, plus rien ne bouge tant
   que tu ne reposes pas un doigt (le code repart d'un geste neuf). Pas de saut de la vue.
6. Un appui court, sans bouger, sur une pièce.
   → attendu : la pièce est sélectionnée, comme au clic de souris sur le Pi.
7. Un double appui sur une pièce.
   → attendu : elle devient le centre de rotation (la vue glisse vers elle). Safari peut ne
   pas transmettre le double appui : note ce qui se passe, ce n'est pas bloquant.
8. **Ajuster** (touche `F` au clavier).
   → attendu : tout le modèle revient dans la vue.

## C'est réussi si

- [ ] Un doigt tourne le modèle, sans faire défiler la page
- [ ] Deux doigts zooment (pincement) et déplacent (glissement), sans zoom de Safari
- [ ] Passer de deux doigts à un ne fait pas sauter la vue
- [ ] Un appui court sélectionne une pièce
- [ ] Le double appui est noté, qu'il marche ou non

## Si ça rate, note

- La version d'iPadOS, et Safari ou un autre navigateur.
- Pour chaque geste raté : ce que fait la vue (rien, saut, sens inverse, trop lent ou trop
  rapide, la page qui défile ou qui zoome). Une courte vidéo d'écran de l'iPad (Centre de
  contrôle → Enregistrement de l'écran) dit tout.
- Si un doigt ne tourne rien alors que le pincement marche : c'est très probablement
  `movementX` à 0 (voir « Pourquoi »), dis-le tel quel.
- Le journal de WEB_3D : bouton **Journal** de sa carte dans WEB·SUITE
  (`lanceur/journaux/web_3d.log`).

## Après

En cas de réussite :
- `WEB_3D/README.md:52-55` : remplacer « est codée, mais pas encore essayée sur
  une vraie tablette » par « essayée sur iPad le <date> » (et le double appui, selon le résultat)
- Rien à cocher ailleurs : ce point n'est suivi dans aucune liste (ni `A-FAIRE.md` de
  web_suite, ni une `A-FAIRE.md` de WEB_3D, qui n'en a pas)
