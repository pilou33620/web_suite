//! Banc du skill web-suite (module savoir-metier, lanceur/docs/SPEC-mcp-savoir-metier.md).
//!
//!     cargo test -p web_suite_lanceur --test banc_savoir
//!
//! Ce que les fiches citent doit exister dans le serveur MCP : chaque outil (`cao_pcb_lire`) et
//! chaque argument à choix, cité « (quoi=liaisons) » après son outil (ou sous un en-tête de
//! tableau qui le nomme). Les fiches restent courtes, leurs liens et ancres mènent quelque part,
//! et rien de privé n'y entre (chemin personnel, clé, nom de projet) : ni dans les fiches, ni dans
//! la consigne MCP, ni dans le relevé des consignes Gemini, dont chaque source est relue à la
//! révision figée en tête du relevé (dans le clone de l'outil : dans web_suite sur le poste, à
//! côté de lui en CI ; sans clone, les sources ne sont pas relues).
mod commun;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use commun::{depot, echanger, msg, racine};
use regex::Regex;
use serde_json::Value;

type Fiches = BTreeMap<String, String>;
type Outils = HashMap<String, Value>;
/// Le texte d'une source du relevé ; None si l'outil n'est pas cloné.
type Lire<'a> = &'a dyn Fn(&str) -> Result<Option<String>, String>;

const MAX_SKILL: usize = 150;
const MAX_FICHE: usize = 200;
const MAX_INSTRUCTIONS: usize = 1500;
// toutes les règles ont leur destination : une nouvelle règle relevée en garde une
const A_FAIRE_PERMIS: bool = false;

fn re(motif: &str) -> Regex {
    Regex::new(motif).unwrap()
}

/// Les outils (nom -> définition) et la consigne du serveur MCP, lus par le protocole, comme Claude Code.
fn serveur_mcp() -> (Outils, String) {
    let (reponses, code, _) = echanger(&[msg(Some(1), "initialize", None), msg(Some(2), "tools/list", None)], None);
    assert_eq!((code, reponses.len()), (0, 2), "serveur MCP");
    let outils = reponses[1]["result"]["tools"].as_array().unwrap().iter()
        .map(|o| (o["name"].as_str().unwrap().to_string(), o.clone())).collect();
    (outils, reponses[0]["result"]["instructions"].as_str().unwrap().to_string())
}

fn skill() -> PathBuf {
    racine().join(".claude").join("skills").join("web-suite")
}

fn outil_cite() -> Regex {
    re(r"`(etat|(?:cao|lib|antenna|web3d|pilote|pcbparts|savoir)_[a-z0-9_]+)`")
}

/// Les noms des projets de PROJETS (quand il est là) : ils n'ont rien à faire dans le skill.
fn projets() -> Vec<String> {
    let mut noms = Vec::new();
    for sous in ["CAO", "ANTENNA", "3D"] {
        if let Ok(liste) = fs::read_dir(racine().join("PROJETS").join(sous)) {
            noms.extend(liste.flatten().filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.chars().count() > 3));
        }
    }
    noms
}

/// Le premier chemin personnel ou la première clé du texte. « ~/ » est personnel, sauf « ~/.claude/ ».
fn trouve_prive(texte: &str) -> Option<String> {
    let cles = re(r"/home/|/Users/|[A-Za-z]:\\|AIza[0-9A-Za-z_-]{20}|ghp_[0-9A-Za-z]{20}|github_pat_|sk-[a-z]*-?[0-9A-Za-z]{16}|AKIA[0-9A-Z]{16}|xox[bp]-");
    let cle = cles.find(texte).map(|m| (m.start(), m.as_str().to_string()));
    let tilde = texte.match_indices("~/").find(|(i, _)| !texte[i + 2..].starts_with(".claude/")).map(|(i, m)| (i, m.to_string()));
    [cle, tilde].into_iter().flatten().min().map(|(_, m)| m)
}

