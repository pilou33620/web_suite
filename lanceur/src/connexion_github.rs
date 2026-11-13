//! Connexion à GitHub, une fois, dans le terminal du serveur (Raspberry Pi).
//!
//!     connexion_github --connecter  se connecter puis récupérer PROJETS
//!
//! Sous Windows, Git Credential Manager ouvre sa fenêtre de connexion au premier
//! push. Sur un téléphone ou un Pi, il n'y en a pas : le push échouait, et l'on
//! ne l'apprenait que depuis la tablette, au premier envoi. Le lanceur pose donc
//! la question au démarrage, là où l'on a un clavier et où l'on voit la réponse :
//! git a-t-il déjà de quoi s'identifier auprès de github.com ? Sinon, il propose
//! de s'y connecter avec gh (installé au besoin), puis de brancher git dessus.
//!
//! Au passage, le nom et l'e-mail des commits viennent du compte GitHub : la
//! tablette n'a plus à les demander au premier envoi.
//!
//! WEB_SUITE_PROJETS est privé : sans cette connexion, PROJETS n'est même pas
//! récupéré. Lancé sans terminal (service, démarrage automatique), le lanceur ne
//! peut pas poser la question : --connecter la pose à la demande.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::processus::{quel, saisir, sortie};
use crate::projets;

// « ne plus demander » : dans le .git de PROJETS, jamais suivi ni envoyé
const MARQUE: &str = "websuite-sans-github";

fn marque(racine: &Path) -> PathBuf {
    projets::dossier(racine).join(".git").join(MARQUE)
}

/// PROJETS pousse-t-il vers github.com en HTTPS ? (en SSH, la clé s'en charge)
pub fn depot_github_https(racine: &Path) -> bool {
    let p = projets::dossier(racine);
    if !p.join(".git").is_dir() {
        // Pas encore cloné (dépôt privé, poste pas connecté) : l'adresse par défaut.
        return projets::depot().to_lowercase().starts_with("https://github.com/");
    }
    // l'adresse telle qu'écrite dans la config du dépôt (get-url appliquerait les insteadOf)
    let (code, url) = projets::git(&p, &["config", "--get", "remote.origin.url"], 10.0);
    code == 0 && url.trim().to_lowercase().starts_with("https://github.com/")
}

/// git a-t-il des identifiants pour github.com ? Ne demande rien, n'envoie rien :
/// `git credential fill` interroge les assistants configurés (gh, store...).
pub fn github_identifie() -> bool {
    let mut cmd = Command::new("git");
    cmd.args(["credential", "fill"]).envs(projets::ENV_GIT.iter().copied());
    sortie(&mut cmd, 20.0, Some("protocol=https\nhost=github.com\n\n"))
        .is_ok_and(|r| r.code == 0 && format!("\n{}", r.stdout).contains("\npassword="))
}

fn commande_installation_gh() -> Option<Vec<&'static str>> {
    quel("apt-get").map(|_| vec!["sudo", "apt-get", "install", "-y", "gh"])
}

/// Nom et e-mail des commits pris au compte GitHub, s'ils manquent encore.
/// E-mail privé : l'adresse « noreply » que GitHub attribue à chaque compte.
/// `reponse` : ce que gh a répondu (None : on le lui demande).
pub fn identite_depuis_gh(racine: &Path, reponse: Option<&str>) -> bool {
    if !projets::identite_manquante(racine) {
        return false;
    }
    let demandee;
    let reponse = match reponse {
        Some(r) => r,
        None => {
            let mut cmd = Command::new("gh");
            cmd.args(["api", "user", "--jq", r#"[.id, .login, .name // "", .email // ""] | map(tostring) | join("\n")"#]);
            match sortie(&mut cmd, 30.0, None) {
                Ok(r) => demandee = r.stdout,
                Err(_) => return false,
            }
            &demandee
        }
    };
    let mut champs = reponse.lines().map(str::trim).chain(std::iter::repeat(""));
    let (ident, login, nom, email) = (champs.next().unwrap(), champs.next().unwrap(), champs.next().unwrap(), champs.next().unwrap());
    if login.is_empty() {
        return false;
    }
    let noreply = format!("{}+{}@users.noreply.github.com", ident, login);
    projets::definir_identite(racine, if nom.is_empty() { login } else { nom }, if email.is_empty() { &noreply } else { email }).0
}

/// Ce dont `proposer` a besoin du poste : le clavier, l'écran, les commandes.
/// Les essais le remplacent par des réponses et des commandes simulées.
pub struct Poste<'a> {
    /// Pose la question ; None en fin de fichier.
    pub demander: &'a dyn Fn(&str) -> Option<String>,
    /// Lance la commande dans ce terminal, avec ces variables en plus.
    pub lancer: &'a dyn Fn(&[&str], &[(&str, &str)]) -> i32,
    pub ecrire: &'a dyn Fn(&str),
    pub identifie: &'a dyn Fn() -> bool,
    /// La commande est-elle installée ?
    pub quel: &'a dyn Fn(&str) -> bool,
    pub nommer: &'a dyn Fn(&Path) -> bool,
}

fn lancer(cmd: &[&str], env: &[(&str, &str)]) -> i32 {
    Command::new(cmd[0]).args(&cmd[1..]).envs(env.iter().copied()).status().ok().and_then(|s| s.code()).unwrap_or(-1)
}

/// Au démarrage, terminal du serveur. Renvoie ce qui s'est passé :
/// "deja", "ignore", "plus-tard", "jamais", "connecte", "echec".
pub fn proposer(racine: &Path) -> &'static str {
    proposer_avec(
        racine,
        &Poste {
            demander: &saisir,
            lancer: &lancer,
            ecrire: &|texte| println!("{}", texte),
            identifie: &github_identifie,
            quel: &|nom| quel(nom).is_some(),
            nommer: &|racine| identite_depuis_gh(racine, None),
        },
    )
}

