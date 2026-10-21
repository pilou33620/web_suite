# Essai : le tutoriel de la Gestion LIB (Chrome du PC, puis Chromium du Pi)

Qui : les deux · Où : Chrome du PC Windows du boulot, puis Chromium du Pi avec son écran ·
Durée : 15 min chacun · Source : `A-FAIRE.md:74-75`

## Pourquoi

`A-FAIRE.md:74` : « son tutoriel ne passe pas en Chromium headless sur le Pi », sans dire à
quelle étape ni avec quel message. Le relevé vient des essais du 19/10/2026
(`tasks/todo-composants.md`, tâche 7), où la Gestion LIB montait à environ 890 Mo au
chargement sur le Pi, jusqu'à faire tuer l'onglet par le noyau (OOM).

L'essai dit si c'est la Gestion LIB ou le mode headless qui pèche : d'abord dans le Chrome du PC
(la référence, sur la vraie LIB), puis dans un vrai Chromium du Pi.

Le tutoriel (`WEB_CAO/commun/tuto.js:117-136`, chargé par `gestion-lib/gestion-lib.html:412`) :
neuf étapes, chacune une bulle posée sur un élément. Seules deux actions changent la page, des
clics d'onglet (`lib("composants")`, `lib("pcb")`, l.47). Rien n'est écrit : l'étape
« Enregistrer » montre le bouton sans le presser. Le **🎓 Mode tuto** de l'accueil WEB_CAO est
gardé par le navigateur (`localStorage`, clé `cao.tuto`) : la consigne le remet comme il était.

Rejoué le 21/10/2026 dans Chromium headless (Playwright, fenêtre 1400 × 900, LIB vide) avec les
commandes de la consigne : les neuf étapes passent et mènent à la visionneuse ; mais aux étapes
7 à 9 le bouton visé est **hors de l'écran** (la barre d'outils déborde à droite), et à l'étape 6
le cadre est plus étroit que la vue des empreintes (1247 px pour 1400). À voir sur le PC, à sa
largeur de fenêtre.

## Ce que tu prépares

1. Double-clic sur `demarrer_WEB_SUITE.cmd` : le lanceur s'ouvre sur `http://127.0.0.1:8100`.
2. Ouvre Claude in Chrome et colle la consigne.
3. Si tu veux le chiffre comparable aux 890 Mo du Pi : ouvre le gestionnaire de tâches de Chrome
   (**Maj+Échap**) et note la colonne « Empreinte mémoire » de l'onglet Gestion LIB pendant le
   chargement (pic) puis une fois stable. Claude in Chrome ne voit que le tas JavaScript.

## Consigne pour Claude in Chrome