fn prive(nom: &str, texte: &str, noms_projets: &[String]) -> Vec<String> {
    if let Some(m) = trouve_prive(texte) {
        return vec![format!("{nom} : chemin personnel ou clé : {m}")];
    }
    noms_projets.iter().filter(|p| texte.to_lowercase().contains(&p.to_lowercase())).map(|p| format!("{nom} : nom de projet : {p}")).collect()
}

/// Les ancres des titres d'une fiche, comme GitHub les fabrique.
fn ancres(texte: &str) -> HashSet<String> {
    let hors = re(r"[^\w\- ]");
    re(r"(?m)^#+ (.+)$").captures_iter(texte).map(|t| hors.replace_all(&t[1].trim().to_lowercase(), "").replace(' ', "-")).collect()
}

/// Les valeurs permises de l'argument `arg` de `outil`, s'il est à choix.
fn enum_de<'a>(outils: &'a Outils, outil: &str, arg: &str) -> Option<&'a Vec<Value>> {
    outils.get(outil)?["inputSchema"]["properties"][arg]["enum"].as_array()
}

fn dans_enum(liste: Option<&Vec<Value>>, val: &str) -> bool {
    liste.is_some_and(|l| l.iter().any(|v| v == val))
}

/// fiches : {nom de fichier: texte}. Rend la liste des erreurs (vide si tout va).
fn controler(fiches: &Fiches, outils: &Outils, dossier: &Path, noms_projets: &[String]) -> Vec<String> {
    // un nom d'outil entre accents graves, ou une citation d'argument « (arg=valeur) »
    let citation = re(r"`([a-z0-9_]+)`|\(([a-z_]+)=([^)\s]+)\)");
    let (outil_cite, lien, externe) = (outil_cite(), re(r"\]\(([^)\s]+)\)"), re(r"^[a-z]+:"));
    let tete = re(r"^---\nname: (.+)\ndescription: (.+)\n(?:.*\n)*?---\n");
    let mut erreurs = Vec::new();
    if !fiches.contains_key("SKILL.md") {
        erreurs.push("SKILL.md manquant".to_string());
    }
    for (nom, texte) in fiches {
        let lignes = texte.matches('\n').count() + 1;
        let maxi = if nom == "SKILL.md" { MAX_SKILL } else { MAX_FICHE };
        if lignes > maxi {
            erreurs.push(format!("{nom} : {lignes} lignes ({maxi} au plus)"));
        }
        if nom == "SKILL.md" && !tete.captures(texte).is_some_and(|t| t[1].trim() == "web-suite" && !t[2].trim().is_empty()) {
            erreurs.push("SKILL.md : en-tête « name: web-suite » et « description » attendu".to_string());
        }
        for o in outil_cite.captures_iter(texte) {
            if !outils.contains_key(&o[1]) {
                erreurs.push(format!("{nom} : outil inconnu du serveur MCP : {}", &o[1]));
            }
        }
        // chaque « (arg=valeur) » vaut pour le dernier outil cité avant lui
        let mut dernier: Option<String> = None;
        for c in citation.captures_iter(texte) {
            if let Some(code) = c.get(1) {
                if outils.contains_key(code.as_str()) {
                    dernier = Some(code.as_str().to_string());
                }
                continue;
            }
            let (arg, val) = (&c[2], &c[3]);
            if !dans_enum(dernier.as_deref().and_then(|d| enum_de(outils, d, arg)), val) {
                erreurs.push(format!("{nom} : {} n'a pas de {arg}={val}", dernier.as_deref().unwrap_or("(aucun outil cité avant)")));
            }
        }
        for cible in lien.captures_iter(texte) {
            let cible = &cible[1];
            if externe.is_match(cible) {
                continue; // lien externe : pas vérifié ici
            }
            let (fichier, ancre) = cible.split_once('#').unwrap_or((cible, ""));
            if !fichier.is_empty() && !fiches.contains_key(fichier) && !dossier.join(fichier).exists() {
                erreurs.push(format!("{nom} : lien mort : {cible}"));
            } else if !ancre.is_empty() {
                let visee = fiches.get(if fichier.is_empty() { nom.as_str() } else { fichier }).map(String::as_str).unwrap_or("");
                if !ancres(visee).contains(ancre) {
                    erreurs.push(format!("{nom} : ancre morte : {cible}"));
                }
            }
        }
        erreurs.extend(prive(nom, texte, noms_projets));
    }
    erreurs
}