pub fn proposer_avec(racine: &Path, poste: &Poste) -> &'static str {
    let ecrire = poste.ecrire;
    if cfg!(windows) || !depot_github_https(racine) || marque(racine).exists() {
        return "ignore";
    }
    if (poste.identifie)() {
        if (poste.quel)("gh") {
            (poste.nommer)(racine);
        }
        return "deja";
    }
    ecrire("");
    ecrire("  GitHub : ce serveur n'est pas encore connecté à votre compte.");
    ecrire("  Le dépôt des projets (WEB_SUITE_PROJETS) est privé : sans cela, ils ne sont");
    ecrire("  ni récupérés ni envoyés sur GitHub.");
    let Some(rep) = (poste.demander)("  Se connecter maintenant ? [O/n, j = ne plus demander] : ") else {
        return "plus-tard";
    };
    let rep = rep.trim().to_lowercase();
    if ["j", "jamais"].contains(&rep.as_str()) {
        if !marque(racine).parent().is_some_and(Path::is_dir) {
            ecrire("  PROJETS n'est pas encore récupéré : la question reviendra au prochain démarrage.");
            return "plus-tard";
        }
        let _ = fs::File::create(marque(racine));
        ecrire(&format!("  D'accord, plus de question. (Supprimer {} pour la retrouver.)", marque(racine).display()));
        return "jamais";
    }
    if !["", "o", "oui", "y", "yes"].contains(&rep.as_str()) {
        ecrire("  Plus tard : la question reviendra au prochain démarrage.");
        return "plus-tard";
    }
    if !(poste.quel)("gh") {
        let Some(cmd) = commande_installation_gh() else {
            ecrire("  [X] Installez gh (https://cli.github.com), puis relancez le lanceur.");
            return "echec";
        };
        ecrire(&format!("  Installation de l'outil GitHub (gh) : {}", cmd.join(" ")));
        (poste.lancer)(&cmd, &[]);
        if !(poste.quel)("gh") {
            ecrire("  [X] gh n'a pas pu être installé (pas d'Internet ?). Nouvel essai au prochain démarrage.");
            return "echec";
        }
    }
    ecrire("");
    ecrire("  gh va afficher un code à 8 caractères : saisissez-le sur");
    ecrire("  https://github.com/login/device (sur ce téléphone, la tablette ou un PC),");
    ecrire("  validez, et revenez ici.");
    // Sur un Pi sans bureau, gh ouvrirait un navigateur texte (lynx, w3m) dans ce
    // terminal et bloquerait tout : on lui donne un navigateur qui ne fait rien.
    let env = [("GH_BROWSER", "true"), ("BROWSER", "true")];
    ecrire("  Quand gh dit « Press Enter », appuyez sur Entrée : aucun navigateur ne");
    ecrire("  s'ouvre ici, gh attend simplement que le code soit validé.");
    ecrire("");
    let _ = std::io::stdout().flush();
    // Pas de --skip-ssh-key : le gh de Raspberry Pi OS / Debian (2.23) ne le connaît
    // pas et s'arrête aussitôt ; en HTTPS, gh ne propose de toute façon aucune clé SSH.
    (poste.lancer)(&["gh", "auth", "login", "--hostname", "github.com", "--git-protocol", "https", "--web"], &env);
    (poste.lancer)(&["gh", "auth", "setup-git", "--hostname", "github.com"], &[]);
    if !(poste.identifie)() {
        ecrire("  [X] Connexion à GitHub non aboutie. Nouvel essai au prochain démarrage.");
        return "echec";
    }
    (poste.nommer)(racine);
    ecrire("  [OK] Ce serveur est connecté à GitHub : les envois partiront.");
    "connecte"
}

/// --connecter : se connecter (même après « ne plus demander »), puis
/// récupérer ou mettre à jour PROJETS.
pub fn connecter(racine: &Path) -> &'static str {
    let _ = fs::remove_file(marque(racine));
    let r = proposer(racine);
    println!("  Projets : {}", projets::preparer(racine, &projets::depot()));
    r
}

#[cfg(test)]
mod essais {
    //! Vérification sans réseau ni vrai gh : un dépôt PROJETS factice, des réponses
    //! et des commandes simulées.
    use super::*;
    use std::cell::RefCell;

