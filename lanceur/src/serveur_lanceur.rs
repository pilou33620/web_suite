//! WEB·SUITE — lanceur des outils WEB_CAO, WEB_ANTENNA et WEB_3D.
//!
//!     web_suite              ouvre le lanceur dans le navigateur
//!                            (sauf sur Raspberry Pi : adresse affichée en console)
//!     web_suite --installer  menu de téléchargement des outils seulement
//!
//! Le lanceur joue l'animation d'intro, puis propose les trois outils : il les
//! télécharge s'ils manquent (git clone à la racine du dépôt), démarre leur serveur
//! et ouvre leur page. Fermer cette fenêtre arrête les outils lancés depuis elle.
//!
//! Par défaut le serveur n'écoute que sur 127.0.0.1 :
//! il lance des programmes. Avec --reseau (défaut sur Raspberry Pi), le lanceur et
//! les outils écoutent sur le réseau local ; le lanceur exige alors des autres
//! appareils un jeton, qu'ils reçoivent en saisissant le code d'appairage affiché en
//! console (ou avec l'adresse « Réseau » complète, ?jeton=...). Les outils exigent
//! le même code, avec le même jeton et le même cookie (appairage.rs) : un appareil
//! appairé une fois l'est pour le lanceur et pour tous les outils. --local
//! force l'écoute locale.

use std::fs;
use std::io::{IsTerminal, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use web_tools_commun::appairage::{self, Appairage, Visites};
use web_tools_commun::serveur::{Reponse, Requete, Serveur};

use crate::catalogue::{depot, ici, par_id, Outil, CATALOGUE, PORT_DEFAUT};
use crate::outils::{self, Gestionnaire};
use crate::processus::fin;
use crate::{connexion_github, installer, projets};

const CORPS_MAX: i64 = 64 * 1024; // les actions ne portent qu'un id et un message
// Pages ouvertes à un appareil qui n'a pas encore le jeton : l'accueil (qui le
// renvoie vers la page du code), la page du code, et la route qui le vérifie.
// La page du code a SA propre adresse : servie à « / » comme le lanceur, Safari
// la ressortait de son cache après un bon code, au lieu du lanceur.
const PAGE_APPAIRAGE: &str = "/appairage";
const PAGES_APPAIRAGE: &[&str] = &["/", "/index.html", PAGE_APPAIRAGE];

fn fichier_servi(chemin: &str) -> Option<&'static str> {
    match chemin {
        "/" | "/index.html" => Some("index.html"),
        "/websuite-intro.html" => Some("websuite-intro.html"),
        _ => None,
    }
}

// Host contrôlé (DNS rebinding), code d'appairage et jeton : appairage.rs, commun
// à tous les web tools. Le jeton est partagé par les outils de ce poste
// (~/.web_tools/jeton-reseau.txt) et le cookie porte le même nom partout : une
// tablette appairée ici l'est aussi pour WEB_CAO, WEB_ANTENNA, WEB_3D, web_ferme…

/// Le jeton du mode réseau, partagé avec les autres web tools du poste et
/// gardé d'un démarrage à l'autre : les favoris de la tablette restent bons.
/// L'ancien lanceur/jeton-reseau.txt est repris : les appareils déjà appairés
/// le restent.
fn jeton_reseau() -> String {
    appairage::jeton(Some(&ici().join("jeton-reseau.txt")))
}

// ---------------------------------------------------------------------------
// Mise à jour de WEB_SUITE elle-même, comme les outils le font au démarrage
// ---------------------------------------------------------------------------
// Attente des vérifications des outils avant d'afficher leur bilan en console.
// Au-delà, le bilan d'un outil s'affiche plus bas, dès qu'il arrive.
const DELAI_BILAN_MAJ: f64 = 30.0;

fn vider() {
    let _ = std::io::stdout().flush();
}

