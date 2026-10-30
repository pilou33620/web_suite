/* Banc de pilote.js (canal-pilote, côté page) : plusieurs « onglets » dans un même
   processus Node, reliés par le vrai BroadcastChannel de Node, face à un faux serveur
   du pilote (file de commandes et long-poll, comme pilote.py).

       node lanceur/banc_pilote.js

   Affiche OK si tout passe. */
"use strict";
const assert = require("assert");
const { creer, touches } = require("./pilote.js");

/* silence bien plus long que l'amorce, comme en vrai (90 s contre 5 s) : l'élection ne doit
   attendre que l'amorce, et un relais qui se ferme proprement (« bye ») est remplacé aussitôt. */
const REGLAGES = { annonce: 40, amorce: 160, silence: 3000, battement: 120, attenteServeur: 80, repos: 30 };
const pause = ms => new Promise(r => setTimeout(r, ms));
const ENTETE = { "X-WEB_CAO": "1" };   // l'en-tête maison que la page de l'outil pose (ENTETE_WEB)
const avecEntete = p => p.entetes["X-WEB_CAO"] === "1" && p.entetes["Content-Type"] === "application/json";

/* Faux serveur du pilote : il note les battements, compte les long-polls en cours et
   rend les commandes au navigateur de l'onglet visé, comme pilote.py. */
function fauxServeur() {
  const s = { battements: [], file: [], reponses: [], enCours: 0, maxEnCours: 0, polls: [], balises: [], postes: [],
              inconnues: new Set(), tropGros: new Set(), demandes: [], lectures: [], conversation: [],
              statut: { lanceur: true, claude: { pret: true, message: "Claude Code 9.9 connecté.", version: "9.9" } } };
  s.fetch = async (url, opts = {}) => {
    const u = new URL(url, "http://outil");
    const corps = opts.body ? JSON.parse(opts.body) : null;
    if (opts.method === "POST") s.postes.push({ route: u.pathname, entetes: opts.headers || {} });
    const rendre = obj => ({ ok: true, status: 200, json: async () => obj });
    if (u.pathname === "/api/pilote/battement") { s.battements.push(corps); return rendre({ ok: true }); }
    if (u.pathname === "/api/pilote/reponse") {
      s.reponses.push(corps);
      if (s.inconnues.has(corps.id)) return { ok: false, status: 404, json: async () => ({}) };
      if (s.tropGros.has(corps.id) && ["finie", "appliquee"].includes(corps.etat))
        return { ok: false, status: 413, json: async () => ({}) };
      return rendre({ ok: true });
    }
    if (u.pathname === "/api/pilote/attendre") {
      const nav = u.searchParams.get("navigateur");
      s.polls.push(nav);
      s.enCours++; s.maxEnCours = Math.max(s.maxEnCours, s.enCours);
      try {
        for (let t = 0; t < REGLAGES.attenteServeur; t += 10) {
          if (opts.signal && opts.signal.aborted) throw new Error("abort");
          const prets = s.file.filter(c => c.navigateur === nav);
          if (prets.length) { s.file = s.file.filter(c => c.navigateur !== nav); return rendre({ commandes: prets }); }
          await pause(10);
        }
        return rendre({ commandes: [] });
      } finally { s.enCours--; }
    }
    // Demandes écrites : le faux serveur rend la conversation préparée, une réponse par lecture
    if (u.pathname === "/api/pilote/demande") {
      s.demandes.push(corps);
      if (s.refusDemande) return { ok: false, status: 409, json: async () => ({ detail: s.refusDemande }) };
      return rendre({ id: "d" + s.demandes.length });
    }
    if (u.pathname === "/api/pilote/statut") return rendre(s.statut);
    if (u.pathname === "/api/pilote/conversation") {
      s.lectures.push(Object.fromEntries(u.searchParams));
      await pause(20);
      const suite = s.conversation.shift();
      if (suite === "coupure") throw new Error("réseau coupé");
      return rendre(suite || { etat: "finie", messages: [] });
    }
    return { ok: false, status: 404, json: async () => ({ detail: "inconnue" }) };
  };
  s.deposer = (onglet, nom, args = {}, ecriture = false) => {
    const c = { id: "c" + Math.random().toString(16).slice(2), onglet, nom, args, ecriture, navigateur: s.nav };
    s.file.push(c);
    return c.id;
  };
  s.reponsesDe = id => s.reponses.filter(r => r.id === id).map(r => r.etat);
  s.finale = id => s.reponses.find(r => r.id === id && ["finie", "echouee", "appliquee", "refusee"].includes(r.etat));
  return s;
}

async function attendreQue(cond, ms = 2000, quoi = "condition") {
  for (let t = 0; t < ms; t += 10) { if (cond()) return; await pause(10); }
  assert.fail("délai dépassé : " + quoi);
}

function stockagePartage() {
  const m = new Map();
  return { getItem: k => (m.has(k) ? m.get(k) : null), setItem: (k, v) => m.set(k, String(v)) };
}

/* DOM factice : juste ce que pilote.js utilise. innerHTML lève une erreur : un texte venu
   de Claude ne doit jamais être interprété comme du HTML. */
function domFactice() {
  class El {
    constructor(tag) { this.tagName = tag; this.enfants = []; this.ecoutes = {}; this.texte = ""; this.classes = new Set();
                       this.attributs = {}; this.style = {}; this.type = ""; this.checked = false; this.parent = null; }
    set innerHTML(_) { throw new Error("innerHTML interdit"); }
    set textContent(v) { this.texte = String(v); this.enfants = []; }
    get textContent() { return this.texte + this.enfants.map(e => e.textContent).join(""); }
    get className() { return [...this.classes].join(" "); }
    set className(v) { this.classes = new Set(String(v).split(/\s+/).filter(Boolean)); }
    get classList() { const c = this.classes; return { add: k => c.add(k), remove: k => c.delete(k), contains: k => c.has(k),
                                                       toggle: (k, f) => ((f === undefined ? !c.has(k) : f) ? c.add(k) : c.delete(k)) }; }
    appendChild(e) { if (e.parent) e.parent.enfants = e.parent.enfants.filter(x => x !== e); this.enfants.push(e); e.parent = this; return e; }
    replaceChildren(...e) { this.enfants = []; e.forEach(x => this.appendChild(x)); }
    remove() { if (this.parent) this.parent.enfants = this.parent.enfants.filter(x => x !== this); this.parent = null; }
    setAttribute(k, v) { this.attributs[k] = String(v); }
    addEventListener(t, f) { (this.ecoutes[t] = this.ecoutes[t] || []).push(f); }
    dispatch(t) { (this.ecoutes[t] || []).forEach(f => f({ target: this })); }
    click() { if (!this.disabled) this.dispatch("click"); }
  }
  return { body: new El("body"), createElement: t => new El(t) };
}
function trouver(el, test, acc = []) { if (test(el)) acc.push(el); el.enfants.forEach(e => trouver(e, test, acc)); return acc; }
function texteDe(el) { return el.textContent; }
function panneauOuvert(doc) { const p = trouver(doc.body, e => e.classes.has("pilote"))[0]; return !!p && p.classes.has("ouvert"); }

