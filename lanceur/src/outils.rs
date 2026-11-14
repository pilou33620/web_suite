//! Les outils WEB·SUITE : comment les installer, les mettre à jour et les lancer.
//!
//! Partagé par serveur_lanceur.rs (le lanceur web) et installer.rs (le menu en
//! console). Le catalogue et les emplacements sont dans catalogue.rs.
//!
//! Chaque outil est un exécutable Rust : le lanceur le lance par son
//! demarrer_<NOM>.sh/.cmd, qui le compile ou le télécharge au besoin.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Seek, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub use crate::catalogue::{dossier, est_installe, par_id, Outil, CATALOGUE};
use crate::processus::{quel, sans_fenetre, sortie as sortie_commande, Sortie, WINDOWS};
use crate::projets;

const DELAI_DEMARRAGE: f64 = 90.0; // s : la vérification GitHub des outils peut prendre du temps
// Envoi automatique de ce qui a été enregistré hors ligne : un essai au
// démarrage, puis toutes les minutes tant que quelque chose attend, en
// espaçant jusqu'à 5 min si GitHub reste injoignable (hors ligne, un essai échoue
// en une fraction de seconde : il ne coûte rien).
const ENVOI_AUTO_MIN: u64 = 60;
const ENVOI_AUTO_MAX: u64 = 300;
pub const JOURNAL_MAX: u64 = 1024 * 1024; // o : au-delà, le journal passe en .log.1 au lancement suivant

// Dernière version stable d'openEMS pour Windows : l'archive binaire officielle.
const OPENEMS_URL: &str = "https://github.com/thliebig/openEMS-Project/releases/download/v0.0.36/openEMS_v0.0.36.zip";
// L'empreinte de cette archive, relevée le 02/10/2026 (GitHub n'en publie pas
// pour cette version). Les binaires qu'elle contient sont exécutés : une archive
// tronquée ou différente est refusée avant d'être extraite. Changer OPENEMS_URL
// demande de changer l'empreinte avec elle.
const OPENEMS_SHA256: &str = "e0d62b1176c0897ad18876b45667de877d7d3b58b37c0be95545f9b988896059";
const DUREE_CODE: f64 = 600.0; // s : un code d'appairage vit 10 min (Appairage::DUREE)

fn re_url() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"https?://([\w.\-]+|\[[0-9a-fA-F:]+\]):(\d{2,5})").expect("motif"))
}

fn re_code() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new(); // appairage.rs, ligne du terminal
    RE.get_or_init(|| Regex::new(r"Code d'appairage[^:\n]*:\s*(\d{3} \d{3})").expect("motif"))
}

pub type Ecrire<'a> = &'a dyn Fn(&str);

fn maintenant() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

fn chaine(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

pub fn git_disponible() -> bool {
    quel("git").is_some()
}

// ---------------------------------------------------------------------------
// Mise à jour (git pull), pour WEB_SUITE comme pour les outils
// ---------------------------------------------------------------------------
// Statuts après lesquels l'outil n'a pas à refaire sa propre vérification.
const MAJ_FAITE: &[&str] = &["a_jour", "maj", "hors_ligne", "sans_git"];
const FRAICHEUR_MAJ: f64 = 600.0; // s : au-delà, l'outil revérifie lui-même à son lancement

/// Bilan affiché en console quand maj_depot n'a rien écrit (copie zip, sans .git).
pub fn bilan_sans_texte(statut: &str) -> &str {
    if statut == "sans_git" { "copie sans git : pas de mise à jour automatique." } else { statut }
}

fn git(dossier: &Path, args: &[&str], delai: f64) -> io::Result<Sortie> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(dossier).env("GIT_TERMINAL_PROMPT", "0");
    sortie_commande(sans_fenetre(&mut cmd), delai, None)
}

/// git pull --ff-only si la branche distante est en avance et rien n'est modifié.
///
/// Renvoie 'maj', 'a_jour', 'hors_ligne', 'sans_git', 'modifie' (fichiers
/// modifiés : rien n'est touché) ou 'echec'.
pub fn maj_depot(dossier: &Path, ecrire: Ecrire) -> &'static str {
    if !dossier.join(".git").is_dir() {
        return "sans_git"; // copie zip : rien à comparer
    }
    if !git_disponible() {
        ecrire("git introuvable : mises à jour non vérifiées.");
        return "sans_git";
    }
    let essai = || -> io::Result<&'static str> {
        if git(dossier, &["fetch", "--quiet", "origin"], 10.0)?.code != 0 {
            ecrire("GitHub injoignable : mise à jour ignorée.");
            return Ok("hors_ligne");
        }
        if git(dossier, &["rev-parse", "--verify", "@{u}"], 10.0)?.code != 0 {
            ecrire("Aucune branche distante suivie.");
            return Ok("a_jour");
        }
        let retard = git(dossier, &["rev-list", "HEAD..@{u}", "--count"], 10.0)?.stdout.trim().to_string();
        if retard.parse::<u64>().map_or(true, |n| n == 0) {
            ecrire("À jour.");
            return Ok("a_jour");
        }
        if !git(dossier, &["status", "--porcelain", "-uno"], 10.0)?.stdout.trim().is_empty() {
            ecrire(&format!("{} mise(s) à jour disponible(s), mais des fichiers sont modifiés.", retard));
            return Ok("modifie");
        }
        ecrire(&format!("{} mise(s) à jour, téléchargement…", retard));
        if git(dossier, &["pull", "--ff-only", "--quiet"], 60.0)?.code != 0 {
            ecrire("git pull a échoué : on garde la version actuelle.");
            return Ok("echec");
        }
        ecrire(&format!("Mis à jour ({} commit(s)).", retard));
        Ok("maj")
    };
    essai().unwrap_or_else(|_| {
        ecrire("Délai dépassé : mise à jour ignorée.");
        "hors_ligne"
    })
}

