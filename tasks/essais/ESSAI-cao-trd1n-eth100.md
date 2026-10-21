# Essai : la carte TRD1_N en « ETH 100 » n'est plus condamnée par la règle `retour`

Où : PC avec WEB_CAO et la carte Ethernet qui porte le net TRD1_N (son IPC-2581 et, si tu l'as, son document de l'éditeur PCB) · Durée : 20 min · Source : `web_cao/tasks/todo.md:79` et `:180`, `web_cao/tasks/plan.md:87` et `:100`, `web_cao/A-FAIRE.md:453` (« Reste à vérifier sur la vraie carte TRD1_N »)

## Pourquoi
Avant les classes d'interface, TRD1_N (100BASE-TX, fronts de 3 à 5 ns) était classé Rapide, jugé à 1 ns, et **condamné par la règle des chemins de retour** alors qu'il tient jusqu'à 2,7 ns (`web_cao/SPEC.md:11-13`). Depuis le 13/10/2026, la classe « ETH 100 » porte t_r = 3 ns, cadence 31,25 MHz, Z_diff 100 Ω (`commun/classes-interface.js:18`, `python/crosstalk.py:914`) ; le serveur juge à min(3 ns ; 0,1 / 31,25 MHz) = 3 ns. La règle `retour` (`python/analyse_carte.py:408`) ne garde un via que si son verdict n'est pas « ok » et l'écrit « jugé au front de … ns de la classe … ; sans risque si le front réel est plus lent que … ns » (l.594-597). Les bancs passent ; l'essai sur la vraie carte, dans la vraie page, n'a jamais été fait. La carte n'est pas dans le dépôt : toi seul l'as.

Ce sont quatre cases pour un seul geste : la même vérification, dans la visionneuse puis dans le PCB.

## Ce qu'il te faut
- L'IPC-2581 de la carte qui porte TRD1_N (celle du constat d'origine, commit `fb326e3`).
- Son document de l'éditeur PCB (`.json`), si tu l'as (l'éditeur PCB n'importe pas d'IPC-2581).
- WEB_CAO lancé avec son serveur (la vérification passe par `analyse_carte`).

## Étapes
1. Visionneuse : ouvre l'IPC-2581. Onglet de simulation « Vérification », « ▶ Vérifier la carte » **sans rien reclasser**.
   → à relever : les lignes de la règle `retour` sur TRD1_N (et TRD1_P, TRD2_x s'il y en a) : sévérité, « jugé au front de 1 ns de la classe Rapide », « sans risque si le front réel est plus lent que … ns ». C'est le témoin d'avant.
2. Bouton « ⚡ Nets (PWR/GND/Sig) » : mets la nature de TRD1_N, TRD1_P et des autres paires TRD à « ETH 100 ». Ferme.
3. « ▶ Vérifier la carte » de nouveau.
   → attendu : plus aucune ligne `retour` « critique » sur ces nets. S'il en reste une, elle dit « jugé au front de 3 ns de la classe ETH 100 » ; note sa sévérité et sa limite en ns.
   → à relever aussi : la règle *paires* juge-t-elle TRD1 face à 100 Ω (et non à la Z_diff de la carte) ?
4. Ferme et rouvre le fichier : la nature « ETH 100 » doit être revenue (mémoire par fichier).
5. Éditeur PCB (si tu as le document) : classe TRD1_N / TRD1_P dans « ETH 100 » (menu Classe du net, groupe « Nouvelle classe », ou gestionnaire des classes), puis onglet « Vérification », « ▶ Vérifier la carte ».
   → attendu : le même résultat qu'à l'étape 3.
6. « Rapport ↗ » dans les deux pages : garde les deux textes.

## C'est réussi si / La réponse
- [ ] Avant (Rapide) : sévérité et limite en ns de la ligne `retour` TRD1_N.
- [ ] Après (ETH 100), visionneuse : aucune ligne `retour` critique sur TRD1_N ; sinon, le texte exact de la ligne.
- [ ] Après, PCB : même résultat (ou « pas de document PCB »).
- [ ] Paire TRD1 jugée à 100 Ω : oui / non.
- [ ] La nature survit à la réouverture : oui / non.
- [ ] Les deux rapports texte, joints.

## Après
- Réussi : je coche `web_cao/tasks/todo.md:79` et `:180`, `web_cao/tasks/plan.md:87` et `:100`, et je retire « Reste à vérifier sur la vraie carte TRD1_N » de `web_cao/A-FAIRE.md:453`.
- Encore critique à 3 ns : ce n'est plus l'ancien défaut (front mal choisi) mais un vrai retour faible ; je lis la ligne avec toi (distance du via de masse, λ/20) avant de toucher au code de `python/analyse_carte.py:408-600`.
- Nature perdue ou PCB différent de la visionneuse : bug dans `visionneuse-ipc2581/js/02-modele.js` ou `editeur-pcb/js/19-simulation.js` (`simNaturePcb`), je corrige avec un essai.
