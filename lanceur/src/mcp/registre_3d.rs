//! Outils MCP du module registre-3d : commander la page WEB_3D ouverte.
//!
//! Spec : lanceur/docs/SPEC-mcp-registre-3d.md. Chaque outil valide ses arguments, puis
//! dépose sa commande à la page « 3d » par le canal de pilotage (pilote::envoyer) ; la
//! page (WEB_3D/js/14-pilote-3d.js) fait le calcul.

use serde_json::{json, Value};

use super::api_outils::{resumer, texte_facultatif};
use super::py::{self, py_repr, Objet};
use super::{outil, pilote, plafonner, refuser, ErreurOutil, OutilMcp, Rendu};

const VUES: [&str; 7] = ["avant", "arriere", "gauche", "droite", "dessus", "dessous", "iso"];
const ACTIONS: [&str; 5] = ["masquer", "afficher", "isoler", "tout_afficher", "selectionner"];
const QUOI: [&str; 3] = ["dimensions", "distance", "faces"];
const MM: f64 = 1e6; // mm : au-delà, la vue casse (positions infinies)
const PROCEDES: [&str; 3] = ["fdm", "injection", "tolerie"];

/// uuid, chemin « 0/3/1 », ou nom unique
fn ref_(v: Option<&Value>, nom: &str) -> Result<Value, ErreurOutil> {
    match v {
        Some(Value::String(t)) if py::une_ligne(t, 1, 200, false) && !py::strip(t).is_empty() => Ok(json!(t)),
        _ => refuser(format!("Argument « {nom} » invalide : uuid, chemin (0/3/1) ou nom unique d'une pièce.")),
    }
}

/// `entre` : les bornes, écrites comme Python les écrivait (« -1000000.0 », « 1e-06 »).
fn nombre(v: &Value, nom: &str, mini: f64, maxi: f64, entre: &str) -> Result<Value, ErreurOutil> {
    if !py::reel(v).is_some_and(|f| mini <= f && f <= maxi) {
        return refuser(format!("Argument « {nom} » invalide : nombre fini entre {entre}."));
    }
    Ok(v.clone())
}

/// Un vrai booléen JSON : en Python, bool("false") vaut True.
pub fn booleen(a: &Objet, nom: &str) -> Result<bool, ErreurOutil> {
    match a.get(nom) {
        None => Ok(false),
        Some(Value::Bool(v)) => Ok(*v),
        Some(_) => refuser(format!("Argument « {nom} » invalide : true ou false.")),
    }
}

pub fn choix<'a>(v: Option<&Value>, nom: &str, possibles: &[&'a str]) -> Result<&'a str, ErreurOutil> {
    match v.and_then(Value::as_str).and_then(|t| possibles.iter().find(|p| **p == t)) {
        Some(p) => Ok(p),
        None => refuser(format!("Argument « {nom} » invalide : {}.", possibles.join(", "))),
    }
}

fn index(v: Option<&Value>, nom: &str) -> Result<i64, ErreurOutil> {
    match v.and_then(py::entier).filter(|n| *n >= 0) {
        Some(n) => Ok(n),
        None => refuser(format!("Argument « {nom} » invalide : entier positif ou nul.")),
    }
}

/// {piece} ou {fichier}, pour cadrer ou mesurer.
fn cible(a: &Value) -> Result<Objet, ErreurOutil> {
    let Some(a) = a.as_object() else {
        return refuser("Cible invalide : {\"piece\": …} ou {\"fichier\": n}.");
    };
    let mut rendu = Objet::new();
    if a.contains_key("piece") {
        rendu.insert("piece".into(), ref_(a.get("piece"), "piece")?);
    } else if a.contains_key("fichier") {
        rendu.insert("fichier".into(), json!(index(a.get("fichier"), "fichier")?));
    }
    Ok(rendu)
}

/// Présent et pas nul : « a.get(cle) is not None ».
pub fn donne<'a>(a: &'a Objet, cle: &str) -> Option<&'a Value> {
    a.get(cle).filter(|v| !v.is_null())
}

