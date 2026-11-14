//! Outils MCP du module routage : router la carte ouverte par points de passage, scorer.
//!
//! Spec : lanceur/docs/SPEC-mcp-routage.md. Chaque outil vérifie la forme de ses arguments, puis les
//! dépose à la page PCB par le canal de pilotage (WEB_CAO/editeur-pcb/js/37-routage-pilote.js). La page
//! joue le lot pour de vrai, relève le résultat, remet la carte et montre la carte d'action avec un
//! aperçu ; au clic, elle charge le résultat relevé en une étape d'historique. cao_score et cao_chemin
//! (module chemin-auto, SPEC-mcp-chemin-auto.md : des points de passage proposés par A*, 40-chemin-auto.js)
//! sont des lectures.

use serde_json::{json, Value};

use super::api_outils::resumer;
use super::ecriture_cao::{parmi, valider, Champs, Controle, NOM, NOMBRE};
use super::pilote;
use super::py::Objet;
use super::registre_3d::{choix, donne};
use super::{outil, refuser, ErreurOutil, OutilMcp, Rendu};

const ROUTES_MAX: usize = 20;
const POINTS_MAX: usize = 200;
const VIAS_MAX: usize = 50;
const POINTS_LOT: usize = 1000;
const MODE: Controle = Controle(|v| parmi(v, &["pousser", "contourner"]), "pousser ou contourner");

fn mm(v: Option<&Value>) -> bool {
    v.is_some_and(|v| NOMBRE.0(v) && v.as_f64().is_some_and(|f| (-1000.0..=1000.0).contains(&f)))
}

fn point(v: &Value) -> bool {
    match v {
        Value::String(_) => NOM.0(v),
        Value::Object(o) => o.keys().all(|k| k == "x" || k == "y" || k == "couche") && mm(o.get("x")) && mm(o.get("y"))
            && o.get("couche").is_none_or(|c| c.is_null() || NOM.0(c)),
        _ => false,
    }
}

const POINT: Controle = Controle(point, "« REF.pastille », ou {x, y, couche?} en mm (±1000)");
const POINTS: Controle = Controle(|v| v.as_array().is_some_and(|l| l.len() <= POINTS_MAX && l.iter().all(point)),
                                  "liste de 200 points au plus");
const ROUTE: &Champs = &[("depart", POINT), ("arrivee", POINT), ("points", POINTS), ("largeur", NOMBRE), ("couche", NOM)];
const VIA: &Champs = &[("x", NOMBRE), ("y", NOMBRE), ("net", NOM), ("de", NOM), ("a", NOM)];

fn envoyer(nom: &str, args: Value, a: &Objet, ecriture: bool) -> Rendu {
    Ok(resumer(pilote::envoyer("web_cao", &["pcb"], nom, args, ecriture, pilote::onglet(a)?)?,
               "Réponse trop longue : relis avec cao_pcb_lire ou cao_score."))
}

fn liste(a: &Objet, cle: &str, maxi: usize, champs: &Champs, requis: &[&str]) -> Result<Vec<Value>, ErreurOutil> {
    let Some(v) = donne(a, cle) else {
        return Ok(Vec::new());
    };
    let Some(v) = v.as_array().filter(|v| v.len() <= maxi) else {
        return refuser(format!("Argument « {cle} » invalide : liste de {maxi} éléments au plus."));
    };
    for (i, e) in v.iter().enumerate() {
        valider(e, champs, requis, &format!("{cle}, élément {}", i + 1))?;
    }
    Ok(v.clone())
}

fn cao_router(a: &Objet) -> Rendu {
    let routes = liste(a, "routes", ROUTES_MAX, ROUTE, &["depart", "arrivee"])?;
    let vias = liste(a, "vias", VIAS_MAX, VIA, &["x", "y", "net"])?;
    if routes.is_empty() && vias.is_empty() {
        return refuser("Rien à router : « routes » ou « vias ».");
    }
    if routes.iter().map(|r| r.get("points").and_then(Value::as_array).map_or(0, Vec::len)).sum::<usize>() > POINTS_LOT {
        return refuser(format!("{POINTS_LOT} points de passage au plus par carte : découpe le lot."));
    }
    let mut args = json!({"routes": routes, "vias": vias});
    if let Some(mode) = donne(a, "mode") {
        args["mode"] = json!(choix(Some(mode), "mode", &["pousser", "contourner"])?);
    }
    envoyer("cao_router", args, a, true)
}

