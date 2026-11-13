//! Le lecteur IPC-2581 (ipc2581_parser.py, version 1.84) : XML -> IPCDesign.
//!
//! Portage fidèle : mêmes balises acceptées, mêmes replis, même ordre de lecture, mêmes
//! messages d'erreur. Le journal (logging) du lecteur Python n'est pas repris.

use crate::data::{
    etendue, py_max, texte, Arc, Component, Contour, ContrePercage, CopperPlane, Drill, Foret, Huray, IPCDesign,
    LayerInfo, PadDef, PadInstance, PadStackDefinition, Package, Pin, Point, ShapeDefinition, StackupLayer,
    TextElement, Track, UserShape,
};
use crate::py::{self, Json, Objet};
use crate::xml::{self, Refus};
use indexmap::IndexMap;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::fmt;
use std::sync::{Arc as Partage, OnceLock};

type N<'a, 'i> = roxmltree::Node<'a, 'i>;
/// Ce qui, en Python, levait une exception hors IPC2581ParseError : son message.
type Py<T> = Result<T, String>;

/// Levée lorsque le fichier IPC-2581 ne peut pas être parsé (fichier absent, XML corrompu, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct IPC2581ParseError(pub String);

impl fmt::Display for IPC2581ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for IPC2581ParseError {}

// layerFunction IPC-2581 qui designent du cuivre (CONDUCTOR, SIGNAL, PLANE,
// MIXED, CONDFILM, CONDFOIL, POWER_GROUND...). Meme famille que celle que la
// page reconnait (js/02-modele.js).
pub(crate) fn fonction_cuivre(fonction: &str) -> bool {
    ["COND", "SIGNAL", "PLANE", "POWER", "GROUND", "MIXED"].iter().any(|m| fonction.contains(m))
}

// Le type d'un composant d'apres ses proprietes TYPE / CATEGORY / DEVICE (mots entiers)
const MOTS_TYPE: [(&str, &[&str]); 7] = [
    ("RESISTOR", &["RES", "RESISTOR", "RESISTORS", "RESISTANCE"]),
    ("CAPACITOR", &["CAP", "CAPS", "CAPACITOR", "CAPACITORS"]),
    ("INDUCTOR", &["IND", "INDUCTOR", "INDUCTORS", "COIL", "COILS", "CHOKE", "CHOKES"]),
    ("DIODE", &["DIODE", "DIODES", "LED", "LEDS"]),
    ("TRANSISTOR", &["TRANSISTOR", "TRANSISTORS", "MOSFET", "MOSFETS"]),
    ("IC", &["IC", "ICS", "CIRCUIT", "CIRCUITS"]),
    ("CONNECTOR", &["CONN", "CONNECTOR", "CONNECTORS", "HEADER", "HEADERS"]),
];

// Les formes standard du XSD rev. C (StandardPrimitive), hors Contour
const PRIMITIVES: [&str; 15] = [
    "Circle", "RectCenter", "Oval", "RectRound", "RectCham", "Ellipse", "Diamond", "Octagon", "Hexagon",
    "Triangle", "RectCorner", "Butterfly", "Donut", "Thermal", "Moire",
];

// Attributs porteurs de la permittivité, du facteur de perte et du nom de
// matériau, tels qu'on les rencontre : le schéma en fixe l'orthographe,
// les outils qui écrivent le fichier s'en écartent parfois.
const SPEC_DK: [&str; 6] = ["dielectricconstant", "permittivity", "relativepermittivity", "dk", "er", "epsilonr"];
const SPEC_DF: [&str; 4] = ["losstangent", "dissipationfactor", "df", "tandelta"];
// Volontairement restreint : « type » et « name » désignent ici la nature
// de la grandeur (DIELECTRIC_CONSTANT, LOSS_TANGENT), pas un matériau.
const SPEC_MAT: [&str; 3] = ["material", "materialref", "materialtype"];

// Les unites de longueur d'une <Spec>, en micrometres. Sans unite, la
// rugosite est lue en MICROMETRES : c'est ainsi que les fiches de cuivre
// et les outils qui l'exportent la donnent.
fn unites_um(unite: &str) -> Option<f64> {
    Some(match unite {
        "MICRON" | "MICRONS" | "UM" | "MICROMETER" => 1.0,
        "MM" | "MILLIMETER" => 1000.0,
        "INCH" | "IN" => 25400.0,
        "MIL" | "MILS" => 25.4,
        _ => return None,
    })
}

// Unites de <Property unit="..."> (propertyUnitType d'IPC-2581) vers le mm :
// la meme table pour les regles, le contre-percage et les dictionnaires.
pub(crate) fn unites_mm(unite: &str) -> Option<f64> {
    Some(match unite {
        "MM" | "MILLIMETER" => 1.0,
        "MICRON" | "UM" => 1e-3,
        "INCH" | "IN" => 25.4,
        "MIL" | "MILS" => 0.0254,
        _ => return None,
    })
}

// Une <Property> de regle, par son nom, vers la cle du gestionnaire de
// contraintes de l'editeur PCB (commun/contraintes.js) : (cle, nature).
// « mm » : une longueur convertie en millimetres ; « n » : un nombre ;
// « txt » : un texte.
fn props_regle(nom: &str) -> Option<(&'static str, &'static str)> {
    Some(match nom {
        "TRACE_WIDTH" => ("w", "mm"),
        "CLEARANCE" => ("clr", "mm"),
        "VIA_DIAMETER" => ("via", "mm"),
        "VIA_DRILL" => ("drill", "mm"),
        "MIN_LENGTH" => ("lMin", "mm"),
        "MAX_LENGTH" => ("lMax", "mm"),
        "MAX_VIAS" => ("viasMax", "n"),
        "MAX_STUB_LENGTH" => ("stubMax", "mm"),
        "MAX_VIA_STUB_LENGTH" => ("viaStubMax", "mm"),
        "STAR_TOLERANCE" => ("etoileTol", "mm"),
        "BACKDRILL_RULE" => ("cp", "txt"),
        "PIN_ORDER" => ("ordre", "txt"),
        "GAP" => ("gap", "mm"),
        "MAX_UNCOUPLED_LENGTH" => ("maxDecouple", "mm"),
        "RULE" => ("regle", "txt"),
        "NET_P" => ("p", "txt"),
        "NET_N" => ("n", "txt"),
        "REFERENCE_NET" => ("ref", "txt"),
        _ => return None,
    })
}

fn topos_regle(t: &str) -> Option<&'static str> {
    Some(match t {
        "POINT_TO_POINT" => "p2p",
        "DAISY_CHAIN" => "chaine",
        "STAR" => "etoile",
        "FLY_BY" => "flyby",
        _ => return None,
    })
}

/// Ce qu'une <Spec> de matière dit : permittivité, pertes, matériau, rugosité.
#[derive(Debug, Clone, Default, PartialEq)]
struct SpecVals {
    dk: Option<String>,
    df: Option<String>,
    material: Option<String>,
    rugosite_um: Option<f64>,
    huray: Option<Huray>,
}

/// Ce qu'une <Spec> de contre-perçage dit.
#[derive(Debug, Clone, Default)]
struct SpecCp {
    depart: Option<String>,
    garde: Option<String>,
    moignon_mm: Option<f64>,
}

/// Le <Location> et le <Xform> d'un <Features> : miroir puis rotation, puis la translation.
#[derive(Clone, Copy)]
struct Xf {
    ox: f64,
    oy: f64,
    co: f64,
    si: f64,
    k: f64,
}

impl Xf {
    fn de(&self, p: Point) -> Point {
        Point::new(self.ox + self.k * p.x * self.co - p.y * self.si, self.oy + self.k * p.x * self.si + p.y * self.co)
    }
}

fn attr<'a>(n: N<'a, '_>, cle: &str) -> Option<&'a str> {
    n.attribute(cle)
}

/// attrib.get(cle, defaut)
fn attr_ou<'a>(n: N<'a, '_>, cle: &str, defaut: &'a str) -> &'a str {
    n.attribute(cle).unwrap_or(defaut)
}

/// attrib.get(cle) or "" : absent et vide se confondent
fn attr_plein<'a>(n: N<'a, '_>, cle: &str) -> Option<&'a str> {
    n.attribute(cle).filter(|v| !v.is_empty())
}

/// Nom de balise sans espace de nom.
fn nom_local<'a>(n: N<'a, '_>) -> &'a str {
    n.tag_name().name()
}

fn enfants<'a, 'i>(n: N<'a, 'i>) -> impl Iterator<Item = N<'a, 'i>> {
    n.children().filter(|c| c.is_element())
}

/// elem.iter() : l'élément lui-même et toute sa descendance, dans l'ordre du document.
fn tous<'a, 'i>(n: N<'a, 'i>) -> impl Iterator<Item = N<'a, 'i>> {
    n.descendants().filter(|c| c.is_element())
}

fn vrai(n: N, cle: &str) -> bool {
    attr_ou(n, cle, "false").eq_ignore_ascii_case("true")
}

fn radians(deg: f64) -> f64 {
    deg * (PI / 180.0)
}

/// Polygone regulier de n sommets sur le cercle de ce rayon.
fn regulier(n: usize, rayon: f64, phase: f64) -> Vec<Point> {
    (0..n)
        .map(|i| {
            let t = phase + 2.0 * PI * i as f64 / n as f64;
            Point::new(rayon * t.cos(), rayon * t.sin())
        })
        .collect()
}

/// Contour d'un Donut ou d'un Thermal ; r est le rayon INSCRIT
/// (demi-cote du carre, demi-distance entre plats).
fn anneau(forme: &str, r: f64) -> Vec<Point> {
    match forme {
        "SQUARE" => vec![Point::new(-r, -r), Point::new(r, -r), Point::new(r, r), Point::new(-r, r)],
        "HEXAGON" => regulier(6, r / (PI / 6.0).cos(), 0.0),
        "OCTAGON" => regulier(8, r / (PI / 8.0).cos(), PI / 8.0),
        _ => regulier(72, r, 0.0),
    }
}

/// int(x) d'un flottant : lève sur un non-fini, comme Python.
fn entier(x: f64) -> Py<f64> {
    if x.is_nan() {
        Err("cannot convert float NaN to integer".into())
    } else if x.is_infinite() {
        Err("cannot convert float infinity to integer".into())
    } else {
        Ok(x.trunc())
    }
}

