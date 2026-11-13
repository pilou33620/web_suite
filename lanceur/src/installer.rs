//! Installation des outils WEB·SUITE : demande lequel télécharger (ou les trois).
//!
//!     installer            menu interactif
//!     installer cao 3d     installe directement WEB_CAO et WEB_3D
//!     installer tous       installe les trois
//!
//! Chaque outil est cloné depuis GitHub à la racine de ce dépôt
//! (WEB_CAO/, WEB_ANTENNA/, WEB_3D/, ignorés par git), et garde son propre dépôt
//! git : il se met à jour seul à
//! chaque démarrage.

use std::path::Path;

use crate::catalogue::{depot, par_id, Outil, CATALOGUE};
use crate::outils;
use crate::processus::saisir;

fn alias(jeton: &str) -> Option<&'static str> {
    Some(match jeton {
        "1" | "cao" | "web_cao" => "web_cao",
        "2" | "antenna" | "antenne" | "web_antenna" => "web_antenna",
        "3" | "3d" | "web_3d" => "web_3d",
        _ => return None,
    })
}
const TOUS: &[&str] = &["4", "t", "tous", "tout", "all", "les3", "*"];

/// '1,3' / '1 3' / 'tous' -> liste d'ids, ou None si la saisie est invalide.
pub fn interpreter(reponse: &str) -> Option<Vec<&'static str>> {
    let bas = reponse.to_lowercase().replace([',', ';'], " ");
    let jetons: Vec<&str> = bas.split_whitespace().collect();
    if jetons.is_empty() || jetons == ["0"] || jetons == ["q"] {
        return Some(Vec::new());
    }
    if jetons.iter().any(|j| TOUS.contains(j)) {
        return Some(CATALOGUE.iter().map(|o| o.id).collect());
    }
    let mut ids = Vec::new();
    for j in jetons {
        let id = alias(j)?;
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    Some(ids)
}

pub fn menu(racine: &Path) -> Vec<&'static str> {
    println!();
    println!("  WEB·SUITE — installation des outils");
    println!("  {}", "-".repeat(56));
    println!("  Dossier : {}", racine.display());
    if !outils::git_disponible() {
        println!("  [!] git introuvable : les outils seront téléchargés en zip et");
        println!("      ne pourront pas se mettre à jour seuls. https://git-scm.com");
    }
    println!();
    for (i, o) in CATALOGUE.iter().enumerate() {
        let etat = if outils::est_installe(o, racine) { "installé" } else { "à télécharger" };
        println!("   {}  {:<12}  {:<30} [{}]", i + 1, o.nom, o.sous_titre, etat);
    }
    println!("   4  Les trois");
    println!("   0  Rien, quitter");
    println!();
    loop {
        let Some(reponse) = saisir("  Quel(s) outil(s) télécharger ? (ex. 1, 1 3 ou 4) : ") else {
            println!();
            return Vec::new();
        };
        if let Some(ids) = interpreter(&reponse) {
            return ids;
        }
        println!("  Réponse non comprise : tapez 1, 2, 3, 4, ou plusieurs numéros.");
    }
}

fn oui(question: &str) -> bool {
    match saisir(&format!("  {} [o/N] : ", question)) {
        Some(r) => ["o", "oui", "y", "yes"].contains(&r.trim().to_lowercase().as_str()),
        None => {
            println!();
            false
        }
    }
}

