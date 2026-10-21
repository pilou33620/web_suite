# Essai : navigation tactile de WEB_3D sur l'iPad

Qui : toi · Où : iPad (Safari), WEB_3D sur le Pi, à la maison · Durée : 15 min ·
Source : `WEB_3D/README.md:52-55`

**Pas pour le PC du boulot** : il faut un vrai doigt sur un vrai écran tactile.

## Pourquoi

Le README le dit : la navigation tactile « est codée, mais pas encore essayée sur une vraie
tablette ». Le code (`WEB_3D/js/03-navigation.js`) : un doigt tourne (l.198-202, puis
`orbiter`) ; deux doigts déplacent et pincent (l.193-196, puis `pincer`, l.293-302) ; un appui
court sans bouger sélectionne (l.258-268) ; le canevas a `touch-action:none`
(`css/visionneuse-3d.css:131`) pour que Safari ne fasse pas défiler la page.

Point à surveiller : le tour à un doigt lit `ev.movementX` / `movementY` (l.229-230). Si Safari
les donne à 0 au toucher, le modèle ne tournera pas, alors que le pincement, qui lit les
positions, marchera. Lecture seule : pas de copie de PROJETS.

## Ce que tu prépares

1. Sur le Pi, démarre WEB·SUITE (`./demarrer_WEB_SUITE.sh`, qui met WEB_3D à jour) ; WEB_3D doit
   contenir `3f6757e` : `git -C WEB_3D merge-base --is-ancestor 3f6757e HEAD && echo ok`.
2. Sur l'iPad, l'adresse **Réseau** (`http://<adresse du Pi>:8100/`), le code d'appairage si
   demandé, puis **▶ Lancer** sur WEB_3D.
3. Sur l'iPad, `http://<adresse du Pi>:8139/?modele=exemples/assemblage-as1.stp` (le port de la
   carte WEB_3D) → l'assemblage s'affiche.

Claude in Chrome ne peut pas : il faut le toucher réel de Safari. Des événements tactiles
fabriqués en JavaScript dans Chrome porteraient le `movementX` qu'on leur donne : ils ne
diraient rien de ce que donne Safari, qui est la question.

## C'est réussi si

- [ ] Un doigt tourne le modèle, sans faire défiler la page
- [ ] Deux doigts zooment (pincement) et déplacent (glissement), sans zoom de Safari
- [ ] Passer de deux doigts à un ne fait pas sauter la vue
- [ ] Un appui court sélectionne une pièce
- [ ] Le double appui est noté, qu'il marche ou non

## Ce qui reste à toi

1. Un doigt glissé lentement, de gauche à droite puis de haut en bas → le modèle tourne autour
   de son centre ; la page ne défile pas, Safari ne zoome pas.
2. Un doigt glissé vite puis lâché → le tour continue un instant puis s'arrête (inertie,
   `js/00-config.js:135`), sans saut.
3. Deux doigts écartés puis rapprochés, sans les déplacer → zoom avant puis arrière, centré
   entre les doigts ; pas de zoom de la page.
4. Deux doigts glissés ensemble → panoramique, sans rotation.
5. Pincer et glisser ensemble, puis lever un doigt et continuer avec l'autre → zoom et
   déplacement ensemble ; doigt levé, plus rien ne bouge jusqu'à un geste neuf ; pas de saut.
6. Un appui court sur une pièce → elle est sélectionnée, comme au clic de souris.
7. Un double appui sur une pièce → elle devient le centre de rotation. Safari peut ne pas le
   transmettre : note ce qui se passe, ce n'est pas bloquant.
8. **Ajuster** (touche `F`) → tout le modèle revient dans la vue.

Si ça rate, note : la version d'iPadOS et le navigateur ; pour chaque geste raté, ce que fait la
vue (rien, saut, sens inverse, trop lent ou rapide, page qui défile ou zoome) — une courte
vidéo d'écran (Centre de contrôle → Enregistrement de l'écran) dit tout ; si un doigt ne tourne
rien alors que le pincement marche, c'est très probablement `movementX` à 0 ; le journal de
WEB_3D (bouton **Journal** de sa carte, `lanceur/journaux/web_3d.log`).

## Après

Pour Claude Code, en cas de réussite :
- [ ] `WEB_3D/README.md:52-55` : « est codée, mais pas encore essayée sur une vraie tablette »
      → « essayée sur iPad le <date> » (et le double appui, selon le résultat)
- Rien d'autre à cocher : ce point n'est suivi dans aucune liste.
