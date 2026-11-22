//! WEB·SUITE sans git : la mise à jour d'une copie de production, et git portable sous Windows.
//!
//! Une copie de production (archive `*-prod-<système>` ou l'archive complète WEB_SUITE) n'a ni
//! `.git` ni sources. Elle se met à jour depuis la publication « binaires » de son dépôt :
//! l'empreinte publiée de l'archive est comparée à celle de l'installation (`bin/prod.sha256`),
//! et si elle a changé, l'archive est téléchargée, vérifiée, puis recopiée par-dessus. Seuls les
//! fichiers de l'archive sont écrits : PROJETS/, openEMS/, réglages et projets restent tels quels.
//!
//! Sous Windows sans git, le lanceur pose Git for Windows portable dans `<racine>/git/` (rien
//! d'installé dans le système) et le met en tête du PATH : la synchronisation de PROJETS et la
//! connexion GitHub (Git Credential Manager, fourni avec) marchent alors comme avec un git installé.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use web_tools_commun::production::{self, client, telecharger, Maj};

use crate::processus::{quel, sans_fenetre, WINDOWS};

pub use web_tools_commun::production::est_copie_prod;

pub type Ecrire<'a> = &'a dyn Fn(&str);

/// Met à jour une copie de production depuis la publication « binaires » de `depot`
/// (https://github.com/<compte>/<dépôt>), archive `<nom>-prod-<système>` : voir
/// web_tools_commun::production.
///
/// Renvoie les mêmes statuts que outils::maj_depot : 'maj', 'a_jour', 'hors_ligne', 'echec'.
pub fn maj_archive(dossier: &Path, depot: &str, nom: &str, ecrire: Ecrire) -> &'static str {
    match production::maj_archive(dossier, depot, nom, ecrire) {
        Ok(Maj::MisAJour) => "maj",
        Ok(Maj::AJour) => "a_jour",
        Ok(Maj::HorsLigne) => "hors_ligne",
        Err(e) => {
            ecrire(&format!("[X] {}", e));
            "echec"
        }
    }
}

/// Le dossier de git portable : <racine>/git/cmd (git.exe).
fn git_portable(racine: &Path) -> PathBuf {
    racine.join("git").join("cmd")
}

fn mettre_en_tete_du_path(dossier: &Path) {
    let reste = std::env::var_os("PATH").unwrap_or_default();
    let tout = std::env::join_paths(std::iter::once(dossier.to_path_buf()).chain(std::env::split_paths(&reste)));
    if let Ok(tout) = tout {
        // au démarrage, avant tout fil : les outils lancés ensuite en héritent
        std::env::set_var("PATH", tout);
    }
}

/// Windows sans git : pose Git for Windows portable dans <racine>/git/ (une fois), et le met
/// en tête du PATH. Ailleurs, ou git déjà là : rien.
pub fn preparer_git(racine: &Path, ecrire: Ecrire) {
    if !WINDOWS || quel("git").is_some() {
        return;
    }
    let cmd = git_portable(racine);
    if !cmd.join("git.exe").is_file() {
        ecrire("git introuvable : installation de Git portable dans git/ (~60 Mo, une seule fois)…");
        if let Err(e) = installer_git_portable(racine) {
            ecrire(&format!("[X] Git portable n'a pas pu être installé : {}", e));
            ecrire("    PROJETS ne sera pas synchronisé. Relancez plus tard, ou installez Git.");
            return;
        }
        ecrire("[OK] Git portable installé.");
    }
    mettre_en_tete_du_path(&cmd);
}

/// La dernière Git for Windows portable 64 bits, vérifiée par l'empreinte que GitHub publie
/// avec elle (champ `digest` de la publication), puis décompressée (archive 7-Zip autoextractible).
fn installer_git_portable(racine: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let texte = client()
        .get("https://api.github.com/repos/git-for-windows/git/releases/latest")
        .set("User-Agent", "web_suite")
        .call()?
        .into_string()?;
    let publication: serde_json::Value = serde_json::from_str(&texte)?;
    let actif = publication["assets"]
        .as_array()
        .and_then(|l| {
            l.iter().find(|a| {
                let n = a["name"].as_str().unwrap_or("");
                n.starts_with("PortableGit-") && n.ends_with("-64-bit.7z.exe")
            })
        })
        .ok_or("pas de PortableGit 64 bits dans la dernière publication")?;
    let url = actif["browser_download_url"].as_str().ok_or("adresse manquante")?;
    let attendu = actif["digest"].as_str().and_then(|d| d.strip_prefix("sha256:")).ok_or("empreinte non publiée")?;
    let exe = racine.join("PortableGit.7z.exe");
    let recu = telecharger(url, &exe)?;
    if recu != attendu.to_lowercase() {
        let _ = fs::remove_file(&exe);
        return Err("empreinte de l'archive incorrecte (téléchargement tronqué ?)".into());
    }
    let cible = racine.join("git");
    let mut c = Command::new(&exe);
    c.arg(format!("-o{}", cible.display())).arg("-y");
    let ok = sans_fenetre(&mut c).status()?.success();
    let _ = fs::remove_file(&exe);
    if !ok || !git_portable(racine).join("git.exe").is_file() {
        return Err("décompression échouée".into());
    }
    Ok(())
}
