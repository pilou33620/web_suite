//! Banc du serveur MCP (exécutable mcp_web_suite, lanceur/src/mcp/) : protocole JSON-RPC sur
//! stdio, outil etat, outils et refus.
//!
//!     cargo test -p web_suite_lanceur --test banc_mcp
//!
//! Lance le serveur en sous-processus, lui écrit des lignes sur stdin et lit ses
//! réponses sur stdout, comme le fait Claude Code : le serveur est une boîte noire. Un faux
//! WEB_CAO (serveur HTTP local) compte les requêtes reçues ; les autres outils sont sur des
//! ports fermés. Ce qu'on n'atteint pas par le protocole (la porte vers les outils appelée
//! avec une route interdite, le lanceur en lecture seule, la lecture plafonnée, un outil qui
//! plante) est vérifié par les tests de lanceur/src/mcp/mod.rs.
mod commun;

use std::cell::RefCell;
use std::io::Write;
use std::net::TcpListener;
use std::process::Command;

use commun::{echanger, msg, port_ferme, racine, verifications, verifier, Faux, AIDE_PILOTE};
use serde_json::{json, Value};
use web_suite_lanceur::mcp::py;

thread_local! {
    /// WEB_SUITE_MCP_PORTS du prochain serveur lancé.
    static PORTS: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Les ports de base, certains remplacés : c'est ce que verra le prochain serveur. Rend la table.
fn ports(base: &Value, changes: &[(&str, u16)]) -> Value {
    let mut p = base.clone();
    for (k, v) in changes {
        p[*k] = json!(v);
    }
    PORTS.with(|c| *c.borrow_mut() = p.to_string());
    p
}

fn dialoguer(lignes: &[String]) -> (Vec<Value>, i32, String) {
    echanger(lignes, Some(&PORTS.with(|c| c.borrow().clone())))
}

/// tools/call par le protocole, dans un serveur neuf (il relit WEB_SUITE_MCP_PORTS) : le résultat MCP.
/// `params` est le texte JSON des paramètres. NaN et Infinity ne sont pas du JSON : le serveur
/// refuse la ligne entière (-32700), rien n'atteint l'outil.
fn appeler_brut(params: &str) -> Value {
    let ligne = format!(r#"{{"jsonrpc": "2.0", "method": "tools/call", "id": 1, "params": {params}}}"#);
    let reponse = dialoguer(&[ligne]).0.remove(0);
    if let Some(e) = reponse.get("error") {
        return json!({"content": [{"type": "text", "text": e["message"]}], "isError": true});
    }
    reponse["result"].clone()
}

fn appeler(params: Value) -> Value {
    appeler_brut(&params.to_string())
}

fn decoder(r: &Value) -> (bool, Value) {
    let texte = r["content"][0]["text"].as_str().expect("contenu texte");
    (r["isError"].as_bool().expect("isError"), serde_json::from_str(texte).unwrap_or_else(|_| json!(texte)))
}

/// Appelle un outil MCP comme le fait Claude : (isError, contenu décodé, ou texte).
fn outil(nom: &str, args: Value) -> (bool, Value) {
    decoder(&appeler(json!({"name": nom, "arguments": args})))
}

/// Comme `outil`, les arguments donnés en texte (pour NaN et Infinity, que Python écrivait tels quels).
fn outil_brut(nom: &str, args: &str) -> (bool, Value) {
    decoder(&appeler_brut(&format!(r#"{{"name": "{nom}", "arguments": {args}}}"#)))
}

/// Le texte d'un rendu (le texte lui-même, ou le JSON écrit).
fn txt(v: &Value) -> String {
    v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string())
}

/// La colonne `i` des requêtes reçues ([méthode, chemin, corps]).
fn colonne(recues: &[Value], i: usize) -> Vec<Value> {
    recues.iter().map(|r| r[i].clone()).collect()
}

fn dernier(recues: &[Value]) -> Value {
    recues.last().expect("une requête reçue").clone()
}

/// Un objet, certaines clés remplacées ou ajoutées (dict(base, cle=valeur)).
fn avec(base: &Value, changes: Value) -> Value {
    let mut v = base.clone();
    for (k, c) in changes.as_object().unwrap() {
        v[k] = c.clone();
    }
    v
}

fn noms_de(outils: &[Value]) -> Vec<String> {
    outils.iter().map(|o| o["name"].as_str().unwrap().to_string()).collect()
}

fn description<'a>(outils: &'a [Value], nom: &str) -> &'a str {
    outils.iter().find(|o| o["name"] == nom).unwrap_or_else(|| panic!("outil {nom} absent"))["description"].as_str().unwrap()
}

/// Ce que rend la page à une commande : (statut, réponse du canal de pilotage).
fn fin(resultat: Value, etat: &str) -> (u16, Value) {
    (200, json!({"id": "c".repeat(32), "nom": "x", "etat": etat, "resultat": resultat,
                 "onglet": {"id": "A", "page": "pcb", "projet": "p", "appareil": "ce poste"}}))
}

fn finie(resultat: Value) -> (u16, Value) {
    fin(resultat, "finie")
}

const COMMANDE: &str = "/api/pilote/commande";

#[test]
fn banc() {
    let faux = Faux::outil();
    let base = json!({"web_suite": port_ferme(), "web_cao": faux.port, "web_antenna": port_ferme(), "web_3d": port_ferme()});
    ports(&base, &[]);
    let port = |cle: &str| base[cle].as_u64().unwrap() as u16;

    // Ouverture de session, notification sans réponse, ping, liste des outils
    let (reponses, code, _) = dialoguer(&[
        msg(Some(1), "initialize", Some(json!({"protocolVersion": "2025-06-18", "capabilities": {},
                                               "clientInfo": {"name": "banc", "version": "0"}}))),
        msg(None, "notifications/initialized", None),
        msg(Some(2), "ping", None),
        msg(Some(3), "tools/list", None),
    ]);
    verifier("fin de stdin : sortie propre", code, 0);
    verifier("une réponse par requête, aucune pour la notification", colonne_id(&reponses), vec![json!(1), json!(2), json!(3)]);
    let init = reponses[0]["result"].clone();
    let instructions = init["instructions"].as_str().unwrap().to_string();
    verifier("version reprise du client", &init["protocolVersion"], &json!("2025-06-18"));
    verifier("nom du serveur", &init["serverInfo"]["name"], &json!("web_suite"));
    verifier("capacité outils", init["capabilities"].get("tools").is_some(), true);
    let longueur = instructions.chars().count();
    verifier("consigne de session : présente, 1500 caractères au plus, renvoi au skill",
             (0 < longueur && longueur <= 1500, instructions.contains("web-suite")), (true, true));
    verifier("ping", &reponses[1]["result"], &json!({}));
    let outils: Vec<Value> = reponses[2]["result"]["tools"].as_array().unwrap().clone();
    verifier("chaque outil a nom, description, schéma",
             outils.iter().all(|o| ["name", "description", "inputSchema"].iter().all(|c| o.get(c).is_some())), true);

    // Version inconnue du client : le serveur propose la sienne
    let (reponses, _, _) = dialoguer(&[msg(Some(1), "initialize", Some(json!({"protocolVersion": "1999-01-01"})))]);
    verifier("version par défaut", &reponses[0]["result"]["protocolVersion"], &json!("2025-06-18"));

    // Erreurs : le serveur répond et continue
    let (reponses, code, _) = dialoguer(&[
        "ceci n'est pas du JSON".to_string(),
        msg(Some(4), "methode/inconnue", None),
        msg(Some(5), "tools/call", Some(json!({"name": "outil_inconnu", "arguments": {}}))),
        msg(Some(6), "ping", None),
    ]);
    verifier("ligne illisible", &reponses[0]["error"]["code"], &json!(-32700));
    verifier("id nul pour une ligne illisible", reponses[0].get("id"), Some(&Value::Null));
    verifier("méthode inconnue", (&reponses[1]["id"], &reponses[1]["error"]["code"]), (&json!(4), &json!(-32601)));
    verifier("outil inconnu : résultat en erreur", (&reponses[2]["id"], &reponses[2]["result"]["isError"]), (&json!(5), &json!(true)));
    verifier("toujours vivant", (&reponses[3]["id"], &reponses[3]["result"]), (&json!(6), &json!({})));
    verifier("toujours une sortie propre", code, 0);

    // Message JSON valide mais pas un objet : requête invalide
    let (reponses, _, _) = dialoguer(&["[1, 2]".to_string(), msg(Some(7), "ping", None)]);
    verifier("requête invalide", &reponses[0]["error"]["code"], &json!(-32600));
    verifier("et la suite répond", &reponses[1]["id"], &json!(7));

    // Outil etat, par le protocole : le faux WEB_CAO tourne, le reste est arrêté
    let noms = noms_de(&outils); // tools/list du tout premier échange, par le protocole
    let listes = |attendus: &[&str]| attendus.iter().all(|n| noms.iter().any(|x| x == n));
    verifier("etat est listé", listes(&["etat"]), true);
    let (reponses, _, _) = dialoguer(&[msg(Some(8), "tools/call", Some(json!({"name": "etat", "arguments": {}})))]);
    let resultat = &reponses[0]["result"];
    verifier("etat sans erreur", &resultat["isError"], &json!(false));
    let etat: Value = serde_json::from_str(resultat["content"][0]["text"].as_str().unwrap()).unwrap();
    // {outil: champ} pour chaque entrée de l'état
    let par_outil = |etat: &Value, champ: &dyn Fn(&Value) -> Value| -> Value {
        Value::Object(etat.as_array().unwrap().iter().map(|e| (e["outil"].as_str().unwrap().to_string(), champ(e))).collect())
    };
    let mut les_quatre: Vec<String> = etat.as_array().unwrap().iter().map(|e| e["outil"].as_str().unwrap().to_string()).collect();
    les_quatre.sort();
    verifier("les quatre", les_quatre, ["web_3d", "web_antenna", "web_cao", "web_suite"].map(String::from).to_vec());
    verifier("lancés", par_outil(&etat, &|e| e["lance"].clone()),
             json!({"web_suite": false, "web_cao": true, "web_antenna": false, "web_3d": false}));
    verifier("ports", par_outil(&etat, &|e| e["port"].clone()), base.clone());
    verifier("adresse", par_outil(&etat, &|e| e["adresse"].clone())["web_cao"].clone(),
             json!(format!("http://127.0.0.1:{}/", faux.port)));

    // La porte vers les outils (routes interdites, lanceur en lecture seule) : tests de lanceur/src/mcp/mod.rs.
    // Par le protocole : une lecture arrive à l'outil, en GET
    faux.oublier();
    verifier("route permise, en GET", outil("cao_projets", json!({})), (false, json!({"route": "/api/projets"})));
    verifier("requêtes reçues", faux.recues(), vec![json!(["GET", "/api/projets", null])]);

    // Le lanceur tourne et a lancé WEB_3D sur un port décalé (le sien était pris) : on le suit
    let lanceur = Faux::fixe(json!({"outils": [
        {"id": "web_3d", "phase": "actif", "port": faux.port},       // port décalé
        {"id": "web_antenna", "phase": "arret", "port": 1},          // arrêté : ignoré
        {"id": "web_cao", "phase": "actif", "port": "pas un port"},  // illisible : ignoré
    ]}).to_string().as_bytes());
    ports(&base, &[("web_suite", lanceur.port)]);
    let vus = par_outil(&outil("etat", json!({})).1, &|e| json!([e["port"], e["lance"]]));
    verifier("WEB_3D suivi sur son port décalé", &vus["web_3d"], &json!([faux.port, true]));
    verifier("outil arrêté : port du catalogue", &vus["web_antenna"], &json!([port("web_antenna"), false]));
    verifier("port illisible : port du catalogue", &vus["web_cao"][0], &json!(faux.port));
    verifier("appel vers le port décalé", &outil("web3d_projets", json!({})).1["route"], &json!("/api/projets"));
    ports(&base, &[]);

    // Pannes de l'autre côté : jamais d'exception qui casse etat ou un appel
    let pas_http = TcpListener::bind("127.0.0.1:0").unwrap();
    let port_pas_http = pas_http.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for mut c in pas_http.incoming().flatten() {
            let _ = c.write_all(b"SSH-2.0-OpenSSH\r\n");
        }
    });
    let illisible = Faux::fixe(b"{pas du json");
    let bizarre = Faux::fixe(json!({"outils": ["texte", null, {"id": "web_3d", "phase": "actif", "port": true}]}).to_string().as_bytes());
    ports(&base, &[("web_suite", bizarre.port), ("web_cao", port_pas_http), ("web_antenna", illisible.port)]);
    let vus = par_outil(&outil("etat", json!({})).1, &|e| json!([e["port"], e["lance"]]));
    verifier("entrées illisibles du lanceur ignorées", &vus["web_3d"], &json!([port("web_3d"), false]));
    verifier("port pris par un service non HTTP : pas WEB_CAO", &vus["web_cao"][1], &json!(false));
    // l'outil arrêté (port pris par autre chose) et l'outil qui répond mal : deux pannes, dites chacune
    for (nom, attendu) in [("cao_projets", "occupé par un autre service"), ("antenna_projets", "JSON illisible")] {
        let (erreur, texte) = outil(nom, json!({}));
        verifier(&format!("aurait dû échouer : {nom}"), (erreur, txt(&texte).contains(attendu)), (true, true));
    }
    let attendus = ports(&base, &[("web_suite", illisible.port)]);
    verifier("lanceur au JSON illisible : ports du catalogue", par_outil(&outil("etat", json!({})).1, &|e| e["port"].clone()), attendus);
    ports(&base, &[]);

    verifier("message d'outil arrêté", outil("web3d_projets", json!({})),
             (true, json!("Échec de web3d_projets : WEB_3D ne tourne pas : lance-le depuis WEB·SUITE")));
    // Un outil inconnu de la porte, un outil MCP qui plante : tests de lanceur/src/mcp/mod.rs

    // -----------------------------------------------------------------------
    // api-outils (lanceur/docs/SPEC-mcp-api-outils.md)
    let api = Faux::outil();
    ports(&base, &[("web_cao", api.port)]);
    verifier("outils du socle listés", listes(&["etat", "cao_projets", "cao_projet_lire", "cao_doc_lire"]), true);

    // Méthodes : GET sans corps, POST avec, ni PUT ni DELETE (la porte n'a pas d'autre méthode) ; le lanceur
    // en GET seulement : tests de lanceur/src/mcp/mod.rs

    // Lectures de projets CAO
    api.oublier();
    verifier("cao_projets", outil("cao_projets", json!({})), (false, json!({"route": "/api/projets"})));
    outil("cao_projet_lire", json!({"chemin": "a b/c", "racine": "/r"}));
    outil("cao_projet_lire", json!({"chemin": "seul"}));
    outil("cao_doc_lire", json!({"chemin": "p", "doc": "pcb"}));
    verifier("routes et méthodes", api.recues(), vec![
        json!(["GET", "/api/projets", null]),
        json!(["GET", "/api/projet?chemin=a+b%2Fc&racine=%2Fr", null]),
        json!(["GET", "/api/projet?chemin=seul", null]),
        json!(["GET", "/api/projet/doc?chemin=p&doc=pcb", null])]);

    // Arguments invalides : erreur qui nomme l'argument, aucune requête
    api.oublier();
    for (nom, args, fautif) in [("cao_projet_lire", json!({}), "chemin"), ("cao_projet_lire", json!({"chemin": 5}), "chemin"),
                                ("cao_doc_lire", json!({"chemin": "p", "doc": "autre"}), "doc"),
                                ("cao_doc_lire", json!({"chemin": "p", "doc": "pcb", "parties": "nom"}), "parties")] {
        let (erreur, texte) = outil(nom, args.clone());
        verifier(&format!("refus de {nom} {args}"), (erreur, txt(&texte).contains(fautif)), (true, true));
    }
    verifier("aucun argument invalide n'atteint l'outil", api.recues(), vec![]);

    // Grosse réponse : résumé sous le plafond, « modifie » toujours là ; « parties » pour le détail
    let gros = json!({"document": {"pistes": (0..30000).map(|i| json!([i, i])).collect::<Vec<_>>(), "nom": "carte"},
                      "fichier": "x-PCB.json", "modifie": 123});
    api.poser("GET", "/api/projet/doc", (200, gros.clone()));
    let lire_pcb = json!({"chemin": "p", "doc": "pcb"});
    let r = appeler(json!({"name": "cao_doc_lire", "arguments": lire_pcb}));
    verifier("réponse de plus de 300 Ko", gros.to_string().len() > 300 * 1024, true);
    verifier("résumé sous 100 Ko", r["content"][0]["text"].as_str().unwrap().len() < 100 * 1024, true);
    let (_, rendu) = outil("cao_doc_lire", lire_pcb.clone());
    verifier("modifie gardé", (&rendu["modifie"], &rendu["fichier"]), (&json!(123), &json!("x-PCB.json")));
    verifier("document résumé", (&rendu["document"]["resume"], &rendu["document"]["forme"]["pistes"]["longueur"]),
             (&json!(true), &json!(30000)));
    verifier("parties", &outil("cao_doc_lire", avec(&lire_pcb, json!({"parties": ["nom", "absente"]}))).1["document"],
             &json!({"nom": "carte"}));
    verifier("parties absentes signalées",
             &outil("cao_doc_lire", avec(&lire_pcb, json!({"parties": ["absente"]}))).1["parties_absentes"], &json!(["absente"]));
    api.vider_table();

    // Lectures CAO, LIB, WEB_3D (le faux outil sert aussi de WEB_3D)
    ports(&base, &[("web_cao", api.port), ("web_3d", api.port)]);
    api.poser("GET", "/api/oeil", (200, json!({"dispo": true, "gabarits": [
        {"id": "usb2", "nom": "USB 2.0", "polygone": vec![json!([0, 0]); 50], "masque": {}}]})));
    api.poser("GET", "/api/ipc2581", (500, json!({"detail": "en panne"})));
    api.oublier();
    let (erreur, solveurs) = outil("cao_solveurs", json!({}));
    verifier("cao_solveurs malgré une panne", erreur, false);
    let mut lues: Vec<String> = api.recues().iter().map(|r| r[1].as_str().unwrap().to_string()).collect();
    lues.sort();
    verifier("les trois GET que la page lit", lues, ["/api/ipc2581", "/api/oeil", "/api/simulation"].map(String::from).to_vec());
    verifier("gabarits réduits", &solveurs["oeil"]["gabarits"], &json!([{"id": "usb2", "nom": "USB 2.0"}]));
    verifier("solveur en panne marqué", solveurs["ipc2581"].get("erreur").is_some(), true);
    verifier("solveur sain intact", &solveurs["simulation"], &json!({"route": "/api/simulation"}));

    api.oublier();
    for (nom, args) in [("cao_profils", json!({})), ("cao_profil_lire", json!({"nom": "jlc 4c"})), ("lib_fichiers", json!({})),
                        ("lib_fichier_lire", json!({"type": "pcb", "nom": "R0603"})), ("lib_parasites", json!({})),
                        ("web3d_projets", json!({}))] {
        verifier(&format!("{nom} sans erreur"), outil(nom, args).0, false);
    }
    verifier("routes des lectures", colonne(&api.recues(), 1), vec![
        json!("/api/profils"), json!("/api/profil?nom=jlc+4c"), json!("/api/lib/fichiers"),
        json!("/api/lib/fichier?type=pcb&nom=R0603"), json!("/api/lib/parasites"), json!("/api/projets")]);
    api.oublier();
    verifier("type LIB inconnu refusé", outil("lib_fichier_lire", json!({"type": "autre", "nom": "x"})).0, true);
    verifier("sans requête", api.recues(), vec![]);

    api.poser("GET", "/api/lib/composants", (200, json!({
        "colonnes": ["ref", "valeur", "boitier"], "total": 120,
        "composants": (0..120).map(|i| json!({"ref": format!("R{i}"), "valeur": if i % 2 == 1 { "10k" } else { "1µF" },
                                             "boitier": "0603", "_id": i})).collect::<Vec<_>>()})));
    let (_, page) = outil("lib_composants", json!({}));
    verifier("50 par défaut", (page["composants"].as_array().unwrap().len(), &page["total"], &page["trouves"]),
             (50, &json!(120), &json!(120)));
    let (_, page) = outil("lib_composants", json!({"filtre": "10K", "debut": 10, "nombre": 5, "colonnes": ["ref"]}));
    verifier("filtre sans casse, page, colonnes", &page["composants"],
             &Value::Array((21..31).step_by(2).map(|i| json!({"ref": format!("R{i}"), "_id": i})).collect()));
    verifier("trouvés après filtre", &page["trouves"], &json!(60));
    for args in [json!({"nombre": 0}), json!({"debut": -1}), json!({"colonnes": "ref"})] {
        verifier(&format!("lib_composants refuse {args}"), outil("lib_composants", args.clone()).0, true);
    }
    api.vider_table();

    let ecritures = |api: &Faux| api.recues().into_iter().filter(|r| r[0] != "GET").collect::<Vec<_>>();
    verifier("chemin absolu refusé en lecture aussi", outil("cao_projet_lire", json!({"chemin": "/etc"})).0, true);
    api.vider_table();

    // Plafond tenu sur le texte rendu à Claude, pas seulement sur le JSON compact
    for (nom_cas, document) in [
        ("coordonnées imbriquées de 91 Ko", json!({"pts": (0..9000).map(|i| json!([[i, i + 1]])).collect::<Vec<_>>()})),
        ("100 clés de 5 Ko", Value::Object((0..100).map(|i| (format!("k{i:03}{}", "x".repeat(5000)), json!(i))).collect())),
    ] {
        api.poser("GET", "/api/projet/doc", (200, json!({"document": document, "modifie": 1})));
        let r = appeler(json!({"name": "cao_doc_lire", "arguments": {"chemin": "p", "doc": "pcb"}}));
        verifier(&format!("sous 100 Ko : {nom_cas}"), r["content"][0]["text"].as_str().unwrap().len() < 100 * 1024, true);
    }
    api.vider_table();

    // Défense en profondeur (routes « Jamais » refusées par le noyau) et lecture plafonnée (une réponse
    // énorme ne remplit pas la mémoire du Pi) : tests de lanceur/src/mcp/mod.rs
    api.poser("GET", "/api/projets", (200, json!({"dispo": true, "fichiers": [
        {"chemin": "a/b.obj"}, {"chemin": "a/b.MTL"}, {"chemin": "a/tex.png"}, {"chemin": "p.json"}, {"chemin": "c.stp"}]})));
    verifier("web3d_projets : sans les annexes, comme la page",
             outil("web3d_projets", json!({})).1["fichiers"].as_array().unwrap().iter().map(|f| f["chemin"].clone()).collect::<Vec<_>>(),
             vec![json!("a/b.obj"), json!("p.json"), json!("c.stp")]);
    api.vider_table();

    // pcbparts par le relais de WEB_CAO : seulement les outils de sa liste blanche
    api.poser("GET", "/api/tools", (200, json!({"tools": [{"name": "jlc_search"}, {"name": "autre"}], "allowed": ["jlc_search"]})));
    api.oublier();
    verifier("pcbparts_outils : la seule liste blanche", outil("pcbparts_outils", json!({})).1,
             json!({"tools": [{"name": "jlc_search"}], "allowed": ["jlc_search"]}));
    verifier("pcbparts_appeler", outil("pcbparts_appeler", json!({"name": "jlc_search", "arguments": {"q": "LM317"}})).0, false);
    verifier("appel relayé", dernier(&api.recues()), json!(["POST", "/api/tool", {"name": "jlc_search", "arguments": {"q": "LM317"}}]));
    api.oublier();
    verifier("outil hors liste refusé", outil("pcbparts_appeler", json!({"name": "autre", "arguments": {}})).0, true);
    verifier("arguments non objet refusés", outil("pcbparts_appeler", json!({"name": "jlc_search", "arguments": "x"})).0, true);
    verifier("arguments de plus de 2 Ko refusés",
             outil("pcbparts_appeler", json!({"name": "jlc_search", "arguments": {"q": "x".repeat(3000)}})).0, true);
    verifier("POST avec l'en-tête maison de WEB_CAO", api.entete("X-WEB_CAO"), Some("1".to_string()));
    verifier("seulement la lecture de la liste", ecritures(&api), vec![]);
    api.vider_table();

    // Projets WEB_ANTENNA : seulement le projet ouvert
    ports(&base, &[("web_antenna", api.port)]);
    let tete = json!({"titre": "T", "outil": "o", "source": "s", "fichier": "f", "dessin": {"d": 1}, "antenne": {"a": 1},
                      "affichage": {"z": 1}});
    api.poser("GET", "/api/projet", (200, json!({"ouvert": {"nom": "dipole", "dossier": "/x"}, "projets": []})));
    api.poser("GET", "/api/projet/lire", (200, avec(&tete, json!({"nom": "dipole", "dossier": "/x", "carte": {"c": 1}, "resultats": {"r": 1}}))));
    api.poser("GET", "/api/pilote/onglets", (200, json!({"onglets": [{"id": "A", "page": "antenna", "projet": "dipole"}]})));
    api.oublier();
    verifier("antenna_projet_lire", &outil("antenna_projet_lire", json!({})).1["nom"], &json!("dipole"));
    verifier("lecture du projet ouvert, sans POST", api.recues(),
             vec![json!(["GET", "/api/projet/lire", null]), json!(["GET", "/api/pilote/onglets", null])]);
    api.poser("GET", "/api/pilote/onglets", (200, json!({"onglets": [{"id": "A", "page": "antenna", "projet": "autre"}]})));
    let (erreur, texte) = outil("antenna_projet_lire", json!({}));
    let texte = txt(&texte);
    verifier("aucun onglet ne déclare ce projet : refus lisible", (erreur, texte.contains("onglet") && texte.contains("dipole")), (true, true));
    api.poser("GET", "/api/projet/lire", (409, json!({"detail": "Aucun projet ouvert"})));
    verifier("aucun projet ouvert : lecture refusée", outil("antenna_projet_lire", json!({})).0, true);
    api.oublier();
    outil("antenna_projets", json!({}));
    outil("antenna_calculs", json!({}));
    verifier("antenna_projets, antenna_calculs", colonne(&api.recues(), 1), vec![json!("/api/projet"), json!("/api/projet/calculs")]);
    api.vider_table();

    // openEMS : chiffrer (par la page, voir plus bas) et suivre, jamais lancer ni arrêter
    api.oublier();
    outil("antenna_openems_etat", json!({}));
    outil("antenna_openems_suivre", json!({"id": "a1b2 c3", "depuis": 3}));
    outil("antenna_openems_suivre", json!({"id": "a1b2c3"}));
    outil("antenna_openems_champs", json!({"id": "a1b2c3"}));
    verifier("état, suivi, champs", colonne(&api.recues(), 1), vec![
        json!("/api/openems"), json!("/api/openems/journal?id=a1b2+c3&depuis=3"), json!("/api/openems/journal?id=a1b2c3"),
        json!("/api/openems/champs?id=a1b2c3")]);
    verifier("suivre sans id refusé", outil("antenna_openems_suivre", json!({})).0, true);
    verifier("jamais lancer ni arrêter", api.recues().into_iter().filter(|r| {
        let chemin = r[1].as_str().unwrap();
        chemin.contains("lancer") || chemin.contains("arreter")
    }).collect::<Vec<_>>(), vec![]);
    api.vider_table();

    // -----------------------------------------------------------------------
    // canal-pilote (lanceur/docs/SPEC-mcp-canal-pilote.md) : les outils déposent une commande
    // sur /api/pilote/commande de l'outil, qui attend la réponse de la page
    ports(&base, &[("web_cao", api.port), ("web_antenna", api.port)]);
    verifier("outils du canal listés", listes(&["pilote_onglets", "pilote_suivre", "pilote_capture", "cao_calcul",
                                               "pilote_interface_lire", "pilote_interface"]), true);
    api.poser("GET", "/api/pilote/onglets", (200, json!({"onglets": [{"id": "A", "page": "pcb"}]})));
    let (erreur, vus) = outil("pilote_onglets", json!({}));
    verifier("onglets des outils lancés, outil arrêté dit", (erreur, &vus["web_cao"], &vus["web_3d"]),
             (false, &json!([{"id": "A", "page": "pcb"}]), &json!("arrêté")));
    api.poser("GET", "/api/pilote/onglets", (404, json!({"detail": "inconnue"})));
    verifier("outil sans canal (pas à jour)", txt(&outil("pilote_onglets", json!({})).1["web_cao"]).contains("canal"), true);

    api.poser("POST", COMMANDE, finie(json!({"rapport": "Z0 = 50 ohms"})));
    api.oublier();
    let (erreur, rendu) = outil("cao_calcul", json!({"analyse": "impedance"}));
    verifier("cao_calcul", (erreur, &rendu["resultat"]), (false, &json!({"rapport": "Z0 = 50 ohms"})));
    outil("cao_calcul", json!({"analyse": "crosstalk", "onglet": "B"}));
    verifier("commandes déposées", colonne(&api.recues(), 2), vec![
        json!({"page": ["pcb", "visionneuse-ipc"], "nom": "cao_calcul", "args": {"analyse": "impedance"}, "ecriture": false}),
        json!({"page": ["pcb", "visionneuse-ipc"], "nom": "cao_calcul", "args": {"analyse": "crosstalk"}, "ecriture": false,
               "onglet": "B"})]);
    api.oublier();
    for (nom, args) in [("cao_calcul", json!({"analyse": "tout"})), ("cao_calcul", json!({})),
                        ("pilote_capture", json!({"outil": "web_ferme"})), ("pilote_suivre", json!({"outil": "web_cao"}))] {
        verifier(&format!("{nom} refuse {args}"), outil(nom, args.clone()).0, true);
    }
    verifier("aucune commande refusée déposée", api.recues(), vec![]);

    // États rendus par la page : refusée n'est pas une erreur ; échouée et expirée en sont
    let geste = json!({"outil": "web_cao", "geste": "touche", "touche": "Escape"});
    api.poser("POST", COMMANDE, fin(Value::Null, "refusee"));
    verifier("refusée", outil("pilote_interface", geste.clone()).0, false);
    verifier("refusée dite", outil("pilote_interface", geste.clone()).1.to_string().contains("refus"), true);
    for etat in ["echouee", "expiree"] {
        api.poser("POST", COMMANDE, fin(json!("Repère R99 introuvable."), etat));
        let (erreur, texte) = outil("pilote_interface", geste.clone());
        let texte = txt(&texte);
        verifier(&format!("{etat} en erreur lisible"),
                 (erreur, texte.contains("R99") || texte.contains("fermé") || texte.contains("expir")), (true, true));
    }
    api.poser("POST", COMMANDE, fin(json!("x".repeat(50000)), "echouee")); // texte venu du projet : borné
    verifier("échec borné", txt(&outil("pilote_interface", geste.clone()).1).chars().count() < 2200, true);
    // Pas encore de réponse (ta carte attend) : en attente, avec l'id pour pilote_suivre
    api.poser("POST", COMMANDE, fin(Value::Null, "proposee"));
    let (erreur, att) = outil("pilote_interface", geste.clone());
    verifier("en attente avec id", (erreur, &att["etat"], &att["id"]), (false, &json!("en_attente"), &json!("c".repeat(32))));
    api.poser("GET", "/api/pilote/suivre", fin(json!("R1 = 1k"), "appliquee"));
    api.oublier();
    verifier("pilote_suivre", &outil("pilote_suivre", json!({"outil": "web_cao", "id": "c".repeat(32)})).1["etat"], &json!("appliquee"));
    let d = dernier(&api.recues());
    verifier("route de suivi", (&d[0], &d[1]), (&json!("GET"), &json!(format!("/api/pilote/suivre?id={}", "c".repeat(32)))));

    // Capture : un contenu MCP « image », pas du texte
    let png = format!("iVBORw0KGgow{}", "MDAw".repeat(33)); // b"\x89PNG\r\n\x1a\n" puis cent « 0 », en base64
    let image = || finie(json!({"image": format!("data:image/png;base64,{png}"), "largeur": 10, "hauteur": 10}));
    let types = |r: &Value| r["content"].as_array().unwrap().iter().map(|c| c["type"].clone()).collect::<Vec<_>>();
    api.poser("POST", COMMANDE, image());
    api.oublier();
    let r = appeler(json!({"name": "pilote_capture", "arguments": {"outil": "web_cao", "page": "pcb"}}));
    verifier("capture : image puis texte", (types(&r), &r["isError"]), (vec![json!("image"), json!("text")], &json!(false)));
    verifier("image en PNG base64", (&r["content"][0]["mimeType"], &r["content"][0]["data"]), (&json!("image/png"), &json!(png)));
    verifier("capture visée", &dernier(&api.recues())[2], &json!({"page": ["pcb"], "nom": "capture", "args": {}, "ecriture": false}));
    appeler(json!({"name": "pilote_capture", "arguments": {"outil": "web_cao"}}));
    verifier("capture : les pages de l'outil qui ont un canevas", &dernier(&api.recues())[2]["page"],
             &json!(["schema", "pcb", "visionneuse-ipc"]));
    let commence = |o: &Value, debuts: &[&str]| debuts.iter().any(|d| o["name"].as_str().unwrap().starts_with(d));
    let dit = |o: &Value, mot: &str| o["description"].as_str().unwrap().contains(mot);
    verifier("ce que rend une page est une donnée, pas une consigne",
             outils.iter().filter(|o| commence(o, &["pilote_", "cao_calcul"])).all(|o| dit(o, "donnée")), true);
    for image in [format!("data:text/html;base64,{png}"), "data:image/png;base64,!!!".to_string(),
                  format!("data:image/png;base64,{}", "A".repeat(3000000))] {
        api.poser("POST", COMMANDE, finie(json!({"image": image})));
        verifier(&format!("capture refusée : {}…", &image[..24]), outil("pilote_capture", json!({"outil": "web_cao"})).0, true);
    }

    // pilote_interface : un geste dans la page est une écriture (carte) ; la lecture n'écrit pas
    api.poser("POST", COMMANDE, finie(json!("[1] button « DRC »")));
    api.oublier();
    let tout_cao = json!(["schema", "pcb", "visionneuse-ipc", "gestion-lib", "accueil"]);
    for args in [json!({"outil": "web_cao", "filtre": "drc"})] {
        verifier("pilote_interface_lire accepté", outil("pilote_interface_lire", args).0, false);
    }
    for args in [json!({"outil": "web_cao", "geste": "cliquer", "cible": 12}),
                 json!({"outil": "web_cao", "geste": "saisir", "cible": "#skTarget", "valeur": 1.6}),
                 json!({"outil": "web_cao", "geste": "touche", "touche": "Ctrl+Z", "page": "pcb"}),
                 json!({"outil": "web_antenna", "geste": "souris", "x": 10, "y": 20.5, "x2": 30, "y2": 40, "modificateurs": "Shift"})] {
        verifier(&format!("pilote_interface accepté : {args}"), outil("pilote_interface", args.clone()).0, false);
    }
    verifier("pilote_interface : arguments, pages et écriture",
             api.recues().iter().map(|r| json!([r[2]["nom"], r[2]["args"], r[2]["page"], r[2]["ecriture"]])).collect::<Vec<_>>(),
             vec![json!(["interface_lire", {"filtre": "drc"}, tout_cao, false]),
                  json!(["interface", {"geste": "cliquer", "cible": 12}, tout_cao, true]),
                  json!(["interface", {"geste": "saisir", "cible": "#skTarget", "valeur": 1.6}, tout_cao, true]),
                  json!(["interface", {"geste": "touche", "touche": "Ctrl+Z"}, ["pcb"], true]),
                  json!(["interface", {"geste": "souris", "x": 10, "y": 20.5, "x2": 30, "y2": 40, "modificateurs": "Shift"}, ["antenna"], true])]);
    api.oublier();
    for args in [json!({"geste": "cliquer", "cible": 1}), json!({"outil": "web_cao"}), json!({"outil": "web_cao", "geste": "voler"}),
                 json!({"outil": "web_cao", "geste": "cliquer"}), json!({"outil": "web_cao", "geste": "cliquer", "cible": 0}),
                 json!({"outil": "web_cao", "geste": "cliquer", "cible": 1.5}), json!({"outil": "web_cao", "geste": "cliquer", "cible": true}),
                 json!({"outil": "web_cao", "geste": "saisir", "cible": 3}), json!({"outil": "web_cao", "geste": "saisir", "cible": 3, "valeur": [1]}),
                 json!({"outil": "web_cao", "geste": "touche"}), json!({"outil": "web_cao", "geste": "souris", "x": 1}),
                 json!({"outil": "web_cao", "geste": "souris", "x": 1, "y": "2"}), json!({"outil": "web_cao", "geste": "souris", "x": 1, "y": 2, "double": "oui"}),
                 json!({"outil": "web_cao", "geste": "cliquer", "cible": "a\nb"}), json!({"outil": "web_cao", "geste": "cliquer", "cible": 1, "page": "antenna"})] {
        verifier(&format!("pilote_interface refuse {args}"), outil("pilote_interface", args.clone()).0, true);
    }
    verifier("pilote_interface refuse une saisie trop longue",
             outil("pilote_interface", json!({"outil": "web_cao", "geste": "saisir", "cible": 3, "valeur": "x".repeat(500001)})).0, true);
    verifier("pilote_interface_lire refuse un filtre sur deux lignes",
             outil("pilote_interface_lire", json!({"outil": "web_cao", "filtre": "a\nb"})).0, true);
    verifier("aucun geste invalide n'atteint la page", api.recues(), vec![]);
    api.vider_table();

    // -----------------------------------------------------------------------
    // registre-3d (lanceur/docs/SPEC-mcp-registre-3d.md) : commandes de la page WEB_3D
    ports(&base, &[("web_3d", api.port)]);
    let o3 = ["web3d_scene", "web3d_vue", "web3d_visibilite", "web3d_coupe", "web3d_mesurer", "web3d_collisions", "web3d_ouvrir"];
    verifier("outils de registre-3d listés", listes(&o3), true);
    api.poser("POST", COMMANDE, finie(json!({"ok": 1})));
    api.oublier();
    let uu = "5e2d8418-4e83-48cb-804a-d5f34f62ae1f";
    for (nom, args) in [
        ("web3d_scene", json!({"filtre": "bolt", "nombre": 20})),
        ("web3d_vue", json!({"vue": "iso", "projection": "ortho", "cadrer": {"piece": "0/3/1"}})),
        ("web3d_visibilite", json!({"action": "isoler", "pieces": [uu]})),
        ("web3d_coupe", json!({"actif": true, "axe": "z", "decalage_mm": -2.5, "inverse": false})),
        ("web3d_mesurer", json!({"quoi": "distance", "pieces": ["0/1", uu], "afficher": true})),
        ("web3d_collisions", json!({"tolerance": 0.2})),
        ("web3d_ouvrir", json!({"chemins": ["cartes/boitier.stp"], "ajout": true})),
    ] {
        verifier(&format!("{nom} accepté"), outil(nom, args).0, false);
    }
    let deposees = colonne(&api.recues(), 2);
    let npe = |d: &Value| json!([d["nom"], d["page"], d["ecriture"]]);
    verifier("commandes déposées à la page 3d, avec leurs arguments", deposees.iter().map(npe).collect::<Vec<_>>(),
             o3.iter().enumerate().map(|(i, n)| json!([n, ["3d"], i == 6])).collect::<Vec<_>>());
    verifier("arguments transmis tels quels", &deposees[6]["args"], &json!({"chemins": ["cartes/boitier.stp"], "ajout": true}));
    api.oublier();
    verifier("web3d_regles accepté", outil("web3d_regles", json!({"procede": "tolerie"})).0, false);
    verifier("web3d_regles : la page conception, sans écriture",
             api.recues().iter().map(|d| json!([d[2]["nom"], d[2]["page"], d[2]["args"], d[2]["ecriture"]])).collect::<Vec<_>>(),
             vec![json!(["web3d_regles", ["conception"], {"procede": "tolerie"}, false])]);
    verifier("web3d_regles refuse un procédé inconnu", outil("web3d_regles", json!({"procede": "bois"})).0, true);
    api.oublier();
    outil("web3d_collisions", json!({}));
    verifier("collisions sans tolérance : celle de la page", &dernier(&api.recues())[2]["args"], &json!({"afficher": false}));
    outil("web3d_mesurer", json!({"piece": "0/1"}));
    verifier("dimensions d'une pièce transmises", &dernier(&api.recues())[2]["args"],
             &json!({"quoi": "dimensions", "afficher": false, "piece": "0/1"}));

    api.oublier();
    for (nom, args) in [("web3d_vue", json!({"vue": "plongee"})), ("web3d_vue", json!({"projection": "fisheye"})),
                        ("web3d_visibilite", json!({"action": "exploser"})), ("web3d_visibilite", json!({"action": "masquer", "pieces": "x"})),
                        ("web3d_coupe", json!({"axe": "w"})),
                        ("web3d_mesurer", json!({"quoi": "masse"})), ("web3d_mesurer", json!({"quoi": "distance", "pieces": [uu]})),
                        ("web3d_mesurer", json!({"quoi": "dimensions"})), ("web3d_mesurer", json!({})),
                        ("web3d_collisions", json!({"tolerance": -1})),
                        ("web3d_ouvrir", json!({"chemins": ["../web_3D.py"]})), ("web3d_ouvrir", json!({"chemins": []})),
                        ("web3d_ouvrir", json!({"chemins": ["/etc/x.stp"]})),
                        // bornes et booléens stricts (revue) : « "false" » valait vrai, 1e9 mm cassait la vue
                        ("web3d_coupe", json!({"actif": "false"})), ("web3d_coupe", json!({"decalage_mm": 5e6})),
                        ("web3d_ouvrir", json!({"chemins": ["a.stp"], "ajout": "oui"})),
                        ("web3d_scene", json!({"nombre": 5000})), ("web3d_vue", json!({"capturer": 1}))] {
        verifier(&format!("{nom} refuse {args}"), outil(nom, args.clone()).0, true);
    }
    verifier("web3d_coupe refuse {\"decalage_mm\": NaN}", outil_brut("web3d_coupe", r#"{"decalage_mm": NaN}"#).0, true);
    verifier("aucun argument invalide n'atteint la page", api.recues(), vec![]);

    // web3d_vue + capturer : la vue, puis la capture du même onglet, en image
    api.poser("POST", COMMANDE, finie(json!({"vue": "iso"})));
    api.oublier();
    api.file(vec![finie(json!({"vue": "iso", "cadre": "scène"})), image()]);
    let r = appeler(json!({"name": "web3d_vue", "arguments": {"vue": "iso", "capturer": true}}));
    verifier("web3d_vue + capturer : image puis texte", types(&r), vec![json!("image"), json!("text")]);
    verifier("la capture vise l'onglet de la vue", &api.recues()[1][2],
             &json!({"page": ["3d"], "nom": "capture", "args": {}, "ecriture": false, "onglet": "A"}));
    api.vider_table();

    // -----------------------------------------------------------------------
    // registre-cao (lanceur/docs/SPEC-mcp-registre-cao.md) : lectures, vue et DRC des pages WEB_CAO
    ports(&base, &[("web_cao", api.port)]);
    let oc = ["cao_schema_lire", "cao_pcb_lire", "cao_ipc_lire", "cao_sim_lire", "cao_vue", "cao_drc", "cao_regles"];
    verifier("outils de registre-cao listés", listes(&oc), true);
    // l'aide de la page (WEB_CAO/commun/pilote-aide.json, Pilote.aide) est tirée de ces définitions
    let contenu = String::from_utf8(Command::new(AIDE_PILOTE).arg("--stdout").output().unwrap().stdout).unwrap();
    let aide: serde_json::Map<String, Value> = outils.iter().filter(|o| commence(o, &["cao_", "lib_"]))
        .map(|o| (o["name"].as_str().unwrap().to_string(), json!({"description": o["description"], "arguments": o["inputSchema"]}))).collect();
    // l'écriture de Python (indentation 1, clés triées) : py::dumps_indente_trie, essayée dans lanceur/src/mcp/py.rs
    verifier("aide_pilote : le fichier que tirait aide_pilote.py des mêmes définitions, à l'octet", contenu.clone(),
             py::dumps_indente_trie(&json!({"source": "web_suite/lanceur/aide_pilote.py (définitions du MCP)", "outils": aide}), 1) + "\n");
    let fichier_aide = racine().join("WEB_CAO").join("commun").join("pilote-aide.json");
    if fichier_aide.parent().unwrap().exists() {
        verifier("pilote-aide.json à jour (sinon : l'exécutable aide_pilote)", std::fs::read_to_string(&fichier_aide).unwrap() == contenu, true);
    }
    verifier("cao_thermique listé, une lecture de la page PCB (donnée)",
             listes(&["cao_thermique"]) && description(&outils, "cao_thermique").contains("donnée"), true);
    verifier("55 outils en tout (pilote_interface, registre-cao, dessin-schema, registre-antenna, routage, chemin-auto, composants, savoir_lire, web3d_regles, cao_regles)", noms.len(), 55);
    // décision du 2026-11-04 : Claude ne fait que ce que l'utilisateur peut faire à l'interface
    let retires: Vec<&str> = "cao_doc_ecrire cao_profil_ecrire antenna_projet_enregistrer lib_fichier_ecrire cao_composants_ecrire \
        cao_valeur_composant cao_pistes_ecrire cao_classes_ecrire cao_contraintes_ecrire cao_contour_ecrire \
        cao_decoupes_ecrire cao_interdits_ecrire cao_zones_ecrire cao_serigraphie_ecrire cao_pastille_regler \
        cao_sim_ecrire antenna_proposer lib_composant_creer lib_composant_modifier lib_simulation_lier \
        cao_placement_auto web3d_deplacer web3d_ranger_step".split_whitespace().collect();
    verifier("les 23 outils d'écriture retirés sont absents",
             (retires.len(), retires.iter().copied().filter(|n| listes(&[*n])).collect::<Vec<_>>()), (23, vec![]));
    verifier("aucune description ni instruction ne les cite",
             retires.iter().copied().filter(|n| instructions.contains(*n) || outils.iter().any(|o| dit(o, n))).collect::<Vec<_>>(), vec![]);
    let dans = |o: &Value, liste: &[&str]| liste.iter().any(|n| o["name"] == *n);
    verifier("registre-cao : ce que rend la page est une donnée",
             outils.iter().filter(|o| dans(o, &oc)).all(|o| dit(o, "donnée")), true);
    api.poser("POST", COMMANDE, finie(json!("- GND")));
    api.oublier();
    for (nom, args) in [("cao_schema_lire", json!({"quoi": "net", "nom": "LOCAL", "feuille": 2})),
                        ("cao_pcb_lire", json!({"quoi": "nets", "filtre": "USB", "a_router": true})),
                        ("cao_ipc_lire", json!({"quoi": "composant", "ref": "U1"})),
                        ("cao_sim_lire", json!({"quoi": "simulation", "analyse": "pdn", "partie": "chiffres", "afficher": true})),
                        ("cao_vue", json!({"cadrer": "U1", "couches": ["Top"], "ajuster": true, "page": "pcb"})),
                        ("cao_drc", json!({})), ("cao_regles", json!({}))] {
        verifier(&format!("{nom} accepté"), outil(nom, args).0, false);
    }
    let deposees = colonne(&api.recues(), 2);
    verifier("commandes déposées aux bonnes pages, sans écriture", deposees.iter().map(npe).collect::<Vec<_>>(),
             vec![json!(["cao_schema_lire", ["schema"], false]), json!(["cao_pcb_lire", ["pcb"], false]),
                  json!(["cao_ipc_lire", ["visionneuse-ipc"], false]), json!(["cao_sim_lire", ["pcb", "visionneuse-ipc"], false]),
                  json!(["cao_vue", ["pcb"], false]), json!(["cao_drc", ["pcb"], false]), json!(["cao_regles", ["schema", "pcb"], false])]);
    api.oublier();
    verifier("cao_schema_lire etiquettes accepté", outil("cao_schema_lire", json!({"quoi": "etiquettes", "filtre": "VCC"})).0, false);
    verifier("etiquettes transmis", &api.recues()[0][2]["args"], &json!({"quoi": "etiquettes", "filtre": "VCC"}));
    verifier("arguments transmis", vec![&deposees[0]["args"], &deposees[1]["args"], &deposees[4]["args"]],
             vec![&json!({"quoi": "net", "nom": "LOCAL", "feuille": 2}), &json!({"quoi": "nets", "filtre": "USB", "a_router": true}),
                  &json!({"cadrer": "U1", "couches": ["Top"], "ajuster": true})]);
    outil("cao_vue", json!({"surligner": "GND"}));
    verifier("cao_vue sans page : les trois pages qui ont une vue", &dernier(&api.recues())[2]["page"], &json!(["schema", "pcb", "visionneuse-ipc"]));
    api.oublier();
    verifier("catalogue de simulation lisible (savoir-metier)", outil("cao_sim_lire", json!({"quoi": "catalogue"})).0, false);
    verifier("catalogue : page PCB ou visionneuse, sans écriture",
             api.recues().iter().map(|r| json!([r[2]["nom"], r[2]["args"], r[2]["ecriture"]])).collect::<Vec<_>>(),
             vec![json!(["cao_sim_lire", {"quoi": "catalogue"}, false])]);

    api.oublier();
    for (nom, args) in [("cao_pcb_lire", json!({})), ("cao_pcb_lire", json!({"quoi": "feuilles"})), ("cao_schema_lire", json!({"quoi": "drc"})),
                        ("cao_ipc_lire", json!({"quoi": "classes"})), ("cao_sim_lire", json!({"quoi": "simulation", "partie": "tout"})),
                        ("cao_pcb_lire", json!({"quoi": "nets", "a_router": "oui"})), ("cao_pcb_lire", json!({"quoi": "net", "nom": 3})),
                        ("cao_pcb_lire", json!({"quoi": "nets", "filtre": "x".repeat(201)})), ("cao_pcb_lire", json!({"quoi": "net", "nom": "a\nb"})),
                        ("cao_schema_lire", json!({"quoi": "net", "nom": "X", "feuille": 0})),
                        ("cao_schema_lire", json!({"quoi": "net", "nom": "X", "feuille": "2"})),
                        ("cao_vue", json!({})), ("cao_vue", json!({"page": "accueil", "ajuster": true})), ("cao_vue", json!({"couches": "Top"})),
                        ("cao_vue", json!({"couches": vec!["x"; 65]})), ("cao_vue", json!({"couches": [null]})), ("cao_vue", json!({"ajuster": 1})),
                        ("cao_vue", json!({"capturer": "oui", "ajuster": true})),
                        ("cao_sim_lire", json!({"quoi": "simulations", "afficher": "true"}))] {
        verifier(&format!("{nom} refuse {args}"), outil(nom, args.clone()).0, true);
    }
    verifier("aucun argument invalide n'atteint la page (cao)", api.recues(), vec![]);

    // cao_vue + capturer : la vue, puis la capture du même onglet
    api.file(vec![finie(json!("Vue : cadré sur U1.")), image()]);
    let r = appeler(json!({"name": "cao_vue", "arguments": {"cadrer": "U1", "capturer": true}}));
    verifier("cao_vue + capturer : image puis texte", types(&r), vec![json!("image"), json!("text")]);
    let d = dernier(&api.recues());
    verifier("la capture vise l'onglet de la vue (cao)", (&d[2]["nom"], &d[2]["onglet"]), (&json!("capture"), &json!("A")));

    ports(&base, &[("web_cao", api.port)]);
    api.poser("POST", COMMANDE, fin(json!("ok"), "appliquee"));
    api.oublier();

    // dessin-schema (lanceur/docs/SPEC-mcp-dessin-schema.md) : une liste d'opérations, une carte
    let d_outil = description(&outils, "cao_schema_dessiner");
    verifier("cao_schema_dessiner : donnée, carte, 40", ["donnée", "carte", "40"].iter().all(|t| d_outil.contains(t)), true);
    let ops = json!([{"op": "feuille", "creer": "Alim"}, {"op": "poser", "type": "regulator", "ref": "U1", "x": 0, "y": 0, "rotation": 90},
        {"op": "poser", "catalogue": "C0402_100NF", "feuille": 3}, {"op": "relier", "de": "U1.IN", "a": "C1.1"},
        {"op": "etiqueter", "broche": "U1.GND", "net": "GND", "genre": "gnd"}, {"op": "deplacer", "ref": "U1", "dx": 5, "dy": 0},
        {"op": "tourner", "ref": "U1", "angle": 180}, {"op": "miroir", "ref": "C1"}, {"op": "supprimer_fil", "de": "U1.IN", "a": "C1.1"},
        {"op": "supprimer", "ref": "C1"}, {"op": "feuille", "renommer": 1, "nom": "Entrée"}, {"op": "renumeroter"},
        {"op": "cadre", "x1": 0, "y1": 0, "x2": 30, "y2": 20, "titre": "Note", "zone": false},
        {"op": "fil", "points": [{"x": 2, "y": 0}, {"x": 8, "y": 0}, {"x": 8, "y": 5}]}]);
    api.oublier();
    verifier("cao_schema_dessiner accepté", outil("cao_schema_dessiner", json!({"operations": ops})).0, false);
    let d = dernier(&api.recues())[2].clone();
    verifier("déposé au schéma, en écriture, opérations telles quelles", json!([d["nom"], d["page"], d["ecriture"], d["args"]]),
             json!(["cao_schema_dessiner", ["schema"], true, {"lot": ops}]));
    api.oublier();
    let court = |v: &Value| v.to_string().chars().take(50).collect::<String>();
    for ops in [json!([]), json!(vec![json!({"op": "poser", "type": "resistor"}); 41]), json!([{"op": "voler"}]), json!(["poser"]),
                json!([{"type": "resistor"}]),
                json!([{"op": "poser", "type": "resistor", "angle": 90}]), json!([{"op": "poser", "type": "resistor", "rotation": 45}]),
                json!([{"op": "tourner", "ref": "R1", "angle": 90.0}]), json!([{"op": "relier", "de": "R1.1", "a": 2}]),
                json!([{"op": "etiqueter", "broche": "R1.1", "net": "X", "genre": "bleu"}]), json!([{"op": "poser", "type": "r", "feuille": 0}]),
                json!([{"op": "renumeroter", "feuille": 1}]),
                json!([{"op": "relier", "de": "R1.1\nR2.1", "a": "R3.1"}]), json!([{"op": "fil", "points": [{"x": 0, "y": 0}]}]),
                json!([{"op": "fil", "points": [{"x": 0, "y": 0}, {"x": 1}]}]), json!([{"op": "fil", "points": "0,0 1,0"}])] {
        verifier(&format!("cao_schema_dessiner refuse {}", court(&ops)), outil("cao_schema_dessiner", json!({"operations": ops})).0, true);
    }
    verifier("cao_schema_dessiner refuse une abscisse infinie",
             outil_brut("cao_schema_dessiner", r#"{"operations": [{"op": "poser", "type": "r", "x": Infinity, "y": 0}]}"#).0, true);
    verifier("aucune opération invalide n'atteint la page", api.recues(), vec![]);

    // registre-antenna (lanceur/docs/SPEC-mcp-registre-antenna.md) : lectures et vue de la page WEB_ANTENNA
    ports(&base, &[("web_antenna", api.port)]);
    let oa = ["antenna_lire", "antenna_vue", "antenna_openems_chiffrer"];
    verifier("outils de registre-antenna listés", listes(&oa), true);
    verifier("registre-antenna : donnée, jamais de calcul",
             outils.iter().filter(|o| dans(o, &oa)).all(|o| dit(o, "donnée") && dit(o, "calcul")), true);
    api.poser("POST", COMMANDE, finie(json!("f0 2.45 GHz")));
    api.oublier();
    for (nom, args) in [("antenna_lire", json!({"quoi": "courbe", "grandeur": "z"})), ("antenna_lire", json!({"quoi": "reglages", "filtre": "bande"})),
                        ("antenna_lire", json!({"quoi": "composant", "ref": "U1"})),
                        ("antenna_vue", json!({"vue": "3d", "maillage": false, "resultats": "s21", "etape": 2, "panneau": "resultats", "toile": "3d"}))] {
        verifier(&format!("{nom} accepté"), outil(nom, args).0, false);
    }
    let deposees = colonne(&api.recues(), 2);
    verifier("antenna : pages, noms, sans écriture", deposees.iter().map(npe).collect::<Vec<_>>(),
             vec![json!(["antenna_lire", ["antenna"], false]), json!(["antenna_lire", ["antenna"], false]),
                  json!(["antenna_lire", ["antenna"], false]), json!(["antenna_vue", ["antenna"], false])]);
    verifier("« resultats » arrive à la page comme onglet de résultats", &deposees[3]["args"],
             &json!({"vue": "3d", "maillage": false, "onglet": "s21", "etape": 2, "panneau": "resultats", "toile": "3d"}));
    api.oublier();
    for (nom, args) in [("antenna_lire", json!({})), ("antenna_lire", json!({"quoi": "tout"})), ("antenna_lire", json!({"quoi": "courbe", "grandeur": "phase"})),
                        ("antenna_lire", json!({"quoi": "nets", "filtre": "a\nb"})), ("antenna_vue", json!({})), ("antenna_vue", json!({"vue": "4d"})),
                        ("antenna_vue", json!({"maillage": "oui"})), ("antenna_vue", json!({"resultats": "phase"})), ("antenna_vue", json!({"etape": 0})),
                        ("antenna_vue", json!({"point_balayage": 1.5})), ("antenna_vue", json!({"toile": "pdf"})), ("antenna_vue", json!({"capturer": true}))] {
        verifier(&format!("{nom} refuse {args}"), outil(nom, args.clone()).0, true);
    }
    verifier("aucun argument invalide n'atteint la page (antenna)", api.recues(), vec![]);
    let quois = ["modele", "balayage", "tableau_s", "script"];
    for quoi in quois {
        verifier(&format!("chiffrer {quoi}"), outil("antenna_openems_chiffrer", json!({"quoi": quoi, "document": {"libre": 1}})).0, false);
    }
    verifier("chiffrer : par la page, sans écriture ; un « document » libre ne passe plus",
             api.recues().iter().map(|r| json!([r[2]["nom"], r[2]["args"], r[2]["ecriture"]])).collect::<Vec<_>>(),
             quois.iter().map(|q| json!(["antenna_openems_chiffrer", {"quoi": q}, false])).collect::<Vec<_>>());
    verifier("POST avec l'en-tête maison de WEB_ANTENNA", api.entete("X-WEB_ANTENNA"), Some("1".to_string()));
    api.oublier();
    for args in [json!({"quoi": "lancer"}), json!({"quoi": "balayage/lancer"}), json!({})] {
        verifier(&format!("chiffrer refuse {args}"), outil("antenna_openems_chiffrer", args.clone()).0, true);
    }
    verifier("rien d'envoyé", api.recues(), vec![]);
    api.file(vec![finie(json!("Vue : vue 3D.")), image()]);
    let r = appeler(json!({"name": "antenna_vue", "arguments": {"vue": "3d", "capturer": true}}));
    verifier("antenna_vue + capturer : image puis texte", types(&r), vec![json!("image"), json!("text")]);
    let d = dernier(&api.recues());
    verifier("la capture vise l'onglet de la vue (antenna)", (&d[2]["nom"], &d[2]["onglet"]), (&json!("capture"), &json!("A")));

    // routage (lanceur/docs/SPEC-mcp-routage.md) : routes par points de passage, paires, scores
    ports(&base, &[("web_cao", api.port)]);
    let or = ["cao_router", "cao_paire", "cao_score"];
    verifier("outils du routage listés", listes(&or), true);
    verifier("routage : donnée ; les écritures disent la carte et l'aperçu",
             outils.iter().filter(|o| dans(o, &or)).all(|o| dit(o, "donnée") && (o["name"] == "cao_score" || dit(o, "aperçu"))), true);
    api.poser("POST", COMMANDE, finie(json!("Score :\n- vias : 2\n- total indicatif : 12 (10 par liaison)")));
    verifier("cao_score : sans le total indicatif, que la page n'affiche pas", &outil("cao_score", json!({"quoi": "routage"})).1["resultat"],
             &json!("Score :\n- vias : 2"));
    api.poser("POST", COMMANDE, fin(json!("ok"), "appliquee"));
    api.oublier();
    let r1 = json!({"depart": "U1.3", "points": [{"x": 10, "y": 5.5}, {"x": 12, "y": 5.5, "couche": "Bottom"}], "arrivee": "R2.1", "largeur": 0.3});
    for (nom, args) in [("cao_router", json!({"routes": [r1], "vias": [{"x": 1, "y": 2, "net": "GND"}], "mode": "contourner"})),
                        ("cao_paire", json!({"paire": "USB", "depart": {"x": 7, "y": 19.7}, "arrivee": "U1.4"})),
                        ("cao_score", json!({"quoi": "routage"})),
                        ("cao_pcb_lire", json!({"quoi": "zone", "x1": 0, "y1": 0, "x2": 20, "y2": 10.5})),
                        ("cao_pcb_lire", json!({"quoi": "liaisons", "filtre": "GND"}))] {
        verifier(&format!("{nom} accepté"), outil(nom, args).0, false);
    }
    let deposees = colonne(&api.recues(), 2);
    verifier("routage : pages et écriture", deposees.iter().map(npe).collect::<Vec<_>>(),
             vec![json!(["cao_router", ["pcb"], true]), json!(["cao_paire", ["pcb"], true]),
                  json!(["cao_score", ["pcb"], false]), json!(["cao_pcb_lire", ["pcb"], false]), json!(["cao_pcb_lire", ["pcb"], false])]);
    verifier("la route arrive telle quelle", &deposees[0]["args"]["routes"][0], &r1);
    verifier("la fenêtre de zone arrive", &deposees[3]["args"], &json!({"quoi": "zone", "x1": 0, "y1": 0, "x2": 20, "y2": 10.5}));
    api.oublier();
    let court = |v: &Value| v.to_string().chars().take(60).collect::<String>();
    let p12 = json!({"x": 1, "y": 2});
    for (nom, args) in [("cao_router", json!({})), ("cao_router", json!({"routes": vec![r1.clone(); 21]})), ("cao_router", json!({"routes": [{"depart": "U1.3"}]})),
                        ("cao_router", json!({"routes": [avec(&r1, json!({"depart": {"x": "a", "y": 1}}))]})),
                        ("cao_router", json!({"routes": [avec(&r1, json!({"points": [{"x": 1, "y": 2, "z": 3}]}))]})),
                        ("cao_router", json!({"routes": [avec(&r1, json!({"points": vec![p12.clone(); 201]}))]})),
                        ("cao_router", json!({"routes": [r1], "mode": "sauter"})), ("cao_router", json!({"vias": [{"x": 1, "y": 2}]})),
                        ("cao_paire", json!({"paire": "USB", "depart": "J1.1"})), ("cao_paire", avec(&r1, json!({"paire": "USB", "largeur": 0.2}))),
                        ("cao_score", json!({"quoi": "tout"})), ("cao_pcb_lire", json!({"quoi": "zone", "x1": 5000})),
                        ("cao_pcb_lire", json!({"quoi": "zone", "x1": "5"})),
                        ("cao_router", json!({"routes": [avec(&r1, json!({"depart": {"x": 5000, "y": 1}}))]})),
                        ("cao_router", json!({"routes": vec![avec(&r1, json!({"points": vec![p12.clone(); 200]})); 6]}))] {
        verifier(&format!("{nom} refuse {}", court(&args)), outil(nom, args).0, true);
    }
    verifier("cao_pcb_lire refuse {\"quoi\": \"zone\", \"x1\": NaN}", outil_brut("cao_pcb_lire", r#"{"quoi": "zone", "x1": NaN}"#).0, true);
    verifier("aucun argument invalide n'atteint la page (routage)", api.recues(), vec![]);

    // chemin-auto (lanceur/docs/SPEC-mcp-chemin-auto.md) : des points de passage proposés, une lecture
    let li = json!({"depart": "U1.3", "arrivee": {"x": 12, "y": 5.5, "couche": "Bottom"}, "largeur": 0.25});
    verifier("cao_chemin accepté", outil("cao_chemin", json!({"liaisons": [li], "pas": 0.2})).0, false);
    verifier("cao_chemin : page PCB, sans écriture, arguments tels quels",
             api.recues().iter().map(|r| json!([r[2]["nom"], r[2]["page"], r[2]["ecriture"], r[2]["args"]])).collect::<Vec<_>>(),
             vec![json!(["cao_chemin", ["pcb"], false, {"liaisons": [li], "pas": 0.2}])]);
    verifier("cao_chemin : la description renvoie à cao_router", description(&outils, "cao_chemin").contains("cao_router"), true);
    api.oublier();
    let lp = json!({"paire": "USB", "depart": ["J1.2", "J1.3"], "arrivee": ["U1.5", "U1.6"]});
    verifier("cao_chemin accepte une paire", outil("cao_chemin", json!({"liaisons": [li, lp]})).0, false);
    verifier("la paire arrive telle quelle", &api.recues()[0][2]["args"], &json!({"liaisons": [li, lp]}));
    api.oublier();
    for args in [json!({"liaisons": [avec(&lp, json!({"depart": "J1.2"}))]}), json!({"liaisons": [avec(&lp, json!({"depart": ["J1.2"]}))]}),
                 json!({"liaisons": [avec(&lp, json!({"depart": ["J1.2", {"x": 1, "y": 2}]}))]}), json!({"liaisons": [avec(&lp, json!({"largeur": 0.2}))]}),
                 json!({"liaisons": [avec(&lp, json!({"paire": ""}))]}), json!({"liaisons": [{"paire": "USB", "depart": ["J1.2", "J1.3"]}]})] {
        verifier(&format!("cao_chemin refuse la paire {}", court(&args)), outil("cao_chemin", args).0, true);
    }
    for args in [json!({}), json!({"liaisons": []}), json!({"liaisons": vec![li.clone(); 21]}), json!({"liaisons": [{"depart": "U1.3"}]}),
                 json!({"liaisons": [avec(&li, json!({"arrivee": {"x": 5000, "y": 1}}))]}), json!({"liaisons": [avec(&li, json!({"autre": 1}))]}),
                 json!({"liaisons": [li], "pas": 0}), json!({"liaisons": [li], "pas": "0.2"}), json!({"liaisons": [li], "pas": 3})] {
        verifier(&format!("cao_chemin refuse {}", court(&args)), outil("cao_chemin", args).0, true);
    }
    verifier("aucun argument invalide n'atteint la page (chemin-auto)", api.recues(), vec![]);

    // composants (lanceur/docs/SPEC-mcp-composants.md) : calculs dans Gestion LIB, sans écriture
    let oc = ["lib_empreinte_generer", "lib_empreinte_verifier", "lib_kicad_convertir", "lib_brochage_verifier"];
    verifier("outils des composants listés", listes(&oc), true);
    verifier("composants : méthode et donnée dans chaque description",
             outils.iter().filter(|o| dans(o, &oc)).all(|o| dit(o, "lib_empreinte_generer") && dit(o, "Gestion LIB") && dit(o, "données")), true);
    api.poser("POST", COMMANDE, fin(json!("ok"), "appliquee"));
    api.oublier();
    let fp = json!({"format": "pcbfp-1", "name": "SOT-23-5-LDO", "pads": [{"n": 1, "x": 0, "y": 0, "w": 1, "h": 1}]});
    for (nom, args) in [("lib_empreinte_generer", json!({"famille": "soic", "parametres": {"broches": 8, "empattement": [5.8, 6.2], "corps": [3.9, 4.9], "densite": "N"}})),
                        ("lib_empreinte_verifier", json!({"empreinte": fp})),
                        ("lib_kicad_convertir", json!({"texte": "(footprint \"X\")"})),
                        ("lib_brochage_verifier", json!({"brochage": "IN=1,GND=2,OUT=5", "empreinte": "SOT-23-5", "symbole": "regulator"}))] {
        verifier(&format!("{nom} accepté"), outil(nom, args).0, false);
    }
    let deposees = colonne(&api.recues(), 2);
    verifier("composants : page Gestion LIB, aucune écriture", deposees.iter().map(npe).collect::<Vec<_>>(),
             oc.iter().map(|n| json!([n, ["gestion-lib"], false])).collect::<Vec<_>>());
    api.oublier();
    for (nom, args) in [("lib_empreinte_generer", json!({"famille": "bga"})), ("lib_empreinte_generer", json!({"famille": "soic", "parametres": {"script": 1}})),
                        ("lib_empreinte_generer", json!({"famille": "soic", "parametres": {"corps": [1, 2, 3]}})),
                        ("lib_empreinte_generer", json!({"famille": "soic", "parametres": "SOIC-8"})),
                        ("lib_empreinte_verifier", json!({"empreinte": "SOT-23"})),
                        ("lib_empreinte_verifier", json!({"empreinte": {"pads": vec!["x".repeat(1000); 600]}})),
                        ("lib_kicad_convertir", json!({"texte": "x".repeat(200 * 1024 + 1)})), ("lib_kicad_convertir", json!({"texte": "  "})),
                        ("lib_brochage_verifier", json!({"brochage": "IN=1;GND=2", "empreinte": "X", "symbole": "regulator"})),
                        ("lib_brochage_verifier", json!({"brochage": "IN=1", "empreinte": 5, "symbole": "regulator"})),
                        ("lib_brochage_verifier", json!({"brochage": "IN=1", "empreinte": "X", "symbole": ""}))] {
        verifier(&format!("{nom} refuse {}", court(&args)), outil(nom, args).0, true);
    }
    verifier("lib_empreinte_generer refuse un pas infini",
             outil_brut("lib_empreinte_generer", r#"{"famille": "soic", "parametres": {"pas": Infinity}}"#).0, true);
    verifier("aucun argument invalide n'atteint la page (composants)", api.recues(), vec![]);

    // le skill servi par le MCP (hors du dépôt) : une liste fermée de fiches, rien d'autre
    let savoir = outils.iter().find(|o| o["name"] == "savoir_lire").unwrap();
    let fiches: Vec<&str> = savoir["inputSchema"]["properties"]["fiche"]["enum"].as_array().unwrap().iter().map(|f| f.as_str().unwrap()).collect();
    verifier("savoir_lire : les fiches du skill",
             fiches.iter().copied().filter(|f| !["conception-3d", "conception-electronique"].contains(f)).collect::<Vec<_>>(),
             vec!["SKILL", "antenne", "composants", "pcb", "routage", "schema", "simulation"]);
    if racine().join("WEB_3D").join("regles-conception.md").is_file() {
        // WEB_3D est un autre dépôt, absent de la CI de web_suite
        let (erreur, texte) = outil("savoir_lire", json!({"fiche": "conception-3d"}));
        verifier("savoir_lire conception-3d : le fichier de WEB_3D", (erreur, txt(&texte).contains("RÈGLES DE CONCEPTION")), (false, true));
    } else {
        verifier("savoir_lire conception-3d : refusée sans WEB_3D", outil("savoir_lire", json!({"fiche": "conception-3d"})).0, true);
    }
    if racine().join("WEB_CAO").join("docs").join("regles").join("index.md").is_file() {
        // WEB_CAO est un autre dépôt
        let (erreur, texte) = outil("savoir_lire", json!({"fiche": "conception-electronique"}));
        let texte = txt(&texte);
        verifier("savoir_lire conception-electronique : les chapitres de WEB_CAO/docs/regles, index d'abord",
                 (erreur, texte.starts_with("# Règles de conception WEB_CAO"), texte.contains("# Routage")), (false, true, true));
    } else {
        verifier("savoir_lire conception-electronique : refusée sans WEB_CAO", outil("savoir_lire", json!({"fiche": "conception-electronique"})).0, true);
    }
    verifier("la consigne de session impose les règles électroniques",
             instructions.contains("conception-electronique") && instructions.contains("cao_regles"), true);
    let (erreur, texte) = outil("savoir_lire", json!({"fiche": "pcb"}));
    verifier("savoir_lire pcb", (erreur, txt(&texte).starts_with("# PCB")), (false, true));
    for f in [json!("../../README"), json!("SKILL.md"), json!("/etc/passwd"), Value::Null] {
        verifier(&format!("savoir_lire refuse {f}"), outil("savoir_lire", json!({"fiche": f})).0, true);
    }

    println!("OK : {} vérifications", verifications());
}

fn colonne_id(reponses: &[Value]) -> Vec<Value> {
    reponses.iter().map(|r| r["id"].clone()).collect()
}
