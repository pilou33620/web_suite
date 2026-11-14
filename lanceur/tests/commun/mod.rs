//! Aide commune aux bancs du lanceur (tests d'intégration) : les exécutables sont pilotés en
//! boîte noire, dans un dossier temporaire.
//!
//!     let poste = Poste::new("serveur");                // dossier temporaire, faux GitHub
//!     let l = poste.lanceur(&["--reseau"], &[("WEB_TOOLS_BANC_DISTANT", "1")]);
//!     let r = l.get("/api/etat");
//!
//! Rien ne part sur le réseau : un dépôt nu local joue GitHub (WEB_SUITE_BANC_PROJETS),
//! le jeton et la CA vont dans le dossier temporaire (WEB_TOOLS_JETON_FICHIER), git
//! ne voit aucune identité du poste.
#![allow(dead_code)]
pub mod http;

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

pub use http::Rep;

pub const WEB_SUITE: &str = env!("CARGO_BIN_EXE_web_suite");
pub const INSTALLER: &str = env!("CARGO_BIN_EXE_installer");
pub const MCP: &str = env!("CARGO_BIN_EXE_mcp_web_suite");
pub const AIDE_PILOTE: &str = env!("CARGO_BIN_EXE_aide_pilote");

/// La racine du dépôt web_suite.
pub fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

/// Le dépôt d'un outil : dans web_suite sur le poste, à côté de lui en CI (cloné dans ../).
pub fn depot(outil: &str) -> Option<PathBuf> {
    let racine = racine();
    let bases = [Some(racine.as_path()), racine.parent()];
    let trouve = bases.into_iter().flatten().map(|base| base.join(outil)).find(|d| d.join(".git").is_dir());
    trouve
}

static VERIFICATIONS: AtomicUsize = AtomicUsize::new(0);

/// Une vérification nommée ; elles sont comptées (voir `verifications`).
#[track_caller]
pub fn verifier<T: PartialEq + Debug>(nom: &str, obtenu: T, attendu: T) {
    VERIFICATIONS.fetch_add(1, Ordering::Relaxed);
    assert!(obtenu == attendu, "{nom} : attendu {attendu:?}, obtenu {obtenu:?}");
}

pub fn verifications() -> usize {
    VERIFICATIONS.load(Ordering::Relaxed)
}