fn regex(cellule: &'static OnceLock<Regex>, motif: &str) -> &'static Regex {
    cellule.get_or_init(|| Regex::new(motif).expect("motif fixe"))
}

/// La première propriété dont le nom, en majuscules, est l'une des clés, dans l'ordre des clés.
fn premiere_prop(props: &IndexMap<String, String>, cles: &[&str]) -> String {
    for cle in cles {
        for (k, v) in props {
            if k.to_uppercase() == *cle {
                if !v.is_empty() {
                    return v.clone();
                }
                break;
            }
        }
    }
    String::new()
}

pub struct IPC2581Parser {
    /// Le nom du fichier, pour les messages d'erreur.
    pub xml_file: String,
    // False : seuls cuivre, percages et contour sont lus (voir role_calque) ;
    // True (defaut) : tout est lu, la visionneuse veut aussi la serigraphie
    pub tout_garder: bool,
    pub design: IPCDesign,
    ns: Option<String>,
    line_width_dict: HashMap<String, f64>,
    line_end_dict: HashMap<String, String>, // id de LineDesc -> lineEnd
    fill_dict: HashMap<String, String>,     // id de FillDesc -> fillProperty
    formes_en_ligne: HashMap<(String, Vec<(String, String)>), String>, // (balise, attributs) -> id de forme
    // vrai pour un export « Editeur PCB » revision 1 : traits du
    // <UserSpecial> en absolu malgre le <Location> du <Features>
    userspecial_absolu: bool,
    specs: HashMap<String, SpecVals>,
    inline_ps_counter: u64,
    inline_pads: HashMap<(String, String, String), String>, // (forme, couche, usage) -> padstack inline
    via_def_names: HashSet<String>,
    // LA PORTÉE DES CALQUES DE PERÇAGE : nom du calque -> (depuis, vers).
    // C'est le seul endroit où IPC-2581 la déclare, et c'est ce qui
    // distingue un via traversant d'un borgne ou d'un enterré. Voir
    // `lire_span`.
    drill_spans: HashMap<String, (String, String)>,
    // LE CONTRE-PERCAGE (voir `parse_contre_percages`) : les <Spec> de
    // back-drill par nom, les <SpecRef> de chaque calque, et les forets
    // lus sur un calque a part, en attente du via qu'ils repassent.
    backdrill_specs: HashMap<String, SpecCp>,
    calque_specs: HashMap<String, Vec<String>>,
    forets_cp: Vec<Foret>,
    // LES REGLES (voir `lire_spec_regle`) : chaque <Spec> de regle lue
    // par son nom -- un <LogicalNet> la pointe par <SpecRef> --, et ce qui
    // en sort, pose sur le design (regles_nets).
    spec_regles: HashMap<String, (&'static str, Objet)>,
    // Les composants nés d'un même <BomItem> partageaient, en Python, UN dict de
    // propriétés : ce qui s'ajoute à l'un s'ajoute aux autres. Rang du <BomItem>, par composant.
    groupes_bom: Vec<Option<usize>>,
}

impl IPC2581Parser {
    pub fn new(xml_file: &str, tout_garder: bool) -> IPC2581Parser {
        IPC2581Parser {
            xml_file: xml_file.to_string(),
            tout_garder,
            design: IPCDesign::default(),
            ns: None,
            line_width_dict: HashMap::new(),
            line_end_dict: HashMap::new(),
            fill_dict: HashMap::new(),
            formes_en_ligne: HashMap::new(),
            userspecial_absolu: false,
            specs: HashMap::new(),
            inline_ps_counter: 0,
            inline_pads: HashMap::new(),
            via_def_names: HashSet::new(),
            drill_spans: HashMap::new(),
            backdrill_specs: HashMap::new(),
            calque_specs: HashMap::new(),
            forets_cp: Vec::new(),
            spec_regles: HashMap::new(),
            groupes_bom: Vec::new(),
        }
    }

    /// Lit le fichier `xml_file` sur le disque.
    pub fn parse_fichier(self) -> Result<IPCDesign, IPC2581ParseError> {
        match std::fs::read(&self.xml_file) {
            Ok(octets) => self.parse(&octets),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(IPC2581ParseError(format!("Fichier introuvable : {}", self.xml_file)))
            }
            // shortcut: le texte de l'OSError de Python (« [Errno 13] Permission denied: '...' »)
            // n'est pas reproduit ; à reprendre si un appelant le montre
            Err(e) => Err(IPC2581ParseError(format!("Erreur lors du parsing de {} : {}", self.xml_file, e))),
        }
    }

    /// Lit les octets d'un fichier IPC-2581 ; `xml_file` ne sert qu'aux messages.
    pub fn parse(mut self, octets: &[u8]) -> Result<IPCDesign, IPC2581ParseError> {
        // Rejette les entites DTD XML (<!ENTITY) pour prevenir les attaques Billion Laughs / DoS.
        // tout le fichier : une entite declaree apres 64 Ko passait
        if octets.windows(8).any(|f| f.eq_ignore_ascii_case(b"<!ENTITY")) {
            return Err(IPC2581ParseError("Declaration d'entite XML interdite (<!ENTITY).".into()));
        }
        let invalide = |e: String| IPC2581ParseError(format!("XML invalide ou corrompu : {} ({})", self.xml_file, e));
        let xml = match xml::decoder(octets) {
            Ok(texte) => texte,
            Err(Refus::Xml(e)) => return Err(invalide(e)),
            Err(Refus::Autre(e)) => {
                return Err(IPC2581ParseError(format!("Erreur lors du parsing de {} : {}", self.xml_file, e)));
            }
        };
        xml::verifier(&xml).map_err(invalide)?;
        let xml = &xml.xml[xml.debut..];
        let options = roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() };
        let doc = roxmltree::Document::parse_with_options(xml, options)
            .map_err(|e| invalide(xml::erreur_roxmltree(xml, &e)))?;
        let racine = doc.root_element();

        // un XML quelconque s'ouvrait comme une carte vide, sans un mot
        if nom_local(racine) != "IPC-2581" {
            return Err(IPC2581ParseError(format!(
                "Ce n'est pas un fichier IPC-2581 : la racine est <{}>, il faut <IPC-2581>.",
                nom_local(racine)
            )));
        }
        self.ns = racine.tag_name().namespace().map(str::to_string);
        match self.lire(racine) {
            Ok(()) => Ok(self.design),
            Err(e) => Err(IPC2581ParseError(format!("Erreur lors du parsing de {} : {}", self.xml_file, e))),
        }
    }

    fn lire(&mut self, racine: N) -> Py<()> {
        self.parse_units(racine);

        for sp in self.iter(racine, "SoftwarePackage") {
            if (attr(sp, "name"), attr(sp, "vendor"), attr(sp, "revision"))
                == (Some("Editeur PCB"), Some("WEB_CAO"), Some("1"))
            {
                self.userspecial_absolu = true;
            }
        }

        self.parse_fill_dictionary(racine);
        self.parse_line_dictionary(racine);
        self.parse_standard_dictionary(racine)?;
        self.parse_user_dictionary(racine)?;
        self.parse_padstack_defs(racine);
        self.parse_specs(racine);
        self.parse_contre_percages(racine);
        self.parse_ecad(racine)?;
        self.rattacher_contre_percages()?;
        self.parse_bom(racine);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Helpers génériques
    // ------------------------------------------------------------------

    /// Convertit en float sans lever d'exception (attribut XML manquant/invalide).
    /// Un non-fini (« inf », « 1e400 », « nan ») est invalide : il ferait lever
    /// l'arrondi de ipc2581_json et tomber la lecture du fichier entier.
    fn safe_float_ou(value: Option<&str>, default: f64) -> f64 {
        match value.and_then(py::float) {
            Some(v) if v.is_finite() => v,
            _ => default,
        }
    }

    fn safe_float(value: Option<&str>) -> f64 {
        Self::safe_float_ou(value, 0.0)
    }

    /// float() qui ÉCHOUE sur un non-fini, pour une coordonnee obligatoire :
    /// l'appelant ignore alors la forme, au lieu de laisser un « inf » faire
    /// tomber l'arrondi de ipc2581_json et la lecture du fichier entier.
    fn fini(value: Option<&str>) -> Option<f64> {
        value.and_then(py::float).filter(|v| v.is_finite())
    }

    /// Convertit en int sans lever d'exception — sauf sur un infini, que int() refuse par
    /// une OverflowError que le lecteur Python ne rattrapait pas. L'entier est rendu dans un
    /// flottant : int(float(...)) de Python n'a pas de borne, i64 en aurait une.
    fn safe_int(value: Option<&str>, default: f64) -> Py<f64> {
        match value.and_then(py::float) {
            None => Ok(default),
            Some(v) if v.is_nan() => Ok(default),
            Some(v) => Ok(entier(v)? + 0.0), // + 0.0 : int(-0.5) vaut 0, pas -0
        }
    }

    /// La balise est-elle celle-ci, dans l'espace de nom de la racine ?
    fn est(&self, n: N, nom: &str) -> bool {
        n.is_element() && n.tag_name().name() == nom && n.tag_name().namespace() == self.ns.as_deref()
    }

    fn find<'a, 'i>(&self, n: N<'a, 'i>, nom: &str) -> Option<N<'a, 'i>> {
        enfants(n).find(|c| self.est(*c, nom))
    }

    fn findall<'a, 'i>(&self, n: N<'a, 'i>, nom: &str) -> Vec<N<'a, 'i>> {
        enfants(n).filter(|c| self.est(*c, nom)).collect()
    }

    /// elem.iter(balise) : l'élément et sa descendance qui portent cette balise.
    fn iter<'a, 'i>(&self, n: N<'a, 'i>, nom: &str) -> Vec<N<'a, 'i>> {
        tous(n).filter(|c| self.est(*c, nom)).collect()
    }

    /// Les id des <SpecRef> d'un élément.
    fn spec_refs(&self, n: N) -> Vec<String> {
        self.findall(n, "SpecRef").iter().map(|r| attr_ou(*r, "id", "").to_string()).collect()
    }

    /// Diamètre équivalent (plus grande dimension) d'une forme référencée.
    fn shape_max_dimension(&self, shape_ref: &str) -> f64 {
        if let Some(shape) = self.design.shapes.get(shape_ref) {
            return shape.max_dimension();
        }
        // forme utilisateur (Donut, Thermal, UserPrimitiveRef) : son encombrement
        let mut pts: Vec<Point> = Vec::new();
        if let Some(user) = self.design.user_shapes.get(shape_ref) {
            pts.extend(user.planes.iter().flat_map(|pl| pl.contours.iter()).flat_map(|c| c.outline.iter().copied()));
            pts.extend(user.tracks.iter().flat_map(|t| t.points.iter().copied()));
        }
        if pts.is_empty() {
            return 0.0;
        }
        etendue(&pts)
    }

    /// Extrait la valeur physique et la tolérance d'une chaîne (ex: 100nF, 10k, 4R7, 10uH, RES-10K).
    pub fn extraire_val_tol(texte: &str) -> (String, String) {
        static TOL: OnceLock<Regex> = OnceLock::new();
        static M1: OnceLock<Regex> = OnceLock::new();
        static M2: OnceLock<Regex> = OnceLock::new();
        if texte.is_empty() {
            return (String::new(), String::new());
        }
        let t = py::strip(texte);
        let tol = regex(&TOL, r"\+-[\d\.]+[a-zA-Z]*|\d+(?:\.\d+)?%").find(t).map_or("", |m| m.as_str());

        // 1. Unité explicite F, H, R, O, Ohm, Hz ou notation 4R7
        // (\x1c-\x1f : des blancs pour Python)
        let m1 = regex(
            &M1,
            r"(?i)(?:^|[\s\x1c-\x1f_\-–])(?P<val>\d+(?:\.\d+)?[pnumkKMG]?(?:[FROH]|Ohm|hz|Hz)+|\d+[R]\d+|\d+(?:\.\d+)?(?:MHz|KHz|Hz))(?:[\s\x1c-\x1f_\-+%/\)]|$)",
        );
        let val = match m1.captures(t) {
            Some(c) => c["val"].to_string(),
            // 2. Multiplicateur seul sans unité explicite (ex: 10k, 4.7k, 1M, 100R, 2K2)
            None => regex(
                &M2,
                r"(?i)(?:^|[\s\x1c-\x1f_\-–])(?P<val>\d+(?:\.\d+)?[KMR]|\d+[KM]\d+)(?:[\s\x1c-\x1f_\-+%/\)]|$)",
            )
            .captures(t)
            .map_or(String::new(), |c| c["val"].to_string()),
        };
        (val, tol.to_string())
    }

    /// Détermine le type de composant : RESISTOR, CAPACITOR, INDUCTOR, DIODE, etc.
    pub fn qualifier_type_composant(ref_des: &str, val: &str, props: Option<&IndexMap<String, String>>) -> String {
        let rd = py::strip(ref_des).to_uppercase();
        // 1. Propriétés explicites
        if let Some(props) = props {
            for k in ["TYPE", "COMPONENT_TYPE", "CATEGORY", "DEVICE"] {
                for (prop_k, prop_v) in props {
                    if prop_k.to_uppercase() == k {
                        // des mots entiers : « Resonators » n'est pas RES, « CERAMIC » pas IC,
                        // « Transformer » pas TRANS(istor)
                        let majuscules = prop_v.to_uppercase();
                        let mots: HashSet<&str> =
                            majuscules.split(|c: char| !c.is_ascii_uppercase()).filter(|m| !m.is_empty()).collect();
                        if mots.contains("TRANSFORMER") || mots.contains("TRANSFORMERS") {
                            return "UNKNOWN".into(); // sinon le préfixe T le ferait transistor
                        }
                        for (genre, noms) in MOTS_TYPE {
                            if noms.iter().any(|n| mots.contains(n)) {
                                return genre.into();
                            }
                        }
                    }
                }
            }
        }

        // 2. Préfixe du RefDes (norme IEEE 315 / ASME Y14.44 / usage CAO)
        let fin = rd.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(rd.len());
        let genre = match &rd[..fin] {
            "R" | "RN" | "RT" | "RV" | "POT" | "RP" | "VR" => "RESISTOR",
            "C" | "CN" | "CP" | "TC" => "CAPACITOR",
            "L" | "FB" | "IND" | "BEAD" | "CH" => "INDUCTOR",
            "D" | "CR" | "LED" | "TVS" | "ZD" => "DIODE",
            "Q" | "TR" | "MOS" | "FET" | "T" => "TRANSISTOR", // T : transistor dans l'usage français
            "U" | "IC" => "IC",
            "J" | "P" | "CON" | "CONN" | "HDR" => "CONNECTOR",
            "Y" | "X" | "XTAL" | "OSC" => "CRYSTAL",
            "TP" | "TEST" => "TESTPOINT",
            "F" | "FUSE" => "FUSE",
            "SW" | "PB" => "SWITCH",
            _ => "",
        };
        if !genre.is_empty() {
            return genre.into();
        }

        // 3. Unité dans la valeur si le préfixe est inhabituel
        static RESISTANCE: OnceLock<Regex> = OnceLock::new();
        let v = val.to_uppercase();
        let un_de = |mots: &[&str]| mots.iter().any(|u| v.contains(u));
        if (un_de(&["PF", "NF", "UF", "MF", " F"]) || v.ends_with('F')) && !un_de(&["HZ", "HENRY"]) {
            return "CAPACITOR".into();
        }
        if un_de(&["NH", "UH", "MH", " H"]) || v.ends_with('H') {
            return "INDUCTOR".into();
        }
        // « $ » de Python tolère un saut de ligne final
        if un_de(&["OHM", "KOHM", "MOHM", "Ω"]) || regex(&RESISTANCE, r"\d+R\d+|\d+[KMR]\n?\z").is_match(&v) {
            return "RESISTOR".into();
        }
        "UNKNOWN".into()
    }

    /// Lit les balises <Property>, <NonstandardAttribute> et <Characteristics> d'un élément XML.
    fn lire_proprietes(&self, elem: N) -> IndexMap<String, String> {
        let mut props = IndexMap::new();
        for p in self.findall(elem, "Property") {
            let nom = attr_plein(p, "name").or(attr_plein(p, "property")).unwrap_or("");
            let val = attr_plein(p, "value").or(attr_plein(p, "val")).unwrap_or("");
            if !nom.is_empty() && !val.is_empty() {
                props.insert(nom.to_string(), val.to_string());
            }
        }
        for p in self.findall(elem, "NonstandardAttribute") {
            let nom = attr_ou(p, "name", "");
            let val = attr_ou(p, "value", "");
            if !nom.is_empty() && !val.is_empty() {
                props.insert(nom.to_string(), val.to_string());
            }
        }
        if let Some(chars) = self.find(elem, "Characteristics") {
            for child in enfants(chars) {
                let tag = nom_local(child).to_lowercase().to_uppercase();
                let c_val = attr_ou(child, "value", "");
                let c_tol = attr_ou(child, "tolerance", "");
                if !c_val.is_empty() {
                    props.insert(tag.clone(), c_val.to_string());
                }
                if !c_tol.is_empty() {
                    props.insert(format!("{tag}_TOLERANCE"), c_tol.to_string());
                }
            }
        }
        props
    }

    // ------------------------------------------------------------------
    // Étape 1 : Unités
    // ------------------------------------------------------------------

    fn parse_units(&mut self, racine: N) {
        if let Some(content) = self.find(racine, "Content") {
            if let Some(u) = self.find(content, "DictionaryLineDesc").and_then(|d| attr(d, "units")) {
                self.design.units = u.to_string();
            }
        }
        // hors du « if content » : un fichier en pouces sans <Content> était lu en mm
        if let Some(ecad) = self.find(racine, "Ecad") {
            if let Some(u) = self.find(ecad, "CadHeader").and_then(|h| attr(h, "units")) {
                self.design.units = u.to_string();
            }
        }
    }

    /// Le facteur des unités du fichier vers le millimètre, 1 si elles sont inconnues.
    fn mm_du_fichier(&self) -> f64 {
        unites_mm(&self.design.units.to_uppercase()).unwrap_or(1.0)
    }

    // ------------------------------------------------------------------
    // Étape 2 : Dictionnaire de lignes
    // ------------------------------------------------------------------

    fn parse_line_dictionary(&mut self, racine: N) {
        let Some(dict_line) = self.find(racine, "Content").and_then(|c| self.find(c, "DictionaryLineDesc")) else {
            return;
        };
        let k = self.k_dico(dict_line);
        for entry in self.findall(dict_line, "EntryLineDesc") {
            // <LineDesc lineWidth="0.2"/> n'a pas d'enfant : il compte quand même
            // (le lecteur Python l'a un temps pris pour faux, et toutes les
            // pistes du fichier se retrouvaient à une largeur nulle).
            if let (Some(entry_id), Some(line_desc)) = (attr_plein(entry, "id"), self.find(entry, "LineDesc")) {
                self.line_width_dict.insert(entry_id.to_string(), Self::safe_float(attr(line_desc, "lineWidth")) * k);
                self.line_end_dict
                    .insert(entry_id.to_string(), attr_plein(line_desc, "lineEnd").unwrap_or("ROUND").to_uppercase());
            }
        }
    }

    /// <DictionaryFillDesc> : id -> fillProperty, pour les <FillDescRef>.
    fn parse_fill_dictionary(&mut self, racine: N) {
        let Some(dico) = self.find(racine, "Content").and_then(|c| self.find(c, "DictionaryFillDesc")) else {
            return;
        };
        for entry in self.findall(dico, "EntryFillDesc") {
            if let (Some(id), Some(fill)) = (attr_plein(entry, "id"), self.find(entry, "FillDesc")) {
                self.fill_dict.insert(id.to_string(), attr_ou(fill, "fillProperty", "HOLLOW").to_uppercase());
            }
        }
    }

    /// Facteur des unites propres d'un dictionnaire vers celles du fichier
    /// (CadHeader) ; 1 si l'une des deux manque ou est inconnue.
    fn k_dico(&self, dico: N) -> f64 {
        let a = unites_mm(&attr_ou(dico, "units", "").to_uppercase());
        let b = unites_mm(&self.design.units.to_uppercase());
        match (a, b) {
            (Some(a), Some(b)) => a / b,
            _ => 1.0,
        }
    }

    // ------------------------------------------------------------------
    // Étape 3 : Dictionnaire standard
    // ------------------------------------------------------------------

    fn parse_standard_dictionary(&mut self, racine: N) -> Py<()> {
        let Some(std_dict) = self.find(racine, "Content").and_then(|c| self.find(c, "DictionaryStandard")) else {
            return Ok(());
        };
        let k = self.k_dico(std_dict);
        for entry in self.findall(std_dict, "EntryStandard") {
            if let Some(entry_id) = attr_plein(entry, "id") {
                if !self.lire_forme(entry_id, entry, k)? {
                    self.design.shapes.insert(entry_id.to_string(), ShapeDefinition::new(entry_id, "UNKNOWN"));
                }
            }
        }
        Ok(())
    }

    /// Un <Thermal> : l'anneau outer/innerDiameter coupe par spokeCount
    /// fentes droites de largeur spokeWidth (gap aux revisions anterieures),
    /// la premiere a spokeStartAngle. Rend [(contour, trous)] : un secteur
    /// par contour.
    fn thermique(noeud: N, k: f64) -> Py<Vec<(Vec<Point>, Vec<Vec<Point>>)>> {
        let f = |cle: &str| Self::safe_float(attr(noeud, cle)) * k;
        let forme = attr_plein(noeud, "shape").unwrap_or("ROUND").to_uppercase();
        let (ro, ri) = (f("outerDiameter") / 2.0, f("innerDiameter") / 2.0);
        let n = Self::safe_int(attr(noeud, "spokeCount"), 4.0)?.min(360.0) as i64; // la boucle suit n
        let largeur = f("spokeWidth");
        let demi = (if largeur != 0.0 { largeur } else { f("gap") }) / 2.0;
        if n <= 0 || demi <= 0.0 {
            return Ok(vec![(anneau(&forme, ro), vec![anneau(&forme, ri)])]);
        }
        // shortcut: un Thermal SQUARE, HEXAGON ou OCTAGON est dessine rond ; a
        // reprendre si un fichier reel en porte
        if demi >= ro {
            return Ok(Vec::new());
        }
        let a0 = radians(Self::safe_float(attr(noeud, "spokeStartAngle")));
        let pas = 2.0 * PI / n as f64;

        let arc = |r: f64, t1: f64, t2: f64| -> Vec<Point> {
            let m = ((((t2 - t1).abs() / (2.0 * PI) * 72.0) as i64).max(2)) as usize;
            (0..=m)
                .map(|i| {
                    let t = t1 + (t2 - t1) * i as f64 / m as f64;
                    Point::new(r * t.cos(), r * t.sin())
                })
                .collect()
        };

        let ao = (demi / ro).asin();
        // les bords de deux fentes voisines se coupent a cette distance du centre
        let coin = demi / (pas / 2.0).min(PI / 2.0).sin();
        let mut secteurs = Vec::new();
        for i in 0..n {
            let (t1, t2) = (a0 + i as f64 * pas, a0 + (i + 1) as f64 * pas);
            if t2 - ao <= t1 + ao || coin >= ro {
                continue;
            }
            let dedans = if ri > coin {
                let ai = (demi / ri).asin();
                arc(ri, t2 - ai, t1 + ai)
            } else {
                vec![Point::new(coin * ((t1 + t2) / 2.0).cos(), coin * ((t1 + t2) / 2.0).sin())]
            };
            let mut contour = arc(ro, t1 + ao, t2 - ao);
            contour.extend(dedans);
            secteurs.push((contour, Vec::new()));
        }
        Ok(secteurs)
    }

    /// Range sous `entry_id` la forme standard que porte `parent` (son
    /// premier enfant reconnu) : une ShapeDefinition, ou une forme
    /// utilisateur quand il lui faut plusieurs contours ou des trous (Donut,
    /// Thermal, Moire, Contour a decoupes) -- la visionneuse et la simulation
    /// les lisent deja. Dimensions multipliees par k. Faux si rien n'est
    /// reconnu.
    fn lire_forme(&mut self, entry_id: &str, parent: N, k: f64) -> Py<bool> {
        let Some(noeud) = enfants(parent).find(|e| {
            let nom = nom_local(*e);
            PRIMITIVES.contains(&nom) || nom == "Contour" || nom == "Polygon"
        }) else {
            return Ok(false);
        };
        let nom = nom_local(noeud);
        let f = |cle: &str| Self::safe_float(attr(noeud, cle)) * k;
        let mul = |pts: Vec<Point>| -> Vec<Point> { pts.iter().map(|p| Point::new(p.x * k, p.y * k)).collect() };
        let mut sd = ShapeDefinition::new(entry_id, "POLYGON");
        sd.fill_property = self.get_fill_desc(noeud);
        let mut plans: Option<Vec<(Vec<Point>, Vec<Vec<Point>>)>> = None; // [(contour, [trous])] : forme utilisateur
        match nom {
            "Circle" => {
                sd.shape_type = "CIRCLE".into();
                sd.diameter = f("diameter");
            }
            "RectCenter" | "Oval" | "RectRound" | "RectCham" => {
                sd.shape_type = nom.to_uppercase();
                sd.width = f("width");
                sd.height = f("height");
                if nom == "RectRound" {
                    sd.radius = f("radius");
                }
                if nom == "RectCham" {
                    sd.chamfer = f("chamfer");
                }
                if nom == "RectRound" || nom == "RectCham" {
                    sd.corners = ["upperRight", "upperLeft", "lowerRight", "lowerLeft"]
                        .iter()
                        .filter(|c| vrai(noeud, c))
                        .map(|c| c.to_string())
                        .collect();
                }
            }
            "Ellipse" => {
                sd.points = regulier(72, 1.0, 0.0)
                    .iter()
                    .map(|p| Point::new(p.x * f("width") / 2.0, p.y * f("height") / 2.0))
                    .collect();
            }
            "Diamond" => {
                let (w, h) = (f("width") / 2.0, f("height") / 2.0);
                sd.points = vec![Point::new(0.0, -h), Point::new(w, 0.0), Point::new(0.0, h), Point::new(-w, 0.0)];
            }
            // length : d'un sommet a l'oppose, un sommet vers +Y (IPC-2581C 3.5.9.7)
            "Hexagon" => sd.points = regulier(6, f("length") / 2.0, PI / 2.0),
            // shortcut: length lu d'un sommet a l'oppose, comme l'Hexagon (lecture
            // de diodeinc/pcb ; boardui le lit entre plats, 8 % plus grand) ; a
            // trancher sur un fichier reel
            "Octagon" => sd.points = regulier(8, f("length") / 2.0, PI / 8.0),
            "Triangle" => {
                let (b, h) = (f("base") / 2.0, f("height") / 2.0);
                sd.points = vec![Point::new(-b, -h), Point::new(b, -h), Point::new(0.0, h)];
            }
            "RectCorner" => {
                let (x1, y1, x2, y2) = (f("lowerLeftX"), f("lowerLeftY"), f("upperRightX"), f("upperRightY"));
                sd.points = vec![Point::new(x1, y1), Point::new(x2, y1), Point::new(x2, y2), Point::new(x1, y2)];
            }
            "Butterfly" => {
                // deux quarts opposes (I et III), d'un seul trace par le centre
                let rond = attr_ou(noeud, "shape", "").to_uppercase() == "ROUND";
                let r = (if rond { f("diameter") } else { f("side") }) / 2.0;
                let o = Point::new(0.0, 0.0);
                if rond {
                    let q: Vec<Point> = (0..19)
                        .map(|t| {
                            let a = t as f64 * PI / 36.0;
                            Point::new(r * a.cos(), r * a.sin())
                        })
                        .collect();
                    sd.points = vec![o];
                    sd.points.extend(q.iter().copied());
                    sd.points.push(o);
                    sd.points.extend(q.iter().map(|p| Point::new(-p.x, -p.y)));
                } else {
                    sd.points = vec![
                        o,
                        Point::new(r, 0.0),
                        Point::new(r, r),
                        Point::new(0.0, r),
                        o,
                        Point::new(-r, 0.0),
                        Point::new(-r, -r),
                        Point::new(0.0, -r),
                    ];
                }
            }
            "Donut" => {
                let forme = attr_plein(noeud, "shape").unwrap_or("ROUND").to_uppercase();
                plans = Some(vec![(
                    anneau(&forme, f("outerDiameter") / 2.0),
                    vec![anneau(&forme, f("innerDiameter") / 2.0)],
                )]);
            }
            "Thermal" => plans = Some(Self::thermique(noeud, k)?),
            "Moire" => {
                // shortcut: anneaux au pas ringWidth + ringGap depuis le bord
                // exterieur (lecture litterale du XSD, sans exemple pour trancher)
                let mut p = Vec::new();
                let rond = |r: f64| regulier(72, r, 0.0);
                let (lw, pas) = (f("ringWidth"), f("ringWidth") + f("ringGap"));
                let r = f("diameter") / 2.0;
                for i in 0..Self::safe_int(attr(noeud, "ringNumber"), 0.0)?.min(100.0) as i64 {
                    let ro = r - i as f64 * pas;
                    if ro <= 0.0 || lw <= 0.0 {
                        break;
                    }
                    p.push((rond(ro), if ro > lw { vec![rond(ro - lw)] } else { Vec::new() }));
                }
                let epais = f("lineWidth");
                if epais > 0.0 {
                    let longueur = f("lineLength");
                    let lg = if longueur != 0.0 { longueur } else { 2.0 * r };
                    for deg in [0.0, 90.0] {
                        let t = radians(Self::safe_float(attr(noeud, "lineAngle")) + deg);
                        let (c, s) = (t.cos(), t.sin());
                        let coins = [
                            (-lg / 2.0, -epais / 2.0),
                            (lg / 2.0, -epais / 2.0),
                            (lg / 2.0, epais / 2.0),
                            (-lg / 2.0, epais / 2.0),
                        ];
                        p.push((coins.iter().map(|(x, y)| Point::new(x * c - y * s, x * s + y * c)).collect(), Vec::new()));
                    }
                }
                plans = Some(p);
            }
            _ => {
                // Contour, ou Polygon nu
                let Some(poly) = (if nom == "Polygon" { Some(noeud) } else { self.find(noeud, "Polygon") }) else {
                    return Ok(false);
                };
                sd.points = mul(self.parse_polygon(poly)?);
                sd.fill_property = self.get_fill_desc(poly);
                let mut trous = Vec::new();
                for c in self.findall(noeud, "Cutout") {
                    trous.push(mul(self.parse_polygon(c)?));
                }
                if nom == "Contour" && trous.iter().any(|t| t.len() >= 3) {
                    plans = Some(vec![(sd.points.clone(), trous)]);
                }
            }
        }
        let Some(plans) = plans else {
            self.design.shapes.insert(entry_id.to_string(), sd);
            return Ok(true);
        };
        let mut us = UserShape { id: entry_id.to_string(), ..UserShape::default() };
        for (contour, trous) in plans {
            if contour.len() >= 3 {
                us.planes.push(CopperPlane {
                    layer_name: String::new(),
                    net_name: String::new(),
                    fill_property: "FILL".into(),
                    contours: vec![Contour {
                        outline: contour,
                        cutouts: trous.into_iter().filter(|t| t.len() >= 3).collect(),
                    }],
                });
            }
        }
        self.design.user_shapes.insert(entry_id.to_string(), us);
        Ok(true)
    }

    /// Id de la forme standard ecrite EN LIGNE dans `elem` (pas par
    /// reference), rangee une fois par jeu d'attributs ; "" s'il n'y en a pas.
    fn forme_en_ligne(&mut self, elem: N) -> Py<String> {
        let Some(enfant) = enfants(elem).find(|e| PRIMITIVES.contains(&nom_local(*e))) else {
            return Ok(String::new());
        };
        let mut attributs: Vec<(String, String)> = enfant
            .attributes()
            .map(|a| {
                let cle = match a.namespace() {
                    Some(uri) => format!("{{{}}}{}", uri, a.name()),
                    None => a.name().to_string(),
                };
                (cle, a.value().to_string())
            })
            .collect();
        attributs.sort();
        let cle = (nom_local(enfant).to_string(), attributs);
        if let Some(fid) = self.formes_en_ligne.get(&cle) {
            return Ok(fid.clone());
        }
        let fid = format!("inline_forme_{}", self.formes_en_ligne.len() + 1);
        self.formes_en_ligne.insert(cle, fid.clone());
        self.lire_forme(&fid, elem, 1.0)?;
        Ok(fid)
    }

    /// Une forme du dictionnaire utilisateur, de ses unites propres a
    /// celles du fichier.
    fn echelle_forme_user(forme: &mut UserShape, k: f64) {
        let mul = |p: &Point| Point::new(p.x * k, p.y * k);
        for t in &mut forme.tracks {
            t.points = t.points.iter().map(mul).collect();
            t.width *= k;
        }
        for a in &mut forme.arcs {
            (a.start, a.end, a.center, a.width) = (mul(&a.start), mul(&a.end), mul(&a.center), a.width * k);
        }
        for pl in &mut forme.planes {
            for c in &mut pl.contours {
                c.outline = c.outline.iter().map(mul).collect();
                for t in &mut c.cutouts {
                    *t = t.iter().map(mul).collect();
                }
            }
        }
        for t in &mut forme.texts {
            t.location = mul(&t.location);
        }
    }

    // ------------------------------------------------------------------
    // Étape 4 : Dictionnaire utilisateur
    // ------------------------------------------------------------------

    fn parse_user_dictionary(&mut self, racine: N) -> Py<()> {
        let Some(user_dict) = self.find(racine, "Content").and_then(|c| self.find(c, "DictionaryUser")) else {
            return Ok(());
        };
        let k = self.k_dico(user_dict);

        for entry in self.findall(user_dict, "EntryUser") {
            let Some(entry_id) = attr_plein(entry, "id") else { continue };
            let mut user_shape = UserShape { id: entry_id.to_string(), ..UserShape::default() };

            for line in self.iter(entry, "Line") {
                // une coordonnée manquante ou illisible : la ligne est ignorée
                if let Some([x1, y1, x2, y2]) = Self::quatre(line, ["startX", "startY", "endX", "endY"]) {
                    user_shape.tracks.push(Track {
                        width: self.get_width(line),
                        points: vec![Point::new(x1, y1), Point::new(x2, y2)],
                        layer_name: String::new(),
                        net_name: String::new(),
                        fill_property: self.get_fill_desc(line),
                        bout: String::new(),
                    });
                }
            }

            for poly in self.iter(entry, "Polyline") {
                let width = self.get_width(poly);
                let fill_prop = self.get_fill_desc(poly);
                let points = self.parse_polygon(poly)?;
                if points.len() >= 2 {
                    user_shape.tracks.push(Track {
                        width,
                        points,
                        layer_name: String::new(),
                        net_name: String::new(),
                        fill_property: fill_prop,
                        bout: String::new(),
                    });
                }
            }

            for arc in self.iter(entry, "Arc") {
                if let Some((start, end, center)) = Self::trois_points(arc) {
                    user_shape.arcs.push(Arc {
                        width: self.get_width(arc),
                        start,
                        end,
                        center,
                        clockwise: vrai(arc, "clockwise"),
                        layer_name: String::new(),
                        net_name: String::new(),
                        fill_property: self.get_fill_desc(arc),
                        bout: String::new(),
                    });
                }
            }

            for contour in self.iter(entry, "Contour") {
                if let Some(polygon) = self.find(contour, "Polygon") {
                    let outline_points = self.parse_polygon(polygon)?;
                    if outline_points.len() >= 3 {
                        let mut contour_data = Contour { outline: outline_points, cutouts: Vec::new() };
                        let fill_prop = self.get_fill_desc(polygon);
                        for cutout in self.findall(contour, "Cutout") {
                            let cutout_points = self.parse_polygon(cutout)?;
                            if cutout_points.len() >= 3 {
                                contour_data.cutouts.push(cutout_points);
                            }
                        }
                        user_shape.planes.push(CopperPlane {
                            layer_name: String::new(),
                            net_name: String::new(),
                            contours: vec![contour_data],
                            fill_property: fill_prop,
                        });
                    }
                }
            }

            for text_node in self.iter(entry, "Text") {
                user_shape.texts.extend(self.process_text(text_node, "", "", None));
            }

            if k != 1.0 {
                Self::echelle_forme_user(&mut user_shape, k);
            }
            self.design.user_shapes.insert(entry_id.to_string(), user_shape);
        }
        Ok(())
    }

    /// Quatre coordonnées obligatoires et finies, ou rien.
    fn quatre(n: N, cles: [&str; 4]) -> Option<[f64; 4]> {
        Some([
            Self::fini(attr(n, cles[0]))?,
            Self::fini(attr(n, cles[1]))?,
            Self::fini(attr(n, cles[2]))?,
            Self::fini(attr(n, cles[3]))?,
        ])
    }

    /// Début, fin et centre d'un <Arc>, ou rien si une coordonnée manque.
    fn trois_points(n: N) -> Option<(Point, Point, Point)> {
        let [sx, sy, ex, ey] = Self::quatre(n, ["startX", "startY", "endX", "endY"])?;
        let (cx, cy) = (Self::fini(attr(n, "centerX"))?, Self::fini(attr(n, "centerY"))?);
        Some((Point::new(sx, sy), Point::new(ex, ey), Point::new(cx, cy)))
    }

    // ------------------------------------------------------------------
    // Accesseurs génériques
    // ------------------------------------------------------------------

    fn get_width(&self, elem: N) -> f64 {
        if let Some(largeur) = self.find(elem, "LineDesc").and_then(|d| attr(d, "lineWidth")) {
            return Self::safe_float(Some(largeur));
        }
        self.find(elem, "LineDescRef")
            .and_then(|r| attr(r, "id"))
            .and_then(|id| self.line_width_dict.get(id).copied())
            .unwrap_or(0.0)
    }

    fn get_location(&self, parent_elem: N) -> Point {
        match self.find(parent_elem, "Location") {
            Some(loc) => Point::new(Self::safe_float(attr(loc, "x")), Self::safe_float(attr(loc, "y"))),
            None => Point::new(0.0, 0.0),
        }
    }

    /// lineEnd du trait (ROUND, SQUARE ou NONE), local ou par LineDescRef.
    fn get_line_end(&self, elem: N) -> String {
        if let Some(line_desc) = self.find(elem, "LineDesc") {
            return attr_plein(line_desc, "lineEnd").unwrap_or("ROUND").to_uppercase();
        }
        self.find(elem, "LineDescRef")
            .and_then(|r| attr(r, "id"))
            .and_then(|id| self.line_end_dict.get(id).cloned())
            .unwrap_or_else(|| "ROUND".into())
    }

    fn get_fill_desc(&self, parent_elem: N) -> String {
        if let Some(fill_desc) = self.find(parent_elem, "FillDesc") {
            return attr_ou(fill_desc, "fillProperty", "HOLLOW").to_uppercase();
        }
        self.find(parent_elem, "FillDescRef")
            .and_then(|r| attr(r, "id"))
            .and_then(|id| self.fill_dict.get(id).cloned())
            .unwrap_or_else(|| "UNKNOWN".into())
    }

    /// <Xform> : (rotation, miroir), ou rien s'il n'y en a pas.
    fn xform(&self, elem: N) -> Option<(f64, bool)> {
        self.find(elem, "Xform").map(|x| (Self::safe_float(attr(x, "rotation")), vrai(x, "mirror")))
    }

    // ------------------------------------------------------------------
    // Étape 5 : Définitions de padstacks
    // ------------------------------------------------------------------

    fn parse_padstack_defs(&mut self, racine: N) {
        const TROUS: [&str; 3] = ["padstackholedef", "layerhole", "hole"];
        for elem in tous(racine) {
            let tag = nom_local(elem).to_lowercase();
            if tag != "padstackdef" && tag != "padstack" {
                continue;
            }
            let Some(name) = attr_plein(elem, "name").or(attr_plein(elem, "id")).or_else(|| {
                enfants(elem)
                    .find(|c| TROUS.contains(&nom_local(*c).to_lowercase().as_str()))
                    .and_then(|c| attr_plein(c, "name"))
            }) else {
                continue;
            };

            let mut pdef = match self.design.padstacks.get(name) {
                Some(p) => (**p).clone(),
                None => PadStackDefinition::new(name),
            };

            let mut is_via = attr_ou(elem, "padUsage", "").to_uppercase() == "VIA";

            for child in enfants(elem) {
                let child_tag = nom_local(child).to_lowercase();

                if TROUS.contains(&child_tag.as_str()) {
                    // LE CALQUE DE PERCAGE QUE LE TROU DESIGNE : c'est lui
                    // qui portera le <Span>, donc la portee du via. On le
                    // retient ici parce que c'est le seul endroit ou le lien
                    // existe -- le trou lui-meme ne dit rien de ses couches.
                    if let Some(calque) = attr_plein(child, "layerRef") {
                        pdef.hole_layer_ref = calque.to_string();
                    }
                    if let Some(d) = attr(child, "diameter") {
                        pdef.hole_diameter = Self::safe_float(Some(d));
                    } else {
                        for sub in enfants(child) {
                            if nom_local(sub).to_lowercase() == "circle" {
                                pdef.hole_diameter = Self::safe_float(attr(sub, "diameter"));
                            }
                        }
                    }
                } else if child_tag == "padstackpaddef" || child_tag == "layerpad" {
                    let layer_ref = attr_ou(child, "layerRef", "UNKNOWN");
                    let pad_use = attr_ou(child, "padUse", "REGULAR").to_uppercase();
                    if attr_ou(child, "padUsage", "").to_uppercase() == "VIA" {
                        is_via = true;
                    }
                    if pad_use == "THERMAL" {
                        // le raccord a un plan negatif, pas la pastille :
                        // il ecrasait celle de la couche
                        continue;
                    }

                    let mut current_pad_diam = Self::safe_float(attr(child, "diameter"));
                    let mut current_shape_ref = "";

                    for sub in enfants(child) {
                        let sub_tag = nom_local(sub).to_lowercase();
                        if sub_tag == "circle" {
                            current_pad_diam = Self::safe_float(attr(sub, "diameter"));
                        } else if sub_tag == "standardprimitiveref" {
                            current_shape_ref = attr_ou(sub, "id", "");
                        }
                    }

                    if !current_shape_ref.is_empty() && current_pad_diam == 0.0 {
                        current_pad_diam = self.shape_max_dimension(current_shape_ref);
                    }

                    if pad_use == "REGULAR" && current_pad_diam > pdef.pad_diameter {
                        pdef.pad_diameter = current_pad_diam;
                    }

                    let rang = match pdef.pads.iter().position(|p| p.layer_ref == layer_ref) {
                        Some(i) => i,
                        None => {
                            pdef.pads.push(PadDef::new(layer_ref));
                            pdef.pads.len() - 1
                        }
                    };
                    let existing_pdef = &mut pdef.pads[rang];

                    if pad_use == "ANTIPAD" {
                        existing_pdef.antipad_diameter = current_pad_diam;
                    } else {
                        existing_pdef.pad_diameter = current_pad_diam;
                        existing_pdef.shape_ref = current_shape_ref.to_string();
                    }
                }
            }

            if is_via {
                self.via_def_names.insert(name.to_string());
            }

            self.design.padstacks.insert(name.to_string(), Partage::new(pdef));
        }
    }

    fn process_step_padstack(&mut self, ps_elem: N) {
        let net_name = attr_ou(ps_elem, "net", "");

        self.inline_ps_counter += 1;
        let ps_name = format!("inline_step_ps_{}", self.inline_ps_counter);
        let mut pdef = PadStackDefinition::new(&ps_name);

        let mut loc: Option<Point> = None;
        let mut rot = 0.0;
        let mut mirror = false;
        let mut pin_ref = String::new();
        let point = |n: N, x: &str, y: &str| Point::new(Self::safe_float(attr(n, x)), Self::safe_float(attr(n, y)));

        for hole in self.findall(ps_elem, "LayerHole") {
            if let Some(calque) = attr_plein(hole, "layerRef") {
                pdef.hole_layer_ref = calque.to_string();
            }
            if let Some(d) = attr(hole, "diameter") {
                pdef.hole_diameter = Self::safe_float(Some(d));
                if let Some(loc_node) = self.find(hole, "Location") {
                    loc = Some(point(loc_node, "x", "y"));
                } else if attr(hole, "x").is_some() && attr(hole, "y").is_some() {
                    loc = Some(point(hole, "x", "y"));
                }
            }
        }

        for lpad in self.findall(ps_elem, "LayerPad") {
            let layer_ref = attr_ou(lpad, "layerRef", "");
            let pad_use = attr_ou(lpad, "padUse", "REGULAR").to_uppercase();
            if pad_use == "THERMAL" {
                continue; // raccord a un plan negatif, pas la pastille
            }

            if loc.is_none() {
                if let Some(loc_node) = self.find(lpad, "Location") {
                    loc = Some(point(loc_node, "x", "y"));
                }
            }

            if let Some((r, m)) = self.xform(lpad) {
                (rot, mirror) = (r, m);
            }

            let pref = self.find(lpad, "StandardPrimitiveRef").or_else(|| self.find(lpad, "UserPrimitiveRef"));
            let shape_ref = pref.map_or("", |p| attr_ou(p, "id", ""));
            let pad_diam = if shape_ref.is_empty() { 0.0 } else { self.shape_max_dimension(shape_ref) };

            let rang = match pdef.pads.iter().position(|p| p.layer_ref == layer_ref) {
                Some(i) => i,
                None => {
                    pdef.pads.push(PadDef::new(layer_ref));
                    pdef.pads.len() - 1
                }
            };
            let existing_pdef = &mut pdef.pads[rang];

            if pad_use == "ANTIPAD" {
                existing_pdef.antipad_diameter = pad_diam;
            } else {
                existing_pdef.pad_diameter = pad_diam;
                existing_pdef.shape_ref = shape_ref.to_string();
            }

            if pin_ref.is_empty() {
                if let Some(pin_node) = self.find(lpad, "PinRef") {
                    pin_ref = Self::pin_ref_de(pin_node);
                }
            }
        }

        let Some(loc) = loc else { return };

        self.design.padstacks.insert(ps_name.clone(), Partage::new(pdef));
        self.design.standalone_pads.push(PadInstance {
            padstack_ref: ps_name,
            location: loc,
            rotation: rot,
            mirror,
            pin_ref,
            net_name: net_name.to_string(),
        });
    }

    /// <PinRef componentRef="U1" pin="3"/> -> « U1.3 », ou la broche seule.
    fn pin_ref_de(pin_node: N) -> String {
        let comp_ref = attr_ou(pin_node, "componentRef", "");
        let pin = attr_ou(pin_node, "pin", "");
        if comp_ref.is_empty() {
            pin.to_string()
        } else {
            format!("{comp_ref}.{pin}")
        }
    }

    // ------------------------------------------------------------------
    // BOM
    // ------------------------------------------------------------------

    fn parse_bom(&mut self, racine: N) {
        let boms: Vec<N> = racine.descendants().skip(1).filter(|n| self.est(*n, "BomItem")).collect();

        // Index O(1) ref_des -> Component (première occurrence, comme l'ancienne recherche linéaire)
        let mut comps_by_refdes: HashMap<String, usize> = HashMap::new();
        for (i, comp) in self.design.components.iter().enumerate() {
            if !comp.ref_des.is_empty() {
                comps_by_refdes.entry(comp.ref_des.clone()).or_insert(i);
            }
        }
        self.groupes_bom.resize(self.design.components.len(), None);

        for (rang_bom, bom) in boms.into_iter().enumerate() {
            let part = attr_ou(bom, "OEMDesignNumberRef", "");
            let bom_props = self.lire_proprietes(bom);

            let mut tol = premiere_prop(
                &bom_props,
                &["TOLERANCE", "TOL", "RESISTANCE_TOLERANCE", "CAPACITANCE_TOLERANCE", "INDUCTANCE_TOLERANCE"],
            );
            let mut val = premiere_prop(
                &bom_props,
                &["VALUE", "VAL", "RESISTANCE", "CAPACITANCE", "INDUCTANCE", "COMMENT", "DEVICE"],
            );

            if !part.is_empty() {
                let (v_part, t_part) = Self::extraire_val_tol(part);
                if val.is_empty() && !v_part.is_empty() {
                    val = v_part;
                }
                if tol.is_empty() && !t_part.is_empty() {
                    tol = t_part;
                }
                if val.is_empty() {
                    val = part.to_string();
                }
            }

            for ref_des_elem in self.findall(bom, "RefDes") {
                let name = attr_ou(ref_des_elem, "name", "");
                if name.is_empty() {
                    continue;
                }
                let pose = py::strip(attr_ou(ref_des_elem, "populate", "true")).to_lowercase() != "false";

                if let Some(&i) = comps_by_refdes.get(name) {
                    let comp = &mut self.design.components[i];
                    if (comp.value.is_empty() || comp.value == comp.part_number) && !val.is_empty() {
                        comp.value = val.clone();
                    }
                    if comp.tolerance.is_empty() && !tol.is_empty() {
                        comp.tolerance = tol.clone();
                    }
                    if comp.part_number.is_empty() && !part.is_empty() {
                        comp.part_number = part.to_string();
                    }
                    if !pose {
                        comp.populate = false;
                    }
                    let mut ajout = false;
                    for (k, v) in &bom_props {
                        if !comp.properties.contains_key(k) {
                            comp.properties.insert(k.clone(), v.clone());
                            ajout = true;
                        }
                    }
                    if comp.comp_type.is_empty() || comp.comp_type == "UNKNOWN" {
                        comp.comp_type = Self::qualifier_type_composant(name, &comp.value, Some(&comp.properties));
                    }
                    // le dict partagé des composants nés du même <BomItem>
                    if let (true, Some(groupe)) = (ajout, self.groupes_bom[i]) {
                        let props = self.design.components[i].properties.clone();
                        for (j, autre) in self.design.components.iter_mut().enumerate() {
                            if j != i && self.groupes_bom[j] == Some(groupe) {
                                autre.properties = props.clone();
                            }
                        }
                    }
                } else {
                    let ctype = Self::qualifier_type_composant(name, &val, Some(&bom_props));
                    self.design.components.push(Component {
                        ref_des: name.to_string(),
                        package_ref: attr_ou(ref_des_elem, "packageRef", "").to_string(),
                        layer_ref: attr_ou(ref_des_elem, "layerRef", "UNKNOWN").to_string(),
                        location: Point::new(0.0, 0.0),
                        rotation: 0.0,
                        mirror: false,
                        mount_type: "UNKNOWN".into(),
                        value: val.clone(),
                        tolerance: tol.clone(),
                        comp_type: ctype,
                        part_number: part.to_string(),
                        properties: bom_props.clone(),
                        package_obj: None,
                        pin_nets: IndexMap::new(),
                        populate: pose,
                        pads_repli: Vec::new(),
                    });
                    self.groupes_bom.push(Some(rang_bom));
                    comps_by_refdes.insert(name.to_string(), self.design.components.len() - 1);
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Spécifications de matériau (<Spec> / <SpecRef>)
    // ------------------------------------------------------------------

    /// <Content>, <Ecad> et <Ecad><CadHeader> : là où vivent les <Spec>.
    fn racines_specs<'a, 'i>(&self, racine: N<'a, 'i>) -> Vec<N<'a, 'i>> {
        let mut racines = Vec::new();
        racines.extend(self.find(racine, "Content"));
        if let Some(ecad) = self.find(racine, "Ecad") {
            racines.push(ecad);
            racines.extend(self.find(ecad, "CadHeader"));
        }
        racines
    }

    /// Indexe les <Spec> du fichier : nom -> {dk, df, material}.
    ///
    /// C'est là que vit la permittivité d'un stratifié dans un fichier
    /// IPC-2581 réel : la couche ne la porte pas, elle pointe une
    /// spécification par un <SpecRef>. Sans cette table, un fichier
    /// parfaitement renseigné ressortait sans le moindre Dk.
    ///
    /// Deux emplacements, parce que les deux se rencontrent : sous <Content>,
    /// et sous <Ecad><CadHeader> -- c'est là que l'écrivent les outils du
    /// commerce.
    ///
    /// Deux écritures, aussi. La valeur peut être un attribut de la balise
    /// (<Dielectric dielectricConstant="4.37"/>), ou tenir dans la forme
    /// « type + Property », de loin la plus courante :
    ///
    ///     <Spec name="DielectricLayer-1-2_Dielectric">
    ///      <Dielectric type="DIELECTRIC_CONSTANT">
    ///       <Property value="4.37"/>
    ///      </Dielectric>
    ///     </Spec>
    ///
    /// où c'est le `type` qui dit de quelle grandeur il s'agit, et le
    /// <Property> qui la porte. On lit les deux formes.
    fn parse_specs(&mut self, racine: N) {
        self.specs.clear();
        for r in self.racines_specs(racine) {
            for spec in self.findall(r, "Spec") {
                let Some(name) = attr_plein(spec, "name").or(attr_plein(spec, "id")) else { continue };
                let mut vals = self.specs.get(name).cloned().unwrap_or_default();
                // une <Spec> de regle (classe, paire, contrainte...) : lue a
                // part, elle ne decrit pas une matiere
                if let Some(regle) = self.lire_spec_regle(spec) {
                    self.spec_regles.insert(name.to_string(), regle);
                    self.ranger_regle(name, None);
                }

                // -- forme « type + Property » ---------------------------
                for enfant in enfants(spec) {
                    let genre = match attr_plein(enfant, "type") {
                        Some(t) => t.to_uppercase(),
                        None => nom_local(enfant).to_lowercase().to_uppercase(),
                    };
                    let rugosite = genre.contains("ROUGHNESS");
                    if rugosite {
                        // <Conductor type="SURFACE_ROUGHNESS_UPFACING"> et
                        // ses soeurs DOWNFACING / TREATED : la rugosite du
                        // feuillard, une par face. On garde la PLUS FORTE --
                        // la face qui regarde le plan n'est pas dite, et c'est
                        // le sens qui n'enjolive pas les pertes.
                        // Les <Property> NOMMEES HURAY_* (export de l'editeur
                        // PCB) sont les parametres du modele de Huray, pas un
                        // Rq : rayon des nodules (longueur) et rapport de
                        // surface (sans unite).
                        let (mut a, mut sr) = match vals.huray {
                            Some(h) => (Some(h.a), Some(h.sr)),
                            None => (None, None),
                        };
                        for p in self.iter(enfant, "Property") {
                            let nom_p = attr_ou(p, "name", "").to_uppercase();
                            if !nom_p.contains("HURAY") {
                                continue;
                            }
                            let rayon = nom_p.contains("RADIUS");
                            let v = if rayon {
                                Self::rugosite_um(p, attr_ou(p, "value", ""))
                            } else {
                                Self::safe_float(attr(p, "value"))
                            };
                            if v > 0.0 {
                                *(if rayon { &mut a } else { &mut sr }) = Some(v);
                            }
                        }
                        if let (Some(a), Some(sr)) = (a, sr) {
                            vals.huray = Some(Huray { a, sr });
                            continue;
                        }
                        if a.is_some() || sr.is_some() {
                            continue;
                        }
                    }
                    let Some(valeur) =
                        tous(enfant).find_map(|noeud| attr_plein(noeud, "value").or(attr_plein(noeud, "text")))
                    else {
                        continue;
                    };
                    if rugosite {
                        let rug = Self::rugosite_um(enfant, valeur);
                        if rug > vals.rugosite_um.unwrap_or(0.0) {
                            vals.rugosite_um = Some(rug);
                        } else if vals.rugosite_um.is_none()
                            && Self::safe_float_ou(Some(&valeur.replace(',', ".")), -1.0) == 0.0
                        {
                            // 0 ecrit : un cuivre DECLARE lisse (1.81), pas
                            // une rugosite absente
                            vals.rugosite_um = Some(0.0);
                        }
                        continue;
                    }
                    let cible = if genre.contains("DIELECTRIC") || genre.contains("PERMITTIV") {
                        &mut vals.dk
                    } else if genre.contains("LOSS") || genre.contains("TANGENT") || genre.contains("DISSIPATION") {
                        &mut vals.df
                    } else if genre.contains("MATERIAL") {
                        &mut vals.material
                    } else {
                        continue;
                    };
                    cible.get_or_insert_with(|| valeur.to_string());
                }

                // -- forme « tout en attributs » -------------------------
                for elem in tous(spec) {
                    let interne = elem != spec;
                    for a in elem.attributes() {
                        if a.value().is_empty() || a.namespace().is_some() {
                            continue;
                        }
                        let bas = a.name().to_lowercase();
                        let cible = if SPEC_DK.contains(&bas.as_str()) {
                            &mut vals.dk
                        } else if SPEC_DF.contains(&bas.as_str()) {
                            &mut vals.df
                        // « name » et « type » ne désignent un matériau que sur
                        // un enfant : sur <Spec>, « name » est le nom de la spec
                        } else if SPEC_MAT.contains(&bas.as_str()) && interne {
                            &mut vals.material
                        } else {
                            continue;
                        };
                        cible.get_or_insert_with(|| a.value().to_string());
                    }
                }

                if vals == SpecVals::default() {
                    self.specs.remove(name);
                } else {
                    self.specs.insert(name.to_string(), vals);
                }
            }
        }
    }

    /// Complète une couche d'empilage avec ce que disent ses <SpecRef>.
    ///
    /// Ce qui est écrit sur la couche elle-même l'emporte : la spec est un
    /// défaut partagé, la propriété locale une précision.
    fn appliquer_specs(specs: &HashMap<String, SpecVals>, refs: &[String], couche: &mut StackupLayer) {
        for id in refs {
            let Some(vals) = specs.get(id) else { continue };
            if couche.dk.is_empty() {
                couche.dk = vals.dk.clone().unwrap_or_default();
            }
            if couche.df.is_empty() {
                couche.df = vals.df.clone().unwrap_or_default();
            }
            if couche.material.is_empty() {
                couche.material = vals.material.clone().unwrap_or_default();
            }
            let sans_rugosite = couche.rugosite_um == 0.0;
            match vals.rugosite_um {
                Some(r) if r != 0.0 && sans_rugosite => {
                    couche.rugosite_um = r;
                    couche.rugosite_declaree = false;
                }
                // 0 ecrit : lisse DECLARE (1.81), si rien d'autre ne l'a dite
                Some(r) if r == 0.0 && sans_rugosite && couche.rugosite_huray.is_none() => {
                    couche.rugosite_um = 0.0;
                    couche.rugosite_declaree = true;
                }
                _ => {}
            }
            // Huray (1.79) : {a (micrometres), sr}
            if couche.rugosite_huray.is_none() {
                couche.rugosite_huray = vals.huray;
            }
        }
    }

    /// La valeur d'une rugosite de <Spec>, en micrometres (0 si illisible).
    fn rugosite_um(elem: N, valeur: &str) -> f64 {
        let unite = tous(elem).find_map(|n| attr_plein(n, "unit").or(attr_plein(n, "units"))).unwrap_or("");
        let Some(v) = py::float(&valeur.replace(',', ".")) else { return 0.0 };
        let k = unites_um(&py::strip(unite).to_uppercase()).unwrap_or(if unite.is_empty() { 1.0 } else { 0.0 });
        if v > 0.0 && k > 0.0 {
            v * k
        } else {
            0.0
        }
    }

    // ------------------------------------------------------------------
    // Les regles : classes de nets, paires, contraintes (<Spec> de regle)
    // ------------------------------------------------------------------

    /// Un attribut de longueur d'une <Property> ou d'une <LineWidth>, en
    /// millimetres ; l'unite est celle de la balise, sinon celle du fichier
    /// (comme le contre-percage) -- en millimetres, une regle de 0.005 sans
    /// unite dans un fichier en pouces se lisait 0,005 mm au lieu de 0,127.
    /// None si rien de lisible.
    fn longueur_mm(&self, elem: N, cle: &str) -> Option<f64> {
        let v = attr_plein(elem, cle)?;
        let x = py::float(&v.replace(',', "."))?;
        let unite = attr_plein(elem, "unit").unwrap_or(if self.design.units.is_empty() { "MM" } else { &self.design.units });
        let k = unites_mm(&py::strip(unite).to_uppercase())?;
        Some(x * k)
    }

    /// Une <Spec> de regle, telle que l'ecrit l'export de l'editeur PCB :
    ///
    ///     <Spec name="CLASSE_Rapide">
    ///      <General type="ELECTRICAL">
    ///       <Property name="NET_CLASS" text="Rapide"/>
    ///       <Property name="TRACE_WIDTH" value="0.25" unit="MM"/>
    ///       <Property name="TRACE_WIDTH" value="0.37" unit="MM" layerOrGroupRef="Top"/>
    ///       ...
    ///      </General>
    ///      <Impedance value="50" tolPlus="10" tolMinus="10" tolPercent="true">
    ///       <SingleEnded structure="MICROSTRIP_MASK_COVERED">...</SingleEnded>
    ///      </Impedance>
    ///     </Spec>
    ///
    /// Rend (genre, regle) -- genre : « classe » (NET_CLASS), « paire »
    /// (DIFF_PAIR), « groupe » (MATCH_GROUP), « matrice » (CLEARANCE en
    /// texte « A|B ») ou « net » (le reste) --, ou None si la spec ne porte
    /// aucune <Property> nommee de regle ni d'<Impedance> : une spec de
    /// matiere n'est pas une regle.
    fn lire_spec_regle(&self, spec: N) -> Option<(&'static str, Objet)> {
        let mut r = Objet::new();
        let mut genre = "net";
        let mut connu = false;
        // r.setdefault(cle, {})[sous] = v
        fn dans(r: &mut Objet, cle: &str, sous: &str, v: Json) {
            if let Json::Objet(o) = r.entry(cle.to_string()).or_insert_with(|| Json::Objet(Objet::new())) {
                o.insert(sous.to_string(), v);
            }
        }
        // r.setdefault(cle, []).append(v)
        fn ajouter(r: &mut Objet, cle: &str, v: &str) {
            if let Json::Liste(l) = r.entry(cle.to_string()).or_insert_with(|| Json::Liste(Vec::new())) {
                l.push(texte(v));
            }
        }
        for gen in self.findall(spec, "General") {
            for p in self.findall(gen, "Property") {
                let nom = py::strip(attr_ou(p, "name", "")).to_uppercase();
                let txt = py::strip(attr_ou(p, "text", ""));
                let couche = py::strip(attr_ou(p, "layerOrGroupRef", ""));
                if nom == "NET_CLASS" && !txt.is_empty() {
                    genre = "classe";
                    r.insert("classe".into(), texte(txt));
                    connu = true;
                } else if nom == "DIFF_PAIR" && !txt.is_empty() {
                    genre = "paire";
                    r.insert("nom".into(), texte(txt));
                    connu = true;
                } else if nom == "MATCH_GROUP" && !txt.is_empty() {
                    genre = "groupe";
                    r.insert("nom".into(), texte(txt));
                    connu = true;
                } else if nom == "NET" && !txt.is_empty() {
                    r.insert("net".into(), texte(txt));
                    connu = true;
                } else if nom == "CLEARANCE" && txt.contains('|') {
                    if let Some(v) = self.longueur_mm(p, "value").filter(|v| *v != 0.0) {
                        genre = "matrice";
                        connu = true;
                        dans(&mut r, "cases", txt, py::round(v, 6).into());
                    }
                } else if nom == "ALLOWED_LAYER" && !couche.is_empty() {
                    ajouter(&mut r, "couches", couche);
                    connu = true;
                } else if nom == "TOPOLOGY" && topos_regle(&txt.to_uppercase()).is_some() {
                    r.insert("topo".into(), texte(topos_regle(&txt.to_uppercase()).unwrap_or("")));
                    connu = true;
                } else if nom == "MATCH_MODE" && !txt.is_empty() {
                    r.insert("mode".into(), texte(if txt.to_uppercase() == "DELAY" { "ps" } else { "mm" }));
                    connu = true;
                } else if nom == "MATCH_TOLERANCE" {
                    // en millimetres (unit="MM"), ou en picosecondes (text="ps")
                    if txt.to_lowercase() == "ps" {
                        r.insert("tol".into(), Self::safe_float(attr(p, "value")).into());
                    } else if let Some(v) = self.longueur_mm(p, "value") {
                        r.insert("tol".into(), py::round(v, 6).into());
                    }
                    connu = true;
                } else if nom == "MEMBER" && !txt.is_empty() {
                    ajouter(&mut r, "nets", txt);
                    connu = true;
                } else if let Some((cle, nature)) = props_regle(&nom) {
                    if nature == "txt" {
                        if !txt.is_empty() {
                            let v = if cle == "ordre" {
                                Json::Liste(txt.split(py::blanc).filter(|m| !m.is_empty()).map(texte).collect())
                            } else {
                                texte(txt)
                            };
                            r.insert(cle.into(), v);
                            connu = true;
                        }
                        continue;
                    }
                    let v = if nature == "mm" {
                        self.longueur_mm(p, "value")
                    } else {
                        attr(p, "value").and_then(py::float).filter(|v| v.is_finite())
                    };
                    let Some(v) = v else { continue };
                    let v = py::round(v, 6);
                    connu = true;
                    if (cle == "w" || cle == "gap") && !couche.is_empty() {
                        // la largeur (ou l'ecart) propre a une couche
                        dans(&mut r, &format!("{cle}L"), couche, v.into());
                    } else {
                        r.insert(cle.into(), v.into());
                        // les bornes d'une paire : value -tolMinus / +tolPlus
                        if (cle == "w" || cle == "gap") && genre == "paire" {
                            let lo = self.longueur_mm(p, "tolMinus");
                            let hi = self.longueur_mm(p, "tolPlus");
                            if let Some(lo) = lo {
                                r.insert(format!("{cle}Min"), py::round(v - lo, 6).into());
                            }
                            if let Some(hi) = hi {
                                r.insert(format!("{cle}Max"), py::round(v + hi, 6).into());
                            }
                        }
                    }
                }
            }
        }
        // l'impedance cible : la premiere <Impedance> (la cible est la meme
        // sur toutes les couches ; seule la structure de ligne change)
        if let Some(imp) = self.find(spec, "Impedance") {
            let z = Self::safe_float(attr(imp, "value"));
            if z > 0.0 {
                r.insert("z".into(), py::round(z, 6).into());
                let tol = Self::safe_float(attr(imp, "tolPlus"));
                if tol > 0.0 {
                    let pourcent = attr_ou(imp, "tolPercent", "").eq_ignore_ascii_case("true");
                    r.insert("zTol".into(), py::round(if pourcent { tol } else { 100.0 * tol / z }, 6).into());
                }
                connu = true;
            }
        }
        if !connu {
            return None;
        }
        if genre == "paire" {
            // la cible d'une paire est differentielle : on la nomme ainsi
            if let Some(z) = r.shift_remove("z") {
                r.insert("zdiff".into(), z);
            }
        }
        Some((genre, r))
    }

    /// Range dans design.regles_nets ce qu'une <Spec> de regle porte : la
    /// regle d'une classe, une paire, un groupe, la matrice, ou les
    /// contraintes propres a un net. Appelee une premiere fois sans net a la
    /// lecture de la spec -- une classe sans net, un groupe dont les membres
    /// n'ont pas de <LogicalNet> sont ainsi gardes --, puis pour chaque net
    /// qui la pointe par <SpecRef> : il rejoint sa classe, sa paire ou son
    /// groupe, et recoit ses contraintes.
    fn ranger_regle(&mut self, nom_spec: &str, net_name: Option<&str>) {
        let net_name = net_name.filter(|n| !n.is_empty());
        let regles = &mut self.design.regles_nets;
        let Some((genre, r)) = self.spec_regles.get(nom_spec) else { return };
        let chaine = |v: Option<&Json>| v.and_then(Json::as_str).unwrap_or("").to_string();
        let de_spec = |o: &Objet| o.get("spec").and_then(Json::as_str) == Some(nom_spec);
        match *genre {
            "classe" => {
                let classe = chaine(r.get("classe"));
                if !regles.regles_classes.contains_key(&classe) {
                    let regle: Objet =
                        r.iter().filter(|(k, _)| *k != "classe" && *k != "nets").map(|(k, v)| (k.clone(), v.clone())).collect();
                    regles.regles_classes.insert(classe.clone(), Json::Objet(regle));
                }
                if let Some(net) = net_name {
                    regles.classes.entry(net.to_string()).or_insert_with(|| texte(&classe));
                } else if let Some(Json::Liste(membres)) = r.get("nets") {
                    // les nets sans broche, que la classe nomme (1.80)
                    for m in membres {
                        regles.classes.entry(chaine(Some(m))).or_insert_with(|| texte(&classe));
                    }
                }
            }
            "paire" => {
                let rang = match regles.paires.iter().position(de_spec) {
                    Some(i) => i,
                    None => {
                        let mut paire = r.clone();
                        paire.insert("spec".into(), texte(nom_spec));
                        paire.entry("p".into()).or_insert_with(|| texte(""));
                        paire.entry("n".into()).or_insert_with(|| texte(""));
                        regles.paires.push(paire);
                        regles.paires.len() - 1
                    }
                };
                let paire = &mut regles.paires[rang];
                if let Some(net) = net_name {
                    let (p, n) = (chaine(paire.get("p")), chaine(paire.get("n")));
                    // « not paire["p"] » : la valeur vient d'un texte du fichier, jamais d'autre chose
                    if net != p && net != n {
                        if !paire.get("p").is_some_and(Json::vrai) {
                            paire.insert("p".into(), texte(net));
                        } else if !paire.get("n").is_some_and(Json::vrai) {
                            paire.insert("n".into(), texte(net));
                        }
                    }
                }
            }
            "groupe" => {
                let rang = match regles.groupes.iter().position(de_spec) {
                    Some(i) => i,
                    None => {
                        let mode = r.get("mode").cloned().unwrap_or_else(|| texte("mm"));
                        let defaut = if mode.as_str() == Some("mm") { 0.5 } else { 10.0 };
                        let mut g = Objet::new();
                        g.insert("spec".into(), texte(nom_spec));
                        g.insert("nom".into(), r.get("nom").cloned().unwrap_or_else(|| texte("")));
                        g.insert("mode".into(), mode);
                        g.insert("tol".into(), r.get("tol").cloned().unwrap_or(Json::Flottant(defaut)));
                        g.insert("ref".into(), r.get("ref").cloned().unwrap_or_else(|| texte("")));
                        g.insert("nets".into(), r.get("nets").filter(|n| n.vrai()).cloned().unwrap_or(Json::Liste(Vec::new())));
                        regles.groupes.push(g);
                        regles.groupes.len() - 1
                    }
                };
                if let (Some(net), Some(Json::Liste(nets))) = (net_name, regles.groupes[rang].get_mut("nets")) {
                    if !nets.iter().any(|n| n.as_str() == Some(net)) {
                        nets.push(texte(net));
                    }
                }
            }
            "matrice" => {
                if let Some(Json::Objet(cases)) = r.get("cases") {
                    for (k, v) in cases {
                        regles.matrice.insert(k.clone(), v.clone());
                    }
                }
            }
            _ => {
                let net = match net_name {
                    Some(n) => n.to_string(),
                    None => chaine(r.get("net")),
                };
                let c: Vec<(&String, &Json)> = r.iter().filter(|(k, _)| *k != "net").collect();
                if !net.is_empty() && !c.is_empty() {
                    if let Json::Objet(o) = regles.contraintes.entry(net).or_insert_with(|| Json::Objet(Objet::new())) {
                        for (k, v) in c {
                            o.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Étape 6 : ECAD
    // ------------------------------------------------------------------

    fn parse_stackup(&mut self, cad_data: N) -> Py<()> {
        let Some(stackup_node) = self.find(cad_data, "Stackup") else { return Ok(()) };
        self.design.total_thickness = Self::safe_float(attr(stackup_node, "overallThickness"));
        for group in self.findall(stackup_node, "StackupGroup") {
            for layer in self.findall(group, "StackupLayer") {
                let mut sl = StackupLayer::new(
                    attr_ou(layer, "layerOrGroupRef", "Unknown"),
                    Self::safe_float(attr(layer, "thickness")),
                    Self::safe_int(attr(layer, "sequence"), 0.0)?,
                );

                for prop in self.findall(layer, "Property") {
                    let p_name = attr_ou(prop, "name", "").to_uppercase();
                    let p_val = attr_ou(prop, "value", "").to_string();
                    match p_name.as_str() {
                        "DIELECTRICCONSTANT" | "DK" | "DIELECTRIC_CONSTANT" => sl.dk = p_val,
                        "LOSSTANGENT" | "DF" | "LOSS_TANGENT" => sl.df = p_val,
                        "MATERIAL" | "MATERIALREF" => sl.material = p_val,
                        _ => {}
                    }
                }

                // Ce que la couche ne porte pas, sa spécification le dit
                Self::appliquer_specs(&self.specs, &self.spec_refs(layer), &mut sl);

                self.design.stackup.push(sl);
            }
        }
        self.design.stackup.sort_by(|a, b| a.sequence.total_cmp(&b.sequence));
        Ok(())
    }

    fn parse_profile(&mut self, step_elem: N) -> Py<()> {
        let Some(profile) = self.find(step_elem, "Profile") else { return Ok(()) };
        let Some(polygon) = self.find(profile, "Polygon") else { return Ok(()) };
        let outline_points = self.parse_polygon(polygon)?;
        if outline_points.len() >= 3 {
            let mut contour_data = Contour { outline: outline_points, cutouts: Vec::new() };
            for cutout in self.findall(profile, "Cutout") {
                let cutout_points = self.parse_polygon(cutout)?;
                if cutout_points.len() >= 3 {
                    contour_data.cutouts.push(cutout_points);
                }
            }
            self.design.board_outline = Some(contour_data);
        }
        Ok(())
    }

    fn parse_ecad(&mut self, racine: N) -> Py<()> {
        let Some(ecad) = self.find(racine, "Ecad") else { return Ok(()) };
        let Some(cad_data) = self.find(ecad, "CadData") else { return Ok(()) };

        self.parse_stackup(cad_data)?;

        let mut layer_functions_map: HashMap<String, String> = HashMap::new();
        // Fonction et face de CHAQUE calque, empilage ou non : la serigraphie
        // (« Symbol-A ») n'est pas dans l'empilage et son nom ne dit pas sa
        // face.
        self.design.layer_info.clear();
        for layer in self.findall(cad_data, "Layer") {
            let Some(lname) = attr_plein(layer, "name") else { continue };
            let lfunc = attr_ou(layer, "layerFunction", "").to_uppercase();
            layer_functions_map.insert(lname.to_string(), lfunc.clone());
            self.design.layer_info.insert(
                lname.to_string(),
                LayerInfo { fonction: lfunc.clone(), face: attr_ou(layer, "side", "").to_uppercase() },
            );
            if let Some(span) = self.lire_span(layer) {
                self.drill_spans.insert(lname.to_string(), span);
            }
            let refs = self.spec_refs(layer);
            if !refs.is_empty() {
                self.calque_specs.insert(lname.to_string(), refs.clone());
            }

            let mut props: HashMap<String, &str> = HashMap::new();
            for prop in self.findall(layer, "Property") {
                props.insert(attr_ou(prop, "name", "").to_uppercase(), attr_ou(prop, "value", ""));
            }
            let prop = |cles: &[&str]| cles.iter().find_map(|c| props.get(*c).filter(|v| !v.is_empty()).copied());

            for sl in self.design.stackup.iter_mut().filter(|sl| sl.name == lname) {
                // layerFunction dit ce qu'est physiquement la couche :
                // CONDUCTOR, PLANE, DIELCORE, DIELPREG, SOLDERMASK...
                // Sans lui, rien ne distingue un conducteur d'un
                // diélectrique dans l'empilage, et le nom seul ment.
                if !lfunc.is_empty() {
                    sl.layer_type = lfunc.clone();
                }
                if let Some(v) = prop(&["DIELECTRICCONSTANT", "DK", "DIELECTRIC_CONSTANT"]) {
                    sl.dk = v.to_string();
                }
                if let Some(v) = prop(&["LOSSTANGENT", "DF", "LOSS_TANGENT"]) {
                    sl.df = v.to_string();
                }
                if let Some(v) = prop(&["MATERIAL", "MATERIALREF"]) {
                    sl.material = v.to_string();
                }
                // <Layer> peut lui aussi pointer une spécification
                Self::appliquer_specs(&self.specs, &refs, sl);
            }
        }

        // panneau d'abord, carte ensuite : le <Step> utile est celui qui porte le cuivre
        let steps = self.findall(cad_data, "Step");
        let Some(step) =
            steps.iter().find(|s| self.find(**s, "LayerFeature").is_some()).or(steps.first()).copied()
        else {
            return Ok(());
        };

        self.parse_profile(step)?;

        for ps_elem in self.findall(step, "PadStack") {
            self.process_step_padstack(ps_elem);
        }
        for pkg in self.findall(step, "Package") {
            self.process_package(pkg)?;
        }
        for comp in self.findall(step, "Component") {
            self.process_component(comp)?;
        }
        self.parse_logical_nets(step, racine);

        for layer_feature in self.findall(step, "LayerFeature") {
            let layer_ref = attr_ou(layer_feature, "layerRef", "Unknown");

            let local_func = attr_ou(layer_feature, "layerFunction", "").to_uppercase();
            let fonction = if local_func.is_empty() {
                layer_functions_map.get(layer_ref).cloned().unwrap_or_default()
            } else {
                local_func
            };

            match self.role_calque(layer_ref, &fonction) {
                "percage" => {
                    self.process_drill_layer(layer_feature, layer_ref);
                    continue;
                }
                "contour" => {
                    self.contour_de_repli(layer_feature)?;
                    continue;
                }
                "ignore" => {
                    self.design.ignored_layers.insert(layer_ref.to_string(), fonction);
                    continue;
                }
                _ => {}
            }

            for item_set in self.findall(layer_feature, "Set") {
                let net_name = attr_plein(item_set, "net").unwrap_or("Non-Net"); // net="" perdait ses pistes

                for pad_node in self.iter(item_set, "Pad") {
                    let pad_inst = self.process_pad_instance(pad_node, layer_ref, net_name, "");
                    self.design.standalone_pads.push(pad_inst);
                }

                for features in self.findall(item_set, "Features") {
                    self.process_features(features, layer_ref, net_name)?;
                }
            }

            for direct_features in self.findall(layer_feature, "Features") {
                self.process_features(direct_features, layer_ref, "Non-Net")?;
            }
        }

        self.repli_broches();
        Ok(())
    }

    /// La forme des broches d'un <Package> ne sert qu'au composant dont
    /// aucune pastille n'est posee dans les calques (`pads_repli`) : un
    /// export qui pose ses pastilles en <Set><Pad> ecrit presque toujours
    /// aussi la forme de la broche, et la lire partout doublerait chaque
    /// pastille de la carte.
    fn repli_broches(&mut self) {
        let poses: HashSet<&str> =
            self.design.standalone_pads.iter().map(|p| p.pin_ref.as_str()).filter(|p| !p.is_empty()).collect();
        for comp in &mut self.design.components {
            let Some(pkg) = &comp.package_obj else { continue };
            if !pkg.pads.is_empty() || pkg.pads_broches.is_empty() {
                continue;
            }
            if !pkg.pins.iter().any(|pin| poses.contains(format!("{}.{}", comp.ref_des, pin.number).as_str())) {
                comp.pads_repli = pkg.pads_broches.clone();
            }
        }
    }

    /// Ce qu'on fait d'un <LayerFeature> : "percage", "cuivre", "contour"
    /// ou "ignore".
    ///
    /// Seul le cuivre et les percages entrent dans une simulation ; le reste
    /// d'un export (serigraphie, masque, pate, zones de composant, keepouts,
    /// cotation) y serait pris pour du metal. Le tri se fait sur layerFunction ;
    /// un calque qui n'en declare pas est garde -- un cuivre perdu coute plus
    /// cher qu'un calque de trop.
    pub fn role_calque(&self, layer_ref: &str, fonction: &str) -> &'static str {
        let nom = layer_ref.to_uppercase();
        // ROUT : le calque des fentes (<SlotCavity>), comme l'ecrivent KiCad et
        // l'editeur ; seuls ses <Hole> et <SlotCavity> y seront lus
        if fonction.contains("DRILL")
            || fonction == "ROUT"
            || (fonction.is_empty() && (nom.contains("DRILL") || nom.contains("HOLE")))
        {
            return "percage";
        }
        if self.tout_garder || fonction.is_empty() || fonction_cuivre(fonction) {
            return "cuivre";
        }
        if ["BOARD_OUTLINE", "PROFILE", "BOARDOUTLINE"].contains(&fonction) {
            return "contour";
        }
        "ignore"
    }

    /// Un calque BOARD_OUTLINE ne sert que si <Profile> n'a rien donne.
    fn contour_de_repli(&mut self, layer_feature: N) -> Py<()> {
        if self.design.board_outline.is_some() {
            return Ok(());
        }
        for contour in self.iter(layer_feature, "Contour") {
            let Some(polygon) = self.find(contour, "Polygon") else { continue };
            let points = self.parse_polygon(polygon)?;
            if points.len() >= 3 {
                let mut contour_data = Contour { outline: points, cutouts: Vec::new() };
                for cutout in self.findall(contour, "Cutout") {
                    let trou = self.parse_polygon(cutout)?;
                    if trou.len() >= 3 {
                        contour_data.cutouts.push(trou);
                    }
                }
                self.design.board_outline = Some(contour_data);
                return Ok(());
            }
        }
        Ok(())
    }

    /// Lignes, polylignes, traits fermés, arcs et contours directement sous `parent`.
    fn simples(&mut self, parent: N, layer_ref: &str, net_name: &str, xf: Option<Xf>, mir: bool) -> Py<()> {
        for line in self.findall(parent, "Line") {
            self.process_simple_line(line, layer_ref, net_name, xf);
        }
        for poly in self.findall(parent, "Polyline") {
            self.process_polyline(poly, layer_ref, net_name, xf, None)?;
        }
        for outline in self.findall(parent, "Outline") {
            if let Some(poly) = self.find(outline, "Polygon") {
                self.process_polyline(poly, layer_ref, net_name, xf, Some(outline))?;
            }
        }
        for arc in self.findall(parent, "Arc") {
            self.process_arc(arc, layer_ref, net_name, xf, mir);
        }
        for contour in self.findall(parent, "Contour") {
            self.process_contour(contour, layer_ref, net_name, xf)?;
        }
        Ok(())
    }

    /// Traite un bloc <Features> : lignes, polylignes, arcs, contours,
    /// traits fermes (<Outline>), textes, <UserSpecial>, et la forme posee
    /// (reference ou en ligne), qui devient une pastille.
    ///
    /// LE <Location> ET LE <Xform> DU <Features> PLACENT CE QU'IL CONTIENT :
    /// le XSD (FeaturesType : Xform?, Location, Feature) donne les
    /// coordonnees du contenu relatives a eux. Miroir puis rotation, puis la
    /// translation : la matrice de la pastille dans la visionneuse
    /// (mdlPadDans). Les exports a (0, 0) -- pyedb, l'editeur PCB -- ne
    /// bougent pas.
    fn process_features(&mut self, features_elem: N, layer_ref: &str, net_name: &str) -> Py<()> {
        let loc_f = self.find(features_elem, "Location");
        let ox = loc_f.map_or(0.0, |l| Self::safe_float(attr(l, "x")));
        let oy = loc_f.map_or(0.0, |l| Self::safe_float(attr(l, "y")));
        let (rot, mir) = self.xform(features_elem).unwrap_or((0.0, false));
        let xf = (ox != 0.0 || oy != 0.0 || rot != 0.0 || mir).then(|| Xf {
            ox,
            oy,
            co: radians(rot).cos(),
            si: radians(rot).sin(),
            k: if mir { -1.0 } else { 1.0 },
        });

        self.simples(features_elem, layer_ref, net_name, xf, mir)?;
        // La norme (XSD rev. C) pose la place d'un <Text> dans le <Location>
        // de son <Features>, a cote de lui ; les premiers exports lus ici la
        // mettaient dans le <Text>. Celle du <Text> passe devant.
        let loc_defaut = loc_f.map(|_| Point::new(ox, oy));
        for text_node in self.findall(features_elem, "Text") {
            let t = self.process_text(text_node, layer_ref, net_name, loc_defaut);
            self.design.texts.extend(t);
        }

        if let Some(special_node) = self.find(features_elem, "UserSpecial") {
            // l'editeur PCB revision 1 y ecrivait ses traits en absolu
            let xf_special = if self.userspecial_absolu { None } else { xf };
            self.simples(special_node, layer_ref, net_name, xf_special, mir)?;
            for text_node in self.findall(special_node, "Text") {
                let t = self.process_text(text_node, layer_ref, net_name, None);
                self.design.texts.extend(t);
            }
        }

        // LA FORME POSEE : <StandardPrimitiveRef>, <UserPrimitiveRef> ou une
        // forme standard en ligne. Le <Features> a la forme d'une <Pad>
        // (Xform, Location, forme) : il en devient une, sur ce calque.
        let forme = self.forme_en_ligne(features_elem)?;
        if !forme.is_empty()
            || self.find(features_elem, "StandardPrimitiveRef").is_some()
            || self.find(features_elem, "UserPrimitiveRef").is_some()
        {
            let pad = self.process_pad_instance(features_elem, layer_ref, net_name, &forme);
            self.design.standalone_pads.push(pad);
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Perçages (calques DRILL)
    // ------------------------------------------------------------------

    /// La portée déclarée d'un calque de perçage, ou None.
    ///
    /// OÙ ELLE EST ÉCRITE. IPC-2581 la met sur le CALQUE et non sur le trou :
    ///
    ///     <Layer name="drill_1_4" layerFunction="DRILL" side="ALL">
    ///       <Span fromLayer="TOP" toLayer="IN2"/>
    ///     </Layer>
    ///
    /// TROIS FORMES ACCEPTÉES, et c'est de la tolérance d'exportateur, pas de
    /// la générosité : `<Span>` est la balise du schéma, `<SpanDescriptor>`
    /// traîne dans des exports plus anciens, et certains outils posent les deux
    /// noms en attributs du calque lui-même. Les trois disent la même chose ;
    /// n'en lire qu'une, c'est perdre la portée sur les fichiers des deux
    /// autres et retomber sur « traversant » sans le savoir.
    ///
    /// CE QU'ON NE FAIT PAS : deviner la portée d'après le NOM du calque
    /// (« drill_1_4 »). Une portée devinée à tort RESTREINT un via traversant,
    /// donc coupe un chemin de courant réel — le cuivre d'une couche se
    /// retrouve flottant et le solveur refuse tout le calcul. Supposer
    /// traversant se paie d'une résistance surestimée ; supposer borgne se paie
    /// d'un résultat faux. On ne suppose donc que dans le sens qui ne perd
    /// aucun chemin, et on le dit.
    fn lire_span(&self, elem: N) -> Option<(String, String)> {
        let paire = |a: &str, b: &str| (!a.is_empty() && !b.is_empty()).then(|| (a.to_string(), b.to_string()));
        for balise in ["Span", "SpanDescriptor"] {
            if let Some(node) = self.find(elem, balise) {
                let a = py::strip(attr_plein(node, "fromLayer").or(attr_plein(node, "from")).unwrap_or(""));
                let b = py::strip(attr_plein(node, "toLayer").or(attr_plein(node, "to")).unwrap_or(""));
                if let Some(p) = paire(a, b) {
                    return Some(p);
                }
            }
        }
        paire(py::strip(attr_ou(elem, "spanFromLayer", "")), py::strip(attr_ou(elem, "spanToLayer", "")))
    }

    /// La portée d'un perçage : son calque d'abord, son padstack ensuite.
    ///
    /// Rend (depuis, vers, provenance). La provenance est vide quand rien n'est
    /// déclaré — et c'est une information, pas un manque à taire.
    fn span_du_percage(&self, layer_ref: &str, padstack_ref: &str) -> (String, String, String) {
        if let Some(span) = self.drill_spans.get(layer_ref) {
            return (span.0.clone(), span.1.clone(), "calque".into());
        }
        if let Some(pdef) = self.design.padstacks.get(padstack_ref) {
            if !pdef.hole_layer_ref.is_empty() {
                if let Some(span) = self.drill_spans.get(&pdef.hole_layer_ref) {
                    return (span.0.clone(), span.1.clone(), "padstack".into());
                }
            }
        }
        (String::new(), String::new(), String::new())
    }

    fn process_drill_layer(&mut self, layer_elem: N, layer_ref: &str) {
        // LA PORTÉE DU CALQUE, LUE UNE FOIS. Un calque de perçage porte des
        // centaines de trous et une seule portée : la relire par trou coûterait
        // un `find` chacun pour le même résultat.
        if let Some(span_calque) = self.lire_span(layer_elem) {
            if !layer_ref.is_empty() {
                self.drill_spans.entry(layer_ref.to_string()).or_insert(span_calque);
            }
        }
        // UN CALQUE DE CONTRE-PERCAGE : sa fonction le dit (BACKDRILL), ou une
        // spec de back-drill pointee par le calque lui-meme.
        let backdrill = self.design.layer_info.get(layer_ref).is_some_and(|i| i.fonction.contains("BACKDRILL"));
        let mut specs_calque = self.calque_specs.get(layer_ref).cloned().unwrap_or_default();
        specs_calque.extend(self.spec_refs(layer_elem));
        for item_set in self.findall(layer_elem, "Set") {
            let padstack_ref = attr_ou(item_set, "geometry", "");
            let net_name = attr_ou(item_set, "net", "");
            let pad_usage = attr_ou(item_set, "padUsage", "").to_uppercase();
            let specs_set = self.spec_refs(item_set);

            let mut holes = self.findall(item_set, "Hole");
            for features in self.findall(item_set, "Features") {
                holes.extend(self.findall(features, "Hole"));
            }

            for hole in holes {
                let diameter = Self::safe_float(attr(hole, "diameter"));
                let plating = attr_ou(hole, "platingStatus", "UNKNOWN");

                let ou = self.find(hole, "Location").unwrap_or(hole);
                let (x, y) = (Self::safe_float(attr(ou, "x")), Self::safe_float(attr(ou, "y")));
                let loc = Point::new(x, y);
                // LE CONTRE-PERCAGE DU TROU, s'il en a un : la premiere spec
                // de back-drill parmi celles du trou, de son <Set>, de son
                // calque. Sur un trou nu, ou sur un calque BACKDRILL, le trou
                // est le FORET : il ne devient pas un via, il attend le sien.
                let nom_spec = self
                    .spec_refs(hole)
                    .iter()
                    .chain(&specs_set)
                    .chain(&specs_calque)
                    .find(|i| self.backdrill_specs.contains_key(*i))
                    .cloned()
                    .unwrap_or_default();
                if backdrill || (!nom_spec.is_empty() && plating.to_uppercase().contains("NON")) {
                    self.forets_cp.push(Foret {
                        x,
                        y,
                        diametre: diameter,
                        spec: nom_spec,
                        calque: layer_ref.to_string(),
                        span: self.drill_spans.get(layer_ref).cloned(),
                    });
                    continue;
                }
                let mut drill = Drill::new(loc, diameter, plating, padstack_ref, net_name);
                if !nom_spec.is_empty() {
                    drill.contre_percage = Some(self.cp_de_spec(&nom_spec));
                }
                // LA PORTÉE, TANT QU'ON A LE CALQUE SOUS LA MAIN. Plus loin,
                // `design.drills` n'est qu'une liste de trous : rien n'y dit
                // plus de quel calque ils viennent.
                (drill.span_from, drill.span_to, drill.span_source) = self.span_du_percage(layer_ref, padstack_ref);

                if pad_usage == "VIA" || self.via_def_names.contains(padstack_ref) {
                    let via_key = if padstack_ref.is_empty() {
                        format!("VIA_HOLE_D{}", py::repr_flottant(diameter))
                    } else {
                        padstack_ref.to_string()
                    };

                    if !self.design.padstacks.contains_key(&via_key) {
                        // AUCUNE DEFINITION DE PADSTACK POUR CE PERCAGE : on en
                        // fabrique une pour avoir quelque chose a dessiner.
                        // « perçage + 0,3 » est un anneau de 0,15 mm pose par
                        // convention -- un ordre de grandeur courant, et rien
                        // de plus. Il est MARQUE comme suppose : la simulation
                        // le fait entrer dans la capacite du via, le controle
                        // d'isolation mesure des distances contre lui, et ni
                        // l'un ni l'autre ne doit le prendre pour une cote du
                        // fichier.
                        let mut pdef = PadStackDefinition::new(&via_key);
                        pdef.hole_diameter = diameter;
                        // 0,3 MM, converti dans l'unite du fichier : en pouces,
                        // « + 0.3 » faisait un pad de 7,9 mm
                        pdef.pad_diameter = diameter + 0.3 / self.mm_du_fichier();
                        pdef.pad_supposee = true;
                        self.design.padstacks.insert(via_key.clone(), Partage::new(pdef));
                        self.via_def_names.insert(via_key.clone());
                    }

                    if drill.padstack_ref.is_empty() {
                        drill.padstack_ref = via_key;
                    }
                }

                drill.padstack_obj = self.design.padstacks.get(&drill.padstack_ref).cloned();
                self.design.drills.push(drill);
            }

            // LES FENTES (<SlotCavity>, calque ROUT) : Location au centre,
            // Xform pour l'angle, Oval longueur x largeur (width >= height
            // dans la norme ; si l'on a l'inverse, la longueur tourne de 90°).
            // Une autre forme que l'Oval (un <Outline>) n'est pas lue : la fente
            // est alors absente, comme avant que les SlotCavity soient lus.
            for fente in self.findall(item_set, "SlotCavity") {
                let Some(loc_node) = self.find(fente, "Location") else { continue };
                let Some(forme) = self.find(fente, "Oval") else { continue };
                let mut larg = Self::safe_float(attr(forme, "width"));
                let mut haut = Self::safe_float(attr(forme, "height"));
                let mut rot = self.xform(fente).map_or(0.0, |x| x.0);
                if haut > larg {
                    (larg, haut, rot) = (haut, larg, rot + 90.0);
                }
                if !(haut > 0.0) {
                    continue;
                }
                let mut drill = Drill::new(
                    Point::new(Self::safe_float(attr(loc_node, "x")), Self::safe_float(attr(loc_node, "y"))),
                    haut,
                    attr_ou(fente, "platingStatus", "UNKNOWN"),
                    padstack_ref,
                    net_name,
                );
                drill.slot_length = larg;
                drill.slot_rotation = modulo(rot, 360.0);
                (drill.span_from, drill.span_to, drill.span_source) = self.span_du_percage(layer_ref, padstack_ref);
                drill.padstack_obj = self.design.padstacks.get(&drill.padstack_ref).cloned();
                self.design.drills.push(drill);
            }
        }
    }

    // ------------------------------------------------------------------
    // Contre-perçage (back-drill)
    // ------------------------------------------------------------------

    /// Indexe les spécifications de contre-perçage : nom -> {depart,
    /// garde, moignon_mm}.
    ///
    /// CE QU'IPC-2581 ÉCRIT. Une <Spec> (sous <Content>, <Ecad> ou
    /// <CadHeader>, comme les spécifications de matériau) porte un élément
    /// <Backdrill> par grandeur, son `type` disant laquelle :
    ///
    ///     <Spec name="BD_1A">
    ///      <Backdrill type="START_LAYER">
    ///       <Property layerOrGroupRef="BOTTOM"/></Backdrill>
    ///      <Backdrill type="MUST_NOT_CUT_LAYER">
    ///       <Property layerOrGroupRef="IN1"/></Backdrill>
    ///      <Backdrill type="MAX_STUB_LENGTH">
    ///       <Property value="0.1" unit="MM"/></Backdrill>
    ///     </Spec>
    ///
    /// et le trou la désigne par un <SpecRef id="BD_1A"/>. C'est la forme de
    /// l'export IPC-2581 de KiCad. Ni la profondeur ni le diamètre du foret
    /// n'y sont : la profondeur se compte dans l'empilage, le diamètre n'est
    /// connu que si le foret a son propre trou (voir `rattacher_...`).
    fn parse_contre_percages(&mut self, racine: N) {
        let defaut = self.mm_du_fichier();
        for r in self.racines_specs(racine) {
            for spec in self.findall(r, "Spec") {
                let elems = self.findall(spec, "Backdrill");
                let Some(nom) = attr_plein(spec, "name").or(attr_plein(spec, "id")) else { continue };
                if elems.is_empty() {
                    continue;
                }
                let mut cp = self.backdrill_specs.get(nom).cloned().unwrap_or_default();
                for bd in elems {
                    let genre = attr_ou(bd, "type", "").to_uppercase();
                    let prop = self.find(bd, "Property");
                    // les attributs du <Property> passent devant ceux du <Backdrill>
                    let a = |cle: &str| prop.and_then(|p| attr(p, cle)).or(attr(bd, cle));
                    let calque =
                        py::strip(a("layerOrGroupRef").filter(|v| !v.is_empty()).or(a("layerRef")).unwrap_or(""));
                    if genre == "START_LAYER" && !calque.is_empty() {
                        cp.depart = Some(calque.to_string());
                    } else if genre == "MUST_NOT_CUT_LAYER" && !calque.is_empty() {
                        cp.garde = Some(calque.to_string());
                    } else if genre == "MAX_STUB_LENGTH" {
                        if let Some(v) = a("value") {
                            if Self::safe_float_ou(Some(v), -1.0) >= 0.0 {
                                let k = unites_mm(&a("unit").unwrap_or("").to_uppercase()).unwrap_or(defaut);
                                cp.moignon_mm = Some(py::round(Self::safe_float(Some(v)) * k, 6));
                            }
                        }
                    }
                }
                self.backdrill_specs.insert(nom.to_string(), cp);
            }
        }
    }

    /// Le contre-perçage d'un via d'après une spec : une copie, que le
    /// rattachement d'un foret peut compléter sans toucher aux autres vias.
    fn cp_de_spec(&self, nom: &str) -> ContrePercage {
        let s = self.backdrill_specs.get(nom).cloned().unwrap_or_default();
        ContrePercage {
            depart: s.depart,
            garde: s.garde,
            moignon_mm: s.moignon_mm,
            spec: Some(nom.to_string()),
            source: "spec".into(),
            calque: None,
            diametre: None,
        }
    }

    /// Les couches de cuivre de l'empilage, de la face du dessus à celle
    /// du dessous : c'est elles qui disent de quel côté le foret repasse.
    fn cuivres_empilage(&self) -> Vec<String> {
        let mut pile: Vec<&StackupLayer> = self.design.stackup.iter().collect();
        pile.sort_by(|a, b| a.sequence.total_cmp(&b.sequence));
        pile.iter()
            .filter(|sl| {
                let fonction = if sl.layer_type.is_empty() {
                    self.design.layer_info.get(&sl.name).map_or("", |i| i.fonction.as_str())
                } else {
                    &sl.layer_type
                };
                fonction_cuivre(&fonction.to_uppercase())
            })
            .map(|sl| sl.name.clone())
            .collect()
    }

    /// Chaque foret lu sur un calque de contre-perçage rejoint le via
    /// MÉTALLISÉ du même emplacement.
    ///
    /// Le <Span> du calque du foret va de la face percée à la dernière
    /// couche retirée : la face est celle des deux bouts qui est une face de
    /// cuivre, et la couche à ne pas couper, faute de MUST_NOT_CUT_LAYER, la
    /// suivante vers l'intérieur. Un foret sans via est gardé à part et
    /// compté : on ne fabrique pas un via pour lui.
    fn rattacher_contre_percages(&mut self) -> Py<()> {
        let cu = self.cuivres_empilage();
        let mut orphelins = Vec::new();
        if self.forets_cp.is_empty() {
            self.design.contre_percages_orphelins = orphelins;
            return Ok(());
        }
        let tol = 1e-3;
        // round(x / tol) : un entier pour Python, qui lève sur un non-fini.
        // shortcut: au-delà de 2^53 les cases voisines se confondent (Python compte en entiers
        // exacts) ; sans effet sous 9e12 unités de coordonnée.
        let case = |v: f64| -> Py<f64> { Ok(entier((v / tol).round_ties_even())? + 0.0) };
        // les trous metallises ranges par case de `tol` : chaque foret ne
        // regarde que les 9 cases voisines, au lieu de toute la liste
        // (2 000 forets x 50 000 trous = 1e8 comparaisons)
        let mut cases: HashMap<(u64, u64), Vec<usize>> = HashMap::new();
        for (rang, d) in self.design.drills.iter().enumerate() {
            if !d.plating.to_uppercase().contains("NON") {
                let cle = (case(d.location.x)?.to_bits(), case(d.location.y)?.to_bits());
                cases.entry(cle).or_default().push(rang);
            }
        }
        for f in std::mem::take(&mut self.forets_cp) {
            let (cx, cy) = (case(f.x)?, case(f.y)?);
            let mut rangs: Vec<usize> = Vec::new();
            for i in [-1.0, 0.0, 1.0] {
                for j in [-1.0, 0.0, 1.0] {
                    if let Some(l) = cases.get(&((cx + i + 0.0).to_bits(), (cy + j + 0.0).to_bits())) {
                        rangs.extend(l);
                    }
                }
            }
            rangs.sort();
            // le premier dans l'ordre du fichier, comme avant
            let cible = rangs.into_iter().find(|r| {
                let d = &self.design.drills[*r];
                (d.location.x - f.x).abs() <= tol && (d.location.y - f.y).abs() <= tol
            });
            let Some(cible) = cible else {
                orphelins.push(f);
                continue;
            };
            let mut cp = self.design.drills[cible].contre_percage.clone().unwrap_or_default();
            if !f.spec.is_empty() {
                let s = self.cp_de_spec(&f.spec);
                cp.depart = cp.depart.or(s.depart);
                cp.garde = cp.garde.or(s.garde);
                cp.moignon_mm = cp.moignon_mm.or(s.moignon_mm);
                cp.spec = cp.spec.or(s.spec);
            }
            cp.source = "calque".into();
            cp.calque = Some(f.calque.clone());
            if f.diametre > 0.0 {
                cp.diametre = Some(f.diametre);
            }
            if let (Some((a, b)), false) = (&f.span, cu.is_empty()) {
                let (haut, bas) = (&cu[0], &cu[cu.len() - 1]);
                let face_fin = if (a == haut || a == bas) && cu.contains(b) {
                    Some((a, b))
                } else if (b == haut || b == bas) && cu.contains(a) {
                    Some((b, a))
                } else {
                    None
                };
                if let Some((face, fin)) = face_fin {
                    cp.depart.get_or_insert_with(|| face.clone());
                    let i = cu.iter().position(|c| c == fin).unwrap_or(0) as i64 + if face == haut { 1 } else { -1 };
                    if cp.garde.is_none() && 0 <= i && (i as usize) < cu.len() {
                        cp.garde = Some(cu[i as usize].clone());
                    }
                }
            }
            self.design.drills[cible].contre_percage = Some(cp);
        }
        self.design.contre_percages_orphelins = orphelins;
        Ok(())
    }

    /// Une <Pad> (ou ce qui en a la forme : un <Features>, un <Pin>).
    /// `forme` : id d'une forme ecrite en ligne (forme_en_ligne), a la
    /// place d'un <StandardPrimitiveRef>.
    fn process_pad_instance(&mut self, pad_elem: N, current_layer: &str, default_net: &str, forme: &str) -> PadInstance {
        let net_name = attr_ou(pad_elem, "net", default_net);

        let mut padstack_ref = attr_plein(pad_elem, "padstackDefRef").unwrap_or(attr_ou(pad_elem, "geometry", "")).to_string();

        let mut pin_ref = attr_ou(pad_elem, "pinRef", "").to_string();
        if pin_ref.is_empty() {
            if let Some(pin_node) = self.find(pad_elem, "PinRef") {
                pin_ref = Self::pin_ref_de(pin_node);
            }
        }

        let loc = self.get_location(pad_elem);
        let (rot, mirror) = self.xform(pad_elem).unwrap_or((0.0, false));

        let pref = self.find(pad_elem, "StandardPrimitiveRef").or_else(|| self.find(pad_elem, "UserPrimitiveRef"));
        let pad_usage = attr_ou(pad_elem, "padUsage", "").to_uppercase();

        let shape_ref = if forme.is_empty() { pref.map_or("", |p| attr_ou(p, "id", "")) } else { forme };
        if !shape_ref.is_empty() {
            // UN PADSTACK PAR (forme, couche, usage), pas par pastille :
            // 20 000 pastilles identiques faisaient 20 000 definitions,
            // plus de la moitie du JSON envoye a la visionneuse.
            let cle = (shape_ref.to_string(), current_layer.to_string(), pad_usage.clone());
            padstack_ref = match self.inline_pads.get(&cle) {
                Some(ps_name) => ps_name.clone(),
                None => {
                    self.inline_ps_counter += 1;
                    let ps_name = format!("inline_pad_{}", self.inline_ps_counter);
                    self.inline_pads.insert(cle, ps_name.clone());
                    let mut pdef = PadStackDefinition::new(&ps_name);
                    let mut pad = PadDef::new(current_layer);
                    pad.pad_diameter = self.shape_max_dimension(shape_ref);
                    pad.shape_ref = shape_ref.to_string();
                    pdef.pads.push(pad);
                    self.design.padstacks.insert(ps_name.clone(), Partage::new(pdef));
                    ps_name
                }
            };
        }

        // Les vias placés sous forme de "Pad" : leur padstack devient un padstack de via
        if pad_usage == "VIA" && !padstack_ref.is_empty() {
            self.via_def_names.insert(padstack_ref.clone());
        }

        PadInstance { padstack_ref, location: loc, rotation: rot, mirror, pin_ref, net_name: net_name.to_string() }
    }

    // ------------------------------------------------------------------
    // Packages et composants
    // ------------------------------------------------------------------

    fn process_package(&mut self, pkg_elem: N) -> Py<()> {
        let Some(name) = attr_plein(pkg_elem, "name").or(attr_plein(pkg_elem, "id")) else { return Ok(()) };

        // Hauteur et corps du boitier
        let hauteur = Self::safe_float(attr(pkg_elem, "height"));
        let mut package = Package {
            name: name.to_string(),
            pins: Vec::new(),
            pads: Vec::new(),
            hauteur: (hauteur > 0.0).then_some(hauteur),
            corps: Vec::new(),
            pads_broches: Vec::new(),
        };
        if let Some(poly) = self.find(pkg_elem, "Outline").and_then(|c| self.find(c, "Polygon")) {
            package.corps = self.parse_polygon(poly)?;
        }

        // La forme ecrite sur la broche elle-meme (XSD : Xform?, Location,
        // StandardShape) : elle ne sert qu'en repli (voir repli_broches).
        for pin_elem in self.findall(pkg_elem, "Pin") {
            let number = attr_ou(pin_elem, "number", "?");
            let loc = self.get_location(pin_elem);
            package.pins.push(Pin {
                number: number.to_string(),
                x: loc.x,
                y: loc.y,
                r#type: attr_ou(pin_elem, "type", "UNKNOWN").to_string(),
            });

            for pad_elem in self.findall(pin_elem, "Pad") {
                let mut pad_inst = self.process_pad_instance(pad_elem, "ALL", "", "");
                if pad_inst.pin_ref.is_empty() {
                    pad_inst.pin_ref = number.to_string();
                }
                package.pads.push(pad_inst);
            }

            let forme = self.forme_en_ligne(pin_elem)?;
            if !forme.is_empty()
                || self.find(pin_elem, "StandardPrimitiveRef").is_some()
                || self.find(pin_elem, "UserPrimitiveRef").is_some()
            {
                let mut pad_inst = self.process_pad_instance(pin_elem, "ALL", "", &forme);
                pad_inst.pin_ref = number.to_string();
                package.pads_broches.push(pad_inst);
            }
        }

        self.design.packages.insert(name.to_string(), Partage::new(package));
        Ok(())
    }

    /// <LogicalNet name="..." netClass="..."><PinRef componentRef="U1" pin="3"/>...
    ///
    /// C'est la seule source fiable du net d'une broche : les <Pad> internes
    /// a un <Pin> qui permettraient de le deduire autrement sont quasiment
    /// toujours absents d'un export reel (verifie sur un export du commerce
    /// de 10 Mo, aucun composant n'en porte). Sans cette lecture, la fiche
    /// d'un boitier ne peut pas repondre a la question qu'on lui pose le plus
    /// souvent -- "la broche 3, elle va ou ?".
    /// De plus, l'attribut netClass ("GROUND", "POWER", "SIGNAL", etc.) permet
    /// d'auto-détecter la nature des équipotentielles pour les simulations.
    ///
    /// Depuis la 1.79, les regles de l'outil de CAO sont lues aussi (voir
    /// `lire_spec_regle`) : netPair, les <NonstandardAttribute> NET_CLASS /
    /// DIFF_PAIR / DIFF_PAIR_POLARITY et les <SpecRef> du net, vers
    /// design.regles_nets.
    fn parse_logical_nets(&mut self, step_elem: N, racine: N) {
        // le dernier composant d'un refDes en double l'emporte
        let index: HashMap<String, usize> =
            self.design.components.iter().enumerate().map(|(i, c)| (c.ref_des.clone(), i)).collect();
        let mut elems = self.findall(step_elem, "LogicalNet");
        // Certains outils placent LogicalNet au niveau de CadData
        if let Some(cad_data) = self.find(racine, "Ecad").and_then(|e| self.find(e, "CadData")) {
            elems.extend(self.findall(cad_data, "LogicalNet"));
        }

        for net_elem in elems {
            let Some(net_name) = attr_plein(net_elem, "name") else { continue };
            let net_class = py::strip(attr_plein(net_elem, "netClass").or(attr_plein(net_elem, "net_class")).unwrap_or(""));
            if !net_class.is_empty() {
                self.design.get_or_create_net(net_name).net_class = net_class.to_string();
            }
            // -- les regles de l'outil de CAO (1.79) ------------------------
            let mut attrs: HashMap<String, &str> = HashMap::new();
            for a in self.findall(net_elem, "NonstandardAttribute") {
                let nom_a = py::strip(attr_ou(a, "name", "")).to_uppercase();
                if !nom_a.is_empty() {
                    attrs.insert(nom_a, py::strip(attr_ou(a, "value", "")));
                }
            }
            let hors_norme = |cle: &str| attrs.get(cle).copied().unwrap_or("");
            if !hors_norme("NET_CLASS").is_empty() {
                self.design.regles_nets.classes.insert(net_name.to_string(), texte(hors_norme("NET_CLASS")));
            }
            for id in self.spec_refs(net_elem) {
                self.ranger_regle(&id, Some(net_name));
            }
            // la paire : netPair est l'attribut normalise ; le nom et la
            // polarite viennent des attributs hors norme ou de la spec
            let mate = py::strip(attr_ou(net_elem, "netPair", ""));
            if !mate.is_empty() && mate != net_name {
                let paires = &mut self.design.regles_nets.paires;
                let bout = |p: &Objet, cle: &str| p.get(cle).and_then(Json::as_str).map(str::to_string);
                let rang = paires.iter().position(|p| {
                    let (a, b) = (bout(p, "p"), bout(p, "n"));
                    [net_name, mate].iter().any(|n| a.as_deref() == Some(n) || b.as_deref() == Some(n))
                });
                let rang = rang.unwrap_or_else(|| {
                    let mut paire = Objet::new();
                    paire.insert("nom".into(), texte(hors_norme("DIFF_PAIR")));
                    paire.insert("p".into(), texte(""));
                    paire.insert("n".into(), texte(""));
                    paires.push(paire);
                    paires.len() - 1
                });
                let paire = &mut paires[rang];
                let vide = |p: &Objet, cle: &str| !p.get(cle).is_some_and(Json::vrai);
                let pol = hors_norme("DIFF_PAIR_POLARITY").to_uppercase();
                if pol == "P" || pol == "N" {
                    paire.insert(pol.to_lowercase(), texte(net_name));
                    paire.insert(if pol == "P" { "n" } else { "p" }.into(), texte(mate));
                } else {
                    if vide(paire, "p") {
                        paire.insert("p".into(), texte(net_name));
                    }
                    if vide(paire, "n") && bout(paire, "p").as_deref() != Some(mate) {
                        paire.insert("n".into(), texte(mate));
                    }
                }
                if vide(paire, "nom") && !hors_norme("DIFF_PAIR").is_empty() {
                    paire.insert("nom".into(), texte(hors_norme("DIFF_PAIR")));
                }
            }
            for pin_ref in self.findall(net_elem, "PinRef") {
                let Some(&i) = index.get(attr_ou(pin_ref, "componentRef", "")) else { continue };
                let Some(pin) = attr_plein(pin_ref, "pin") else { continue };
                // Le premier net gagne : deux <LogicalNet> ne devraient jamais
                // revendiquer la meme broche, mais un fichier mal forme ne
                // doit pas faire clignoter la valeur au hasard de l'ordre.
                self.design.components[i].pin_nets.entry(pin.to_string()).or_insert_with(|| net_name.to_string());
            }
        }
    }

    fn process_component(&mut self, comp_elem: N) -> Py<()> {
        let Some(ref_des) = attr_plein(comp_elem, "refDes") else { return Ok(()) };

        let props = self.lire_proprietes(comp_elem);
        let part_attr = attr_ou(comp_elem, "part", "");

        // 1. Tolérance depuis Property ou Characteristics
        let mut tolerance = premiere_prop(
            &props,
            &["TOLERANCE", "TOL", "RESISTANCE_TOLERANCE", "CAPACITANCE_TOLERANCE", "INDUCTANCE_TOLERANCE"],
        );

        // 2. Valeur depuis attribut, Property ou Characteristics
        let mut value = attr_ou(comp_elem, "value", "").to_string();
        if value.is_empty() {
            value =
                premiere_prop(&props, &["VALUE", "VAL", "RESISTANCE", "CAPACITANCE", "INDUCTANCE", "COMMENT", "DEVICE"]);
        }

        // 3. Si toujours vide, analyse de part_attr (ex: RES-10K, 100nF, C0603_10uF)
        if value.is_empty() && !part_attr.is_empty() {
            let (v_part, t_part) = Self::extraire_val_tol(part_attr);
            if !v_part.is_empty() {
                value = v_part;
            }
            if tolerance.is_empty() && !t_part.is_empty() {
                tolerance = t_part;
            }
        }

        // 4. Si la valeur trouvée contient aussi une tolérance et qu'elle n'était pas renseignée
        if !value.is_empty() && tolerance.is_empty() {
            tolerance = Self::extraire_val_tol(&value).1;
        }

        // Part number
        let mut part_number = premiere_prop(&props, &["PART_NUMBER", "PARTNUMBER", "MPN", "PN", "PART"]);
        if part_number.is_empty() {
            part_number = part_attr.to_string();
        }

        // Qualification du type
        let comp_type = Self::qualifier_type_composant(ref_des, &value, Some(&props));

        let actual_pkg_ref: Option<String> = if let Some(inline_pkg) = self.find(comp_elem, "Package") {
            let pkg_name = attr(inline_pkg, "name");
            if !pkg_name.is_some_and(|n| self.design.packages.contains_key(n)) {
                self.process_package(inline_pkg)?;
            }
            pkg_name.map(str::to_string)
        } else {
            // C'est « packageRef » qui designe le <Package> : « part » est le
            // nom de la reference (celui de la nomenclature), qui n'a aucune
            // raison de ressembler au nom de l'empreinte -- sur un fichier
            // reel, part="CONN_8-2mm-reflow" pour packageRef="CONN_8pts-2mm_reflow".
            // Lire « part » ici laissait donc les composants sans empreinte,
            // donc sans broches : la fiche d'un boitier annonçait « 0 broche ».
            Some(
                attr_plein(comp_elem, "packageRef")
                    .unwrap_or(if part_attr.is_empty() { "Unknown" } else { part_attr })
                    .to_string(),
            )
        };

        let (rotation, mirror) = self.xform(comp_elem).unwrap_or((0.0, false));

        let package_obj = match &actual_pkg_ref {
            Some(nom) => match self.design.packages.get(nom) {
                Some(p) => Some(p.clone()),
                None => self.design.packages.iter().find(|(key, _)| key.contains(nom.as_str())).map(|(_, p)| p.clone()),
            },
            // un <Package> en ligne sans nom : Python cherchait None dans chaque nom de boîtier
            None if !self.design.packages.is_empty() => {
                return Err("'in <string>' requires string as left operand, not NoneType".into());
            }
            None => None,
        };

        self.design.components.push(Component {
            ref_des: ref_des.to_string(),
            package_ref: actual_pkg_ref.unwrap_or_default(),
            layer_ref: attr_ou(comp_elem, "layerRef", "TOP").to_string(),
            location: self.get_location(comp_elem),
            rotation,
            mirror,
            mount_type: attr_ou(comp_elem, "mountType", "UNKNOWN").to_string(),
            value,
            tolerance,
            comp_type,
            part_number,
            properties: props,
            package_obj,
            pin_nets: IndexMap::new(),
            populate: true,
            pads_repli: Vec::new(),
        });
        Ok(())
    }

    // ------------------------------------------------------------------
    // Primitives graphiques
    // ------------------------------------------------------------------

    fn process_text(&self, text_elem: N, layer: &str, default_net: &str, loc_defaut: Option<Point>) -> Option<TextElement> {
        // « textString » est l'attribut de la norme ; « text » celui
        // qu'ecrivaient les premiers exports lus ici. Les deux sont acceptes.
        let text_val = attr_plein(text_elem, "textString").or(attr_plein(text_elem, "text"))?;

        let loc = match (self.find(text_elem, "Location"), loc_defaut) {
            (None, Some(defaut)) => defaut,
            _ => self.get_location(text_elem),
        };
        let (rot, mirror) = self.xform(text_elem).unwrap_or((0.0, false));

        let mut text_obj = TextElement {
            text: text_val.to_string(),
            location: loc,
            layer_name: layer.to_string(),
            net_name: attr_ou(text_elem, "net", default_net).to_string(),
            rotation: rot,
            mirror,
            height: 0.0,
            bbox: None,
        };
        // La TAILLE, que reprend l'editeur PCB : la boite englobante de la
        // norme (<BoundingBox>), a defaut l'attribut fontSize.
        if let Some(boite) = self.find(text_elem, "BoundingBox") {
            let bb = ["lowerLeftX", "lowerLeftY", "upperRightX", "upperRightY"].map(|k| Self::safe_float(attr(boite, k)));
            if bb[2] > bb[0] && bb[3] > bb[1] {
                text_obj.bbox = Some(bb);
                text_obj.height = bb[3] - bb[1];
            }
        }
        if text_obj.height == 0.0 {
            let taille = Self::safe_float(attr(text_elem, "fontSize"));
            if taille > 0.0 {
                text_obj.height = taille;
            }
        }
        Some(text_obj)
    }

    /// lineEnd en `bout`, seulement s'il n'est pas ROUND (le cas courant).
    fn bout(&self, elem: N) -> String {
        let bout = self.get_line_end(elem);
        if bout == "ROUND" {
            String::new()
        } else {
            bout
        }
    }

    fn process_simple_line(&mut self, line_elem: N, layer: &str, default_net: &str, xf: Option<Xf>) {
        // une coordonnée manquante, illisible ou non finie : la ligne est ignorée
        let Some([x1, y1, x2, y2]) = Self::quatre(line_elem, ["startX", "startY", "endX", "endY"]) else { return };
        let (mut p1, mut p2) = (Point::new(x1, y1), Point::new(x2, y2));
        if let Some(xf) = xf {
            (p1, p2) = (xf.de(p1), xf.de(p2));
        }
        self.design.add_track(Track {
            width: self.get_width(line_elem),
            points: vec![p1, p2],
            layer_name: layer.to_string(),
            net_name: attr_ou(line_elem, "net", default_net).to_string(),
            fill_property: self.get_fill_desc(line_elem),
            bout: self.bout(line_elem),
        });
    }

    /// Les points d'un <Polygon>, d'une <Polyline> ou d'un <Cutout> ; un arc
    /// (<PolyStepCurve>) y est découpé en cordes.
    pub fn parse_polygon(&self, poly_elem: N) -> Py<Vec<Point>> {
        let mut points: Vec<Point> = Vec::new();
        for child in enfants(poly_elem) {
            let xy = || (Self::safe_float(attr(child, "x")), Self::safe_float(attr(child, "y")));
            match nom_local(child) {
                "PolyBegin" | "PolyStepSegment" => {
                    let (x, y) = xy();
                    points.push(Point::new(x, y));
                }
                "PolyStepCurve" => {
                    let (end_x, end_y) = xy();

                    if let Some(dernier) = points.last() {
                        let (start_x, start_y) = (dernier.x, dernier.y);
                        let center_x = Self::safe_float(attr(child, "centerX"));
                        let center_y = Self::safe_float(attr(child, "centerY"));
                        let cw = vrai(child, "clockwise");

                        let angle_start = (start_y - center_y).atan2(start_x - center_x);
                        let mut angle_end = (end_y - center_y).atan2(end_x - center_x);

                        // cercle complet (fin = debut) : un tour entier. Avec des
                        // angles egaux, le balayage valait 0 et le cercle devenait
                        // un point -- anti-pastilles et contours ronds disparaissaient.
                        let tour = py::hypot(end_x - start_x, end_y - start_y) < 1e-9;
                        if cw {
                            if angle_end > angle_start || tour {
                                angle_end -= 2.0 * PI;
                            }
                        } else if angle_end < angle_start || tour {
                            angle_end += 2.0 * PI;
                        }

                        let radius = py::hypot(start_x - center_x, start_y - center_y);

                        let sweep_angle = (angle_end - angle_start).abs();
                        let arc_length = radius * sweep_angle;
                        // une corde par 0,05 mm, dans l'unite du fichier
                        let pas = 0.05 / self.mm_du_fichier();
                        let steps = py_max(12.0, entier(arc_length / pas)?).min(360.0) as usize;

                        for i in 1..steps {
                            let angle = angle_start + (angle_end - angle_start) * (i as f64 / steps as f64);
                            points.push(Point::new(center_x + radius * angle.cos(), center_y + radius * angle.sin()));
                        }
                    }

                    points.push(Point::new(end_x, end_y));
                }
                _ => {}
            }
        }
        Ok(points)
    }

    /// Une <Polyline>, ou le <Polygon> d'un <Outline> (trait_elem : le
    /// <Outline>, qui porte la largeur du trait).
    fn process_polyline(
        &mut self, poly_elem: N, layer: &str, default_net: &str, xf: Option<Xf>, trait_elem: Option<N>,
    ) -> Py<()> {
        let trait_elem = trait_elem.unwrap_or(poly_elem);
        let width = self.get_width(trait_elem);
        let fill_prop = self.get_fill_desc(poly_elem);
        let mut points = self.parse_polygon(poly_elem)?;
        if let Some(xf) = xf {
            points = points.into_iter().map(|p| xf.de(p)).collect();
        }
        if points.len() >= 2 {
            self.design.add_track(Track {
                width,
                points,
                layer_name: layer.to_string(),
                net_name: attr_ou(poly_elem, "net", default_net).to_string(),
                fill_property: fill_prop,
                bout: self.bout(trait_elem),
            });
        }
        Ok(())
    }

    fn process_arc(&mut self, arc_elem: N, layer: &str, default_net: &str, xf: Option<Xf>, miroir: bool) {
        let Some((mut start, mut end, mut center)) = Self::trois_points(arc_elem) else { return };
        let mut clockwise = vrai(arc_elem, "clockwise");
        if let Some(xf) = xf {
            (start, end, center) = (xf.de(start), xf.de(end), xf.de(center));
            clockwise ^= miroir; // le miroir inverse le sens de parcours
        }
        self.design.add_arc(Arc {
            width: self.get_width(arc_elem),
            start,
            end,
            center,
            clockwise,
            layer_name: layer.to_string(),
            net_name: attr_ou(arc_elem, "net", default_net).to_string(),
            fill_property: self.get_fill_desc(arc_elem),
            bout: self.bout(arc_elem),
        });
    }

    fn process_contour(&mut self, contour_elem: N, layer: &str, default_net: &str, xf: Option<Xf>) -> Py<()> {
        let Some(polygon) = self.find(contour_elem, "Polygon") else { return Ok(()) };
        let place = |pts: Vec<Point>| -> Vec<Point> {
            match xf {
                Some(xf) => pts.into_iter().map(|p| xf.de(p)).collect(),
                None => pts,
            }
        };

        let outline_points = place(self.parse_polygon(polygon)?);
        if outline_points.len() < 3 {
            return Ok(());
        }

        let mut contour_data = Contour { outline: outline_points, cutouts: Vec::new() };
        let fill_prop = self.get_fill_desc(polygon);

        for cutout in self.findall(contour_elem, "Cutout") {
            let cutout_points = place(self.parse_polygon(cutout)?);
            if cutout_points.len() >= 3 {
                contour_data.cutouts.push(cutout_points);
            }
        }

        self.design.add_copper_plane(CopperPlane {
            layer_name: layer.to_string(),
            net_name: attr_ou(contour_elem, "net", default_net).to_string(),
            contours: vec![contour_data],
            fill_property: fill_prop,
        });
        Ok(())
    }
}

/// a % b de Python pour des flottants : le signe du diviseur.
fn modulo(a: f64, b: f64) -> f64 {
    let r = a % b;
    if r != 0.0 && (r < 0.0) != (b < 0.0) {
        r + b
    } else if r == 0.0 {
        0.0f64.copysign(b)
    } else {
        r
    }
}
