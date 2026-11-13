//! Banc de l'installation des dépendances des outils.
//!
//!     cargo test -p web_suite_lanceur --test banc_dependances
//!
//! pip n'est lancé que s'il manque un paquet (sur Debian, il refuse la Python du
//! système même quand apt a tout installé) ; un lancement à qui il en manque un
//! lance l'installation ; openEMS ne pose aucune question hors Windows. Rien n'est
//! installé : les exécutables (installer, web_suite) sont pilotés en boîte noire, et
//! la Python qu'ils lancent (WEB_SUITE_PYTHON) est une fausse, qui note ses arguments,
//! dit quels modules manquent et fait échouer pip. Affiche OK si tout passe.
//!
//! La fausse Python est cet exécutable de test lui-même, relancé avec
//! FAUSSE_PYTHON_JOURNAL (d'où « harness = false » dans Cargo.toml) : ni script ni
//! vraie Python, et la même chose sous Windows.
mod commun;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use commun::{chaine, Poste, INSTALLER};
use serde_json::{json, Value};

const REQ: &str = "\
# commentaire
numpy>=1.24.0
scipy>=1.10.0   # solveur DC
NumPy==2.0      # doublon
";

/// La fausse Python : note ses arguments (une ligne JSON), puis rend 1 pour un module
/// manquant ou pour « -m pip install », 0 sinon.
fn fausse_python(journal: &str) -> i32 {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mut f = fs::File::options().create(true).append(true).open(journal).unwrap();
    writeln!(f, "{}", json!(a)).unwrap();
    if a.first().is_some_and(|x| x == "-c") && a.get(1).is_some_and(|x| x.starts_with("import ")) {
        let module = a[1].split_whitespace().nth(1).unwrap_or("");
        let manquants = std::env::var("FAUSSE_PYTHON_MANQUANTS").unwrap_or_default();
        return manquants.split(',').any(|m| m == module) as i32;
    }
    (a.len() >= 3 && a[..3] == ["-m", "pip", "install"]) as i32
}

/// Les commandes reçues par la fausse Python, puis le journal est vidé.
fn lancees(journal: &Path) -> Vec<Value> {
    let Ok(texte) = fs::read_to_string(journal) else { return Vec::new() };
    fs::remove_file(journal).unwrap();
    texte.lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}

fn pip_install(c: &Value) -> bool {
    c.as_array().is_some_and(|a| a.len() >= 3 && a[..3] == [json!("-m"), json!("pip"), json!("install")])
}

fn main() {
    if let Ok(journal) = std::env::var("FAUSSE_PYTHON_JOURNAL") {
        std::process::exit(fausse_python(&journal));
    }
    let poste = Poste::new("dependances");
    let tmp = poste.racine.clone();
    let racine = chaine(&tmp);
    let exe = chaine(&std::env::current_exe().unwrap());
    let journal = poste.tmp.join("fausse_python.log");
    let journal_txt = chaine(&journal);
    let env = [("WEB_SUITE_PYTHON", exe.as_str()), ("FAUSSE_PYTHON_JOURNAL", journal_txt.as_str())];
    let manque_scipy = [env[0], env[1], ("FAUSSE_PYTHON_MANQUANTS", "scipy")];

    // 1. Rien à faire si tout est là, et un échec de pip est dit, avec la commande apt.
    fs::create_dir_all(tmp.join("WEB_CAO")).unwrap();
    fs::write(tmp.join("WEB_CAO").join("requirements.txt"), REQ).unwrap();
    fs::write(tmp.join("WEB_CAO").join("web_CAO.py"), "").unwrap();
    let (code, sortie) = poste.lancer(INSTALLER, &["cao", "--racine", &racine], &env);
    let cmds = lancees(&journal);
    assert!(code == 0 && sortie.contains("Dépendances déjà présentes"), "{sortie}");
    // doublon écarté, pip pas lancé
    assert_eq!(cmds, [json!(["-c", "import numpy"]), json!(["-c", "import scipy"])]);
    let (code, sortie) = poste.lancer(INSTALLER, &["cao", "--racine", &racine], &manque_scipy);
    let cmds = lancees(&journal);
    let lignes: Vec<&str> = sortie.lines().collect();
    assert!(code == 1 && cmds.iter().any(pip_install), "{code} {cmds:?}");
    assert!(lignes.iter().any(|l| l.contains("ne démarrera pas") && l.contains("scipy") && !l.contains("numpy")), "{lignes:?}");
    if !cfg!(windows) {
        assert!(lignes.iter().any(|l| l.contains("apt install python3-scipy")), "{lignes:?}");
    }
    let (code, sortie) = poste.lancer(INSTALLER, &["cao", "--racine", &racine, "--sans-dependances"], &manque_scipy);
    assert!(code == 0 && lancees(&journal).is_empty(), "{sortie}");

    // 2. Lancer un outil à qui il manque un paquet : pas de serveur, l'installation part.
    {
        let l = poste.lanceur(&["--local"], &manque_scipy);
        let cao = || l.get("/api/etat").json()["outils"][0].clone();
        let r = l.req("POST", "/api/lancer", &[("X-WebSuite", "1")], br#"{"id":"web_cao"}"#);
        assert!(r.code == 200, "{}", r.texte());
        for _ in 0..200 {
            let phase = cao()["phase"].clone();
            if phase != "installation" && phase != "maj" {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let cmds: Vec<Value> = lancees(&journal).into_iter().filter(|c| c[0] != "-c").collect();
        assert!(cmds.first().is_some_and(pip_install), "{cmds:?}");
        let e = cao();
        assert!(e["id"] == "web_cao" && e["phase"] == "erreur" && e["url"].is_null(), "{e}");
    }

    // 3. Hors Windows, openEMS ne s'installe pas tout seul : aucune question à poser.
    if !cfg!(windows) {
        fs::create_dir_all(tmp.join("WEB_ANTENNA")).unwrap();
        fs::write(tmp.join("WEB_ANTENNA").join("web_antenna.py"), "").unwrap();
        let (code, sortie) = poste.lancer(INSTALLER, &["antenna", "--racine", &racine], &env);
        assert!(code == 0 && sortie.contains("que sous Windows"), "{sortie}");
        assert!(!sortie.contains("Installer openEMS maintenant"), "question openEMS posée hors Windows");
    }

    println!("OK");
}
