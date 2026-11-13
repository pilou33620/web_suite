//! web_bench — banc d'essai visible des outils de web_suite (WEB_CAO, WEB_ANTENNA, WEB_3D).
//!
//!     web_bench                 # outils cherchés à la racine de web_suite
//!     web_bench --outil 3d      # ouvre d'abord WEB_3D
//!     web_bench --racine D:\WEB_TOOLS --port 8200
//!
//! Ce que fait le serveur :
//! - il démarre l'outil choisi sur un port libre, avec des dossiers temporaires : projets vides
//!   et, pour WEB_CAO, une copie de la LIB. Rien de tes vrais projets n'est touché, et tout est
//!   effacé à l'arrêt ;
//! - il relaie les pages de l'outil actif sous sa propre adresse : la page Banc et l'outil sont
//!   ainsi de même origine, et la page Banc peut cliquer dans l'outil ouvert dans son cadre.
//!   Chaque page HTML relayée reçoit en tête la sonde (page/sonde.js), qui note les erreurs JS,
//!   les requêtes en échec et les dialogues ;
//! - il découvre et lance les scripts banc-* de chaque outil, un par un ;
//! - il garde les rapports du clic-partout dans web_bench/rapports/.
//!
//! Écoute sur 127.0.0.1 uniquement : le relais parle aux outils depuis la boucle locale, il ne
//! doit donc jamais être ouvert au réseau (il contournerait l'appairage).

mod bancs;
mod outils;
mod service;

use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::sync::Arc;
use std::time::Duration;

use web_tools_commun::serveur::{Requete, Serveur};

use outils::CATALOGUE;
use service::{Banc, PREFIXE};

const USAGE: &str = "usage: web_bench [-h] [--racine RACINE] [--outil {cao,antenna,3d}] [--port PORT] [--sans-navigateur]";
const AIDE: &str = "
web_bench : banc d'essai visible des outils de web_suite

options:
  -h, --help            show this help message and exit
  --racine RACINE       dossier où vivent WEB_CAO, WEB_ANTENNA, WEB_3D (défaut : la racine de web_suite)
  --outil {cao,antenna,3d}
                        outil ouvert au démarrage
  --port PORT           port de la page Banc (défaut 8200)
  --sans-navigateur     ne pas ouvrir le navigateur";

/// La racine du dépôt git WEB_SUITE, où vivent web_bench/page/ et les outils. Même règle que
/// le lanceur (catalogue::depot) : WEB_SUITE_DEPOT (scripts demarrer_*), sinon on remonte
/// depuis l'exécutable puis depuis le dossier courant jusqu'à trouver lanceur/index.html.
fn depot() -> PathBuf {
    if let Some(d) = std::env::var_os("WEB_SUITE_DEPOT").filter(|d| !d.is_empty()) {
        return PathBuf::from(d);
    }
    let departs = [std::env::current_exe().ok(), std::env::current_dir().ok()];
    for depart in departs.into_iter().flatten() {
        if let Some(d) = depart.ancestors().find(|d| d.join("lanceur").join("index.html").is_file()) {
            return d.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_default()
}

#[derive(Debug)]
struct Arguments {
    racine: Option<PathBuf>,
    outil: Option<String>,
    port: u16,
    sans_navigateur: bool,
}

/// Les options de l'ancien argparse (« --option valeur » ou « --option=valeur »).
// shortcut: pas d'abréviations (--rac), qu'argparse acceptait ; à ajouter si quelqu'un s'en sert.
fn lire_arguments(args: impl Iterator<Item = String>) -> Result<Option<Arguments>, String> {
    let mut lus = Arguments { racine: None, outil: None, port: 8200, sans_navigateur: false };
    let mut args = args;
    while let Some(arg) = args.next() {
        let (nom, mut collee) = match arg.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n.to_string(), Some(v.to_string())),
            _ => (arg.clone(), None),
        };
        let mut valeur = |args: &mut dyn Iterator<Item = String>| {
            collee.take().or_else(|| args.next()).ok_or(format!("argument {}: expected one argument", nom))
        };
        match nom.as_str() {
            "-h" | "--help" => return Ok(None),
            "--racine" => lus.racine = Some(PathBuf::from(valeur(&mut args)?)),
            "--outil" => {
                let v = valeur(&mut args)?;
                if !CATALOGUE.iter().any(|c| c.id == v) {
                    return Err(format!("argument --outil: invalid choice: '{}' (choose from cao, antenna, 3d)", v));
                }
                lus.outil = Some(v);
            }
            "--port" => {
                let v = valeur(&mut args)?;
                lus.port = v.parse().map_err(|_| format!("argument --port: invalid int value: '{}'", v))?;
            }
            "--sans-navigateur" if collee.is_none() => lus.sans_navigateur = true,
            _ => return Err(format!("unrecognized arguments: {}", arg)),
        }
    }
    Ok(Some(lus))
}

