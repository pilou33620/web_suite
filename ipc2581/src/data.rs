//! Les structures de données du lecteur IPC-2581 (ipc2581_data.py), avec en champs ce que le
//! lecteur Python posait en attributs après coup (`bout`, `height`, `contre_percage`,
//! `rugosite_um`, `layer_info`, `regles_nets`...).

use crate::py::{self, Json, Objet};
use indexmap::IndexMap;
use std::sync::Arc as Partage; // `Arc` est ici un arc de cercle

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
}

/// La rugosité de Huray d'un cuivre : rayon des nodules en micromètres, rapport de surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Huray {
    pub a: f64,
    pub sr: f64,
}

/// Représente une couche physique dans l'empilage (Stackup).
#[derive(Debug, Clone, PartialEq)]
pub struct StackupLayer {
    pub name: String,
    pub thickness: f64,
    /// Le rang dans l'empilage : un entier, tenu dans un flottant parce que celui de Python
    /// (int(float("1e300"))) n'a pas de borne.
    pub sequence: f64,
    pub material: String,
    pub dk: String,
    pub df: String,
    /// "Signal", "GND", "Power", or "Unknown"
    pub layer_type: String,
    /// La rugosité du cuivre en micromètres (Rq), 0 si le fichier ne la porte pas.
    pub rugosite_um: f64,
    /// 0 écrit : un cuivre DÉCLARÉ lisse, pas une rugosité absente.
    pub rugosite_declaree: bool,
    pub rugosite_huray: Option<Huray>,
}

impl StackupLayer {
    pub fn new(name: &str, thickness: f64, sequence: f64) -> StackupLayer {
        StackupLayer {
            name: name.to_string(),
            thickness,
            sequence,
            material: String::new(),
            dk: String::new(),
            df: String::new(),
            layer_type: "Unknown".into(),
            rugosite_um: 0.0,
            rugosite_declaree: false,
            rugosite_huray: None,
        }
    }
}

/// Définit une primitive standard (cercle, rectangle, ovale, polygone).
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeDefinition {
    pub id: String,
    /// 'CIRCLE', 'RECTCENTER', 'OVAL', 'POLYGON', 'RECTROUND', 'RECTCHAM', 'UNKNOWN'
    pub shape_type: String,
    pub diameter: f64,
    pub width: f64,
    pub height: f64,
    pub radius: f64,
    pub chamfer: f64,
    /// Coins affectés ('upperRight', etc.)
    pub corners: Vec<String>,
    pub points: Vec<Point>,
    pub fill_property: String,
}

impl ShapeDefinition {
    pub fn new(id: &str, shape_type: &str) -> ShapeDefinition {
        ShapeDefinition {
            id: id.to_string(),
            shape_type: shape_type.to_string(),
            diameter: 0.0,
            width: 0.0,
            height: 0.0,
            radius: 0.0,
            chamfer: 0.0,
            corners: Vec::new(),
            points: Vec::new(),
            fill_property: "UNKNOWN".into(),
        }
    }

    /// Plus grande dimension hors-tout de la forme (diamètre équivalent).
    pub fn max_dimension(&self) -> f64 {
        match self.shape_type.as_str() {
            "CIRCLE" => self.diameter,
            "RECTCENTER" | "RECTROUND" | "OVAL" | "RECTCHAM" => py_max(self.width, self.height),
            "POLYGON" if !self.points.is_empty() => etendue(&self.points),
            _ => 0.0,
        }
    }
}

/// max(a, b) de Python : le premier, sauf si le second est strictement plus grand.
pub(crate) fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// min(a, b) de Python : le premier, sauf si le second est strictement plus petit.
pub(crate) fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// max(largeur, hauteur) de la boîte d'une suite de points non vide.
pub(crate) fn etendue(pts: &[Point]) -> f64 {
    let p = pts[0];
    let (mut x0, mut x1, mut y0, mut y1) = (p.x, p.x, p.y, p.y);
    for p in &pts[1..] {
        x0 = py_min(x0, p.x);
        x1 = py_max(x1, p.x);
        y0 = py_min(y0, p.y);
        y1 = py_max(y1, p.y);
    }
    py_max(x1 - x0, y1 - y0)
}

