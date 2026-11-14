//! WEB·SUITE : le lanceur (voir serveur_lanceur.rs).
use web_suite_lanceur::catalogue::{depot, ici};

fn main() {
    // Comme l'ancien web_suite.py, on travaille depuis lanceur/ ; la racine du dépôt
    // est fixée avant, pour qu'un WEB_SUITE_DEPOT relatif reste juste.
    let racine = depot();
    std::env::set_var("WEB_SUITE_DEPOT", std::path::absolute(&racine).unwrap_or(racine));
    let _ = std::env::set_current_dir(ici());
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(web_suite_lanceur::serveur_lanceur::principal(&args));
}
