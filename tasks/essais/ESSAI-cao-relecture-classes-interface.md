# Essai : relecture humaine des classes d'interface (USB, Ethernet, LVDS, CAN, RS-485)

Où : PC avec un clone complet de web_cao (historique entier) et `SPEC.md` ouvert · Durée : 1 h à 1 h 30 · Source : `web_cao/tasks/todo.md:80` et `:181`, `web_cao/tasks/plan.md:88`

## Pourquoi
Les trois cases demandent la même chose : que toi, pas un agent, relises le code des classes d'interface (spec `web_cao/SPEC.md`, faites les 13–14/10/2026). Elles étaient prévues en deux temps — serveur avant de passer au JS (`plan.md:88`, `todo.md:80`), puis pages (`todo.md:181`) — mais tout est déjà livré dans `main` : une seule relecture, de bout en bout, les lève toutes. « Puis un commit par tâche » (`todo.md:80`) est dépassé : les commits sont faits. Seule la `/review` automatique est passée ; personne n'a relu.

La vérification sur la carte TRD1_N, qui suit les mêmes points d'étape, est un autre geste : `ESSAI-cao-trd1n-eth100.md`.

## Ce qu'il te faut
- Un clone complet (pas superficiel) : `git log --oneline -- commun/classes-interface.js python/crosstalk.py python/analyse_carte.py` donne les commits à relire (depuis `fb326e3`).
- `web_cao/SPEC.md` : tableau des sept classes (l.32-40), critères 1 à 8 (l.57-95), décisions (l.184-193).

## Étapes
1. **Les valeurs.** Compare le tableau de `SPEC.md:32-40` à `python/crosstalk.py:909-925` (`INTERFACES`) et à `commun/classes-interface.js:15-25` (`CLASSES_INTERFACE`).
   → attendu : mêmes noms, familles, t_r, cadences, Z_diff ; aucune Z₀ par défaut ; cadence ≤ 0,1 / t_r (USB HS 200 MHz, ETH 1000 50 MHz).
   → à juger : ces valeurs te conviennent-elles comme concepteur (fronts ETH 100 à 3 ns, LVDS 300 ps, CAN 50 ns…) ?
2. **Le serveur.** `crosstalk.famille` (l.927) et `front_de_classe` (l.2353) ; dans `python/analyse_carte.py`, les règles qui testent la famille et non le nom (l.1747, nets rapides par couche dans `empilage` ; l.2410, découplage), `zdiff_classes` (l.319-322, l.2760) et la note du rapport (l.3501-3502).
   → attendu : une carte sans interface donne le rapport d'avant à l'octet près (`web_cao/A-FAIRE.md:453`, tâche T2 de `plan.md`) ; une classe ancienne nommée « CAN » ou « LVDS » est jugée comme l'interface et le rapport le dit.
3. **Les pages.** PCB : `classOptionsHtml` (`editeur-pcb/js/01-core.js:2869`, groupe « Nouvelle classe »), `simNaturePcb` (`editeur-pcb/js/19-simulation.js:2546`), `autoPairs` ; panneau de vérification (`commun/simulation-em.js`, une ligne par interface, Z₀ et Z_diff) ; schéma : `editeur-schematique/js/04-etat.js:33` ; visionneuse : `pnlSelNature` (`visionneuse-ipc2581/js/05-panneaux.js:344`) et `visionneuse-ipc2581/js/02-modele.js`.
   → attendu : aucune heuristique de nom de net (`SPEC.md:25`, « à la main ») ; la même table chargée par les trois pages.
4. Note chaque remarque avec fichier:ligne, et ce que tu veux à la place.

## C'est réussi si / La réponse
- [ ] Valeurs du tableau : acceptées / à changer (lesquelles, quelle valeur, quelle source).
- [ ] Serveur : rien à redire / remarques (fichier:ligne).
- [ ] Pages : rien à redire / remarques (fichier:ligne).
- [ ] Ton verdict : « relu, accepté » ou « relu, à reprendre ».

## Après
- Accepté : je coche `web_cao/tasks/todo.md:80` et `:181`, `web_cao/tasks/plan.md:88`.
- Remarques : je les traite une par une (un commit chacune, essai à l'appui dans `python/test/banc-analyse-carte.py`, `banc-crosstalk.py` ou les harnais), puis je coche les mêmes cases.