// ---------------------------------------------------------------------------
// Installation
// ---------------------------------------------------------------------------
/// urllib.request.urlopen(url, timeout=60) : le proxy de l'environnement est suivi.
fn client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_connect(Duration::from_secs(60))
        .timeout_read(Duration::from_secs(60))
        .build()
}

fn non_vide(d: &Path) -> bool {
    fs::read_dir(d).map_or(d.exists(), |mut l| l.next().is_some())
}

/// Clone l'outil dans un sous-dossier de la racine. Renvoie true si l'outil est prêt.
///
/// git d'abord : c'est lui qui permet aux outils de se mettre à jour seuls
/// au démarrage. Sans git, on se rabat sur l'archive zip de la branche main.
pub fn installer(outil: &Outil, racine: &Path, ecrire: Ecrire) -> bool {
    let cible = dossier(outil, racine);
    if est_installe(outil, racine) {
        ecrire(&format!("{} est déjà installé dans {}", outil.nom, cible.display()));
        return true;
    }
    if non_vide(&cible) {
        ecrire(&format!("[X] {} existe mais ne contient pas demarrer_{} : dossier laissé intact.", cible.display(), outil.nom));
        return false;
    }
    let _ = fs::create_dir_all(racine);

    if git_disponible() {
        ecrire(&format!("git clone {}", outil.depot));
        let code = executer(&["git", "clone", "--progress", outil.depot, &chaine(&cible)], ecrire, None);
        if code == 0 && est_installe(outil, racine) {
            ecrire(&format!("[OK] {} installé.", outil.nom));
            return true;
        }
        ecrire(&format!("[X] git clone a échoué (code {}).", code));
        return false;
    }

    let url = format!("{}/archive/refs/heads/main.zip", &outil.depot[..outil.depot.len() - 4]);
    ecrire(&format!("git introuvable : téléchargement de l'archive {}", url));
    ecrire("(sans git, l'outil ne pourra pas se mettre à jour tout seul)");
    let telecharger = || -> Result<(), Box<dyn std::error::Error>> {
        let mut donnees = Vec::new();
        client().get(&url).call()?.into_reader().read_to_end(&mut donnees)?;
        let mut z = zip::ZipArchive::new(io::Cursor::new(donnees))?;
        let premier = z.name_for_index(0).unwrap_or("").split('/').next().unwrap_or("").to_string();
        extraire(&mut z, &format!("{}/", premier), &cible)?;
        Ok(())
    };
    if let Err(exc) = telecharger() {
        ecrire(&format!("[X] Téléchargement impossible : {}", exc));
        return false;
    }
    let ok = est_installe(outil, racine);
    ecrire(&if ok { format!("[OK] {} installé.", outil.nom) } else { "[X] Archive incomplète.".to_string() });
    ok
}

/// Lance une commande et recopie sa sortie ligne à ligne. Renvoie le code de sortie.
fn executer(cmd: &[&str], ecrire: Ecrire, cwd: Option<&Path>) -> i32 {
    let lancer = || -> io::Result<(Child, io::PipeReader)> {
        let (lecteur, ecrivain) = io::pipe()?; // sortie et erreurs dans le même tuyau
        let mut c = Command::new(cmd[0]);
        c.args(&cmd[1..]).stdin(Stdio::null()).stdout(ecrivain.try_clone()?).stderr(ecrivain);
        if let Some(d) = cwd {
            c.current_dir(d);
        }
        Ok((sans_fenetre(&mut c).spawn()?, lecteur))
    };
    let (mut proc, mut lecteur) = match lancer() {
        Ok(r) => r,
        Err(exc) => {
            ecrire(&format!("[X] {} : {}", cmd[0], exc));
            return -1;
        }
    };
    // git réécrit sa ligne de progression avec \r : on découpe dessus aussi.
    let mut tampon: Vec<u8> = Vec::new();
    let dire = |ligne: &[u8]| {
        let texte = String::from_utf8_lossy(ligne);
        if !texte.trim().is_empty() {
            ecrire(texte.trim());
        }
    };
    let mut bloc = [0u8; 256];
    loop {
        match lecteur.read(&mut bloc) {
            Ok(0) | Err(_) => break,
            Ok(n) => tampon.extend_from_slice(&bloc[..n]),
        }
        while let Some(i) = tampon.iter().position(|c| *c == b'\r' || *c == b'\n') {
            dire(&tampon[..i]);
            tampon.drain(..=i);
        }
    }
    dire(&tampon);
    proc.wait().ok().and_then(|s| s.code()).unwrap_or(-1)
}

