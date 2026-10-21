# Essai : relecture humaine du dossier mécanique

Où : PC avec un clone complet de web_cao et `SPEC-dossier-meca.md` ouvert · Durée : 1 h · Source : `web_cao/tasks/todo-dossier-meca.md:129`

## Pourquoi
La case demandait une relecture humaine avant la visionneuse ; le mode `/build auto` l'a reportée « à la fin ». La fin est arrivée (phase 3 faite, `todo-dossier-meca.md:131-178`) et seule la `/review` automatique est passée (l.183). Personne n'a relu le code. C'est un autre geste que l'import dans une CAO méca (`ESSAI-cao-dossier-meca-import.md`) et que l'essai de l'IA (`ESSAI-cao-dossier-meca-ia.md`) : ici tu lis, tu n'exécutes pas.

## Ce qu'il te faut
- Un clone complet : `git log --oneline -- commun/dossier-meca.js` donne les commits (15/10/2026 → fentes du 21/10/2026, `b11bee2`).
- `web_cao/SPEC-dossier-meca.md` : JSON (l.110-139), architecture (l.141-165), limites « Toujours / Demander d’abord / Jamais » (l.278-288), critères (l.290-308).

## Étapes
1. **Le calcul commun** `commun/dossier-meca.js` (451 lignes) : `mecaDonnees` (fixations, l.138-150 ; dégagement = pastille, sinon Ø + 2 mm, l.142 ; hauteurs saisie → fichier → LIB, l.159-161 ; avertissements l.190-193), `mecaDxf` (l.401-426), `mecaOblong` (l.33-41).
   → à juger : la règle « Ø + 2 mm » pour le dégagement d'une vis ; la borne 100 mm (`MECA_H_MAX`, l.25) ; la colonne LIB lue sous le nom « Maximun Height » (faute d'orthographe gardée, l.131).
2. **L'éditeur** `editeur-pcb/js/36-dossier-meca.js` (134 lignes : tableau des hauteurs et ouvertures, `mecaRegler` l.74, bouton l.112) et la feuille « Plan mécanique » (`editeur-pcb/js/29-draftsman.js:63`), `commun/zip.js`.
   → attendu : rien n'écrit dans la LIB (`SPEC-dossier-meca.md`, « Jamais ») ; une saisie hors bornes est ignorée sans casser le document.
3. **La visionneuse** `visionneuse-ipc2581/js/08-dossier-meca.js` (192 lignes : corps du fichier ou estimé, l.30-38 ; NPTH rattaché à son boîtier, l.61-72) ; le parseur `python/ipc2581_parser.py:2157-2172` ; l'export de la hauteur `editeur-pcb/js/35-ipc2581-export.js` (autour de l.747).
4. Note chaque remarque avec fichier:ligne, et ce que tu veux à la place.

## C'est réussi si / La réponse
- [ ] Calcul commun : rien à redire / remarques (fichier:ligne).
- [ ] Éditeur : rien à redire / remarques.
- [ ] Visionneuse et IPC-2581 : rien à redire / remarques.
- [ ] Ton verdict : « relu, accepté » ou « relu, à reprendre ».

## Après
- Accepté : je coche `web_cao/tasks/todo-dossier-meca.md:129`.
- Remarques : je les traite une par une (commit et essai dans `editeur-pcb/test/harness.js` ou `visionneuse-ipc2581/test/harness-sim.js`), puis je coche.
- Le point de contrôle final (`todo-dossier-meca.md:181`) attend aussi `ESSAI-cao-dossier-meca-ia.md`.
