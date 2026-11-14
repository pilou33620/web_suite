//! Les bancs d'un outil : scripts node banc-*.js / .mjs et cibles de `cargo test`, lancés un par un.

use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::outils::{horloge, lancer, trouver_chromium, tuer, which, Outil, DOSSIERS_IGNORES};

const DELAI_BANC: Duration = Duration::from_secs(300); // au-delà, un banc est arrêté et compté « délai dépassé »
// shortcut: un seul délai pour compiler et essayer ; à séparer (cargo test --no-run d'abord) si un banc Rust traîne.
const DELAI_CARGO: Duration = Duration::from_secs(1800);
const MAX_SORTIE: usize = 40000; // caractères gardés de la sortie d'un banc

/// L'ancienne expression : ^banc[-_].+\.(js|mjs)$
fn est_banc(nom: &str) -> bool {
    let Some(suite) = nom.strip_prefix("banc-").or_else(|| nom.strip_prefix("banc_")) else { return false };
    [".js", ".mjs"].iter().any(|ext| suite.strip_suffix(ext).is_some_and(|milieu| !milieu.is_empty()))
}

/// Les bancs Rust d'un espace de travail, d'après `cargo metadata` : par caisse, ses essais
/// unitaires (« cargo test -p caisse --lib --bins »), puis une entrée par cible de tests/.
/// L'identifiant d'un banc Rust est sa commande.
fn bancs_cargo(meta: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    for caisse in meta["packages"].as_array().into_iter().flatten() {
        let Some(nom) = caisse["name"].as_str() else { continue };
        let cibles: Vec<(&str, &str)> = caisse["targets"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["test"] != false)
            .filter_map(|c| Some((c["kind"][0].as_str()?, c["name"].as_str()?)))
            .collect();
        let a = |genres: &[&str]| cibles.iter().any(|(genre, _)| genres.contains(genre));
        let unitaires: Vec<&str> = [(a(&["lib", "rlib", "cdylib", "staticlib", "dylib", "proc-macro"]), "--lib"), (a(&["bin"]), "--bins")]
            .iter()
            .filter_map(|(present, option)| present.then_some(*option))
            .collect();
        if !unitaires.is_empty() {
            ids.push(format!("cargo test -p {} {}", nom, unitaires.join(" ")));
        }
        ids.extend(cibles.iter().filter(|(genre, _)| *genre == "test").map(|(_, cible)| format!("cargo test -p {} --test {}", nom, cible)));
    }
    ids
}

/// La CI lance-t-elle ce banc Rust ? Tout l'espace de travail, sa caisse entière, ou lui.
fn cargo_dans_ci(ci: &str, id: &str) -> bool {
    let caisse: String = id.split(' ').take(4).collect::<Vec<_>>().join(" "); // « cargo test -p caisse »
    ci.contains("cargo test --workspace") || ci.lines().any(|l| l.trim_end().ends_with(&caisse) || l.contains(id))
}

/// Les options que la CI donne à node pour ce fichier (--experimental-websocket…).
/// L'ancienne expression : node((?:\s+--[\w-]+)*)\s+<fichier>(?:\s|$)
fn options_node(ci: &str, fichier: &str) -> Vec<String> {
    'essai: for (debut, _) in ci.match_indices("node") {
        let mut reste = &ci[debut + 4..];
        let mut options = Vec::new();
        loop {
            let suite = reste.trim_start();
            if suite.len() == reste.len() {
                continue 'essai; // pas d'espace après « node » ou après une option
            }
            if let Some(apres) = suite.strip_prefix(fichier) {
                if apres.is_empty() || apres.starts_with(char::is_whitespace) {
                    return options;
                }
                continue 'essai;
            }
            let fin = suite.find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-')).unwrap_or(suite.len());
            if !suite.starts_with("--") || fin <= 2 {
                continue 'essai;
            }
            options.push(suite[..fin].to_string());
            reste = &suite[fin..];
        }
    }
    Vec::new()
}

struct Essai {
    id: String,
    ci: bool,
    debut: Option<f64>,
    etat: &'static str,
    duree: Option<f64>,
    sortie: String,
    code: Option<i64>,
    preparation: Vec<String>,
    lib_reelle: bool,
}

#[derive(Default)]
struct Etat {
    liste: Vec<Essai>,
    en_cours: bool,
}

