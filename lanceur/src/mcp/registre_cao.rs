//! Outils MCP du module registre-cao : lire les pages WEB_CAO ouvertes, régler leur vue, lancer le DRC.
//!
//! Spec : lanceur/docs/SPEC-mcp-registre-cao.md. Chaque outil valide ses arguments, puis dépose sa
//! commande aux pages WEB_CAO par le canal de pilotage (pilote::envoyer) ; la page
//! (WEB_CAO/commun/pilote-cao.js, lectures dans commun/lecture-cao.js) répond en texte. Rien n'est
//! écrit dans le projet.

use serde_json::{json, Value};

use super::api_outils::resumer;
use super::pilote::{self, DONNEES};
use super::py::{self, Objet};
use super::registre_3d::{booleen, choix, donne};
use super::{outil, refuser, ErreurOutil, OutilMcp, Rendu};

const QUOI: [(&str, &[&str]); 4] = [
    ("schema", &["feuilles", "composants", "composant", "etiquettes", "nets", "net"]),
    ("pcb", &["empilage", "classes", "composants", "composant", "nets", "net", "drc", "liaisons", "zone",
              "carte", "zones", "serigraphie", "interdits"]),
    ("ipc", &["empilage", "composants", "composant", "nets", "net"]),
    ("sim", &["simulations", "simulation", "catalogue"]),
];
const PAGES: [(&str, &[&str]); 4] = [("schema", &["schema"]), ("pcb", &["pcb"]), ("ipc", &["visionneuse-ipc"]),
                                     ("sim", &["pcb", "visionneuse-ipc"])];
const PAGES_VUE: [&str; 3] = ["schema", "pcb", "visionneuse-ipc"];
const PARTIES: [&str; 9] = ["chiffres", "diagnostics", "methode", "piste", "bornes", "reglages", "resultats", "texte", "json"];
const CONSEIL: &str = "Trop long : précise le filtre, ou demande un seul composant ou un seul net.";

fn de(table: &[(&str, &'static [&'static str])], famille: &str) -> &'static [&'static str] {
    table.iter().find(|(f, _)| *f == famille).map_or(&[], |(_, v)| v)
}

/// repère, net, filtre, couche : une ligne, bornée
pub fn texte<'a>(a: &'a Objet, nom: &str) -> Result<Option<&'a Value>, ErreurOutil> {
    match donne(a, nom) {
        None => Ok(None),
        Some(v) if v.as_str().is_some_and(|t| py::une_ligne(t, 1, 200, false)) => Ok(Some(v)),
        Some(_) => refuser(format!("Argument « {nom} » invalide : texte d'une ligne, 200 caractères au plus.")),
    }
}

pub fn entier<'a>(a: &'a Objet, nom: &str, maxi: i64) -> Result<Option<&'a Value>, ErreurOutil> {
    match donne(a, nom) {
        None => Ok(None),
        Some(v) if py::entier(v).is_some_and(|n| (1..=maxi).contains(&n)) => Ok(Some(v)),
        Some(_) => refuser(format!("Argument « {nom} » invalide : entier de 1 à {maxi}.")),
    }
}

fn envoyer(pages: &[&str], nom: &str, args: Objet, a: &Objet) -> Rendu {
    pilote::envoyer("web_cao", pages, nom, Value::Object(args), false, pilote::onglet(a)?)
}

fn coordonnee<'a>(a: &'a Objet, nom: &str) -> Result<Option<&'a Value>, ErreurOutil> {
    match donne(a, nom) {
        None => Ok(None),
        Some(v) if py::reel(v).is_some_and(|f| (-1000.0..=1000.0).contains(&f)) => Ok(Some(v)),
        Some(_) => refuser(format!("Argument « {nom} » invalide : nombre de mm, à ±1000.")),
    }
}

fn lire(famille: &str, a: &Objet, cles: &[&str]) -> Rendu {
    let mut args = Objet::new();
    args.insert("quoi".into(), json!(choix(a.get("quoi"), "quoi", de(&QUOI, famille))?));
    for cle in cles {
        let v = match *cle {
            "a_router" | "afficher" => booleen(a, cle)?.then(|| json!(true)), // faux : pas transmis
            "feuille" => entier(a, cle, 1000)?.cloned(),
            "x1" | "y1" | "x2" | "y2" => coordonnee(a, cle)?.cloned(),
            _ => texte(a, cle)?.cloned(),
        };
        if let Some(v) = v {
            args.insert((*cle).into(), v);
        }
    }
    if let Some(partie) = args.get("partie") {
        choix(Some(partie), "partie", &PARTIES)?;
    }
    Ok(resumer(envoyer(de(&PAGES, famille), &format!("cao_{famille}_lire"), args, a)?, CONSEIL))
}

