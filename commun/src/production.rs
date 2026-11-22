//! La mise à jour sans git d'une copie de production, commune à tous les web tools.
//!
//! Une copie de production (archive `<nom>-prod-<système>`) n'a ni `.git` ni sources. Elle se
//! met à jour depuis la publication « binaires » de son dépôt : l'empreinte publiée de l'archive
//! (`<archive>.sha256`) est comparée à celle de l'installation (`bin/prod.sha256`) ; si elle a
//! changé, l'archive est téléchargée, vérifiée (SHA-256), décompressée à côté, puis recopiée
//! par-dessus. Seuls les fichiers de l'archive sont écrits : données, réglages et projets restent.
//!
//! Une mise à jour interrompue ne casse rien : tant que l'archive n'est pas vérifiée et
//! décompressée en entier, l'installation n'est pas touchée ; chaque fichier est ensuite remplacé
//! d'un seul geste (renommage), et `bin/prod.sha256` n'est écrit qu'à la fin, si bien qu'une
//! recopie coupée en route est refaite au lancement suivant.
//!
//! Usage dans un outil (au démarrage, avant le serveur) :
//! ```text
//! match production::mettre_a_jour(DEPOT, "mon_outil", &racine, &|t| println!("{t}")) {
//!     Ok(true) => std::process::exit(production::CODE_MIS_A_JOUR), // demarrer_<NOM> relance
//!     Ok(false) => {}
//!     Err(e) => println!("[X] {e}"), // l'installation actuelle est intacte : on démarre
//! }
//! ```
//!
//! Feature cargo `production` (ureq, zip) : les outils qui n'en veulent pas ne les tirent pas.

use std::fs;
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use sha2::{Digest, Sha256};

/// Le code de sortie d'un outil qui vient de se mettre à jour : demarrer_<NOM> le relance.
pub const CODE_MIS_A_JOUR: i32 = 75;

const EMPREINTE_LOCALE: &str = "prod.sha256"; // dans bin/

/// Ce que la vérification a trouvé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Maj {
    MisAJour,
    AJour,
    HorsLigne,
}

/// Le client HTTP(S) des mises à jour : rustls et webpki-roots (pas de bibliothèque C, musl).
pub fn client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(60))
        .build()
}

/// Le système de ce poste dans les noms d'archives : (système-processeur, extension).
pub fn systeme() -> Option<(&'static str, &'static str)> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Some(("windows-x86_64", "zip")),
        ("linux", "x86_64") => Some(("linux-x86_64", "tar.gz")),
        ("linux", "aarch64") => Some(("linux-aarch64", "tar.gz")),
        ("macos", "aarch64") => Some(("macos-aarch64", "tar.gz")),
        _ => None,
    }
}

/// Le nom de l'archive de production de `nom` pour ce poste : `<nom>-prod-<système>.<ext>`.
pub fn nom_archive(nom: &str) -> Option<String> {
    systeme().map(|(sys, ext)| format!("{}-prod-{}.{}", nom, sys, ext))
}

/// Une copie de production : des exécutables dans bin/, ni .git ni sources (Cargo.toml).
pub fn est_copie_prod(racine: &Path) -> bool {
    !racine.join(".git").exists() && !racine.join("Cargo.toml").exists() && racine.join("bin").is_dir()
}

/// L'empreinte publiée à `url` (un fichier `.sha256`), en minuscules.
pub fn empreinte_publiee(url: &str) -> Result<String, String> {
    let texte = client().get(url).call().map_err(|e| e.to_string())?.into_string().map_err(|e| e.to_string())?;
    // « <hexa> » seul, ou « <hexa>  <fichier> » comme sha256sum
    Ok(texte.split_whitespace().next().unwrap_or("").to_lowercase())
}

/// Vrai si l'installation porte déjà l'empreinte `publiee` (bin/prod.sha256).
pub fn a_jour(racine: &Path, publiee: &str) -> bool {
    let locale = fs::read_to_string(racine.join("bin").join(EMPREINTE_LOCALE)).unwrap_or_default();
    let locale = locale.split_whitespace().next().unwrap_or("").to_lowercase();
    !locale.is_empty() && locale == publiee.trim().to_lowercase()
}