pub struct Bancs {
    outil: Arc<Outil>,
    ci: String,
    etat: Mutex<Etat>,
    courant: Mutex<Option<Child>>, // processus en cours
    arret: AtomicBool,
}

impl Bancs {
    pub fn new(outil: Arc<Outil>) -> Arc<Bancs> {
        let ci = fs::read_to_string(outil.dossier.join(".github").join("workflows").join("ci.yml")).unwrap_or_default();
        let liste = decouvrir(&outil, &ci);
        Arc::new(Bancs {
            outil,
            ci,
            etat: Mutex::new(Etat { liste, en_cours: false }),
            courant: Mutex::new(None),
            arret: AtomicBool::new(false),
        })
    }

    pub fn nombre(&self) -> usize {
        self.etat.lock().unwrap().liste.len()
    }

    /// Les bancs que la CI de l'outil lance : ceux qui sont cochés d'office.
    pub fn ids_ci(&self) -> Vec<String> {
        self.etat.lock().unwrap().liste.iter().filter(|b| b.ci).map(|b| b.id.clone()).collect()
    }

    pub fn etat(&self) -> Value {
        let e = self.etat.lock().unwrap();
        let bancs: Vec<Value> = e
            .liste
            .iter()
            .map(|b| {
                json!({"id": b.id, "ci": b.ci, "etat": b.etat, "duree": b.duree, "code": b.code, "debut": b.debut,
                       "preparation": b.preparation, "lib_reelle": b.lib_reelle})
            })
            .collect();
        json!({"en_cours": e.en_cours, "bancs": bancs})
    }

    pub fn sortie(&self, id: &str) -> Option<String> {
        self.etat.lock().unwrap().liste.iter().find(|b| b.id == id).map(|b| b.sortie.clone())
    }

    pub fn lancer(self: &Arc<Self>, ids: &[String]) -> bool {
        let mut e = self.etat.lock().unwrap();
        if e.en_cours {
            return false;
        }
        let file: Vec<usize> = ids.iter().filter_map(|i| e.liste.iter().position(|b| &b.id == i)).collect();
        for &i in &file {
            let b = &mut e.liste[i];
            (b.etat, b.duree, b.code, b.debut) = ("prevu", None, None, None);
            b.sortie.clear();
        }
        self.arret.store(false, Ordering::SeqCst);
        e.en_cours = true;
        let bancs = self.clone();
        std::thread::spawn(move || {
            bancs.derouler(&file);
            *bancs.courant.lock().unwrap() = None;
            bancs.etat.lock().unwrap().en_cours = false;
        });
        true
    }

    pub fn arreter(&self) {
        self.arret.store(true, Ordering::SeqCst);
        if let Some(p) = self.courant.lock().unwrap().as_mut() {
            tuer(p, true);
        }
    }

