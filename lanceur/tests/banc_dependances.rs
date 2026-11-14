//! Banc de l'installateur devant un outil déjà là.
//!
//!     cargo test -p web_suite_lanceur --test banc_dependances
//!
//! Un outil est installé dès que son demarrer_<NOM> est là : rien d'autre n'est demandé
//! (plus de paquets à installer), lancer un outil absent est refusé, et openEMS ne pose aucune
//! question hors Windows. Rien n'est téléchargé : les exécutables sont pilotés en boîte noire.
mod commun;

use std::fs;

use commun::{chaine, Poste, INSTALLER};

#[test]
fn banc() {
    let poste = Poste::new("dependances");
    let racine = chaine(&poste.racine);
    let ext = if cfg!(windows) { "cmd" } else { "sh" };
    for nom in ["WEB_CAO", "WEB_ANTENNA"] {
        fs::create_dir_all(poste.racine.join(nom)).unwrap();
        fs::write(poste.racine.join(nom).join(format!("demarrer_{nom}.{ext}")), "").unwrap();
    }

    let (code, sortie) = poste.lancer(INSTALLER, &["cao", "--racine", &racine], &[]);
    assert!(code == 0 && sortie.contains("WEB_CAO est déjà installé") && sortie.contains("1 outil(s) prêt(s)"), "{sortie}");

    // Lancer un outil qui n'est pas là : pas de serveur, et la carte le dit.
    {
        let l = poste.lanceur(&["--local"], &[]);
        let r = l.req("POST", "/api/lancer", &[("X-WebSuite", "1")], br#"{"id":"web_3d"}"#);
        assert!(r.code == 200, "{}", r.texte());
        let e = l.get("/api/etat").json()["outils"][2].clone();
        assert!(e["id"] == "web_3d" && e["phase"] == "erreur" && e["message"] == "Outil non installé." && e["url"].is_null(), "{e}");
    }

    // Hors Windows, openEMS ne s'installe pas tout seul : aucune question à poser.
    if !cfg!(windows) {
        let (code, sortie) = poste.lancer(INSTALLER, &["antenna", "--racine", &racine], &[]);
        assert!(code == 0 && sortie.contains("que sous Windows") && sortie.contains("nf2ff"), "{sortie}");
        assert!(!sortie.contains("Installer openEMS maintenant"), "question openEMS posée hors Windows");
    }
}
