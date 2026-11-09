/* Sonde de web_bench : injectée en tête de chaque page de l'outil relayée par le banc.
   Elle note ce qui se passe mal (erreurs JS, promesses rejetées, requêtes en échec,
   console.error) et neutralise ce qui bloquerait un essai automatique : alert / confirm /
   prompt, impression, fenêtres et téléchargements. La page Banc lit window.__banc. */
(function () {
  "use strict";
  if (window.__banc) return;
  var B = window.__banc = { evts: [], requetes: 0, confirmer: false, debut: Date.now(), decharge: false };
  var MAX = 2000;

  function noter(type, message, detail, plus) {
    if (B.evts.length >= MAX) B.evts.shift();
    var e = { t: Date.now(), type: type, message: String(message == null ? "" : message).slice(0, 600),
              detail: detail ? String(detail).slice(0, 4000) : "", page: location.pathname };
    if (plus) for (var k in plus) e[k] = plus[k];
    B.evts.push(e);
  }
  function extrait(v, n) {
    if (v == null) return "";
    if (typeof v !== "string") { try { v = JSON.stringify(v); } catch (_) { v = String(v); } }
    return v.slice(0, n || 1500);
  }
  B.noter = noter;

  // -- erreurs
  window.addEventListener("error", function (e) {
    var cible = e.target;
    if (cible && cible !== window && (cible.src || cible.href)) {
      var url = cible.src || cible.href;
      if (!/favicon/i.test(url)) noter("ressource", "chargement échoué : " + url);
      return;
    }
    noter("js", e.message || "erreur", (e.error && e.error.stack) || (e.filename + ":" + e.lineno),
          { source: (e.filename || "").replace(location.origin, "") + ":" + e.lineno + ":" + e.colno });
  }, true);
  window.addEventListener("unhandledrejection", function (e) {
    var r = e.reason;
    noter("promesse", (r && (r.message || r)) || "promesse rejetée", r && r.stack);
  });
  var consoleErreur = console.error;
  console.error = function () {
    try { noter("console", Array.prototype.map.call(arguments, function (a) {
      return a && a.stack ? a.stack : (typeof a === "object" ? safeJson(a) : String(a)); }).join(" "), "",
      { pile: (new Error()).stack }); } catch (_) {}
    return consoleErreur.apply(this, arguments);
  };
  function safeJson(o) { try { return JSON.stringify(o).slice(0, 300); } catch (_) { return String(o); } }

  // -- réseau : le long-poll du canal pilote n'est pas une requête « en cours »
  function suivie(url) { return !/\/api\/pilote\//.test(url); }
  var fetchOrig = window.fetch;
  if (fetchOrig) {
    window.fetch = function (entree, init) {
      var url = typeof entree === "string" ? entree : (entree && entree.url) || String(entree);
      var methode = (init && init.method) || (entree && entree.method) || "GET";
      var envoye = init && typeof init.body === "string" ? extrait(init.body, 1000) : "";
      var compte = suivie(url);
      if (compte) B.requetes++;
      return fetchOrig.apply(this, arguments).then(function (r) {
        if (r.status < 400) { if (compte) B.requetes--; return r; }
        if (r.headers.get("X-Web-Bench-Bloque")) {
          if (compte) B.requetes--;
          noter("bloque", "requête arrêtée par le banc : " + methode + " " + url);
          return r;
        }
        // la réponse d'erreur du serveur dit souvent pourquoi : on la garde (sans gêner la page)
        var lu = r.clone().text().then(function (t) { return t; }, function () { return ""; });
        return lu.then(function (t) {
          if (compte) B.requetes--;
          noter(r.status >= 500 ? "reseau" : "reseau4xx", r.status + " " + methode + " " + url, "",
                { requete: { methode: methode, url: url, statut: r.status, envoye: envoye, reponse: extrait(t, 1500) },
                  pile: (new Error()).stack });
          return r;
        });
      }, function (err) {
        if (compte) B.requetes--;
        if (!(err && err.name === "AbortError") && !B.decharge) noter("reseau", "échec " + methode + " " + url + " : " + (err && err.message),
          "", { requete: { methode: methode, url: url, statut: 0, envoye: envoye, reponse: "" } });
        throw err;
      });
    };
  }
  var XO = XMLHttpRequest.prototype.open, XS = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.open = function (m, u) { this.__banc = { m: m, u: String(u) }; return XO.apply(this, arguments); };
  XMLHttpRequest.prototype.send = function (corps) {
    var x = this, info = x.__banc || { m: "?", u: "?" }, compte = suivie(info.u);
    var envoye = typeof corps === "string" ? extrait(corps, 1000) : "";
    if (compte) B.requetes++;
    x.addEventListener("loadend", function () {
      if (compte) B.requetes--;
      var req = { methode: info.m, url: info.u, statut: x.status, envoye: envoye, reponse: "" };
      try { if (!x.responseType || x.responseType === "text") req.reponse = extrait(x.responseText, 1500); } catch (_) {}
      if (x.status === 403 && x.getResponseHeader("X-Web-Bench-Bloque")) { noter("bloque", "requête arrêtée par le banc : " + info.m + " " + info.u); return; }
      if (x.status === 0) { if (!B.decharge) noter("reseau", "échec " + info.m + " " + info.u, "", { requete: req }); }
      else if (x.status >= 400) noter(x.status >= 500 ? "reseau" : "reseau4xx", x.status + " " + info.m + " " + info.u, "", { requete: req });
    });
    return XS.apply(this, arguments);
  };

  // -- ce qui bloquerait l'essai
  window.alert = function (m) { noter("dialogue", "alert : " + m); };
  window.confirm = function (m) { noter("dialogue", "confirm : " + m + " → " + (B.confirmer ? "oui" : "non")); return B.confirmer; };
  window.prompt = function (m) { noter("dialogue", "prompt : " + m + " → annulé"); return null; };
  window.print = function () { noter("info", "impression bloquée"); };
  window.open = function (u) { noter("info", "fenêtre bloquée : " + u); return null; };
  var clicLien = HTMLAnchorElement.prototype.click;
  HTMLAnchorElement.prototype.click = function () {
    if (this.hasAttribute("download") || /^blob:|^data:/.test(this.href)) {
      noter("telechargement", "téléchargement : " + (this.download || this.href.slice(0, 80)));
      return;
    }
    return clicLien.apply(this, arguments);
  };
  ["showSaveFilePicker", "showOpenFilePicker", "showDirectoryPicker"].forEach(function (nom) {
    if (window[nom]) window[nom] = function () {
      noter("info", nom + " bloqué (annulé)");
      return Promise.reject(new DOMException("annulé par le banc", "AbortError"));
    };
  });
  // une page qui s'en va coupe ses requêtes en cours (long-poll du pilote…) : ce n'est pas une panne
  window.addEventListener("beforeunload", function () { B.decharge = true; noter("navigation", "la page se décharge"); });
  window.addEventListener("pagehide", function () { B.decharge = true; });
})();