fn envoyer(nom: &str, args: Value, ecriture: bool, a: &Objet) -> Rendu {
    pilote::envoyer("web_3d", &["3d"], nom, args, ecriture, pilote::onglet(a)?)
}

fn web3d_regles(a: &Objet) -> Rendu {
    let procede = texte_facultatif(a, "procede")?.unwrap_or("fdm");
    if !PROCEDES.contains(&procede) {
        return refuser(format!("Procédé inconnu : {procede} ({}).", PROCEDES.join(", ")));
    }
    pilote::envoyer("web_3d", &["conception"], "web3d_regles", json!({"procede": procede}), false, pilote::onglet(a)?)
}

fn web3d_scene(a: &Objet) -> Rendu {
    let nb = index(a.get("nombre").or(Some(&json!(200))), "nombre")?;
    if nb > 1000 {
        return refuser("Argument « nombre » invalide : 1000 au plus (paginer avec « debut »).");
    }
    let args = json!({"filtre": texte_facultatif(a, "filtre")?.unwrap_or(""),
                      "debut": index(a.get("debut").or(Some(&json!(0))), "debut")?, "nombre": nb});
    envoyer("web3d_scene", args, false, a)
}

fn web3d_vue(a: &Objet) -> Rendu {
    let mut args = Objet::new();
    if let Some(v) = donne(a, "vue") {
        args.insert("vue".into(), json!(choix(Some(v), "vue", &VUES)?));
    }
    if let Some(v) = donne(a, "projection") {
        args.insert("projection".into(), json!(choix(Some(v), "projection", &["ortho", "perspective"])?));
    }
    if let Some(v) = donne(a, "cadrer") {
        args.insert("cadrer".into(), Value::Object(cible(v)?));
    }
    let capturer = booleen(a, "capturer")?;
    let vue = envoyer("web3d_vue", Value::Object(args), false, a)?;
    if !capturer || vue["etat"] != "finie" {
        return Ok(vue);
    }
    // La capture du même onglet, dans la foulée : l'image et la vue en un appel
    let onglet = vue.get("onglet").ok_or_else(|| ErreurOutil::new("'onglet'"))?;
    let id = onglet.get("id").ok_or_else(|| ErreurOutil::new("'id'"))?;
    pilote::capture_de_la_vue(&vue, json!({"outil": "web_3d", "page": "3d", "onglet": id}))
}

fn web3d_visibilite(a: &Objet) -> Rendu {
    let action = choix(a.get("action"), "action", &ACTIONS)?;
    let refs = match a.get("pieces") {
        None => Vec::new(),
        Some(Value::Array(refs)) if refs.len() <= 500 => refs.iter().map(|r| ref_(Some(r), "piece")).collect::<Result<_, _>>()?,
        Some(_) => return refuser("Argument « pieces » invalide : liste de références."),
    };
    envoyer("web3d_visibilite", json!({"action": action, "pieces": refs}), false, a)
}

fn web3d_coupe(a: &Objet) -> Rendu {
    let mut args = Objet::new();
    if a.contains_key("actif") {
        args.insert("actif".into(), json!(booleen(a, "actif")?));
    }
    if let Some(v) = donne(a, "axe") {
        args.insert("axe".into(), json!(choix(Some(v), "axe", &["x", "y", "z"])?));
    }
    if let Some(f) = donne(a, "face") {
        let Some(f) = f.as_object() else {
            return refuser("Argument « face » invalide : {\"piece\": …, \"index\": n}.");
        };
        args.insert("face".into(), json!({"piece": ref_(f.get("piece"), "piece")?, "index": index(f.get("index"), "face.index")?}));
    }
    if let Some(v) = donne(a, "decalage_mm") {
        args.insert("decalage_mm".into(), nombre(v, "decalage_mm", -MM, MM, "-1000000.0 et 1000000.0")?);
    }
    if a.contains_key("inverse") {
        args.insert("inverse".into(), json!(booleen(a, "inverse")?));
    }
    envoyer("web3d_coupe", Value::Object(args), false, a)
}

