//! Les outils essayés, lancés à part avec des dossiers temporaires.

use std::fs;
use std::io::{self, BufRead, BufReader, PipeReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime};

/// Un outil du catalogue, et comment le lancer sans toucher aux données de l'utilisateur.
pub struct Conf {
    pub id: &'static str,
    pub nom: &'static str,
    pub script: &'static str,
    pub lib: bool,
    /// {projets} et {lib} sont remplacés par les dossiers temporaires.
    pub args: &'static [&'static str],
    pub noire: &'static str,
    pub essais: &'static [&'static str],
    pub preparations: &'static [(&'static str, &'static [&'static [&'static str]])],
    pub lib_reelle: &'static [&'static str],
    /// Requêtes (autres que GET) que le relais arrête pour cet outil.
    pub routes: Option<fn(&str) -> bool>,
}

pub const CATALOGUE: &[Conf] = &[
    Conf {
        id: "cao",
        nom: "WEB_CAO",
        script: "web_CAO.py",
        lib: true,
        args: &["--local", "--sans-navigateur", "--sans-pause", "--sans-maj", "--projets", "{projets}", "--lib", "{lib}"],
        // la simulation se lance à la main ici : rien de lourd par défaut
        noire: "",
        // Ce que la CI de WEB_CAO fait en plus des banc-* (voir son .github/workflows/ci.yml) :
        // - les essais qui ne s'appellent pas banc-* ;
        essais: &[
            "editeur-pcb/test/harness.js",
            "editeur-schematique/test/harness.js",
            "visionneuse-ipc2581/test/harness-sim.js",
        ],
        // - les préparations : l'assemblage du monofichier, et l'export que harness.js écrit ;
        preparations: &[
            ("editeur-pcb/test/harness.js", &[&["python", "editeur-pcb/outils/build-monofichier.py"]]),
            ("editeur-schematique/test/harness.js", &[&["python", "editeur-schematique/outils/build-monofichier.py"]]),
            (
                "editeur-pcb/test/banc-ipc2581-export.py",
                &[&["python", "editeur-pcb/outils/build-monofichier.py"], &["node", "editeur-pcb/test/harness.js"]],
            ),
        ],
        // - la LIB privée (WEB_SUITE_PROJETS) : sans elle, la CI saute ces bancs-là et donne aux
        //   autres une LIB factice minimale (WEB_CAO_LIB_MINIMALE, python/test/lib_essai.py).
        lib_reelle: &["python/test/banc-lib-routes.py", "python/test/banc-rf.py", "gestion-lib/test/banc-catalogue.js"],
        routes: None,
    },
    Conf {
        id: "antenna",
        nom: "WEB_ANTENNA",
        script: "web_antenna.py",
        lib: false,
        // --sans-banc : pas de mesure de vitesse du poste au démarrage
        args: &["--local", "--sans-navigateur", "--sans-maj", "--sans-banc", "--projets", "{projets}"],
        noire: "",
        essais: &[],
        preparations: &[],
        lib_reelle: &[],
        // une simulation openEMS peut durer des heures : le bouton est cliqué (son code est essayé),
        // mais la requête qui la lancerait est arrêtée par le relais
        // (l'ancienne expression : ^/api/openems/.*lancer)
        routes: Some(|chemin| chemin.strip_prefix("/api/openems/").is_some_and(|suite| suite.contains("lancer"))),
    },
    Conf {
        id: "3d",
        nom: "WEB_3D",
        script: "web_3D.py",
        lib: false,
        args: &["--local", "--sans-navigateur", "--sans-pause", "--sans-maj", "--projets", "{projets}"],
        noire: "",
        // les essais propres de appairage.py et pilote.py, que la CI de WEB_3D lance aussi
        essais: &["appairage.py", "pilote.py"],
        preparations: &[],
        lib_reelle: &[],
        routes: None,
    },
];

/// dossiers où l'on ne cherche ni bancs ni pages
pub const DOSSIERS_IGNORES: &[&str] = &[
    ".git", "graphify-out", "node_modules", "tasks", "__pycache__", "projets", "LIB", "vendor", "env", "venv", ".venv",
    "dist", "docs",
];
const MAX_JOURNAL: usize = 4000; // lignes gardées du journal d'un outil
const GROS_FICHIER: u64 = 256 * 1024; // o : au-delà (modèles 3D STEP…), un lien dur plutôt qu'une copie

