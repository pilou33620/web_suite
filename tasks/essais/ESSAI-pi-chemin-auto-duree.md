# Essai : `cao_chemin`, durée d'un appel de 20 liaisons (PC, puis Pi)

Qui : les deux · Où : Chrome du PC Windows du boulot (référence), puis Chromium du Pi ·
Durée : 15 min (PC), 20 min (Pi) · Source : `tasks/todo-chemin-auto.md:25-28`, `WEB_CAO/A-FAIRE.md:136-140`

## Pourquoi

`cao_chemin` cherche ses chemins (A* sur grille) **dans la page** : `cheminCommande`,
`WEB_CAO/editeur-pcb/js/40-chemin-auto.js:290`. La durée est donc celle du navigateur qui
affiche la page. Budget : **8 s par appel** (`CHEMIN.BUDGET_MS`, `40-chemin-auto.js:19`) ;
au-delà, chaque liaison restante est refusée « temps épuisé (8 s pour l'appel) ».

Fait le 21/10/2026 sous Linux (`A-FAIRE.md:23-24`) : exemple 2, 20 liaisons en **0,5 s**, 19
trouvées, 19 posées par `cao_router` ; en finissant la route U3.1 → U3.3 (+5V), le routeur
entre de **0,13 mm** dans la pastille U3.2 (GND) (`WEB_CAO/A-FAIRE.md:136-140`). Le routeur
(`11-pns-*` à `13-pns-shove.js`) ne se change pas sans ton accord.

Claude in Chrome mesure la même chose dans le Chrome du PC, sans MCP : il appelle dans la page
`cheminCommande` (ce que fait `cao_chemin`) et la commande `cao_router` de la page
(`routagePiloteCommandes().cao_router`, `editeur-pcb/js/37-routage-pilote.js:371-384`), comme le
harnais (`editeur-pcb/test/harness.js:27806`, « chemin-auto : cao_chemin propose… »). Pas
besoin de remplacer `window.Pilote` : les définitions des commandes s'appellent directement.
Rien n'est enregistré : l'exemple est chargé en mémoire, hors projet.

Rejoué le 21/10/2026 avec ces commandes mot pour mot, au harnais (node) et dans Chromium headless
(Playwright) : 0,3 à 0,5 s, 19 trouvées sur 20 (la 10, GND, sans chemin), 19 posées, DRC 0 → 2
(U3.2, et la paire USB découplée), entrée de 0,129 mm dans U3.2.

## Ce que tu prépares

1. Double-clic sur `demarrer_WEB_SUITE.cmd` : le lanceur s'ouvre sur `http://127.0.0.1:8100`.
2. Ouvre Claude in Chrome et colle la consigne ci-dessous.

## Consigne pour Claude in Chrome

```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-pi-chemin-auto-duree.md. Durée du chercheur de chemins de l'éditeur PCB
(cheminCommande, ce qu'appelle l'outil cao_chemin) sur 20 liaisons, puis pose des routes par la
commande cao_router de la page, et l'isolation sur la pastille U3.2. Tout se fait sur un exemple
chargé en mémoire : rien n'est enregistré. Les commandes JavaScript se collent telles quelles
dans l'outil JavaScript, dans l'onglet de l'éditeur PCB ; n'en écris pas d'autre qui modifie la carte.

1. Ouvre http://127.0.0.1:8100 et relève les versions des outils. Si la carte WEB_CAO dit
   qu'il est arrêté, clique ▶ Lancer sur elle.
   → attendu : WEB_CAO lancé, port 8000 (sinon, prends le port de sa carte dans les adresses).
2. Ouvre un onglet neuf sur http://127.0.0.1:8000/editeur-pcb/editeur-pcb.html et garde-le au
   premier plan jusqu'à la fin.
   → attendu : l'éditeur PCB.
3. Charge l'exemple 2 (« Interface IoT & Bus Haute Vitesse — 4 couches »), déroute-le (pistes et
   vias retirés, zones gardées) et prends ses 20 premières liaisons de pastille à pastille :

(() => {
  loadDoc(exemple2()); fit();
  S.tracks = []; S.vias = []; touch(); refreshPanels(); draw();
  const c = conn(true), parPos = new Map(c.padNodes.map(q => [q.q.x + "," + q.q.y + "," + q.q.net, q]));
  const nom = (x, y, net) => { const p = parPos.get(x + "," + y + "," + net); return p && p.fp.ref + "." + p.q.n; };
  globalThis.LIAISONS = c.rats.map(r => ({depart: nom(r.x1, r.y1, r.net), arrivee: nom(r.x2, r.y2, r.net)}))
    .filter(l => l.depart && l.arrivee).slice(0, 20);
  return JSON.stringify({liaisons_restantes: c.rats.length, retenues: LIAISONS.length,
    U3: LIAISONS.filter(l => /^U3\./.test(l.depart) && /^U3\./.test(l.arrivee))});
})()

   → attendu : {"liaisons_restantes":38,"retenues":20,"U3":[{"depart":"U3.1","arrivee":"U3.3"}]}.
   Capture : la carte sans pistes, avec son chevelu.
4. Chronomètre l'appel. Lance cette commande TROIS fois (elle n'écrit rien) :

(() => {
  const t = performance.now();
  const R = cheminCommande({liaisons: globalThis.LIAISONS});
  const ms = Math.round(performance.now() - t);
  globalThis.CHEMIN_R = R;
  return JSON.stringify({duree_ms: ms, pas_mm: R.pas_mm, trouvees: R.routes.length,
    refus: R.resultats.filter(x => !x.trouve)});
})()

   → attendu, à chaque fois : duree_ms bien sous 8000 (300 à 500 au banc sous Linux) ;
   pas_mm 0.25 ; trouvees 19 ; un seul refus, liaison 10, « aucun chemin sur la grille de
   0.25 mm (isolation de GND… » ; aucune raison « temps épuisé ». Note les trois durées.
5. Pose les 19 routes par le routeur de la page (proposition, puis application en une étape) :

(() => {
  const cmd = routagePiloteCommandes().cao_router, routes = globalThis.CHEMIN_R.routes;
  const t = performance.now();
  const prop = cmd.proposer({routes});
  const ms = Math.round(performance.now() - t);
  cmd.appliquer({routes}, prop);
  S.dirty = false;   // un exemple : rien à perdre en fermant l'onglet
  return "routeur : " + ms + " ms\n" + prop.apres;
})()

   → attendu : « routeur : … ms », 19 lignes « route k (net) : posée », puis « DRC : 0 erreur
   avant → 2 après. », avec « Isolation piste/pastille U3.2 : 0.000 mm : la règle en exige
   0.200 mm » et « Paire USB : … mm découplés pour … mm admis » (D+ et D− posées chacune de leur
   côté : attendu ici). Capture de la carte routée.
6. Mesure l'entrée de la piste dans U3.2 (positif : la piste mord la pastille de tant de mm ;
   négatif : elle passe à cette distance) :

(() => {
  const fp = S.fps.find(f => f.ref === "U3"), q = padsWorld(fp).find(p => String(p.n) === "2");
  let prof = -Infinity, net = "";
  for (const t of S.tracks) {
    if (t.net === q.net || arcOf(t) || !padCuLayers(fp, q).includes(t.l)) continue;
    for (let s = 0; s <= 200; s++) {
      const x = t.x1 + (t.x2 - t.x1) * s / 200, y = t.y1 + (t.y2 - t.y1) * s / 200;
      const p = t.w / 2 - padDist(x, y, q);
      if (p > prof) { prof = p; net = t.net; }
    }
  }
  return JSON.stringify({pastille: "U3.2", net: q.net, piste_la_plus_proche: net,
    entree_mm: Math.round(prof * 1000) / 1000, liaisons_restantes: conn(true).unrouted});
})()

   → attendu : {"pastille":"U3.2","net":"GND","piste_la_plus_proche":"+5V","entree_mm":0.129,"liaisons_restantes":19}.
7. Cadre U3 (commande de lecture de la page, comme cao_vue) :

piloteVue("pcb", {cadrer: "U3"})

   → attendu : « Vue : cadré sur U3. », U3 sélectionné. Fais une capture resserrée sur U3 (zoom
   de capture, ou molette au-dessus de U3) : la piste +5V en diagonale qui mord sa pastille 2.
8. Relève le poste :

JSON.stringify({ua: navigator.userAgent, coeurs: navigator.hardwareConcurrency, memoire_go: navigator.deviceMemory})

Rapport, en plus du format commun : les trois duree_ms ; la durée du routeur ; le texte complet
rendu à l'étape 5 ; le JSON de l'étape 6 ; celui de l'étape 8.
```

## C'est réussi si

- [ ] PC : un appel de 20 liaisons dure moins de 8 s (trois durées notées), sans « temps épuisé »
- [ ] PC : 19 routes posées par le routeur de la page, DRC relevé
- [ ] PC : l'entrée dans U3.2 est notée (reproduite, avec sa profondeur, ou non)
- [ ] Pi : les mêmes chiffres, dans le Chromium du Pi (« Ce qui reste à toi »)

## Ce qui reste à toi

La même mesure dans le **Chromium du Pi**, à la maison (écran branché ou bureau à distance ; une
page ouverte sur l'iPad mesurerait l'iPad). Pas de copie de PROJETS : rien n'est enregistré.

1. Démarre WEB·SUITE sur le Pi (`./demarrer_WEB_SUITE.sh`), **▶ Lancer** sur WEB_CAO. Contrôle
   que le coût de via revu y est : `git -C WEB_CAO merge-base --is-ancestor 6c50425 HEAD && echo ok`.
2. Chromium du Pi : `http://127.0.0.1:8000/editeur-pcb/editeur-pcb.html`, F12 → Console (au
   premier collage, Chromium demande de taper `allow pasting`).
3. Colle une à une les commandes des étapes 3 à 7 de la consigne, et compare aux attendus.
4. Note le modèle et la mémoire du Pi : `cat /proc/device-tree/model ; free -m`.

Si ça rate, note : la sortie de chaque commande, telle quelle ; les lignes rouges de la console ;
`chrome://version`.

## Après

Pour Claude Code :
- [ ] PC fait : `A-FAIRE.md:23-24`, ajouter la durée dans Chrome sous Windows (référence)
- [ ] Pi fait : `tasks/todo-chemin-auto.md:25-28` `[~]` → `[x]` avec la durée sur le Pi ;
      `A-FAIRE.md:11` retirer « Reste : l'essai réel sur le Pi » ; `:24` « Reste la durée sur le
      Pi. » → la durée ; `:39-40` retirer « l'essai réel sur le Pi reste… »
- [ ] Pi fait : `WEB_CAO/A-FAIRE.md:141` et `:48` : retirer « la durée de `cao_chemin` sur le Pi »
- [ ] `WEB_CAO/A-FAIRE.md:136-140` (routeur dans une pastille) : **ne pas cocher** ; ajouter ce
      que le PC (et le Pi) ont montré ; la correction se décide avec toi