fn web3d_mesurer(a: &Objet) -> Rendu {
    if booleen(a, "effacer")? {
        return envoyer("web3d_mesurer", json!({"effacer": true}), false, a);
    }
    let quoi = choix(a.get("quoi").or(Some(&json!("dimensions"))), "quoi", &QUOI)?;
    let mut args = Objet::new();
    args.insert("quoi".into(), json!(quoi));
    args.insert("afficher".into(), json!(booleen(a, "afficher")?));
    match quoi {
        "distance" => {
            let Some(refs) = a.get("pieces").and_then(Value::as_array).filter(|r| r.len() == 2) else {
                return refuser("distance : deux pièces dans « pieces ».");
            };
            args.insert("pieces".into(), Value::Array(refs.iter().map(|r| ref_(Some(r), "piece")).collect::<Result<_, _>>()?));
        }
        "faces" => {
            args.insert("piece".into(), ref_(a.get("piece"), "piece")?);
        }
        _ => {
            args.extend(cible(&Value::Object(a.iter().filter(|(k, _)| *k == "piece" || *k == "fichier")
                .map(|(k, v)| (k.clone(), v.clone())).collect()))?);
            if args.len() == 2 {
                return refuser("dimensions : « piece » ou « fichier » ; la fiche ne mesure pas la scène entière.");
            }
        }
    }
    Ok(resumer(envoyer("web3d_mesurer", Value::Object(args), false, a)?, "Trop de faces : demande une autre pièce."))
}

fn web3d_collisions(a: &Objet) -> Rendu {
    let mut args = Objet::new();
    args.insert("afficher".into(), json!(booleen(a, "afficher")?));
    if let Some(v) = donne(a, "tolerance") { // absente : la préférence de la page, comme le bouton
        args.insert("tolerance".into(), nombre(v, "tolerance", 1e-6, 10.0, "1e-06 et 10")?);
    }
    envoyer("web3d_collisions", Value::Object(args), false, a)
}

fn web3d_ouvrir(a: &Objet) -> Rendu {
    let Some(chemins) = a.get("chemins").and_then(Value::as_array).filter(|c| (1..=20).contains(&c.len())) else {
        return refuser("Argument « chemins » invalide : 1 à 20 fichiers de PROJETS/3D (voir web3d_projets).");
    };
    for c in chemins {
        let bon = c.as_str().is_some_and(|c| !c.is_empty() && !c.starts_with('/') && !c.contains('\\') && !c.contains(':')
            && !c.split('/').any(|b| b.is_empty() || b == "." || b == ".."));
        if !bon {
            return refuser(format!("Chemin invalide : {} (relatif à PROJETS/3D, tel que le rend web3d_projets).", py_repr(c)));
        }
    }
    envoyer("web3d_ouvrir", json!({"chemins": chemins, "ajout": booleen(a, "ajout")?}), true, a)
}

const CARTE: &str = concat!(" Carte dans la page : rien ne change avant le clic de l'utilisateur (sauf interrupteur allumé) ; ",
                            "l'appel attend jusqu'à 2 min, puis rend « en_attente » (pilote_suivre).");

