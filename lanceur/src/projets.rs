//! Dossier PROJETS : les projets des outils, synchronisés avec GitHub.
//!
//!     PROJETS/              <- dépôt git WEB_SUITE_PROJETS, ignoré par WEB_SUITE
//!     ├── projets.txt       <- la liste des projets, réécrite à chaque envoi
//!     ├── CAO/<projet>/     <- WEB_CAO --projets
//!     ├── LIB_CAO/          <- WEB_CAO --lib
//!     ├── ANTENNA/<projet>/ <- WEB_ANTENNA --projets
//!     └── 3D/               <- WEB_3D --projets (modèles rangés depuis la visionneuse)
//!
//! Le lanceur tire (pull) au démarrage et avant chaque lancement d'outil, et
//! envoie (commit + push) quand on arrête un outil. Un seul utilisateur : en cas
//! de conflit on s'arrête et on prévient, sans rien fusionner tout seul.
//!
//! WEB_SUITE_PROJETS est privé : sans compte GitHub connecté sur le poste, git
//! ne peut ni le cloner ni le tirer. On ne crée alors pas de dépôt local (son
//! historique serait sans lien avec GitHub) : PROJETS reste un simple dossier,
//! cloné dès que le poste est connecté (connexion_github.rs).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard};

use crate::processus::{fin, sans_fenetre, sortie};

pub const DEPOT: &str = "https://github.com/pilou33620/WEB_SUITE_PROJETS.git";
pub const BRANCHE: &str = "main";

/// Le dépôt des projets. WEB_SUITE_BANC_PROJETS (bancs d'essai) : un dépôt local
/// joue GitHub, rien ne part sur le réseau ; la variable ne donne aucun droit.
pub fn depot() -> String {
    std::env::var("WEB_SUITE_BANC_PROJETS").ok().filter(|d| !d.is_empty()).unwrap_or_else(|| DEPOT.to_string())
}

/// Les dossiers de chaque outil : ce qu'on lui passe, et ce qu'on envoie à l'arrêt.
pub fn dossiers(oid: &str) -> &'static [&'static str] {
    match oid {
        "web_cao" => &["CAO", "LIB_CAO"],
        "web_antenna" => &["ANTENNA"],
        "web_3d" => &["3D"],
        _ => &[],
    }
}
const SECTIONS: &[(&str, &str)] = &[("WEB_CAO", "CAO"), ("WEB_ANTENNA", "ANTENNA"), ("WEB_3D", "3D")];
// Les résultats d'openEMS se recalculent et pèsent vite des centaines de Mo.
// Les zip des packs Murata sont déballés à côté (lier_modeles_murata.py), et
// c'est le déballé qui est suivi : un zip ne se compresse pas en différences.
// Les brouillons de WEB_CAO (sauvegarde automatique des éditeurs, effacée à
// l'enregistrement) restent sur le poste : un brouillon n'est pas un document.
const BROUILLON: &str = "*.brouillon.json";
const GITIGNORE: &str = "ANTENNA/*/calculs/\nLIB_CAO/**/*.zip\n*.brouillon.json\n";
// Fins de ligne normalisées dans le dépôt : un PC Windows (autocrlf) et le Pi
// ne doivent pas se renvoyer des fichiers modifiés de la première à la dernière ligne.
const GITATTRIBUTES: &str = "* text=auto\n";
// Sans LC_ALL=C, un git traduit (Pi en fr_FR) ne dit plus « CONFLICT » ;
// GIT_OPTIONAL_LOCKS=0 : le git status de l'affichage ne prend plus index.lock
// pendant le commit d'un envoi.
pub const ENV_GIT: &[(&str, &str)] =
    &[("GIT_TERMINAL_PROMPT", "0"), ("LC_ALL", "C"), ("LANGUAGE", "C"), ("GIT_OPTIONAL_LOCKS", "0")];

static VERROU: Mutex<()> = Mutex::new(()); // une commande git à la fois

fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn dossier(racine: &Path) -> PathBuf {
    racine.join("PROJETS")
}