    /// La commande d'un banc ou d'une préparation : cargo, ou node avec les options que la CI
    /// lui donne pour ce fichier (--experimental-websocket…). Err : le programme qui manque.
    fn commande(&self, args: &[&str]) -> Result<Vec<String>, &'static str> {
        let suite = args[1..].iter().map(|a| a.to_string());
        if args[0] == "cargo" {
            return Ok(std::iter::once(which("cargo").ok_or("cargo")?).chain(suite).collect());
        }
        let node = which("node").ok_or("node")?;
        Ok(std::iter::once(node).chain(options_node(&self.ci, args[1])).chain(suite).collect())
    }

    /// Lance une commande et rend (code, sortie) ; code None si le délai est dépassé.
    fn executer(&self, cmd: &[String], env: &[(String, String)]) -> io::Result<(Option<i64>, String)> {
        let (processus, mut tuyau) = lancer(cmd, &self.outil.dossier, env)?;
        *self.courant.lock().unwrap() = Some(processus);
        let (envoi, reception) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut sortie = Vec::new();
            let _ = tuyau.read_to_end(&mut sortie);
            let _ = envoi.send(sortie);
        });
        let cargo = Path::new(&cmd[0]).file_stem().is_some_and(|n| n == "cargo");
        let limite = Instant::now() + if cargo { DELAI_CARGO } else { DELAI_BANC };
        let (mut sortie, mut code) = (None, None);
        // le verrou de `courant` n'est tenu qu'un instant : arreter() peut tuer le processus
        while sortie.is_none() || code.is_none() {
            if sortie.is_none() {
                sortie = reception.recv_timeout(Duration::from_millis(20)).ok();
            } else {
                std::thread::sleep(Duration::from_millis(20));
            }
            if code.is_none() {
                code = self.courant.lock().unwrap().as_mut().and_then(|p| p.try_wait().ok().flatten());
            }
            if Instant::now() > limite {
                if let Some(p) = self.courant.lock().unwrap().as_mut() {
                    tuer(p, true);
                    let _ = p.wait();
                }
                let sortie = sortie.or_else(|| reception.recv().ok()).unwrap_or_default();
                return Ok((None, String::from_utf8_lossy(&sortie).into_owned()));
            }
        }
        Ok((Some(code_de_retour(code.unwrap())), String::from_utf8_lossy(&sortie.unwrap()).into_owned()))
    }

    fn noter(&self, i: usize, etat: &'static str, sortie: String) {
        let mut e = self.etat.lock().unwrap();
        e.liste[i].etat = etat;
        e.liste[i].sortie = sortie;
    }

    fn derouler(&self, file: &[usize]) {
        let conf = self.outil.conf;
        let mut env = vec![("WEB_TOOLS_HTTP".to_string(), "1".to_string())];
        if let Some(chromium) = trouver_chromium() {
            env.push(("CHROMIUM".into(), chromium));
        }
        let lib = if conf.lib { self.outil.lib_reelle() } else { None };
        if let Some(lib) = lib.as_ref().filter(|_| std::env::var_os("WEB_CAO_LIB").is_none()) {
            env.push(("WEB_CAO_LIB".into(), lib.to_string_lossy().into_owned()));
        }
        let arret = || self.arret.load(Ordering::SeqCst);
        for &i in file {
            let (id, lib_reelle) = {
                let e = self.etat.lock().unwrap();
                (e.liste[i].id.clone(), e.liste[i].lib_reelle)
            };
            if arret() {
                self.etat.lock().unwrap().liste[i].etat = "attente";
                continue;
            }
            if lib_reelle && lib.is_none() {
                self.noter(
                    i,
                    "ignore",
                    "Sauté : ce banc éprouve le CONTENU de la LIB privée, introuvable ici \
                     (PROJETS/LIB_CAO, WEB_CAO/LIB ou WEB_CAO_LIB). La CI de WEB_CAO le saute aussi \
                     sans elle. Clone WEB_SUITE_PROJETS dans PROJETS/ pour le lancer."
                        .into(),
                );
                continue;
            }
            let cmds: Result<Vec<Vec<String>>, _> = preparations(conf, &id).iter().map(|c| self.commande(c)).collect();
            // un banc Rust est sa commande ; les autres sont des scripts node
            let mots: Vec<&str> = if id.starts_with("cargo ") { id.split(' ').collect() } else { vec!["node", &id] };
            let (cmds, cmd) = match (cmds, self.commande(&mots)) {
                (Ok(cmds), Ok(cmd)) => (cmds, cmd),
                (Err(manque), _) | (_, Err(manque)) => {
                    self.noter(i, "ignore", format!("{} est introuvable : ce banc ne peut pas tourner.", manque));
                    continue;
                }
            };
            let debut = horloge();
            {
                let mut e = self.etat.lock().unwrap();
                e.liste[i].etat = "en_cours";
                e.liste[i].debut = Some(debut);
            }
            let mut texte = String::new();
            let (mut code, mut etat) = (None, None);
            let mut derouler_un = || -> io::Result<()> {
                for prep in &cmds {
                    let programme = Path::new(&prep[0]).file_name().unwrap_or_default().to_string_lossy().into_owned();
                    let mots: Vec<&str> = std::iter::once(programme.as_str()).chain(prep[1..].iter().map(String::as_str)).collect();
                    texte += &format!("$ {}   (préparation)\n", mots.join(" "));
                    let (c, sortie) = self.executer(prep, &env)?;
                    code = c;
                    texte += &sortie;
                    texte += "\n";
                    if c != Some(0) {
                        etat = Some(if c.is_none() { "delai" } else if arret() { "arrete" } else { "echec" });
                        texte += "[préparation en échec : le banc n'est pas lancé]\n";
                        return Ok(());
                    }
                }
                if !cmds.is_empty() {
                    texte += &format!("$ {}\n", id);
                }
                let (c, sortie) = self.executer(&cmd, &env)?;
                code = c;
                texte += &sortie;
                etat = Some(match c {
                    None => "delai",
                    Some(0) => "ok",
                    Some(_) if arret() => "arrete",
                    Some(_) => "echec",
                });
                Ok(())
            };
            if let Err(exc) = derouler_un() {
                (code, etat) = (None, Some("echec"));
                texte += &format!("Lancement impossible : {}", exc);
            }
            let trop = texte.chars().count().saturating_sub(MAX_SORTIE);
            let mut e = self.etat.lock().unwrap();
            let b = &mut e.liste[i];
            b.etat = etat.unwrap_or("echec");
            b.code = code;
            b.duree = Some(((horloge() - debut) * 10.0).round() / 10.0);
            b.sortie = texte.chars().skip(trop).collect();
        }
    }
}