fn hexa(e: Sha256) -> String {
    e.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

/// Télécharge `url` dans `fichier` et rend son empreinte SHA-256.
pub fn telecharger(url: &str, fichier: &Path) -> Result<String, String> {
    let essai = || -> Result<String, Box<dyn std::error::Error>> {
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
        f.sync_all()?; // disque plein : l'erreur arrive ici, pas après
        Ok(hexa(empreinte))
    };
    essai().map_err(|e| e.to_string())
}

/// Extrait les entrées de `z` qui commencent par `prefixe` dans `cible`, sans le préfixe.
/// Une entrée qui sortirait de `cible` (« ../ », chemin absolu, lecteur Windows) est ignorée.
pub fn extraire<R: Read + Seek>(z: &mut zip::ZipArchive<R>, prefixe: &str, cible: &Path) -> io::Result<()> {
    let windows = cfg!(windows);
    for i in 0..z.len() {
        let mut entree = z.by_index(i)?;
        let nom = entree.name().to_string();
        let Some(relatif) = nom.strip_prefix(prefixe).filter(|r| !r.is_empty()) else { continue };
        // Le chemin remis d'aplomb (normpath) doit rester sous `cible`.
        let mut morceaux: Vec<&str> = Vec::new();
        let mut dehors = relatif.starts_with('/');
        for m in relatif.split(|c| c == '/' || (windows && c == '\\')) {
            match m {
                "" | "." => {}
                ".." => dehors |= morceaux.pop().is_none(),
                _ => {
                    dehors |= windows && m.contains(':');
                    morceaux.push(m);
                }
            }
        }
        if dehors || morceaux.is_empty() {
            continue; // entrée hors du dossier : ignorée
        }
        let chemin = morceaux.iter().fold(cible.to_path_buf(), |p, m| p.join(m));
        if nom.ends_with('/') {
            fs::create_dir_all(&chemin)?;
        } else {
            if let Some(parent) = chemin.parent() {
                fs::create_dir_all(parent)?;
            }
            io::copy(&mut entree, &mut fs::File::create(&chemin)?)?;
        }
    }
    Ok(())
}

/// Décompresse l'archive (zip ou tar.gz) dans `vers`, sans son dossier de tête (mon_outil/…).
pub fn decompresser(archive: &Path, vers: &Path) -> Result<(), String> {
    let essai = || -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(vers)?;
        if archive.to_string_lossy().ends_with(".zip") {
            let mut z = zip::ZipArchive::new(fs::File::open(archive)?)?;
            let tete = z.name_for_index(0).unwrap_or("").split('/').next().unwrap_or("").to_string();
            extraire(&mut z, &format!("{}/", tete), vers)?;
        } else {
            // tar.gz : hors Windows seulement (Linux, macOS), où tar est toujours là
            let ok = Command::new("tar")
                .arg("-xzf")
                .arg(archive)
                .arg("--strip-components=1")
                .arg("-C")
                .arg(vers)
                .status()?
                .success();
            if !ok {
                return Err("tar n'a pas pu décompresser l'archive".into());
            }
        }
        Ok(())
    };
    essai().map_err(|e| e.to_string())
}

/// Remplace `vers` par `de` d'un seul geste. Windows refuse de remplacer un exécutable en cours
/// d'usage (l'outil lui-même) mais accepte de le renommer : l'ancien passe alors en `.ancien`,
/// et revient à sa place si le nouveau ne peut pas être posé (jamais de trou).
fn remplacer(de: &Path, vers: &Path) -> io::Result<()> {
    let ancien = PathBuf::from(format!("{}.ancien", vers.display()));
    let _ = fs::remove_file(&ancien); // celui de la fois d'avant, s'il n'est plus en usage
    let erreur = match fs::rename(de, vers) {
        Ok(()) => return Ok(()),
        Err(e) => e,
    };
    if !vers.is_file() {
        return Err(erreur);
    }
    fs::rename(vers, &ancien)?;
    if let Err(e) = fs::rename(de, vers) {
        let _ = fs::rename(&ancien, vers);
        return Err(e);
    }
    let _ = fs::remove_file(&ancien); // en usage (Windows) : il part à la mise à jour suivante
    Ok(())
}

/// Recopie l'arbre `source` dans `cible`, fichier par fichier (renommages : même disque).
/// Les fichiers de `cible` absents de `source` ne sont pas touchés.
pub fn recopier(source: &Path, cible: &Path) -> io::Result<()> {
    fs::create_dir_all(cible)?;
    for entree in fs::read_dir(source)? {
        let entree = entree?;
        let (de, vers) = (entree.path(), cible.join(entree.file_name()));
        if entree.file_type()?.is_dir() {
            recopier(&de, &vers)?;
        } else {
            remplacer(&de, &vers)?;
        }
    }
    Ok(())
}