/// Définit un polygone avec de potentiels trous (cutouts).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Contour {
    pub outline: Vec<Point>,
    pub cutouts: Vec<Vec<Point>>,
}

/// Représente une zone de cuivre remplie sur le PCB.
#[derive(Debug, Clone, PartialEq)]
pub struct CopperPlane {
    pub layer_name: String,
    pub net_name: String,
    pub contours: Vec<Contour>,
    pub fill_property: String,
}

/// Définit la géométrie d'un pad sur une couche spécifique.
#[derive(Debug, Clone, PartialEq)]
pub struct PadDef {
    pub layer_ref: String,
    pub pad_diameter: f64,
    pub shape_ref: String,
    pub antipad_diameter: f64,
}

impl PadDef {
    pub fn new(layer_ref: &str) -> PadDef {
        PadDef { layer_ref: layer_ref.to_string(), pad_diameter: 0.0, shape_ref: String::new(), antipad_diameter: 0.0 }
    }
}

/// Définit les propriétés physiques d'un via/pad multicouches.
#[derive(Debug, Clone, PartialEq)]
pub struct PadStackDefinition {
    pub name: String,
    /// Conservé pour rétro-compatibilité (diamètre max ou par défaut)
    pub pad_diameter: f64,
    /// Diamètre défini dans le stack (par défaut)
    pub hole_diameter: f64,
    /// Conservé pour rétro-compatibilité
    pub shape_ref: String,
    /// Définitions par couche
    pub pads: Vec<PadDef>,
    // LA PASTILLE A-T-ELLE ÉTÉ LUE, OU DEVINÉE ? Quand un perçage de via ne
    // renvoie à aucune définition de padstack, le lecteur en fabrique une avec
    // « perçage + 0,3 mm » — soit un anneau de 0,15 mm posé par convention.
    // C'est un repli raisonnable pour dessiner quelque chose ; ce n'est PAS une
    // cote du fichier, et tout ce qui s'en sert doit pouvoir le savoir : la
    // simulation la fait entrer dans la capacité du via, le contrôle
    // d'isolation mesure des distances contre elle.
    pub pad_supposee: bool,
    // LE CALQUE DE PERCAGE QUE CE PADSTACK DESIGNE. C'est <LayerHole
    // layerRef="..."> : le padstack ne dit pas lui-meme entre quelles couches
    // son trou court, il RENVOIE au calque DRILL, et c'est ce calque qui porte
    // le <Span>. Sans ce chainon, la portee d'un via se perd des que le trou est
    // decouvert par le padstack plutot que par le calque.
    pub hole_layer_ref: String,
}

impl PadStackDefinition {
    pub fn new(name: &str) -> PadStackDefinition {
        PadStackDefinition {
            name: name.to_string(),
            pad_diameter: 0.0,
            hole_diameter: 0.0,
            shape_ref: String::new(),
            pads: Vec::new(),
            pad_supposee: false,
            hole_layer_ref: String::new(),
        }
    }
}

/// Représente l'instanciation physique d'un Pad (dans un boîtier ou libre).
#[derive(Debug, Clone, PartialEq)]
pub struct PadInstance {
    pub padstack_ref: String,
    pub location: Point,
    pub rotation: f64,
    pub mirror: bool,
    pub pin_ref: String,
    pub net_name: String,
}

/// Le contre-perçage (back-drill) d'un perçage, tel que le fichier le déclare.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContrePercage {
    /// la face d'où le foret repasse (START_LAYER)
    pub depart: Option<String>,
    /// la couche à ne pas couper (MUST_NOT_CUT_LAYER)
    pub garde: Option<String>,
    /// le moignon résiduel admis, en mm (MAX_STUB_LENGTH)
    pub moignon_mm: Option<f64>,
    /// la <Spec> d'où il vient
    pub spec: Option<String>,
    /// "spec" ou "calque"
    pub source: String,
    /// le calque du foret, quand il a son propre trou
    pub calque: Option<String>,
    /// le diamètre du foret, en unités du fichier (s'il a son trou)
    pub diametre: Option<f64>,
}

/// Un foret de contre-perçage lu sur un calque à part, en attente du via qu'il repasse.
#[derive(Debug, Clone, PartialEq)]
pub struct Foret {
    pub x: f64,
    pub y: f64,
    pub diametre: f64,
    pub spec: String,
    pub calque: String,
    pub span: Option<(String, String)>,
}

