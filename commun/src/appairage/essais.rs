//! Vérification : un vrai serveur sur la boucle locale ; « l'autre appareil » est
//! simulé en faisant croire au garde que la requête ne vient pas de ce poste.
use super::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};

pub static DISTANT: AtomicBool = AtomicBool::new(false);

struct Rep {
    code: u16,
    entetes: Vec<(String, String)>,
    corps: Vec<u8>,
}

impl Rep {
    fn entete(&self, nom: &str) -> Option<&str> {
        self.entetes.iter().find(|(k, _)| k.eq_ignore_ascii_case(nom)).map(|(_, v)| v.as_str())
    }
    fn json(&self) -> Value {
        serde_json::from_slice(&self.corps).unwrap()
    }
}

fn lire_reponse(brut: &[u8]) -> Rep {
    let coupe = brut.windows(4).position(|w| w == b"\r\n\r\n").expect("réponse HTTP");
    let tete = String::from_utf8_lossy(&brut[..coupe]).into_owned();
    let mut lignes = tete.split("\r\n");
    let code = lignes.next().unwrap().split(' ').nth(1).unwrap().parse().unwrap();
    let entetes = lignes.filter_map(|l| l.split_once(": ")).map(|(k, v)| (k.to_string(), v.to_string())).collect();
    Rep { code, entetes, corps: brut[coupe + 4..].to_vec() }
}

fn demande(methode: &str, chemin: &str, entetes: &[(&str, &str)], corps: &str) -> Vec<u8> {
    let mut d = format!("{} {} HTTP/1.1\r\nContent-Length: {}\r\n", methode, chemin, corps.len());
    if !entetes.iter().any(|(k, _)| *k == "Host") {
        d += "Host: 127.0.0.1\r\n";
    }
    for (k, v) in entetes {
        d += &format!("{}: {}\r\n", k, v);
    }
    (d + "\r\n" + corps).into_bytes()
}

fn req(port: u16, methode: &str, chemin: &str, entetes: &[(&str, &str)], corps: &str) -> Rep {
    let mut c = TcpStream::connect(("127.0.0.1", port)).unwrap();
    c.write_all(&demande(methode, chemin, entetes, corps)).unwrap();
    let mut brut = Vec::new();
    c.read_to_end(&mut brut).unwrap();
    lire_reponse(&brut)
}

fn reqs(port: u16, chemin: &str, ca: &Path) -> Rep {
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
    let mut c = rustls::StreamOwned::new(conn, TcpStream::connect(("127.0.0.1", port)).unwrap());
    c.write_all(&demande("GET", chemin, &[], "")).unwrap();
    let mut brut = Vec::new();
    let _ = c.read_to_end(&mut brut); // certificat vérifié par la CA à la poignée de main
    lire_reponse(&brut)
}

fn servir(garde: Arc<Garde>) -> (Arc<Serveur>, u16) {
    let serveur = Serveur::ouvrir(0, false, 1).unwrap();
    let g = garde.clone();
    serveur.servir_en_fond(Arc::new(move |r: &mut Requete| g.filtrer(r).unwrap_or_else(|| Reponse::texte(200, "ok"))));
    let port = serveur.port();
    (serveur, port)
}

