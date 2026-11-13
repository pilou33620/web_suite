//! Commun aux web tools (web_suite, WEB_CAO, WEB_ANTENNA, WEB_3D) : une seule
//! version, prise en dépendance git par chaque outil (plus de copies à comparer).
pub mod appairage;
pub mod pilote;
pub mod serveur;

use std::path::Path;

/// Un outil porté en Rust n'a plus son script Python : son demarrer_<NOM>.sh/.cmd obtient
/// l'exécutable (compilé ou téléchargé) puis le lance avec les arguments reçus.
/// Rend la tête de la commande de lancement, ou rien tant que l'outil est en Python.
pub fn demarreur(dossier_outil: &Path, nom: &str, script: &str) -> Option<Vec<String>> {
    if dossier_outil.join(script).is_file() {
        return None;
    }
    let d = dossier_outil.join(format!("demarrer_{}.{}", nom, if cfg!(windows) { "cmd" } else { "sh" }));
    let chemin = d.is_file().then(|| d.to_string_lossy().into_owned())?;
    Some(if cfg!(windows) { vec![chemin] } else { vec!["sh".into(), chemin] })
}