fn chaine(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// Les options qui pointent l'outil vers ses dossiers de PROJETS.
///
/// reseau : le lanceur écoute sur le réseau (--reseau). WEB_CAO y ferme
/// d'office ses routes de projet ; --projets-reseau les rouvre, confinées
/// à PROJETS/CAO et à la LIB. WEB_ANTENNA, lui, les sert déjà au réseau.
pub fn arguments(oid: &str, racine: &Path, reseau: bool) -> Vec<String> {
    let p = dossier(racine);
    match oid {
        "web_cao" => {
            let mut a = vec!["--projets".to_string(), chaine(&p.join("CAO")), "--lib".to_string(), chaine(&p.join("LIB_CAO"))];
            if reseau {
                a.push("--projets-reseau".to_string());
            }
            a
        }
        "web_antenna" => vec!["--projets".to_string(), chaine(&p.join("ANTENNA"))],
        "web_3d" => vec!["--projets".to_string(), chaine(&p.join("3D"))],
        _ => Vec::new(),
    }
}

/// -> (code de sortie, sortie et erreurs réunies).
pub fn git(cwd: &Path, args: &[&str], delai: f64) -> (i32, String) {
    #[cfg(test)]
    if essais::REFUS.load(std::sync::atomic::Ordering::SeqCst) && ["clone", "pull", "push"].contains(&args[0]) {
        return (128, "fatal: could not read Username for 'https://github.com': terminal prompts disabled".to_string());
    }
    // GIT_TERMINAL_PROMPT=0 : pas de question en console invisible. Sous Windows,
    // Git Credential Manager ouvre sa propre fenêtre de connexion au premier push.
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(cwd).envs(ENV_GIT.iter().copied());
    match sortie(sans_fenetre(&mut cmd), delai, None) {
        Ok(r) => (r.code, (r.stdout + &r.stderr).trim().to_string()),
        Err(exc) => (1, exc.to_string()),
    }
}

/// Crée ou récupère PROJETS/, puis le met à jour. Renvoie un message lisible.
pub fn preparer(racine: &Path, depot: &str) -> String {
    let p = dossier(racine);
    let mut note = String::new();
    let _tenu = verrou();
    if !p.join(".git").is_dir() {
        if !vide(&p) {
            let (ok, texte) = adopter(racine, &p, depot);
            if !ok {
                return texte;
            }
            note = texte;
        } else {
            let _ = fs::remove_dir_all(&p); // squelette laissé par un essai précédent
            let (code, sortie) = git(racine, &["clone", depot, &chaine(&p)], 600.0);
            if code != 0 && refus_auth(&sortie) {
                squelette(&p, false);
                return acces_refuse();
            }
            if code != 0 {
                // Dépôt pas encore créé sur GitHub, ou hors ligne : on part en
                // local, le premier envoi réussi le remplira.
                let _ = fs::create_dir_all(&p);
                git(&p, &["init"], 60.0);
                git(&p, &["remote", "add", "origin", depot], 60.0);
            }
        }
        if git(&p, &["rev-parse", "--verify", "HEAD"], 60.0).0 != 0 {
            // Branche encore vide (init, ou clone d'un dépôt vide) : on la
            // nomme main quel que soit le réglage init.defaultBranch du poste.
            git(&p, &["symbolic-ref", "HEAD", &format!("refs/heads/{}", BRANCHE)], 60.0);
        }
    }
    // Tirer avant de créer le squelette : un .gitkeep local non suivi
    // bloquerait le pull du même fichier venu de GitHub.
    let etat = tirer_dans(&p);
    squelette(&p, true);
    for (nom, texte) in [(".gitignore", GITIGNORE), (".gitattributes", GITATTRIBUTES)] {
        if !p.join(nom).exists() {
            // un dossier à ce nom : on n'y touche pas
            let _ = fs::write(p.join(nom), texte);
        }
    }
    ignorer(&p.join(".gitignore"), BROUILLON);
    format!("{} {}", note, etat).trim().to_string()
}

/// Ajoute `motif` à un .gitignore d'avant qui ne l'a pas, sans toucher au reste.
/// Illisible (dossier, droits) : on renonce, le démarrage continue.
fn ignorer(chemin: &Path, motif: &str) {
    // En octets : un .gitignore en Latin-1 se lit aussi ; on n'y ajoute que de l'ASCII
    let Ok(texte) = fs::read(chemin) else { return };
    if texte.split(|c| *c == b'\n' || *c == b'\r').any(|l| l.trim_ascii() == motif.as_bytes()) {
        return;
    }
    let debut = if !texte.is_empty() && !texte.ends_with(b"\n") { "\n" } else { "" };
    if let Ok(mut f) = fs::OpenOptions::new().append(true).open(chemin) {
        let _ = writeln!(f, "{}{}", debut, motif);
    }
}

/// Les dossiers des outils. .gitkeep : git ignore les dossiers vides.
fn squelette(p: &Path, gitkeep: bool) {
    for nom in ["CAO", "3D", "ANTENNA", "LIB_CAO"] {
        let d = p.join(nom);
        let _ = fs::create_dir_all(&d);
        if gitkeep && fs::read_dir(&d).is_ok_and(|mut l| l.next().is_none()) {
            let _ = fs::File::create(d.join(".gitkeep"));
        }
    }
}

/// Rien d'autre que des dossiers vides et des .gitkeep : le squelette d'un
/// démarrage où GitHub a refusé le clone. On peut cloner à sa place.
fn vide(p: &Path) -> bool {
    let Ok(entrees) = fs::read_dir(p) else { return true };
    entrees.flatten().all(|e| match e.file_type() {
        Ok(t) if t.is_dir() => vide(&e.path()),
        Ok(t) if t.is_symlink() && e.path().is_dir() => true, // os.walk n'y entre pas
        _ => e.file_name() == ".gitkeep",
    })
}

/// copytree : les fichiers de `src` absents de `dst` (dates gardées).
fn copier_absents(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let (de, vers) = (e.path(), dst.join(e.file_name()));
        let meta = fs::metadata(&de)?;
        if meta.is_dir() {
            copier_absents(&de, &vers)?;
        } else if !vers.exists() {
            fs::copy(&de, &vers)?;
            if let Ok(date) = meta.modified() {
                let _ = fs::File::options().write(true).open(&vers).and_then(|f| f.set_modified(date));
            }
        }
    }
    Ok(())
}

