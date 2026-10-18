/* Banc de pilote.js (canal-pilote, côté page) : plusieurs « onglets » dans un même
   processus Node, reliés par le vrai BroadcastChannel de Node, face à un faux serveur
   du pilote (file de commandes et long-poll, comme pilote.py).

       node lanceur/banc_pilote.js

   Affiche OK si tout passe. */
"use strict";
const assert = require("assert");
const { creer } = require("./pilote.js");

const REGLAGES = { annonce: 40, silence: 160, battement: 120, attenteServeur: 80, repos: 30 };
const pause = ms => new Promise(r => setTimeout(r, ms));

/* Faux serveur du pilote : il note les battements, compte les long-polls en cours et
   rend les commandes au navigateur de l'onglet visé, comme pilote.py. */
function fauxServeur() {
  const s = { battements: [], file: [], reponses: [], enCours: 0, maxEnCours: 0, polls: [] };
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

function onglet(serveur, stockage, page, nomCanal) {
  return creer({ fetch: serveur.fetch, BroadcastChannel, stockage, canal: nomCanal }, REGLAGES).declarer({
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
  await pause(REGLAGES.silence * 2);
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
  const cible = restants[restants.length - 1];
  cible.commande("apres", { executer: () => "toujours là" });
  id = serveur.deposer(cible.id, "apres");
  await attendreQue(() => restants.some(o => o.relais), REGLAGES.silence + REGLAGES.annonce * 3, "nouveau relais");
  await attendreQue(() => serveur.finale(id), 2000, "commande après changement de relais");
  assert.strictEqual(serveur.finale(id).resultat, "toujours là");
  await attendreQue(() => {
    const b = serveur.battements[serveur.battements.length - 1];
    return b && b.onglets.length === 2 && !b.onglets.some(o => o.id === ancien.id);
  }, 2000, "battement sans l'onglet fermé");

  restants.forEach(o => o.arreter());
  await pause(REGLAGES.attenteServeur + 50);
  console.log("OK");
  process.exit(0);
})().catch(e => { console.error(e); process.exit(1); });