/// Vérifie et applique la mise à jour de la copie de production `racine` depuis la publication
/// « binaires » de `depot` (https://github.com/<compte>/<dépôt>), archive `<nom>-prod-<système>`.
/// Ne regarde pas si `racine` est une copie de production : voir `mettre_a_jour`.
///
/// Err : la mise à jour a échoué, l'installation actuelle est intacte (message prêt à afficher,
/// non écrit par `ecrire`).
pub fn maj_archive(racine: &Path, depot: &str, nom: &str, ecrire: &dyn Fn(&str)) -> Result<Maj, String> {
    let Some(archive) = nom_archive(nom) else {
        ecrire("Pas d'archive publiée pour ce système : mises à jour non vérifiées.");
        return Ok(Maj::AJour);
    };
    let url = format!("{}/releases/download/binaires/{}", depot.trim_end_matches('/'), archive);
    let publiee = match empreinte_publiee(&format!("{}.sha256", url)) {
        Ok(e) if !e.is_empty() => e,
        _ => {
            ecrire("GitHub injoignable : mise à jour ignorée.");
            return Ok(Maj::HorsLigne);
        }
    };
    if a_jour(racine, &publiee) {
        ecrire("À jour.");
        return Ok(Maj::AJour);
    }
    ecrire("Nouvelle version publiée, téléchargement…");
    let ext = if archive.ends_with(".zip") { "zip" } else { "tar.gz" };
    let (part, neuf) = (racine.join(format!(".maj.{}", ext)), racine.join(".maj"));
    let _ = fs::remove_dir_all(&neuf); // les restes d'une mise à jour interrompue
    let essai = || -> Result<(), String> {
        if telecharger(&url, &part)? != publiee {
            return Err("Empreinte de l'archive incorrecte (téléchargement tronqué ?) : on garde la version actuelle.".into());
        }
        decompresser(&part, &neuf)?;
        if !neuf.join("bin").is_dir() {
            return Err("Archive inattendue (pas de bin/) : on garde la version actuelle.".into());
        }
        recopier(&neuf, racine).map_err(|e| e.to_string())?;
        // à la fin seulement : une recopie interrompue sera refaite au lancement suivant
        fs::write(racine.join("bin").join(EMPREINTE_LOCALE), format!("{}\n", publiee)).map_err(|e| e.to_string())?;
        Ok(())
    };
    let fait = essai().map_err(|e| if e.starts_with("Empreinte") || e.starts_with("Archive") { e } else { format!("Mise à jour impossible : {}", e) });
    let _ = fs::remove_file(&part);
    let _ = fs::remove_dir_all(&neuf);
    fait?;
    ecrire("Mis à jour.");
    Ok(Maj::MisAJour)
}

/// La mise à jour au démarrage d'un outil. Ok(true) : mis à jour, l'outil sort avec
/// `CODE_MIS_A_JOUR` (75) pour que demarrer_<NOM> relance la nouvelle version. Ok(false) : rien à
/// faire (à jour, hors ligne, système sans archive, ou pas une copie de production : clone git,
/// sources). Err : échec, l'installation actuelle est intacte et peut démarrer.
pub fn mettre_a_jour(depot: &str, nom_archive: &str, racine: &Path, ecrire: &dyn Fn(&str)) -> Result<bool, String> {
    if !est_copie_prod(racine) {
        return Ok(false);
    }
    maj_archive(racine, depot, nom_archive, ecrire).map(|m| m == Maj::MisAJour)
}

