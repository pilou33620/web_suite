//! Ce que banc_serveur.py vérifiait dans outils.py sans passer par le serveur.
use super::*;

fn tmp(nom: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("outils-essai-{}-{}", nom, std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

// Journaux : un journal trop gros passe en .log.1 au lancement suivant.
#[test]
fn journal_tourne() {
    let d = tmp("journal");
    let journal = d.join("web_3d.log");
    let ancien = d.join("web_3d.log.1");
    fs::write(&journal, vec![b'a'; JOURNAL_MAX as usize + 1]).unwrap();
    tourner(&journal);
    assert_eq!((journal.exists(), ancien.exists()), (false, true), "journal tourné");
    fs::write(&journal, "petit").unwrap();
    tourner(&journal);
    assert!(journal.exists(), "petit journal gardé");
    let _ = fs::remove_dir_all(&d);
}

// Hors Windows (Pi), l'installation d'openEMS est refusée : pas de bouton.
#[cfg(not(windows))]
#[test]
fn pas_de_bouton_openems_hors_windows() {
    assert_eq!(openems_pret(par_id("web_antenna").unwrap(), &crate::catalogue::depot()), None);
}

// openEMS : une archive dont l'empreinte n'est pas la bonne n'est pas extraite.
#[test]
fn archive_openems_refusee() {
    let mut fausse = io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut fausse);
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        z.start_file("openEMS/CSXCAD.dll", options).unwrap();
        z.write_all(b"pas la vraie").unwrap();
        z.finish().unwrap();
    }
    let fausse = fausse.into_inner();
    // un faux GitHub : il sert l'archive une fois
    let ecoute = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = ecoute.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let (mut c, _) = ecoute.accept().unwrap();
        let mut demande = [0u8; 4096];
        let _ = c.read(&mut demande);
        let _ = write!(c, "HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n", fausse.len());
        let _ = c.write_all(&fausse);
    });
    let d = tmp("openems");
    let cible = d.join("openEMS");
    let lignes = Mutex::new(Vec::new());
    let fait = telecharger_openems(&ureq::agent(), &format!("http://127.0.0.1:{}/openEMS.zip", port), &cible, &|t| {
        lignes.lock().unwrap().push(t.to_string())
    });
    assert!(!fait, "archive refusée");
    assert!(lignes.lock().unwrap().iter().any(|l| l.contains("empreinte SHA-256")), "{:?}", lignes);
    assert_eq!(fs::read_dir(&d).unwrap().count(), 0, "rien d'extrait ni de .part laissé");
    let _ = fs::remove_dir_all(&d);
}

// Une entrée qui sort du dossier (« ../ ») n'est pas extraite.
#[test]
fn extraction_confinee() {
    let mut archive = io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut archive);
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for nom in ["o/dedans/a.txt", "o/../../dehors.txt", "o/x/../b.txt", "autre/c.txt"] {
            z.start_file(nom, options).unwrap();
            z.write_all(b"x").unwrap();
        }
        z.finish().unwrap();
    }
    let d = tmp("zip");
    let cible = d.join("cible");
    archive.set_position(0);
    extraire(&mut zip::ZipArchive::new(archive).unwrap(), "o/", &cible).unwrap();
    assert!(cible.join("dedans/a.txt").is_file() && cible.join("b.txt").is_file());
    assert!(!d.join("dehors.txt").exists() && !cible.join("c.txt").exists());
    let _ = fs::remove_dir_all(&d);
}

// Le code d'appairage d'un outil : lu dans son journal, le dernier depuis le lancement, 10 min
#[test]
fn code_appairage_du_journal() {
    let d = tmp("code");
    let journal = d.join("x.log");
    fs::write(
        &journal,
        "  Code d'appairage WEB_CAO : 111 111\n=== lancement\n  \
         Code d'appairage WEB_CAO : 222 222   (à saisir…)\n  Code d'appairage WEB_CAO : 333 333\n",
    )
    .unwrap();
    let mut e = Etat { phase: Some("actif"), ..Etat::default() };
    assert_eq!(code_du_journal(&journal, &mut e).as_deref(), Some("333 333"), "code : le dernier depuis le lancement");
    e.code = Some(("333 333".to_string(), 0.0)); // vu il y a très longtemps
    assert_eq!(code_du_journal(&journal, &mut e), None, "code expiré : plus montré");
    assert_eq!(code_du_journal(&journal, &mut Etat::default()), None, "outil arrêté : pas de code");
    let mut e = Etat { phase: Some("actif"), ..Etat::default() };
    assert_eq!(code_du_journal(&d.join("absent.log"), &mut e), None, "pas de journal : pas de code");
    let _ = fs::remove_dir_all(&d);
}
