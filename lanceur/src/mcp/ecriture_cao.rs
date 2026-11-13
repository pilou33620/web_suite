//! Outil MCP cao_schema_dessiner : dessiner dans le schéma ouvert, en une carte.
//!
//! Spec : lanceur/docs/SPEC-mcp-dessin-schema.md. L'outil vérifie la forme des opérations (clés connues,
//! types, textes d'une ligne), puis les dépose par le canal de pilotage (pilote::envoyer, écriture).
//! La page (WEB_CAO/editeur-schematique/js/28-dessin-pilote.js, sur le socle commun/ecriture-cao.js) les
//! simule, montre la carte et applique au clic, en une étape d'historique. Toute autre écriture dans une
//! page passe par pilote_interface (décision du 2026-11-04).

use serde_json::{json, Value};

use super::api_outils::resumer;
use super::pilote;
use super::py::{self, Objet};
use super::{outil, refuser, ErreurOutil, OutilMcp, Rendu};

/// (contrôle, ce qu'on attend) : le message de refus le dit
#[derive(Clone, Copy)]
pub struct Controle(pub fn(&Value) -> bool, pub &'static str);

/// Les clés permises d'un objet, dans l'ordre où le message de refus les cite.
pub type Champs = [(&'static str, Controle)];

fn texte(v: &Value) -> bool {
    v.as_str().is_some_and(|t| py::une_ligne(t, 0, 200, true))
}

fn nombre(v: &Value) -> bool {
    py::reel(v).is_some_and(|f| f.abs() <= 1e6)
}

fn nom(v: &Value) -> bool {
    texte(v) && v.as_str().is_some_and(|t| !py::strip(t).is_empty())
}

pub fn parmi(v: &Value, choix: &[&str]) -> bool {
    v.as_str().is_some_and(|t| choix.contains(&t))
}

pub const TEXTE: Controle = Controle(texte, "texte d'une ligne, 200 caractères au plus");
pub const NOM: Controle = Controle(nom, "texte d'une ligne non vide, 200 caractères au plus");
pub const NOMBRE: Controle = Controle(nombre, "nombre fini");
pub const BOOLEEN: Controle = Controle(Value::is_boolean, "true ou false");

// dessin-schema : les clés de chaque opération (la page vérifie les valeurs)
const DESSIN_MAX: usize = 40;
const ENTIER: Controle = Controle(|v| py::entier(v).is_some_and(|n| (1..=1000).contains(&n)), "entier de 1 à 1000");
const ANGLE: Controle = Controle(|v| py::entier(v).is_some_and(|n| [0, 90, 180, 270].contains(&n)), "0, 90, 180 ou 270");
const GENRE: Controle = Controle(|v| parmi(v, &["local", "global", "vcc", "gnd"]), "local ou global ou vcc ou gnd");
const POINTS_FIL: Controle = Controle(
    |v| v.as_array().is_some_and(|l| (2..=50).contains(&l.len()) && l.iter().all(|p| p.as_object().is_some_and(
        |p| p.len() == 2 && p.get("x").is_some_and(nombre) && p.get("y").is_some_and(nombre)))),
    "liste de 2 à 50 points {x, y} en mm");
const F: (&str, Controle) = ("feuille", ENTIER);
const OPERATIONS: [(&str, &Champs); 12] = [
    ("poser", &[F, ("type", NOM), ("catalogue", NOM), ("ref", NOM), ("valeur", NOM), ("x", NOMBRE), ("y", NOMBRE),
                ("rotation", ANGLE), ("miroir", BOOLEEN)]),
    ("deplacer", &[F, ("ref", NOM), ("x", NOMBRE), ("y", NOMBRE), ("dx", NOMBRE), ("dy", NOMBRE)]),
    ("tourner", &[F, ("ref", NOM), ("angle", ANGLE)]),
    ("miroir", &[F, ("ref", NOM)]),
    ("supprimer", &[F, ("ref", NOM)]),
    ("relier", &[F, ("de", NOM), ("a", NOM)]),
    ("fil", &[F, ("points", POINTS_FIL)]),
    ("supprimer_fil", &[F, ("de", NOM), ("a", NOM)]),
    ("etiqueter", &[F, ("broche", NOM), ("net", NOM), ("genre", GENRE)]),
    ("feuille", &[("creer", NOM), ("renommer", ENTIER), ("nom", NOM)]),
    ("renumeroter", &[]),
    ("cadre", &[F, ("x1", NOMBRE), ("y1", NOMBRE), ("x2", NOMBRE), ("y2", NOMBRE), ("titre", TEXTE), ("zone", BOOLEEN)]),
];

pub fn valider(element: &Value, champs: &Champs, requis: &[&str], ou: &str) -> Result<(), ErreurOutil> {
    let Some(element) = element.as_object() else {
        return refuser(format!("{ou} : un objet attendu."));
    };
    if let Some(k) = requis.iter().find(|k| !element.contains_key(**k)) {
        return refuser(format!("{ou} : « {k} » manquant."));
    }
    for (k, v) in element {
        let Some((_, Controle(controle, attendu))) = champs.iter().find(|(nom, _)| nom == k) else {
            let connues: Vec<&str> = champs.iter().map(|(nom, _)| *nom).collect();
            return refuser(format!("{ou} : clé « {k} » inconnue ({}).", connues.join(", ")));
        };
        if !controle(v) {
            return refuser(format!("{ou} : « {k} » invalide ({attendu})."));
        }
    }
    Ok(())
}

fn envoyer(pages: &[&str], nom: &str, liste: &Value, a: &Objet) -> Rendu {
    // la réponse reprend les lignes de la carte (textes du projet) : résumée au-delà de 100 Ko
    Ok(resumer(pilote::envoyer("web_cao", pages, nom, json!({"lot": liste}), true, pilote::onglet(a)?)?,
               "Lot trop gros pour la réponse : la carte est dans la page ; relis avec cao_*_lire."))
}

fn cao_schema_dessiner(a: &Objet) -> Rendu {
    let Some(ops) = a.get("operations").filter(|o| o.as_array().is_some_and(|o| (1..=DESSIN_MAX).contains(&o.len()))) else {
        return refuser(format!("Argument « operations » invalide : liste de 1 à {DESSIN_MAX} opérations (une carte)."));
    };
    for (i, o) in ops.as_array().into_iter().flatten().enumerate() {
        let i = i + 1;
        let op = o.as_object().and_then(|o| o.get("op")).and_then(Value::as_str);
        let Some((op, champs)) = op.and_then(|op| OPERATIONS.iter().find(|(nom, _)| *nom == op)) else {
            let connues: Vec<&str> = OPERATIONS.iter().map(|(nom, _)| *nom).collect();
            return refuser(format!("operations, élément {i} : « op » parmi {}.", connues.join(", ")));
        };
        let mut champs = champs.to_vec();
        champs.push(("op", Controle(|_| true, "")));
        valider(o, &champs, &[], &format!("operations, élément {i} ({op})"))?;
    }
    envoyer(&["schema"], "cao_schema_dessiner", ops, a)
}

pub fn outils() -> Vec<OutilMcp> {
    let onglet = pilote::onglet_doc();
    let operations: Vec<&str> = OPERATIONS.iter().map(|(nom, _)| *nom).collect();

    vec![
        outil("cao_schema_dessiner",
              &["Dessine dans le schéma ouvert, en une carte : une liste ordonnée d'opérations (40 au ",
                "plus), simulée par la page avant la carte puis appliquée en une étape d'historique ",
                "(Ctrl+Z la défait). Opérations (« op ») : poser {type (symbole intégré : resistor, ",
                "capacitor, ic, opamp, regulator…) ou catalogue (Part Name de lib_composants), ref, ",
                "valeur, x, y (mm ; sans position, place libre), rotation, miroir} ; deplacer {ref, x, y ",
                "ou dx, dy} ; tourner {ref, angle} ; miroir {ref} ; supprimer {ref} ; relier {de, a : ",
                "« R1.2 » ou « U1.OUT »} ; fil {points : [{x, y}…] en mm, segments horizontaux ou ",
                "verticaux ; il ne relie que par ses deux bouts (broche, fil ou rien)} ; supprimer_fil {de, a} ; etiqueter {broche, net, genre : local, ",
                "global, vcc, gnd} ; feuille {creer} ou {renommer, nom} ; renumeroter (tout le document) ; ",
                "cadre {x1, y1, x2, y2, titre?, zone? : true = zone fonctionnelle, une room au PCB ; sinon ",
                "un cadre de note}. Texte libre : poser {type: annot_text (ou annot_note, encadrée), valeur ; ",
                "« | » sépare les lignes}. ",
                "Les autres opérations peuvent viser une « feuille » (n°), sinon la feuille affichée. Les ",
                "étiquettes et alimentations n'ont pas de repère : « #id » les désigne (rendu par etiqueter, ou ",
                "cao_schema_lire etiquettes) pour deplacer, tourner, miroir, supprimer. Un repère posé sert aux ",
                "opérations suivantes. Un fil qui relierait d'autres broches que celles demandées est ",
                "refusé : étiquette alors. Un seul refus et rien ne change.",
                pilote::ATTENTE_DOC, pilote::DONNEES],
              json!({"type": "object", "required": ["operations"], "properties": {
                  "operations": {"type": "array", "minItems": 1, "maxItems": DESSIN_MAX, "items": {
                      "type": "object", "required": ["op"], "properties": {"op": {"type": "string", "enum": operations}}}},
                  "onglet": onglet}}),
              cao_schema_dessiner),
    ]
}