```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-pi-tutoriel-gestion-lib.md. Dérouler le tutoriel de la Gestion LIB de WEB_CAO
jusqu'au bout et relever, à chaque étape, si le cadre tombe sur le bon élément, et la mémoire.
Le tutoriel ne fait que montrer : ne clique jamais toi-même « Enregistrer », « Auto-associer »
ni « Importer » dans la Gestion LIB. Les commandes JavaScript se collent telles quelles.

1. Ouvre http://127.0.0.1:8100 et relève les versions des outils. Si la carte WEB_CAO dit
   qu'il est arrêté, clique ▶ Lancer sur elle.
   → attendu : WEB_CAO lancé, port 8000 (sinon, prends le port de sa carte).
2. Ouvre http://127.0.0.1:8000/ (l'accueil WEB_CAO) et relève l'état du mode tuto :

document.getElementById("tutoBascule").getAttribute("aria-pressed")

   Note la réponse (« true » ou « false ») : tu le remettras ainsi à la fin. Si c'est « false »,
   clique le bouton « 🎓 Mode tuto ».
   → attendu : le bouton allumé ; la commande rend « true ».
3. Clique la carte « Gestion LIB » : elle s'ouvre dans un nouvel onglet,
   gestion-lib/gestion-lib.html?tuto. Passe sur cet onglet (si tu n'y arrives pas, ouvre
   toi-même http://127.0.0.1:8000/gestion-lib/gestion-lib.html?tuto dans un onglet neuf). Dès
   qu'il répond, lance le relevé de mémoire :

(() => {
  globalThis.ESSAI_MEM = {pic: 0, dernier: 0};
  const f = () => { const m = performance.memory; if (!m) return; const v = m.usedJSHeapSize / 1048576; ESSAI_MEM.pic = Math.max(ESSAI_MEM.pic, v); ESSAI_MEM.dernier = v; };
  f(); setInterval(f, 250);
  return performance.memory ? "relevé lancé : " + Math.round(ESSAI_MEM.dernier) + " Mo de tas JavaScript" : "performance.memory absent";
})()

   → attendu : le catalogue se charge ; une bulle « Mode tuto · 4 / 5 », « Gestion LIB »,
   « 9 étapes pour en faire le tour… », boutons « Plus tard » / « Commencer › ».
4. Attends 10 s que le catalogue soit chargé, puis lance la commande de contrôle (tu la
   relanceras à chaque étape) :

(() => {
  const b = document.querySelector(".tuto-bulle"), s = document.querySelector(".tuto-spot");
  if (!b || b.hidden) return "pas de bulle";
  const CIBLE = {"Quatre vues": ".nav-tabs", "Associations": ".stats-band", "Chercher": "#searchComp",
    "L'inspecteur": "#inspectorPanel", "Empreintes": "#viewPcb", "Importer JLCPCB / LCSC": "#bImportJlc",
    "Auto-associer": "#bAutoAssoc", "Enregistrer": "#bSave"};
  const titre = (b.querySelector("h4") || {}).textContent, c = CIBLE[titre] && document.querySelector(CIBLE[titre]);
  const R = e => { const x = e.getBoundingClientRect(); return {g: Math.round(x.left), h: Math.round(x.top), d: Math.round(x.right), b: Math.round(x.bottom)}; };
  const dedans = r => r.g >= 0 && r.h >= 0 && r.d <= innerWidth && r.b <= innerHeight;
  const rs = s.hidden ? null : R(s), rc = c ? R(c) : null, rb = R(b), m = globalThis.ESSAI_MEM;
  return JSON.stringify({entete: (b.querySelector(".tuto-num") || {}).textContent, titre,
    cadre: rs ? "oui" : "non",
    cadre_sur_cible: rs && rc ? ["g", "h", "d", "b"].every(k => Math.abs(rs[k] - rc[k]) <= 6) : null,
    cible_dans_ecran: rc ? dedans(rc) : null, bulle_dans_ecran: dedans(rb),
    ecran: innerWidth + " x " + innerHeight, cadre_px: rs, cible_px: rc,
    tas_mo: m ? Math.round(m.dernier) : null, pic_mo: m ? Math.round(m.pic) : null});
})()

   → attendu : entete « Mode tuto · 4 / 5 » ; note tas_mo et pic_mo.
5. Clique « Commencer › ». Puis, à chaque étape : attends 1 s, lance la commande de contrôle,
   fais une capture, et clique « Suivant › ». Titres attendus, « Gestion LIB · étape i / 9 » en tête :
   1 « Bienvenue dans la gestion LIB » (bulle au centre, cadre « non ») ; 2 « Quatre vues » ;
   3 « Associations » ; 4 « Chercher » ; 5 « L'inspecteur » ; 6 « Empreintes » (la vue passe
   aux empreintes PCB) ; 7 « Importer JLCPCB / LCSC » (retour au catalogue) ; 8 « Auto-associer » ;
   9 « Enregistrer », avec « Suite du parcours : Visionneuse IPC-2581. ».
   → attendu, étapes 2 à 9 : cadre « oui », cadre_sur_cible true, cible_dans_ecran true,
   bulle_dans_ecran true. Relève tel quel tout écart (au banc headless, étapes 7 à 9 :
   cible_dans_ecran false ; étape 6 : cadre_sur_cible false). Note aussi si la page ne répond
   plus, ou si Chrome affiche sa page d'onglet planté (« Oh non ! » / « Aïe, aïe, aïe »).
6. À l'étape 9, avant de quitter la page, lance la commande de contrôle une dernière fois
   (pour le tas et le pic), puis :

LIB_STATE.sale

   → attendu : false. Si c'est true, arrête-toi : une fenêtre « quitter la page ? » bloquerait.
   Clique alors « Visionneuse IPC-2581 › ».
   → attendu : la visionneuse IPC-2581 s'ouvre dans le même onglet, avec sa bulle « Mode tuto ·
   5 / 5 », « Visionneuse IPC-2581 ». Capture, puis clique « Plus tard ».
7. Reviens en arrière (bouton Précédent du navigateur) à la Gestion LIB, recharge-la, attends
   qu'elle soit chargée, puis :

[...document.querySelectorAll(".tuto-pastille")].filter(p => !p.hidden).length

   → attendu : un nombre de pastilles « ? » (7 au banc). Survole celle du champ de recherche :
   une bulle redonne l'explication « Chercher ». Capture.
8. Si le mode tuto était « false » à l'étape 2 : retourne sur l'onglet de l'accueil WEB_CAO,
   clique « 🎓 Mode tuto », et vérifie avec la commande de l'étape 2 qu'il rend « false ».

Rapport, en plus du format commun : le JSON de chaque commande de contrôle (accueil du tuto et
étapes 1 à 9) ; le tas et le pic à la fin ; le nombre de pastilles ; l'état du mode tuto au début
et à la fin ; les erreurs de la console de la Gestion LIB.
```