/// Extrait les entrées de l'archive qui commencent par `prefixe` dans `cible`.
fn extraire<R: Read + Seek>(z: &mut zip::ZipArchive<R>, prefixe: &str, cible: &Path) -> io::Result<()> {
    for i in 0..z.len() {
        let mut entree = z.by_index(i)?;
        let nom = entree.name().to_string();
        let Some(relatif) = nom.strip_prefix(prefixe).filter(|r| !r.is_empty()) else { continue };
        // Le chemin remis d'aplomb (normpath) doit rester sous `cible`.
        let mut morceaux: Vec<&str> = Vec::new();
        let mut dehors = relatif.starts_with('/');
        for m in relatif.split(|c| c == '/' || (WINDOWS && c == '\\')) {
            match m {
                "" | "." => {}
                ".." => dehors |= morceaux.pop().is_none(),
                _ => {
                    dehors |= WINDOWS && m.contains(':');
                    morceaux.push(m);
                }
            }
        }
        if dehors || morceaux.is_empty() {
            continue; // entrée hors du dossier : ignorée
        }
        let chemin = morceaux.iter().fold(cible.to_path_buf(), |p, m| p.join(m));
        if nom.ends_with('/') {
            fs::create_dir_all(&chemin)?;
        } else {
            if let Some(parent) = chemin.parent() {
                fs::create_dir_all(parent)?;
            }
            io::copy(&mut entree, &mut fs::File::create(&chemin)?)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// openEMS (WEB_ANTENNA)
// ---------------------------------------------------------------------------
// WEB_ANTENNA lance les deux exécutables d'openEMS, « openEMS » et « nf2ff ». Il les cherche
// dans WEB_ANTENNA/openEMS/ (l'archive binaire de Windows, avec ses DLL), sinon dans le PATH.
fn dossier_openems(outil: &Outil, racine: &Path) -> PathBuf {
    dossier(outil, racine).join("openEMS")
}

/// Les deux exécutables sont-ils dans WEB_ANTENNA/openEMS/ ?
///
/// None hors Windows : rien ne s'y installe tout seul (openEMS vient du système, par
/// le PATH), la page n'a donc pas à proposer son bouton.
pub fn openems_pret(outil: &Outil, racine: &Path) -> Option<bool> {
    if outil.dependances != Some("openems") || !WINDOWS {
        return None;
    }
    let oe = dossier_openems(outil, racine);
    Some(oe.join("openEMS.exe").is_file() && oe.join("nf2ff.exe").is_file())
}

/// Télécharge l'archive openEMS et l'extrait dans `cible`. Renvoie true si CSXCAD.dll y est.
fn telecharger_openems(agent: &ureq::Agent, url: &str, cible: &Path, ecrire: Ecrire) -> bool {
    ecrire(&format!("Téléchargement de {}", url));
    let tmp = PathBuf::from(format!("{}.zip.part", cible.display()));
    let essai = || -> Result<bool, Box<dyn std::error::Error>> {
        let rep = agent.get(url).call()?;
        let total: u64 = rep.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0);
        let mut flux = rep.into_reader();
        let mut f = fs::File::create(&tmp)?;
        let mut empreinte = Sha256::new();
        let (mut lu, mut palier) = (0u64, 0u64);
        let mut bloc = vec![0u8; 1 << 20];
        loop {
            let n = flux.read(&mut bloc)?;
            if n == 0 {
                break;
            }
            f.write_all(&bloc[..n])?;
            empreinte.update(&bloc[..n]);
            lu += n as u64;
            if total > 0 && lu * 100 / total >= palier + 10 {
                palier = lu * 100 / total / 10 * 10;
                ecrire(&format!("  {} %  ({:.0} / {:.0} Mo)", palier, lu as f64 / 1e6, total as f64 / 1e6));
            }
        }
        drop(f);
        let hexa: String = empreinte.finalize().iter().map(|b| format!("{:02x}", b)).collect();
        if hexa != OPENEMS_SHA256 {
            ecrire(
                "[X] L'archive téléchargée n'est pas celle attendue (empreinte SHA-256 \
                 différente) : rien n'est installé. Téléchargement tronqué ? Réessayez.",
            );
            return Ok(false);
        }
        let mut z = zip::ZipArchive::new(fs::File::open(&tmp)?)?;
        // L'archive range tout sous un dossier (openEMS/ aujourd'hui) : on
        // prend celui qui contient CSXCAD.dll, pour ne pas finir en openEMS/openEMS/.
        let dll = z
            .file_names()
            .filter(|n| n.to_lowercase().ends_with("/csxcad.dll") || n.to_lowercase() == "csxcad.dll")
            .min_by_key(|n| n.len())
            .map(str::to_string);
        let Some(dll) = dll else {
            ecrire("[X] L'archive ne contient pas CSXCAD.dll : format inattendu.");
            return Ok(false);
        };
        ecrire(&format!("Extraction dans {}", cible.display()));
        extraire(&mut z, &dll[..dll.len() - "CSXCAD.dll".len()], cible)?;
        Ok(true)
    };
    let fait = essai().unwrap_or_else(|exc| {
        ecrire(&format!("[X] Téléchargement d'openEMS impossible : {}", exc));
        false
    });
    let _ = fs::remove_file(&tmp);
    fait && cible.join("CSXCAD.dll").is_file()
}

/// Installe openEMS pour WEB_ANTENNA : l'archive binaire, extraite dans WEB_ANTENNA/openEMS/.
/// Renvoie true si openEMS.exe et nf2ff.exe y sont.
pub fn installer_openems(outil: &Outil, racine: &Path, ecrire: Ecrire) -> bool {
    if outil.dependances != Some("openems") {
        return true;
    }
    if !WINDOWS {
        ecrire("[!] Installation automatique d'openEMS : Windows seulement. Ailleurs, installez openEMS");
        ecrire("    de sorte que ses exécutables « openEMS » et « nf2ff » soient dans le PATH.");
        return false;
    }
    if !est_installe(outil, racine) {
        ecrire(&format!("[X] {} n'est pas installé.", outil.nom));
        return false;
    }
    let oe = dossier_openems(outil, racine);
    if openems_pret(outil, racine) == Some(true) {
        ecrire(&format!("[OK] openEMS déjà présent : {}", oe.display()));
        return true;
    }
    if oe.is_dir() && non_vide(&oe) {
        ecrire(&format!("[X] {} existe mais ne contient pas openEMS.exe et nf2ff.exe : dossier laissé intact.", oe.display()));
        ecrire("    Videz-le ou supprimez-le, puis relancez l'installation d'openEMS.");
        return false;
    }
    if !telecharger_openems(&client(), OPENEMS_URL, &oe, ecrire) {
        return false;
    }
    if openems_pret(outil, racine) != Some(true) {
        ecrire("[X] L'archive ne contient pas openEMS.exe et nf2ff.exe à côté de CSXCAD.dll : format inattendu.");
        return false;
    }
    ecrire(&format!("[OK] openEMS installé — les simulations de {} sont disponibles.", outil.nom));
    true
}

// ---------------------------------------------------------------------------
// Lancement
// ---------------------------------------------------------------------------
/// Le premier port libre à partir de `depart` (20 essais), 0 s'il n'y en a pas.
pub fn port_libre(depart: u16) -> u16 {
    // TcpListener::bind pose SO_REUSEADDR hors Windows : sinon un outil qu'on vient
    // d'arrêter (TIME_WAIT) prend le port suivant ; pas sous Windows, où cette option
    // laisserait deux serveurs partager un port
    (depart..depart.saturating_add(20)).find(|port| TcpListener::bind((Ipv4Addr::LOCALHOST, *port)).is_ok()).unwrap_or(0)
}

pub fn repond(port: u16) -> bool {
    TcpStream::connect_timeout(&SocketAddr::from((Ipv4Addr::LOCALHOST, port)), Duration::from_millis(300)).is_ok()
}

/// Objet Job Windows : les serveurs lancés meurent avec le lanceur.
///
/// Sans lui, fermer la fenêtre de WEB_SUITE laisserait tourner des serveurs
/// sans fenêtre, impossibles à arrêter autrement que par le gestionnaire des
/// tâches. Les petits-enfants (l'exécutable que lance demarrer_<NOM>.cmd)
/// héritent du job.
#[cfg(windows)]
struct Job(Option<windows_sys::Win32::Foundation::HANDLE>);

#[cfg(windows)]
// Le handle du job ne sert qu'à AssignProcessToJobObject, sûr depuis n'importe quel fil.
unsafe impl Send for Job {}
#[cfg(windows)]
unsafe impl Sync for Job {}

#[cfg(windows)]
impl Job {
    fn new() -> Job {
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let regle = !h.is_null()
                && SetInformationJobObject(
                    h,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                ) != 0;
            Job(regle.then_some(h))
        }
    }

    fn ajouter(&self, proc: &Child) {
        use std::os::windows::io::AsRawHandle;
        if let Some(h) = self.0 {
            unsafe {
                windows_sys::Win32::System::JobObjects::AssignProcessToJobObject(h, proc.as_raw_handle() as _);
            }
        }
    }
}

#[cfg(not(windows))]
struct Job;

#[cfg(not(windows))]
impl Job {
    fn new() -> Job {
        Job
    }
    fn ajouter(&self, _proc: &Child) {}
}

/// threading.Event.
struct Evenement {
    fait: Mutex<bool>,
    signal: Condvar,
}

impl Evenement {
    fn new() -> Self {
        Evenement { fait: Mutex::new(true), signal: Condvar::new() }
    }
    fn poser(&self, fait: bool) {
        *self.fait.lock().unwrap() = fait;
        self.signal.notify_all();
    }
    fn est_pose(&self) -> bool {
        *self.fait.lock().unwrap()
    }
    fn attendre(&self, delai: f64) -> bool {
        let fin = Instant::now() + Duration::from_secs_f64(delai.max(0.0));
        let mut fait = self.fait.lock().unwrap();
        while !*fait {
            let reste = fin.saturating_duration_since(Instant::now());
            if reste.is_zero() {
                return false;
            }
            fait = self.signal.wait_timeout(fait, reste).unwrap().0;
        }
        true
    }
}

#[derive(Default)]
pub struct Etat {
    pub phase: Option<&'static str>,
    pub message: String,
    pub url: Option<String>,
    pub port: Option<u16>,
    proc: Option<Child>,
    debut: f64,
    code: Option<(String, f64)>, // le dernier code d'appairage lu, et quand on l'a vu
}

impl Etat {
    fn en_route(&self) -> bool {
        matches!(self.phase, Some("demarrage") | Some("actif"))
    }
}

#[derive(Default, Clone)]
struct EnvoiAuto {
    attente: u64,
    message: String,
    ok: Option<bool>,
}

type Installation = fn(&Outil, &Path, Ecrire) -> bool;

/// État des outils (installation, serveur lancé) partagé par le lanceur web.
pub struct Gestionnaire {
    pub racine: PathBuf,
    // reseau : les outils écoutent sur le réseau local (sans --local) pour
    // qu'une tablette ou un autre poste puisse les ouvrir.
    pub reseau: bool,
    // L'adresse du lanceur sur ce poste et le jeton du mode réseau, fixés
    // par serveur_lanceur.rs une fois le serveur ouvert. Passés aux outils par
    // l'environnement (une option inconnue ferait échouer un outil plus
    // ancien) : WEB_CAO y relaie « Enregistrer + GitHub » depuis l'éditeur,
    // en exigeant le même jeton que le lanceur des autres appareils.
    pub lanceur: Mutex<Option<String>>,
    pub jeton: Mutex<Option<String>>,
    // Dernier essai d'envoi automatique, affiché sur la page : {"attente": n
    // commits pas encore sur GitHub, "message": ..., "ok": bool|None}.
    envoi_auto: Mutex<EnvoiAuto>,
    job: Job,
    etats: Mutex<HashMap<&'static str, Etat>>,
    verifs: Mutex<HashMap<&'static str, (&'static str, f64, String)>>, // id -> (statut maj_depot, heure, dernier message)
    maj_finie: HashMap<&'static str, Evenement>,
    journaux: PathBuf,
}

