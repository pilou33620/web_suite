//! Banc de web_bench lui-même, sans les vrais outils.
//!
//!     cargo test -p web_bench --test banc_web_bench
//!
//! L'exécutable web_bench est piloté de l'extérieur : un sous-processus, et des requêtes HTTP.
//! Un faux WEB_CAO et un faux WEB_3D (de petits serveurs qui acceptent les options de web_bench)
//! sont créés dans une fausse racine de web_suite ; le faux WEB_CAO a des bancs node, dont un qui
//! échoue, et une petite caisse Rust (ses bancs cargo, et l'outil de préparation). On vérifie le
//! relais (la page reçoit la sonde, un POST passe avec son corps et ses en-têtes), le passage d'un
//! outil à l'autre, les gardes (Host étranger, POST de l'API sans en-tête), la découverte et le
//! lancement des bancs, l'enregistrement d'un rapport, et l'arrêt (outils arrêtés, dossiers
//! temporaires effacés).
//!
//! web_bench lance un outil par son demarrer_<NOM> : celui des faux lance un petit serveur node,
//! écrit à la volée dans le dossier temporaire. Sans node, seules les options refusées sont
//! vérifiées et le reste est sauté.
#[path = "../../lanceur/tests/commun/http.rs"]
mod http;

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use http::Rep;
use serde_json::{json, Value};

const EXE: &str = env!("CARGO_BIN_EXE_web_bench");

const FAUX_SERVEUR: &str = r#"
const http = require("http"), path = require("path");
const arg = (nom) => { const i = process.argv.indexOf(nom); return i < 0 ? null : process.argv[i + 1]; };
const NOM = path.basename(__dirname);
http.createServer((q, r) => {
  console.log("requete", q.url);
  const rendre = (code, corps, type) => {
    r.writeHead(code, { "Content-Type": type, "Content-Length": Buffer.byteLength(corps) });
    r.end(corps);
  };
  if (q.method === "POST") {
    const morceaux = [];
    q.on("data", (m) => morceaux.push(m));
    q.on("end", () => rendre(200, JSON.stringify({ corps: Buffer.concat(morceaux).toString(), host: q.headers.host,
      entete: q.headers["x-web_cao"], lib: arg("--lib"), projets: arg("--projets") }), "application/json"));
  } else if (q.url.endsWith(".html") || q.url === "/") {
    rendre(200, "<!DOCTYPE html><html><head><title>t</title></head><body>" + NOM + "</body></html>", "text/html");
  } else rendre(404, "non", "text/plain");
}).listen(Number(arg("--port")), "127.0.0.1");
process.on("SIGTERM", () => process.exit(0));
"#;

/// Un faux outil : son demarrer_<NOM>, comme les vrais, et le serveur node qu'il lance.
fn faux_outil(dossier: &Path, nom: &str) {
    ecrire(&dossier.join("serveur.js"), FAUX_SERVEUR);
    if cfg!(windows) {
        ecrire(&dossier.join(format!("demarrer_{nom}.cmd")), "@node \"%~dp0serveur.js\" %*\r\n");
    } else {
        ecrire(&dossier.join(format!("demarrer_{nom}.sh")), "node \"$(dirname \"$0\")/serveur.js\" \"$@\"\n");
    }
}

fn ecrire(chemin: &Path, texte: &str) {
    fs::create_dir_all(chemin.parent().unwrap()).unwrap();
    fs::write(chemin, texte).unwrap();
}

fn copier_arbre(source: &Path, cible: &Path) {
    fs::create_dir_all(cible).unwrap();
    for e in fs::read_dir(source).unwrap().flatten() {
        if e.path().is_dir() {
            copier_arbre(&e.path(), &cible.join(e.file_name()));
        } else {
            fs::copy(e.path(), cible.join(e.file_name())).unwrap();
        }
    }
}

/// La commande répond-elle (node) ?
fn present(commande: &str) -> bool {
    Command::new(commande).arg("--version").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .status().is_ok_and(|s| s.success())
}

/// L'arrêt demandé de l'extérieur : SIGTERM, Ctrl+Pause sous Windows.
#[cfg(unix)]
fn demander_arret(proc: &Child) {
    unsafe { libc::kill(proc.id() as libc::pid_t, libc::SIGTERM) };
}