/// Comme Popen.returncode : le code de sortie, ou −signal si un signal a arrêté le processus.
fn code_de_retour(statut: std::process::ExitStatus) -> i64 {
    #[cfg(unix)]
    if let Some(signal) = std::os::unix::process::ExitStatusExt::signal(&statut) {
        return -(signal as i64);
    }
    statut.code().unwrap_or(-1) as i64
}

fn preparations(conf: &crate::outils::Conf, id: &str) -> &'static [&'static [&'static str]] {
    conf.preparations.iter().find(|(banc, _)| *banc == id).map(|(_, p)| *p).unwrap_or(&[])
}

/// Parcours comme os.walk : les fichiers du dossier, puis ses sous-dossiers, par ordre de nom.
fn parcourir(dossier: &Path, rel: &str, trouves: &mut Vec<String>) {
    let Ok(entrees) = fs::read_dir(dossier) else { return };
    let mut noms: Vec<String> = entrees.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    noms.sort();
    let mut dossiers = Vec::new();
    for nom in noms {
        let chemin = dossier.join(&nom);
        if chemin.is_dir() {
            if !DOSSIERS_IGNORES.contains(&nom.as_str()) && !chemin.is_symlink() {
                dossiers.push(nom);
            }
        } else if est_banc(&nom) {
            trouves.push(format!("{}{}", rel, nom));
        }
    }
    for d in dossiers {
        parcourir(&dossier.join(&d), &format!("{}{}/", rel, d), trouves);
    }
}