/// webbrowser.open : l'adresse dans le navigateur du poste.
fn ouvrir_navigateur(adresse: &str) {
    let mut commande = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/c", "start", ""]);
        c
    } else if cfg!(target_os = "macos") {
        Command::new("open")
    } else {
        Command::new("xdg-open")
    };
    let _ = commande.arg(adresse).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
}

fn main() -> ExitCode {
    let args = match lire_arguments(std::env::args().skip(1)) {
        Ok(Some(args)) => args,
        Ok(None) => {
            println!("{}\n{}", USAGE, AIDE);
            return ExitCode::SUCCESS;
        }
        Err(erreur) => {
            eprintln!("{}\nweb_bench: error: {}", USAGE, erreur);
            return ExitCode::from(2);
        }
    };
    let depot = depot();
    let racine = args.racine.unwrap_or_else(|| depot.clone());

    let outils = outils::trouver_outils(&racine);
    if outils.is_empty() {
        println!(
            "[!] Aucun outil trouvé dans {}. Installe-les avec installer.cmd / installer.sh, \
             ou indique leur dossier : --racine CHEMIN",
            racine.display()
        );
        return ExitCode::from(2);
    }
    let premier = outils.iter().find(|o| Some(o.conf.id) == args.outil.as_deref()).unwrap_or(&outils[0]).clone();
    println!("[*] Outils trouvés : {}", outils.iter().map(|o| o.nom()).collect::<Vec<_>>().join(", "));
    println!("[*] Démarrage de {} (dossiers temporaires)...", premier.nom());
    if let Err(exc) = premier.demarrer() {
        println!("[!] {}", exc);
        premier.arreter();
        return ExitCode::from(1);
    }

    let banc = Arc::new(Banc::new(outils, premier.conf.id, &depot.join("web_bench")));
    // 127.0.0.1 seulement (reseau = false) : ouvert au réseau, le relais contournerait l'appairage
    let serveur = match Serveur::ouvrir(args.port, false, 1).or_else(|_| Serveur::ouvrir(0, false, 1)) {
        Ok(s) => s,
        Err(exc) => {
            println!("[!] {}", exc);
            banc.arreter();
            return ExitCode::from(1);
        }
    };
    // fermé autrement que par Ctrl+C (kill, fermeture du terminal) : on arrête quand même
    // les outils et on efface leurs dossiers temporaires
    let a_arreter = serveur.clone();
    let _ = ctrlc::set_handler(move || a_arreter.arreter());

    let adresse = format!("http://127.0.0.1:{}{}/", serveur.port(), PREFIXE);
    let trouves: Vec<String> = banc.outils.iter().zip(&banc.bancs).map(|(o, b)| format!("{} {}", o.nom(), b.nombre())).collect();
    println!("[*] Bancs trouvés : {}", trouves.join(", "));
    println!("[*] Page Banc : {}   (Ctrl+C pour arrêter)", adresse);
    if !args.sans_navigateur {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            ouvrir_navigateur(&adresse);
        });
    }
    let service = banc.clone();
    serveur.servir(Arc::new(move |r: &mut Requete| service.servir(r)));
    println!("\n[*] Arrêt.");
    banc.arreter();
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lire(ligne: &str) -> Result<Option<Arguments>, String> {
        lire_arguments(ligne.split_whitespace().map(str::to_string))
    }

    #[test]
    fn options() {
        let a = lire("").unwrap().unwrap();
        assert!(a.racine.is_none() && a.outil.is_none() && a.port == 8200 && !a.sans_navigateur);
        let a = lire("--outil 3d --racine /x --port=0 --sans-navigateur").unwrap().unwrap();
        assert_eq!((a.racine, a.outil.as_deref(), a.port, a.sans_navigateur), (Some("/x".into()), Some("3d"), 0, true));
        assert!(lire("--help").unwrap().is_none() && lire("--port 1 -h").unwrap().is_none());
        assert_eq!(lire("--outil kicad").unwrap_err(), "argument --outil: invalid choice: 'kicad' (choose from cao, antenna, 3d)");
        assert_eq!(lire("--port abc").unwrap_err(), "argument --port: invalid int value: 'abc'");
        assert_eq!(lire("--port").unwrap_err(), "argument --port: expected one argument");
        assert_eq!(lire("--reseau").unwrap_err(), "unrecognized arguments: --reseau");
    }
}