/// Le texte d'une source à la révision figée du relevé ; None si l'outil n'est pas cloné.
/// Un clone superficiel (CI) va chercher la révision au besoin.
fn source_figee(chemin: &str, revisions: &HashMap<String, String>) -> Result<Option<String>, String> {
    let (outil, reste) = chemin.split_once('/').unwrap_or((chemin, ""));
    let Some(d) = depot(outil) else { return Ok(None) };
    let Some(rev) = revisions.get(outil) else {
        return Err(format!("pas de révision figée pour {outil} en tête du relevé"));
    };
    for essai in 0..2 {
        let r = Command::new("git").arg("-C").arg(&d).args(["show", &format!("{rev}:{reste}")]).output().map_err(|e| e.to_string())?;
        if r.status.success() {
            return Ok(Some(String::from_utf8_lossy(&r.stdout).into_owned()));
        }
        if essai == 0 {
            let _ = Command::new("git").arg("-C").arg(&d).args(["fetch", "--quiet", "--depth", "1", "origin", rev]).output();
        }
    }
    Err(format!("{chemin} introuvable à la révision {rev}"))
}

/// Le relevé des consignes Gemini : chaque ligne a une source qui existe et une destination valable.
/// `lire` : None pour relire chaque source à la révision figée en tête du relevé. Rend (erreurs, « à faire »).
fn controler_trace(texte: &str, fiches: &Fiches, outils: &Outils, a_faire_permis: bool, lire: Option<Lire>,
                   noms_projets: &[String]) -> (Vec<String>, usize) {
    let ligne_trace = re(r"^\| ([A-Z]\d+) \| ([^|]+?):(\d+) \| (.+) \| (.+?) \|$");
    let revisions: HashMap<String, String> =
        re(r"`(WEB_[A-Z0-9]+)@([0-9a-f]{7,40})`").captures_iter(texte).map(|c| (c[1].to_string(), c[2].to_string())).collect();
    let figee = |chemin: &str| source_figee(chemin, &revisions);
    let lire: Lire = lire.unwrap_or(&figee);
    let (outil_cite, genere, virgule) = (outil_cite(), re(r"^généré : ([a-z0-9_]+) \(([a-z_]+)=([a-z_]+)\)$"), re(r",\s*"));
    let (mut erreurs, mut ids, mut a_faire) = (Vec::new(), HashSet::new(), 0);
    let mut sources: HashMap<String, Option<String>> = HashMap::new();
    let debut = re(r"^\|\s*[A-Z]+\d+");
    let lignes: Vec<&str> = texte.split('\n').filter(|l| debut.is_match(l)).collect();
    if lignes.is_empty() {
        erreurs.push("relevé vide".to_string());
    }
    for l in lignes {
        let Some(m) = ligne_trace.captures(l) else {
            erreurs.push(format!("relevé : ligne illisible : {}", l.chars().take(60).collect::<String>()));
            continue;
        };
        let (ident, fichier, num, dest) = (&m[1], &m[2], m[3].parse::<usize>().unwrap_or(0), &m[5]);
        if !ids.insert(ident.to_string()) {
            erreurs.push(format!("relevé : {ident} en double"));
        }
        if !sources.contains_key(fichier) {
            let lue = lire(fichier).unwrap_or_else(|e| {
                erreurs.push(format!("relevé {ident} : {e}"));
                None
            });
            sources.insert(fichier.to_string(), lue);
        }
        if let Some(src) = &sources[fichier] {
            let lignes_src: Vec<&str> = src.split('\n').collect();
            if !(0 < num && num <= lignes_src.len()) {
                erreurs.push(format!("relevé {ident} : {fichier} n'a pas de ligne {num}"));
            } else if lignes_src[num - 1].trim().is_empty() {
                erreurs.push(format!("relevé {ident} : {fichier} ligne {num} est vide"));
            }
        }
        for o in outil_cite.captures_iter(dest) {
            if !outils.contains_key(&o[1]) {
                erreurs.push(format!("relevé {ident} : outil inconnu : {}", &o[1]));
            }
        }
        if dest == "à faire" {
            a_faire += 1;
            if !a_faire_permis {
                erreurs.push(format!("relevé {ident} : encore « à faire »"));
            }
        } else if dest.starts_with("écartée : ") && dest.chars().count() > 12 {
        } else if let Some(nom) = dest.strip_prefix("outil : ") {
            if !outils.contains_key(nom.trim()) {
                erreurs.push(format!("relevé {ident} : outil inconnu : {nom}"));
            }
        } else if dest.starts_with("généré : ") {
            if !genere.captures(dest).is_some_and(|g| dans_enum(enum_de(outils, &g[1], &g[2]), &g[3])) {
                erreurs.push(format!("relevé {ident} : lecture inconnue : {dest}"));
            }
        } else {
            for f in virgule.split(dest) {
                if !fiches.contains_key(f) {
                    erreurs.push(format!("relevé {ident} : destination inconnue : {f}"));
                }
            }
        }
    }
    erreurs.extend(prive("savoir-trace.md", texte, noms_projets));
    (erreurs, a_faire)
}

