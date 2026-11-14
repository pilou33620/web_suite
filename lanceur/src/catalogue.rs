//! Le catalogue des outils et les emplacements sur le disque.
//!
//!     WEB_TOOLS/         <- ce dépôt (installer.cmd, demarrer_WEB_SUITE.cmd)
//!     ├── lanceur/       <- pages du lanceur, journaux/
//!     ├── WEB_CAO/       <- un dépôt git par outil, cloné ici
//!     ├── WEB_ANTENNA/
//!     └── WEB_3D/

use std::path::{Path, PathBuf};

pub const PORT_DEFAUT: u16 = 8100; // le port du lanceur

pub struct Outil {
    pub id: &'static str,
    pub nom: &'static str,
    pub sous_titre: &'static str,
    pub couleur: &'static str,
    pub depot: &'static str,
    pub port: u16,
    /// En-tête maison exigé sur tout POST/PUT/DELETE (REGLES §2.3).
    pub entete: &'static str,
    pub args: &'static [&'static str],
    /// "openems" (les exécutables openEMS et nf2ff, pour les simulations) ou rien.
    pub dependances: Option<&'static str>,
}

pub const CATALOGUE: &[Outil] = &[
    Outil {
        id: "web_cao",
        nom: "WEB_CAO",
        sous_titre: "Schéma & routage PCB",
        couleur: "#ff9a4d",
        depot: "https://github.com/pilou33620/WEB_CAO.git",
        port: 8000,
        entete: "X-WEB_CAO",
        args: &["--local", "--sans-navigateur", "--sans-pause"],
        dependances: None,
    },
    Outil {
        id: "web_antenna",
        nom: "WEB_ANTENNA",
        sous_titre: "Simulation d’antennes RF",
        couleur: "#4fe0ff",
        depot: "https://github.com/pilou33620/WEB_ANTENNA.git",
        port: 8732,
        entete: "X-WEB_ANTENNA",
        args: &["--local", "--sans-navigateur"],
        dependances: Some("openems"),
    },
    Outil {
        id: "web_3d",
        nom: "WEB_3D",
        sous_titre: "Modélisation & assemblage 3D",
        couleur: "#8dffb8",
        depot: "https://github.com/pilou33620/WEB_3D.git",
        port: 8139,
        entete: "X-Web3D",
        args: &["--local", "--sans-navigateur", "--sans-pause"],
        dependances: None,
    },
];

pub fn par_id(id: &str) -> Option<&'static Outil> {
    CATALOGUE.iter().find(|o| o.id == id)
}

/// La racine du dépôt git WEB_SUITE (là où vivent lanceur/, les outils, PROJETS/).
/// L'exécutable n'est pas à côté des pages : on la donne
/// par WEB_SUITE_DEPOT (scripts demarrer_*), sinon on remonte depuis l'exécutable
/// puis depuis le dossier courant jusqu'à trouver lanceur/index.html.
pub fn depot() -> PathBuf {
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

/// lanceur/ : les pages (index.html, appairage.html…) et journaux/. L'ancien ICI.
pub fn ici() -> PathBuf {
    depot().join("lanceur")
}

pub fn dossier(outil: &Outil, racine: &Path) -> PathBuf {
    racine.join(outil.nom)
}

pub use web_tools_commun::demarreur;

pub fn est_installe(outil: &Outil, racine: &Path) -> bool {
    demarreur(&dossier(outil, racine), outil.nom).is_some()
}