/// les gros modèles Murata de lib_simulation : ni l'outil ni le clic-partout n'en ont besoin, et
/// les recopier à chaque démarrage prendrait des minutes (même choix que python/test/lib_essai.py)
/// (les anciens motifs : ".git", "CAPA G*", "INDUC *", "*.zip")
fn lib_ignoree(nom: &str) -> bool {
    // fnmatch ne distingue pas la casse sous Windows
    let nom = if cfg!(windows) { nom.to_lowercase() } else { nom.to_string() };
    let motif = |m: &str| if cfg!(windows) { m.to_lowercase() } else { m.to_string() };
    nom == ".git" || nom.starts_with(&motif("CAPA G")) || nom.starts_with(&motif("INDUC ")) || nom.ends_with(".zip")
}

/// Copie d'un fichier de la LIB pour le banc. Les gros fichiers (Models/*.stp, 290 Mo en tout)
/// sont liés en dur quand le disque le permet : instantané, et sans risque pour la vraie LIB,
/// car l'outil les remplace par écriture atomique (nouveau fichier puis renommage) sans jamais
/// les réécrire en place. Le catalogue et les petits fichiers sont toujours vraiment copiés.
fn copier_ou_lier(source: &Path, cible: &Path) -> io::Result<()> {
    let meta = fs::metadata(source)?;
    if meta.len() > GROS_FICHIER && fs::hard_link(source, cible).is_ok() {
        return Ok(());
    }
    fs::copy(source, cible)?;
    // comme shutil.copy2 : la date du fichier suit (un fichier en lecture seule la garde neuve)
    if let (Ok(date), Ok(f)) = (meta.modified(), fs::File::options().write(true).open(cible)) {
        let _ = f.set_modified(date);
    }
    Ok(())
}

// shortcut: s'arrête à la première erreur (copytree les rassemblait toutes avant d'échouer).
fn copier_arbre(source: &Path, cible: &Path) -> io::Result<()> {
    fs::create_dir(cible)?;
    for e in fs::read_dir(source)? {
        let e = e?;
        let nom = e.file_name();
        if lib_ignoree(&nom.to_string_lossy()) {
            continue;
        }
        let (s, c) = (e.path(), cible.join(&nom));
        if s.is_dir() {
            copier_arbre(&s, &c)?;
        } else {
            copier_ou_lier(&s, &c)?;
        }
    }
    Ok(())
}

/// La LIB factice de la CI de WEB_CAO sans la LIB privée (python/test/lib_essai.py) : de quoi
/// que l'outil démarre et réponde, au lieu d'un dossier vide qui fait échouer chaque page.
fn lib_minimale(dossier: &Path) -> io::Result<()> {
    for d in ["lib_empreinte_pcb", "lib_empreinte_schematique", "lib_simulation"] {
        fs::create_dir_all(dossier.join(d))?;
    }
    fs::write(dossier.join("LIB_composants.csv"), "Part Name;Part Type;Description\n")?;
    fs::write(dossier.join("lib_empreinte_pcb").join("0603.json"), "{}\n")
}

fn port_libre() -> io::Result<u16> {
    Ok(std::net::TcpListener::bind(("127.0.0.1", 0))?.local_addr()?.port())
}

