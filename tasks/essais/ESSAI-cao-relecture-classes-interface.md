# Essai : relecture des classes d'interface (USB, Ethernet, LVDS, CAN, RS-485)

Qui : Claude in Chrome (ta relecture humaine reste possible en plus, et le verdict est le tien) · Où : PC Windows du boulot (navigateur, github.com) · Durée : 1 h à 1 h 30 · Source : `web_cao/tasks/todo.md:80` et `:181`, `web_cao/tasks/plan.md:88`

## Pourquoi
Les trois cases demandent la même chose : qu'un relecteur, pas l'agent qui l'a écrit, relise le code des classes d'interface (spec `web_cao/SPEC.md`, faites les 13–14/10/2026). Elles étaient prévues en deux temps — serveur avant de passer au JS (`plan.md:88`, `todo.md:80`), puis pages (`todo.md:181`) — mais tout est déjà livré dans `main` : une seule relecture, de bout en bout, les lève toutes. « Puis un commit par tâche » (`todo.md:80`) est dépassé : les commits sont faits. Seule la `/review` automatique est passée ; personne n'a relu.

Le dépôt WEB_CAO est public sur github.com : Claude in Chrome lit le code dans le navigateur, avec des questions précises et un format de remarques fixe. Les jugements de concepteur (les valeurs te conviennent-elles ?) et le verdict final restent à toi ; il te les prépare, sources à l'appui.

La vérification sur la carte TRD1_N, qui suit les mêmes points d'étape, est un autre geste : `ESSAI-cao-trd1n-eth100.md`.

## Ce que tu prépares
Rien : Claude in Chrome n'a besoin que de github.com. Lis son rapport à la fin et tranche.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-relecture-classes-interface.md. But : relire, sur github.com, le code des
classes d'interface de WEB_CAO (dépôt public pilou33620/WEB_CAO, branche main) et rendre des
remarques précises. Tu ne modifies rien sur GitHub (pas d'issue, pas de commentaire, pas de
fork, pas d'édition) : tu lis et tu écris le rapport.

Méthode de lecture : les fichiers s'ouvrent en brut à
https://raw.githubusercontent.com/pilou33620/WEB_CAO/main/<chemin>
Dans une page brute, cette commande donne les lignes numérotées qui contiennent un motif
(remplace MOTIF) :
(()=>{const re=/MOTIF/;return document.body.innerText.split("\n").map((l,i)=>(i+1)+": "+l).filter(l=>re.test(l)).join("\n")})()
et celle-ci un extrait (remplace DEBUT et FIN par des numéros de ligne) :
(()=>{const L=document.body.innerText.split("\n");return L.slice(DEBUT-1,FIN).map((l,k)=>(DEBUT+k)+": "+l).join("\n")})()
Les numéros de ligne donnés ci-dessous datent du 21/10/2026 : cherche par nom de fonction s'ils
ont glissé, et donne toujours les numéros que tu as lus.

0. Note le commit de main : https://github.com/pilou33620/WEB_CAO/commits/main (7 premiers
   caractères du premier commit). Liste les commits qui ont touché la table :
   https://github.com/pilou33620/WEB_CAO/commits/main/commun/classes-interface.js
   (et ceux de python/crosstalk.py et python/analyse_carte.py depuis le 13/10/2026).
1. LES VALEURS. Compare trois tables :
   - SPEC.md, tableau « Classes proposées » (vers les lignes 32-40) ;
   - python/crosstalk.py, `INTERFACES` (vers la ligne 909) ;
   - commun/classes-interface.js, `CLASSES_INTERFACE` (vers les lignes 15-23).
   → attendu : mêmes noms courts, familles, t_r, cadences, Z_diff dans les trois ; aucune Z₀ par
     défaut ; cadence ≤ 0,1 / t_r pour chacune (USB HS 200 MHz, ETH 1000 50 MHz…). Calcule 0,1/t_r.
   → à juger, sources à l'appui (norme publique, fiche de circuit d'interface d'un fabricant,
     PRIMAIRE / SECONDAIRE) : t_r d'ETH 100 à 3 ns (ANSI X3.263 : 3–5 ns), USB HS 500 ps, LVDS
     300 ps, CAN 50 ns, RS-485 20 ns, Z_diff 90 / 100 / 120 Ω. Pour chaque valeur : plausible /
     douteuse (pourquoi, quelle valeur, quelle source).
