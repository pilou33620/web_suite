//! Vérifie l'état git de plusieurs projets et propose de les mettre à jour.
//!
//! Usage : maj_git

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Projets à vérifier (chemins absolus ou relatifs à la racine du dépôt)
const PROJETS: &[&str] = &[
    ".",
    "WEB_CAO",
    "WEB_3D",
    "WEB_ANTENNA",
    "PROJETS", // projets + LIB_CAO (dépôt WEB_SUITE_PROJETS, géré aussi par le lanceur)
];

fn git(repo: &Path, args: &[&str]) -> (bool, String) {
    match Command::new("git").args(args).current_dir(repo).stdin(Stdio::inherit()).output() {
        Ok(r) => {
            let texte = [String::from_utf8_lossy(&r.stdout), String::from_utf8_lossy(&r.stderr)].concat();
            // comme le mode texte de Python : les retours chariot de git (« Rebasing (1/2)\r ») font des lignes
            (r.status.success(), texte.replace("\r\n", "\n").replace('\r', "\n").trim().to_string())
        }
        Err(e) => (false, e.to_string()),
    }
}

struct Etat {
    modifs: Vec<String>, // modifications locales
    avance: u32,         // commits à pousser
    retard: u32,         // commits à tirer
    upstream: bool,      // a une branche distante suivie
}

fn etat(repo: &Path) -> Etat {
    git(repo, &["fetch", "--quiet"]);
    let (_, out) = git(repo, &["status", "--porcelain", "-b"]);
    let mut lignes = out.lines().map(str::to_string);
    let entete = lignes.next().unwrap_or_default();
    let mut e = Etat { modifs: lignes.collect(), avance: 0, retard: 0, upstream: entete.contains("...") };
    if e.upstream {
        let (ok, cpt) = git(repo, &["rev-list", "--left-right", "--count", "HEAD...@{u}"]);
        let nombres: Vec<u32> = cpt.split_whitespace().filter_map(|n| n.parse().ok()).collect();
        if ok && nombres.len() == 2 {
            (e.avance, e.retard) = (nombres[0], nombres[1]);
        }
    }
    e
}

/// input() : la question, puis la ligne tapée (vide si l'entrée est fermée).
fn demander(question: &str) -> String {
    print!("{}", question);
    let _ = std::io::stdout().flush();
    let mut ligne = String::new();
    let _ = std::io::stdin().lock().read_line(&mut ligne);
    ligne.trim().to_string()
}

fn oui(question: &str) -> bool {
    ["o", "oui", "y"].contains(&demander(&format!("  {} [o/N] ", question)).to_lowercase().as_str())
}

fn mettre_a_jour(repo: &Path, e: &Etat) {
    let mut avance = e.avance;
    if !e.modifs.is_empty() {
        println!("  Modifications locales :");
        for m in &e.modifs {
            println!("    {}", m);
        }
        if oui("Commiter ces modifications ?") {
            let saisi = demander("  Message de commit : ");
            let msg = if saisi.is_empty() { "maj" } else { &saisi };
            git(repo, &["add", "-A"]);
            let (ok, out) = git(repo, &["commit", "-m", msg]);
            println!("{}", out.lines().next().map(|l| format!("  {}", l)).unwrap_or_default());
            if !ok {
                return;
            }
            avance += 1;
        }
    }
    if !e.upstream {
        println!("  Pas de branche distante suivie, pull/push ignorés.");
        return;
    }
    if e.retard > 0 && oui(&format!("Récupérer {} commit(s) distant(s) (pull --rebase) ?", e.retard)) {
        let (ok, out) = git(repo, &["pull", "--rebase", "--autostash"]);
        println!("  {}", out.replace('\n', "\n  "));
        if !ok {
            // Ne pas laisser le dépôt en plein rebase : le lanceur et les outils
            // le liraient ensuite tel quel. Les commits locaux restent intacts.
            if ["rebase-merge", "rebase-apply"].iter().any(|d| repo.join(".git").join(d).is_dir()) {
                git(repo, &["rebase", "--abort"]);
                println!("  !! Conflit : rebase annulé, rien n'a été fusionné. À régler à la main dans ce dossier.");
            } else {
                println!("  !! Pull échoué, à régler à la main dans ce dossier.");
            }
            return;
        }
    }
    if avance > 0 && oui(&format!("Pousser {} commit(s) local(aux) (push) ?", avance)) {
        let (_, out) = git(repo, &["push"]);
        println!("  {}", out.replace('\n', "\n  "));
    }
}

fn nom(repo: &Path) -> String {
    repo.file_name().unwrap_or_default().to_string_lossy().into_owned()
}

fn main() {
    let ici = web_suite_lanceur::catalogue::depot();
    let mut a_faire: Vec<(PathBuf, Etat)> = Vec::new();
    println!("=== Vérification des projets ===");
    for p in PROJETS {
        let repo = ici.join(p);
        // absolute plutôt que canonicalize, qui rend des chemins \\?\ sous Windows
        let repo = std::path::absolute(&repo).unwrap_or(repo);
        if !repo.join(".git").exists() {
            println!("[??] {} : pas un dépôt git", repo.display());
            continue;
        }
        let e = etat(&repo);
        let mut infos = Vec::new();
        if !e.modifs.is_empty() {
            infos.push(format!("{} fichier(s) modifié(s)", e.modifs.len()));
        }
        if e.avance > 0 {
            infos.push(format!("{} commit(s) à pousser", e.avance));
        }
        if e.retard > 0 {
            infos.push(format!("{} commit(s) à récupérer", e.retard));
        }
        if !infos.is_empty() {
            println!("[!!] {} : {}", nom(&repo), infos.join(", "));
            a_faire.push((repo, e));
        } else {
            println!("[OK] {}{}", nom(&repo), if e.upstream { "" } else { " (pas de distant)" });
        }
    }

    if a_faire.is_empty() {
        println!("\nTout est à jour.");
    } else {
        println!("\n{} projet(s) à mettre à jour.", a_faire.len());
        for (repo, e) in &a_faire {
            println!("\n--- {} ({}) ---", nom(repo), repo.display());
            if oui("Mettre à jour ce projet ?") {
                mettre_a_jour(repo, e);
            }
        }
    }
    if std::io::stdin().is_terminal() {
        demander("\nEntrée pour quitter...");
    }
}