function onglet(serveur, stockage, page, nomCanal) {
  const balise = (url, corps) => { serveur.balises.push({ url, corps: JSON.parse(corps) }); return true; };
  return creer({ fetch: serveur.fetch, BroadcastChannel, stockage, canal: nomCanal, balise, entete: ENTETE }, REGLAGES).declarer({
    page, projet: () => "carte PIR" });
}

(async () => {
  const canal = "banc." + Math.random();
  const serveur = fauxServeur();
  const stockage = stockagePartage();
  const A = onglet(serveur, stockage, "pcb", canal);
  const B = onglet(serveur, stockage, "schema", canal);
  const C = onglet(serveur, stockage, "pcb", canal);
  serveur.nav = stockage.getItem("websuite.pilote.navigateur");
  assert.ok(/^[0-9a-f]{24}$/.test(serveur.nav), "id de navigateur partagé et aléatoire : " + serveur.nav);
  assert.ok([A, B, C].every(o => /^[0-9a-f]{24}$/.test(o.id)) && new Set([A.id, B.id, C.id]).size === 3);

  // Un seul relais (le plus petit id), un seul long-poll à la fois
  await pause(REGLAGES.amorce * 2);
  const relais = [A, B, C].filter(o => o.relais);
  assert.strictEqual(relais.length, 1, "un seul relais");
  assert.strictEqual(relais[0].id, [A.id, B.id, C.id].sort()[0], "le plus petit id");
  assert.strictEqual(serveur.maxEnCours, 1, "jamais deux long-polls en même temps");
  assert.ok(serveur.polls.every(n => n === serveur.nav), "les polls portent l'id du navigateur");

  // Le battement liste les trois onglets, avec page, projet et focus relatif
  await attendreQue(() => serveur.battements.some(b => b.onglets.length === 3), 2000, "battement à 3 onglets");
  const bat = serveur.battements.filter(b => b.onglets.length === 3).pop();
  assert.strictEqual(bat.navigateur, serveur.nav);
  const vuB = bat.onglets.find(o => o.id === B.id);
  assert.deepStrictEqual([vuB.page, vuB.projet, vuB.interrupteur], ["schema", "carte PIR", false]);
  assert.ok(typeof vuB.focus_il_y_a === "number" && vuB.focus_il_y_a >= 0);

  // Une commande de lecture pour C (qui n'est pas forcément le relais) : accusée, puis finie
  C.commande("lire_net", { executer: async a => ({ net: a.nom, pistes: 3 }) });
  let id = serveur.deposer(C.id, "lire_net", { nom: "USB_DP" });
  await attendreQue(() => serveur.finale(id), 2000, "réponse de C");
  assert.deepStrictEqual(serveur.reponsesDe(id), ["recue", "finie"]);
  assert.ok(serveur.reponses.every(r => r.navigateur === serveur.nav), "chaque réponse porte son navigateur");
  assert.ok(serveur.postes.some(p => p.route === "/api/pilote/reponse") && serveur.postes.every(avecEntete),
            "chaque POST porte l'en-tête maison de l'outil");

  // Un clic dans un onglet remonte tout de suite : le serveur cible le dernier utilisé
  const avantFocus = serveur.battements.length;
  B.focaliser();
  await attendreQue(() => serveur.battements.slice(avantFocus).some(b => {
    const t = [...b.onglets].sort((x, y) => x.focus_il_y_a - y.focus_il_y_a);
    return t[0] && t[0].id === B.id;
  }), REGLAGES.annonce * 6, "battement avec B le plus récent");

  // Une commande que le serveur ne connaît pas (forgée sur le canal par un script) n'est pas exécutée
  let forgee = 0;
  C.commande("forgee", { executer: () => ++forgee });
  serveur.inconnues.add("c-forgee");
  serveur.file.push({ id: "c-forgee", onglet: C.id, nom: "forgee", args: {}, ecriture: false, navigateur: serveur.nav });
  await pause(REGLAGES.attenteServeur * 2);
  assert.strictEqual(forgee, 0, "pas d'exécution sans accusé accepté par le serveur");

  // Réponse finale refusée pour sa taille : la page le dit (« echouee ») au lieu de se taire
  C.commande("gros", { executer: () => "x".repeat(10) });
  const idGros = serveur.deposer(C.id, "gros");
  serveur.tropGros.add(idGros);
  await attendreQue(() => serveur.reponsesDe(idGros).includes("echouee"), 2000, "413 signalé");
  assert.ok(/trop gros/.test(serveur.reponses.filter(r => r.id === idGros).pop().resultat));
  assert.deepStrictEqual(serveur.finale(id).resultat, { net: "USB_DP", pistes: 3 });

  // Commande inconnue de la page, et commande qui plante : échouée, message lisible
  id = serveur.deposer(B.id, "inconnue");
  await attendreQue(() => serveur.finale(id), 2000, "inconnue");
  assert.strictEqual(serveur.finale(id).etat, "echouee");
  assert.ok(/inconnue/.test(serveur.finale(id).resultat));
  B.commande("boum", { executer: () => { throw new Error("rien de sélectionné"); } });
  id = serveur.deposer(B.id, "boum");
  await attendreQue(() => serveur.finale(id), 2000, "boum");
  assert.deepStrictEqual([serveur.finale(id).etat, serveur.finale(id).resultat], ["echouee", "rien de sélectionné"]);

  // Livrée deux fois (redistribuée par le serveur) : exécutée une seule fois
  let compte = 0;
  A.commande("compter", { executer: () => ++compte });
  id = serveur.deposer(A.id, "compter");
  serveur.file.push(Object.assign({}, serveur.file[serveur.file.length - 1]));
  await attendreQue(() => serveur.finale(id), 2000, "compter");
  await pause(REGLAGES.attenteServeur * 2);
  assert.strictEqual(compte, 1, "pas de double exécution");

  // Capture : PNG en data URL, réduite à 1600 px de large au plus
  const toile = (w, h) => ({ width: w, height: h, getContext: () => ({ drawImage() {} }),
                             toDataURL(t) { return "data:" + t + ";base64,QUJD" + this.width; } });
  A.capture(() => toile(3200, 1000));
  const env2 = { document: { createElement: () => toile(0, 0) } };
  A._env.document = env2.document;
  id = serveur.deposer(A.id, "capture");
  await attendreQue(() => serveur.finale(id), 2000, "capture");
  const cap = serveur.finale(id).resultat;
  assert.deepStrictEqual([cap.largeur, cap.hauteur], [1600, 500]);
  assert.strictEqual(cap.image, "data:image/png;base64,QUJD1600");
  A.capture(() => toile(1000, 4000));
  id = serveur.deposer(A.id, "capture");
  await attendreQue(() => serveur.finale(id), 2000, "capture haute");
  assert.deepStrictEqual([serveur.finale(id).resultat.largeur, serveur.finale(id).resultat.hauteur], [400, 1600]);
  id = serveur.deposer(B.id, "capture");
  await attendreQue(() => serveur.finale(id), 2000, "capture sans canevas");
  assert.ok(/canevas/.test(serveur.finale(id).resultat) && serveur.finale(id).etat === "echouee");

  // Geste « interface » : les touches se lisent comme on les écrit ; un geste inconnu échoue avant toute carte
  const k = t => { const o = touches(t); return [o.key, o.code, o.ctrlKey, o.shiftKey]; };
  assert.deepStrictEqual(k("Ctrl+Z"), ["z", "KeyZ", true, false]);
  assert.deepStrictEqual(k("Ctrl+Shift+z"), ["Z", "KeyZ", true, true]);
  assert.deepStrictEqual(k("Delete"), ["Delete", "Delete", false, false]);
  assert.deepStrictEqual(k("Ctrl++"), ["+", "+", true, false]);
  assert.deepStrictEqual(k("2"), ["2", "Digit2", false, false]);
  assert.throws(() => touches("Hyper+x"), /Modificateur/);
  id = serveur.deposer(A.id, "interface", { geste: "voler" }, true);
  await attendreQue(() => serveur.finale(id), 2000, "geste inconnu");
  assert.ok(serveur.finale(id).etat === "echouee" && /geste/.test(serveur.finale(id).resultat));

  // Une écriture se déclare avec un moyen d'annuler : historique de l'éditeur, ou annuler + etat
  assert.throws(() => A.commande("ecrire", { ecriture: true, appliquer() {} }), /annuler/);
  assert.throws(() => A.commande("ecrire", { ecriture: true, appliquer() {}, annuler() {} }), /etat/);
  A.commande("ecrire_h", { ecriture: true, historique: true, proposer: () => ({}), appliquer() {} });
  A.commande("ecrire_a", { ecriture: true, proposer: () => ({}), appliquer() {}, annuler() {}, etat: () => 1 });

  // Le relais se ferme : un autre est élu, et une commande déposée entre-temps aboutit
  const ancien = relais[0];
  const restants = [A, B, C].filter(o => o !== ancien);
  ancien.arreter();
  // Le relais qui se ferme prévient le serveur tout de suite (balise), sans attendre l'oubli
  const adieu = serveur.balises.pop();
  assert.ok(adieu && adieu.url === "/api/pilote/battement", "battement d'adieu envoyé par balise");
  assert.strictEqual(adieu.corps.navigateur, serveur.nav);
  assert.ok(!adieu.corps.onglets.some(o => o.id === ancien.id) && adieu.corps.onglets.length === 2,
            "l'adieu liste les autres onglets, pas celui qui se ferme");
  const cible = restants[restants.length - 1];
  cible.commande("apres", { executer: () => "toujours là" });
  id = serveur.deposer(cible.id, "apres");
  await attendreQue(() => restants.some(o => o.relais), REGLAGES.amorce + REGLAGES.annonce * 3, "nouveau relais");
  await attendreQue(() => serveur.finale(id), 2000, "commande après changement de relais");
  assert.strictEqual(serveur.finale(id).resultat, "toujours là");
  await attendreQue(() => {
    const b = serveur.battements[serveur.battements.length - 1];
    return b && b.onglets.length === 2 && !b.onglets.some(o => o.id === ancien.id);
  }, 2000, "battement sans l'onglet fermé");

  // ---- Tâche 5 : panneau « Claude », cartes, interrupteur, annuler -----------------
  const D = creer({ fetch: serveur.fetch, BroadcastChannel, stockage, canal, document: domFactice(), entete: ENTETE }, REGLAGES)
    .declarer({ page: "antenna", projet: () => "dipole" });
  const etat = { v: "1" };
  let annule = 0;
  const refusees = [];
  D.commande("regler", {
    ecriture: true,
    proposer: a => ({ titre: "<img src=x onerror=alert(1)>", avant: etat.v, apres: a.v }),
    appliquer: a => { const avant = etat.v; etat.v = a.v; return avant; },
    annuler: (a, avant) => { etat.v = avant; annule++; },
    etat: () => etat.v,
    refuser: (a, prop) => { refusees.push([a.v, prop.apres]); },
  });
  const doc = D._env.document;
  // Le bouton « Claude » et l'interrupteur sont là dès le chargement, avant toute carte :
  // sinon on ne pourrait pas allumer l'interrupteur à l'avance (vu à l'essai réel)
  assert.strictEqual(trouver(doc.body, e => e.classes.has("pilote")).length, 1, "panneau construit au chargement");
  assert.ok(trouver(doc.body, e => e.tagName === "input" && e.type === "checkbox").length === 1, "interrupteur présent");
  assert.ok(!panneauOuvert(doc), "replié au chargement");
  const boutons = () => trouver(doc.body, e => e.tagName === "button");
  const bouton = texte => boutons().find(b => b.textContent === texte);
  await attendreQue(() => restants.concat(D).some(o => o.relais), 2000, "relais avec D");

  // × masque le panneau, et le masquage est gardé ; une carte à valider le fait revenir (plus bas)
  const panneauD = trouver(doc.body, e => e.classes.has("pilote"))[0];
  bouton("×").click();
  assert.ok(panneauD.classes.has("masque"), "× masque le panneau");
  assert.strictEqual(stockage.getItem("websuite.pilote.masque"), "1", "masquage gardé");
  // Masqué, il reste une pastille « Claude » : un clic le rouvre (vu à l'essai sous Windows : sans elle,
  // le panneau ne revenait plus)
  trouver(doc.body, e => e.classes.has("pilote-tete"))[0].click();
  assert.ok(!panneauD.classes.has("masque") && panneauOuvert(doc), "la pastille rouvre le panneau");
  assert.strictEqual(stockage.getItem("websuite.pilote.masque"), "0");
  bouton("×").click();
  assert.ok(panneauD.classes.has("masque") && !panneauOuvert(doc), "× replie et réduit");

  // Proposée : rien n'est appliqué avant le clic ; la carte montre le texte tel quel
  id = serveur.deposer(D.id, "regler", { v: "2" }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte proposée");
  assert.strictEqual(etat.v, "1", "rien appliqué avant le clic");
  assert.ok(texteDe(doc.body).includes("<img src=x onerror=alert(1)>"), "titre affiché tel quel");
  assert.ok(texteDe(doc.body).includes("1") && texteDe(doc.body).includes("2"), "avant et après affichés");
  assert.ok(panneauOuvert(doc), "le panneau s'ouvre à l'arrivée d'une carte");
  assert.ok(!panneauD.classes.has("masque"), "une carte à valider fait revenir le panneau masqué");
  bouton("Appliquer").click();
  await attendreQue(() => serveur.finale(id), 2000, "appliquée");
  assert.deepStrictEqual([serveur.finale(id).etat, etat.v], ["appliquee", "2"]);

  // Annuler : refusé si l'état a bougé depuis ; accepté s'il est revenu tel qu'appliqué
  etat.v = "3";
  bouton("Annuler").click();
  assert.deepStrictEqual([etat.v, annule], ["3", 0], "l'état a bougé : pas d'annulation");
  assert.ok(/Ctrl\+Z|changé/.test(texteDe(doc.body)), "la carte dit pourquoi");
  etat.v = "2";
  bouton("Annuler").click();
  assert.deepStrictEqual([etat.v, annule], ["1", 1], "annulé");

  // Refusée : rien d'appliqué, « refusee » rendu
  id = serveur.deposer(D.id, "regler", { v: "9" }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "deuxième carte");
  bouton("Refuser").click();
  await attendreQue(() => serveur.finale(id), 2000, "refusée");
  assert.deepStrictEqual([serveur.finale(id).etat, etat.v], ["refusee", "1"]);
  assert.deepStrictEqual(refusees, [["9", "9"]], "le crochet refuser de la commande est appelé, avec ses arguments et sa carte");

  // Interrupteur allumé : appliquée sans clic, la trace reste ; il est éteint au départ
  const inter = trouver(doc.body, e => e.tagName === "input" && e.type === "checkbox")[0];
  assert.ok(inter && inter.checked === false && D.interrupteur === false, "interrupteur éteint au chargement");
  inter.checked = true;
  inter.dispatch("change");
  assert.strictEqual(D.interrupteur, true);
  id = serveur.deposer(D.id, "regler", { v: "5" }, true);
  await attendreQue(() => serveur.finale(id), 2000, "appliquée directement");
  assert.deepStrictEqual([serveur.finale(id).etat, etat.v], ["appliquee", "5"]);
  assert.ok(!serveur.reponsesDe(id).includes("proposee"), "pas de carte à valider");
  assert.ok(/directement/.test(texteDe(doc.body)), "trace de l'application directe");
  // « clic » : même interrupteur allumé, la carte attend le clic (création dans la LIB partagée)
  let creees = 0;
  D.commande("creer", { ecriture: true, clic: true, irreversible: "Écrit dans la LIB.", proposer: a => ({ titre: a.v, apres: a.v }),
                        appliquer: () => { creees++; return "LIB/x.json"; } });
  id = serveur.deposer(D.id, "creer", { v: "x" }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte à clic malgré l'interrupteur");
  assert.strictEqual(creees, 0, "rien d'écrit avant le clic, interrupteur allumé");
  boutons().filter(b => b.textContent === "Appliquer").pop().click();
  await attendreQue(() => serveur.finale(id), 2000, "carte à clic appliquée");
  assert.deepStrictEqual([serveur.finale(id).etat, creees, serveur.finale(id).resultat.direct], ["appliquee", 1, false]);
  inter.checked = false;
  inter.dispatch("change");

  // Écriture par l'historique de l'éditeur : pas de bouton Annuler, renvoi au Ctrl+Z
  let pousse = 0;
  D.commande("valeur", { ecriture: true, historique: true, proposer: a => ({ titre: a.ref, avant: "1k", apres: a.v }),
                         appliquer: () => { pousse++; } });
  id = serveur.deposer(D.id, "valeur", { ref: "R1", v: "4k7" }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte historique");
  const avantClic = boutons().filter(b => b.textContent === "Annuler").length;
  boutons().filter(b => b.textContent === "Appliquer").pop().click();
  await attendreQue(() => serveur.finale(id), 2000, "historique appliquée");
  assert.strictEqual(pousse, 1);
  assert.strictEqual(boutons().filter(b => b.textContent === "Annuler").length, avantClic, "pas d'Annuler de carte");
  assert.ok(/Ctrl\+Z/.test(texteDe(doc.body)), "renvoi au Ctrl+Z");
  // Le clic n'a rien signalé ; un « appliquer » appelé sans carte (console) écrit quand même, et se voit
  assert.ok(!/sans carte ni clic/.test(texteDe(doc.body)), "le clic ne laisse pas de signalement");
  const avertir = console.warn; let averti = "";
  console.warn = m => { averti = m; };
  try { D.commandes.valeur.appliquer({ ref: "R1", v: "1k" }, {}); } finally { console.warn = avertir; }
  assert.strictEqual(pousse, 2, "pas une barrière : l'écriture a lieu");
  assert.ok(/« valeur » appliquée sans carte ni clic/.test(texteDe(doc.body)) && /sans carte ni clic/.test(averti),
            "un appliquer hors carte est signalé dans le panneau et la console");

  // Une proposition qui plante : échouée ; une lecture : trace, sans ouvrir le panneau
  D.commande("casse", { ecriture: true, historique: true, proposer: () => { throw new Error("R99 introuvable"); },
                        appliquer() {} });
  id = serveur.deposer(D.id, "casse", {}, true);
  await attendreQue(() => serveur.finale(id), 2000, "proposition qui plante");
  assert.deepStrictEqual([serveur.finale(id).etat, serveur.finale(id).resultat], ["echouee", "R99 introuvable"]);
  D.replier();
  D.commande("lire", { executer: () => "ok" });
  id = serveur.deposer(D.id, "lire");
  await attendreQue(() => serveur.finale(id), 2000, "lecture");
  assert.ok(/Claude a lancé : lire/.test(texteDe(doc.body)), "trace de la lecture");
  assert.ok(!panneauOuvert(doc), "une lecture n'ouvre pas le panneau");

  // appliquer reçoit la proposition affichée : l'adaptateur vérifie que rien n'a changé depuis
  let vuParAppliquer = null;
  D.commande("verifiee", { ecriture: true, historique: true, proposer: a => ({ titre: "t", apres: a.v, cle: 42 }),
                           appliquer: (a, prop) => { vuParAppliquer = prop; } });
  id = serveur.deposer(D.id, "verifiee", { v: "7" }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte vérifiée");
  boutons().filter(b => b.textContent === "Appliquer").pop().click();
  await attendreQue(() => serveur.finale(id), 2000, "vérifiée appliquée");
  assert.deepStrictEqual([vuParAppliquer.apres, vuParAppliquer.cle], ["7", 42]);

  // appliquer peut être asynchrone (ouvrir un fichier) : un rejet remonte en « echouee »,
  // une réussite en « appliquee » ; un double clic pendant l'attente ne rejoue rien
  let appels = 0;
  D.commande("lent", { ecriture: true, historique: true, proposer: a => ({ titre: "lent " + a.ok }),
                       appliquer: async a => { appels++; await pause(60); if (!a.ok) throw new Error("fichier illisible"); } });
  id = serveur.deposer(D.id, "lent", { ok: false }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte lente");
  boutons().filter(b => b.textContent === "Appliquer").pop().click();
  await attendreQue(() => serveur.finale(id), 2000, "rejet asynchrone");
  assert.deepStrictEqual([serveur.finale(id).etat, serveur.finale(id).resultat], ["echouee", "fichier illisible"]);
  appels = 0;
  id = serveur.deposer(D.id, "lent", { ok: true }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte lente 2");
  const bLent = boutons().filter(b => b.textContent === "Appliquer").pop();
  bLent.click(); bLent.click();
  await attendreQue(() => serveur.finale(id), 2000, "réussite asynchrone");
  assert.deepStrictEqual([serveur.finale(id).etat, appels], ["appliquee", 1], "appliquée une seule fois");

  // Un seul côté donné (avant sans après) : pas de « → undefined » sur la carte
  D.commande("un_seul", { ecriture: true, historique: true, proposer: () => ({ titre: "un côté", avant: "position 0 ; 0 ; 0" }),
                         appliquer() {} });
  id = serveur.deposer(D.id, "un_seul", {}, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte à un côté");
  assert.ok(!/undefined/.test(texteDe(doc.body)) && /position 0 ; 0 ; 0/.test(texteDe(doc.body)));
  boutons().filter(b => b.textContent === "Refuser").pop().click();

  // proposer peut être asynchrone aussi (lire une liste avant de montrer la carte)
  D.commande("lister", { ecriture: true, historique: true,
                         proposer: async a => { await pause(30); if (!a.ok) throw new Error("liste illisible"); return { titre: "liste lue" }; },
                         appliquer() {} });
  id = serveur.deposer(D.id, "lister", { ok: true }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte après proposer async");
  assert.ok(/liste lue/.test(texteDe(doc.body)));
  boutons().filter(b => b.textContent === "Refuser").pop().click();
  id = serveur.deposer(D.id, "lister", { ok: false }, true);
  await attendreQue(() => serveur.finale(id), 2000, "proposer async qui rejette");
  assert.deepStrictEqual([serveur.finale(id).etat, serveur.finale(id).resultat], ["echouee", "liste illisible"]);

  // Écriture irréversible (ranger un fichier) : déclarée sans annuler, la carte prévient
  // avant le clic, n'offre pas Annuler, et rend le résultat à Claude
  assert.throws(() => D.commande("sans", { ecriture: true, proposer: () => ({}), appliquer() {} }), /annuler|irreversible/);
  D.commande("ranger", { ecriture: true, irreversible: "Crée un nouveau fichier ; non annulable ici.",
                         proposer: a => ({ titre: "Ranger " + a.nom }), appliquer: async a => ({ chemin: a.nom + " (2).stp" }) });
  id = serveur.deposer(D.id, "ranger", { nom: "bolt" }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte irréversible");
  assert.ok(/non annulable/.test(texteDe(doc.body)), "avertissement affiché avant le clic");
  const avantAnnuler = boutons().filter(b => b.textContent === "Annuler").length;
  boutons().filter(b => b.textContent === "Appliquer").pop().click();
  await attendreQue(() => serveur.finale(id), 2000, "irréversible appliquée");
  assert.deepStrictEqual(serveur.finale(id).resultat.retour, { chemin: "bolt (2).stp" });
  assert.strictEqual(boutons().filter(b => b.textContent === "Annuler").length, avantAnnuler, "pas d'Annuler");

  // annuler asynchrone : un échec ramène la carte à « appliquée » (message), un double clic n'annule qu'une fois
  let annulations = 0, echouer = true;
  const etatA = { v: 0 };
  D.commande("annulable", { ecriture: true, proposer: () => ({ titre: "annulable" }),
                            appliquer: () => { etatA.v = 1; return 0; },
                            annuler: async () => { annulations++; await pause(40); if (echouer) throw new Error("disque plein"); etatA.v = 0; },
                            etat: () => etatA.v });
  id = serveur.deposer(D.id, "annulable", {}, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte annulable");
  boutons().filter(b => b.textContent === "Appliquer").pop().click();
  await attendreQue(() => serveur.finale(id), 2000, "annulable appliquée");
  const carteAnn = () => boutons().filter(b => b.textContent === "Annuler").pop();
  carteAnn().click();
  await attendreQue(() => /disque plein/.test(texteDe(doc.body)), 2000, "échec d'annulation affiché");
  assert.ok(carteAnn(), "après un échec, la carte offre encore Annuler");
  echouer = false; annulations = 0;
  const b2 = carteAnn(); b2.click(); b2.click();
  await attendreQue(() => etatA.v === 0, 2000, "annulée");
  await pause(80);
  assert.strictEqual(annulations, 1, "un double clic n'annule qu'une fois");

  // Une carte appliquée annulable reste dans le panneau malgré une série de lectures
  etatA.v = 1;
  id = serveur.deposer(D.id, "annulable", {}, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte annulable 2");
  boutons().filter(b => b.textContent === "Appliquer").pop().click();
  await attendreQue(() => serveur.finale(id), 2000, "annulable 2 appliquée");
  for (let i = 0; i < 40; i++) D.commande("t" + i, { executer: () => i });
  for (let i = 0; i < 40; i++) serveur.deposer(D.id, "t" + i);
  await attendreQue(() => serveur.reponses.filter(r => r.etat === "finie" && /^\d+$/.test(String(r.resultat))).length >= 40, 3000, "40 lectures");
  assert.ok(carteAnn(), "l'Annuler de la carte appliquée est toujours là");

  // La capture se voit dans le panneau, et le panneau ne grossit pas sans fin
  D.capture(() => ({ width: 10, height: 10, toDataURL: () => "data:image/png;base64,QQ" }));
  id = serveur.deposer(D.id, "capture");
  await attendreQue(() => serveur.finale(id), 2000, "capture de D");
  assert.ok(/Claude a capturé/.test(texteDe(doc.body)), "capture signalée dans le panneau");
  for (let i = 0; i < 40; i++) { D.commande("l" + i, { executer: () => i }); }
  for (let i = 0; i < 40; i++) serveur.deposer(D.id, "l" + i);
  await attendreQue(() => serveur.reponses.filter(r => r.etat === "finie").length > 40, 3000, "40 lectures");
  const liste = trouver(doc.body, e => e.classes.has("pilote-liste"))[0];
  assert.ok(liste.enfants.length <= 30, "panneau borné : " + liste.enfants.length);
  assert.strictEqual(liste.attributs["aria-live"], "polite");

  // La zone « Demander à Claude » : la demande part avec l'onglet et le navigateur, la
  // conversation s'affiche au fil de l'eau (texte, outils, fin), une coupure réseau se retente
  await attendreQue(() => /Claude a lancé : l39/.test(texteDe(doc.body)), 3000, "fin des 40 lectures");
  const zone = trouver(doc.body, e => e.tagName === "textarea")[0];
  assert.ok(zone && zone.maxLength === 8000, "zone de texte présente");
  serveur.conversation = [
    { etat: "prise", messages: [{ genre: "texte", texte: "Je lis <b>la carte</b>." }] },
    "coupure",
    { etat: "prise", messages: [{ genre: "outil", texte: "cao_doc_lire" }] },
    { etat: "finie", messages: [{ genre: "texte", texte: "USB_DP fait 42 mm." }] }];
  zone.value = "  longueur de USB_DP ?  ";
  let entree = false;
  zone.ecoutes.keydown.forEach(f => f({ key: "Enter", shiftKey: false, preventDefault: () => { entree = true; } }));
  assert.ok(entree && zone.value === "", "Entrée envoie et vide la zone");
  assert.strictEqual(bouton("Claude travaille…").disabled, true, "pas de seconde demande pendant la première");
  await attendreQue(() => bouton("Envoyer"), 3000, "conversation finie");
  assert.deepStrictEqual(serveur.demandes[0], { navigateur: serveur.nav, onglet: D.id, texte: "longueur de USB_DP ?",
                                                 nouvelle: false, pieces: [], modele: "opus" });
  assert.ok(avecEntete(serveur.postes.filter(p => p.route === "/api/pilote/demande").pop()), "la demande porte l'en-tête maison");
  assert.deepStrictEqual(serveur.lectures.map(l => l.depuis), ["0", "1", "1", "2"], "lecture reprise après la coupure");
  assert.ok(serveur.lectures.every(l => l.id === "d1" && l.navigateur === serveur.nav));
  const lu = texteDe(doc.body);
  assert.ok(/longueur de USB_DP \?/.test(lu) && /Je lis <b>la carte<\/b>\./.test(lu) && /→ cao_doc_lire/.test(lu)
            && /USB_DP fait 42 mm\./.test(lu), "conversation affichée en texte brut");
  // « Nouvelle conversation » : le panneau se vide (sauf une carte qui attend un clic), et la
  // demande suivante le dit au lanceur, une seule fois
  const idCarte = serveur.deposer(D.id, "regler", { v: "7" }, true);
  await attendreQue(() => serveur.reponsesDe(idCarte).includes("proposee"), 2000, "carte avant la nouvelle conversation");
  bouton("Nouvelle conversation").click();
  const listeD = trouver(doc.body, e => e.classes.has("pilote-liste"))[0];
  assert.ok(!/longueur de USB_DP|USB_DP fait 42 mm|cao_doc_lire/.test(texteDe(listeD)), "l'ancienne conversation est effacée");
  const restees = trouver(listeD, e => e.classes.has("pilote-carte"));
  assert.ok(restees.filter(c => c.classes.has("proposee")).length === 1 && restees.every(c => c.aGarder),
            "restent la carte à valider et celles qu'on peut encore annuler, rien d'autre");
  assert.ok(/Nouvelle conversation : Claude repart de zéro/.test(texteDe(listeD)));
  bouton("Refuser").click();
  await attendreQue(() => serveur.finale(idCarte), 2000, "carte refusée");
  await D.demander("on repart");
  await D.demander("et ensuite");
  assert.deepStrictEqual(serveur.demandes.slice(1).map(d => d.nouvelle), [true, false]);
  // Un refus du serveur (Claude occupé, onglet inconnu) s'affiche, et la zone reste utilisable
  serveur.refusDemande = "Claude travaille déjà sur ta demande précédente.";
  await D.demander("encore");
  assert.ok(/Demande non aboutie : Claude travaille déjà/.test(texteDe(doc.body)) && bouton("Envoyer"));
  serveur.refusDemande = null;

  // Modèle : Opus par défaut, le choix part avec la demande et reste sur l'appareil
  const modele = trouver(doc.body, e => e.tagName === "select")[0];
  assert.deepStrictEqual(modele.enfants.map(o => o.value), ["opus", "sonnet", "haiku"]);
  modele.value = "haiku";
  modele.dispatch("change");
  await D.demander("vite");
  assert.strictEqual(serveur.demandes[serveur.demandes.length - 1].modele, "haiku");
  assert.strictEqual(stockage.getItem("websuite.pilote.modele"), "haiku", "choix gardé sur l'appareil");
  modele.value = "opus";
  modele.dispatch("change");

  // Test de connexion à chaque ouverture : vert, Envoyer actif ; rouge (lanceur absent, Claude Code
  // pas connecté), Envoyer désactivé et la raison affichée
  const tete = trouver(doc.body, e => e.classes.has("pilote-tete"))[0];
  const ligne = trouver(doc.body, e => e.classes.has("pilote-etat"))[0];
  const rouvrir = async attendu => {
    if (panneauOuvert(doc)) tete.click();
    tete.click();
    await attendreQue(() => ligne.textContent === attendu, 2000, attendu);
  };
  await rouvrir("Claude Code 9.9 connecté.");
  assert.ok(ligne.classes.has("ok") && !bouton("Envoyer").disabled);
  serveur.statut = { lanceur: false, claude: null };
  await rouvrir("Le lanceur WEB·SUITE ne répond pas : lance l'outil depuis WEB·SUITE pour écrire à Claude.");
  assert.ok(ligne.classes.has("ko") && bouton("Envoyer").disabled, "Envoyer désactivé en rouge");
  serveur.statut = { lanceur: true, claude: { pret: false, message: "Claude Code 9.9 n'est pas connecté." } };
  await rouvrir("Claude Code 9.9 n'est pas connecté.");
  assert.ok(bouton("Envoyer").disabled);
  serveur.statut = { lanceur: true, claude: { pret: true, message: "Claude Code 9.9 connecté.", version: "9.9" } };
  await rouvrir("Claude Code 9.9 connecté.");
  assert.ok(!bouton("Envoyer").disabled, "de nouveau vert à la réouverture");

  // Pièces jointes : trombone, coller, déposer, « Vue » ; un binaire est refusé, une photo HEIC
  // de 4000 px redessinée en JPEG de 2000 px, un × retire, la demande les emporte puis les oublie
  const toileJ = (w, h) => ({ width: w, height: h, getContext: () => ({ fillRect() {}, drawImage() {} }),
                              toDataURL(t) { return "data:" + t + ";base64,QUJD" + this.width + "x" + this.height; } });
  const creerEl = doc.createElement;
  doc.createElement = t => (t === "canvas" ? toileJ(0, 0) : creerEl(t));
  D._env.decoderImage = async f => (f.type === "image/heic" ? { width: 4000, height: 3000 }
                                   : f.type === "image/png" ? { width: 100, height: 80 } : Promise.reject(new Error("illisible")));
  const choix = trouver(doc.body, e => e.tagName === "input" && e.type === "file")[0];
  const puces = () => trouver(doc.body, e => e.classes.has("pilote-piece"));
  assert.ok(choix && choix.multiple && choix.hidden, "sélecteur de fichiers caché, plusieurs fichiers");
  D.capture(null);
  assert.ok(bouton("Joindre") && bouton("Vue").hidden, "« Joindre » ; « Vue » caché sans canevas");
  choix.files = [new File(["# Note\nUSB_DP : 90 ohms"], "notes.md", { type: "text/markdown" }),
                 new File(["%PDF-1.4"], "ds.pdf", { type: "application/pdf" }),
                 new File([new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0xff, 0x00])], "projet.zip", { type: "application/zip" }),
                 new File(["x"], "photo.heic", { type: "image/heic" }),
                 new File(["x"], "bizarre.tiff", { type: "image/tiff" }),
                 new File(["<svg/>"], "schema.svg", { type: "image/svg+xml" })];   // du texte, pas une image à décoder
  choix.dispatch("change");
  await attendreQue(() => puces().length === 4, 2000, "quatre pièces jointes");
  const lu2 = texteDe(doc.body);
  assert.ok(/projet\.zip » : fichier binaire/.test(lu2) && /bizarre\.tiff » : image que ce navigateur ne sait pas lire/.test(lu2), lu2);
  const png = new File([new Uint8Array([0x89, 0x50, 0x4e, 0x47])], "image.png", { type: "image/png" });
  let colle = false;
  zone.ecoutes.paste.forEach(f => f({ clipboardData: { files: [png] }, preventDefault: () => { colle = true; } }));
  zone.ecoutes.paste.forEach(f => f({ clipboardData: { files: [] }, preventDefault: () => assert.fail("texte collé") }));
  await attendreQue(() => puces().length === 5, 2000, "image collée");
  assert.ok(colle);
  const panneauJ = trouver(doc.body, e => e.classes.has("pilote"))[0];
  let arrete = false;                                  // la page, qui ouvre ce qu'on lui dépose, ne le voit pas
  panneauJ.ecoutes.drop.forEach(f => f({ dataTransfer: { files: [new File(["a,b"], "mesures.csv", { type: "text/csv" })] },
                                         preventDefault() {}, stopPropagation() { arrete = true; } }));
  assert.ok(arrete, "dépôt arrêté au panneau");
  await attendreQue(() => puces().length === 6, 2000, "fichier déposé");
  D.capture(() => toileJ(800, 600));
  assert.ok(!bouton("Vue").hidden, "« Vue » apparaît avec un canevas");
  bouton("Vue").click();
  assert.strictEqual(puces().length, 7);
  // × retire le CSV
  trouver(puces().find(p => /mesures\.csv/.test(p.textContent)), e => e.tagName === "button")[0].click();
  assert.strictEqual(puces().length, 6);
  serveur.conversation = [{ etat: "finie", messages: [{ genre: "info", texte: "Nouvelle conversation : trop de fichiers." }] }];
  await D.demander("lis tout ça");
  assert.ok(trouver(doc.body, e => e.classes.has("pilote-trace") && e.textContent === "Nouvelle conversation : trop de fichiers.").length,
            "une information du lanceur s'affiche en trace");
  const avec = serveur.demandes[serveur.demandes.length - 1];
  assert.deepStrictEqual(avec.pieces, [
    { nom: "notes.md", texte: "# Note\nUSB_DP : 90 ohms" },
    { nom: "ds.pdf", type: "application/pdf", donnees: "JVBERi0xLjQ=" },
    { nom: "photo.heic", type: "image/jpeg", donnees: "QUJD2000x1500" },          // 4000 px ramenée à 2000
    { nom: "schema.svg", texte: "<svg/>" },
    { nom: "image.png", type: "image/png", donnees: "iVBORw==" },                // gardée telle quelle
    { nom: "vue-antenna.png", type: "image/png", donnees: "QUJD800x600" }]);
  assert.ok(/lis tout ça\nJoint : notes\.md, ds\.pdf, photo\.heic, schema\.svg, image\.png, vue-antenna\.png/.test(texteDe(doc.body)));
  assert.strictEqual(puces().length, 0, "pièces oubliées une fois parties");
  // Une demande refusée garde ses pièces ; les limites préviennent avant l'envoi
  choix.files = [new File(["a"], "a.md")];
  choix.dispatch("change");
  await attendreQue(() => puces().length === 1, 2000, "a.md");
  serveur.refusDemande = "Pièces jointes de plus de 20 Mo au total.";
  await D.demander("encore");
  assert.ok(/Demande non aboutie : Pièces jointes de plus/.test(texteDe(doc.body)) && puces().length === 1);
  serveur.refusDemande = null;
  choix.files = Array.from({ length: 10 }, (_, i) => new File(["x"], i + ".md"));
  choix.dispatch("change");
  await attendreQue(() => /au plus 10 pièces jointes/.test(texteDe(doc.body)), 2000, "11e pièce refusée");
  assert.strictEqual(puces().length, 10);
  trouver(doc.body, e => e.classes.has("pilote-pieces"))[0].enfants.slice().forEach(p => p.enfants[0].click());
  choix.files = [new File(["x".repeat(400001)], "long.md")];
  choix.dispatch("change");
  await attendreQue(() => /dépasse les 400000 caractères/.test(texteDe(doc.body)), 2000, "texte trop long");
  assert.strictEqual(puces().length, 0);
  // Envoyer tout de suite après un dépôt : la demande attend la fin de la lecture et emporte le fichier
  D._env.decoderImage = async () => { await pause(150); return { width: 10, height: 10 }; };
  choix.files = [new File(["x"], "lente.png", { type: "image/png" })];
  choix.dispatch("change");
  await D.demander("tout de suite");
  assert.deepStrictEqual(serveur.demandes[serveur.demandes.length - 1].pieces.map(p => p.nom), ["lente.png"],
                         "Envoyer attend la fin des lectures");
  doc.createElement = creerEl;

  restants.concat(D).forEach(o => o.arreter());
  await pause(REGLAGES.attenteServeur + 50);

  // Un onglet visible est préféré comme relais : un onglet caché, ralenti par le navigateur, s'annonce mal
  const canal2 = "banc2." + Math.random(), stock2 = stockagePartage();
  const cache = creer({ fetch: serveur.fetch, BroadcastChannel, stockage: stock2, canal: canal2, visible: () => false },
                      REGLAGES).declarer({ page: "pcb" });
  const vu = creer({ fetch: serveur.fetch, BroadcastChannel, stockage: stock2, canal: canal2, visible: () => true },
                   REGLAGES).declarer({ page: "schema" });
  await pause(REGLAGES.amorce * 2);
  assert.deepStrictEqual([vu.relais, cache.relais], [true, false], "le visible tient le relais, quel que soit l'id");
  cache.arreter(); vu.arreter();
  await pause(REGLAGES.attenteServeur + 50);

  // Un clic dans un onglet qui n'est pas le relais part au serveur tout de suite, sans attendre
  // la prochaine élection du relais (annonce lente ici : 1 s). Vu à l'essai réel dans Chromium.
  const canal3 = "banc3." + Math.random(), stock3 = stockagePartage(), lent = Object.assign({}, REGLAGES, { annonce: 1000, battement: 10000 });
  const t1 = creer({ fetch: serveur.fetch, BroadcastChannel, stockage: stock3, canal: canal3 }, lent).declarer({ page: "pcb" });
  const t2 = creer({ fetch: serveur.fetch, BroadcastChannel, stockage: stock3, canal: canal3 }, lent).declarer({ page: "schema" });
  await attendreQue(() => t1.relais || t2.relais, 3000, "relais (annonce lente)");
  const [rel, autre] = t1.relais ? [t1, t2] : [t2, t1];
  await pause(1100);                                   // laisser passer une élection
  rel.focaliser();                                     // le relais devient le plus récent…
  await pause(150);
  const n0 = serveur.battements.length;                // … puis un clic dans l'autre onglet
  autre.focaliser();
  await pause(150);                                    // bien avant la prochaine élection (1 s)
  assert.ok(serveur.battements.slice(n0).some(b => {
    const t = [...b.onglets].sort((x, y) => x.focus_il_y_a - y.focus_il_y_a);
    return t[0] && t[0].id === autre.id;
  }), "le clic dans un onglet non relais est transmis aussitôt");
  t1.arreter(); t2.arreter();
  await pause(REGLAGES.attenteServeur + 50);

  // Dans la page : l'en-tête vient de ENTETE_WEB, et l'adieu part en fetch keepalive (sendBeacon
  // ne porte pas d'en-tête, le serveur le refuserait)
  const envois = [];
  const fen = { location: { protocol: "http:" }, BroadcastChannel, localStorage: stockagePartage(), addEventListener() {},
                fetch: async (u, o) => { envois.push([u, o]); return { ok: true, status: 200, json: async () => ({}) }; } };
  const ctx = { window: fen, document: { visibilityState: "visible" }, navigator: {}, crypto: require("crypto").webcrypto,
                ENTETE_WEB: { "X-WEB_ANTENNA": "1" } };
  require("vm").runInNewContext(require("fs").readFileSync(require("path").join(__dirname, "pilote.js"), "utf8"), ctx);
  await fen.Pilote._env.balise("/api/pilote/battement", "{}");
  assert.deepStrictEqual(JSON.parse(JSON.stringify(envois)), [["/api/pilote/battement", { method: "POST", keepalive: true, body: "{}",
    headers: { "Content-Type": "application/json", "X-WEB_ANTENNA": "1" } }]], "balise de la page : fetch keepalive avec l'en-tête");
  console.log("OK");
  process.exit(0);
})().catch(e => { console.error(e); process.exit(1); });