#[cfg(test)]
mod essais {
    use super::*;
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    fn tmp(nom: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("commun-production-{}-{}", nom, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// Une copie de production : bin/outil (« 1 ») et une donnée PROJETS/carte.json.
    fn installation(d: &Path) -> PathBuf {
        let copie = d.join("copie");
        fs::create_dir_all(copie.join("bin")).unwrap();
        fs::create_dir_all(copie.join("PROJETS")).unwrap();
        fs::write(copie.join("bin").join("outil"), "1").unwrap();
        fs::write(copie.join("PROJETS").join("carte.json"), "à moi").unwrap();
        copie
    }

    /// Un faux serveur local (jamais le réseau) : sert `fichiers` (chemin, corps), 404 sinon,
    /// et note les chemins demandés.
    fn faux_serveur(fichiers: Vec<(String, Vec<u8>)>) -> (String, Arc<Mutex<Vec<String>>>) {
        let ecoute = TcpListener::bind("127.0.0.1:0").unwrap();
        let adresse = format!("http://{}", ecoute.local_addr().unwrap());
        let demandes = Arc::new(Mutex::new(Vec::new()));
        let notes = demandes.clone();
        std::thread::spawn(move || {
            for flux in ecoute.incoming() {
                let Ok(mut flux) = flux else { continue };
                let (mut tete, mut octet) = (Vec::new(), [0u8; 1]);
                while !tete.ends_with(b"\r\n\r\n") && matches!(flux.read(&mut octet), Ok(1)) {
                    tete.push(octet[0]);
                }
                let chemin = String::from_utf8_lossy(&tete).split_whitespace().nth(1).unwrap_or("").to_string();
                notes.lock().unwrap().push(chemin.clone());
                let reponse = match fichiers.iter().find(|(c, _)| *c == chemin) {
                    Some((_, corps)) => {
                        let tete = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", corps.len());
                        [tete.into_bytes(), corps.clone()].concat()
                    }
                    None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
                };
                let _ = flux.write_all(&reponse);
            }
        });
        (adresse, demandes)
    }

    /// L'archive de production de « outil » au format de ce poste : outil/bin/outil (« 2 »),
    /// outil/index.html.
    fn archive(d: &Path) -> Vec<u8> {
        let (_, ext) = systeme().unwrap();
        let fichier = d.join(format!("archive.{}", ext));
        if ext == "zip" {
            let mut z = zip::ZipWriter::new(fs::File::create(&fichier).unwrap());
            let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            for (nom, corps) in [("outil/bin/outil", "2"), ("outil/index.html", "neuf")] {
                z.start_file(nom, options).unwrap();
                z.write_all(corps.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        } else {
            let source = d.join("source").join("outil");
            fs::create_dir_all(source.join("bin")).unwrap();
            fs::write(source.join("bin").join("outil"), "2").unwrap();
            fs::write(source.join("index.html"), "neuf").unwrap();
            let ok = Command::new("tar").arg("-czf").arg(&fichier).arg("-C").arg(d.join("source")).arg("outil").status();
            assert!(ok.unwrap().success());
        }
        fs::read(&fichier).unwrap()
    }

    fn empreinte(octets: &[u8]) -> String {
        let mut e = Sha256::new();
        e.update(octets);
        hexa(e)
    }

    const CHEMIN: &str = "/compte/outil/releases/download/binaires/";

    /// La recopie remplace les fichiers de l'archive, crée les nouveaux, et ne touche pas au reste.
    #[test]
    fn recopie_par_dessus() {
        let d = tmp("recopie");
        let copie = installation(&d);
        let neuf = d.join("neuf");
        fs::create_dir_all(neuf.join("bin")).unwrap();
        fs::write(neuf.join("bin").join("outil"), "2").unwrap();
        fs::write(neuf.join("index.html"), "neuf").unwrap();
        assert!(est_copie_prod(&copie));
        recopier(&neuf, &copie).unwrap();
        assert_eq!(fs::read_to_string(copie.join("bin").join("outil")).unwrap(), "2");
        assert_eq!(fs::read_to_string(copie.join("index.html")).unwrap(), "neuf");
        assert_eq!(fs::read_to_string(copie.join("PROJETS").join("carte.json")).unwrap(), "à moi");
        assert!(!copie.join("bin").join("outil.ancien").exists());
        fs::write(copie.join("Cargo.toml"), "").unwrap();
        assert!(!est_copie_prod(&copie)); // des sources : ce n'est plus une copie de production
        let _ = fs::remove_dir_all(&d);
    }

    /// Comparaison des empreintes : casse, blancs et nom de fichier de sha256sum ne comptent pas.
    #[test]
    fn comparaison_des_empreintes() {
        let d = tmp("comparaison");
        let copie = installation(&d);
        assert!(!a_jour(&copie, "abc"), "pas de bin/prod.sha256 : pas à jour");
        fs::write(copie.join("bin").join(EMPREINTE_LOCALE), "ABC\n").unwrap();
        assert!(a_jour(&copie, "abc"));
        assert!(a_jour(&copie, " abc \n"));
        assert!(!a_jour(&copie, "abd"));
        assert!(!a_jour(&copie, ""));
        let _ = fs::remove_dir_all(&d);
    }

    /// Déjà à jour : rien n'est téléchargé, rien n'est touché.
    #[test]
    fn deja_a_jour() {
        let Some(nom) = nom_archive("outil") else { return };
        let d = tmp("a_jour");
        let copie = installation(&d);
        fs::write(copie.join("bin").join(EMPREINTE_LOCALE), "abc123\n").unwrap();
        let sha = format!("{}{}.sha256", CHEMIN, nom);
        let (adresse, demandes) = faux_serveur(vec![(sha.clone(), b"ABC123  outil.tar.gz\n".to_vec())]);
        let depot = format!("{}/compte/outil", adresse);
        assert_eq!(mettre_a_jour(&depot, "outil", &copie, &|_| {}), Ok(false));
        assert_eq!(*demandes.lock().unwrap(), vec![sha]);
        assert_eq!(fs::read_to_string(copie.join("bin").join("outil")).unwrap(), "1");
        let _ = fs::remove_dir_all(&d);
    }

    /// Nouvelle version : téléchargée, vérifiée, recopiée par-dessus ; la donnée reste.
    #[test]
    fn mise_a_jour_par_dessus() {
        let Some(nom) = nom_archive("outil") else { return };
        let d = tmp("maj");
        let copie = installation(&d);
        let octets = archive(&d);
        let attendu = empreinte(&octets);
        let (adresse, _) = faux_serveur(vec![
            (format!("{}{}.sha256", CHEMIN, nom), format!("{}\n", attendu).into_bytes()),
            (format!("{}{}", CHEMIN, nom), octets),
        ]);
        let depot = format!("{}/compte/outil", adresse);
        assert_eq!(mettre_a_jour(&depot, "outil", &copie, &|_| {}), Ok(true));
        assert_eq!(fs::read_to_string(copie.join("bin").join("outil")).unwrap(), "2");
        assert_eq!(fs::read_to_string(copie.join("index.html")).unwrap(), "neuf");
        assert_eq!(fs::read_to_string(copie.join("PROJETS").join("carte.json")).unwrap(), "à moi");
        assert!(a_jour(&copie, &attendu));
        assert!(!copie.join(".maj").exists() && !copie.join(".maj.zip").exists() && !copie.join(".maj.tar.gz").exists());
        // la fois suivante : à jour
        assert_eq!(mettre_a_jour(&depot, "outil", &copie, &|_| {}), Ok(false));
        // un clone de sources n'est jamais écrasé
        fs::write(copie.join("Cargo.toml"), "").unwrap();
        fs::remove_file(copie.join("bin").join(EMPREINTE_LOCALE)).unwrap();
        assert_eq!(mettre_a_jour(&depot, "outil", &copie, &|_| {}), Ok(false));
        assert!(!a_jour(&copie, &attendu));
        let _ = fs::remove_dir_all(&d);
    }

    /// Une archive dont l'empreinte est fausse est refusée : l'installation reste intacte.
    #[test]
    fn empreinte_fausse_refusee() {
        let Some(nom) = nom_archive("outil") else { return };
        let d = tmp("fausse");
        let copie = installation(&d);
        let octets = archive(&d);
        let (adresse, _) = faux_serveur(vec![
            (format!("{}{}.sha256", CHEMIN, nom), format!("{}\n", "0".repeat(64)).into_bytes()),
            (format!("{}{}", CHEMIN, nom), octets),
        ]);
        let depot = format!("{}/compte/outil", adresse);
        let resultat = mettre_a_jour(&depot, "outil", &copie, &|_| {});
        assert!(resultat.as_ref().is_err_and(|e| e.contains("Empreinte")), "{:?}", resultat);
        assert_eq!(fs::read_to_string(copie.join("bin").join("outil")).unwrap(), "1");
        assert!(!copie.join("index.html").exists());
        assert!(!copie.join("bin").join(EMPREINTE_LOCALE).exists());
        let restes: Vec<_> = fs::read_dir(&copie).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(restes.len(), 2, "ni .maj ni archive laissés : {:?}", restes);
        let _ = fs::remove_dir_all(&d);
    }

    /// Hors ligne : pas d'erreur, l'outil démarre tel quel.
    #[test]
    fn hors_ligne() {
        let d = tmp("hors_ligne");
        let copie = installation(&d);
        let (adresse, _) = faux_serveur(vec![]); // 404 partout
        let depot = format!("{}/compte/outil", adresse);
        assert_eq!(mettre_a_jour(&depot, "outil", &copie, &|_| {}), Ok(false));
        assert_eq!(fs::read_to_string(copie.join("bin").join("outil")).unwrap(), "1");
        let _ = fs::remove_dir_all(&d);
    }
}
