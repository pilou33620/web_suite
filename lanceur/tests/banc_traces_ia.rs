//! Banc du module retrait-gemini : plus aucune trace
//! active de l'ancien assistant IA (Gemini / Gemma) dans web_suite ni dans ses trois outils.
//!
//!     cargo test -p web_suite_lanceur --test banc_traces_ia
//!
//! Restent permis :
//! - l'historique : les docs de lanceur/docs (specs, relevé du savoir) et les tasks/ ;
//! - ce qui cite l'assistant pour s'en garder : les routes interdites du MCP (lanceur/src/mcp/mod.rs) et
//!   leurs tests (tests/banc_mcp.rs), les tests de 404 des serveurs (python/test/), les .gitignore du
//!   fichier de clé ;
//! - graphify-out/, l'analyse générée du code d'un outil (elle garde les noms de l'ancien code) ;
//! - le banc du savoir, qui relit les anciennes consignes à une révision figée, et ce banc-ci.
//!
//! Les outils sont relus dans leur clone : dans web_suite sur le poste, à côté de lui en CI
//! (cloné dans ../) ; un outil non cloné n'est pas vérifié.
mod commun;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use commun::{depot, racine};
use regex::Regex;

const PERMIS_WEB_SUITE: &[&str] = &[r"^lanceur/docs/(SPEC-|savoir-trace)", r"^lanceur/src/mcp/mod\.rs$",
                                    r"^lanceur/tests/banc_(mcp|savoir|traces_ia)\.rs$", r"\.gitignore$"];
const PERMIS_OUTIL: &[&str] = &[r"^tasks/", r"^graphify-out/", r"^python/test/banc-(serveur|lib-routes|serveur-routes)\.py$", r"\.gitignore$"];

/// fichiers : {chemin: texte}. Rend les traces hors des chemins permis : [(chemin, n°, ligne)].
fn chercher(fichiers: &BTreeMap<String, String>, permis: &[&str]) -> Vec<(String, usize, String)> {
    // les noms de l'ancien assistant, et ses crochets dans les pages (un crochet remis se verrait)
    let trace = Regex::new(r"(?i)generativelanguage|gemma|gemini|/api/ia/cle|ia-assistant|06-ia-lib|30-ia\.js|bIaAssistant|iaAfficherMenuContextuel|iaPanneau|tb-ia\b").unwrap();
    let permis: Vec<Regex> = permis.iter().map(|p| Regex::new(p).unwrap()).collect();
    let mut traces = Vec::new();
    for (chemin, texte) in fichiers {
        if permis.iter().any(|p| p.is_match(chemin)) {
            continue;
        }
        for (n, ligne) in texte.split('\n').enumerate() {
            // le nom du module (et de sa spec), cité tel quel
            if trace.is_match(&ligne.replace("retrait-gemini", "")) {
                traces.push((chemin.clone(), n + 1, ligne.trim().chars().take(120).collect()));
            }
        }
    }
    traces
}

fn fichiers_suivis(depot: &Path) -> BTreeMap<String, String> {
    let r = Command::new("git").arg("-C").arg(depot).arg("ls-files").output().unwrap();
    assert!(r.status.success(), "git ls-files dans {}", depot.display());
    // binaire (wasm, image) ou fichier absent de la copie de travail : passé
    String::from_utf8_lossy(&r.stdout).split('\n').filter(|n| !n.is_empty())
        .filter_map(|nom| Some((nom.to_string(), fs::read_to_string(depot.join(nom)).ok()?))).collect()
}

fn fichiers(paires: &[(&str, &str)]) -> BTreeMap<String, String> {
    paires.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

/// Le banc lui-même : une trace hors des chemins permis doit être vue.
#[test]
fn essais() {
    let faux = fichiers(&[("js/page.js", "fetch('https://generativelanguage.googleapis.com')"),
                          ("tasks/todo.md", "retirer ia-assistant.js"),
                          ("python/test/banc-serveur.py", "GET /api/ia/cle -> 404"),
                          ("python/test/autre.py", "bIaAssistant"),
                          ("css/x.css", "rien à signaler")]);
    let t = chercher(&faux, PERMIS_OUTIL);
    assert_eq!(t.iter().map(|(c, _, _)| c.as_str()).collect::<Vec<_>>(), ["js/page.js", "python/test/autre.py"], "{t:?}");
    assert!(chercher(&fichiers(&[("lanceur/docs/SPEC-x.md", "comme Gemini")]), PERMIS_WEB_SUITE).is_empty());
    assert!(!chercher(&fichiers(&[("lanceur/docs/DETAILS.md", "Gemini")]), PERMIS_WEB_SUITE).is_empty()); // doc vivante : pas permise
    assert!(!chercher(&fichiers(&[("lanceur/outil.py", "Gemma 4")]), PERMIS_WEB_SUITE).is_empty());
    assert!(chercher(&fichiers(&[("A-FAIRE.md", "fait jusqu'à retrait-gemini (SPEC-mcp-retrait-gemini.md)")]), PERMIS_WEB_SUITE).is_empty());
}

#[test]
fn aucune_trace() {
    let (mut traces, mut vus) = (Vec::new(), Vec::new());
    let depots = [("web_suite", Some(racine()), PERMIS_WEB_SUITE)].into_iter()
        .chain(["WEB_CAO", "WEB_ANTENNA", "WEB_3D"].map(|o| (o, depot(o), PERMIS_OUTIL)));
    for (nom, d, permis) in depots {
        let Some(d) = d else {
            println!("({nom} non cloné : pas vérifié)");
            continue;
        };
        vus.push(nom);
        traces.extend(chercher(&fichiers_suivis(&d), permis).into_iter().map(|(chemin, n, ligne)| format!("{nom}/{chemin}:{n} : {ligne}")));
    }
    for t in &traces {
        println!("TRACE : {t}");
    }
    assert!(traces.is_empty(), "{} trace(s) de l'ancien assistant IA : {traces:?}", traces.len());
    println!("OK : aucune trace de l'ancien assistant IA dans {}", vus.join(", "));
}
