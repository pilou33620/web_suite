//! Le lecteur IPC-2581 partagé par WEB_CAO et WEB_ANTENNA : le portage en Rust de
//! python/ipc2581_data.py, ipc2581_parser.py (1.84) et ipc2581_json.py (1.5.0).
//!
//!     let modele = ipc2581::ipc2581_en_dict(&octets, "carte.xml", true)?;   // XML ou ZIP
//!     let texte = ipc2581::dumps(&modele);                                  // le JSON des pages
//!
//! Le JSON rendu est, à l'octet, celui de `json.dumps(modele, ensure_ascii=False)` du lecteur
//! Python (essais/ garde les sorties de référence). Bibliothèque pure : ni journal, ni disque
//! hors `IPC2581Parser::parse_fichier`.

pub mod data;
pub mod json;
pub mod parser;
pub mod py;
mod xml;

#[cfg(test)]
mod essais;

pub use data::*;
pub use json::{
    charger_octets, design_en_dict, design_en_dict_avec, ipc2581_en_dict, ipc2581_en_dict_avec, Erreur, Motifs,
    DECIMALES, EXTENSIONS, FORMAT,
};
pub use parser::{IPC2581ParseError, IPC2581Parser};
pub use py::{dumps, Json, Objet};
