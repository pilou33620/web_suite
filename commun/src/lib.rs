//! Commun aux web tools (web_suite, WEB_CAO, WEB_ANTENNA, WEB_3D) : une seule
//! version, prise en dépendance git par chaque outil (plus de copies à comparer).
pub mod appairage;
pub mod pilote;
pub mod serveur;

use std::path::Path;

/// La tête de la commande qui lance un outil : son demarrer_<NOM>.sh/.cmd, qui obtient
/// l'exécutable (compilé ou téléchargé) puis le lance avec les arguments reçus.
/// Rien si l'outil n'est pas installé dans ce dossier.
pub fn demarreur(dossier_outil: &Path, nom: &str) -> Option<Vec<String>> {
    let d = dossier_outil.join(format!("demarrer_{}.{}", nom, if cfg!(windows) { "cmd" } else { "sh" }));
    let chemin = d.is_file().then(|| d.to_string_lossy().into_owned())?;
    Some(if cfg!(windows) { vec![chemin] } else { vec!["sh".into(), chemin] })
}
