//! Vérification : un dépôt nu joue GitHub, deux racines jouent deux PC.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// git refuse clone, pull et push comme GitHub à un poste pas connecté.
pub static REFUS: AtomicBool = AtomicBool::new(false);
/// Les essais qui règlent l'environnement de git passent un par un.
pub static ENV: Mutex<()> = Mutex::new(());

/// Un dossier temporaire neuf, et un poste neuf : aucune identité git, ni globale ni système.
pub fn poste_neuf(nom: &str) -> PathBuf {
    let tmp = std::env::temp_dir().join(format!("{}-{}", nom, std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let vide = tmp.join("gitconfig-vide");
    fs::write(&vide, "").unwrap();
    std::env::set_var("GIT_CONFIG_GLOBAL", &vide);
    std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
    tmp
}

fn g(cwd: &Path, args: &[&str]) -> String {
    git(cwd, args, 60.0).1
}

#[test]
fn banc() {
    let _seul = ENV.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = poste_neuf("projets-essai");
    let nu = tmp.join("github.git");
    let nu_s = chaine(&nu);
    g(&tmp, &["init", "--bare", &nu_s]);
    let (a, b) = (tmp.join("pc_a"), tmp.join("pc_b"));
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    println!("{}", preparer(&a, &nu_s)); // dépôt vide : départ en local
    fs::create_dir_all(dossier(&a).join("CAO/carte_alim")).unwrap();
    fs::write(dossier(&a).join("CAO/carte_alim/projet.cao.json"), "").unwrap();
    assert!(en_attente(&a, "web_cao"));
    assert!(identite_manquante(&a));
    assert!(!envoyer(&a, "web_cao", "sans identité").0);
    assert!(!definir_identite(&a, "test", "pas-un-mail").0);
    assert!(definir_identite(&a, " test ", "test@example.com").0);
    assert!(!identite_manquante(&a));
    let (ok, msg) = envoyer(&a, "web_cao", "nouveau projet");
    assert!(ok, "{}", msg);
    assert!(!en_attente(&a, "web_cao"));
    println!("{}", preparer(&b, &nu_s)); // l'autre PC clone
    assert!(dossier(&b).join("CAO/carte_alim/projet.cao.json").is_file());
    assert!(fs::read_to_string(dossier(&b).join("projets.txt")).unwrap().contains("[WEB_CAO]\ncarte_alim\n"));
    assert!(dossier(&b).join("3D").is_dir());
    assert!(dossier(&a).join(".gitattributes").is_file());
    // Les zip de la LIB ne partent pas, leur contenu déballé si.
    fs::create_dir_all(dossier(&a).join("LIB_CAO/pack")).unwrap();
    for nom in ["pack.zip", "pack/m.mod"] {
        fs::write(dossier(&a).join("LIB_CAO").join(nom), "").unwrap();
    }
    assert!(envoyer(&a, "web_cao", "pack").0);
    let suivis = g(&dossier(&a), &["ls-files", "LIB_CAO"]);
    assert!(suivis.contains("pack/m.mod") && !suivis.contains(".zip"), "{}", suivis);
    // Le brouillon d'un éditeur ne part pas ; un .gitignore d'avant le reçoit, une fois.
    fs::write(dossier(&a).join("CAO/carte_alim/c-SCH.brouillon.json"), "").unwrap();
    assert!(envoyer(&a, "web_cao", "brouillon").0);
    assert!(!g(&dossier(&a), &["ls-files", "CAO"]).contains("brouillon"));
    let gi = tmp.join("gitignore-ancien");
    fs::write(&gi, "ANTENNA/*/calculs/").unwrap();
    ignorer(&gi, BROUILLON);
    ignorer(&gi, BROUILLON);
    assert_eq!(fs::read_to_string(&gi).unwrap(), "ANTENNA/*/calculs/\n*.brouillon.json\n");
    fs::write(&gi, b"# r\xe9sultats\n").unwrap(); // Latin-1, pas UTF-8
    ignorer(&gi, BROUILLON);
    ignorer(&gi, BROUILLON);
    assert_eq!(fs::read(&gi).unwrap(), b"# r\xe9sultats\n*.brouillon.json\n");
    fs::remove_file(&gi).unwrap();
    fs::create_dir(&gi).unwrap(); // un dossier : on renonce sans planter
    ignorer(&gi, BROUILLON);
    // Envoi hors ligne : rien à commiter ensuite, mais un commit à pousser.
    let fa = dossier(&a).join("CAO/carte_alim/projet.cao.json");
    let fb = dossier(&b).join("CAO/carte_alim/projet.cao.json");
    assert!(definir_identite(&b, "b", "b@example.com").0);
    let absent = chaine(&tmp.join("absent.git"));
    g(&dossier(&b), &["remote", "set-url", "origin", &absent]);
    fs::write(&fb, "b").unwrap();
    let (ok, msg) = envoyer(&b, "web_cao", "hors ligne");
    assert!(!ok && msg.starts_with("[!]"), "{}", msg);
    assert!(g(&dossier(&b), &["status", "--porcelain", "--", "CAO", "LIB_CAO"]).is_empty());
    assert!(en_attente(&b, "web_cao"));
    g(&dossier(&b), &["remote", "set-url", "origin", &nu_s]);
    // Conflit : détecté, et PROJETS n'est pas laissé en plein rebase.
    fs::write(&fa, "a").unwrap();
    assert!(envoyer(&a, "web_cao", "a").0);
    let (ok, msg) = envoyer(&b, "web_cao", "conflit");
    assert!(!ok && msg.starts_with("[X] Conflit"), "{}", msg);
    assert!(!dossier(&b).join(".git/rebase-merge").is_dir());
    // PROJETS rempli sans .git : cloné à côté, le local ajouté, l'ancien gardé.
    let c = tmp.join("pc_c");
    fs::create_dir_all(dossier(&c).join("CAO/perso")).unwrap();
    fs::write(dossier(&c).join("CAO/perso/p.json"), "").unwrap();
    println!("{}", preparer(&c, &nu_s));
    assert!(dossier(&c).join(".git").is_dir());
    assert!(dossier(&c).join("CAO/perso/p.json").is_file());
    assert_eq!(fs::read_to_string(dossier(&c).join("CAO/carte_alim/projet.cao.json")).unwrap(), "a");
    let avant_git = |d: &Path| {
        fs::read_dir(d).unwrap().flatten().any(|e| e.file_name().to_string_lossy().starts_with("PROJETS.avant-git-"))
    };
    assert!(avant_git(&c));
    // Hors ligne puis retour d'Internet : ce qui attend repart tout seul.
    assert!(definir_identite(&c, "c", "c@example.com").0);
    assert!(pousser_en_attente(&c, 120.0).is_none()); // rien n'attend
    fs::write(dossier(&c).join("CAO/perso/p.json"), "hors ligne").unwrap();
    g(&dossier(&c), &["remote", "set-url", "origin", &absent]);
    let (ok, msg) = envoyer(&c, "web_cao", "dans le train");
    assert!(!ok && msg.contains("repart tout seul"), "{}", msg);
    assert!(commits_a_pousser(&c) >= 1);
    let (ok, msg) = pousser_en_attente(&c, 120.0).unwrap(); // toujours hors ligne
    assert!(!ok && commits_a_pousser(&c) >= 1, "{}", msg);
    g(&dossier(&c), &["remote", "set-url", "origin", &nu_s]); // Internet revient
    let (ok, msg) = pousser_en_attente(&c, 120.0).unwrap();
    assert!(ok && msg.contains("hors ligne"), "{}", msg);
    assert!(commits_a_pousser(&c) == 0 && !en_attente(&c, "web_cao"));
    assert!(g(&nu, &["log", "-1", "--format=%s", BRANCHE]).contains("dans le train"));
    assert!(pousser_en_attente(&c, 120.0).is_none());
    // Dépôt privé, poste pas connecté à GitHub : pas de dépôt local sans lien
    // avec GitHub, juste les dossiers ; le clone se fait une fois connecté.
    REFUS.store(true, Ordering::SeqCst);
    let d = tmp.join("pc_d");
    assert_eq!(preparer(&d, &nu_s), acces_refuse());
    assert!(!dossier(&d).join(".git").is_dir());
    assert!(dossier(&d).join("CAO").is_dir() && vide(&dossier(&d)));
    assert_eq!(tirer(&d), acces_refuse()); // et pas un pull dans un dépôt parent
    assert!(!en_attente(&d, "web_cao") && commits_a_pousser(&d) == 0);
    assert!(!envoyer(&d, "web_cao", "x").0);
    assert_eq!(preparer(&d, &nu_s), acces_refuse()); // redémarrage, toujours pas connecté
    assert_eq!(tirer(&a), acces_refuse()); // PROJETS déjà cloné avant le passage en privé
    let (ok, msg) = envoyer(&a, "web_cao", "refusé");
    assert!(!ok && msg.contains("gh auth login"), "{}", msg);
    REFUS.store(false, Ordering::SeqCst); // le poste est connecté
    println!("{}", preparer(&d, &nu_s));
    assert!(dossier(&d).join("CAO/carte_alim/projet.cao.json").is_file());
    assert!(!avant_git(&d));
    let _ = fs::remove_dir_all(&tmp);

    // Push refusé faute d'identifiants (Pi) : le message dit quoi faire.
    assert!(echec_push("fatal: could not read Username for 'https://github.com': terminal prompts disabled").contains("gh auth login"));
    assert!(echec_push(
        "remote: Permission to x.git denied.\nfatal: unable to access '...': The requested URL returned error: 403"
    )
    .contains("gh auth login"));
    assert!(echec_push("remote: Repository not found.\nfatal: repository 'https://github.com/x/y.git/' not found")
        .contains("gh auth login"));
    assert!(!echec_push("fatal: unable to access '...': Could not resolve host: github.com").contains("gh auth"));
    assert!(echec_push("fatal: unable to access 'https://github.com/x.git/': Could not resolve host: github.com\n")
        .contains("Could not resolve host"));
    assert!(echec_push(
        "fatal: '/x' does not appear to be a git repository\nfatal: Could not read from remote \
         repository.\n\nPlease make sure you have the correct access rights\nand the repository exists."
    )
    .contains("fatal: '/x' does not appear"));
    assert!(arguments("web_cao", Path::new("x"), true).contains(&"--projets-reseau".to_string()));
    assert!(!arguments("web_cao", Path::new("x"), false).contains(&"--projets-reseau".to_string()));
}
