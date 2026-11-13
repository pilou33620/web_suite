//! L'aide des commandes de pilotage, pour la page : WEB_CAO/commun/pilote-aide.json.
//!
//! Un script qui pilote une page depuis sa console (sans le MCP branché) ne voit pas les
//! descriptions ni les schémas d'arguments des outils : `Pilote.aide("cao_schema_dessiner")`
//! (WEB_CAO/commun/pilote-cao.js) les lit dans ce fichier. Il est TIRÉ des définitions du MCP
//! (outils_mcp), une seule source : relancer cet exécutable après avoir changé un outil cao_* ou lib_*.
//! lanceur/tests/banc_mcp.rs échoue tant que le fichier n'est pas à jour, et la CI de web_suite aussi
//! (sur son clone de WEB_CAO) : pousser WEB_CAO avant web_suite.
//!
//!     aide_pilote             écrit le fichier
//!     aide_pilote --stdout    écrit son contenu sur la sortie standard, sans toucher au fichier (bancs, CI)

use std::io::Write;

use serde_json::{json, Value};
use web_suite_lanceur::{catalogue, mcp};

fn contenu() -> String {
    let outils: serde_json::Map<String, Value> = mcp::outils_mcp().iter()
        .filter(|o| o.nom.starts_with("cao_") || o.nom.starts_with("lib_"))
        .map(|o| (o.nom.to_string(), json!({"description": o.description, "arguments": o.schema}))).collect();
    // « source » nomme toujours l'ancien script : le fichier de WEB_CAO reste le même à l'octet
    let aide = json!({"source": "web_suite/lanceur/aide_pilote.py (définitions du MCP)", "outils": outils});
    mcp::py::dumps_indente_trie(&aide, 1) + "\n"
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--stdout") => {
            std::io::stdout().write_all(contenu().as_bytes()).expect("sortie standard fermée");
        }
        Some(autre) => {
            eprintln!("Argument inconnu : {autre}. Usage : aide_pilote [--stdout]");
            std::process::exit(2);
        }
        None => {
            let fichier = catalogue::depot().join("WEB_CAO").join("commun").join("pilote-aide.json");
            if let Err(e) = std::fs::write(&fichier, contenu()) {
                eprintln!("{} : {e}", fichier.display());
                std::process::exit(1);
            }
            println!("écrit : {}", fichier.display());
        }
    }
}