fn lire_skill(dossier: &Path) -> Fiches {
    fs::read_dir(dossier).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).filter(|f| f.ends_with(".md"))
        .map(|f| (f.clone(), fs::read_to_string(dossier.join(&f)).unwrap())).collect()
}

fn fiches_de(paires: &[(&str, &str)]) -> Fiches {
    paires.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

/// Le banc lui-même : chaque faute connue doit être vue.
#[test]
fn essais() {
    let (outils, _) = serveur_mcp();
    let skill = skill();
    let contient = |e: &[String], motif: &str| e.iter().any(|x| x.contains(motif));
    let bon = "---\nname: web-suite\ndescription: x\n---\n# Titre\nVoir `cao_pcb_lire` (quoi=liaisons) puis \
               (quoi=zone), et [pcb](pcb.md#règles), et `etat`.\n| Question | `cao_calcul` |\n|---|---|\n| x | (analyse=dc) |\n".to_string();
    let fiches = fiches_de(&[("SKILL.md", &bon), ("pcb.md", "# Règles\nok")]);
    assert_eq!(controler(&fiches, &outils, &skill, &[]), Vec::<String>::new());
    for (faute, motif) in [(bon.clone() + "`cao_inventer`", "outil inconnu"),
                           (bon.clone() + "`cao_pcb_lire` (quoi=tout)", "quoi=tout"),
                           (bon.clone() + "`cao_pcb_lire` x (quoi=Reglages)", "quoi=Reglages"),
                           (bon.clone() + "| y | (analyse=thermique) |", "analyse=thermique"),
                           ("---\nname: web-suite\ndescription: x\n---\n(quoi=nets)".to_string(), "aucun outil cité"),
                           (bon.clone() + &"x\n".repeat(160), "lignes"),
                           (bon.clone() + "[x](absent.md)", "lien mort"),
                           (bon.clone() + "[x](pcb.md#rien)", "ancre morte"),
                           (bon.clone() + "dans /home/pilou/x", "chemin personnel"),
                           (bon.clone() + "C:\\Users\\pilou\\x", "chemin personnel"),
                           (bon.clone() + "~/Documents", "chemin personnel"),
                           (bon.clone() + "github_pat_11ABC", "chemin personnel"),
                           (bon.clone() + "le projet Carte Secrète", "nom de projet"),
                           (bon.replace("web-suite", "web_suite"), "en-tête")] {
        let e = controler(&fiches_de(&[("SKILL.md", &faute), ("pcb.md", "# Règles\nok")]), &outils, &skill, &["carte secrète".to_string()]);
        assert!(contient(&e, motif), "{motif} : {e:?}");
    }
    let permis = bon.clone() + "~/.claude/skills";
    assert_eq!(controler(&fiches_de(&[("SKILL.md", &permis), ("pcb.md", "# Règles")]), &outils, &skill, &[]), Vec::<String>::new());
    assert!(contient(&controler(&fiches_de(&[("pcb.md", "ok")]), &outils, &skill, &[]), "SKILL.md manquant"));

    let src = |_: &str| Ok(Some("ligne 1\n\nligne 3\n".to_string()));
    let l = |dest: &str| format!("| C1 | WEB_CAO/x.js:1 | une règle | {dest} |");
    for dest in ["pcb.md", "à faire", "écartée : propre à Gemini", "outil : cao_drc",
                 "généré : cao_pcb_lire (quoi=liaisons)", "pcb.md, SKILL.md"] {
        let e = controler_trace(&l(dest), &fiches, &outils, true, Some(&src), &[]).0;
        assert!(e.is_empty(), "{dest} : {e:?}");
    }
    for (dest, motif) in [("antenne.md", "destination inconnue"), ("outil : cao_rien", "outil inconnu"),
                          ("généré : cao_pcb_lire (quoi=tout)", "lecture inconnue"), ("écartée : ", "destination inconnue")] {
        let e = controler_trace(&l(dest), &fiches, &outils, A_FAIRE_PERMIS, Some(&src), &[]).0;
        assert!(contient(&e, motif), "{dest} : {e:?}");
    }
    for (ligne, motif) in [("| C1 | WEB_CAO/x.js:9 | r | pcb.md |".to_string(), "pas de ligne 9"),
                           ("| C1 | WEB_CAO/x.js:2 | r | pcb.md |".to_string(), "ligne 2 est vide"),
                           (l("pcb.md") + "\n" + &l("pcb.md"), "en double"),
                           ("|C26| WEB_CAO/x.js:1 | r | pcb.md |".to_string(), "ligne illisible"),
                           (l("à faire"), "encore")] {
        let e = controler_trace(&ligne, &fiches, &outils, A_FAIRE_PERMIS, Some(&src), &[]).0;
        assert!(contient(&e, motif), "{motif} : {e:?}");
    }
    if depot("WEB_CAO").is_some() {
        // outil cloné : une source sans révision figée est une erreur
        assert!(contient(&controler_trace(&l("pcb.md"), &fiches, &outils, A_FAIRE_PERMIS, None, &[]).0, "révision"));
    }
}

/// Le skill du dépôt, le relevé des consignes et la consigne du serveur MCP.
#[test]
fn skill_releve_et_consigne() {
    let (outils, consigne) = serveur_mcp();
    let noms = projets();
    let fiches = lire_skill(&skill());
    let mut erreurs = controler(&fiches, &outils, &skill(), &noms);
    let trace = fs::read_to_string(racine().join("lanceur").join("docs").join("savoir-trace.md")).unwrap();
    let (err_trace, a_faire) = controler_trace(&trace, &fiches, &outils, A_FAIRE_PERMIS, None, &noms);
    erreurs.extend(err_trace);
    if consigne.chars().count() > MAX_INSTRUCTIONS || !consigne.contains("web-suite") {
        erreurs.push(format!("consigne MCP : {MAX_INSTRUCTIONS} caractères au plus, avec le renvoi au skill web-suite"));
    }
    erreurs.extend(prive("consigne MCP (INSTRUCTIONS)", &consigne, &noms));
    for e in &erreurs {
        println!("ÉCHEC : {e}");
    }
    assert!(erreurs.is_empty(), "{} erreur(s) : {erreurs:?}", erreurs.len());
    println!("OK : {} fiche(s), {} règle(s) encore « à faire » dans le relevé{}", fiches.len(), a_faire,
             if depot("WEB_CAO").is_some() { "" } else { " (sources non clonées : non relues)" });
}