#[test]
fn banc() {
    let tmp = std::env::temp_dir().join(format!("appairage-essai-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let fichier = tmp.join("partage").join("jeton.txt");
    std::env::set_var("WEB_TOOLS_JETON_FICHIER", &fichier);
    let ancien = tmp.join("ancien.txt");
    fs::write(&ancien, "a".repeat(22) + "\n").unwrap();
    assert_eq!(jeton(Some(&ancien)), "a".repeat(22)); // l'ancien jeton est repris
    assert_eq!(jeton(None), "a".repeat(22)); // puis partagé
    fs::remove_file(&fichier).unwrap();
    let j = jeton(None);
    assert!(j.len() >= 16 && j != "a".repeat(22) && jeton(None) == j);
    fs::write(&fichier, "court").unwrap(); // fichier abîmé : remplacé et gardé
    let j = jeton(None);
    assert!(j.len() >= 16 && jeton(None) == j && lire(&fichier).as_deref() == Some(&j));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(&fichier).unwrap().permissions().mode() & 0o777, 0o600);
    }
    std::env::set_var("WEB_TOOLS_JETON_FICHIER", ancien.join("impossible").join("jeton.txt"));
    assert!(jeton(None).len() >= 16); // pas de plantage sans dossier inscriptible
    std::env::set_var("WEB_TOOLS_JETON_FICHIER", &fichier);

    let affiches: Arc<Mutex<Vec<String>>> = Arc::default();
    let a2 = affiches.clone();
    let afficher: Afficher = Arc::new(move |t| a2.lock().unwrap().push(t.to_string()));
    let dernier = || affiches.lock().unwrap().last().cloned().unwrap_or_default();
    let g = Arc::new(Garde::new("TEST", true, Some(afficher), None));
    DISTANT.store(true, Ordering::SeqCst);
    let (srv, port) = servir(g.clone());
    let code_courant = || g.code.as_ref().unwrap().etat.lock().unwrap().code.clone().unwrap();
    let appairer = |code: &str, x: bool, biscuit: Option<&str>| {
        let mut e = vec![("Content-Type", "application/json")];
        if x {
            e.push((ENTETE, "1"));
        }
        if let Some(b) = biscuit {
            e.push(("Cookie", b));
        }
        req(port, "POST", ROUTE, &e, &json!({ "code": code }).to_string())
    };
    let get = |chemin: &str, entetes: &[(&str, &str)]| req(port, "GET", chemin, entetes, "");

    let r = get("/", &[]);
    assert_eq!((r.code, r.entete("Location")), (303, Some(PAGE)));
    assert_eq!(get("/app.js", &[]).code, 403);
    assert_eq!(get("/app.js", &[("Accept", "text/html")]).entete("Location"), Some(PAGE));
    assert_eq!(req(port, "POST", "/api/x", &[], "").code, 403);
    assert_eq!(get("/", &[("Host", "piege.example:80")]).code, 403);
    let ts = || "pi.tail1.ts.net".to_string();
    assert!(hote_permis_avec(Some("PI.tail1.ts.net:8000"), &ts) && !hote_permis_avec(Some("autre.tail1.ts.net:8000"), &ts));
    assert!(!hote_permis_avec(Some("pi.tail1.ts.net"), &String::new)); // sans Tailscale : aucun nom en .ts.net
    assert!(hote_permis(Some("[::1]:8000")) && hote_permis(Some("192.168.1.2:80")) && !hote_permis(Some("[")));
    let r = get(PAGE, &[]);
    let page = String::from_utf8_lossy(&r.corps).into_owned();
    assert!(r.code == 200 && page.contains("/api/appairer") && page.contains("TEST"));
    assert!(page.contains("opaqueredirect")); // déjà autorisé : la page repart seule
    let code = code_courant();
    assert!(dernier().contains(&Appairage::lisible(&code)));
    assert_eq!(appairer(&code, false, None).code, 403); // sans en-tête : refusé
    assert_eq!(code_courant(), code); // … et sans brûler d'essai
    let faux = if code != "000000" { "000000" } else { "111111" };
    assert_eq!(appairer(faux, true, None).code, 403);
    let r = appairer(&Appairage::lisible(&code), true, None); // tel qu'affiché
    assert!(r.code == 200 && r.json()["ok"] == true);
    let biscuit = r.entete("Set-Cookie").unwrap().split(';').next().unwrap().to_string();
    assert_eq!(biscuit, format!("{}={}", BISCUIT, j));
    assert_eq!(appairer(&code, true, None).code, 403); // usage unique
    assert!(!affiches.lock().unwrap().iter().any(|a| a.contains(MARQUE))); // non appairé : pas un utilisateur
    let b = [("Cookie", biscuit.as_str())];
    assert_eq!(get("/", &b).corps, b"ok");
    assert!(dernier().contains(MARQUE) && dernier().contains("127.0.0.1"));
    assert_eq!(get("/app.js", &b).corps, b"ok");
    assert_eq!(req(port, "POST", "/api/x", &b, "").corps, b"ok");
    assert_eq!(get(PAGE, &b).entete("Location"), Some("/"));
    assert!(appairer("x", true, Some(&biscuit)).json()["ok"] == true);
    assert_eq!(get("/", &[("Cookie", "websuite_jeton=faux")]).code, 303);
    assert_eq!(get("/", &[("Cookie", "\";;==")]).code, 303);
    let r = get(&format!("/outil?a=1&jeton={}", j), &[]); // adresse avec ?jeton=
    assert_eq!((r.code, r.entete("Location")), (303, Some("/outil?a=1")));
    assert!(r.entete("Set-Cookie").unwrap().starts_with(&format!("{}={}", BISCUIT, j)));
    assert_eq!(get("/?jeton=faux", &[]).entete("Location"), Some(PAGE));
    // cinq erreurs : code brûlé, pause
    get(PAGE, &[]);
    let code = code_courant();
    for _ in 0..Appairage::ESSAIS {
        appairer(if code != "000000" { "000000" } else { "111111" }, true, None);
    }
    assert_eq!(appairer(&code, true, None).code, 403);
    assert!(appairer(&code, true, None).json()["message"].as_str().unwrap().contains("attendez"));
    g.code.as_ref().unwrap().etat.lock().unwrap().bloque = 0.0;
    get(PAGE, &[]);
    assert_eq!(appairer(&code_courant(), true, None).code, 200);
    assert_eq!(masquer(&format!("\"GET /?a=1&jeton={} HTTP/1.1\" 303", j)), "\"GET /?a=1&jeton=*** HTTP/1.1\" 303");
    // l'expiration d'un code ne remet pas les erreurs à zéro
    let a = Appairage::new(Some(Arc::new(|_| ())), "");
    a.preparer();
    for _ in 0..Appairage::ESSAIS - 1 {
        a.essayer("x");
    }
    a.etat.lock().unwrap().expire = 0.0;
    a.essayer("x"); // code expiré : un nouveau
    assert!(a.essayer("x").1.contains("annulé")); // 5e erreur : brûlé
    // adresses publiques refusées, réseau local et Tailscale acceptés
    let ip = |t: &str| t.split('%').next().unwrap().parse::<IpAddr>().unwrap();
    let ici4 = ip("192.168.1.2");
    for t in ["192.168.1.20", "10.0.0.2", "100.80.111.121", "fd7a:115c:a1e0::1", "fe80::1%eth0", "127.0.0.1", "::ffff:192.168.1.20"] {
        assert!(!internet(ip(t), ici4), "{}", t);
    }
    for t in ["8.8.8.8", "2a01:cb19:690:a100::1", "::ffff:8.8.8.8"] {
        assert!(internet(ip(t), ici4), "{}", t);
    }
    // IPv6 globale du /64 du poste (mDNS pilou.local) : réseau local ; ailleurs, Internet
    assert!(!internet(ip("2a01:cb19:690:a100::1"), ip("2a01:cb19:690:a100:abcd::2")));
    for (c, i) in [
        ("2a01:cb19:690:a101::1", "2a01:cb19:690:a100::2"), // autre /64
        ("2a01:cb19:690:a100::1", "fd00::2"),               // arrivée par une ULA
        ("2a01:cb19:690:a100::1", "::ffff:192.168.1.2"),
        ("8.8.8.8", "8.8.8.1"), // IPv4 globale : toujours refusée
        ("::ffff:8.8.8.8", "::ffff:8.8.8.1"),
    ] {
        assert!(internet(ip(c), ip(i)), "{} {}", c, i);
    }
    DISTANT.store(false, Ordering::SeqCst); // ce poste : pas de code
    assert_eq!(get("/", &[]).corps, b"ok");
    assert!(Garde::new("LOCAL", false, None, None).jeton.is_none());
    let lignes: Arc<Mutex<Vec<String>>> = Arc::default();
    let l2 = lignes.clone();
    let v = Visites::new("TEST", Some(Arc::new(move |t| l2.lock().unwrap().push(t.to_string()))));
    v.noter(ip("192.168.1.20"));
    v.noter(ip("192.168.1.20")); // déjà vu : rien
    v.noter(ip("::ffff:192.168.1.21"));
    assert!(lignes.lock().unwrap().len() == 2 && lignes.lock().unwrap()[1].contains("2 appareil(s) actif(s) : 192.168.1.20, 192.168.1.21"));
    *v.vues.lock().unwrap().get_mut("192.168.1.20").unwrap() -= Visites::INACTIF + 1.0; // parti depuis 10 min
    v.noter(ip("192.168.1.21"));
    assert_eq!(lignes.lock().unwrap().len(), 2);
    v.noter(ip("192.168.1.20")); // revenu : affiché de nouveau
    assert!(lignes.lock().unwrap().len() == 3 && lignes.lock().unwrap()[2].contains("2 appareil(s)"));
    srv.arreter();

    // HTTPS : la CA et le certificat créés une fois, gardés ensuite
    std::env::set_var("WEB_TOOLS_HTTP", "1");
    assert_eq!(contexte_tls().err().as_deref(), Some("WEB_TOOLS_HTTP=1"));
    std::env::remove_var("WEB_TOOLS_HTTP");
    let Some(ssl) = openssl() else {
        println!("(openssl absent : HTTPS non vérifié)");
        return;
    };
    contexte_tls().unwrap();
    let d = dossier_tls();
    let date_ca = fs::metadata(d.join("ca.pem")).unwrap().modified().unwrap();
    assert!(!a_refaire(&d, &adresses_poste()) && contexte_tls().is_ok());
    assert_eq!(fs::metadata(d.join("ca.pem")).unwrap().modified().unwrap(), date_ca);
    assert_eq!(empreinte_ca().unwrap().split(' ').count(), 32);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(d.join("ca.key")).unwrap().permissions().mode() & 0o077, 0);
    }
    // adresse nouvelle : le certificat est refait, la CA reste
    let mut info = lire_json(&d.join("serveur.json")).unwrap();
    info["adresses"] = json!(["127.0.0.1"]);
    fs::write(d.join("serveur.json"), info.to_string()).unwrap();
    let mut plus = adresses_poste();
    plus.push("10.9.9.9".into());
    assert!(a_refaire(&d, &plus));
    let g2 = Arc::new(Garde::new("TLS", true, Some(Arc::new(|_| ())), None));
    DISTANT.store(true, Ordering::SeqCst);
    let srv = Serveur::ouvrir(0, false, 1).unwrap();
    assert!(g2.brancher(&srv) == "https" && g2.adresse("h", 1) == "https://h:1/" && g2.adresse("fd7a::1", 1) == "https://[fd7a::1]:1/");
    assert_eq!(fs::metadata(d.join("ca.pem")).unwrap().modified().unwrap(), date_ca);
    let g3 = g2.clone();
    srv.servir_en_fond(Arc::new(move |r: &mut Requete| g3.filtrer(r).unwrap_or_else(|| Reponse::texte(200, "ok"))));
    let port = srv.port();
    let r = req(port, "GET", "/outil?a=1", &[("Host", &format!("127.0.0.1:{}", port))], ""); // autre appareil en HTTP
    assert_eq!((r.code, r.entete("Location").unwrap().to_string()), (307, format!("https://127.0.0.1:{}/outil?a=1", port)));
    let r = req(port, "GET", CA, &[], ""); // la CA, même en HTTP, sans jeton
    assert!(r.code == 200 && r.corps.starts_with(b"-----BEGIN CERTIFICATE"));
    let r = reqs(port, "/", &d.join("ca.pem")); // certificat vérifié par la CA
    assert_eq!((r.code, r.entete("Location")), (303, Some(PAGE)));
    assert!(String::from_utf8_lossy(&reqs(port, PAGE, &d.join("ca.pem")).corps).contains("id=\"ca\""));
    let r = reqs(port, &format!("/?jeton={}", g2.jeton.as_ref().unwrap()), &d.join("ca.pem"));
    assert!(r.entete("Set-Cookie").unwrap().ends_with("; Secure")); // en HTTPS, le cookie ne repart jamais en clair
    DISTANT.store(false, Ordering::SeqCst);
    assert_eq!(req(port, "GET", "/", &[], "").corps, b"ok"); // la boucle locale garde HTTP
    srv.arreter();

    // la CA ne signe que des adresses privées
    let signe = |adresses: &[&str]| {
        let adresses: Vec<String> = adresses.iter().map(|a| a.to_string()).collect();
        generer(&d, &ssl, &adresses, false).unwrap();
        Command::new(&ssl).args(["verify", "-CAfile", "ca.pem", "serveur.pem"]).current_dir(&d).output().unwrap().status.success()
    };
    let texte = Command::new(&ssl).args(["x509", "-in", "ca.pem", "-noout", "-text"]).current_dir(&d).output().unwrap();
    assert!(String::from_utf8_lossy(&texte.stdout).to_uppercase().contains("IP:FC00:0:0:0:0:0:0:0/FE00:0:0:0:0:0:0:0"));
    assert!(signe(&["127.0.0.1", "fd7a:115c:a1e0::1", "fd12:3456::1"]));
    assert!(!signe(&["2a01:cb19:690:a100::1"]) && !signe(&["fe80::1"]));
    // une CA d'avant IPv6 (sans ca.json) est refaite, une fois
    fs::remove_file(d.join("ca.json")).unwrap();
    let vieille = empreinte_ca().unwrap();
    assert!(a_refaire(&d, &["127.0.0.1".to_string()]) && contexte_tls().is_ok());
    assert!(empreinte_ca().unwrap() != vieille && !a_refaire(&d, &adresses_poste()));
    let _ = fs::remove_dir_all(&tmp);
}
