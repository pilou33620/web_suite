# Essai : la carte TRD1_N en « ETH 100 » n'est plus condamnée par la règle `retour`

Qui : les deux (toi : déposer les fichiers de la carte ; Claude in Chrome : la vérification dans la visionneuse et le PCB) · Où : PC Windows du boulot · Durée : 5 min pour toi, 30 min pour Claude in Chrome · Source : `web_cao/tasks/todo.md:79` et `:180`, `web_cao/tasks/plan.md:87` et `:100`, `web_cao/A-FAIRE.md:453` (« Reste à vérifier sur la vraie carte TRD1_N »)

## Pourquoi
Avant les classes d'interface, TRD1_N (100BASE-TX, fronts de 3 à 5 ns) était classé Rapide, jugé à 1 ns, et **condamné par la règle des chemins de retour** alors qu'il tient jusqu'à 2,7 ns (`web_cao/SPEC.md:11-13`). Depuis le 13/10/2026, la classe « ETH 100 » porte t_r = 3 ns, cadence 31,25 MHz, Z_diff 100 Ω (`commun/classes-interface.js:18`, `python/crosstalk.py:914`) ; le serveur juge à min(3 ns ; 0,1 / 31,25 MHz) = 3 ns. La règle `retour` (`python/analyse_carte.py:408`, `retours`) ne garde un via que si son verdict n'est pas « ok » et l'écrit « jugé au front de … ns de la classe … ; sans risque si le front réel est plus lent que … ns » (l.595-597). Les bancs passent ; l'essai sur la vraie carte, dans la vraie page, n'a jamais été fait. La carte n'est pas dans le dépôt : toi seul l'as.

Quatre cases pour un seul geste : la même vérification, dans la visionneuse puis dans le PCB. Claude in Chrome la fait entièrement dans les pages ; il lui faut seulement les fichiers, servis par WEB_CAO.

## Ce que tu prépares
1. L'IPC-2581 de la carte qui porte TRD1_N (celle du constat d'origine, commit `fb326e3`) : renomme-le exactement `essai-trd1n.xml`.
2. Si tu as son document de l'éditeur PCB (`…-PCB.json`) : une copie renommée `essai-trd1n-PCB.json`. Prends-la où elle est (dans PROJETS, copie seulement : ne déplace ni ne modifie l'original).
3. Avec l'explorateur Windows, copie-les dans `<dossier de WEB·SUITE>\WEB_CAO\IPC2581_Exemple\` (le dossier `WEB_CAO` est à côté de `demarrer_WEB_SUITE.cmd` ; crée `IPC2581_Exemple` s'il manque). Ce dossier est ignoré par git : rien ne part sur GitHub. Pas dans PROJETS.
4. WEB·SUITE lancé, avec le serveur de WEB_CAO (la vérification passe par lui).

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-trd1n-eth100.md. But : vérifier que les nets TRD de la carte, classés
« ETH 100 », ne sont plus condamnés par la règle `retour`. Fichiers déposés par l'utilisateur,
servis en lecture : http://127.0.0.1:8000/IPC2581_Exemple/essai-trd1n.xml et, peut-être,
.../essai-trd1n-PCB.json. Rien ne s'enregistre : pas de « ☁ Enregistrer », pas de Ctrl+S,
pas de « Exporter .json ». Le bouton « Rapport ↗ » télécharge un .txt : ne t'en sers pas, lis le
rapport par la commande donnée.

