/* Banc de pilote.js (canal-pilote, côté page) : plusieurs « onglets » dans un même
   processus Node, reliés par le vrai BroadcastChannel de Node, face à un faux serveur
   du pilote (file de commandes et long-poll, comme pilote.py).

       node lanceur/banc_pilote.js

   Affiche OK si tout passe. */
"use strict";
const assert = require("assert");
const { creer } = require("./pilote.js");

/* silence bien plus long que l'amorce, comme en vrai (90 s contre 5 s) : l'élection ne doit
   attendre que l'amorce, et un relais qui se ferme proprement (« bye ») est remplacé aussitôt. */
const REGLAGES = { annonce: 40, amorce: 160, silence: 3000, battement: 120, attenteServeur: 80, repos: 30 };
const pause = ms => new Promise(r => setTimeout(r, ms));

/* Faux serveur du pilote : il note les battements, compte les long-polls en cours et
   rend les commandes au navigateur de l'onglet visé, comme pilote.py. */
function fauxServeur() {
  const s = { battements: [], file: [], reponses: [], enCours: 0, maxEnCours: 0, polls: [], balises: [] };
  s.fetch = async (url, opts = {}) => {
    const u = new URL(url, "http://outil");
    const corps = opts.body ? JSON.parse(opts.body) : null;
    const rendre = obj => ({ ok: true, status: 200, json: async () => obj });
    if (u.pathname === "/api/pilote/battement") { s.battements.push(corps); return rendre({ ok: true }); }
    if (u.pathname === "/api/pilote/reponse") { s.reponses.push(corps); return rendre({ ok: true }); }
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
  return creer({ fetch: serveur.fetch, BroadcastChannel, stockage, canal: nomCanal, balise }, REGLAGES).declarer({
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
  id = serveur.deposer(B.id, "capture");
  await attendreQue(() => serveur.finale(id), 2000, "capture sans canevas");
  assert.ok(/canevas/.test(serveur.finale(id).resultat) && serveur.finale(id).etat === "echouee");

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
  const D = creer({ fetch: serveur.fetch, BroadcastChannel, stockage, canal, document: domFactice() }, REGLAGES)
    .declarer({ page: "antenna", projet: () => "dipole" });
  const etat = { v: "1" };
  let annule = 0;
  D.commande("regler", {
    ecriture: true,
    proposer: a => ({ titre: "<img src=x onerror=alert(1)>", avant: etat.v, apres: a.v }),
    appliquer: a => { const avant = etat.v; etat.v = a.v; return avant; },
    annuler: (a, avant) => { etat.v = avant; annule++; },
    etat: () => etat.v,
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

  // Proposée : rien n'est appliqué avant le clic ; la carte montre le texte tel quel
  id = serveur.deposer(D.id, "regler", { v: "2" }, true);
  await attendreQue(() => serveur.reponsesDe(id).includes("proposee"), 2000, "carte proposée");
  assert.strictEqual(etat.v, "1", "rien appliqué avant le clic");
  assert.ok(texteDe(doc.body).includes("<img src=x onerror=alert(1)>"), "titre affiché tel quel");
  assert.ok(texteDe(doc.body).includes("1") && texteDe(doc.body).includes("2"), "avant et après affichés");
  assert.ok(panneauOuvert(doc), "le panneau s'ouvre à l'arrivée d'une carte");
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

  restants.concat(D).forEach(o => o.arreter());
  await pause(REGLAGES.attenteServeur + 50);
  console.log("OK");
  process.exit(0);
})().catch(e => { console.error(e); process.exit(1); });