/// -> (true si WEB_SUITE vient d'être mis à jour et doit redémarrer, dernier message).
fn verifier_et_appliquer_maj() -> (bool, String) {
    println!("  Recherche de mises à jour de WEB_SUITE...");
    vider();
    let dernier = std::cell::RefCell::new(String::new());
    let statut = outils::maj_depot(&depot(), &|texte| {
        *dernier.borrow_mut() = texte.to_string();
        println!("    {}", texte);
        vider();
    });
    let dernier = dernier.into_inner();
    let texte = if dernier.is_empty() { outils::bilan_sans_texte(statut).to_string() } else { dernier };
    if statut != "maj" {
        return (false, texte);
    }
    println!("  [*] WEB_SUITE mis à jour, redémarrage.");
    vider();
    (true, texte)
}

/// -> ([(outil, état pour le bandeau)], [outils dont la vérification tourne encore]).
fn bilans_outils(g: &Gestionnaire, racine: &Path, verifie: bool) -> (Vec<(&'static Outil, String)>, Vec<&'static Outil>) {
    let installe = |o: &Outil| outils::est_installe(o, racine);
    if verifie && CATALOGUE.iter().any(|o| installe(o) && !g.maj_est_finie(o.id)) {
        println!("  Recherche de mises à jour des outils...");
        vider();
    }
    let limite = Instant::now() + Duration::from_secs_f64(DELAI_BILAN_MAJ);
    let (mut etats, mut en_retard) = (Vec::new(), Vec::new());
    for o in CATALOGUE {
        if !installe(o) {
            etats.push((o, "non installé".to_string()));
            continue;
        }
        let reste = limite.saturating_duration_since(Instant::now()).as_secs_f64();
        let bilan = if verifie { g.bilan_maj(o.id, reste) } else { Some(String::new()) };
        let bilan = bilan.unwrap_or_else(|| {
            en_retard.push(o);
            "vérification des mises à jour en cours...".to_string()
        });
        etats.push((o, if bilan.is_empty() { "installé".to_string() } else { format!("installé · {}", bilan) }));
    }
    (etats, en_retard)
}

/// Affiche le bilan des vérifications qui ont dépassé DELAI_BILAN_MAJ.
fn annoncer_plus_tard(g: Arc<Gestionnaire>, en_retard: Vec<&'static Outil>) {
    std::thread::spawn(move || {
        for o in en_retard {
            let bilan = g.bilan_maj(o.id, 120.0).filter(|b| !b.is_empty());
            println!("  Mise à jour {} : {}", o.nom, bilan.as_deref().unwrap_or("toujours sans réponse."));
            vider();
        }
    });
}

#[cfg(windows)]
unsafe extern "system" fn ctrl_c_ignore(evenement: u32) -> i32 {
    (evenement == windows_sys::Win32::System::Console::CTRL_C_EVENT) as i32
}

fn redemarrer(bilan: &str) -> ! {
    vider();
    if std::env::var("WEB_SUITE_RELANCEUR").as_deref() == Ok("1") {
        // Le script demarrer_* reboucle : il récupère le nouvel exécutable et le relance
        // (ce binaire-ci ne peut pas se recompiler).
        std::process::exit(75);
    }
    // Le bilan passe au nouveau lanceur, qui l'affiche avec celui des outils.
    let mut cmd = std::process::Command::new(std::env::current_exe().unwrap_or_else(|_| "web_suite".into()));
    cmd.args(std::env::args_os().skip(1)).env("WEB_SUITE_DEJA_MAJ", "1").env("WEB_SUITE_BILAN_MAJ", bilan);
    #[cfg(unix)]
    {
        // Linux, Pi : le nouveau lanceur REMPLACE celui-ci, comme le
        // fait WEB_CAO. Un parent qui attendait recevait lui aussi Ctrl+C et
        // l'affichait en trace d'erreur sous le « Arrêt. » de l'enfant ; et un
        // kill visait le parent en laissant l'enfant tourner.
        let exc = std::os::unix::process::CommandExt::exec(&mut cmd);
        println!("  [X] Redémarrage impossible : {}", exc);
        std::process::exit(1);
    }
    #[cfg(not(unix))]
    {
        // Ctrl+C atteint aussi l'enfant, qui arrête ses outils et sort
        // proprement : on l'attend, sans trace d'erreur.
        #[cfg(windows)]
        unsafe {
            windows_sys::Win32::System::Console::SetConsoleCtrlHandler(Some(ctrl_c_ignore), 1);
        }
        let code = cmd.status().ok().and_then(|s| s.code()).unwrap_or(1);
        std::process::exit(code);
    }
}

/// Recopie dans ce terminal les lignes [connexion] des journaux des outils
/// (leur sortie va dans journaux/, pas ici) : qui utilise quel outil, depuis quelle IP.
fn relayer_connexions(g: Arc<Gestionnaire>) {
    // ce qui précède ce démarrage est déjà vu
    let mut lus: Vec<(&str, u64)> =
        CATALOGUE.iter().map(|o| (o.id, fs::metadata(g.journal(o.id)).map(|m| m.len()).unwrap_or(0))).collect();
    loop {
        std::thread::sleep(Duration::from_secs(1));
        for (oid, lu) in lus.iter_mut() {
            let mut lire = || -> std::io::Result<Vec<u8>> {
                let mut f = fs::File::open(g.journal(oid))?;
                if f.seek(SeekFrom::End(0))? < *lu {
                    *lu = 0; // journal tourné en .log.1 au lancement
                }
                f.seek(SeekFrom::Start(*lu))?;
                let mut bloc = Vec::new();
                f.read_to_end(&mut bloc)?;
                Ok(bloc)
            };
            let Ok(bloc) = lire() else { continue };
            let complet = bloc.iter().rposition(|c| *c == b'\n').map_or(0, |i| i + 1); // les lignes complètes seulement
            *lu += complet as u64;
            for ligne in String::from_utf8_lossy(&bloc[..complet]).lines() {
                if ligne.contains(appairage::MARQUE) {
                    println!("{}", ligne.trim_end());
                    vider();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Serveur HTTP
// ---------------------------------------------------------------------------
pub struct Lanceur {
    pub gestionnaire: Arc<Gestionnaire>,
    /// en --reseau : exigé des autres appareils
    pub jeton: Option<String>,
    /// en --reseau : le code court
    pub appairage: Option<Appairage>,
    /// les appareils connectés, dans ce terminal
    visites: Visites,
}

fn erreur(code: u16, texte: &str) -> Reponse {
    Reponse::json(code, &json!({ "erreur": texte }))
}

/// str(valeur or "") pour une valeur JSON.
fn texte_de(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) | Some(Value::Bool(false)) => String::new(),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => String::new(),
        Some(Value::Array(a)) if a.is_empty() => String::new(),
        Some(Value::Object(o)) if o.is_empty() => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(true)) => "True".to_string(),
        Some(v) => v.to_string(),
    }
}

impl Lanceur {
    pub fn new(gestionnaire: Arc<Gestionnaire>, jeton: Option<String>, appairage: Option<Appairage>) -> Self {
        Lanceur { gestionnaire, jeton, appairage, visites: Visites::new("WEB·SUITE", None) }
    }

    pub fn servir(&self, r: &mut Requete) -> Reponse {
        let autorise = match self.filtrer(r) {
            Ok(autorise) => autorise,
            Err(refus) => return refus,
        };
        match r.methode.as_str() {
            "GET" | "HEAD" => self.lecture(r, autorise),
            "POST" => self.action(r, autorise),
            autre => Reponse::texte(501, &format!("Unsupported method ('{}')", autre)),
        }
    }

    /// Ok(l'appareil est autorisé) : la requête suit son cours. Err : la réponse à envoyer.
    fn filtrer(&self, r: &Requete) -> Result<bool, Reponse> {
        if !appairage::hote_permis(r.entete("Host")) {
            return Err(Reponse::texte(403, "Host non autorise (protection DNS rebinding)"));
        }
        if appairage::internet(r.client.ip(), r.local.ip()) {
            // « :: » écoute aussi l'IPv6 publique du poste
            return Err(Reponse::texte(403, "Acces depuis Internet refuse : passez par le reseau local ou Tailscale"));
        }
        if let Some(reponse) = appairage::filtrer_tls(r) {
            return Err(reponse); // la CA du poste, ou le renvoi vers https://
        }
        let autorise = match &self.jeton {
            Some(jeton) => appairage::ce_poste(r) || appairage::jeton_recu(r, jeton),
            None => true,
        };
        let chemin = r.chemin();
        if !autorise
            && self.appairage.is_some()
            && ((r.methode == "GET" && PAGES_APPAIRAGE.contains(&chemin)) || (r.methode == "POST" && chemin == "/api/appairer"))
        {
            return Ok(false); // la page du code, et sa vérification
        }
        if !autorise {
            // Sans cela, tout appareil du réseau poussait sur GitHub avec les
            // identifiants de ce poste et changeait son identité git.
            return Err(Reponse::texte(
                403,
                "Jeton manquant\nOuvrir l'adresse Reseau affichee dans la console du lanceur (elle se termine par ?jeton=...).",
            ));
        }
        self.visites.noter(r.client.ip());
        Ok(true)
    }

    fn fichier(&self, nom: &str) -> Reponse {
        // no-store (posé par Reponse::new) : ces pages dépendent de l'autorisation de
        // l'appareil, une copie gardée par le navigateur montrerait la mauvaise
        match fs::read(ici().join(nom)) {
            Ok(corps) => Reponse::new(200, "text/html; charset=utf-8", corps),
            Err(exc) => Reponse::texte(500, &format!("{} : {}", nom, exc)),
        }
    }

    fn biscuit_jeton(&self, r: &Requete) -> String {
        appairage::biscuit(self.jeton.as_deref().unwrap_or(""), r.https)
    }

    fn lecture(&self, r: &Requete, autorise: bool) -> Reponse {
        let chemin = r.chemin();
        let g = &self.gestionnaire;
        if !autorise {
            // Appareil sans jeton : la page qui demande le code d'appairage
            // (le code s'affiche, s'il le faut, dans le terminal du serveur).
            if chemin != PAGE_APPAIRAGE {
                return Reponse::redirection(303, PAGE_APPAIRAGE);
            }
            if let Some(a) = &self.appairage {
                a.preparer();
            }
            return self.fichier("appairage.html");
        }
        if chemin == PAGE_APPAIRAGE {
            return Reponse::redirection(303, "/"); // déjà autorisé : au lanceur
        }
        if self.jeton.is_some() && r.param("jeton").is_some() {
            // Le jeton passe dans un cookie et quitte l'adresse : il ne reste
            // ni dans l'historique, ni dans un lien recopié.
            return Reponse { code: 303, entetes: Vec::new(), corps: Vec::new() }
                .entete("Set-Cookie", &self.biscuit_jeton(r))
                .entete("Location", if chemin.is_empty() { "/" } else { chemin });
        }
        if let Some(nom) = fichier_servi(chemin) {
            return self.fichier(nom);
        }
        if chemin == "/api/etat" {
            return Reponse::json(
                200,
                &json!({
                    "racine": g.racine.to_string_lossy(), "git": outils::git_disponible(),
                    "projets": projets::dossier(&g.racine).to_string_lossy(), "outils": g.etat(),
                    "envoi_auto": g.envoi_auto(),
                }),
            );
        }
        if chemin == "/api/journal" {
            let Some(o) = r.param("id").as_deref().and_then(par_id) else { return erreur(404, "outil inconnu") };
            let texte = fs::read(g.journal(o.id)).map(|t| String::from_utf8_lossy(&t).into_owned()).unwrap_or_default();
            return Reponse::json(200, &json!({ "texte": fin(&texte, 20000) }));
        }
        Reponse::texte(404, "Not Found")
    }

    fn action(&self, r: &mut Requete, autorise: bool) -> Reponse {
        // En-tête maison obligatoire : une autre page ouverte dans le navigateur
        // ne peut pas l'ajouter sans pré-vol CORS, que ce serveur ne valide pas.
        // Aucun site tiers ne peut donc lancer ou télécharger quoi que ce soit.
        if r.entete("X-WebSuite") != Some("1") {
            return erreur(403, "requête refusée");
        }
        let longueur = match r.entete("Content-Length").map(str::trim) {
            None | Some("") => Ok(0),
            Some(v) => v.parse::<i64>(),
        };
        let Ok(longueur) = longueur else { return erreur(400, "JSON invalide") };
        if !(0..=CORPS_MAX).contains(&longueur) {
            return erreur(413, "requête trop grande");
        }
        let Ok(brut) = r.lire_corps(CORPS_MAX as usize) else { return erreur(400, "JSON invalide") };
        let Ok(Value::Object(corps)) = serde_json::from_slice::<Value>(if brut.is_empty() { b"{}" } else { &brut }) else {
            return erreur(400, "JSON invalide");
        };
        if !autorise {
            // Seule route ouverte sans jeton (filtrer) : le bon code
            // donne à l'appareil le cookie du jeton, rien de plus.
            let Some(a) = &self.appairage else { return erreur(403, "requête refusée") };
            let (ok, texte) = a.essayer(&texte_de(corps.get("code")));
            let reponse = Reponse::json(if ok { 200 } else { 403 }, &json!({"ok": ok, "message": texte}));
            return if ok { reponse.entete("Set-Cookie", &self.biscuit_jeton(r)) } else { reponse };
        }
        let chemin = r.chemin();
        if chemin == "/api/appairer" {
            // Déjà autorisé (un premier envoi du code a réussi, la page a été
            // validée deux fois) : rien à vérifier, la page passe au lanceur.
            return Reponse::json(200, &json!({"ok": true, "message": "Cet appareil est déjà autorisé."}));
        }
        let oid = corps.get("id").and_then(Value::as_str);
        let g = &self.gestionnaire;
        if chemin == "/api/identite" {
            let (ok, texte) = projets::definir_identite(&g.racine, &texte_de(corps.get("nom")), &texte_de(corps.get("email")));
            return Reponse::json(200, &json!({"ok": ok, "message": texte}));
        }
        if chemin == "/api/envoyer" {
            let Some(o) = oid.and_then(par_id) else { return erreur(400, "outil inconnu") };
            if projets::identite_manquante(&g.racine) {
                // Poste neuf (Raspberry Pi...) : la page demande nom et e-mail, puis renvoie.
                return Reponse::json(
                    200,
                    &json!({"ok": false, "identite": true, "outils": g.etat(),
                            "message": "git ne connaît pas encore votre nom et e-mail."}),
                );
            }
            let (ok, texte) = g.envoyer(o.id, &texte_de(corps.get("message")));
            return Reponse::json(200, &json!({"ok": ok, "message": texte, "outils": g.etat()}));
        }
        if !["/api/installer", "/api/lancer", "/api/arreter", "/api/openems"].contains(&chemin) {
            return erreur(404, "action inconnue");
        }
        let cibles: Vec<&Outil> = if oid == Some("tous") { CATALOGUE.iter().collect() } else { oid.and_then(par_id).into_iter().collect() };
        if cibles.is_empty() {
            return erreur(400, "outil inconnu");
        }
        for o in cibles {
            if chemin == "/api/installer" && outils::est_installe(o, &g.racine) {
                continue;
            }
            if chemin == "/api/lancer" || chemin == "/api/arreter" {
                println!(
                    "  {} {}  {:<11} {}  ->  {} {}",
                    appairage::MARQUE,
                    appairage::heure(),
                    "WEB·SUITE",
                    r.client.ip(),
                    if chemin == "/api/lancer" { "ouvre" } else { "arrête" },
                    o.nom
                );
                vider();
            }
            match chemin {
                "/api/installer" => g.installer(o.id),
                "/api/lancer" => g.lancer(o.id),
                "/api/arreter" => g.arreter(o.id),
                _ => g.installer_openems(o.id),
            };
        }
        Reponse::json(200, &json!({"ok": true, "outils": g.etat()}))
    }
}

/// Vrai si la machine est un Raspberry Pi (modèle lu dans le device tree).
fn est_raspberry_pi() -> bool {
    ["/proc/device-tree/model", "/sys/firmware/devicetree/base/model"]
        .iter()
        .any(|chemin| fs::read(chemin).is_ok_and(|m| String::from_utf8_lossy(&m).to_lowercase().contains("raspberry pi")))
}

/// Vrai sur une machine qui sert les autres appareils (Raspberry Pi) : réseau
/// local par défaut, et pas de navigateur ouvert sur place.
fn poste_serveur() -> bool {
    est_raspberry_pi()
}

/// webbrowser.open : la page dans le navigateur par défaut du poste.
fn ouvrir_navigateur(url: &str) {
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("rundll32");
        c.args(["url.dll,FileProtocolHandler", url]);
        c
    } else {
        let mut c = std::process::Command::new(if cfg!(target_os = "macos") { "open" } else { "xdg-open" });
        c.arg(url);
        c
    };
    use std::process::Stdio;
    let _ = cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
}

static ARRET: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn sur_signal(_: libc::c_int) {
    ARRET.store(true, Ordering::SeqCst); // un second signal n'interrompt pas l'arrêt
}

#[cfg(windows)]
unsafe extern "system" fn sur_ctrl_c(evenement: u32) -> i32 {
    if evenement != windows_sys::Win32::System::Console::CTRL_C_EVENT {
        return 0; // fenêtre fermée : l'objet Job arrête les outils
    }
    ARRET.store(true, Ordering::SeqCst);
    1
}

/// Ctrl+C, et hors Windows SIGHUP et SIGTERM, demandent l'arrêt.
fn ecouter_signaux() {
    #[cfg(unix)]
    unsafe {
        // Fermer le terminal (SIGHUP) ou un kill/systemctl stop (SIGTERM) doit
        // arrêter les outils comme Ctrl+C : ils sont dans leur propre session,
        // rien d'autre ne les arrêterait. Sous Windows, l'objet Job s'en charge.
        for signal in [libc::SIGINT, libc::SIGHUP, libc::SIGTERM] {
            libc::signal(signal, sur_signal as extern "C" fn(libc::c_int) as libc::sighandler_t);
        }
    }
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::System::Console::SetConsoleCtrlHandler(Some(sur_ctrl_c), 1);
    }
}

const USAGE: &str = "usage: web_suite [-h] [--port PORT] [--racine RACINE] [--installer] [--navigateur] \
                     [--sans-navigateur] [--reseau | --local] [--sans-maj]";

fn aide() {
    println!("{}\n", USAGE);
    println!("Lanceur WEB·SUITE.\n");
    println!("options:");
    println!("  -h, --help         show this help message and exit");
    println!("  --port PORT        port du lanceur (défaut : {})", PORT_DEFAUT);
    println!("  --racine RACINE    dossier contenant les outils (défaut : le dossier de WEB_SUITE)");
    println!("  --installer        ouvrir seulement le menu de téléchargement des outils");
    println!("  --navigateur       ouvrir le navigateur au démarrage même sur Raspberry Pi");
    println!("  --sans-navigateur  ne pas ouvrir le navigateur au démarrage");
    println!("  --reseau           écouter sur le réseau local : ouvrir WEB·SUITE depuis un autre appareil (défaut sur");
    println!("                     Raspberry Pi)");
    println!("  --local            n'écouter que sur 127.0.0.1 (défaut ailleurs)");
    println!("  --sans-maj         ne pas vérifier les mises à jour GitHub au démarrage");
}

/// La ligne de commande (argv sans le nom du programme). Renvoie le code de sortie.
pub fn principal(argv: &[String]) -> i32 {
    let mut port = PORT_DEFAUT;
    let mut racine = depot();
    let (mut seulement_installer, mut navigateur, mut sans_navigateur, mut verifier_maj) = (false, false, false, true);
    let mut ecoute: Option<bool> = None;
    let erreur = |texte: &str| {
        eprintln!("{}\nweb_suite: error: {}", USAGE, texte);
        2
    };
    let mut args = argv.iter().map(String::as_str);
    while let Some(arg) = args.next() {
        let (nom, mut valeur) = match arg.split_once('=') {
            Some((nom, valeur)) if nom.starts_with("--") => (nom, Some(valeur)),
            _ => (arg, None),
        };
        match nom {
            "-h" | "--help" => {
                aide();
                return 0;
            }
            "--port" | "--racine" => {
                let Some(v) = valeur.take().or_else(|| args.next()) else {
                    return erreur(&format!("argument {}: expected one argument", nom));
                };
                if nom == "--racine" {
                    racine = PathBuf::from(v);
                } else {
                    match v.parse() {
                        Ok(p) => port = p,
                        Err(_) => return erreur(&format!("argument --port: invalid int value: '{}'", v)),
                    }
                }
            }
            "--installer" => seulement_installer = true,
            "--navigateur" => navigateur = true,
            "--sans-navigateur" => sans_navigateur = true,
            "--reseau" | "--local" => {
                let choix = nom == "--reseau";
                if ecoute.is_some_and(|e| e != choix) {
                    return erreur(&format!("argument {}: not allowed with argument {}", nom, if choix { "--local" } else { "--reseau" }));
                }
                ecoute = Some(choix);
            }
            "--sans-maj" => verifier_maj = false,
            _ => return erreur(&format!("unrecognized arguments: {}", arg)),
        }
        if valeur.is_some() {
            return erreur(&format!("argument {}: ignored explicit argument", nom));
        }
    }
    let racine = std::path::absolute(&racine).unwrap_or(racine);
    let serveur_dedie = poste_serveur();
    let reseau = ecoute.unwrap_or(serveur_dedie);

    if seulement_installer {
        return installer::principal(&["--racine".to_string(), racine.to_string_lossy().into_owned()]);
    }

    let deja_maj = std::env::var("WEB_SUITE_DEJA_MAJ").as_deref() == Ok("1");
    let mut bilan_suite = std::env::var("WEB_SUITE_BILAN_MAJ").ok().filter(|b| !b.is_empty()).unwrap_or_else(|| {
        if deja_maj { "mis à jour, redémarré." } else { "" }.to_string()
    });
    std::env::remove_var("WEB_SUITE_BILAN_MAJ");
    if !verifier_maj {
        bilan_suite = "mises à jour non vérifiées (--sans-maj)".to_string();
    } else if !deja_maj {
        let (a_relancer, bilan) = verifier_et_appliquer_maj();
        if a_relancer {
            redemarrer(&bilan);
        }
        bilan_suite = bilan;
    }

    // Premier démarrage sans aucun outil : autant demander tout de suite, en
    // console, lequel télécharger. La page le propose aussi, pour les autres cas.
    let terminal = std::io::stdin().is_terminal();
    let aucun = !CATALOGUE.iter().any(|o| outils::est_installe(o, &racine));
    if aucun && terminal {
        println!("\n  Aucun outil installé dans {}.", racine.display());
        installer::installer(&installer::menu(&racine), &racine, true);
    }

    if outils::git_disponible() {
        println!("  Projets : {}", projets::preparer(&racine, &projets::depot()));
        vider();
        // Téléphone, Pi : se connecter à GitHub ici, une fois, plutôt que de
        // l'apprendre à la tablette au premier envoi (connexion_github.rs).
        if terminal && connexion_github::proposer(&racine) == "connecte" {
            // Dépôt privé : le clone du démarrage a pu être refusé, il passe maintenant.
            println!("  Projets : {}", projets::preparer(&racine, &projets::depot()));
            vider();
        }
    }

    let g = Gestionnaire::new(&racine, reseau);
    let lanceur = Arc::new(Lanceur::new(g.clone(), reseau.then(jeton_reseau), reseau.then(|| Appairage::new(None, ""))));
    if verifier_maj {
        // Pendant l'animation d'intro : les outils lancés ensuite n'ont plus à le faire.
        g.verifier_maj_outils();
    }
    let serveur = match Serveur::ouvrir(port, reseau, 10) {
        Ok(serveur) => serveur,
        Err(exc) => {
            println!("  [X] {}", exc);
            return 1;
        }
    };
    let port = serveur.port();
    let url = format!("http://127.0.0.1:{}/", port);
    // En réseau : HTTPS sur le même port (la boucle locale garde HTTP), voir appairage.rs.
    let (mut tls, mut sans_tls) = (false, String::new());
    if reseau {
        match appairage::contexte_tls() {
            Ok(cfg) => {
                serveur.brancher_tls(cfg);
                tls = true;
            }
            Err(raison) => sans_tls = raison,
        }
    }
    let schema = if tls { "https" } else { "http" };
    let (etats_outils, en_retard) = bilans_outils(&g, &racine, verifier_maj);
    *g.lanceur.lock().unwrap() = Some(url.trim_end_matches('/').to_string());
    *g.jeton.lock().unwrap() = lanceur.jeton.clone();
    if outils::git_disponible() {
        // Ce qui a été enregistré hors ligne repart dès que GitHub répond.
        g.demarrer_envoi_auto();
    }

    println!();
    println!("  WEB·SUITE");
    println!("  {}", "-".repeat(48));
    println!("  Adresse   {}", url);
    let ip = if reseau { appairage::adresse_reseau() } else { None };
    let jeton = lanceur.jeton.as_deref().unwrap_or("");
    if let Some(ip) = ip {
        println!("  Réseau    {}://{}:{}/", schema, ip, port);
        if let Some(ts) = appairage::ip_tailscale() {
            println!("  Tailscale {}://{}:{}/", schema, ts, port);
        }
        println!("            tablette, téléphone, autre poste : ouvrir cette adresse,");
        println!("            puis saisir le code d'appairage affiché ci-dessous");
        println!("            (ou directement, sans code : {}://{}:{}/?jeton={} ;", schema, ip, port, jeton);
        println!("             supprimer {} pour changer de jeton)", appairage::fichier_jeton().display());
        if tls {
            println!("{}", appairage::ligne_https(&ip.to_string(), port));
        } else if !sans_tls.is_empty() {
            println!("  HTTP en clair sur le réseau ({}) : le jeton peut être lu sur le Wi-Fi.", sans_tls);
        }
    } else if reseau {
        println!("  Réseau    écoute sur le réseau, adresse IP du poste introuvable");
        println!("            jeton à ajouter à l'adresse : ?jeton={}", jeton);
    } else {
        println!("  Réseau    désactivé (--reseau pour y accéder depuis un autre appareil)");
    }
    println!("  Outils    {}", racine.display());
    println!("  Projets   {}", projets::dossier(&racine).display());
    println!("            {:<12} {}", "WEB_SUITE", bilan_suite);
    for (o, etat) in &etats_outils {
        println!("            {:<12} {}", o.nom, etat);
    }
    println!();
    println!("  Fermer cette fenêtre (ou Ctrl+C) arrête aussi les outils lancés.");
    println!();
    vider();

    let ouvrir = !sans_navigateur && (navigateur || !serveur_dedie);
    if !ouvrir {
        match ip {
            Some(ip) => println!("  Navigateur non ouvert : aller sur {}://{}:{}/", schema, ip, port),
            None => println!("  Navigateur non ouvert : aller sur {}", url),
        }
        if let Some(a) = &lanceur.appairage {
            // en dernier, là où l'œil tombe ; un nouveau code s'affiche ici aussi
            a.preparer();
        }
        println!();
        vider();
    } else {
        let url = url.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(600));
            ouvrir_navigateur(&url);
        });
        if let Some(a) = &lanceur.appairage {
            a.preparer();
        }
    }
    ecouter_signaux();
    if !en_retard.is_empty() {
        annoncer_plus_tard(g.clone(), en_retard);
    }
    let relais = g.clone();
    std::thread::spawn(move || relayer_connexions(relais));
    crate::claude_pages::demarrer(); // le panneau « Claude » des pages écrit à Claude Code de ce poste
    serveur.servir_en_fond(Arc::new(move |r: &mut Requete| lanceur.servir(r)));
    while !ARRET.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
    }
    println!("\n  Arrêt.");
    g.arreter_tout();
    serveur.arreter();
    0
}