/// Représente un perçage physique sur la carte.
#[derive(Debug, Clone, PartialEq)]
pub struct Drill {
    pub location: Point,
    /// Diamètre du trou
    pub diameter: f64,
    /// PLATED (PTH) ou NONPLATED (NPTH)
    pub plating: String,
    /// Référence vers la définition
    pub padstack_ref: String,
    pub net_name: String,

    // ==================================================================
    // LA PORTÉE DU PERÇAGE — ENTRE QUELLES COUCHES IL COURT
    // ------------------------------------------------------------------
    // POURQUOI ELLE COMPTE, ET DES DEUX CÔTÉS. Un trou pris pour traversant
    // alors qu'il est borgne est modélisé PLUS LONG qu'il n'est : sa
    // résistance ressort surestimée dans le solveur DC — l'erreur du côté
    // prudent — mais surtout il RELIE des couches qu'il ne relie pas, ce qui
    // invente un chemin de courant. Et pour le contrôle du chemin de retour,
    // l'erreur penche du côté qui rassure : un via enterré compté comme
    // traversant fait passer pour refermée une boucle qui ne l'est pas.
    //
    // OÙ ELLE SE LIT. IPC-2581 ne la met pas sur le trou : elle est sur le
    // CALQUE de perçage, `<Layer layerFunction="DRILL"><Span fromLayer="..."
    // toLayer="..."/></Layer>`. Le perçage y renvoie par son `layerRef`, le
    // padstack par le `layerRef` de son `<LayerHole>`.
    //
    // VIDE VEUT DIRE « LE FICHIER NE LE DIT PAS », et non « traversant » :
    // tout ce qui s'en sert doit alors le supposer ET LE DIRE. Les deux
    // cas ne se confondent pas — `span_source` les distingue.
    // ==================================================================
    /// nom de la couche de départ, tel que déclaré
    pub span_from: String,
    /// nom de la couche d'arrivée
    pub span_to: String,
    /// "calque", "padstack", ou "" (non déclarée)
    pub span_source: String,

    /// Référence vers l'objet définition pour calculs (anneaux)
    pub padstack_obj: Option<Partage<PadStackDefinition>>,

    // UNE FENTE (<SlotCavity>) : `diameter` est sa largeur, `slot_length` sa
    // longueur hors tout et `slot_rotation` l'angle de la longueur, en degrés
    // dans le repère du fichier. 0 : un trou rond.
    pub slot_length: f64,
    pub slot_rotation: f64,

    pub contre_percage: Option<ContrePercage>,
}

impl Drill {
    pub fn new(location: Point, diameter: f64, plating: &str, padstack_ref: &str, net_name: &str) -> Drill {
        Drill {
            location,
            diameter,
            plating: plating.to_string(),
            padstack_ref: padstack_ref.to_string(),
            net_name: net_name.to_string(),
            span_from: String::new(),
            span_to: String::new(),
            span_source: String::new(),
            padstack_obj: None,
            slot_length: 0.0,
            slot_rotation: 0.0,
            contre_percage: None,
        }
    }

    /// La portée vient-elle du fichier ?
    pub fn span_declaree(&self) -> bool {
        !self.span_from.is_empty() && !self.span_to.is_empty() && !self.span_source.is_empty()
    }

    /// Calcule la taille de l'anneau : (Diamètre Pad Max - Diamètre Trou) / 2
    pub fn annular_ring(&self) -> f64 {
        match &self.padstack_obj {
            Some(p) if p.pad_diameter > self.diameter => (p.pad_diameter - self.diameter) / 2.0,
            _ => 0.0,
        }
    }