/// tempfile.mkdtemp : un dossier neuf, à soi seul, dans le dossier temporaire du système.
fn dossier_temporaire(prefixe: &str) -> io::Result<PathBuf> {
    use std::hash::{BuildHasher, Hasher};
    loop {
        // RandomState : le seul aléa de la bibliothèque standard
        let alea = std::collections::hash_map::RandomState::new().build_hasher().finish() as u32;
        let dossier = std::env::temp_dir().join(format!("{}{:08x}", prefixe, alea));
        let mut creation = fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut creation, 0o700);
        match creation.create(&dossier) {
            Ok(()) => return Ok(dossier),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
}

fn executable(chemin: &Path) -> bool {
    let Ok(meta) = fs::metadata(chemin) else { return false };
    #[cfg(unix)]
    if std::os::unix::fs::PermissionsExt::mode(&meta.permissions()) & 0o111 == 0 {
        return false;
    }
    meta.is_file()
}

/// shutil.which : le programme `nom` dans le PATH (sous Windows : le dossier courant
/// d'abord, et les extensions de PATHEXT).
pub fn which(nom: &str) -> Option<String> {
    let mut extensions = vec![String::new()];
    let mut dossiers: Vec<PathBuf> = std::env::split_paths(&std::env::var_os("PATH")?).collect();
    if cfg!(windows) {
        let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
        let connues: Vec<String> = pathext.split(';').filter(|e| !e.is_empty()).map(str::to_string).collect();
        if !connues.iter().any(|e| nom.to_lowercase().ends_with(&e.to_lowercase())) {
            extensions = connues;
        }
        dossiers.insert(0, PathBuf::from("."));
    }
    for d in dossiers {
        for e in &extensions {
            let candidat = d.join(format!("{}{}", nom, e));
            if executable(&candidat) {
                return Some(candidat.to_string_lossy().into_owned());
            }
        }
    }
    None
}

pub fn trouver_chromium() -> Option<String> {
    if let Some(c) = std::env::var("CHROMIUM").ok().filter(|c| !c.is_empty()) {
        return Some(c);
    }
    for nom in ["chromium", "chromium-browser", "google-chrome", "google-chrome-stable", "chrome", "msedge"] {
        if let Some(chemin) = which(nom) {
            return Some(chemin);
        }
    }
    [r"C:\Program Files\Google\Chrome\Application\chrome.exe", r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"]
        .into_iter()
        .find(|c| Path::new(c).is_file())
        .map(str::to_string)
}

/// L'interpréteur Python qui lance les outils et leurs bancs (l'ancien sys.executable). On
/// demande son vrai chemin au lanceur trouvé : arrêter « py » n'arrêterait pas le Python
/// qu'il a lancé.
pub fn python() -> &'static str {
    static PYTHON: OnceLock<String> = OnceLock::new();
    PYTHON.get_or_init(|| {
        let candidats: &[&str] = if cfg!(windows) { &["py", "python"] } else { &["python3", "python"] };
        for c in candidats {
            let sortie = Command::new(c)
                .args(["-c", "import sys; print(sys.executable)"])
                .env("PYTHONIOENCODING", "utf-8")
                .stdin(Stdio::null())
                .stderr(Stdio::null())
                .output();
            if let Ok(s) = sortie {
                let chemin = String::from_utf8_lossy(&s.stdout).trim().to_string();
                if s.status.success() && !chemin.is_empty() {
                    return chemin;
                }
            }
        }
        candidats[candidats.len() - 1].to_string()
    })
}

/// Lance une commande dans `dossier`, sans entrée, la sortie et les erreurs dans le même tuyau.
pub fn lancer(cmd: &[String], dossier: &Path, env: &[(String, String)]) -> io::Result<(Child, PipeReader)> {
    let (lecture, ecriture) = io::pipe()?;
    let mut commande = Command::new(&cmd[0]);
    commande
        .args(&cmd[1..])
        .current_dir(dossier)
        .envs(env.iter().map(|(k, v)| (k, v)))
        .stdin(Stdio::null())
        .stdout(ecriture.try_clone()?)
        .stderr(ecriture);
    // `commande` (et avec elle notre bout d'écriture du tuyau) est lâchée au retour :
    // la lecture finit quand le processus ferme sa sortie.
    Ok((commande.spawn()?, lecture))
}

fn git(dossier: &Path, args: &[&str]) -> io::Result<(bool, String)> {
    let s = Command::new("git").arg("-C").arg(dossier).args(args).stdin(Stdio::null()).output()?;
    Ok((s.status.success(), String::from_utf8_lossy(&s.stdout).trim().to_string()))
}

/// « branche · commit · date · message » du dépôt de l'outil : le rapport dit quel code a été essayé.
// shortcut: pas de délai de 10 s sur git comme en Python ; ces trois commandes sont locales.
pub fn version_git(dossier: &Path) -> String {
    let lire = || -> io::Result<Option<String>> {
        let (ok, sortie) = git(dossier, &["log", "-1", "--format=%h · %ci · %s"])?;
        let (_, branche) = git(dossier, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        let (_, modifie) = git(dossier, &["status", "--porcelain"])?;
        let suite = if modifie.is_empty() { "" } else { " · modifications locales" };
        Ok(ok.then(|| format!("{} · {}{}", branche, sortie, suite)))
    };
    lire().ok().flatten().unwrap_or_else(|| "version inconnue (pas de git)".into())
}

#[derive(Default)]
struct Journal {
    lignes: Vec<String>, // lignes de sortie de l'outil
    debut: usize,        // numéro de la première ligne gardée
}

#[derive(Default)]
struct Marche {
    port: Option<u16>,
    processus: Option<Child>,
    temp: Option<PathBuf>,
}

pub struct Outil {
    pub conf: &'static Conf,
    pub dossier: PathBuf,
    pub version: String,
    marche: Mutex<Marche>,
    journal: Arc<Mutex<Journal>>,
    verrou_demarrage: Mutex<()>,
}

impl Outil {
    pub fn new(conf: &'static Conf, dossier: PathBuf) -> Outil {
        Outil {
            conf,
            version: version_git(&dossier),
            dossier,
            marche: Mutex::default(),
            journal: Arc::default(),
            verrou_demarrage: Mutex::new(()),
        }
    }

    pub fn nom(&self) -> &'static str {
        self.conf.nom
    }

    pub fn port(&self) -> Option<u16> {
        self.marche.lock().unwrap().port
    }

    /// La LIB que WEB_CAO prend par défaut (même ordre que lui et ses bancs) : WEB_CAO_LIB,
    /// LIB/ à côté de l'outil, sinon ../PROJETS/LIB_CAO de web_suite. Seulement une vraie LIB,
    /// avec son catalogue ; les bancs la cherchent aussi par le chemin réel de l'outil.
    pub fn lib_reelle(&self) -> Option<PathBuf> {
        let reel = fs::canonicalize(&self.dossier).unwrap_or_else(|_| self.dossier.clone());
        let mut parents: Vec<PathBuf> =
            [&self.dossier, &reel].iter().filter_map(|d| d.parent()).map(Path::to_path_buf).collect();
        parents.sort();
        parents.dedup();
        let mut candidats: Vec<PathBuf> =
            std::env::var_os("WEB_CAO_LIB").filter(|c| !c.is_empty()).map(PathBuf::from).into_iter().collect();
        candidats.push(self.dossier.join("LIB"));
        candidats.extend(parents.iter().map(|p| p.join("PROJETS").join("LIB_CAO")));
        candidats.into_iter().find(|c| c.join("LIB_composants.csv").is_file())
    }

    pub fn demarrer(&self) -> Result<(), String> {
        let _un_seul = self.verrou_demarrage.lock().unwrap();
        if self.en_marche() {
            return Ok(());
        }
        self.demarrer_neuf()
    }

    fn demarrer_neuf(&self) -> Result<(), String> {
        let es = |e: io::Error| e.to_string();
        let temp = dossier_temporaire(&format!("web_bench_{}_", self.conf.id)).map_err(es)?;
        self.marche.lock().unwrap().temp = Some(temp.clone());
        let projets = temp.join("projets");
        let lib = temp.join("LIB");
        fs::create_dir_all(&projets).map_err(es)?;
        if self.conf.lib {
            // une copie de la LIB que l'outil prendrait : le banc peut y écrire sans risque
            match self.lib_reelle() {
                Some(source) => copier_arbre(&source, &lib),
                None => lib_minimale(&lib),
            }
            .map_err(es)?;
        }
        let port = port_libre().map_err(es)?;
        let mut cmd = web_tools_commun::demarreur(&self.dossier, self.conf.nom, self.conf.script)
            .unwrap_or_else(|| vec![python().to_string(), self.conf.script.to_string()]);
        cmd.extend(["--port".into(), port.to_string()]);
        cmd.extend(self.conf.args.iter().map(|a| remplir(a, &projets, &lib)));
        let env = [("WEB_TOOLS_HTTP", "1"), ("PYTHONUNBUFFERED", "1"), ("PYTHONIOENCODING", "utf-8")]
            .map(|(k, v)| (k.to_string(), v.to_string()));
        let (processus, sortie) = lancer(&cmd, &self.dossier, &env).map_err(es)?;
        {
            let mut m = self.marche.lock().unwrap();
            m.port = Some(port);
            m.processus = Some(processus);
        }
        let journal = self.journal.clone();
        std::thread::spawn(move || lire(sortie, &journal));
        for _ in 0..300 {
            if !self.en_marche() {
                let j = self.journal.lock().unwrap();
                let fin = &j.lignes[j.lignes.len().saturating_sub(30)..];
                return Err(format!("{} s'est arrêté au démarrage :\n{}", self.nom(), fin.join("\n")));
            }
            // comme urlopen : les redirections sont suivies, une réponse 4xx ou 5xx ne compte pas
            if ureq::get(&format!("http://127.0.0.1:{}/", port)).timeout(Duration::from_secs(1)).call().is_ok() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        Err(format!("{} ne répond pas sur le port {}", self.nom(), port))
    }

    pub fn noter(&self, ligne: String) {
        self.journal.lock().unwrap().lignes.push(ligne);
    }

    /// Les lignes du journal à partir du numéro `n`, et le numéro de la prochaine.
    pub fn lignes_depuis(&self, n: i64) -> (Vec<String>, usize) {
        let j = self.journal.lock().unwrap();
        let fin = j.debut + j.lignes.len();
        let n = (n.max(0) as usize).max(j.debut).min(fin);
        (j.lignes[n - j.debut..].to_vec(), fin)
    }

    pub fn en_marche(&self) -> bool {
        matches!(self.marche.lock().unwrap().processus.as_mut().map(Child::try_wait), Some(Ok(None)))
    }

    pub fn redemarrer(&self) -> Result<(), String> {
        self.arreter();
        *self.journal.lock().unwrap() = Journal::default();
        self.demarrer()
    }

    pub fn arreter(&self) {
        let (processus, temp) = {
            let mut m = self.marche.lock().unwrap();
            (m.processus.take(), m.temp.take())
        };
        if let Some(mut p) = processus {
            if matches!(p.try_wait(), Ok(None)) {
                terminer(&mut p);
                for _ in 0..100 {
                    if !matches!(p.try_wait(), Ok(None)) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                let _ = p.kill();
                let _ = p.wait();
            }
            // l'état « arrêté » reste lisible (en_marche), comme l'ancien poll()
            self.marche.lock().unwrap().processus = Some(p);
        }
        if let Some(t) = temp {
            let _ = fs::remove_dir_all(t);
        }
    }
}

/// Demande l'arrêt (SIGTERM) : l'outil ferme ses propres processus. Windows n'a que l'arrêt net.
fn terminer(p: &mut Child) {
    #[cfg(unix)]
    // SAFETY: kill(2) sur le PID d'un enfant pas encore attendu : le PID est à nous.
    unsafe {
        libc::kill(p.id() as libc::pid_t, libc::SIGTERM);
    }
    #[cfg(not(unix))]
    let _ = p.kill();
}

fn remplir(arg: &str, projets: &Path, lib: &Path) -> String {
    arg.replace("{projets}", &projets.to_string_lossy()).replace("{lib}", &lib.to_string_lossy())
}

fn lire(sortie: PipeReader, journal: &Mutex<Journal>) {
    let mut lecteur = BufReader::new(sortie);
    let mut brut = Vec::new();
    while matches!(lecteur.read_until(b'\n', &mut brut), Ok(n) if n > 0) {
        let ligne = String::from_utf8_lossy(&brut).trim_end_matches(['\r', '\n']).to_string();
        brut.clear();
        let mut j = journal.lock().unwrap();
        j.lignes.push(ligne);
        if j.lignes.len() > MAX_JOURNAL {
            let trop = j.lignes.len() - MAX_JOURNAL;
            j.lignes.drain(..trop);
            j.debut += trop;
        }
    }
}

/// Les outils installés à la racine de web_suite (un dossier par outil, comme installer.py),
/// dans l'ordre du catalogue.
pub fn trouver_outils(racine: &Path) -> Vec<Arc<Outil>> {
    CATALOGUE
        .iter()
        .filter(|conf| {
            let d = racine.join(conf.nom);
            d.join(conf.script).is_file() || web_tools_commun::demarreur(&d, conf.nom, conf.script).is_some()
        })
        .map(|conf| {
            let dossier = racine.join(conf.nom);
            Arc::new(Outil::new(conf, std::path::absolute(&dossier).unwrap_or(dossier)))
        })
        .collect()
}

/// Secondes depuis 1970 (time.time()).
pub fn horloge() -> f64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_et_motifs() {
        assert_eq!(remplir("{projets}", Path::new("/t/projets"), Path::new("/t/LIB")), "/t/projets");
        assert_eq!(remplir("{lib}", Path::new("/t/projets"), Path::new("/t/LIB")), "/t/LIB");
        assert_eq!(remplir("--local", Path::new("/t/projets"), Path::new("/t/LIB")), "--local");
        for nom in [".git", "CAPA GRM", "INDUC LQW", "modeles.zip"] {
            assert!(lib_ignoree(nom), "{}", nom);
        }
        for nom in ["LIB_composants.csv", "CAPA", "Models", ".gitignore"] {
            assert!(!lib_ignoree(nom), "{}", nom);
        }
        let routes = CATALOGUE[1].routes.unwrap();
        assert!(routes("/api/openems/lancer") && routes("/api/openems/sim/relancer"));
        assert!(!routes("/api/openems/etat") && !routes("/x/api/openems/lancer"));
    }

    #[test]
    fn journal_borne() {
        let o = Outil::new(&CATALOGUE[0], PathBuf::from("/nulle-part"));
        assert_eq!(o.version, "version inconnue (pas de git)");
        o.noter("a".into());
        o.noter("b".into());
        assert_eq!(o.lignes_depuis(1), (vec!["b".to_string()], 2));
        assert_eq!(o.lignes_depuis(9), (vec![], 2));
        assert_eq!(o.lignes_depuis(-3).0.len(), 2);
        assert!(!o.en_marche());
    }
}