pub fn chaine(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// Un dossier temporaire, effacé à la fin.
pub struct Temp(pub PathBuf);

impl Temp {
    pub fn new(nom: &str) -> Temp {
        let d = std::env::temp_dir().join(format!("banc-{}-{}", nom, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        Temp(d)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Lance la commande, lui écrit `entree` puis ferme stdin : (code de sortie, stdout, stderr).
/// `fusion` : stderr va dans stdout. Le processus est tué au bout de `delai`.
pub fn executer(mut cmd: Command, entree: &[u8], fusion: bool, delai: u64) -> (i32, Vec<u8>, Vec<u8>) {
    let lire = |mut f: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut tout = Vec::new();
            let _ = f.read_to_end(&mut tout);
            tout
        })
    };
    cmd.stdin(Stdio::piped());
    let (mut enfant, sortie, erreurs);
    if fusion {
        let (lecture, ecriture) = io::pipe().unwrap();
        cmd.stdout(ecriture.try_clone().unwrap()).stderr(ecriture);
        enfant = cmd.spawn().unwrap();
        drop(cmd); // nos copies du bout d'écriture : sans cela, jamais de fin de flux
        (sortie, erreurs) = (lire(Box::new(lecture)), None);
    } else {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        enfant = cmd.spawn().unwrap();
        sortie = lire(Box::new(enfant.stdout.take().unwrap()));
        erreurs = Some(lire(Box::new(enfant.stderr.take().unwrap())));
    }
    let mut stdin = enfant.stdin.take().unwrap();
    let entree = entree.to_vec();
    std::thread::spawn(move || {
        let _ = stdin.write_all(&entree);
    });
    let fin = Instant::now() + Duration::from_secs(delai);
    let code = loop {
        if let Some(etat) = enfant.try_wait().unwrap() {
            break etat.code().unwrap_or(-1);
        }
        if Instant::now() > fin {
            let _ = enfant.kill();
            panic!("le processus ne finit pas ({delai} s)");
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    (code, sortie.join().unwrap(), erreurs.map(|e| e.join().unwrap()).unwrap_or_default())
}

// ---------------------------------------------------------------------------
// Le poste d'essai et son lanceur
// ---------------------------------------------------------------------------
pub struct Poste {
    _tmp: Temp,
    pub tmp: PathBuf,
    pub racine: PathBuf,
    pub fichier_jeton: PathBuf,
    env: Vec<(String, String)>,
}

impl Poste {
    pub fn new(nom: &str) -> Poste {
        let temp = Temp::new(nom);
        let tmp = temp.0.clone();
        let racine = tmp.join("racine");
        fs::create_dir_all(&racine).unwrap();
        let vide = tmp.join("gitconfig-vide");
        fs::write(&vide, "").unwrap();
        let nu = tmp.join("github.git");
        let fichier_jeton = tmp.join("partage").join("jeton.txt");
        let env = vec![
            ("GIT_CONFIG_GLOBAL".to_string(), chaine(&vide)),
            ("GIT_CONFIG_NOSYSTEM".to_string(), "1".to_string()),
            ("WEB_SUITE_BANC_PROJETS".to_string(), chaine(&nu)),
            ("WEB_TOOLS_JETON_FICHIER".to_string(), chaine(&fichier_jeton)),
        ];
        let poste = Poste { _tmp: temp, tmp, racine, fichier_jeton, env };
        let _ = poste.commande("git").args(["init", "--quiet", "--bare"]).arg(&nu)
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
        poste
    }

    /// Une commande dans l'environnement du poste, `env` en plus.
    pub fn commande_env(&self, exe: &str, env: &[(&str, &str)]) -> Command {
        let mut c = Command::new(exe);
        for cle in ["WEB_TOOLS_BANC_DISTANT", "WEB_TOOLS_HTTP", "WEB_SUITE_RELANCEUR", "WEB_SUITE_DEJA_MAJ"] {
            c.env_remove(cle);
        }
        c.envs(self.env.iter().map(|(k, v)| (k.as_str(), v.as_str()))).envs(env.iter().copied());
        c
    }

    pub fn commande(&self, exe: &str) -> Command {
        self.commande_env(exe, &[])
    }

    pub fn jeton(&self) -> String {
        fs::read_to_string(&self.fichier_jeton).unwrap().trim().to_string()
    }

    /// Le lanceur, sans mise à jour de WEB_SUITE au démarrage.
    pub fn lanceur(&self, options: &[&str], env: &[(&str, &str)]) -> Lanceur {
        Lanceur::new(self, options, env, false)
    }

    /// Le lanceur, qui cherche sa mise à jour au démarrage.
    pub fn lanceur_maj(&self, options: &[&str], env: &[(&str, &str)]) -> Lanceur {
        Lanceur::new(self, options, env, true)
    }

    /// Un exécutable court (installer…) : -> (code de sortie, sortie et erreurs mêlées).
    pub fn lancer(&self, exe: &str, args: &[&str], env: &[(&str, &str)]) -> (i32, String) {
        let mut c = self.commande_env(exe, env);
        c.args(args);
        let (code, sortie, _) = executer(c, b"", true, 120);
        (code, String::from_utf8_lossy(&sortie).into_owned())
    }
}

pub struct Lanceur {
    proc: Child,
    lignes: Arc<Mutex<String>>,
    pub port: u16,
}

impl Lanceur {
    fn new(poste: &Poste, options: &[&str], env: &[(&str, &str)], maj: bool) -> Lanceur {
        let (lecture, ecriture) = io::pipe().unwrap();
        let mut c = poste.commande_env(WEB_SUITE, env);
        c.args(["--port", "0", "--racine"]).arg(&poste.racine);
        if !maj {
            c.arg("--sans-maj");
        }
        c.arg("--sans-navigateur").args(options);
        c.stdin(Stdio::null()).stdout(ecriture.try_clone().unwrap()).stderr(ecriture);
        let proc = c.spawn().unwrap();
        drop(c);
        let lignes: Arc<Mutex<String>> = Arc::default();
        let l2 = lignes.clone();
        std::thread::spawn(move || {
            let mut lecteur = BufReader::new(lecture);
            let mut ligne = Vec::new();
            while lecteur.read_until(b'\n', &mut ligne).unwrap_or(0) > 0 {
                l2.lock().unwrap().push_str(&String::from_utf8_lossy(&ligne));
                ligne.clear();
            }
        });
        let mut l = Lanceur { proc, lignes, port: 0 };
        // la dernière ligne du démarrage : le serveur écoute déjà
        l.attendre("Navigateur non ouvert");
        if options.contains(&"--reseau") {
            l.attendre("Code d'appairage");
        }
        let sortie = l.sortie();
        let re = regex::Regex::new(r"Adresse   http://127\.0\.0\.1:(\d+)/").unwrap();
        l.port = re.captures(&sortie).unwrap_or_else(|| panic!("adresse attendue dans :\n{sortie}"))[1].parse().unwrap();
        l
    }

    pub fn sortie(&self) -> String {
        self.lignes.lock().unwrap().clone()
    }

    /// Les codes d'appairage affichés dans le terminal du lanceur, sans l'espace.
    pub fn codes(&self) -> Vec<String> {
        let re = regex::Regex::new(r"Code d'appairage[^:\n]*:\s*(\d{3} \d{3})").unwrap();
        re.captures_iter(&self.sortie()).map(|c| c[1].replace(' ', "")).collect()
    }

    pub fn attendre(&mut self, texte: &str) {
        let fin = Instant::now() + Duration::from_secs(30);
        while !self.sortie().contains(texte) {
            assert!(self.proc.try_wait().unwrap().is_none(), "le lanceur s'est arrêté :\n{}", self.sortie());
            assert!(Instant::now() < fin, "« {} » attendu dans :\n{}", texte, self.sortie());
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn req(&self, methode: &str, chemin: &str, entetes: &[(&str, &str)], corps: &[u8]) -> Rep {
        http::requete(self.port, methode, chemin, entetes, corps).unwrap()
    }

    pub fn get(&self, chemin: &str) -> Rep {
        self.req("GET", chemin, &[], b"")
    }

    /// SIGTERM (kill, systemctl stop) : -> code de sortie.
    #[cfg(unix)]
    pub fn arreter(&mut self) -> i32 {
        unsafe { libc::kill(self.proc.id() as libc::pid_t, libc::SIGTERM) };
        let fin = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(etat) = self.proc.try_wait().unwrap() {
                // le temps que le fil de lecture vide le tuyau
                std::thread::sleep(Duration::from_millis(100));
                return etat.code().unwrap_or(-1);
            }
            assert!(Instant::now() < fin, "le lanceur ne s'arrête pas :\n{}", self.sortie());
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Lanceur {
    fn drop(&mut self) {
        if self.proc.try_wait().ok().flatten().is_none() {
            let _ = self.proc.kill();
            let _ = self.proc.wait();
        }
    }
}

// ---------------------------------------------------------------------------
// Dialogue JSON-RPC sur stdio avec le serveur MCP
// ---------------------------------------------------------------------------
/// Un message JSON-RPC (sans `id` : une notification).
pub fn msg(id: Option<i64>, methode: &str, params: Option<Value>) -> String {
    let mut m = json!({"jsonrpc": "2.0", "method": methode});
    if let Some(id) = id {
        m["id"] = json!(id);
    }
    if let Some(p) = params {
        m["params"] = p;
    }
    m.to_string()
}

/// Envoie les lignes au serveur MCP puis ferme stdin : (réponses décodées, code de sortie, stderr).
/// Le skill et les règles relus sont ceux de ce dépôt (WEB_SUITE_DEPOT) ; `ports` : WEB_SUITE_MCP_PORTS.
pub fn echanger(lignes: &[String], ports: Option<&str>) -> (Vec<Value>, i32, String) {
    let mut c = Command::new(MCP);
    c.env("WEB_SUITE_DEPOT", racine());
    if let Some(p) = ports {
        c.env("WEB_SUITE_MCP_PORTS", p);
    }
    let entree: String = lignes.iter().map(|l| format!("{l}\n")).collect();
    let (code, sortie, erreurs) = executer(c, entree.as_bytes(), false, 30);
    let sortie = String::from_utf8(sortie).expect("stdout en UTF-8");
    // stdout ne porte que du JSON-RPC : chaque ligne doit se décoder
    let reponses = sortie.split('\n').filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("ligne non JSON sur stdout ({e}) : {l}"))).collect();
    verifier("pas de \\r sur stdout", sortie.contains('\r'), false);
    (reponses, code, String::from_utf8_lossy(&erreurs).into_owned())
}

// ---------------------------------------------------------------------------
// Faux outil : un serveur HTTP local qui note ce qu'il reçoit
// ---------------------------------------------------------------------------
#[derive(Default)]
struct EtatFaux {
    recues: Vec<Value>,
    entetes: Vec<Vec<(String, String)>>,
    table: HashMap<(String, String), (u16, Value)>,
    file: VecDeque<(u16, Value)>,
}

/// Faux outil piloté par une table : (méthode, chemin sans requête) -> (statut, JSON) ; sans
/// entrée dans la table : 200 {"route": chemin}. Note [méthode, chemin complet, corps] de chaque
/// requête reçue. `fixe` : le même corps brut à toute requête, sans rien noter.
#[derive(Clone)]
pub struct Faux {
    pub port: u16,
    etat: Arc<Mutex<EtatFaux>>,
}

impl Faux {
    pub fn outil() -> Faux {
        Self::servir(None)
    }

    pub fn fixe(corps: &[u8]) -> Faux {
        Self::servir(Some(corps.to_vec()))
    }

    fn servir(fixe: Option<Vec<u8>>) -> Faux {
        let ecoute = TcpListener::bind("127.0.0.1:0").unwrap();
        let faux = Faux { port: ecoute.local_addr().unwrap().port(), etat: Arc::default() };
        let etat = faux.etat.clone();
        std::thread::spawn(move || {
            for flux in ecoute.incoming().flatten() {
                let (etat, fixe) = (etat.clone(), fixe.clone());
                std::thread::spawn(move || {
                    let mut lecteur = BufReader::new(flux);
                    let (mut premiere, mut entetes) = (String::new(), Vec::new());
                    lecteur.read_line(&mut premiere).ok();
                    loop {
                        let mut ligne = String::new();
                        if lecteur.read_line(&mut ligne).unwrap_or(0) == 0 || ligne.trim().is_empty() {
                            break;
                        }
                        if let Some((k, v)) = ligne.split_once(':') {
                            entetes.push((k.trim().to_string(), v.trim().to_string()));
                        }
                    }
                    let n = entetes.iter().find(|(k, _)| k.eq_ignore_ascii_case("Content-Length"))
                        .and_then(|(_, v)| v.parse().ok()).unwrap_or(0usize);
                    let mut corps = vec![0u8; n];
                    lecteur.read_exact(&mut corps).ok();
                    let mut mots = premiere.split_whitespace();
                    let (methode, chemin) = (mots.next().unwrap_or("").to_string(), mots.next().unwrap_or("").to_string());
                    let (statut, rendu) = match fixe {
                        Some(f) => (200, f),
                        None => {
                            let mut e = etat.lock().unwrap();
                            let recu = if n > 0 { serde_json::from_slice(&corps).unwrap_or(Value::Null) } else { Value::Null };
                            e.recues.push(json!([methode, chemin, recu]));
                            e.entetes.push(entetes);
                            let cle = (methode, chemin.split('?').next().unwrap().to_string());
                            let (statut, rep) = if cle.0 == "POST" && cle.1 == "/api/pilote/commande" && !e.file.is_empty() {
                                e.file.pop_front().unwrap()
                            } else {
                                e.table.get(&cle).cloned().unwrap_or_else(|| (200, json!({"route": chemin})))
                            };
                            (statut, rep.to_string().into_bytes())
                        }
                    };
                    let mut flux = lecteur.into_inner();
                    let _ = write!(flux, "HTTP/1.0 {statut} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n", rendu.len());
                    let _ = flux.write_all(&rendu);
                });
            }
        });
        faux
    }

    /// La réponse à (méthode, chemin sans requête).
    pub fn poser(&self, methode: &str, chemin: &str, reponse: (u16, Value)) {
        self.etat.lock().unwrap().table.insert((methode.to_string(), chemin.to_string()), reponse);
    }

    pub fn vider_table(&self) {
        self.etat.lock().unwrap().table.clear();
    }

    /// Les réponses aux prochains POST /api/pilote/commande, dans l'ordre, avant la table.
    pub fn file(&self, reponses: Vec<(u16, Value)>) {
        self.etat.lock().unwrap().file = reponses.into();
    }

    /// Les requêtes reçues : [[méthode, chemin complet, corps JSON ou null], …].
    pub fn recues(&self) -> Vec<Value> {
        self.etat.lock().unwrap().recues.clone()
    }

    pub fn oublier(&self) {
        self.etat.lock().unwrap().recues.clear();
    }

    /// Un en-tête de la dernière requête reçue, sans égard à la casse.
    pub fn entete(&self, nom: &str) -> Option<String> {
        let e = self.etat.lock().unwrap();
        e.entetes.last()?.iter().find(|(k, _)| k.eq_ignore_ascii_case(nom)).map(|(_, v)| v.clone())
    }
}

/// Un port où rien n'écoute.
pub fn port_ferme() -> u16 {
    TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}
