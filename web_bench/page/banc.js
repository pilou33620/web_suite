/* web_bench — page Banc : l'outil dans le cadre, le clic-partout, les bancs, les rapports.
   L'outil est relayé par le serveur du banc : il est de même origine que cette page,
   qui peut donc lire son DOM et y cliquer. La sonde (sonde.js) note ses erreurs. */
"use strict";

const $ = (s) => document.querySelector(s);
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const cadre = $("#cadre");

async function api(route, corps) {
  const opts = corps === undefined ? {} : {
    method: "POST", headers: { "Content-Type": "application/json", "X-Web-Bench": "1" }, body: JSON.stringify(corps),
  };
  const r = await fetch("/__banc/api/" + route, opts);
  return r.json();
}
function memo(cle, valeur) {   // réglages gardés dans ce navigateur (sans garantie)
  try {
    if (valeur === undefined) return localStorage.getItem("web_bench." + cle);
    localStorage.setItem("web_bench." + cle, valeur);
  } catch (_) { return null; }
}
function echapper(t) {
  return String(t).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
}

/* ---------------------------------------------------------------- onglets */
document.querySelectorAll(".onglets button").forEach((b) => b.addEventListener("click", () => {
  document.querySelectorAll(".onglets button, .volet").forEach((x) => x.classList.remove("actif"));
  b.classList.add("actif");
  $("#volet-" + b.dataset.onglet).classList.add("actif");
  if (b.dataset.onglet === "bancs") rafraichirBancs();
  if (b.dataset.onglet === "rapports") rafraichirRapports();
}));

/* ---------------------------------------------------------------- l'outil */
let pages = [];
let chargements = 0;              // nombre de « load » du cadre : une navigation se voit ici
cadre.addEventListener("load", () => { chargements++; });

/* l'outil affiché : WEB_CAO, WEB_ANTENNA ou WEB_3D, un seul à la fois dans le cadre */
const banc = { actif: null, outils: [] };
function outilActif() { return banc.outils.find((o) => o.id === banc.actif) || { nom: "?", noire: "" }; }
function nomOutil() { return outilActif().nom; }

async function rafraichirEtat() {
  try {
    const e = await api("etat");
    banc.actif = e.actif;
    banc.outils = e.outils;
    const o = outilActif();
    const p = $("#etat-outil");
    p.textContent = o.en_marche ? "port " + o.port : "arrêté";
    p.className = "pastille " + (o.en_marche ? "ok" : "ko");
    p.title = o.dossier || "";
  } catch (_) {
    $("#etat-outil").textContent = "banc injoignable";
    $("#etat-outil").className = "pastille ko";
  }
}

function chargerPage(url) {
  return new Promise((res) => {
    const vers = () => {
      cadre.removeEventListener("load", vers);
      cadre.addEventListener("load", fini);
      cadre.src = url;
    };
    const fini = () => { cadre.removeEventListener("load", fini); res(); };
    // passer par about:blank recharge vraiment, même si l'adresse ne change pas
    cadre.addEventListener("load", vers);
    cadre.src = "about:blank";
  }).then(() => attendreCalme(8000)).then(() => pause(500));
}

function fenetre() { return cadre.contentWindow; }
function sonde() { const w = fenetre(); return w && w.__banc; }

async function attendreCalme(maxi) {
  const fin = Date.now() + maxi;
  let calmes = 0;
  while (Date.now() < fin) {
    const s = sonde();
    if (s && s.requetes <= 0 && fenetre().document.readyState === "complete") {
      if (++calmes >= 2) return true;
    } else calmes = 0;
    await pause(100);
  }
  return false;
}

$("#page-cadre").addEventListener("change", (e) => chargerPage(e.target.value));
$("#b-recharger").addEventListener("click", () => chargerPage($("#page-cadre").value));
$("#b-redemarrer").addEventListener("click", async () => {
  $("#b-redemarrer").disabled = true;
  $("#etat-outil").textContent = "redémarrage…";
  const r = await api("outil/redemarrer", {});
  if (r.erreur) alert(r.erreur);
  journalFin = 0;
  $("#journal").textContent = "";
  await rafraichirEtat();
  await chargerPage($("#page-cadre").value);
  $("#b-redemarrer").disabled = false;
});

/* ---------------------------------------------------------------- surlignage */
function surligner(el) {
  const s = $("#surlignage");
  if (!el || !el.isConnected) { s.hidden = true; return; }
  const r = el.getBoundingClientRect();
  s.style.left = r.left - 3 + "px";
  s.style.top = r.top - 3 + "px";
  s.style.width = r.width + 6 + "px";
  s.style.height = r.height + 6 + "px";
  s.hidden = false;
}

/* ---------------------------------------------------------------- éléments cliquables */
const SELECTEUR = [
  "button", "[role=button]", "[role=menuitem]", "[role=tab]", "[role=option]", "a[href]", "summary", "select",
  "input[type=button]", "input[type=submit]", "input[type=reset]", "input[type=checkbox]", "input[type=radio]",
  "input[type=range]", "input[type=file]", "[onclick]", ".mnu-t",
].join(",");

function libelle(el) {
  // jamais la valeur d'une liste ou d'un curseur : elle change à chaque essai
  const champ = el.tagName === "SELECT" || el.tagName === "INPUT";
  const t = el.getAttribute("aria-label") || el.getAttribute("title") ||
    (champ ? (el.labels && el.labels[0] && el.labels[0].textContent) || (el.type === "button" || el.type === "submit" ? el.value : "")
      : el.textContent) || el.getAttribute("name") || el.id || "";
  return t.replace(/\s+/g, " ").trim().slice(0, 80) || "<" + el.tagName.toLowerCase() + ">";
}

function cle(el) {
  const doc = el.ownerDocument;
  const morceaux = [];
  let n = el;
  while (n && n.nodeType === 1 && n !== doc.body) {
    if (n.id) { morceaux.unshift("#" + n.id); break; }
    let i = 1, s = n;
    while ((s = s.previousElementSibling)) if (s.tagName === n.tagName) i++;
    morceaux.unshift(n.tagName.toLowerCase() + ":" + i);
    n = n.parentElement;
  }
  // le chemin seul : un texte qui change (compteur, valeur) ne doit pas faire un nouvel élément
  return morceaux.join(">");
}