/// Les scripts banc-* de l'outil, les essais que sa CI lance sous un autre nom (harness.js…),
/// et ses bancs Rust. Ceux que la CI lance (.github/workflows/ci.yml) sont marqués : ils sont
/// cochés d'office. Les autres demandent parfois une préparation (LIB privée, dossier PROJETS).
fn decouvrir(outil: &Outil, ci: &str) -> Vec<Essai> {
    let conf = outil.conf;
    let mut trouves = Vec::new();
    parcourir(&outil.dossier, "", &mut trouves);
    for e in conf.essais {
        if outil.dossier.join(e).is_file() && !trouves.iter().any(|t| t == e) {
            trouves.push(e.to_string());
        }
    }
    // sans cargo (poste qui prend les exécutables publiés) : pas de bancs Rust
    if let (true, Some(cargo)) = (outil.dossier.join("Cargo.toml").is_file(), which("cargo")) {
        let sortie = Command::new(cargo)
            .args(["metadata", "--no-deps", "--format-version", "1"])
            .current_dir(&outil.dossier)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        if let Some(meta) = sortie.ok().and_then(|s| serde_json::from_slice::<Value>(&s.stdout).ok()) {
            trouves.extend(bancs_cargo(&meta));
        }
    }
    let mut liste: Vec<Essai> = trouves
        .into_iter()
        .map(|rel| Essai {
            ci: if rel.starts_with("cargo ") { cargo_dans_ci(ci, &rel) } else { ci.contains(&rel) },
            debut: None,
            etat: "attente",
            duree: None,
            sortie: String::new(),
            code: None,
            preparation: preparations(conf, &rel).iter().map(|c| c.join(" ")).collect(),
            lib_reelle: conf.lib_reelle.contains(&rel.as_str()),
            id: rel,
        })
        .collect();
    liste.sort_by(|a, b| (!a.ci, &a.id).cmp(&(!b.ci, &b.id)));
    liste
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outils::Conf;

    #[test]
    fn noms_des_bancs() {
        for nom in ["banc-a.js", "banc_b.js", "banc-c.mjs", "banc-x.py.js"] {
            assert!(est_banc(nom), "{}", nom);
        }
        for nom in ["banc-.js", "banc.js", "xbanc-a.js", "banc-a.txt", "banc-a.js~", "banc-a.py", "harness.js"] {
            assert!(!est_banc(nom), "{}", nom);
        }
    }

    #[test]
    fn bancs_rust() {
        let cible = |genre: &str, nom: &str| json!({"kind": [genre], "name": nom, "test": true});
        let meta = json!({"packages": [
            {"name": "serveur", "targets": [cible("lib", "serveur"), cible("bin", "web_x"), cible("test", "banc_routes"),
                                            cible("example", "demo"), {"kind": ["test"], "name": "eteint", "test": false}]},
            {"name": "outils", "targets": [cible("bin", "build-monofichier"), cible("custom-build", "build-script-build")]},
            {"name": "vide", "targets": [cible("bench", "vitesse")]},
        ]});
        let ids = bancs_cargo(&meta);
        assert_eq!(ids, ["cargo test -p serveur --lib --bins", "cargo test -p serveur --test banc_routes", "cargo test -p outils --bins"]);
        assert!(bancs_cargo(&json!({})).is_empty());
        assert!(ids.iter().all(|id| cargo_dans_ci("  run: cargo test --workspace\n", id)));
        let ci = "  run: cargo test -p serveur\n  run: cargo test -p outils --test banc_essai -- --nocapture\n";
        assert!(cargo_dans_ci(ci, &ids[0]) && cargo_dans_ci(ci, &ids[1]) && !cargo_dans_ci(ci, &ids[2]));
        assert!(cargo_dans_ci(ci, "cargo test -p outils --test banc_essai"));
        assert!(!cargo_dans_ci(ci, "cargo test -p outils --test banc_ruban"));
    }

    #[test]
    fn options_de_node_dans_la_ci() {
        let ci = "run: node test/autre.js\n  run: node --experimental-websocket --no-warnings test/banc-a.js\n\
                  run: node test/banc-b.js --verbeux\nrun: node test/banc-c.jsx\nrun: nodemon test/banc-d.js";
        assert_eq!(options_node(ci, "test/banc-a.js"), ["--experimental-websocket", "--no-warnings"]);
        assert!(options_node(ci, "test/banc-b.js").is_empty());
        assert!(options_node(ci, "test/banc-c.js").is_empty());
        assert!(options_node(ci, "test/banc-d.js").is_empty());
        assert_eq!(options_node("node --a x.js", "x.js"), ["--a"]);
    }

    /// Un faux WEB_CAO : un essai sous un autre nom, une préparation, un banc qui exige la LIB privée.
    static FAUX: Conf = Conf {
        id: "cao",
        nom: "WEB_CAO",
        lib: true,
        args: &[],
        noire: "",
        essais: &["test/harness.js"],
        preparations: &[("test/banc-bon.js", &[&["node", "prep.js"]])],
        lib_reelle: &["test/banc-lib.js"],
        routes: None,
    };

    fn attendre(bancs: &Bancs) -> Value {
        for _ in 0..600 {
            let e = bancs.etat();
            if e["en_cours"] == false {
                return e;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("les bancs ne finissent pas");
    }

    #[test]
    fn bancs_du_faux_outil() {
        let temp = std::env::temp_dir().join(format!("essai_web_bench_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let cao = temp.join("WEB_CAO");
        fs::create_dir_all(cao.join("test")).unwrap();
        fs::create_dir_all(cao.join("graphify-out")).unwrap();
        fs::create_dir_all(cao.join(".github/workflows")).unwrap();
        fs::create_dir_all(temp.join("PROJETS/LIB_CAO")).unwrap();
        fs::write(temp.join("PROJETS/LIB_CAO/LIB_composants.csv"), "ref;valeur\n").unwrap();
        fs::write(cao.join(".github/workflows/ci.yml"), "run: node test/banc-mauvais.js\n").unwrap();
        fs::write(cao.join("test/harness.js"), "console.log('harness ok')\n").unwrap();
        fs::write(cao.join("prep.js"), "require('fs').writeFileSync('prepare.txt', 'fait')\nconsole.log('préparé')\n").unwrap();
        fs::write(
            cao.join("test/banc-lib.js"),
            "console.log(process.env.WEB_CAO_LIB)\n\
             process.exit(require('fs').existsSync(process.env.WEB_CAO_LIB + '/LIB_composants.csv') ? 0 : 4)\n",
        )
        .unwrap();
        fs::write(cao.join("test/banc-bon.js"), "console.log('OK')\n").unwrap();
        fs::write(cao.join("test/banc-mauvais.js"), "console.log('raté')\nprocess.exit(3)\n").unwrap();
        fs::write(cao.join("graphify-out/banc-ignore.js"), "process.exit(1)\n").unwrap();

        let bancs = Bancs::new(Arc::new(Outil::new(&FAUX, cao.clone())));
        // bancs : graphify-out ignoré, l'essai harness.js ajouté ; celui de la CI vient en tête
        let e = bancs.etat();
        let ids: Vec<String> = e["bancs"].as_array().unwrap().iter().map(|b| b["id"].as_str().unwrap().to_string()).collect();
        assert_eq!(ids, ["test/banc-mauvais.js", "test/banc-bon.js", "test/banc-lib.js", "test/harness.js"]);
        assert_eq!(bancs.ids_ci(), ["test/banc-mauvais.js"]);
        assert_eq!(e["bancs"][1]["preparation"], json!(["node prep.js"]));
        assert_eq!(e["bancs"][2]["lib_reelle"], true);

        assert!(bancs.lancer(&ids));
        assert!(!bancs.lancer(&ids), "un seul déroulement à la fois");
        let e = attendre(&bancs);
        let etats: Vec<(&str, Option<i64>)> =
            e["bancs"].as_array().unwrap().iter().map(|b| (b["etat"].as_str().unwrap(), b["code"].as_i64())).collect();
        if which("node").is_none() {
            assert_eq!(etats, [("ignore", None); 4], "{}", e);
            assert!(bancs.sortie("test/banc-bon.js").unwrap().starts_with("node est introuvable"));
            println!("SAUTÉ : pas de node sur ce poste");
            fs::remove_dir_all(&temp).unwrap();
            return;
        }
        assert_eq!(etats, [("echec", Some(3)), ("ok", Some(0)), ("ok", Some(0)), ("ok", Some(0))], "{}", e);
        let bon = bancs.sortie("test/banc-bon.js").unwrap();
        assert!(bon.contains("préparé") && bon.contains("(préparation)") && bon.contains("$ test/banc-bon.js"), "{}", bon);
        assert!(cao.join("prepare.txt").is_file());
        assert!(bancs.sortie("test/banc-mauvais.js").unwrap().contains("raté"));
        assert!(bancs.sortie("inconnu").is_none());

        // sans la LIB privée : le banc qui l'exige est sauté, comme dans la CI
        fs::remove_dir_all(temp.join("PROJETS")).unwrap();
        assert!(bancs.lancer(&["test/banc-lib.js".to_string()]));
        let e = attendre(&bancs);
        assert_eq!(e["bancs"][2]["etat"], "ignore", "{}", e);
        assert!(bancs.sortie("test/banc-lib.js").unwrap().starts_with("Sauté"));

        // un banc arrêté en route : compté « arrete », avec −signal pour code
        fs::write(cao.join("test/banc-bon.js"), "console.log('parti')\nsetTimeout(() => {}, 60000)\n").unwrap();
        fs::write(cao.join("prep.js"), "").unwrap();
        assert!(bancs.lancer(&["test/banc-bon.js".to_string(), "test/banc-mauvais.js".to_string()]));
        for _ in 0..600 {
            if bancs.sortie("test/banc-bon.js").is_some() && bancs.etat()["bancs"][1]["etat"] == "en_cours" {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        std::thread::sleep(Duration::from_millis(500));
        bancs.arreter();
        let e = attendre(&bancs);
        assert_eq!(e["bancs"][1]["etat"], "arrete", "{}", e);
        assert_eq!(e["bancs"][0]["etat"], "attente", "{}", e);
        fs::remove_dir_all(&temp).unwrap();
    }
}
