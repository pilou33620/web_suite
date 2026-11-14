//! Connexion à GitHub dans le terminal du serveur (voir connexion_github.rs).
use web_suite_lanceur::{catalogue::depot, connexion_github::connecter};

fn main() {
    if std::env::args().any(|a| a == "--connecter") {
        connecter(&depot());
        return;
    }
    println!("usage: connexion_github [-h] --connecter\n");
    println!("  --connecter  se connecter à GitHub dans ce terminal, puis récupérer PROJETS");
    println!("  -h, --help   afficher cette aide");
    println!("\nLa vérification sans réseau de l'ancien script est un test : cargo test -p web_suite_lanceur");
}