#[cfg(windows)]
fn demander_arret(proc: &Child) {
    extern "system" {
        fn GenerateConsoleCtrlEvent(evenement: u32, groupe: u32) -> i32;
    }
    unsafe { GenerateConsoleCtrlEvent(1, proc.id()) }; // CTRL_BREAK_EVENT, au groupe du processus
}

/// Le dossier temporaire et web_bench : arrêtés et effacés quoi qu'il arrive.
struct Menage {
    temp: PathBuf,
    proc: Option<Child>,
}

impl Drop for Menage {
    fn drop(&mut self) {
        if let Some(p) = &mut self.proc {
            let _ = p.kill();
            let _ = p.wait();
        }
        let _ = fs::remove_dir_all(&self.temp);
    }
}

#[test]
fn banc() {
    let temp = std::env::temp_dir().join(format!("banc_web_bench_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    let mut menage = Menage { temp: temp.clone(), proc: None };
    // une fausse racine de web_suite : les pages du banc, et les rapports, y vivent aussi
    copier_arbre(&Path::new(env!("CARGO_MANIFEST_DIR")).join("page"), &temp.join("web_bench").join("page"));
    let cao = temp.join("WEB_CAO");
    faux_outil(&cao, "WEB_CAO");
    faux_outil(&temp.join("WEB_3D"), "WEB_3D");
    ecrire(&temp.join("WEB_3D").join("conception.html"), "<html></html>");
    // la vraie LIB de WEB_CAO, que le banc copie sans y toucher
    ecrire(&temp.join("PROJETS").join("LIB_CAO").join("LIB_composants.csv"), "ref;valeur\n");
    // le catalogue de web_bench : un essai node sous un autre nom, sa préparation par un outil
    // Rust de la caisse « outils », et un banc qui exige la LIB privée (vue par WEB_CAO_LIB)
    let (harness, banc_lib) = ("editeur-pcb/test/harness.js", "gestion-lib/test/banc-catalogue.js");
    let (rust_unitaires, rust_banc) = ("cargo test -p outils --bins", "cargo test -p outils --test banc_bon");
    ecrire(&cao.join(harness), "console.log('harness ok')\n");
    ecrire(&cao.join("Cargo.toml"),
           "[workspace]\n[package]\nname = \"outils\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
            [[bin]]\nname = \"build-monofichier\"\npath = \"src/main.rs\"\n");
    ecrire(&cao.join("src").join("main.rs"),
           "fn main() {\n    std::fs::write(\"prepare.txt\", \"fait\").unwrap();\n    \
            println!(\"préparé {}\", std::env::args().nth(1).unwrap_or_default());\n}\n");
    ecrire(&cao.join("tests").join("banc_bon.rs"), "#[test]\nfn bon() {\n    println!(\"banc rust\");\n}\n");
    ecrire(&cao.join(banc_lib),
           "console.log(process.env.WEB_CAO_LIB)\n\
            process.exit(require('fs').existsSync(process.env.WEB_CAO_LIB + '/LIB_composants.csv') ? 0 : 4)\n");
    ecrire(&cao.join("index.html"), "<html></html>");
    ecrire(&cao.join("test").join("banc-bon.js"), "console.log('OK')\n");
    ecrire(&cao.join("test").join("banc-mauvais.js"), "console.log('raté')\nprocess.exit(3)\n");
    ecrire(&cao.join("graphify-out").join("banc-ignore.js"), "process.exit(1)\n");

    // options refusées et aide, sans rien démarrer
    let contient = |octets: &[u8], motif: &str| String::from_utf8_lossy(octets).contains(motif);
    let r = Command::new(EXE).args(["--outil", "kicad"]).output().unwrap();
    assert!(r.status.code() == Some(2) && contient(&r.stderr, "--outil"), "{r:?}");
    let r = Command::new(EXE).arg("--help").output().unwrap();
    assert!(r.status.code() == Some(0) && contient(&r.stdout, "--racine") && contient(&r.stdout, "--sans-navigateur"), "{r:?}");
    let vide = temp.join("vide");
    fs::create_dir_all(&vide).unwrap();
    let r = Command::new(EXE).arg("--racine").arg(&vide).output().unwrap();
    assert!(r.status.code() == Some(2) && contient(&r.stdout, "Aucun outil trouvé"), "{r:?}");

    if !present("node") {
        println!("SAUTÉ : pas de node sur ce poste (les faux outils sont des serveurs node)");
        return;
    }

    let mut commande = Command::new(EXE);
    commande.arg("--racine").arg(&temp).args(["--outil", "cao", "--port", "0", "--sans-navigateur"])
        .env_remove("WEB_CAO_LIB").env("WEB_SUITE_DEPOT", &temp).stdin(Stdio::null()).stdout(Stdio::piped());
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut commande, 0x200); // CREATE_NEW_PROCESS_GROUP
    menage.proc = Some(commande.spawn().unwrap());
    let mut stdout = BufReader::new(menage.proc.as_mut().unwrap().stdout.take().unwrap());
    let mut debut = String::new();
    while !debut.contains("Page Banc") {
        assert!(stdout.read_line(&mut debut).unwrap() > 0, "web_bench s'est arrêté au démarrage :\n{debut}");
    }
    assert!(debut.contains("[*] Outils trouvés : WEB_CAO, WEB_3D"), "{debut}");
    assert!(debut.contains("[*] Démarrage de WEB_CAO (dossiers temporaires)..."), "{debut}");
    assert!(debut.contains("[*] Bancs trouvés : WEB_CAO 6, WEB_3D 0"), "{debut}");
    let (avant, apres) = ("Page Banc : http://127.0.0.1:", "/__banc/   (Ctrl+C pour arrêter)");
    let port: u16 = debut.split_once(avant).and_then(|(_, suite)| suite.split_once(apres)).and_then(|(p, _)| p.parse().ok())
        .unwrap_or_else(|| panic!("adresse de la page Banc attendue dans :\n{debut}"));
    let hote_port = format!("127.0.0.1:{port}");
    let hote = [("Host", hote_port.as_str())];
    let maison = [hote[0], ("X-Web-Bench", "1")];

    let requete = |methode: &str, chemin: &str, corps: &[u8], entetes: &[(&str, &str)]| -> Rep {
        http::requete(port, methode, chemin, entetes, corps).unwrap()
    };
    let lire = |chemin: &str| requete("GET", chemin, b"", &hote);
    let journal = || -> Vec<String> {
        lire("/__banc/api/journal?depuis=0").json()["lignes"].as_array().unwrap().iter().map(|l| l.as_str().unwrap().to_string()).collect()
    };
    let attendre_bancs = || -> Value {
        // les bancs Rust du faux outil se compilent d'abord
        for _ in 0..3000 {
            let e = lire("/__banc/api/bancs").json();
            if e["en_cours"] == false {
                return e;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("les bancs ne finissent pas");
    };

    // la page relayée porte la sonde en tête
    let r = lire("/index.html");
    assert!(r.code == 200 && r.texte().contains(r#"<head><script src="/__banc/sonde.js"></script><title>"#), "{}", r.texte());
    assert!(r.texte().contains("WEB_CAO"), "{}", r.texte());
    // ce qui n'est pas une page passe tel quel, avec son code
    let r = lire("/absent.txt");
    assert!((r.code, r.corps.as_slice()) == (404, b"non"), "{} {}", r.code, r.texte());

    // un POST passe avec son corps, son Host et l'en-tête maison
    let r = requete("POST", "/api/projet", br#"{"a":1}"#, &[hote[0], ("X-WEB_CAO", "1"), ("Content-Type", "application/json")]);
    let rep = r.json();
    assert!(r.code == 200 && rep["corps"] == r#"{"a":1}"# && rep["entete"] == "1", "{rep}");
    assert!(rep["host"] == hote_port.as_str(), "{rep}");
    // l'outil tourne sur des dossiers temporaires, pas sur ceux de l'utilisateur
    let lib = PathBuf::from(rep["lib"].as_str().unwrap());
    let temp_outil = lib.parent().unwrap().to_path_buf();
    assert!(temp_outil.file_name().unwrap().to_string_lossy().starts_with("web_bench_cao_")
            && !temp_outil.to_string_lossy().contains(&*temp.to_string_lossy()), "{rep}");
    assert!(rep["projets"] == temp_outil.join("projets").to_string_lossy().as_ref(), "{rep}");
    assert!(lib.join("LIB_composants.csv").is_file(), "LIB non copiée");

    // routes protégées : arrêtées par le relais, l'outil ne les reçoit jamais
    let deja = journal().len();
    let r = requete("POST", "/api/github/envoyer", b"{}", &[hote[0], ("X-WEB_CAO", "1")]);
    assert!(r.code == 403 && r.texte().contains("web_bench"), "{} {}", r.code, r.texte());
    assert!(r.json() == json!({"detail": "Arrêté par web_bench (route protégée pendant les essais)"}), "{}", r.texte());
    std::thread::sleep(Duration::from_millis(300));
    let lignes = journal();
    assert!(!lignes[deja..].iter().any(|l| l.starts_with("requete") && l.contains("github")), "{lignes:?}");
    assert!(lignes.iter().any(|l| l == "[web_bench] requête arrêtée par le banc : POST /api/github/envoyer"), "{lignes:?}");

    // gardes
    let piege = format!("piege.exemple:{port}");
    assert_eq!(requete("GET", "/index.html", b"", &[("Host", &piege)]).code, 403);
    let r = requete("POST", "/__banc/api/bancs/lancer", b"{}", &hote);
    assert!(r.code == 403 && r.json() == json!({"erreur": "en-tête X-Web-Bench manquant"}), "{} {}", r.code, r.texte());
    assert_eq!(lire("/__banc/../../WEB_CAO/serveur.js").code, 404);

    // page Banc et sonde servies
    for f in ["/__banc/", "/__banc/banc.js", "/__banc/sonde.js", "/__banc/banc.css"] {
        assert_eq!(lire(f).code, 200, "{f}");
    }

    // bancs : graphify-out et target ignorés, l'essai harness.js et les bancs Rust ajoutés, tous lancés
    let ids: Vec<Value> = lire("/__banc/api/bancs").json()["bancs"].as_array().unwrap().iter().map(|b| b["id"].clone()).collect();
    assert_eq!(ids, [rust_unitaires, rust_banc, harness, banc_lib, "test/banc-bon.js", "test/banc-mauvais.js"].map(|i| json!(i)));
    let r = requete("POST", "/__banc/api/bancs/lancer", json!({"ids": ids}).to_string().as_bytes(), &maison);
    assert!(r.code == 200 && r.json() == json!({"ok": true}), "{} {}", r.code, r.texte());
    let e = attendre_bancs();
    let etats = |e: &Value, champs: &dyn Fn(&Value) -> Value| -> Value {
        Value::Object(e["bancs"].as_array().unwrap().iter().map(|b| (b["id"].as_str().unwrap().to_string(), champs(b))).collect())
    };
    let sortie = |id: &str| lire(&format!("/__banc/api/bancs/sortie?id={}", id.replace(' ', "%20"))).json()["sortie"].as_str().unwrap().to_string();
    let attendu = json!({rust_unitaires: ["ok", 0], rust_banc: ["ok", 0], harness: ["ok", 0], banc_lib: ["ok", 0],
                         "test/banc-bon.js": ["ok", 0], "test/banc-mauvais.js": ["echec", 3]});
    assert_eq!(etats(&e, &|b| json!([b["etat"], b["code"]])), attendu, "{}", sortie(rust_banc));
    assert!(sortie(rust_banc).contains("test bon ... ok"), "{}", sortie(rust_banc));
    let s = sortie(harness);
    assert!(s.contains("préparé editeur-pcb") && s.contains("(préparation)") && s.contains("harness ok"), "{s}");
    assert!(cao.join("prepare.txt").is_file(), "préparation non lancée");
    // sans la LIB privée : le banc qui l'exige est sauté, comme dans la CI
    fs::remove_dir_all(temp.join("PROJETS")).unwrap();
    requete("POST", "/__banc/api/bancs/lancer", json!({"ids": [banc_lib]}).to_string().as_bytes(), &maison);
    let e = attendre_bancs();
    assert!(etats(&e, &|b| b["etat"].clone())[banc_lib] == "ignore", "{e}");
    assert!(sortie("test/banc-mauvais.js").contains("raté"));

    // journal de l'outil
    assert!(journal().iter().any(|l| l.contains("requete")));

    // rapport enregistré puis relu
    let rapport = json!({"date": "2026-11-09T10:00:00Z", "outil": "WEB_CAO", "bilan": {"essais": 1}, "resultats": [],
                         "markdown": "# Rapport\n"});
    let r = requete("POST", "/__banc/api/rapports", rapport.to_string().as_bytes(), &maison);
    let nom = r.json()["nom"].as_str().unwrap().to_string();
    // \d{4}-\d\d-\d\d_\d{6}_WEB_CAO\.json
    let gabarit = "dddd-dd-dd_dddddd_WEB_CAO.json";
    assert!(nom.len() == gabarit.len() && nom.chars().zip(gabarit.chars()).all(|(c, g)| if g == 'd' { c.is_ascii_digit() } else { c == g }), "{nom}");
    let liste = lire("/__banc/api/rapports").json()["rapports"].clone();
    assert_eq!(liste.as_array().unwrap().iter().map(|x| json!([x["nom"], x["md"], x["outil"]])).collect::<Vec<_>>(),
               [json!([nom, true, "WEB_CAO"])], "{liste}");
    let relu = lire(&format!("/__banc/api/rapport?nom={nom}")).json();
    assert!(relu["bilan"] == json!({"essais": 1}) && relu.get("markdown").is_none(), "{relu}");
    let r = lire(&format!("/__banc/api/rapport.md?nom={nom}"));
    assert_eq!((r.code, r.texte().as_str()), (200, "# Rapport\n"));
    assert!(temp.join("web_bench").join("rapports").join(&nom).is_file(), "rapport hors du dépôt");

    // passer à WEB_3D : il démarre à la demande, le relais le suit, ses pages sont listées
    assert_eq!(requete("POST", "/__banc/api/outil/choisir", br#"{"id": "3d"}"#, &maison).code, 200);
    let r = lire("/index.html");
    assert!(r.texte().contains("WEB_3D"), "{}", r.texte());
    let pages = lire("/__banc/api/pages").json()["pages"].clone();
    assert_eq!(pages.as_array().unwrap().iter().map(|p| p["url"].clone()).collect::<Vec<_>>(), [json!("/conception.html")], "{pages}");
    let r = requete("POST", "/__banc/api/outil/choisir", br#"{"id": "antenna"}"#, &maison);
    assert!(r.code == 404 && r.json() == json!({"erreur": "outil non installé : antenna"}), "{} {}", r.code, r.texte());
    let etat = lire("/__banc/api/etat").json();
    let par_outil = |champ: &str| etat["outils"].as_array().unwrap().iter().map(|o| o[champ].clone()).collect::<Vec<_>>();
    assert!(etat["actif"] == "3d" && par_outil("installe") == [json!(true), json!(false), json!(true)], "{etat}");
    assert!(par_outil("en_marche") == [json!(true), json!(false), json!(true)], "{etat}");
    let ports: Vec<u16> = par_outil("port").iter().filter_map(|p| p.as_u64()).filter(|p| *p != 0).map(|p| p as u16).collect();
    assert_eq!(lire("/__banc/api/inconnue").code, 404);

    // arrêt demandé de l'extérieur : les outils s'arrêtent, leurs dossiers temporaires partent
    let mut proc = menage.proc.take().unwrap();
    demander_arret(&proc);
    let limite = Instant::now() + Duration::from_secs(30);
    let code = loop {
        if let Some(etat) = proc.try_wait().unwrap() {
            break etat.code();
        }
        if Instant::now() > limite {
            let _ = proc.kill();
            panic!("web_bench ne s'arrête pas");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let mut fin = String::new();
    stdout.read_to_string(&mut fin).unwrap();
    assert!(code == Some(0) && fin.contains("[*] Arrêt."), "{code:?} {fin}");
    assert!(!temp_outil.exists(), "dossiers temporaires de l'outil non effacés");
    for p in ports {
        assert!(http::requete(p, "GET", "/", &[], b"").is_err(), "un outil tourne encore sur le port {p}");
    }
    println!("OK");
}
