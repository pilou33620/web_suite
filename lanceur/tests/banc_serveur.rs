//! Banc du serveur du lanceur : Host, jeton du mode réseau, en-tête X-WebSuite.
//!
//!     cargo test -p web_suite_lanceur --test banc_serveur
//!
//! Pilote l'exécutable web_suite en boîte noire, sur 127.0.0.1, dans un dossier
//! temporaire sans aucun outil (rien n'est lancé, cloné ni poussé : un dépôt nu local
//! joue GitHub). L'appareil distant est joué par WEB_TOOLS_BANC_DISTANT=1 : le serveur
//! ne fait alors plus confiance à la boucle locale.
//!
//! Ce que l'ancien banc vérifiait en touchant l'intérieur du serveur est dans les tests
//! unitaires :
//!   - client d'Internet refusé (8.8.8.8, 2a01:…, ::ffff:8.8.8.8), fin de la pause après
//!     cinq erreurs, code expiré refusé et remplacé : commun/src/appairage/essais.rs ;
//!   - journal tourné en .log.1, pas de bouton openEMS hors Windows, archive openEMS à
//!     l'empreinte fausse refusée, code d'appairage lu dans le journal d'un outil :
//!     lanceur/src/outils/essais.rs.
mod commun;

use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use commun::{chaine, http, verifications, verifier, Lanceur, Poste, Rep, WEB_SUITE};
use serde_json::json;

const X: (&str, &str) = ("X-WebSuite", "1");

fn arret_propre(l: &mut Lanceur) {
    #[cfg(unix)] // sous Windows, l'arrêt d'un processus le tue sans prévenir
    {
        let code = l.arreter();
        verifier("SIGTERM : arrêt propre", (code, l.sortie().contains("Arrêt.")), (0, true));
    }
    #[cfg(not(unix))]
    let _ = l;
}

/// GET en HTTPS, le certificat vérifié par la CA du poste.
fn https(port: u16, chemin: &str, ca: &Path) -> Rep {
    let mut racines = rustls::RootCertStore::empty();
    for c in rustls_pemfile::certs(&mut &fs::read(ca).unwrap()[..]) {
        racines.add(c.unwrap()).unwrap();
    }
    let cfg = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(racines)
        .with_no_client_auth();
    let nom = rustls::pki_types::ServerName::try_from("127.0.0.1").unwrap();
    let conn = rustls::ClientConnection::new(Arc::new(cfg), nom).unwrap();
    let mut c = rustls::StreamOwned::new(conn, http::connecter(port).unwrap());
    http::echanger(&mut c, port, "GET", chemin, &[], b"").unwrap()
}

fn biscuit_de(r: &Rep) -> String {
    r.entete("Set-Cookie").expect("Set-Cookie").split(';').next().unwrap().to_string()
}