pub fn installer(ids: &[&str], racine: &Path, demander_dependances: bool) -> bool {
    let ecrire = |t: &str| println!("  {}", t);
    let mut installes: Vec<&Outil> = Vec::new();
    let mut dependances_ok = true;
    for o in ids.iter().filter_map(|oid| par_id(oid)) {
        println!();
        println!("  == {} ==", o.nom);
        if outils::installer(o, racine, &ecrire) {
            installes.push(o);
        }
    }

    for o in &installes {
        if o.dependances == Some("pip") && demander_dependances {
            println!();
            if !outils::installer_dependances(o, racine, &ecrire) {
                dependances_ok = false;
            }
        } else if o.dependances == Some("openems") && outils::openems_pret(o, racine).is_none() {
            println!();
            println!("  {} : openEMS ne s'installe tout seul que sous Windows. Ailleurs, voir son requirements.txt.", o.nom);
        } else if o.dependances == Some("openems") && outils::openems_pret(o, racine) == Some(false) {
            println!();
            println!("  {} démarre sans rien d'autre, mais ses simulations demandent openEMS.", o.nom);
            if demander_dependances && oui("Installer openEMS maintenant (~50 Mo, Python 3.10 ou 3.11 requis) ?") {
                outils::installer_openems(o, racine, &ecrire);
            } else {
                println!("  Plus tard : installer antenna (installer.cmd ou ./installer.sh)");
            }
        }
    }

    let echecs = ids.len() - installes.len();
    println!();
    if !ids.is_empty() {
        let suite = if echecs > 0 { format!(", {} échec(s)", echecs) } else { String::new() };
        println!("  Terminé : {} outil(s) prêt(s){}.", installes.len(), suite);
    }
    echecs == 0 && dependances_ok
}

const USAGE: &str = "usage: installer [-h] [--racine RACINE] [--sans-dependances] [outils ...]";

/// La ligne de commande (argv sans le nom du programme). Renvoie le code de sortie.
pub fn principal(argv: &[String]) -> i32 {
    let mut noms: Vec<&str> = Vec::new();
    let mut racine = depot();
    let mut sans_dependances = false;
    let erreur = |texte: &str| {
        eprintln!("{}\ninstaller: error: {}", USAGE, texte);
        2
    };
    let mut args = argv.iter().map(String::as_str);
    while let Some(arg) = args.next() {
        match arg {
            "-h" | "--help" => {
                println!("{}\n", USAGE);
                println!("Télécharge un, plusieurs ou les trois outils WEB·SUITE.\n");
                println!("positional arguments:");
                println!("  outils              cao, antenna, 3d, ou tous (sans argument : menu interactif)\n");
                println!("options:");
                println!("  -h, --help          show this help message and exit");
                println!("  --racine RACINE     dossier où cloner les outils (défaut : le dossier de WEB_SUITE)");
                println!("  --sans-dependances  ne pas installer les dépendances (numpy et scipy, sans lesquels WEB_CAO");
                println!("                      ne démarre pas) ni proposer openEMS");
                return 0;
            }
            "--racine" => match args.next() {
                Some(v) => racine = v.into(),
                None => return erreur("argument --racine: expected one argument"),
            },
            "--sans-dependances" => sans_dependances = true,
            _ if arg.starts_with("--racine=") => racine = arg["--racine=".len()..].into(),
            _ if arg.starts_with('-') && arg.len() > 1 => return erreur(&format!("unrecognized arguments: {}", arg)),
            _ => noms.push(arg),
        }
    }
    let ids = if noms.is_empty() {
        menu(&racine)
    } else {
        match interpreter(&noms.join(" ")) {
            Some(ids) => ids,
            None => return erreur("outil inconnu : choisir parmi cao, antenna, 3d, tous"),
        }
    };
    if installer(&ids, &racine, !sans_dependances) { 0 } else { 1 }
}

#[cfg(test)]
mod essais {
    use super::*;

    #[test]
    fn saisies() {
        assert_eq!(interpreter("1, 3"), Some(vec!["web_cao", "web_3d"]));
        assert_eq!(interpreter("CAO;cao antenne"), Some(vec!["web_cao", "web_antenna"]));
        assert_eq!(interpreter("2 tous").map(|i| i.len()), Some(3));
        assert_eq!(interpreter(" 0 "), Some(vec![]));
        assert_eq!(interpreter(""), Some(vec![]));
        assert_eq!(interpreter("1 5"), None);
    }
}
