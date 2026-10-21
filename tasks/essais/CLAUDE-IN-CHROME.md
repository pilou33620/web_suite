# Faire un essai avec Claude in Chrome

Les essais de ce dossier se donnent à **Claude in Chrome** (l'extension Claude du navigateur),
sur le **PC Windows du boulot**. Claude in Chrome n'a que le navigateur : pas de terminal, pas
de fichier local, pas de Claude Code ni de MCP. Ce qui demande autre chose (le Pi, l'iPad, une
pièce en main, une norme payante) reste à toi : chaque spec le dit en tête (« Qui »).

## Ce que tu fais avant

1. Lance WEB·SUITE sur le PC (double-clic sur `demarrer_WEB_SUITE.cmd`) : la page du lanceur
   s'ouvre sur `http://127.0.0.1:8100`.
2. Ouvre Claude in Chrome et colle la **consigne** de la spec (le bloc « Consigne pour Claude in
   Chrome »). Elle commence toujours par lui faire lire ce fichier-ci.
3. Quand il a fini, copie son **rapport** et donne-le-moi (Claude Code, dans le dépôt) : je
   coche les cases et corrige ce qu'il faut.

## Règles pour Claude in Chrome (à respecter dans tous les essais)

- **Adresses** : lanceur `http://127.0.0.1:8100` ; WEB_CAO `http://127.0.0.1:8000`
  (schéma `/editeur-schematique/editeur-schematique.html`, PCB `/editeur-pcb/editeur-pcb.html`,
  visionneuse `/visionneuse-ipc2581/visionneuse-ipc2581.html`) ; WEB_ANTENNA
  `http://127.0.0.1:8732` ; WEB_3D `http://127.0.0.1:8139`. Si un outil n'est pas lancé, le
  bouton **▶ Lancer** de sa carte dans le lanceur le démarre ; son adresse réelle est sur la carte
  (le lanceur prend le port suivant si le sien est pris).
- **Ne jamais toucher aux projets existants de PROJETS** : WEB·SUITE enregistre dans PROJETS,
  qui part sur GitHub. On travaille sur un **exemple** chargé dans la page (rien n'est
  enregistré), ou, quand la spec l'exige, dans un projet **nouveau** nommé `essai-claude-<sujet>`.
- **Jamais** : « ⇧ Envoyer sur GitHub », « Mise à jour… », « ⇩ Télécharger », ■ Arrêter un outil
  que la spec n'a pas lancé, supprimer quoi que ce soit, Enregistrer (Ctrl+S) hors d'un projet
  `essai-claude-*`.
- **Fenêtres de confirmation** : un `confirm()` du navigateur bloque l'extension. Ouvrir un
  exemple sur une carte non vide en pose un (« La carte en cours sera perdue ») : ouvre les
  exemples dans un onglet neuf, ou par l'outil JavaScript quand la spec donne la commande.
  Si une fenêtre bloque quand même, arrête-toi et demande à l'utilisateur de la fermer.
- **Outil JavaScript** : permis pour lire l'état de la page et pour les commandes que la spec
  donne mot pour mot ; pas pour modifier un document autrement que par la spec.
- **Arrêt** : après 2 ou 3 échecs du même geste, ou si la page ne répond plus, arrête-toi et
  écris dans le rapport ce que tu as tenté et vu.
- **Captures** : une capture d'écran à chaque « → attendu » qui la demande ; un GIF seulement si
  la spec le dit.

## Le rapport

Toujours ce format, en texte, à la fin :

```
ESSAI : <nom du fichier de la spec>
DATE : <jour et heure>   NAVIGATEUR : <version de Chrome>   OUTILS : <versions affichées par le lanceur>
RÉSULTAT : réussi | raté | partiel
ÉTAPES :
1. <ce qui a été fait> → <ce qui a été vu, chiffré> (OK / écart)
...
CRITÈRES : <chaque case « C'est réussi si », cochée ou non, avec la preuve>
ÉCARTS : <tout ce qui diffère de l'attendu, messages exacts>
CONSOLE : <erreurs de la console de la page, s'il y en a>
```