/// Le journal d'un outil, créé en 0600 : il contient son code d'appairage.
fn ouvrir_journal(chemin: &Path) -> io::Result<fs::File> {
    let mut o = fs::OpenOptions::new();
    o.append(true).create(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut o, 0o600);
    o.open(chemin)
}

fn ajouter_au_journal(chemin: &Path, texte: &str) {
    if let Ok(mut f) = ouvrir_journal(chemin) {
        let _ = f.write_all(texte.as_bytes());
    }
}

/// Le journal à partir du dernier « === lancement » (rien sans cette ligne).
fn depuis_lancement(texte: &str) -> &str {
    texte.rfind("=== lancement").map_or("", |i| &texte[i..])
}

/// Le dernier code d'appairage de l'outil, tant qu'il vaut : l'outil l'écrit dans son
/// journal, que personne ne voit. La page le montre aux appareils déjà appairés.
fn code_du_journal(journal: &Path, e: &mut Etat) -> Option<String> {
    if !e.en_route() {
        return None;
    }
    let mut f = fs::File::open(journal).ok()?;
    let taille = f.seek(io::SeekFrom::End(0)).ok()?;
    f.seek(io::SeekFrom::Start(taille.saturating_sub(32768))).ok()?;
    let mut octets = Vec::new();
    f.read_to_end(&mut octets).ok()?;
    let texte = String::from_utf8_lossy(&octets);
    let dernier = re_code().captures_iter(depuis_lancement(&texte)).last()?[1].to_string();
    // vu pour la première fois maintenant : l'heure de la ligne n'est pas écrite
    if e.code.as_ref().map(|c| &c.0) != Some(&dernier) {
        e.code = Some((dernier, maintenant()));
    }
    let (code, vu) = e.code.as_ref()?;
    (maintenant() - vu < DUREE_CODE).then(|| code.clone())
}

