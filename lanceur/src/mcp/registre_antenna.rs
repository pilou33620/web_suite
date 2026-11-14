//! Outils MCP du module registre-antenna : lire la page WEB_ANTENNA ouverte et régler sa vue.
//!
//! Spec : lanceur/docs/SPEC-mcp-registre-antenna.md. Chaque outil valide ses arguments, puis dépose sa
//! commande à la page « antenna » par le canal de pilotage (pilote::envoyer, sans écriture) ; la page
//! (WEB_ANTENNA/js/pilote-antenna.js, lectures dans js/29-lecture-antenna.js) répond en texte. Rien
//! n'est écrit, rien n'est lancé : les calculs openEMS restent à lancer depuis la page.

use serde_json::{json, Value};

use super::api_outils::resumer;
use super::pilote;
use super::py::Objet;
use super::registre_3d::{booleen, choix, donne};
use super::registre_cao::{entier, texte};
use super::{outil, refuser, OutilMcp, Rendu};

const QUOI: [&str; 10] = ["conception", "resultats", "courbe", "balayage", "audit", "reglages", "composants", "composant", "nets", "net"];
const GRANDEURS: [&str; 4] = ["s11", "s21", "z", "roe"];
const ONGLETS: [&str; 7] = ["bal", "ts", "s11", "s21", "roe", "z", "ff"];
const TOILES: [&str; 3] = ["carte", "3d", "courbe"];
const CHIFFRAGES: [&str; 4] = ["modele", "balayage", "tableau_s", "script"];
const CONSEIL: &str = "Trop long : précise le filtre, ou demande un seul composant, un seul net ou une seule grandeur.";

fn envoyer(nom: &str, args: Objet, a: &Objet) -> Rendu {
    pilote::envoyer("web_antenna", &["antenna"], nom, Value::Object(args), false, pilote::onglet(a)?)
}

fn antenna_lire(a: &Objet) -> Rendu {
    let mut args = Objet::new();
    args.insert("quoi".into(), json!(choix(a.get("quoi"), "quoi", &QUOI)?));
    if let Some(v) = donne(a, "grandeur") {
        args.insert("grandeur".into(), json!(choix(Some(v), "grandeur", &GRANDEURS)?));
    }
    for cle in ["filtre", "ref", "nom"] {
        if let Some(v) = texte(a, cle)? {
            args.insert(cle.into(), v.clone());
        }
    }
    Ok(resumer(envoyer("antenna_lire", args, a)?, CONSEIL))
}

fn antenna_vue(a: &Objet) -> Rendu {
    let mut args = Objet::new();
    if let Some(v) = donne(a, "vue") {
        args.insert("vue".into(), json!(choix(Some(v), "vue", &["2d", "3d"])?));
    }
    if donne(a, "maillage").is_some() {
        args.insert("maillage".into(), json!(booleen(a, "maillage")?));
    }
    if booleen(a, "ajuster")? {
        args.insert("ajuster".into(), json!(true));
    }
    // « onglet » désigne l'onglet du navigateur dans tout le canal : l'onglet de résultats s'appelle ici « resultats »
    if let Some(v) = donne(a, "resultats") {
        args.insert("onglet".into(), json!(choix(Some(v), "resultats", &ONGLETS)?));
    }
    if let Some(v) = entier(a, "point_balayage", 100000)? { // un balayage croisé dépasse vite 1000 points
        args.insert("point_balayage".into(), v.clone());
    }
    if let Some(v) = entier(a, "etape", 20)? {
        args.insert("etape".into(), v.clone());
    }
    if let Some(v) = texte(a, "panneau")? {
        args.insert("panneau".into(), v.clone());
    }
    if let Some(v) = donne(a, "toile") {
        args.insert("toile".into(), json!(choix(Some(v), "toile", &TOILES)?));
    }
    let capturer = booleen(a, "capturer")?;
    if args.is_empty() {
        return refuser("Rien à faire : vue, maillage, ajuster, resultats, point_balayage, etape, panneau ou toile.");
    }
    let vue = envoyer("antenna_vue", args, a)?;
    if !capturer || vue["etat"] != "finie" {
        return Ok(vue);
    }
    // la capture du même onglet, dans la foulée : l'image et la vue en un appel
    pilote::capture_de_la_vue(&vue, json!({"outil": "web_antenna", "onglet": vue["onglet"]["id"]}))
}

