//! Sous-processus : sans fenêtre console sous Windows, et avec un délai
//! (subprocess.run(capture_output=True, text=True, timeout=…) de Python).

use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const WINDOWS: bool = cfg!(windows);

/// Pas de fenêtre console pour les serveurs lancés : leur sortie va au journal.
pub fn sans_fenetre(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(cmd, 0x0800_0000); // CREATE_NO_WINDOW
    cmd
}

/// shutil.which : l'exécutable `nom` dans le PATH.
pub fn quel(nom: &str) -> Option<PathBuf> {
    let extensions: &[&str] = if WINDOWS { &[".exe", ".cmd", ".bat", ""] } else { &[""] };
    std::env::split_paths(&std::env::var_os("PATH")?)
        .flat_map(|d| extensions.iter().map(move |e| d.join(format!("{}{}", nom, e))))
        .find(|p| p.is_file())
}

pub struct Sortie {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

fn texte(octets: Vec<u8>) -> String {
    String::from_utf8_lossy(&octets).replace("\r\n", "\n")
}

/// Lance la commande et rend sa sortie ; au-delà de `delai` secondes, elle est tuée
/// et l'erreur le dit comme le faisait Python (ce texte finit dans des messages).
pub fn sortie(cmd: &mut Command, delai: f64, entree: Option<&str>) -> io::Result<Sortie> {
    cmd.stdin(if entree.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut proc = cmd.spawn()?;
    if let (Some(mut stdin), Some(entree)) = (proc.stdin.take(), entree) {
        let _ = stdin.write_all(entree.as_bytes());
    }
    let lire = |mut flux: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut tout = Vec::new();
            let _ = flux.read_to_end(&mut tout);
            tout
        })
    };
    let sortie = lire(Box::new(proc.stdout.take().expect("stdout")));
    let erreur = lire(Box::new(proc.stderr.take().expect("stderr")));
    let fin = Instant::now() + Duration::from_secs_f64(delai);
    let mut pause = Duration::from_millis(1);
    let statut = loop {
        if let Some(statut) = proc.try_wait()? {
            break statut;
        }
        if Instant::now() >= fin {
            let _ = proc.kill();
            let _ = proc.wait();
            let args: Vec<String> = std::iter::once(cmd.get_program())
                .chain(cmd.get_args())
                .map(|a| format!("'{}'", a.to_string_lossy()))
                .collect();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("Command '[{}]' timed out after {} seconds", args.join(", "), delai),
            ));
        }
        std::thread::sleep(pause);
        pause = (pause * 2).min(Duration::from_millis(20));
    };
    Ok(Sortie {
        code: statut.code().unwrap_or(-1),
        stdout: texte(sortie.join().unwrap_or_default()),
        stderr: texte(erreur.join().unwrap_or_default()),
    })
}

/// Les `n` derniers caractères (texte[-n:]).
pub fn fin(texte: &str, n: usize) -> &str {
    let total = texte.chars().count();
    match texte.char_indices().nth(total.saturating_sub(n)) {
        Some((i, _)) => &texte[i..],
        None => "",
    }
}

/// input() : affiche l'invite et lit une ligne du terminal ; None en fin de fichier.
pub fn saisir(invite: &str) -> Option<String> {
    print!("{}", invite);
    let _ = io::stdout().flush();
    let mut ligne = String::new();
    match io::stdin().read_line(&mut ligne) {
        Ok(n) if n > 0 => Some(ligne.trim_end_matches(['\r', '\n']).to_string()),
        _ => None,
    }
}