impl Gestionnaire {
    pub fn new(racine: &Path, reseau: bool) -> Arc<Self> {
        let journaux = crate::catalogue::ici().join("journaux");
        let _ = fs::create_dir_all(&journaux);
        Arc::new(Gestionnaire {
            racine: racine.to_path_buf(),
            reseau,
            lanceur: Mutex::new(None),
            jeton: Mutex::new(None),
            envoi_auto: Mutex::new(EnvoiAuto::default()),
            job: Job::new(),
            etats: Mutex::new(CATALOGUE.iter().map(|o| (o.id, Etat::default())).collect()),
            verifs: Mutex::new(HashMap::new()),
            maj_finie: CATALOGUE.iter().map(|o| (o.id, Evenement::new())).collect(),
            journaux,
        })
    }

    pub fn journal(&self, oid: &str) -> PathBuf {
        self.journaux.join(format!("{}.log", oid))
    }

    fn etats(&self) -> MutexGuard<'_, HashMap<&'static str, Etat>> {
        self.etats.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn noter(&self, oid: &str, texte: &str) {
        if let Some(e) = self.etats().get_mut(oid) {
            e.message = texte.to_string();
        }
        ajouter_au_journal(&self.journal(oid), &format!("{}\n", texte));
    }

    // -- état lisible par la page ------------------------------------------
    pub fn etat(&self) -> Value {
        let mut sortie = Vec::new();
        for o in CATALOGUE {
            let a_envoyer = projets::en_attente(&self.racine, o.id); // git status, hors verrou
            let mut etats = self.etats();
            let e = etats.get_mut(o.id).expect("outil du catalogue");
            self.actualiser(o, e);
            sortie.push(json!({
                "id": o.id, "nom": o.nom, "sous_titre": o.sous_titre,
                "couleur": o.couleur, "dependances": o.dependances,
                "installe": est_installe(o, &self.racine),
                "openems": openems_pret(o, &self.racine),
                "dossier": chaine(&dossier(o, &self.racine)),
                "phase": e.phase, "message": e.message,
                "url": e.url, "port": e.port, "a_envoyer": a_envoyer,
                "code_appairage": code_du_journal(&self.journal(o.id), e),
            }));
        }
        Value::Array(sortie)
    }

    pub fn envoi_auto(&self) -> Value {
        let e = self.envoi_auto.lock().unwrap().clone();
        json!({"attente": e.attente, "message": e.message, "ok": e.ok})
    }

    /// La phase d'un outil (None : au repos).
    pub fn phase(&self, oid: &str) -> Option<&'static str> {
        self.etats().get(oid).and_then(|e| e.phase)
    }