/// PROJETS/ rempli mais sans .git : on clone à côté plutôt que git init.
///
/// Un init donnait un historique sans lien avec GitHub, donc un conflit
/// garanti au premier envoi. On clone dans PROJETS.clone, on y ajoute les
/// fichiers locaux que GitHub n'a pas (GitHub gagne quand les deux existent),
/// et l'ancien dossier reste intact à côté, en sauvegarde. Renvoie (ok, message).
fn adopter(racine: &Path, p: &Path, depot: &str) -> (bool, String) {
    let neuf = PathBuf::from(format!("{}.clone", p.display()));
    let _ = fs::remove_dir_all(&neuf);
    let (code, sortie) = git(racine, &["clone", depot, &chaine(&neuf)], 600.0);
    if code != 0 {
        let _ = fs::remove_dir_all(&neuf);
        if refus_auth(&sortie) {
            return (false, format!("{} Vos fichiers locaux restent dans {}.", acces_refuse(), p.display()));
        }
        return (
            false,
            format!(
                "[!] {} existe sans git et GitHub est injoignable : rien n'est \
                 synchronisé, nouvel essai au prochain démarrage.",
                p.display()
            ),
        );
    }
    if let Err(exc) = copier_absents(p, &neuf) {
        // Python s'arrêtait ici sur une trace d'erreur ; l'ancien dossier reste intact.
        let _ = fs::remove_dir_all(&neuf);
        return (false, format!("[!] {} n'a pas pu être copié à côté du clone : {}", p.display(), exc));
    }
    let sauvegarde = PathBuf::from(format!("{}{}", p.display(), chrono::Local::now().format(".avant-git-%Y%m%d-%H%M%S")));
    if let Err(exc) = fs::rename(p, &sauvegarde) {
        let _ = fs::remove_dir_all(&neuf);
        return (false, format!("[!] {} n'a pas pu être mis de côté (fichier ouvert ?) : {}", p.display(), exc));
    }
    let _ = fs::rename(&neuf, p);
    (true, format!("PROJETS relié à GitHub ; l'ancien dossier est gardé dans {}.", sauvegarde.display()))
}

pub fn tirer(racine: &Path) -> String {
    let _tenu = verrou();
    tirer_dans(&dossier(racine))
}

fn tirer_dans(p: &Path) -> String {
    if !p.join(".git").is_dir() {
        // Sans ce test, git remonterait au dépôt parent (WEB_SUITE cloné avec git).
        return acces_refuse();
    }
    let (code, sortie) = git(p, &["pull", "--rebase", "--autostash", "origin", BRANCHE], 120.0);
    if code == 0 {
        return "Projets à jour.".to_string();
    }
    if sortie.contains("couldn't find remote ref") {
        return "Dépôt GitHub vide : projets gardés en local.".to_string();
    }
    if refus_auth(&sortie) {
        return acces_refuse();
    }
    // Le dossier de rebase plutôt que le texte : il reste aussi quand git a été
    // coupé par le délai, et ne laisse jamais PROJETS en plein rebase.
    let en_rebase = ["rebase-merge", "rebase-apply"].iter().any(|d| p.join(".git").join(d).is_dir());
    if en_rebase || sortie.contains("CONFLICT") {
        git(p, &["rebase", "--abort"], 60.0);
        return format!("[X] Conflit avec GitHub : rien n'a été fusionné, à régler à la main dans {}.", p.display());
    }
    format!("GitHub injoignable : on travaille en local ({}).", sortie.lines().last().unwrap_or("?"))
}