function estVisible(el) {
  if (el.disabled) return false;
  const r = el.getBoundingClientRect();
  if (r.width < 2 || r.height < 2) return false;
  if (el.checkVisibility) return el.checkVisibility({ checkOpacity: true, checkVisibilityCSS: true });
  return el.offsetParent !== null;
}

/* Les candidats visibles : ceux du sélecteur, plus les éléments au curseur « main »
   (div ou span avec un écouteur de clic) dont le parent n'a pas ce curseur. */
function candidats(doc) {
  const w = doc.defaultView;
  const trouves = new Set(doc.querySelectorAll(SELECTEUR));
  for (const el of doc.body.querySelectorAll("div, span, li, td, img, svg, label")) {
    if (trouves.has(el) || el.closest(SELECTEUR)) continue;
    const st = w.getComputedStyle(el);
    if (st.cursor !== "pointer") continue;
    const p = el.parentElement;
    if (p && w.getComputedStyle(p).cursor === "pointer") continue;
    trouves.add(el);
  }
  const carte = new Map();
  for (const el of trouves) {
    if (el.tagName === "CANVAS" || !estVisible(el)) continue;
    const k = cle(el);
    if (!carte.has(k)) carte.set(k, el);
  }
  return carte;
}

/* Accessible : au centre de l'écran du cadre, et pas caché sous une fenêtre ou un menu. */
function accessible(el) {
  const doc = el.ownerDocument, w = doc.defaultView;
  try { el.scrollIntoView({ block: "nearest", inline: "nearest" }); } catch (_) {}
  const r = el.getBoundingClientRect();
  const x = r.left + Math.min(r.width / 2, 20), y = r.top + r.height / 2;
  if (x < 0 || y < 0 || x > w.innerWidth || y > w.innerHeight) return false;
  const dessus = doc.elementFromPoint(x, y);
  return !!dessus && (dessus === el || el.contains(dessus));
}

function raisonIgnorer(el, noire) {
  const texte = [libelle(el), el.id, el.getAttribute("title"), el.getAttribute("aria-label"),
    el.getAttribute("href")].filter(Boolean).join(" ");
  if (noire && noire.test(texte)) return "liste noire";
  if (el.tagName === "INPUT" && el.type === "file") return "sélection de fichier";
  if (el.tagName === "A") {
    const href = el.getAttribute("href") || "";
    if (el.target === "_blank") return "lien vers un nouvel onglet";
    try {
      const u = new URL(href, el.ownerDocument.baseURI);
      if (u.origin !== location.origin) return "lien externe";
      if (/^(mailto|tel|javascript):/i.test(href)) return "lien spécial";
    } catch (_) { return "lien illisible"; }
  }
  return null;
}

function cliquer(el) {
  const w = el.ownerDocument.defaultView;
  const r = el.getBoundingClientRect();
  const x = r.left + Math.min(r.width / 2, 20), y = r.top + r.height / 2;
  const base = { bubbles: true, cancelable: true, composed: true, clientX: x, clientY: y, view: w, button: 0 };
  const ptr = Object.assign({ pointerId: 1, pointerType: "mouse", isPrimary: true }, base);
  if (el.tagName === "SELECT") {
    if (el.options.length > 1) el.selectedIndex = (el.selectedIndex + 1) % el.options.length;
    el.dispatchEvent(new w.Event("input", { bubbles: true }));
    el.dispatchEvent(new w.Event("change", { bubbles: true }));
    return;
  }
  if (el.tagName === "INPUT" && el.type === "range") {
    const min = +el.min || 0, max = el.max === "" ? 100 : +el.max;
    el.value = String(+el.value >= (min + max) / 2 ? min : max);
    el.dispatchEvent(new w.Event("input", { bubbles: true }));
    el.dispatchEvent(new w.Event("change", { bubbles: true }));
    return;
  }
  el.dispatchEvent(new w.PointerEvent("pointerdown", ptr));
  el.dispatchEvent(new w.MouseEvent("mousedown", base));
  if (el.focus) el.focus({ preventScroll: true });
  el.dispatchEvent(new w.PointerEvent("pointerup", ptr));
  el.dispatchEvent(new w.MouseEvent("mouseup", base));
  el.click();
}