    /// Fait avancer 'demarrage' -> 'actif', et repère les serveurs tombés.
    fn actualiser(&self, o: &Outil, e: &mut Etat) {
        if e.en_route() {
            if let Some(Ok(Some(statut))) = e.proc.as_mut().map(|p| p.try_wait()) {
                #[cfg(unix)]
                let signal = std::os::unix::process::ExitStatusExt::signal(&statut).map(|s| -s);
                #[cfg(not(unix))]
                let signal = None;
                let code = statut.code().or(signal).unwrap_or(-1);
                e.phase = Some("erreur");
                e.url = None;
                e.proc = None;
                e.message = format!("Le serveur s'est arrêté (code {}) : voir le journal.", code);
                return;
            }
        }
        if e.phase != Some("demarrage") {
            return;
        }
        let url = self.url_du_journal(o.id).or_else(|| {
            e.port.filter(|port| repond(*port)).map(|port| format!("http://127.0.0.1:{}/", port))
        });
        if let Some(url) = url {
            e.phase = Some("actif");
            e.message = format!("En cours sur {}", url);
            e.url = Some(url);
        } else if maintenant() - e.debut > DELAI_DEMARRAGE {
            e.message = "Toujours pas de réponse : voir le journal.".to_string();
        }
    }

    fn url_du_journal(&self, oid: &str) -> Option<String> {
        let texte = String::from_utf8_lossy(&fs::read(self.journal(oid)).ok()?).into_owned();
        re_url()
            .captures_iter(depuis_lancement(&texte))
            .filter_map(|m| m[2].parse::<u16>().ok())
            .find(|port| repond(*port))
            // Le lanceur tourne sur la même machine : 127.0.0.1 répond
            // toujours. La page remplace l'hôte par celui qu'elle utilise.
            .map(|port| format!("http://127.0.0.1:{}/", port))
    }

    // -- actions -------------------------------------------------------------
    /// Cherche les mises à jour de chaque outil installé, en parallèle et en fond.
    ///
    /// Lancé au démarrage du lanceur, pendant l'animation : les outils lancés
    /// ensuite reçoivent --sans-maj et démarrent sans refaire la vérification.
    pub fn verifier_maj_outils(self: &Arc<Self>) {
        for o in CATALOGUE {
            if !est_installe(o, &self.racine) {
                continue;
            }
            if let Some(e) = self.etats().get_mut(o.id) {
                e.phase = Some("maj");
                e.message = "Recherche de mises à jour…".to_string();
            }
            self.maj_finie[o.id].poser(false);
            let g = self.clone();
            std::thread::spawn(move || {
                let dernier = Mutex::new(String::new());
                let statut = maj_depot(&dossier(o, &g.racine), &|texte| {
                    *dernier.lock().unwrap() = texte.to_string();
                    g.noter(o.id, texte);
                });
                g.verifs.lock().unwrap().insert(o.id, (statut, maintenant(), dernier.into_inner().unwrap()));
                if let Some(e) = g.etats().get_mut(o.id) {
                    if e.phase == Some("maj") {
                        e.phase = None;
                    }
                }
                g.maj_finie[o.id].poser(true);
            });
        }
    }