    /// L'anneau vient-il du fichier, ou de la pastille devinée ?
    ///
    /// UN ANNEAU DE 0,15 mm QUI SORT TOUJOURS A 0,15 mm N'EST PAS UNE MESURE.
    /// C'est « perçage + 0,3 » divise par deux, c'est-a-dire la convention du
    /// lecteur relue a l'envers. Affiche comme une cote, il donne a croire que
    /// le fabricant a declare cet anneau-la.
    pub fn annular_ring_supposee(&self) -> bool {
        self.padstack_obj.as_ref().is_some_and(|p| p.pad_supposee)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pin {
    pub number: String,
    pub x: f64,
    pub y: f64,
    pub r#type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub name: String,
    pub pins: Vec<Pin>,
    pub pads: Vec<PadInstance>,
    /// <Package height>, None si absente
    pub hauteur: Option<f64>,
    /// le polygone de <Package><Outline>, dans les coordonnées du boîtier
    pub corps: Vec<Point>,
    /// La forme écrite sur la broche elle-même : elle ne sert qu'en repli
    /// (voir `pads_repli` du composant).
    pub pads_broches: Vec<PadInstance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub ref_des: String,
    pub package_ref: String,
    pub layer_ref: String,
    pub location: Point,
    pub rotation: f64,
    pub mirror: bool,
    pub mount_type: String,
    /// Valeur du composant (ex: 10k, 100nF)
    pub value: String,
    /// Tolérance du composant (ex: +-0.1pF, 5%)
    pub tolerance: String,
    /// Type de composant (RESISTOR, CAPACITOR, INDUCTOR, etc.)
    pub comp_type: String,
    /// Référence de pièce / OEMDesignNumberRef
    pub part_number: String,
    /// Propriétés personnalisées (<Property>)
    pub properties: IndexMap<String, String>,
    pub package_obj: Option<Partage<Package>>,
    // Broche -> nom de net, tel que le donnent les <LogicalNet><PinRef> du
    // fichier -- pas les <Pad> internes a un <Pin>, quasi toujours absents
    // d'un export reel. Cle : le numero de broche (Pin.number), tel quel.
    pub pin_nets: IndexMap<String, String>,
    /// <BomItem><RefDes populate="false"> : la variante ne monte pas ce composant
    pub populate: bool,
    /// La forme des broches du boîtier, pour le composant dont aucune pastille n'est posée
    /// dans les calques.
    pub pads_repli: Vec<PadInstance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub width: f64,
    pub points: Vec<Point>,
    pub layer_name: String,
    pub net_name: String,
    pub fill_property: String,
    /// lineEnd quand il n'est pas ROUND (SQUARE ou NONE) ; vide sinon
    pub bout: String,
}

impl Track {
    /// Longueur totale du tracé.
    pub fn length(&self) -> f64 {
        let mut total = 0.0;
        for p in self.points.windows(2) {
            total += py::hypot(p[1].x - p[0].x, p[1].y - p[0].y);
        }
        total
    }
}

/// Représente un arc de cercle.
#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    pub width: f64,
    pub start: Point,
    pub end: Point,
    pub center: Point,
    pub clockwise: bool,
    pub layer_name: String,
    pub net_name: String,
    pub fill_property: String,
    /// lineEnd quand il n'est pas ROUND ; vide sinon
    pub bout: String,
}

/// Représente un texte graphique (référence, valeur, sérigraphie libre).
#[derive(Debug, Clone, PartialEq)]
pub struct TextElement {
    pub text: String,
    pub location: Point,
    pub layer_name: String,
    pub net_name: String,
    pub rotation: f64,
    pub mirror: bool,
    /// hauteur du texte (<BoundingBox>, à défaut fontSize) ; 0 si le fichier ne la donne pas
    pub height: f64,
    /// la boîte englobante [x1, y1, x2, y2] de la norme
    pub bbox: Option<[f64; 4]>,
}

/// Représente une forme personnalisée (empreinte ou polygone complexe) du dictionnaire utilisateur.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UserShape {
    pub id: String,
    pub tracks: Vec<Track>,
    pub arcs: Vec<Arc>,
    pub planes: Vec<CopperPlane>,
    pub texts: Vec<TextElement>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Net {
    pub name: String,
    pub net_class: String,
    pub tracks: Vec<Track>,
    pub arcs: Vec<Arc>,
    pub copper_planes: Vec<CopperPlane>,
}

impl Net {
    /// La somme des longueurs de ses pistes ; None pour un net sans piste (le 0 entier de
    /// Python, que `sum` ne compte pas comme un flottant).
    pub(crate) fn longueur_pistes(&self) -> Option<f64> {
        if self.tracks.is_empty() {
            None
        } else {
            Some(py::sum(self.tracks.iter().map(Track::length)))
        }
    }