fn compte(code: i32, sortie: &str) -> u64 {
    if code == 0 && !sortie.is_empty() && sortie.bytes().all(|c| c.is_ascii_digit()) {
        sortie.parse().unwrap_or(0)
    } else {
        0
    }
}

/// True si les dossiers de l'outil ont des modifs à envoyer, ou des commits
/// pas encore poussés (envoi précédent fait hors ligne).
pub fn en_attente(racine: &Path, oid: &str) -> bool {
    let p = dossier(racine);
    if !p.join(".git").is_dir() {
        return false;
    }
    let (code, sortie) = git(&p, &[&["status", "--porcelain", "--"], dossiers(oid)].concat(), 20.0);
    if code == 0 && !sortie.is_empty() {
        return true;
    }
    // --not --remotes=origin : marche aussi avant le tout premier push (pas d'@{u}).
    let (code, sortie) =
        git(&p, &[&["rev-list", "--count", "HEAD", "--not", "--remotes=origin", "--"], dossiers(oid)].concat(), 20.0);
    compte(code, &sortie) > 0
}

pub fn ecrire_liste(p: &Path) -> std::io::Result<()> {
    let mut lignes = Vec::new();
    for (titre, nom) in SECTIONS {
        lignes.push(format!("[{}]", titre));
        // Un dossier par projet (CAO, ANTENNA), un fichier par modèle (3D).
        if let Ok(entrees) = fs::read_dir(p.join(nom)) {
            let mut noms: Vec<String> = entrees
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| !n.starts_with('.'))
                .collect();
            noms.sort();
            lignes.extend(noms);
        }
        lignes.push(String::new());
    }
    fs::write(p.join("projets.txt"), lignes.join("\n"))
}

/// True si git ne sait pas qui signe les commits de PROJETS.
///
/// Cas d'un poste neuf, typiquement le Raspberry Pi : sans user.name et
/// user.email, le commit échoue. La page les demande alors une fois.
pub fn identite_manquante(racine: &Path) -> bool {
    let p = dossier(racine);
    ["user.name", "user.email"].iter().any(|cle| git(&p, &["config", cle], 10.0).0 != 0)
}

/// Enregistre nom et e-mail dans la config du dépôt PROJETS (pas en global :
/// on ne règle que ce que le lanceur utilise). Renvoie (ok, message).
pub fn definir_identite(racine: &Path, nom: &str, email: &str) -> (bool, String) {
    let nom = nom.split_whitespace().collect::<Vec<_>>().join(" ");
    let email = email.trim();
    if nom.is_empty() || nom.chars().count() > 100 {
        return (false, "Nom invalide.".to_string());
    }
    let forme = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("motif");
    if email.chars().count() > 200 || !forme.is_match(email) {
        return (false, "Adresse e-mail invalide.".to_string());
    }
    let _tenu = verrou();
    let p = dossier(racine);
    for (cle, valeur) in [("user.name", nom.as_str()), ("user.email", email)] {
        if git(&p, &["config", cle, valeur], 10.0).0 != 0 {
            return (false, format!("git config {} a échoué.", cle));
        }
    }
    (true, "Identité git enregistrée pour PROJETS.".to_string())
}

// Ce que git répond quand GitHub ne sait pas qui pousse. Sous Windows, Git
// Credential Manager ouvre sa fenêtre ; sur un Raspberry Pi, il
// n'y en a pas, et GIT_TERMINAL_PROMPT=0 interdit la question en console.
// Dépôt privé : sans compte autorisé, GitHub répond « Repository not found ».
const REFUS_AUTH: &[&str] = &[
    "could not read username",
    "could not read password",
    "terminal prompts disabled",
    "authentication failed",
    "invalid username or password",
    "permission denied",
    "the requested url returned error: 403",
    "error: 403",
    "repository not found",
];
const CONNEXION: &str = "relancez le lanceur dans son terminal : il propose de s'y connecter, une \
                         seule fois (ou « python lanceur/connexion_github.py --connecter » ; \
                         à la main : « gh auth login » puis « gh auth setup-git »).";

pub fn acces_refuse() -> String {
    format!(
        "[!] GitHub refuse l'accès à WEB_SUITE_PROJETS (dépôt privé) : ce poste n'est \
         pas encore connecté à votre compte GitHub, les projets ne sont pas \
         synchronisés. Pour le connecter, {}",
        CONNEXION
    )
}