fn antenna_openems_chiffrer(a: &Objet) -> Rendu {
    let mut args = Objet::new();
    args.insert("quoi".into(), json!(choix(a.get("quoi"), "quoi", &CHIFFRAGES)?));
    Ok(resumer(envoyer("antenna_openems_chiffrer", args, a)?, "Réponse trop grosse."))
}

pub fn outils() -> Vec<OutilMcp> {
    let onglet = pilote::onglet_doc();
    let t = json!({"type": "string"});

    vec![
        outil("antenna_lire",
              &["Lit la page WEB_ANTENNA ouverte, ce qui n'est pas encore enregistré compris : conception ",
                "(le résumé de l'assistant : carte, empilage, bande, ports, boîte, maillage, motif et cotes, ",
                "chiffrage du serveur), resultats (f0, S11, Z, bande à -10 dB, Dmax, couplages), courbe ",
                "(grandeur s11, s21, z ou roe ; 200 points au plus), balayage (tableau du balayage fini), ",
                "audit (contrôles hors ligne, chacun avec la correction proposée), reglages ",
                "(réglages de l'assistant : chemin, valeur, unité, bornes, choix), et pour la carte ",
                "importée composants, composant, nets, net. Ne modifie rien, ne lance aucun calcul.",
                pilote::DONNEES],
              json!({"type": "object", "required": ["quoi"], "properties": {
                  "quoi": {"type": "string", "enum": QUOI}, "grandeur": {"type": "string", "enum": GRANDEURS},
                  "filtre": t, "ref": t, "nom": t, "onglet": onglet}}),
              antenna_lire),
        outil("antenna_vue",
              &["Règle la vue de la page WEB_ANTENNA, sans rien changer au projet ni aux préférences : vue ",
                "(2d, 3d), maillage (montré ou caché), ajuster (2D), resultats (onglet de résultats : bal, ts, ",
                "s11, s21, roe, z, ff), point_balayage (n°), etape (n° de l'étape de l'assistant ; le projet ",
                "enregistre celle de l'utilisateur), panneau (assistant, resultats, couches…), toile ",
                "à capturer (carte ou 3d : la vue affichée ; courbe : ouvre le panneau des résultats s'il est caché). ",
                "Un panneau montré ne s'enregistre pas dans la disposition tant que l'utilisateur n'y touche pas. capturer=true rend ",
                "l'image dans la foulée. Tout est vérifié ",
                "avant le premier changement. Jamais de calcul lancé.", pilote::DONNEES],
              json!({"type": "object", "properties": {
                  "vue": {"type": "string", "enum": ["2d", "3d"]}, "maillage": {"type": "boolean"}, "ajuster": {"type": "boolean"},
                  "resultats": {"type": "string", "enum": ONGLETS}, "point_balayage": {"type": "integer", "minimum": 1},
                  "etape": {"type": "integer", "minimum": 1}, "panneau": t, "toile": {"type": "string", "enum": TOILES},
                  "capturer": {"type": "boolean"}, "onglet": onglet}}),
              antenna_vue),
        outil("antenna_openems_chiffrer",
              &["Vérifie et chiffre le modèle de la page WEB_ANTENNA ouverte, comme ses devis, sans rien lancer : ",
                "modele (modèle complété, estimation, avis), balayage (points du balayage réglé), tableau_s ",
                "(matrice S complète), script (fichier d'entrée d'openEMS, autonome). Lancer un calcul reste à faire depuis la page.",
                pilote::DONNEES],
              json!({"type": "object", "required": ["quoi"], "properties": {
                  "quoi": {"type": "string", "enum": CHIFFRAGES}, "onglet": onglet}}),
              antenna_openems_chiffrer),
    ]
}