2. LE SERVEUR (python/).
   - crosstalk.py : `famille` (vers 927) et `front_de_classe` (vers 2353). Question : une classe
     d'interface est-elle jugée à min(t_r ; 0,1 / cadence) ? une classe inconnue garde-t-elle son
     comportement d'avant ?
   - analyse_carte.py : les règles qui testent la famille et non le nom (vers 1747, nets rapides
     par couche dans `empilage` ; vers 2410, découplage) ; `zdiff_classes` (vers 319-322 et 2760) ;
     la note du rapport (vers 3498-3502) ; la règle `retours` (vers 408, texte vers 595-597).
     Questions : une carte SANS classe d'interface produit-elle exactement le rapport d'avant
     (aucune entrée d'interface ajoutée aux réglages ni au texte) ? Montre la ligne qui le
     garantit, ou dis qu'aucune ne le garantit. Une classe ancienne nommée « CAN » ou « LVDS »
     est-elle jugée comme l'interface, et le rapport le dit-il (texte de la note) ? La règle des
     paires juge-t-elle chaque paire face à la Z_diff de SA classe ?
3. LES PAGES (JS).
   - PCB : `classOptionsHtml` (editeur-pcb/js/01-core.js, vers 2869, groupe « Nouvelle classe »),
     `autoPairs` (même fichier, vers 2956), `simNaturePcb` (editeur-pcb/js/19-simulation.js,
     vers 2546) ; panneau de vérification `commun/simulation-em.js` (cherche `zdiff_classes`,
     `CLASSES_INTERFACE`, `simCarteZdc` : une ligne par interface, Z₀ et Z_diff).
   - Schéma : editeur-schematique/js/04-etat.js (vers 31-33).
   - Visionneuse : `pnlSelNature` (visionneuse-ipc2581/js/05-panneaux.js, vers 344) et
     `MDL_NATURES_SIGNAL` dans visionneuse-ipc2581/js/02-modele.js (vers 1649 et 1688).
   - Les trois pages HTML chargent-elles la même table ? (cherche classes-interface.js dans
     editeur-pcb/editeur-pcb.html, editeur-schematique/editeur-schematique.html,
     visionneuse-ipc2581/visionneuse-ipc2581.html).
   Question centrale : AUCUNE heuristique de nom de net ne propose une classe d'interface
   (SPEC.md vers la ligne 25 : « à la main »). Cherche dans ces fichiers et dans
   python/analyse_carte.py et python/crosstalk.py les motifs USB|ETH|LVDS|CAN|RS.?485|TRD|D\+|DP|DM
   appliqués à des NOMS DE NETS ; chaque endroit trouvé : heuristique ou non ?
4. Pour chaque problème, une remarque au format ci-dessous. Pas de remarque de style : seulement
   ce qui fausse un verdict, casse une compatibilité, ou contredit SPEC.md.

Format de chaque remarque :
  <fichier>:<ligne> (main @ <commit>) — BLOQUANT | À CORRIGER | REMARQUE — <constat en une
  phrase> — <ce qu'il faudrait à la place>

Rapport, en plus du format commun :
- COMMIT lu, et la liste des commits de l'étape 0.
- VALEURS : le tableau des trois sources côte à côte, 0,1/t_r, et ton avis valeur par valeur avec
  sa source.
- SERVEUR, PAGES : réponses aux questions, puis les remarques au format.
- PROPOSITION DE VERDICT : « rien de bloquant » ou « à reprendre » (le verdict final est celui de
  l'utilisateur).
```

## C'est réussi si
- [ ] Valeurs du tableau comparées dans les trois sources, avis sourcé valeur par valeur.
- [ ] Serveur : réponses aux questions, remarques (fichier:ligne, gravité) ou « rien à redire ».
- [ ] Pages : même chose ; heuristiques de nom de net cherchées et classées.
- [ ] Ton verdict : « relu, accepté » ou « relu, à reprendre ».

## Ce qui reste à toi
- Trancher les valeurs (fronts ETH 100 à 3 ns, LVDS 300 ps, CAN 50 ns…) : te conviennent-elles comme concepteur ? L'avis sourcé du rapport t'aide, il ne décide pas.
- Donner le verdict. Tu peux relire toi-même les endroits marqués BLOQUANT avant.

## Après
Pour Claude Code, avec le rapport et ton verdict :
- Accepté :
  - [ ] cocher `web_cao/tasks/todo.md:80` et `:181`, `web_cao/tasks/plan.md:88`.
- Remarques :
  - [ ] les traiter une par une (un commit chacune, essai à l'appui dans `web_cao/python/test/banc-analyse-carte.py`, `banc-crosstalk.py` ou les harnais), puis cocher les mêmes cases.