    pub fn total_track_length(&self) -> f64 {
        self.longueur_pistes().unwrap_or(0.0)
    }
}

/// Fonction (SILKSCREEN, SIGNAL...) et face (TOP, BOTTOM) d'un calque déclaré.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerInfo {
    pub fonction: String,
    pub face: String,
}

/// Les règles de l'outil de CAO qui a écrit le fichier (voir `_lire_spec_regle` du lecteur),
/// en dictionnaires tels qu'ils partent dans le JSON, longueurs en millimètres.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReglesNets {
    /// {net: nom de classe}
    pub classes: Objet,
    /// {classe: {w, clr, via, drill, wL, z, zTol...}}
    pub regles_classes: Objet,
    /// [{nom, p, n, spec, ...}]
    pub paires: Vec<Objet>,
    /// {net: {z, zTol, lMin, lMax, viasMax...}}
    pub contraintes: Objet,
    /// [{spec, nom, mode, tol, ref, nets}]
    pub groupes: Vec<Objet>,
    /// {"A|B": isolement en mm}
    pub matrice: Objet,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IPCDesign {
    pub units: String,
    pub total_thickness: f64,
    pub board_outline: Option<Contour>,
    pub stackup: Vec<StackupLayer>,
    pub nets: IndexMap<String, Net>,
    pub packages: IndexMap<String, Partage<Package>>,
    pub components: Vec<Component>,

    pub padstacks: IndexMap<String, Partage<PadStackDefinition>>,
    pub standalone_pads: Vec<PadInstance>,
    pub drills: Vec<Drill>,
    pub shapes: IndexMap<String, ShapeDefinition>,
    pub user_shapes: IndexMap<String, UserShape>,
    pub texts: Vec<TextElement>,
    // Calques du fichier volontairement laisses de cote (serigraphie, masque,
    // cotation, keepouts...) : nom -> layerFunction. Voir le filtre de
    // IPC2581Parser::role_calque.
    pub ignored_layers: IndexMap<String, String>,
    /// Fonction et face de CHAQUE calque déclaré, empilage ou non.
    pub layer_info: IndexMap<String, LayerInfo>,
    pub regles_nets: ReglesNets,
    /// Les forets de contre-perçage sans via au même emplacement.
    pub contre_percages_orphelins: Vec<Foret>,
}

impl Default for IPCDesign {
    fn default() -> IPCDesign {
        IPCDesign {
            units: "MILLIMETER".into(),
            total_thickness: 0.0,
            board_outline: None,
            stackup: Vec::new(),
            nets: IndexMap::new(),
            packages: IndexMap::new(),
            components: Vec::new(),
            padstacks: IndexMap::new(),
            standalone_pads: Vec::new(),
            drills: Vec::new(),
            shapes: IndexMap::new(),
            user_shapes: IndexMap::new(),
            texts: Vec::new(),
            ignored_layers: IndexMap::new(),
            layer_info: IndexMap::new(),
            regles_nets: ReglesNets::default(),
            contre_percages_orphelins: Vec::new(),
        }
    }
}

impl IPCDesign {
    pub fn get_or_create_net(&mut self, net_name: &str) -> &mut Net {
        self.nets
            .entry(net_name.to_string())
            .or_insert_with(|| Net { name: net_name.to_string(), ..Net::default() })
    }

    /// Une piste sans net est perdue (le lecteur Python le signalait au journal).
    pub fn add_track(&mut self, track: Track) {
        if !track.net_name.is_empty() {
            let nom = track.net_name.clone();
            self.get_or_create_net(&nom).tracks.push(track);
        }
    }

    pub fn add_arc(&mut self, arc: Arc) {
        if !arc.net_name.is_empty() {
            let nom = arc.net_name.clone();
            self.get_or_create_net(&nom).arcs.push(arc);
        }
    }

    pub fn add_copper_plane(&mut self, plane: CopperPlane) {
        if !plane.net_name.is_empty() {
            let nom = plane.net_name.clone();
            self.get_or_create_net(&nom).copper_planes.push(plane);
        }
    }
}

/// Une valeur de règle telle qu'elle part dans le JSON.
pub(crate) fn texte(t: &str) -> Json {
    Json::Texte(t.to_string())
}