/* Le bouton qui referme la fenêtre ou le menu du dessus : Annuler, Fermer, ×, OK… */
const FERMEURS = /^(annuler|fermer|close|cancel|non|×|✕|✖|ok|compris|j'ai compris|retour)$/i;
function trouverFermeur(carte) {
  const possibles = [...carte.values()].filter((el) => FERMEURS.test(libelle(el)) && accessible(el));
  // d'abord ce qui renonce (Annuler, ×), en dernier ce qui valide (OK)
  possibles.sort((a, b) => (/^ok|compris/i.test(libelle(a)) ? 1 : 0) - (/^ok|compris/i.test(libelle(b)) ? 1 : 0));
  return possibles[0] || null;
}

function echap() {
  const w = fenetre();
  if (!w) return;
  const doc = w.document;
  const cible = doc.activeElement || doc.body;
  for (const type of ["keydown", "keyup"]) {
    cible.dispatchEvent(new w.KeyboardEvent(type, { key: "Escape", code: "Escape", keyCode: 27, which: 27, bubbles: true }));
  }
}

/* ---------------------------------------------------------------- journal du serveur */
let journalFin = 0;
const TRACE = /Traceback|^\s*\w*(Error|Exception)\b/;
async function lireJournal() {
  const r = await api("journal?depuis=" + journalFin);
  journalFin = r.fin;
  if (r.lignes.length) {
    const pre = $("#journal");
    const html = r.lignes.map((l) => TRACE.test(l) ? '<span class="l-err">' + echapper(l) + "</span>" : echapper(l)).join("\n");
    pre.insertAdjacentHTML("beforeend", html + "\n");
    while (pre.childNodes.length > 3000) pre.removeChild(pre.firstChild);
    if ($("#volet-journal").classList.contains("actif")) pre.scrollTop = pre.scrollHeight;
  }
  return r.lignes;
}
setInterval(() => { if (!clic.enCours) lireJournal().catch(() => {}); }, 2000);

/* ---------------------------------------------------------------- clic-partout */
const GRAVITE = { js: "erreur", promesse: "erreur", reseau: "erreur", serveur: "erreur",
  reseau4xx: "avert", console: "avert", ressource: "avert",
  dialogue: "info", telechargement: "info", navigation: "info", info: "info", bloque: "info", attendu: "info", externe: "avert", lent: "avert" };
/* Ce qui est normal quand l'outil est lancé seul par le banc, et pas par le lanceur : pas un problème. */
const ATTENDUS = [
  /^404 GET \/api\/github(\?|$)/,      // « Envoi sur GitHub indisponible : outil pas lancé par WEB_SUITE »
];
/* Un service Internet que l'outil interroge (pcbparts.dev…) injoignable depuis ce poste. */
const EXTERNE = /connexion impossible|urlopen error|name or service not known|getaddrinfo|timed out|unreachable/i;
function classer(evts) {
  for (const e of evts) {
    if (e.type === "reseau" && e.requete && e.requete.statut >= 502 && EXTERNE.test(e.requete.reponse || "")) {
      e.type = "externe";
      e.message += " (service Internet injoignable depuis ce poste : vérifier la connexion)";
    }
    if (e.type === "reseau4xx" && ATTENDUS.some((m) => m.test(e.message))) {
      e.type = "attendu";
      e.message += " (normal : l'outil n'est pas lancé par le lanceur)";
    }
  }
  return evts;
}
const NOIRE_DEFAUT = "github|publier|pousser|\\bpush\\b|synchro|mise à jour|mettre à jour|\\bmaj\\b|" +
  "redémarr|redemarr|arrêter le serveur|quitter|déconnex|deconnex|demander à claude";

const clic = { enCours: false, pause: false, stop: false, resultats: [], filtre: "tous", choisi: null, debut: null };

function lireRegex() {
  const t = $("#r-noire").value.trim();
  if (!t) return null;
  try { return new RegExp(t, "i"); } catch (e) { alert("Liste noire illisible : " + e.message); throw e; }
}

/* De quoi retrouver l'élément dans le code : sélecteur CSS, id, classes, balise ouvrante. */
function selecteurCss(k) {
  if (!k) return "";
  const parts = k.split(">").map((p) => p.startsWith("#") ? "#" + CSS.escape(p.slice(1)) :
    p.replace(/^([a-z0-9-]+):(\d+)$/i, "$1:nth-of-type($2)"));
  return (k.startsWith("#") ? "" : "body > ") + parts.join(" > ");
}
function decrireElement(el, k) {
  const html = el.outerHTML.replace(/\s+/g, " ");
  return { selecteur: selecteurCss(k), id: el.id || "",
    classes: typeof el.className === "string" ? el.className : "", balise: html.slice(0, html.indexOf(">") + 1).slice(0, 400) };
}
function etape(h) {
  const quoi = { cliquer: "Cliquer", rouvrir: "Rouvrir le menu", fermer: "Refermer la fenêtre avec", echap: "Appuyer sur Échap" }[h.action];
  return h.action === "echap" ? quoi : quoi + " « " + h.libelle + " »" + (h.cle ? " (`" + selecteurCss(h.cle) + "`)" : "");
}

async function essayerElement(page, el, k, delai, hist, url) {
  const s = sonde();
  const n0 = s.evts.length;
  const c0 = chargements;
  const j0 = journalFin;
  const avant = new Set(candidats(fenetre().document).keys());
  const res = { page, url, cle: k, libelle: libelle(el), balise: el.tagName.toLowerCase() + (el.type ? "[" + el.type + "]" : ""),
    element: decrireElement(el, k), evts: [], duree: 0,
    // le clic-partout empile les clics : les derniers gestes avant celui-ci font partie du contexte
    etapes: ["Ouvrir `" + url + "`"].concat(hist.length > 12 ? ["… (" + (hist.length - 12) + " gestes plus tôt)"] : [],
      hist.slice(-12).map(etape), [etape({ action: "cliquer", libelle: libelle(el), cle: k })]) };
  surligner(el);
  const t0 = performance.now();
  try { cliquer(el); } catch (e) { s.noter("js", "le clic a levé : " + e.message, e.stack); }
  res.duree = Math.round(performance.now() - t0);
  await pause(delai);
  const calme = await attendreCalme(5000);
  // la page a peut-être changé : on relit la sonde de la page d'origine si elle existe encore
  const s2 = sonde();
  if (chargements !== c0) {
    res.evts.push({ type: "navigation", message: "le clic a changé de page : " + (fenetre().location.pathname) });
    res.evts.push(...s.evts.slice(n0).filter((e) => e.type !== "navigation"));
    res.navigation = true;
  } else {
    res.evts.push(...s2.evts.slice(n0));
  }
  classer(res.evts);
  // après un changement de page, ce sont les requêtes de la NOUVELLE page qui sont en cours : pas une lenteur du clic
  if (!calme && !res.navigation) res.evts.push({ type: "lent", message: "requêtes encore en cours 5 s après le clic" });
  if (res.duree > 1500) res.evts.push({ type: "lent", message: "le clic a bloqué la page " + res.duree + " ms" });
  journalFin = j0;
  const lignes = await lireJournal().catch(() => []);
  const i = lignes.findIndex((l) => /Traceback/.test(l));
  if (i >= 0) {
    // le Traceback entier : de « Traceback » à la ligne de l'exception (sans indentation)
    let f = i + 1;
    while (f < lignes.length && f < i + 80 && /^\s/.test(lignes[f])) f++;
    res.evts.push({ type: "serveur", message: "erreur Python dans " + nomOutil() + " : " + (lignes[f] || "").trim(),
      detail: lignes.slice(i, f + 1).join("\n") });
  }
  const graves = res.evts.map((e) => GRAVITE[e.type] || "info");
  res.etat = graves.includes("erreur") ? "erreur" : graves.includes("avert") ? "avert" : "ok";
  // les éléments apparus avec ce clic (menu, fenêtre) : on les essaie juste après
  res.nouveaux = res.navigation ? [] : [...candidats(fenetre().document).keys()].filter((x) => !avant.has(x));
  return res;
}

async function explorerPage(page, options) {
  $("#page-cadre").value = page.url;
  await chargerPage(page.url);
  if (!sonde()) {
    ajouterResultat({ page: page.nom, libelle: "(page)", etat: "erreur", balise: "",
      evts: [{ type: "js", message: "la sonde n'est pas chargée : la page n'a pas été relayée par le banc" }] });
    return;
  }
  sonde().confirmer = options.confirmer;
  const vus = new Set();
  const file = [];
  const ouvreur = new Map();      // élément -> celui dont le clic l'a fait apparaître
  const tentatives = new Map();
  let clics = 0, echapEssaye = false, deblocages = 0;
  let hist = [];                  // gestes faits sur la page depuis son chargement : pour reproduire
  // ce que la page a déjà signalé à son chargement
  const auChargement = classer(sonde().evts.slice());
  if (auChargement.length) {
    const graves = auChargement.map((e) => GRAVITE[e.type] || "info");
    ajouterResultat({ page: page.nom, url: page.url, libelle: "(chargement de la page)", balise: "", evts: auChargement, duree: 0,
      etapes: ["Ouvrir `" + page.url + "`"],
      etat: graves.includes("erreur") ? "erreur" : graves.includes("avert") ? "avert" : "ok" });
  }

  while (!clic.stop && clics < options.max) {
    while (clic.pause && !clic.stop) await pause(200);
    let carte = candidats(fenetre().document);
    for (const k of carte.keys()) if (!vus.has(k) && !file.includes(k)) file.push(k);

    let choisi = null;
    for (const k of file) {
      const el = carte.get(k);
      if (el && accessible(el)) { choisi = k; break; }
    }
    if (!choisi) {
      // un élément d'un menu refermé : on rouvre le menu, sans compter ce clic
      for (const k of file) {
        const o = ouvreur.get(k);
        if (!o || (tentatives.get(k) || 0) >= 2) continue;
        tentatives.set(k, (tentatives.get(k) || 0) + 1);
        const elO = carte.get(o);
        if (!elO || !accessible(elO)) continue;
        try { cliquer(elO); } catch (_) {}
        hist.push({ action: "rouvrir", libelle: libelle(elO), cle: o });
        await pause(options.delai);
        await attendreCalme(3000);
        carte = candidats(fenetre().document);
        const el = carte.get(k);
        if (el && accessible(el)) { choisi = k; break; }
      }
    }
    if (!choisi && deblocages < 60) {
      // une fenêtre déjà vue (même bouton OK, même ×) cache tout le reste : on la referme
      // avec son bouton, sans compter ce clic, puis on reprend
      const fermeur = trouverFermeur(carte);
      if (fermeur) {
        deblocages++;
        try { cliquer(fermeur); } catch (_) {}
        hist.push({ action: "fermer", libelle: libelle(fermeur), cle: cle(fermeur) });
        await pause(options.delai);
        await attendreCalme(3000);
        continue;
      }
    }
    if (!choisi) {
      if (!echapEssaye) { echap(); hist.push({ action: "echap" }); echapEssaye = true; await pause(200); continue; }
      break;
    }
    echapEssaye = false;
    file.splice(file.indexOf(choisi), 1);
    vus.add(choisi);
    const el = carte.get(choisi);
    const raison = raisonIgnorer(el, options.noire);
    if (raison) {
      ajouterResultat({ page: page.nom, cle: choisi, libelle: libelle(el), balise: el.tagName.toLowerCase(),
        etat: "ignore", evts: [{ type: "info", message: "non cliqué : " + raison }], duree: 0 });
      continue;
    }
    clics++;
    majProgression(page, clics, file.length);
    const res = await essayerElement(page.nom, el, choisi, options.delai, hist, page.url);
    ajouterResultat(res);
    hist.push({ action: "cliquer", libelle: res.libelle, cle: choisi });
    for (const n of res.nouveaux.reverse()) {
      if (vus.has(n)) continue;
      if (!ouvreur.has(n)) ouvreur.set(n, choisi);
      const i = file.indexOf(n);
      if (i >= 0) file.splice(i, 1);
      file.unshift(n);
    }
    if (res.navigation) {
      hist = [];
      await chargerPage(page.url);
      sonde().confirmer = options.confirmer;
    }
  }
  surligner(null);
  if (file.length && !clic.stop) {
    ajouterResultat({ page: page.nom, libelle: file.length + " élément(s) jamais atteints (cachés ou sous une fenêtre)",
      balise: "", etat: "ignore", evts: file.slice(0, 60).map((k) => ({ type: "info", message: k })), duree: 0 });
  }
  if (clics >= options.max) {
    ajouterResultat({ page: page.nom, libelle: "limite de " + options.max + " clics atteinte", balise: "", etat: "ignore",
      evts: [], duree: 0 });
  }
}

function majProgression(page, clics, reste) {
  const n = clic.resultats;
  $("#progression").innerHTML = "<b>" + echapper(page.nom) + "</b> · " + clics + " clics · " + reste +
    " en attente · " + n.filter((r) => r.etat === "erreur").length + " erreur(s)";
}

async function lancerClic() {
  const choisies = [...document.querySelectorAll("#choix-pages input:checked")].map((c) => pages[+c.value]);
  if (!choisies.length) return alert("Coche au moins une page.");
  let noire;
  try { noire = lireRegex(); } catch (_) { return; }
  const options = { max: +$("#r-max").value || 300, delai: +$("#r-delai").value || 0,
    confirmer: $("#r-confirmer").checked, noire };
  memo("noire." + banc.actif, $("#r-noire").value);
  memo("max", $("#r-max").value);
  memo("delai", $("#r-delai").value);
  Object.assign(clic, { enCours: true, pause: false, stop: false, resultats: [], choisi: null, debut: new Date(),
    outil: nomOutil(), rapport: null });
  $("#resultats").innerHTML = "";
  $("#detail").hidden = true;
  boutonsClic();
  await lireJournal().catch(() => {});
  for (const p of choisies) {
    if (clic.stop) break;
    await explorerPage(p, options);
  }
  const arrete = clic.stop;
  clic.enCours = false;
  boutonsClic();
  $("#progression").textContent = "Rédaction du rapport…";
  const rapport = await construireRapport(options, choisies, arrete);
  clic.rapport = rapport;
  try { await api("rapports", Object.assign({ markdown: rapportMarkdown(rapport) }, rapport)); } catch (_) {}
  $("#progression").innerHTML = (arrete ? "Arrêté" : "Terminé") + " · " + resumer(rapport.bilan) +
    " · rapport gardé dans <code>web_bench/rapports/</code> (.json et .md à donner à Claude)";
}

/* Où chercher dans le code : la première ligne de la pile qui n'est pas du banc, ou le
   fichier Python du Traceback, ou la route appelée. */
function lignePile(pile) {
  for (const l of String(pile || "").split("\n")) {
    const m = l.match(/(?:https?:\/\/[^/\s)]+)?(\/[^\s):]+\.(?:js|mjs|html))(?::(\d+))(?::(\d+))?/);
    if (m && !/\/__banc\//.test(m[1])) return nomOutil() + m[1] + ":" + m[2] + (m[3] ? ":" + m[3] : "");
  }
  return "";
}
/* Une pile lisible : sans les lignes du banc, avec des chemins du dépôt de l'outil. */
function nettoyerPile(t, outil) {
  return String(t || "").split("\n").filter((l) => !/\/__banc\//.test(l))
    .map((l) => l.split(location.origin + "/").join((outil || nomOutil()) + "/")).join("\n");
}
function sourceDe(e) {
  if (e.type === "serveur") {
    const ms = [...String(e.detail).matchAll(/File "([^"]+)", line (\d+)(?:, in (\S+))?/g)];
    const d = ms[ms.length - 1];
    return d ? d[1].replace(/^.*?[\\/](WEB_[A-Z0-9]+[\\/])/i, "$1").replace(/\\/g, "/") + ":" + d[2] + (d[3] ? " (" + d[3] + ")" : "") : "";
  }
  if (e.requete) return e.requete.methode + " " + e.requete.url.split("?")[0];
  if (e.source && !/^:|__banc|^undefined/.test(e.source)) return nomOutil() + e.source;
  return lignePile(e.detail) || lignePile(e.pile);
}
function cleProbleme(e) {
  const msg = e.message.replace(/\?\S*/g, "?…").replace(/\b[0-9a-f]{12,}\b/gi, "…");
  return e.type + "|" + msg + "|" + sourceDe(e);
}

async function construireRapport(options, choisies, arrete) {
  const r = clic.resultats;
  const o = outilActif();
  // un même problème déclenché par vingt boutons est UN problème à corriger : on regroupe
  const problemes = new Map();
  for (const res of r) {
    for (const e of res.evts) {
      const gravite = GRAVITE[e.type] || "info";
      if (gravite === "info") continue;
      const k = cleProbleme(e);
      if (!problemes.has(k)) {
        problemes.set(k, { gravite, type: e.type, message: e.message, source: sourceDe(e),
          appelant: e.requete ? lignePile(e.pile) : "", occurrences: 0, elements: [],
          exemple: { page: res.page, url: res.url, libelle: res.libelle, element: res.element || null, etapes: res.etapes || [],
            detail: e.type === "serveur" ? e.detail || "" : nettoyerPile(e.detail),
            requete: e.requete || null, pile: e.requete || e.type === "console" ? nettoyerPile(e.pile) : "" } });
      }
      const p = problemes.get(k);
      p.occurrences++;
      if (p.elements.length < 20 && !p.elements.some((x) => x.page === res.page && x.libelle === res.libelle)) {
        p.elements.push({ page: res.page, libelle: res.libelle, selecteur: res.element ? res.element.selecteur : "" });
      }
    }
  }
  const liste = [...problemes.values()].sort((a, b) =>
    (a.gravite === "erreur" ? 0 : 1) - (b.gravite === "erreur" ? 0 : 1) || b.occurrences - a.occurrences);
  // les bancs de l'outil lancés dans cette session, avec la sortie de ceux qui échouent
  let bancs = [];
  try {
    const e = await api("bancs?outil=" + banc.actif);
    for (const b of e.bancs.filter((x) => ["ok", "echec", "delai", "ignore", "arrete"].includes(x.etat))) {
      const entree = { id: b.id, ci: b.ci, etat: b.etat, code: b.code, duree: b.duree };
      if (b.etat !== "ok") {
        const s = await api("bancs/sortie?outil=" + banc.actif + "&id=" + encodeURIComponent(b.id));
        entree.sortie = (s.sortie || "").slice(-6000);
      }
      bancs.push(entree);
    }
  } catch (_) { bancs = []; }
  const w = fenetre();
  return {
    date: clic.debut.toISOString(), fin: new Date().toISOString(), outil: nomOutil(), arrete,
    environnement: { version: o.version || "", dossier: o.dossier || "", navigateur: navigator.userAgent,
      cadre: w ? w.innerWidth + " × " + w.innerHeight : "", lancement: "outil lancé par web_bench : projets vides" +
        (banc.actif === "cao" ? ", copie de la LIB" : "") + ", --local --sans-maj" },
    options: { max: options.max, delai: options.delai, confirmer: options.confirmer, noire: $("#r-noire").value,
      pages: choisies.map((p) => p.url) },
    bilan: { essais: r.filter((x) => x.etat !== "ignore").length, erreurs: r.filter((x) => x.etat === "erreur").length,
      avertissements: r.filter((x) => x.etat === "avert").length, ignores: r.filter((x) => x.etat === "ignore").length,
      problemes: liste.length },
    problemes: liste,
    bancs,
    resultats: r.map(({ nouveaux, ...reste }) => reste),
  };
}

/* La version lisible du rapport, faite pour être donnée telle quelle à Claude. */
function rapportMarkdown(rap) {
  const L = [];
  const bloc = (t, lang) => { if (t) L.push("", "```" + (lang || ""), String(t).replace(/```/g, "ʼʼʼ").trimEnd(), "```"); };
  const env = rap.environnement || {};
  L.push("# Rapport web_bench — " + rap.outil, "",
    "- **Date** : " + new Date(rap.date).toLocaleString("fr-FR") + (rap.arrete ? " (arrêté avant la fin)" : ""),
    "- **Version essayée** : " + (env.version || "?"),
    "- **Dossier** : `" + (env.dossier || "?") + "`",
    "- **Lancement** : " + (env.lancement || "?"),
    "- **Navigateur** : " + (env.navigateur || "?") + (env.cadre ? " — cadre " + env.cadre + " px" : ""),
    "- **Pages essayées** : " + (rap.options ? rap.options.pages.map((p) => "`" + p + "`").join(", ") : "?"),
    "- **Réglages** : " + (rap.options ? rap.options.max + " clics max par page, pause " + rap.options.delai + " ms, confirmations : " +
      (rap.options.confirmer ? "oui" : "non") + ", liste noire `" + rap.options.noire + "`" : "?"),
    "- **Bilan** : " + resumer(rap.bilan) + " → **" + (rap.problemes || []).length + " problème(s) distinct(s)**", "",
    "> Comment lire : chaque problème est regroupé (même message, même endroit du code) avec le nombre de fois où il",
    "> est apparu et les boutons qui le déclenchent. « Pour reproduire » donne les gestes faits sur la page depuis son",
    "> chargement : le clic-partout les enchaîne, le problème peut dépendre d'un geste précédent. Les pages sont",
    "> servies par web_bench (même chemin que dans l'outil), avec une sonde qui neutralise alert/confirm/prompt.", "");
  const probs = rap.problemes || [];
  L.push("## Problèmes à corriger (" + probs.length + ")");
  if (!probs.length) L.push("", "Aucun : rien n'a cassé pendant le clic-partout.");
  probs.forEach((p, i) => {
    const ex = p.exemple || {};
    L.push("", "### " + (i + 1) + ". [" + (p.gravite === "erreur" ? "ERREUR" : "avertissement") + "] " + p.type + " — " + p.message.split("\n")[0], "");
    if (p.source) L.push("- **Où dans le code** : `" + p.source + "`");
    if (p.appelant) L.push("- **Appelé depuis** : `" + p.appelant + "`");
    L.push("- **Occurrences** : " + p.occurrences);
    L.push("- **Déclenché par** : " + p.elements.map((x) => "« " + x.libelle + " » (" + x.page + (x.selecteur ? ", `" + x.selecteur + "`" : "") + ")").join(" ; "));
    if (ex.element) L.push("- **Élément de l'exemple** : `" + ex.element.balise.replace(/`/g, "'") + "`");
    if (ex.etapes && ex.etapes.length) {
      L.push("- **Pour reproduire** (exemple : « " + ex.libelle + " ») :");
      ex.etapes.forEach((t, j) => L.push("  " + (j + 1) + ". " + t));
    }
    if (ex.requete) {
      const q = ex.requete;
      L.push("- **Requête** : `" + q.methode + " " + q.url + "` → **" + (q.statut || "aucune réponse") + "**");
      if (q.envoye) { L.push("- **Corps envoyé** :"); bloc(q.envoye, "json"); }
      if (q.reponse) { L.push("- **Réponse du serveur** :"); bloc(q.reponse); }
    }
    if (ex.detail) { L.push("- **" + (p.type === "serveur" ? "Traceback Python" : "Pile d'appels") + "** :"); bloc(ex.detail, p.type === "serveur" ? "python" : ""); }
    else if (ex.pile && !ex.requete) { L.push("- **Pile d'appels** :"); bloc(ex.pile); }
  });
  const bancs = rap.bancs || [];
  L.push("", "## Bancs de " + rap.outil);
  if (!bancs.length) L.push("", "Aucun banc lancé dans cette session (volet Bancs).");
  else {
    const ko = bancs.filter((b) => b.etat !== "ok");
    L.push("", bancs.length + " lancé(s) : " + (bancs.length - ko.length) + " ok, " + ko.length + " en échec ou arrêté(s).");
    for (const b of ko) {
      L.push("", "### `" + b.id + "` — " + b.etat + (b.code != null ? " (code " + b.code + ")" : "") + (b.ci ? " · lancé par la CI" : " · hors CI") +
        (b.duree != null ? " · " + b.duree + " s" : ""));
      bloc(b.sortie);
    }
    const ok = bancs.filter((b) => b.etat === "ok");
    if (ok.length) L.push("", "Passés : " + ok.map((b) => "`" + b.id + "`").join(", "));
  }
  const ign = (rap.resultats || []).filter((r) => r.etat === "ignore");
  if (ign.length) {
    L.push("", "## Non essayés (" + ign.length + ")", "");
    ign.forEach((r) => L.push("- " + r.page + " · « " + r.libelle + " »" + (r.evts[0] ? " — " + r.evts[0].message : "")));
  }
  return L.join("\n") + "\n";
}
function resumer(b) {
  return b.essais + " clics, " + b.erreurs + " erreur(s), " + b.avertissements + " avertissement(s), " + b.ignores + " ignoré(s)";
}

function boutonsClic() {
  $("#b-lancer").disabled = clic.enCours;
  $("#outil").disabled = clic.enCours;
  $("#b-redemarrer").disabled = clic.enCours;
  $("#b-pause").disabled = !clic.enCours;
  $("#b-stop").disabled = !clic.enCours;
  $("#b-pause").textContent = clic.pause ? "Reprendre" : "Pause";
}
$("#b-lancer").addEventListener("click", lancerClic);
$("#b-pause").addEventListener("click", () => { clic.pause = !clic.pause; boutonsClic(); });
$("#b-stop").addEventListener("click", () => { clic.stop = true; clic.pause = false; });

/* ---- affichage des résultats */
function ajouterResultat(res) {
  res.n = clic.resultats.length + 1;
  clic.resultats.push(res);
  if (passeFiltre(res)) $("#resultats").insertAdjacentElement("beforeend", rangResultat(res));
  const t = $("#resultats");
  if (t.scrollHeight - t.scrollTop - t.clientHeight < 80) t.scrollTop = t.scrollHeight;
  majCompteurs();
}
function passeFiltre(r) { return clic.filtre === "tous" || r.etat === clic.filtre; }
function rangResultat(r) {
  const d = document.createElement("div");
  d.className = "rang";
  const premier = r.evts.find((e) => GRAVITE[e.type] === r.etat) || r.evts[0];
  d.innerHTML = '<span class="point ' + r.etat + '"></span><div><div class="titre">' + echapper(r.libelle) +
    '</div><div class="sous">' + echapper(r.page + (r.balise ? " · " + r.balise : "") + (premier ? " · " + premier.message : "")) +
    '</div></div><span class="duree">' + (r.duree ? r.duree + " ms" : "") + "</span>";
  d.addEventListener("click", () => montrerDetail(r, d));
  return d;
}
function redessinerResultats() {
  const t = $("#resultats");
  t.innerHTML = "";
  clic.resultats.filter(passeFiltre).forEach((r) => t.appendChild(rangResultat(r)));
}
document.querySelectorAll(".filtres [data-filtre]").forEach((b) => b.addEventListener("click", () => {
  document.querySelectorAll(".filtres [data-filtre]").forEach((x) => x.classList.remove("actif"));
  b.classList.add("actif");
  clic.filtre = b.dataset.filtre;
  redessinerResultats();
}));
function majCompteurs() {
  const r = clic.resultats;
  const n = (e) => r.filter((x) => x.etat === e).length;
  $("#compteurs").innerHTML = '<span style="color:var(--accent)">' + n("ok") + " ok</span>" +
    '<span style="color:var(--erreur)">' + n("erreur") + " erreurs</span>" +
    '<span style="color:var(--avert)">' + n("avert") + " avert.</span>";
}

function montrerDetail(r, rang) {
  document.querySelectorAll("#resultats .rang.choisi").forEach((x) => x.classList.remove("choisi"));
  rang.classList.add("choisi");
  const d = $("#detail");
  d.hidden = false;
  const q = (e) => e.requete ? "\nrequête : " + e.requete.methode + " " + e.requete.url + " → " + (e.requete.statut || "aucune réponse") +
    (e.requete.envoye ? "\nenvoyé : " + e.requete.envoye : "") + (e.requete.reponse ? "\nréponse : " + e.requete.reponse : "") : "";
  d.innerHTML = "<b>" + echapper(r.libelle) + "</b>\n" + echapper(r.page + (r.element ? " · " + r.element.selecteur : "")) +
    (r.element ? "\n" + echapper(r.element.balise) : "") + "\n\n" +
    (r.evts.length ? r.evts.map((e) => '<div class="evt ' + (GRAVITE[e.type] || "info") + '">[' + e.type + "] " +
      echapper(e.message) + (sourceDe(e) ? "\n→ " + echapper(sourceDe(e)) : "") + echapper(q(e)) +
      (e.detail ? "\n" + echapper(e.detail) : "") + "</div>").join("") : "Rien à signaler.") +
    (r.etapes && r.evts.some((e) => GRAVITE[e.type] === "erreur" || GRAVITE[e.type] === "avert")
      ? "\n<b>Pour reproduire</b>\n" + r.etapes.map((t, i) => (i + 1) + ". " + echapper(t.replace(/`/g, ""))).join("\n") : "");
  // montrer l'élément s'il est sur la page affichée
  if (!clic.enCours && r.cle && sonde()) {
    const el = candidats(fenetre().document).get(r.cle);
    surligner(el || null);
  }
}

$("#b-export").addEventListener("click", async () => {
  if (!clic.resultats.length) return;
  const rap = clic.rapport || await construireRapport({ max: +$("#r-max").value, delai: +$("#r-delai").value,
    confirmer: $("#r-confirmer").checked }, [], clic.stop);
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([rapportMarkdown(rap)], { type: "text/markdown" }));
  a.download = "web_bench_" + (rap.outil || "outil") + "_" + new Date(rap.date).toISOString().slice(0, 16).replace(/[:T]/g, "-") + ".md";
  a.click();
});

/* ---------------------------------------------------------------- bancs */
let bancsSuivi = null;
const cochesBancs = new Set();
async function rafraichirBancs() {
  const e = await api("bancs?outil=" + banc.actif);
  const liste = $("#liste-bancs");
  const premier = !liste.dataset.pret;
  // cochés d'office : ceux que la CI de l'outil lance, qui tournent sans préparation
  if (premier) e.bancs.forEach((b) => { if (b.ci) cochesBancs.add(b.id); });
  liste.dataset.pret = "1";
  const dossier = (b) => (b.ci ? "CI · " : "Hors CI (préparation parfois nécessaire) · ") +
    (b.id.includes("/") ? b.id.slice(0, b.id.lastIndexOf("/")) : ".");
  let html = "", courant = null;
  for (const b of e.bancs) {
    if (dossier(b) !== courant) { courant = dossier(b); html += '<div class="entete-groupe">' + echapper(courant) + "</div>"; }
    html += '<div class="rang" data-id="' + echapper(b.id) + '"><input type="checkbox" ' + (cochesBancs.has(b.id) ? "checked" : "") +
      '><div><div class="titre"><span class="point ' + b.etat + '" style="display:inline-block;margin-right:6px"></span>' +
      echapper(b.id.slice(b.id.lastIndexOf("/") + 1)) + "</div>" +
      (b.preparation.length || b.lib_reelle ? '<div class="sous">' + echapper([b.preparation.length ? "prépare : " +
        b.preparation.map((c) => c.split(" ").pop().split("/").pop()).join(" puis ") : "",
        b.lib_reelle ? "demande la LIB privée" : ""].filter(Boolean).join(" · ")) + "</div>" : "") +
      '</div><span class="duree">' + (b.duree != null ? b.duree + " s" : b.debut ? Math.round(Date.now() / 1000 - b.debut) + " s…" : "") + "</span></div>";
  }
  liste.innerHTML = html;
  liste.querySelectorAll(".rang").forEach((r) => {
    const id = r.dataset.id;
    r.querySelector("input").addEventListener("click", (ev) => {
      ev.stopPropagation();
      if (ev.target.checked) cochesBancs.add(id); else cochesBancs.delete(id);
    });
    r.addEventListener("click", async () => {
      const s = await api("bancs/sortie?outil=" + banc.actif + "&id=" + encodeURIComponent(id));
      const pre = $("#sortie-banc");
      pre.hidden = false;
      pre.textContent = id + "\n\n" + (s.sortie || "(pas encore lancé)");
    });
  });
  const n = (x) => e.bancs.filter((b) => b.etat === x).length;
  $("#bilan-bancs").textContent = e.bancs.length + " bancs · " + n("ok") + " ok · " + (n("echec") + n("delai")) +
    " en échec · " + n("ignore") + " ignorés" + (e.en_cours ? " · en cours…" : "");
  clearTimeout(bancsSuivi);
  if (e.en_cours) bancsSuivi = setTimeout(rafraichirBancs, 1000);
}
$("#b-bancs-lancer").addEventListener("click", async () => {
  const r = await api("bancs/lancer", { outil: banc.actif, ids: [...cochesBancs] });
  if (!r.ok) alert("Des bancs tournent déjà.");
  rafraichirBancs();
});
$("#b-bancs-tous").addEventListener("click", async () => {
  const e = await api("bancs?outil=" + banc.actif);
  const tous = e.bancs.every((b) => cochesBancs.has(b.id));
  cochesBancs.clear();
  if (!tous) e.bancs.forEach((b) => cochesBancs.add(b.id));
  rafraichirBancs();
});
$("#b-bancs-stop").addEventListener("click", async () => { await api("bancs/arreter", { outil: banc.actif }); rafraichirBancs(); });

/* ---------------------------------------------------------------- rapports */
async function rafraichirRapports() {
  const e = await api("rapports");
  const l = $("#liste-rapports");
  if (!e.rapports.length) { l.innerHTML = '<div class="rang"><span></span><div class="sous">Aucun rapport pour l\'instant.</div></div>'; return; }
  l.innerHTML = "";
  for (const r of e.rapports) {
    const d = document.createElement("div");
    d.className = "rang";
    const b = r.bilan || {};
    d.innerHTML = '<span class="point ' + (b.erreurs ? "erreur" : b.avertissements ? "avert" : "ok") + '"></span><div><div class="titre">' +
      echapper((r.outil || "") + " · " + new Date(r.date).toLocaleString("fr-FR")) + '</div><div class="sous">' + echapper(resumer(b)) + "</div></div>" +
      (r.md ? '<a class="lien-md" title="Le rapport lisible, à donner à Claude" download href="/__banc/api/rapport.md?nom=' +
        encodeURIComponent(r.nom) + '">.md</a>' : "<span></span>");
    d.addEventListener("click", async (ev) => {
      if (ev.target.closest("a")) return;
      const rap = await api("rapport?nom=" + encodeURIComponent(r.nom));
      clic.resultats = [];
      clic.debut = new Date(rap.date);
      clic.outil = rap.outil;
      clic.rapport = rap;
      $("#resultats").innerHTML = "";
      rap.resultats.forEach((x) => { delete x.n; ajouterResultat(x); });
      document.querySelector('.onglets [data-onglet="clic"]').click();
      $("#progression").textContent = (rap.outil || "") + " · rapport du " + clic.debut.toLocaleString("fr-FR") + " · " + resumer(b);
    });
    l.appendChild(d);
  }
}

/* ---------------------------------------------------------------- démarrage */
function listeNoire() {
  const o = outilActif();
  return memo("noire." + banc.actif) || NOIRE_DEFAUT + (o.noire ? "|" + o.noire : "");
}

/* Affiche l'outil actif : ses pages, sa liste noire, ses bancs, son journal. */
async function chargerOutil() {
  await rafraichirEtat();
  document.querySelectorAll(".nom-outil").forEach((x) => { x.textContent = nomOutil(); });
  document.title = "web_bench · " + nomOutil();
  $("#outil").value = banc.actif;
  $("#r-noire").value = listeNoire();
  pages = (await api("pages")).pages;
  $("#page-cadre").innerHTML = pages.map((p) => '<option value="' + echapper(p.url) + '">' + echapper(p.nom) + "</option>").join("");
  // l'accueil de WEB_CAO n'est qu'un aiguillage : pas coché d'office quand il y a d'autres pages
  $("#choix-pages").innerHTML = pages.map((p, i) =>
    '<label class="case"><input type="checkbox" value="' + i + '"' + (p.url === "/index.html" && pages.length > 2 ? "" : " checked") +
    "> " + echapper(p.nom) + "</label>").join("");
  journalFin = 0;
  $("#journal").textContent = "";
  cochesBancs.clear();
  delete $("#liste-bancs").dataset.pret;
  $("#sortie-banc").hidden = true;
  if ($("#volet-bancs").classList.contains("actif")) rafraichirBancs();
  await chargerPage(pages.length ? pages[0].url : "/");
}

$("#outil").addEventListener("change", async (e) => {
  const id = e.target.value;
  e.target.disabled = true;
  $("#etat-outil").textContent = "démarrage…";
  $("#etat-outil").className = "pastille";
  const r = await api("outil/choisir", { id });
  e.target.disabled = false;
  if (r.erreur) { alert(r.erreur); e.target.value = banc.actif; return; }
  await chargerOutil();
});

(async function () {
  if (memo("max")) $("#r-max").value = memo("max");
  if (memo("delai")) $("#r-delai").value = memo("delai");
  await rafraichirEtat();
  $("#outil").innerHTML = banc.outils.map((o) => '<option value="' + o.id + '"' + (o.installe ? "" : " disabled") + ">" +
    echapper(o.nom) + (o.installe ? "" : " (non installé)") + "</option>").join("");
  majCompteurs();
  await chargerOutil();
  setInterval(rafraichirEtat, 5000);
})();