fn cao_paire(a: &Objet) -> Rendu {
    let champs: &Champs = &[("depart", POINT), ("arrivee", POINT), ("points", POINTS), ("couche", NOM), ("paire", NOM),
                            ("mode", MODE)];
    let corps = Value::Object(a.iter().filter(|(k, _)| *k != "onglet").map(|(k, v)| (k.clone(), v.clone())).collect());
    valider(&corps, champs, &["paire", "depart", "arrivee"], "paire")?;
    envoyer("cao_paire", corps, a, true)
}

const BOUTS_PAIRE: Controle = Controle(|v| v.as_array().is_some_and(|l| l.len() == 2 && l.iter().all(|x| x.is_string() && NOM.0(x))),
                                       "[« REF.pastille » de P, « REF.pastille » de N]");

fn cao_chemin(a: &Objet) -> Rendu {
    let Some(liaisons) = a.get("liaisons").and_then(Value::as_array).filter(|l| l.len() <= ROUTES_MAX) else {
        return refuser(format!("Argument « liaisons » invalide : liste de {ROUTES_MAX} éléments au plus."));
    };
    for (i, e) in liaisons.iter().enumerate() {
        let ou = format!("liaisons, élément {}", i + 1);
        if e.as_object().is_some_and(|e| e.contains_key("paire")) { // une paire : son axe, de deux pastilles à deux pastilles
            valider(e, &[("paire", NOM), ("depart", BOUTS_PAIRE), ("arrivee", BOUTS_PAIRE)], &["paire", "depart", "arrivee"], &ou)?;
        } else {
            valider(e, &[("depart", POINT), ("arrivee", POINT), ("largeur", NOMBRE)], &["depart", "arrivee"], &ou)?;
        }
    }
    let mut args = json!({"liaisons": liaisons});
    if liaisons.is_empty() {
        return refuser(format!("Argument « liaisons » manquant : de 1 à {ROUTES_MAX} liaisons {{depart, arrivee, largeur?}}."));
    }
    if let Some(pas) = donne(a, "pas") {
        if !(NOMBRE.0(pas) && pas.as_f64().is_some_and(|f| (0.05..=2.0).contains(&f))) {
            return refuser("Argument « pas » invalide : de 0,05 à 2 mm.");
        }
        args["pas"] = pas.clone();
    }
    envoyer("cao_chemin", args, a, false)
}

fn cao_score(a: &Objet) -> Rendu {
    let mut vue = envoyer("cao_score", json!({"quoi": choix(a.get("quoi"), "quoi", &["routage", "placement"])?}), a, false)?;
    // shortcut: la page calcule encore ce total (37-routage-pilote.js, éprouvé par son harness) ; à retirer là-bas
    if let Some(resultat) = vue.get("resultat").and_then(Value::as_str) {
        let sans_total: Vec<&str> = resultat.split('\n').filter(|l| !l.starts_with("- total indicatif")).collect();
        vue["resultat"] = json!(sans_total.join("\n"));
    }
    Ok(vue)
}

const CARTE: &str = concat!(" Le lot est joué pour de vrai par la page puis défait ; la carte montre route par route « posée » ",
                            "ou « refusée : raison », et un aperçu en pointillés (le cuivre déplacé par la poussée en rouge ; ",
                            "pilote_capture le montre). Au clic, le cuivre vu est chargé en une étape d'historique (Ctrl+Z la ",
                            "défait), avec le DRC avant → après ; les réglages de l'éditeur ne changent pas. Le lot entier est ",
                            "borné à 10 s de calcul dans la page.");
const PT_DOC: &str = "« REF.pastille » (U1.3) ou {x, y, couche?} en mm, repère de cao_pcb_lire";