pub fn refus_auth(sortie: &str) -> bool {
    let bas = sortie.to_lowercase();
    REFUS_AUTH.iter().any(|m| bas.contains(m))
}

/// Le message d'un push refusé : si c'est l'authentification, dire quoi faire.
pub fn echec_push(sortie: &str) -> String {
    if refus_auth(sortie) {
        return format!(
            "[!] Enregistré sur ce serveur, mais GitHub refuse l'envoi : le serveur \
             n'est pas encore connecté à votre compte GitHub. Pour le connecter, \
             {} Rien n'est perdu : l'envoi repartira tout seul une fois connecté.",
            CONNEXION
        );
    }
    format!(
        "[!] Enregistré sur ce poste ; GitHub injoignable pour l'instant (pas \
         d'Internet ?). Rien n'est perdu : l'envoi repart tout seul dès que la \
         connexion revient, et au prochain démarrage. ({})",
        cause(sortie)
    )
}

/// La ligne de git qui dit pourquoi : « fatal: ... », sinon la dernière.
fn cause(sortie: &str) -> &str {
    let lignes: Vec<&str> = sortie.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    lignes.iter().find(|l| l.starts_with("fatal:")).or(lignes.last()).copied().unwrap_or("?")
}

/// Commits de PROJETS pas encore sur GitHub : les envois faits hors ligne.
///
/// --not --remotes=origin : marche aussi avant le tout premier push (pas d'@{u}).
pub fn commits_a_pousser(racine: &Path) -> u64 {
    let p = dossier(racine);
    if !p.join(".git").is_dir() {
        return 0;
    }
    let (code, sortie) = git(&p, &["rev-list", "--count", "HEAD", "--not", "--remotes=origin"], 20.0);
    compte(code, &sortie)
}

/// Renvoie à GitHub ce qui a été enregistré hors ligne (pull --rebase + push).
///
/// Rien n'est commité ici : on ne pousse que ce qu'un envoi a déjà enregistré,
/// avec son message. Renvoie None si rien n'attend, sinon (ok, message).
/// Délai plus court qu'un envoi demandé (120 s) : un Wi-Fi sans Internet peut laisser
/// une connexion pendre, et l'essai suivant viendra de toute façon.
pub fn pousser_en_attente(racine: &Path, delai: f64) -> Option<(bool, String)> {
    let p = dossier(racine);
    let _tenu = verrou();
    let n = commits_a_pousser(racine);
    if n == 0 {
        return None;
    }
    let tire = tirer_dans(&p);
    if tire.starts_with("[X]") {
        return Some((false, tire));
    }
    let (code, sortie) = git(&p, &["push", "-u", "origin", BRANCHE], delai);
    if code != 0 {
        return Some((false, echec_push(&sortie)));
    }
    Some((true, format!("Revenu en ligne : {} enregistrement(s) fait(s) hors ligne envoyé(s) sur GitHub.", n)))
}

/// commit + pull --rebase + push des dossiers de l'outil. Renvoie (ok, message).
pub fn envoyer(racine: &Path, oid: &str, message: &str) -> (bool, String) {
    let p = dossier(racine);
    let _tenu = verrou();
    if !p.join(".git").is_dir() {
        return (false, "PROJETS n'est pas un dépôt git.".to_string());
    }
    if let Err(exc) = ecrire_liste(&p) {
        return (false, format!("[X] projets.txt n'a pas pu être écrit : {}", exc));
    }
    // Un chemin absent ferait échouer tout l'add (« pathspec did not match »).
    let racine_fichiers: Vec<&str> =
        ["projets.txt", ".gitignore", ".gitattributes"].into_iter().filter(|f| p.join(f).exists()).collect();
    git(&p, &[&["add", "-A", "--"], dossiers(oid), &racine_fichiers[..]].concat(), 60.0);
    if git(&p, &["diff", "--cached", "--quiet"], 60.0).0 != 0 {
        let defaut = format!("Session {}", oid);
        let message = if message.trim().is_empty() { defaut.as_str() } else { message.trim() };
        let (code, sortie) = git(&p, &["commit", "-m", message], 60.0);
        if code != 0 {
            return (false, format!("[X] git commit a échoué : {}", fin(&sortie, 300)));
        }
    }
    let tire = tirer_dans(&p);
    if tire.starts_with("[X]") {
        return (false, tire);
    }
    let (code, sortie) = git(&p, &["push", "-u", "origin", BRANCHE], 600.0);
    if code != 0 {
        return (false, echec_push(&sortie));
    }
    (true, "Envoyé sur GitHub.".to_string())
}

#[cfg(test)]
pub(crate) mod essais;