    /// La vérification du démarrage de l'outil est-elle finie ?
    pub fn maj_est_finie(&self, oid: &str) -> bool {
        self.maj_finie.get(oid).is_none_or(|e| e.est_pose())
    }

    /// Résultat de la vérification du démarrage, pour la console.
    ///
    /// None si elle tourne encore après `delai` secondes, "" si l'outil n'a
    /// pas été vérifié (non installé, --sans-maj).
    pub fn bilan_maj(&self, oid: &str, delai: f64) -> Option<String> {
        if !self.maj_finie[oid].attendre(delai) {
            return None;
        }
        Some(match self.verifs.lock().unwrap().get(oid) {
            None => String::new(),
            Some((statut, _, texte)) => if texte.is_empty() { bilan_sans_texte(statut) } else { texte }.to_string(),
        })
    }

    fn maj_deja_faite(&self, oid: &str) -> bool {
        self.verifs.lock().unwrap().get(oid).is_some_and(|(statut, quand, _)| {
            MAJ_FAITE.contains(statut) && maintenant() - quand < FRAICHEUR_MAJ
        })
    }

    pub fn installer(self: &Arc<Self>, oid: &str) -> bool {
        self.installer_en_fond(oid, installer, "Installé.")
    }

    pub fn installer_openems(self: &Arc<Self>, oid: &str) -> bool {
        let Some(o) = par_id(oid) else { return false };
        if o.dependances != Some("openems") || !est_installe(o, &self.racine) {
            return false;
        }
        if let Some(e) = self.etats().get_mut(oid).filter(|e| e.en_route()) {
            e.message = "Arrêtez l'outil avant d'installer openEMS.".to_string();
            return false;
        }
        self.installer_en_fond(oid, installer_openems, "openEMS installé : les simulations sont disponibles.")
    }

    fn installer_en_fond(self: &Arc<Self>, oid: &str, fonction: Installation, message_ok: &'static str) -> bool {
        let Some(o) = par_id(oid) else { return false };
        {
            let mut etats = self.etats();
            let e = etats.get_mut(o.id).expect("outil du catalogue");
            if e.phase == Some("installation") {
                return false;
            }
            e.phase = Some("installation");
            e.message = "Préparation…".to_string();
        }
        let g = self.clone();
        std::thread::spawn(move || {
            let ok = fonction(o, &g.racine, &|t| g.noter(o.id, t));
            if let Some(e) = g.etats().get_mut(o.id) {
                e.phase = if ok { None } else { Some("erreur") };
                if ok {
                    e.message = message_ok.to_string();
                }
            }
        });
        true
    }

