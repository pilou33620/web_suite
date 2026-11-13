//! Les bancs d'un outil : scripts banc-*.py / .js / .mjs, lancés un par un.

use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::outils::{horloge, lancer, python, trouver_chromium, which, Outil, DOSSIERS_IGNORES};

const DELAI_BANC: Duration = Duration::from_secs(300); // au-delà, un banc est arrêté et compté « délai dépassé »
const MAX_SORTIE: usize = 40000; // caractères gardés de la sortie d'un banc

/// L'ancienne expression : ^banc[-_].+\.(py|js|mjs)$
fn est_banc(nom: &str) -> bool {
    let Some(suite) = nom.strip_prefix("banc-").or_else(|| nom.strip_prefix("banc_")) else { return false };
    [".py", ".js", ".mjs"].iter().any(|ext| suite.strip_suffix(ext).is_some_and(|milieu| !milieu.is_empty()))
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
            let _ = p.kill();
        }
    }

    /// La commande d'un banc ou d'une préparation : la même Python que les outils, ou node avec
    /// les options que la CI lui donne pour ce fichier (--experimental-websocket…).
    fn commande(&self, args: &[&str]) -> Option<Vec<String>> {
        let suite = args[1..].iter().map(|a| a.to_string());
        if args[0] == "python" {
            return Some(std::iter::once(python().to_string()).chain(suite).collect());
        }
        let node = which("node")?;
        Some(std::iter::once(node).chain(options_node(&self.ci, args[1])).chain(suite).collect())
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
        let limite = Instant::now() + DELAI_BANC;
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
                    let _ = p.kill();
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
        let mut env = vec![("PYTHONIOENCODING".to_string(), "utf-8".to_string()), ("WEB_TOOLS_HTTP".into(), "1".into())];
        if let Some(chromium) = trouver_chromium() {
            env.push(("CHROMIUM".into(), chromium));
        }
        let lib = if conf.lib { self.outil.lib_reelle() } else { None };
        if conf.lib {
            match &lib {
                Some(lib) if std::env::var_os("WEB_CAO_LIB").is_none() => {
                    env.push(("WEB_CAO_LIB".into(), lib.to_string_lossy().into_owned()))
                }
                Some(_) => {}
                None => env.push(("WEB_CAO_LIB_MINIMALE".into(), "1".into())), // comme la CI sans la LIB privée
            }
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
            let cmds: Option<Vec<Vec<String>>> = preparations(conf, &id).iter().map(|c| self.commande(c)).collect();
            let cmd = self.commande(&[if id.ends_with(".py") { "python" } else { "node" }, &id]);
            let (Some(cmds), Some(cmd)) = (cmds, cmd) else {
                self.noter(i, "ignore", "node est introuvable : ce banc JavaScript ne peut pas tourner.".into());
                continue;
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

/// Les scripts banc-* de l'outil, plus les essais que sa CI lance sous un autre nom
/// (harness.js…). Ceux que la CI lance (.github/workflows/ci.yml) sont marqués : ils sont
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
    let mut liste: Vec<Essai> = trouves
        .into_iter()
        .map(|rel| Essai {
            ci: ci.contains(&rel),
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
        for nom in ["banc-a.py", "banc_b.js", "banc-c.mjs", "banc-x.py.js"] {
            assert!(est_banc(nom), "{}", nom);
        }
        for nom in ["banc-.py", "banc.py", "xbanc-a.py", "banc-a.txt", "banc-a.py~", "harness.js"] {
            assert!(!est_banc(nom), "{}", nom);
        }
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

    /// Le faux WEB_CAO de l'ancien banc_web_bench.py, avec le catalogue qu'il y substituait :
    /// un essai sous un autre nom, une préparation en Python, un banc qui exige la LIB privée.
    static FAUX: Conf = Conf {
        id: "cao",
        nom: "WEB_CAO",
        script: "web_CAO.py",
        lib: true,
        args: &[],
        noire: "",
        essais: &["test/harness.js"],
        preparations: &[("test/banc-bon.py", &[&["python", "prep.py"]])],
        lib_reelle: &["test/banc-lib.py"],
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
        fs::write(cao.join(".github/workflows/ci.yml"), "run: python test/banc-mauvais.py\n").unwrap();
        fs::write(cao.join("test/harness.js"), "console.log('harness ok')\n").unwrap();
        fs::write(cao.join("prep.py"), "open('prepare.txt', 'w').write('fait')\nprint('préparé')\n").unwrap();
        fs::write(
            cao.join("test/banc-lib.py"),
            "import os, sys\nprint(os.environ.get('WEB_CAO_LIB'))\n\
             sys.exit(0 if os.path.isfile(os.path.join(os.environ['WEB_CAO_LIB'], 'LIB_composants.csv')) else 4)\n",
        )
        .unwrap();
        fs::write(cao.join("test/banc-bon.py"), "print('OK')\n").unwrap();
        fs::write(cao.join("test/banc-mauvais.py"), "import sys\nprint('raté')\nsys.exit(3)\n").unwrap();
        fs::write(cao.join("graphify-out/banc-ignore.py"), "raise SystemExit(1)\n").unwrap();

        let bancs = Bancs::new(Arc::new(Outil::new(&FAUX, cao.clone())));
        // bancs : graphify-out ignoré, l'essai harness.js ajouté ; celui de la CI vient en tête
        let e = bancs.etat();
        let ids: Vec<String> = e["bancs"].as_array().unwrap().iter().map(|b| b["id"].as_str().unwrap().to_string()).collect();
        assert_eq!(ids, ["test/banc-mauvais.py", "test/banc-bon.py", "test/banc-lib.py", "test/harness.js"]);
        assert_eq!(bancs.ids_ci(), ["test/banc-mauvais.py"]);
        assert_eq!(e["bancs"][1]["preparation"], json!(["python prep.py"]));
        assert_eq!(e["bancs"][2]["lib_reelle"], true);

        assert!(bancs.lancer(&ids));
        assert!(!bancs.lancer(&ids), "un seul déroulement à la fois");
        let e = attendre(&bancs);
        let etats: Vec<(&str, Option<i64>)> =
            e["bancs"].as_array().unwrap().iter().map(|b| (b["etat"].as_str().unwrap(), b["code"].as_i64())).collect();
        let harness = if which("node").is_some() { ("ok", Some(0)) } else { ("ignore", None) };
        assert_eq!(etats, [("echec", Some(3)), ("ok", Some(0)), ("ok", Some(0)), harness], "{}", e);
        let bon = bancs.sortie("test/banc-bon.py").unwrap();
        assert!(bon.contains("préparé") && bon.contains("(préparation)") && bon.contains("$ test/banc-bon.py"), "{}", bon);
        assert!(cao.join("prepare.txt").is_file());
        assert!(bancs.sortie("test/banc-mauvais.py").unwrap().contains("raté"));
        assert!(bancs.sortie("inconnu").is_none());

        // sans la LIB privée : le banc qui l'exige est sauté, comme dans la CI
        fs::remove_dir_all(temp.join("PROJETS")).unwrap();
        assert!(bancs.lancer(&["test/banc-lib.py".to_string()]));
        let e = attendre(&bancs);
        assert_eq!(e["bancs"][2]["etat"], "ignore", "{}", e);
        assert!(bancs.sortie("test/banc-lib.py").unwrap().starts_with("Sauté"));

        // un banc arrêté en route : compté « arrete », avec −signal pour code
        fs::write(cao.join("test/banc-bon.py"), "import time\nprint('parti', flush=True)\ntime.sleep(60)\n").unwrap();
        fs::write(cao.join("prep.py"), "").unwrap();
        assert!(bancs.lancer(&["test/banc-bon.py".to_string(), "test/banc-mauvais.py".to_string()]));
        for _ in 0..600 {
            if bancs.sortie("test/banc-bon.py").is_some() && bancs.etat()["bancs"][1]["etat"] == "en_cours" {
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
