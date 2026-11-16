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
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::processus::{quel, sans_fenetre, WINDOWS};

pub type Ecrire<'a> = &'a dyn Fn(&str);

const EMPREINTE_LOCALE: &str = "prod.sha256"; // dans bin/

fn client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(60))
        .build()
}

/// Le nom de l'archive de ce poste dans les publications : (système-processeur, extension).
fn systeme() -> Option<(&'static str, &'static str)> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Some(("windows-x86_64", "zip")),
        ("linux", "x86_64") => Some(("linux-x86_64", "tar.gz")),
        ("linux", "aarch64") => Some(("linux-aarch64", "tar.gz")),
        ("macos", "aarch64") => Some(("macos-aarch64", "tar.gz")),
        _ => None,
    }
}

/// Une copie de production : des exécutables dans bin/, ni .git ni sources (Cargo.toml).
pub fn est_copie_prod(dossier: &Path) -> bool {
    !dossier.join(".git").exists() && !dossier.join("Cargo.toml").exists() && dossier.join("bin").is_dir()
}

fn hexa(e: Sha256) -> String {
    e.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

/// Télécharge `url` dans `fichier` et rend son empreinte SHA-256.
fn telecharger(url: &str, fichier: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut flux = client().get(url).call()?.into_reader();
    let mut f = fs::File::create(fichier)?;
    let mut empreinte = Sha256::new();
    let mut bloc = vec![0u8; 1 << 20];
    loop {
        let n = flux.read(&mut bloc)?;
        if n == 0 {
            break;
        }
        f.write_all(&bloc[..n])?;
        empreinte.update(&bloc[..n]);
    }
    Ok(hexa(empreinte))
}

/// Recopie l'arbre `source` dans `cible`, fichier par fichier.
///
/// Un fichier déjà là est d'abord renommé en `.ancien` : Windows refuse de réécrire un
/// exécutable en cours d'usage (le lanceur lui-même), mais accepte de le renommer.
pub fn recopier(source: &Path, cible: &Path) -> io::Result<()> {
    fs::create_dir_all(cible)?;
    for entree in fs::read_dir(source)? {
        let entree = entree?;
        let (de, vers) = (entree.path(), cible.join(entree.file_name()));
        if entree.file_type()?.is_dir() {
            recopier(&de, &vers)?;
            continue;
        }
        if vers.exists() {
            let ancien = PathBuf::from(format!("{}.ancien", vers.display()));
            let _ = fs::remove_file(&ancien); // celui de la fois d'avant, s'il n'est plus en usage
            fs::rename(&vers, &ancien)?;
            fs::rename(&de, &vers)?;
            let _ = fs::remove_file(&ancien); // en usage (Windows) : il part à la mise à jour suivante
        } else {
            fs::rename(&de, &vers)?;
        }
    }
    Ok(())
}

/// Décompresse l'archive dans `vers`, sans son dossier de tête (web_3d/, web_suite/…).
fn decompresser(archive: &Path, vers: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(vers)?;
    if archive.to_string_lossy().ends_with(".zip") {
        let mut z = zip::ZipArchive::new(fs::File::open(archive)?)?;
        let tete = z.name_for_index(0).unwrap_or("").split('/').next().unwrap_or("").to_string();
        crate::outils::extraire(&mut z, &format!("{}/", tete), vers)?;
    } else {
        let mut cmd = Command::new("tar");
        cmd.arg("-xzf").arg(archive).arg("--strip-components=1").arg("-C").arg(vers);
        if !sans_fenetre(&mut cmd).status()?.success() {
            return Err("tar n'a pas pu décompresser l'archive".into());
        }
    }
    Ok(())
}

/// Met à jour une copie de production depuis la publication « binaires » de `depot`
/// (https://github.com/<compte>/<dépôt>), archive `<nom>-prod-<système>`.
///
/// Renvoie les mêmes statuts que outils::maj_depot : 'maj', 'a_jour', 'hors_ligne', 'echec'.
pub fn maj_archive(dossier: &Path, depot: &str, nom: &str, ecrire: Ecrire) -> &'static str {
    let Some((sys, ext)) = systeme() else {
        ecrire("Pas d'archive publiée pour ce système : mises à jour non vérifiées.");
        return "a_jour";
    };
    let url = format!("{}/releases/download/binaires/{}-prod-{}.{}", depot, nom, sys, ext);
    let publiee = match client().get(&format!("{}.sha256", url)).call().map(|r| r.into_string()) {
        Ok(Ok(t)) => t.trim().to_lowercase(),
        _ => {
            ecrire("GitHub injoignable : mise à jour ignorée.");
            return "hors_ligne";
        }
    };
    let locale = dossier.join("bin").join(EMPREINTE_LOCALE);
    if fs::read_to_string(&locale).is_ok_and(|t| t.trim() == publiee) {
        ecrire("À jour.");
        return "a_jour";
    }
    ecrire("Nouvelle version publiée, téléchargement…");
    let (part, neuf) = (dossier.join(format!(".maj.{}", ext)), dossier.join(".maj"));
    let _ = fs::remove_dir_all(&neuf);
    let essai = || -> Result<bool, Box<dyn std::error::Error>> {
        if telecharger(&url, &part)? != publiee {
            ecrire("[X] Empreinte de l'archive incorrecte (téléchargement tronqué ?) : on garde la version actuelle.");
            return Ok(false);
        }
        decompresser(&part, &neuf)?;
        if !neuf.join("bin").is_dir() {
            ecrire("[X] Archive inattendue (pas de bin/) : on garde la version actuelle.");
            return Ok(false);
        }
        recopier(&neuf, dossier)?;
        fs::write(&locale, format!("{}\n", publiee))?;
        Ok(true)
    };
    let fait = essai().unwrap_or_else(|e| {
        ecrire(&format!("[X] Mise à jour impossible : {}", e));
        false
    });
    let _ = fs::remove_file(&part);
    let _ = fs::remove_dir_all(&neuf);
    if fait {
        ecrire("Mis à jour.");
        "maj"
    } else {
        "echec"
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

#[cfg(test)]
mod essais {
    use super::*;

    /// La recopie remplace les fichiers de l'archive, crée les nouveaux, et ne touche pas au reste.
    #[test]
    fn recopie_par_dessus() {
        let tmp = std::env::temp_dir().join(format!("websuite-sans-git-{}", std::process::id()));
        let (neuf, copie) = (tmp.join("neuf"), tmp.join("copie"));
        fs::create_dir_all(neuf.join("bin")).unwrap();
        fs::create_dir_all(copie.join("bin")).unwrap();
        fs::create_dir_all(copie.join("PROJETS")).unwrap();
        fs::write(neuf.join("bin").join("outil"), "2").unwrap();
        fs::write(neuf.join("index.html"), "neuf").unwrap();
        fs::write(copie.join("bin").join("outil"), "1").unwrap();
        fs::write(copie.join("PROJETS").join("carte.json"), "à moi").unwrap();
        assert!(est_copie_prod(&copie));
        recopier(&neuf, &copie).unwrap();
        assert_eq!(fs::read_to_string(copie.join("bin").join("outil")).unwrap(), "2");
        assert_eq!(fs::read_to_string(copie.join("index.html")).unwrap(), "neuf");
        assert_eq!(fs::read_to_string(copie.join("PROJETS").join("carte.json")).unwrap(), "à moi");
        assert!(!copie.join("bin").join("outil.ancien").exists());
        fs::write(copie.join("Cargo.toml"), "").unwrap();
        assert!(!est_copie_prod(&copie)); // des sources : ce n'est plus une copie de production
        fs::remove_dir_all(&tmp).unwrap();
    }
}