fn cao_vue(a: &Objet) -> Rendu {
    let pages = match donne(a, "page") {
        Some(page) => vec![choix(Some(page), "page", &PAGES_VUE)?],
        None => PAGES_VUE.to_vec(),
    };
    let mut args = Objet::new();
    for k in ["cadrer", "surligner"] {
        if let Some(v) = texte(a, k)? {
            args.insert(k.into(), v.clone());
        }
    }
    if let Some(v) = entier(a, "feuille", 1000)? {
        args.insert("feuille".into(), v.clone());
    }
    if let Some(c) = donne(a, "couches") {
        let Some(c) = c.as_array().filter(|c| (1..=64).contains(&c.len()) && c.iter().all(Value::is_string)) else {
            return refuser("Argument « couches » invalide : liste de 1 à 64 noms de couches.");
        };
        let mut couches = Vec::new();
        for x in c {
            let mut une = Objet::new();
            une.insert("couche".into(), x.clone());
            couches.push(texte(&une, "couche")?.cloned());
        }
        args.insert("couches".into(), json!(couches));
    }
    if booleen(a, "ajuster")? {
        args.insert("ajuster".into(), json!(true));
    }
    let capturer = booleen(a, "capturer")?;
    if args.is_empty() {
        return refuser("Rien à faire : cadrer, surligner, feuille, couches ou ajuster.");
    }
    let vue = envoyer(&pages, "cao_vue", args, a)?;
    if !capturer || vue["etat"] != "finie" {
        return Ok(vue);
    }
    // La capture du même onglet, dans la foulée : l'image et la vue en un appel
    pilote::capture_de_la_vue(&vue, json!({"outil": "web_cao", "onglet": vue["onglet"]["id"]}))
}

fn cao_thermique(a: &Objet) -> Rendu {
    envoyer(&["pcb"], "cao_thermique", Objet::new(), a)
}

fn cao_drc(a: &Objet) -> Rendu {
    Ok(resumer(envoyer(&["pcb"], "cao_drc", Objet::new(), a)?, "Trop d'erreurs : corrige les premières, puis relance."))
}

fn cao_regles(a: &Objet) -> Rendu {
    envoyer(&["schema", "pcb"], "cao_regles", Objet::new(), a)
}

/// Le schéma d'entrée d'une lecture : les propriétés communes, « quoi », puis celles de la famille.
fn schema_lire(famille: &str, extra: Value) -> Value {
    let mut props = json!({
        "filtre": {"type": "string", "description": "Sous-chaîne, sans tenir compte de la casse (repère, valeur, boîtier, nom de net)."},
        "ref": {"type": "string", "description": "Repère d'un composant (quoi = composant)."},
        "nom": {"type": "string", "description": "Nom d'un net (quoi = net)."},
        "onglet": pilote::onglet_doc(),
        "quoi": {"type": "string", "enum": de(&QUOI, famille)},
    });
    if let (Some(props), Value::Object(extra)) = (props.as_object_mut(), extra) {
        props.extend(extra);
    }
    json!({"type": "object", "properties": props, "required": ["quoi"]})
}