    pub fn lancer(self: &Arc<Self>, oid: &str) -> bool {
        let Some(o) = par_id(oid) else { return false };
        self.maj_finie[o.id].attendre(90.0); // vérification du démarrage pas finie : on l'attend
        let en_route = self.etats().get(o.id).is_some_and(|e| e.en_route());
        if !en_route && git_disponible() {
            // La lib ou un projet a pu changer sur un autre PC depuis le démarrage.
            self.noter(o.id, &projets::tirer(&self.racine));
        }
        let mut etats = self.etats();
        let e = etats.get_mut(o.id).expect("outil du catalogue");
        self.actualiser(o, e);
        if e.en_route() {
            return true;
        }
        let Some(mut cmd) = crate::catalogue::demarreur(&dossier(o, &self.racine), o.nom) else {
            e.phase = Some("erreur");
            e.message = "Outil non installé.".to_string();
            return false;
        };
        let port = port_libre(o.port);
        cmd.extend(["--port".into(), port.to_string()]);
        cmd.extend(o.args.iter().filter(|x| !(self.reseau && **x == "--local")).map(|x| x.to_string()));
        let deja_maj = self.maj_deja_faite(o.id);
        if deja_maj {
            cmd.push("--sans-maj".into());
        }
        cmd.extend(projets::arguments(o.id, &self.racine, self.reseau));
        let journal = self.journal(o.id);
        tourner(&journal);
        ajouter_au_journal(
            &journal,
            &format!("\n=== lancement {} : {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), cmd.join(" ")),
        );
        let lancer = || -> io::Result<Child> {
            let sortie = ouvrir_journal(&journal)?;
            let mut c = Command::new(&cmd[0]);
            c.args(&cmd[1..])
                .current_dir(dossier(o, &self.racine))
                .env_remove("WEBSUITE_JETON")
                .stdin(Stdio::null())
                .stdout(sortie.try_clone()?)
                .stderr(sortie);
            if let Some(lanceur) = self.lanceur.lock().unwrap().as_ref() {
                c.env("WEBSUITE_LANCEUR", lanceur);
                if let Some(jeton) = self.jeton.lock().unwrap().as_ref() {
                    c.env("WEBSUITE_JETON", jeton);
                }
            }
            #[cfg(unix)]
            unsafe {
                // Sa propre session : Ctrl+C dans le terminal du lanceur ne l'atteint pas,
                // et tuer() arrête tout son groupe.
                std::os::unix::process::CommandExt::pre_exec(&mut c, || {
                    libc::setsid();
                    Ok(())
                });
            }
            sans_fenetre(&mut c).spawn()
        };
        let proc = match lancer() {
            Ok(proc) => proc,
            Err(exc) => {
                e.phase = Some("erreur");
                e.message = format!("Lancement impossible : {}", exc);
                return false;
            }
        };
        self.job.ajouter(&proc);
        e.phase = Some("demarrage");
        e.proc = Some(proc);
        e.port = Some(port);
        e.url = None;
        e.debut = maintenant();
        e.message = if deja_maj { "Démarrage…" } else { "Démarrage (vérification des mises à jour)…" }.to_string();
        true
    }

    pub fn arreter(&self, oid: &str) -> bool {
        let proc = {
            let mut etats = self.etats();
            let Some(e) = etats.get_mut(oid) else { return false };
            e.phase = None;
            e.url = None;
            e.message = "Arrêté.".to_string();
            e.proc.take()
        };
        if let Some(mut proc) = proc {
            if matches!(proc.try_wait(), Ok(None)) {
                tuer(&mut proc);
            }
        }
        true
    }

    pub fn envoyer(&self, oid: &str, message: &str) -> (bool, String) {
        let (ok, texte) = projets::envoyer(&self.racine, oid, message);
        self.noter(oid, &texte);
        let attente = projets::commits_a_pousser(&self.racine);
        *self.envoi_auto.lock().unwrap() =
            EnvoiAuto { attente, message: if ok { String::new() } else { texte.clone() }, ok: if ok { None } else { Some(false) } };
        (ok, texte)
    }

    // -- envoi automatique de ce qui a été enregistré hors ligne -----------
    /// Un essai. Renvoie le résultat de projets::pousser_en_attente (None :
    /// rien n'attendait). Le journal ne note que ce qui change, pas chaque échec.
    pub fn essai_envoi_auto(&self) -> Option<(bool, String)> {
        let r = projets::pousser_en_attente(&self.racine, 120.0);
        let attente = projets::commits_a_pousser(&self.racine);
        let mut e = self.envoi_auto.lock().unwrap();
        if let Some((ok, texte)) = &r {
            if *texte != e.message {
                ajouter_au_journal(&self.journal("web_cao"), &format!("[envoi automatique] {}\n", texte));
            }
            e.message = texte.clone();
            e.ok = Some(*ok);
        } else if attente == 0 {
            e.message = String::new();
            e.ok = None;
        }
        e.attente = attente;
        r
    }

    /// Fil de fond : un essai tout de suite (redémarrage du serveur), puis
    /// tant que quelque chose attend. Ce qui n'attend rien coûte un git rev-list.
    pub fn demarrer_envoi_auto(self: &Arc<Self>) {
        let g = self.clone();
        let _ = std::thread::Builder::new().name("envoi-auto".into()).spawn(move || {
            let mut delai = ENVOI_AUTO_MIN;
            loop {
                let r = g.essai_envoi_auto();
                delai = if r.is_some_and(|(ok, _)| !ok) { (delai * 2).min(ENVOI_AUTO_MAX) } else { ENVOI_AUTO_MIN };
                std::thread::sleep(Duration::from_secs(delai));
            }
        });
    }

    pub fn arreter_tout(&self) {
        for o in CATALOGUE {
            self.arreter(o.id);
        }
    }
}

/// Un journal trop gros passe en .log.1 (l'ancien .log.1 est remplacé).
///
/// Au lancement seulement, avant d'y écrire : l'outil précédent est arrêté,
/// personne n'a le fichier ouvert. Le bouton Journal ne lit que la fin du .log.
fn tourner(chemin: &Path) {
    if fs::metadata(chemin).is_ok_and(|m| m.len() > JOURNAL_MAX) {
        let _ = fs::rename(chemin, format!("{}.1", chemin.display()));
    }
}

/// Arrête le serveur et ses enfants (l'exécutable que lance demarrer_<NOM> en est un).
fn tuer(proc: &mut Child) {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/PID", &proc.id().to_string(), "/T", "/F"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        let _ = sans_fenetre(&mut cmd).status();
    }
    #[cfg(unix)]
    unsafe {
        if libc::killpg(proc.id() as libc::pid_t, libc::SIGTERM) != 0 {
            libc::kill(proc.id() as libc::pid_t, libc::SIGTERM);
        }
    }
    let fin = Instant::now() + Duration::from_secs(5);
    while Instant::now() < fin {
        if !matches!(proc.try_wait(), Ok(None)) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    #[cfg(unix)]
    unsafe {
        // Tout le groupe : l'exécutable lancé par demarrer_<NOM>.sh aussi.
        libc::killpg(proc.id() as libc::pid_t, libc::SIGKILL);
    }
    let _ = proc.kill();
    let _ = proc.wait();
}

#[cfg(test)]
mod essais;