    struct Essai<'a> {
        tmp: &'a Path,
        derniere_env: &'a RefCell<Vec<(String, String)>>,
    }

    impl Essai<'_> {
        /// -> (résultat, commandes lancées)
        fn jouer(&self, reponses: &[&str], identifie_avant: bool, identifie_apres: bool, gh: bool, gh_apres: bool) -> (&'static str, Vec<Vec<String>>) {
            let reponses = RefCell::new(reponses.to_vec());
            let (gh, connecte) = (RefCell::new(gh), RefCell::new(false));
            let cmds = RefCell::new(Vec::new());
            let r = proposer_avec(
                self.tmp,
                &Poste {
                    demander: &|_| Some(reponses.borrow_mut().remove(0).to_string()),
                    lancer: &|cmd, env| {
                        cmds.borrow_mut().push(cmd.iter().map(|c| c.to_string()).collect::<Vec<_>>());
                        if cmd[..3] == ["pkg", "install", "-y"] || cmd[..3] == ["sudo", "apt-get", "install"] {
                            *gh.borrow_mut() = gh_apres;
                        }
                        if cmd[..3] == ["gh", "auth", "login"] {
                            *connecte.borrow_mut() = identifie_apres;
                            *self.derniere_env.borrow_mut() = env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
                        }
                        0
                    },
                    ecrire: &|_| (),
                    identifie: &|| identifie_avant || *connecte.borrow(),
                    quel: &|_| *gh.borrow(),
                    nommer: &|_| false,
                },
            );
            (r, cmds.into_inner())
        }
    }

    #[test]
    fn banc() {
        let _seul = projets::essais::ENV.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = projets::essais::poste_neuf("connexion-essai");
        let p = projets::dossier(&tmp);
        fs::create_dir_all(&p).unwrap();
        // PROJETS pas encore cloné (dépôt privé) : la question est posée quand même,
        // et « ne plus demander » n'a nulle part où s'écrire.
        assert!(depot_github_https(&tmp));
        let muet = Poste {
            demander: &|_| Some("j".to_string()),
            lancer: &lancer,
            ecrire: &|_| (),
            identifie: &|| false,
            quel: &|nom| quel(nom).is_some(),
            nommer: &|racine| identite_depuis_gh(racine, None),
        };
        assert_eq!(proposer_avec(&tmp, &muet), "plus-tard");
        projets::git(&p, &["init"], 60.0);
        projets::git(&p, &["remote", "add", "origin", "https://github.com/x/WEB_SUITE_PROJETS.git"], 60.0);

        let derniere_env = RefCell::new(Vec::new());
        let e = Essai { tmp: &tmp, derniere_env: &derniere_env };
        let contient = |cmds: &[Vec<String>], cmd: &[&str]| cmds.iter().any(|c| c == cmd);

        assert!(!github_identifie()); // aucun assistant configuré
        assert_eq!(e.jouer(&[], true, true, true, true).0, "deja"); // déjà connecté : aucune question
        let (r, cmds) = e.jouer(&[""], false, true, true, true); // Entrée = oui
        assert_eq!(r, "connecte");
        assert!(contient(&cmds, &["gh", "auth", "setup-git", "--hostname", "github.com"]));
        assert!(cmds.iter().any(|c| c[..3] == ["gh", "auth", "login"] && c.contains(&"--web".to_string())));
        assert!(derniere_env.borrow().contains(&("GH_BROWSER".to_string(), "true".to_string()))); // pas de lynx qui bloque
        assert_eq!(e.jouer(&["n"], false, true, true, true).0, "plus-tard");
        if quel("apt-get").is_some() {
            let (r, cmds) = e.jouer(&["o"], false, true, false, true); // gh absent : installé d'abord
            assert!(r == "connecte" && cmds[0].last().unwrap() == "gh", "{:?}", cmds);
        }
        assert_eq!(e.jouer(&["o"], false, true, false, false).0, "echec");
        assert_eq!(e.jouer(&["o"], false, false, true, true).0, "echec");
        assert_eq!(e.jouer(&["j"], false, true, true, true).0, "jamais");
        assert_eq!(e.jouer(&[], false, true, true, true).0, "ignore"); // plus jamais demandé
        fs::remove_file(marque(&tmp)).unwrap();
        projets::git(&p, &["remote", "set-url", "origin", "git@github.com:x/y.git"], 60.0);
        assert_eq!(e.jouer(&[], false, true, true, true).0, "ignore"); // SSH : la clé s'en charge
        // Nom et e-mail tirés du compte GitHub (e-mail privé : adresse noreply).
        assert!(projets::identite_manquante(&tmp));
        assert!(identite_depuis_gh(&tmp, Some("123\npilou33620\n\n\n")));
        assert_eq!(projets::git(&p, &["config", "user.name"], 60.0).1, "pilou33620");
        assert_eq!(projets::git(&p, &["config", "user.email"], 60.0).1, "123+pilou33620@users.noreply.github.com");
        assert!(!identite_depuis_gh(&tmp, Some("9\nautre\nAutre\na@b.fr\n"))); // déjà fixée : on n'y touche pas
        let _ = fs::remove_dir_all(&tmp);
    }
}