#[test]
fn banc() {
    let poste = Poste::new("serveur");

    // Écoute locale : pas de jeton, mais Host contrôlé
    {
        let mut l = poste.lanceur(&["--local"], &[]);
        let arreter: &[u8] = br#"{"id":"web_3d"}"#;
        verifier("page locale", l.get("/").code, 200);
        verifier("Host étranger", l.req("GET", "/", &[("Host", "evil.example:8100")], b"").code, 403);
        verifier("Host étranger en POST", l.req("POST", "/api/arreter", &[("Host", "evil.example"), X], arreter).code, 403);
        verifier("sans X-WebSuite", l.req("POST", "/api/arreter", &[], arreter).code, 403);
        verifier("corps géant", l.req("POST", "/api/arreter", &[X, ("Content-Length", "1000000000")], b"").code, 413);
        verifier("JSON non objet", l.req("POST", "/api/arreter", &[X], b"[]").code, 400);
        arret_propre(&mut l);
    }

    // Mode réseau, appareil distant
    {
        let mut l = poste.lanceur(&["--reseau"], &[("WEB_TOOLS_BANC_DISTANT", "1"), ("WEB_TOOLS_HTTP", "1")]);
        let arreter: &[u8] = br#"{"id":"web_3d"}"#;
        let jeton = poste.jeton();
        let r = l.get("/");
        verifier("distant sans jeton : renvoyé vers la page du code", (r.code, r.entete("Location")), (303, Some("/appairage")));
        let r = l.get("/appairage");
        verifier("distant sans jeton : page du code", (r.code, r.texte().contains("appairer")), (200, true));
        verifier("page du code jamais en cache", r.entete("Cache-Control"), Some("no-store"));
        verifier("le code s'affiche dans le terminal", l.codes().len(), 1);
        verifier("distant sans jeton : pas l'état", l.get("/api/etat").code, 403);
        verifier("distant sans jeton : pas l'intro", l.get("/websuite-intro.html").code, 403);
        verifier("distant sans jeton : pas d'action", l.req("POST", "/api/arreter", &[X], arreter).code, 403);
        verifier("distant, faux jeton", l.get("/?jeton=faux").entete("Location"), Some("/appairage"));
        verifier("distant, faux biscuit", l.req("GET", "/api/etat", &[("Cookie", "websuite_jeton=faux")], b"").code, 403);
        let r = l.get(&format!("/?jeton={jeton}"));
        verifier("bon jeton -> redirection", r.code, 303);
        verifier("redirection sans jeton", r.entete("Location"), Some("/"));
        let biscuit = biscuit_de(&r);
        verifier("biscuit -> page", l.req("GET", "/", &[("Cookie", &biscuit)], b"").code, 200);
        let r = l.req("POST", "/api/arreter", &[("Cookie", &biscuit), X], arreter);
        verifier("biscuit -> action", (r.code, r.json()["ok"].clone()), (200, json!(true)));
        l.attendre("arrête WEB_3D"); // action reçue, et dite dans le terminal
        verifier("biscuit illisible -> page du code",
                 l.req("GET", "/", &[("Cookie", "\";;==")], b"").entete("Location"), Some("/appairage"));

        // Code d'appairage : le bon donne le cookie du jeton, une seule fois.
        let port = l.port;
        let appairer_x = |code: &str, x: bool| {
            let entetes: &[(&str, &str)] = if x { &[X] } else { &[] };
            http::requete(port, "POST", "/api/appairer", entetes, json!({"code": code}).to_string().as_bytes()).unwrap()
        };
        let appairer = |code: &str| appairer_x(code, true);

        let code = l.codes().last().unwrap().clone();
        verifier("code à 6 chiffres", (code.len(), code.chars().all(|c| c.is_ascii_digit())), (6, true));
        verifier("appairer sans X-WebSuite", appairer_x(&code, false).code, 403);
        let faux = format!("{:06}", (code.parse::<u32>().unwrap() + 1) % 1_000_000);
        let r = appairer(&faux);
        verifier("faux code refusé", (r.code, r.entete("Set-Cookie")), (403, None));
        let r = appairer(&format!("{} {}", &code[..3], &code[3..])); // tel qu'affiché, espace compris
        verifier("bon code accepté", r.code, 200);
        let biscuit = biscuit_de(&r);
        verifier("le cookie est celui du jeton", biscuit.clone(), format!("websuite_jeton={jeton}"));
        verifier("cookie du code -> outils", l.req("GET", "/api/etat", &[("Cookie", &biscuit)], b"").code, 200);
        verifier("code à usage unique", appairer(&code).code, 403);
        // Le cas de l'iPad : code accepté, puis la page renvoie le code (copie en cache,
        // double validation). L'appareil est déjà autorisé : pas d'erreur 404.
        let r = l.req("POST", "/api/appairer", &[X, ("Cookie", &biscuit)], br#"{"code": "999999"}"#);
        verifier("déjà autorisé : le code renvoyé passe", (r.code, r.json()["ok"].clone()), (200, json!(true)));
        let r = l.req("GET", "/appairage", &[("Cookie", &biscuit)], b"");
        verifier("déjà autorisé : la page du code renvoie au lanceur", (r.code, r.entete("Location")), (303, Some("/")));
        let r = l.req("GET", "/", &[("Cookie", &biscuit)], b"");
        verifier("le lanceur jamais en cache", (r.code, r.entete("Cache-Control")), (200, Some("no-store")));
        l.get("/appairage"); // un nouveau code pour la suite
        verifier("nouveau code affiché après usage", l.codes().len() >= 2, true);
        // Cinq erreurs brûlent le code, et l'on attend avant le suivant.
        let code = l.codes().last().unwrap().clone();
        let mut r = None;
        for _ in 0..5 {
            // Appairage::ESSAIS
            r = Some(appairer(if code != "000000" { "000000" } else { "111111" }));
        }
        let message = |r: &Rep| r.json()["message"].as_str().unwrap().to_string();
        verifier("cinq erreurs : code annulé", message(&r.unwrap()).contains("code annulé"), true);
        verifier("pendant la pause, même le bon code échoue", appairer(&code).code, 403);
        verifier("la pause est annoncée", message(&appairer(&code)).contains("attendez"), true);
        verifier("pendant la pause, pas de nouveau code", (l.get("/appairage").code, l.codes().last().unwrap().clone()), (200, code));
        arret_propre(&mut l);
    }

    // HTTPS en réseau : même port, la boucle locale garde HTTP, un autre appareil en
    // HTTP est renvoyé vers https://, le cookie y est Secure.
    let ca = poste.fichier_jeton.parent().unwrap().join("tls").join("ca.pem");
    let avec_https;
    {
        let l = poste.lanceur(&["--reseau"], &[]);
        verifier("ce poste sans jeton", l.get("/").code, 200);
        avec_https = l.sortie().contains("HTTPS : la première fois");
        if avec_https {
            verifier("ce poste en HTTP", l.get("/").code, 200);
            verifier("ce poste en HTTPS", https(l.port, "/", &ca).code, 200);
        } else {
            verifier("pas de HTTPS : le terminal dit pourquoi", l.sortie().contains("HTTP en clair sur le réseau")
                     || l.sortie().contains("adresse IP du poste introuvable"), true);
            println!("  (openssl absent : HTTPS non vérifié)");
        }
    }
    if avec_https {
        let l = poste.lanceur(&["--reseau"], &[("WEB_TOOLS_BANC_DISTANT", "1")]);
        let r = l.get("/api/etat");
        verifier("distant en HTTP : vers https", (r.code, r.entete("Location").map(str::to_string)),
                 (307, Some(format!("https://127.0.0.1:{}/api/etat", l.port))));
        verifier("la CA, sans jeton", l.get("/appairage/ca.pem").corps.starts_with(b"-----BEGIN"), true);
        let r = https(l.port, &format!("/?jeton={}", poste.jeton()), &ca);
        verifier("cookie Secure", (r.code, r.entete("Set-Cookie").unwrap_or("").contains("; Secure")), (303, true));
        verifier("page du code en HTTPS", https(l.port, "/appairage", &ca).texte().contains("id=\"ca\""), true);
    }

    // Mise à jour de WEB_SUITE au démarrage : avec un script relanceur (WEB_SUITE_RELANCEUR=1)
    // le lanceur sort en 75 ; sinon il se relance lui-même et le bilan suit.
    let git = |cwd: &Path, args: &[&str]| {
        let etat = poste.commande("git").args(["-c", "user.name=banc", "-c", "user.email=banc@example.com"]).args(args)
            .current_dir(cwd).stdout(Stdio::null()).stderr(Stdio::null()).status().unwrap();
        assert!(etat.success(), "git {args:?}");
    };
    let (origine, suite) = (poste.tmp.join("suite-origine"), poste.tmp.join("suite"));
    let publier = |n: u32| {
        // un commit de plus sur le « GitHub » de WEB_SUITE
        fs::write(origine.join("lanceur").join("index.html"), format!("version {n}")).unwrap();
        git(&origine, &["add", "-A"]);
        git(&origine, &["commit", "-m", &format!("version {n}")]);
    };
    fs::create_dir_all(origine.join("lanceur")).unwrap();
    git(&origine, &["init"]);
    publier(1);
    git(&poste.tmp, &["clone", &chaine(&origine), &chaine(&suite)]);
    publier(2);
    let (code, sortie) = poste.lancer(
        WEB_SUITE, &["--port", "0", "--racine", &chaine(&poste.racine), "--sans-navigateur", "--local"],
        &[("WEB_SUITE_DEPOT", &chaine(&suite)), ("WEB_SUITE_RELANCEUR", "1")]);
    verifier("mise à jour avec relanceur : code 75",
             (code, sortie.contains("[*] WEB_SUITE mis à jour, redémarrage.")), (75, true));
    #[cfg(unix)] // sous Windows le relancé est un enfant, que l'arrêt du lanceur laisserait
    {
        publier(3);
        let mut l = poste.lanceur_maj(&["--local"], &[("WEB_SUITE_DEPOT", &chaine(&suite))]);
        verifier("mise à jour sans relanceur : relancé, bilan transmis",
                 l.sortie().contains("WEB_SUITE    Mis à jour (1 commit(s))."), true);
        verifier("le relancé sert la nouvelle page", l.get("/").corps, b"version 3".to_vec());
        arret_propre(&mut l);
    }
    println!("OK : {} vérifications", verifications());
}