pub fn outils() -> Vec<OutilMcp> {
    let onglet = pilote::onglet_doc();
    let fenetre = json!({"type": "number", "description": "zone : fenêtre en mm"});

    vec![
        outil("cao_schema_lire",
              &["Lit l'éditeur schématique ouvert (toutes les feuilles) : feuilles, composants (repère, valeur, ",
                "boîtier, référence fabricant, feuille, position en mm), composant (net de chaque broche), ",
                "etiquettes (étiquettes de net et alimentations : #id, genre, net, feuille, position), ",
                "nets (global ou local, broches, feuilles), net (broches REPÈRE.patte). Ne modifie rien.", DONNEES],
              schema_lire("schema", json!({"feuille": {"type": "integer", "minimum": 1,
                                                       "description": "Feuille (1, 2…) : départage deux nets locaux de même nom."}})),
              |a| lire("schema", a, &["filtre", "ref", "nom", "feuille"])),
        outil("cao_pcb_lire",
              &["Lit l'éditeur PCB ouvert : empilage, classes et règles, composants, composant (net de chaque ",
                "broche), nets (état du routage), net (longueur et largeurs par couche, vias, zones, paire), ",
                "drc (dernier DRC lancé ; cao_drc le relance), liaisons (ce qui reste à router, de REF.pastille ",
                "à REF.pastille, avec positions et couches), zone (pastilles, pistes, vias, zones et contour ",
                "dans la fenêtre x1, y1, x2, y2 en mm), carte (contour : rectangle, disque ou polygone et ses ",
                "sommets, découpes), zones (id, couche, net, liaison des pastilles, emprise), serigraphie (id ",
                "des dessins, repères déplacés), interdits (zones interdites par face, hauteurs maximales). Positions ",
                "en mm, le repère de tous les outils CAO. ",
                "Ne modifie rien.", DONNEES],
              schema_lire("pcb", json!({"a_router": {"type": "boolean", "description": "nets : seulement ceux qui restent à router."},
                                        "x1": fenetre, "y1": fenetre, "x2": fenetre, "y2": fenetre})),
              |a| lire("pcb", a, &["filtre", "ref", "nom", "a_router", "x1", "y1", "x2", "y2"])),
        outil("cao_ipc_lire",
              &["Lit la visionneuse IPC-2581 ouverte : empilage, composants, composant (net de chaque broche), ",
                "nets (classe, longueur), net (pistes, plans, pastilles, broches). Ne modifie rien.", DONNEES],
              schema_lire("ipc", json!({})),
              |a| lire("ipc", a, &["filtre", "ref", "nom"])),
        outil("cao_sim_lire",
              &["Lit les simulations de la page PCB ou de la visionneuse : simulations (analyses, celles qui ont un ",
                "résultat, chiffres clés), simulation (une partie du rapport d'une analyse ; commence par chiffres ",
                "et diagnostics), catalogue (paramètres réglables au panneau de simulation : groupe.clé, unité ou ",
                "choix, valeur actuelle, où la lire dans une datasheet). Le rapport complet n'existe que pour ",
                "l'analyse affichée : afficher=true y passe le panneau. cao_calcul lance un calcul.", DONNEES],
              json!({"type": "object", "required": ["quoi"], "properties": {
                  "quoi": {"type": "string", "enum": de(&QUOI, "sim")},
                  "analyse": {"type": "string", "description": "Clé de l'analyse (rendue par quoi = simulations)."},
                  "partie": {"type": "string", "enum": PARTIES},
                  "afficher": {"type": "boolean", "description": "Passer le panneau sur cette analyse avant de lire."},
                  "onglet": onglet}}),
              |a| lire("sim", a, &["analyse", "partie", "afficher"])),
        outil("cao_vue",
              &["Règle la vue d'une page WEB_CAO, sans rien changer au projet : cadrer sur un repère ou un net ",
                "(sélectionné), surligner sans bouger la vue ni la sélection (au schéma, sur la feuille ",
                "affichée ou donnée), feuille (schéma), couches visibles (PCB, ",
                "visionneuse : les autres sont masquées), ajuster à tout le document. capturer=true rend ",
                "l'image dans la foulée. Un repère ou une couche inconnus ne changent rien. La visibilité des ",
                "couches du PCB part avec le prochain Enregistrer de l'utilisateur.", DONNEES],
              json!({"type": "object", "properties": {
                  "page": {"type": "string", "enum": PAGES_VUE, "description": "Page visée ; par défaut, la dernière utilisée."},
                  "cadrer": {"type": "string"}, "surligner": {"type": "string"},
                  "feuille": {"type": "integer", "minimum": 1}, "couches": {"type": "array", "items": {"type": "string"}},
                  "ajuster": {"type": "boolean"}, "capturer": {"type": "boolean"}, "onglet": onglet}}),
              cao_vue),
        outil("cao_thermique",
              &["Estime l'échauffement des composants du PCB ouvert qui dissipent (champ puissance en W de ",
                "l'inspecteur de l'empreinte) : le cuivre relié à leurs pastilles (îlots de zone ",
                "touchés par un cuivre réel, pistes de leurs nets) est mesuré par la page, puis le modèle ",
                "d'étalement de la chute DC (dc_solver) rend ΔT, la résistance thermique et ce qui est supposé. ",
                "Un ordre de grandeur (±30 %), pas une simulation. Le DRC signale ensuite ce qui dépasse ",
                "l'échauffement max de la carte, tant qu'elle n'a pas bougé. N'entre pas dans l'historique.", DONNEES],
              json!({"type": "object", "properties": {"onglet": onglet}}),
              cao_thermique),
        outil("cao_drc",
              &["Lance le DRC de l'éditeur PCB ouvert (comme le bouton DRC) et rend les erreurs puis les ",
                "remarques, avec leur position en mm. N'entre pas dans l'historique.", DONNEES],
              json!({"type": "object", "properties": {"onglet": onglet}}),
              cao_drc),
        outil("cao_regles",
              &["Contrôle les règles de conception vérifiables (savoir_lire conception-electronique) sur l'éditeur ",
                "schématique ou PCB ouvert, et rend chaque règle avec son repère : ✓ respectée, ✗ et ses écarts. ",
                "Schéma : cartouche (S3), préfixes des repères (S6), repères uniques et numérotés (S7), paires ",
                "nommées (S13), référence fabricant (S27), empreinte (S29), ERC (S34). PCB : cartouche (C46), ",
                "empilage symétrique (C7), repères de pose (C30), sérigraphie (C36), anneaux (R16), points de ",
                "test (C39), DRC (R1, relancé). Ne couvre pas le reste des règles. Ne modifie rien.", DONNEES],
              json!({"type": "object", "properties": {"onglet": onglet}}),
              cao_regles),
    ]
}