pub fn outils() -> Vec<OutilMcp> {
    let onglet = pilote::onglet_doc();
    let ref_doc = json!({"type": "string", "description": "Pièce : uuid ou chemin (0/3/1) rendus par web3d_scene ; un nom seulement s'il est unique."});

    let mut table = vec![
        outil("web3d_scene",
              &["Scène WEB_3D ouverte : fichiers (index, position, rotation) et pièces (uuid, chemin, nom, ",
                "visible, couleur), avec filtre sur le nom et pagination ; sélection, coupe, projection."],
              json!({"type": "object", "properties": {"filtre": {"type": "string"}, "debut": {"type": "integer"},
                                                      "nombre": {"type": "integer"}, "onglet": onglet}}),
              web3d_scene),
        outil("web3d_vue",
              &["Règle la vue 3D (avant, arriere, gauche, droite, dessus, dessous, iso), la projection, et ",
                "cadre une pièce ou un fichier ; capturer: true rend l'image dans la foulée. La projection s'écrit dans ",
                "les préférences, comme le bouton de la page."],
              json!({"type": "object", "properties": {
                  "vue": {"type": "string", "enum": VUES}, "projection": {"type": "string", "enum": ["ortho", "perspective"]},
                  "cadrer": {"type": "object", "description": "{\"piece\": ref} ou {\"fichier\": n}"},
                  "capturer": {"type": "boolean"}, "onglet": onglet}}),
              web3d_vue),
        outil("web3d_visibilite",
              &["Masquer, afficher, isoler, tout afficher ou sélectionner des pièces dans la vue 3D."],
              json!({"type": "object", "properties": {
                  "action": {"type": "string", "enum": ACTIONS}, "pieces": {"type": "array", "items": ref_doc},
                  "onglet": onglet}, "required": ["action"]}),
              web3d_visibilite),
        outil("web3d_coupe",
              &["Plan de coupe : actif, axe x/y/z ou face plane d'une pièce visible (index de web3d_mesurer faces), ",
                "décalage en mm (borné par la page), sens inversé."],
              json!({"type": "object", "properties": {
                  "actif": {"type": "boolean"}, "axe": {"type": "string", "enum": ["x", "y", "z"]},
                  "face": {"type": "object", "description": "{\"piece\": ref, \"index\": n}"},
                  "decalage_mm": {"type": "number"}, "inverse": {"type": "boolean"}, "onglet": onglet}}),
              web3d_coupe),
        outil("web3d_mesurer",
              &["Mesures en mm (repère du monde), ce que montre la page : dimensions (encombrement, et volume et surface ",
                "jusqu'à 800 000 triangles) d'une pièce ou d'un fichier ; distance minimale entre deux pièces visibles ",
                "(afficher: true pose la cote dans la vue ; effacer: true, ou Effacer la mesure dans la page, la retire) ; ",
                "faces d'une pièce visible (index ; plan : aire, normale ; cylindre : rayon, hauteur, axe, centre ; ",
                "autre : aire)."],
              json!({"type": "object", "properties": {
                  "quoi": {"type": "string", "enum": QUOI}, "piece": ref_doc, "fichier": {"type": "integer"},
                  "pieces": {"type": "array", "items": ref_doc}, "afficher": {"type": "boolean"},
                  "effacer": {"type": "boolean"}, "onglet": onglet}}),
              web3d_mesurer),
        outil("web3d_collisions",
              &["Interférences entre pièces visibles (surfaces qui se croisent ; un contact ne compte pas) : ",
                "couples et longueur des courbes ; contenue: true quand une pièce est noyée tout entière ",
                "dans l'autre (aucune courbe, longueur 0). afficher: true dessine les courbes dans la vue et écrit la ",
                "tolérance dans les préférences, comme le champ de la page."],
              json!({"type": "object", "properties": {
                  "tolerance": {"type": "number", "description": "mm ; absente : celle de la page"}, "afficher": {"type": "boolean"},
                  "onglet": onglet}}),
              web3d_collisions),
        outil("web3d_ouvrir",
              &["Ouvre des fichiers de PROJETS/3D (chemins de web3d_projets) dans la page, en remplacement ou en ",
                "ajout ; un .obj emmène son .mtl. Annuler recharge le modèle d'avant (ou retire l'ajout).", CARTE],
              json!({"type": "object", "properties": {
                  "chemins": {"type": "array", "items": {"type": "string"}}, "ajout": {"type": "boolean"}, "onglet": onglet},
                  "required": ["chemins"]}),
              web3d_ouvrir),
        outil("web3d_regles",
              &["Page de conception : contrôle les règles de conception vérifiables, comme le bouton Règles de ",
                "l'onglet Inspecter : esquisses entièrement contraintes sans Fixe posé à la main, noms des paramètres, ",
                "et en FDM la grille (hauteurs × 0,2 mm, parois × 0,5 mm). Ne couvre pas le reste des règles."],
              json!({"type": "object", "properties": {
                  "procede": {"type": "string", "enum": PROCEDES, "description": "fdm si absent"}, "onglet": onglet}}),
              web3d_regles),
    ];
    for o in &mut table {
        o.description += pilote::DONNEES;
        if o.nom != "web3d_vue" { // la vue avec capture garde son image (rendue à part)
            plafonner(o);
        }
    }
    table
}