## C'est réussi si

- [ ] PC : les neuf étapes défilent jusqu'au bout ; les écarts de cadre ou de cible hors écran sont relevés, étape par étape
- [ ] PC : l'onglet n'est pas planté, la page reste utilisable pendant et après
- [ ] PC : la dernière étape mène à la visionneuse
- [ ] PC : tas JavaScript et pic notés (et, si tu l'as fait, l'empreinte mémoire du gestionnaire de tâches)
- [ ] PC : le mode tuto est remis comme il était
- [ ] Pi : les mêmes points dans le vrai Chromium du Pi (« Ce qui reste à toi »)

## Ce qui reste à toi

Dans le **Chromium du Pi, avec son écran** (branché ou bureau à distance), pas en headless.

1. Démarre WEB·SUITE sur le Pi, **▶ Lancer** sur WEB_CAO. Ferme les autres onglets lourds ;
   ouvre le gestionnaire de tâches de Chromium (**Maj+Échap**) ; `free -m`, note la ligne `Mem:`.
2. Déroule la consigne à la main (étapes 2 à 8), les commandes dans la console (F12 → Console,
   `allow pasting` au premier collage). Note le pic et la valeur stable de l'« Empreinte
   mémoire » de l'onglet Gestion LIB, puis `free -m` de nouveau.

Si ça rate, note : l'étape (numéro et titre) et ce qu'on voit (bulle absente, cadre au mauvais
endroit, page figée, onglet tué), en capture ; les lignes rouges de la console ; le gestionnaire
de tâches et `free -m` au moment du blocage ; si l'onglet a été tué :
`journalctl -k --since "10 min ago" | grep -i -E "oom|killed process"` ; `cat /proc/device-tree/model`,
`free -h`, `chrome://version` ; le journal de WEB_CAO (bouton **Journal** de sa carte).

## Après

Pour Claude Code :
- [ ] Un écart de cadre ou une cible hors écran relevé sur le PC : l'ajouter à `WEB_CAO/A-FAIRE.md`
      (défaut du tutoriel, `commun/tuto.js`), qui n'en parle pas encore
- [ ] Le tutoriel passe sur le Pi : `A-FAIRE.md:74-75`, écrire que dans un vrai Chromium du Pi il
      passe (date, mémoire notée) et que seul le headless pèche ; cocher si le headless ne compte
      pas, sinon laisser la case pour lui
- [ ] Il casse sur le Pi : `A-FAIRE.md:74-75`, remplacer « ne passe pas en Chromium headless » par
      l'étape et le symptôme ; l'ajouter aussi à `WEB_CAO/A-FAIRE.md`