pub fn outils() -> Vec<OutilMcp> {
    let onglet = pilote::onglet_doc();
    let t = json!({"type": "string"});
    let pt = json!({"description": PT_DOC});
    let pt_paire = [PT_DOC, " ; pour une paire, [P, N]"].concat();

    vec![
        outil("cao_router",
              &["Route des liaisons de la carte PCB ouverte par points de passage : le routeur de la page ",
                "(push-and-shove) fait le détail entre les points. routes : [{depart, points?, arrivee, ",
                "largeur?, couche?}] (20 au plus ; un point avec « couche » change de couche par une via) ; ",
                "vias : [{x, y, net, de?, a?}] seules. mode : pousser (défaut) ou contourner. Tout ou rien par ",
                "route : une route bloquée ne laisse rien, les autres restent. Lis d'abord cao_pcb_lire ",
                "liaisons et zone.", CARTE, pilote::ATTENTE_DOC, pilote::DONNEES],
              json!({"type": "object", "properties": {
                  "routes": {"type": "array", "maxItems": ROUTES_MAX, "items": {"type": "object", "required": ["depart", "arrivee"],
                             "properties": {"depart": pt, "arrivee": pt, "points": {"type": "array", "items": pt},
                                            "largeur": {"type": "number"}, "couche": t}}},
                  "vias": {"type": "array", "maxItems": VIAS_MAX, "items": {"type": "object", "required": ["x", "y", "net"]}},
                  "mode": {"type": "string", "enum": ["pousser", "contourner"]}, "onglet": onglet}}),
              cao_router),
        outil("cao_paire",
              &["Route une paire différentielle déclarée (cao_pcb_lire classes les liste) : depart (un bout ",
                "libre de ses deux nets, ou le milieu de ses deux pastilles), points? de passage, arrivee ; ",
                "les deux pistes restent au pas de la règle de paire.", CARTE, pilote::ATTENTE_DOC,
                pilote::DONNEES],
              json!({"type": "object", "required": ["paire", "depart", "arrivee"], "properties": {
                  "paire": t, "depart": pt, "arrivee": pt, "points": {"type": "array", "items": pt}, "couche": t,
                  "mode": {"type": "string", "enum": ["pousser", "contourner"]}, "onglet": onglet}}),
              cao_paire),
        outil("cao_score",
              &["Score de la carte PCB ouverte. routage : liaisons restantes, erreurs et remarques du DRC ",
                "(dont contraintes, groupes de longueur, topologie), vias, longueur. placement : le score du serveur (longueur des liaisons, congestion, ",
                "découplage, pires contributeurs). Ne modifie rien.", pilote::DONNEES],
              json!({"type": "object", "required": ["quoi"], "properties": {
                  "quoi": {"type": "string", "enum": ["routage", "placement"]}, "onglet": onglet}}),
              cao_score),
        outil("cao_chemin",
              &["Propose des points de passage pour des liaisons de la carte PCB ouverte (module ",
                "chemin-auto) : un chercheur de chemin A* sur une grille par couche (pas de 0,25 mm par ",
                "défaut), angles de 45°, vias là où toutes les couches traversées sont libres. Il évite le ",
                "cuivre des autres nets à l'isolation de leurs classes, le bord de carte, les découpes et ",
                "les trous ; les zones de cuivre ne gênent pas (elles se recoulent). Les liaisons se ",
                "cherchent dans l'ordre, chaque chemin trouvé gênant les suivantes. Rend, par liaison, ",
                "trouvée ou la raison, et « routes » à donner tel quel à cao_router (qui fait le détail et ",
                "montre sa carte). Une PAIRE différentielle déclarée (cao_pcb_lire classes) : {paire, ",
                "depart: [pastille de P, pastille de N], arrivee: [idem]} ; son axe est cherché aussi large ",
                "que ses deux pistes et leur écart (règle de paire) ; il change de couche par une via de ",
                "paire (deux vias côte à côte, au point qui précède le point portant « couche »), jamais à ",
                "portée de ses pastilles, et « paires » rend, pour chacune, les arguments de cao_paire (une carte par paire, ",
                "qui fait les éventails jusqu'aux pastilles). Ne modifie rien. 20 liaisons et 8 s au plus ",
                "par appel.", pilote::DONNEES],
              json!({"type": "object", "required": ["liaisons"], "properties": {
                  "liaisons": {"type": "array", "minItems": 1, "maxItems": ROUTES_MAX, "items": {
                      "type": "object", "required": ["depart", "arrivee"], "properties": {
                          "paire": {"type": "string", "description": concat!("nom d'une paire déclarée : depart et arrivee sont alors ",
                                                                             "deux « REF.pastille », celle de P et celle de N")},
                          "depart": {"description": pt_paire},
                          "arrivee": {"description": pt_paire},
                          "largeur": {"type": "number", "description": "mm ; absente : la classe du net (pas pour une paire)"}}}},
                  "pas": {"type": "number", "description": "pas de la grille en mm (0,05 à 2), 0,25 par défaut"},
                  "onglet": onglet}}),
              cao_chemin),
    ]
}