Visionneuse
1. Onglet neuf : http://127.0.0.1:8000/visionneuse-ipc2581/visionneuse-ipc2581.html, puis :
   (async()=>{const n="essai-trd1n.xml";const r=await fetch("/IPC2581_Exemple/"+n);if(!r.ok)return "HTTP "+r.status+" : fichier absent";await charger(new File([await r.blob()],n));return V.fichier+" : "+V.modele.composants.length+" composants";})()
   → une fenêtre de classement des nets s'ouvre : clique « Passer » (on ne reclasse RIEN ici).
   → à relever : `JSON.stringify(V.modele.netNatures||{})` (attendu {} ou sans TRD : sinon la
     carte garde un classement d'un essai précédent, note-le).
2. Bouton « Simulation EM… », groupe « AUDIT » (Audit de la carte), onglet « Vérification », « ▶ Vérifier la carte ». Attends la fin,
   puis :
   simCarteTexte().split("\n").filter(l=>/^==|TRD|ETH|Réserve|interface/i.test(l)).join("\n")
   → à relever (témoin d'AVANT) : les lignes de la règle des chemins de retour sur TRD1_N (et
     TRD1_P, TRD2_x s'il y en a) : sévérité, « jugé au front de … ns de la classe … », « sans
     risque si le front réel est plus lent que … ns ». Capture d'écran de l'onglet.
3. Bouton « ⚡ Nets (PWR/GND/Sig) ». Dans la recherche de la fenêtre, tape TRD ; pour chaque net
   TRD (TRD1_N, TRD1_P et les autres paires), choisis « ETH 100 » dans sa colonne de
   classification. Puis « ✓ Valider et appliquer les préréglages ».
   → à relever : JSON.stringify(V.modele.netNatures)  (attendu : chaque TRD → "ETH 100").
4. Onglet « Vérification », « ▶ Vérifier la carte » de nouveau, puis la même commande qu'en 2.
   → attendu : plus aucune ligne `retour` « CRITIQUE » sur les nets TRD. S'il en reste une, elle
     dit « jugé au front de 3 ns de la classe ETH 100 » : recopie-la entière.
   → à relever aussi : la règle des paires différentielles juge-t-elle la paire TRD1 face à
     100 Ω (et non à la Z_diff de la carte) ? Une ligne « Classe « ETH 100 » jugée comme
     l'interface Ethernet 100BASE-TX … » apparaît-elle ?
   Garde le rapport entier pour le rapport final : simCarteTexte()
5. Mémoire par fichier (la nature doit survivre à la réouverture) :
   localStorage.getItem(mdlCleMemoireNets())
   → attendu : un JSON dont `natures` donne "ETH 100" pour les TRD.
   Puis rouvre le fichier avec la commande de l'étape 1 (« Passer » dans la fenêtre des nets), et :
   JSON.stringify(V.modele.netNatures)
   → attendu : les TRD toujours en "ETH 100".

Éditeur PCB (seulement si http://127.0.0.1:8000/IPC2581_Exemple/essai-trd1n-PCB.json répond ;
sinon écris « pas de document PCB » et passe au rapport)
6. Onglet neuf : http://127.0.0.1:8000/editeur-pcb/editeur-pcb.html, puis :
   (async()=>{const r=await fetch("/IPC2581_Exemple/essai-trd1n-PCB.json");if(!r.ok)return "HTTP "+r.status;push();loadDoc(await r.json());fit();return S.fps.length+" empreintes, classe de TRD1_N : "+(S.netClass["TRD1_N"]||"Défaut");})()
   → à relever : la classe de TRD1_N avant.
7. Panneau « Nets & composants » : pour TRD1_N, TRD1_P (et les autres TRD), choisis « ETH 100 »
   dans le menu de classe du net (groupe « Nouvelle classe » s'il n'existe pas encore sur la
   carte). Contrôle : JSON.stringify(Object.fromEntries(Object.entries(S.netClass).filter(([n])=>/TRD/.test(n))))
8. Menu « Outils » → « Simulation EM… », groupe « AUDIT », onglet « Vérification », « ▶ Vérifier la carte », puis la
   commande de l'étape 2.
   → attendu : le même résultat qu'à l'étape 4 (sévérités, fronts, paires à 100 Ω).
   Garde le rapport entier : simCarteTexte()
   Ne ferme pas cet onglet à la fin (le navigateur demanderait « Quitter le site ? », qui te
   bloquerait) : l'utilisateur le fermera sans enregistrer.

Rapport, en plus du format commun :
- AVANT (visionneuse, sans reclassement) : les lignes TRD de la règle de retour (sévérité, front, limite).
- APRÈS, visionneuse : lignes TRD restantes (texte exact) ou « aucune ligne retour critique ».
- APRÈS, PCB : même chose, ou « pas de document PCB ».
- PAIRE TRD1 jugée à 100 Ω : oui / non (ligne du rapport à l'appui).
- MÉMOIRE : la nature survit à la réouverture : oui / non.
- Les deux rapports entiers (simCarteTexte) en annexe.
```

## C'est réussi si
- [ ] Avant (sans reclassement) : sévérité et limite en ns de la ligne `retour` TRD1_N.
- [ ] Après (ETH 100), visionneuse : aucune ligne `retour` critique sur TRD1_N ; sinon, le texte exact de la ligne.
- [ ] Après, PCB : même résultat (ou « pas de document PCB »).
- [ ] Paire TRD1 jugée à 100 Ω : oui / non.
- [ ] La nature survit à la réouverture : oui / non.
- [ ] Les deux rapports texte, joints.

## Ce qui reste à toi
- Déposer les fichiers (ci-dessus) : Claude in Chrome n'a pas accès au disque.
- À la fin, fermer l'onglet PCB sans enregistrer (« Quitter » à la question du navigateur), et, si tu veux, effacer les deux fichiers de `IPC2581_Exemple`.
- Si une ligne `retour` reste critique à 3 ns : la lire avec moi (distance du via de masse, λ/20).

## Après
Pour Claude Code, avec le rapport :
- Réussi :
  - [ ] cocher `web_cao/tasks/todo.md:79` et `:180`, `web_cao/tasks/plan.md:87` et `:100` ;
  - [ ] retirer « Reste à vérifier sur la vraie carte TRD1_N » de `web_cao/A-FAIRE.md:453`.
- Encore critique à 3 ns : ce n'est plus l'ancien défaut (front mal choisi) mais un vrai retour faible ;
  - [ ] lire la ligne avec l'utilisateur avant de toucher à `web_cao/python/analyse_carte.py:408-600`.
- Nature perdue à la réouverture, ou PCB différent de la visionneuse :
  - [ ] bug dans `web_cao/visionneuse-ipc2581/js/02-modele.js` (`mdlMemoireNets`, l.738-752) ou `web_cao/editeur-pcb/js/19-simulation.js:2546` (`simNaturePcb`) : corriger avec un essai (`harness-sim.js` ou harnais PCB).
