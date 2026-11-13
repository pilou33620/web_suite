//! Pont entre le lecteur IPC-2581 (parser.rs, qui rend un IPCDesign) et la visionneuse du
//! navigateur (qui ne sait lire que du JSON) : ipc2581_json.py, version 1.5.0. Ce module ne
//! parse rien et ne dessine rien : il traduit, et c'est tout ce qu'il doit faire. Deux partis
//! pris commandent la forme du dictionnaire produit :
//!   1. les noms qui se repetent -- couche, net -- deviennent des index dans
//!      deux tableaux places en tete. Sur une carte de quelques milliers de
//!      pistes, ecrire « TOP » et « GND » a chaque ligne triple le poids du
//!      fichier pour rien.
//!   2. les suites de points deviennent des tableaux plats [x1,y1,x2,y2,...].
//!      Un objet {"x":..,"y":..} par sommet, sur un plan de masse decoupe, c'est
//!      dix fois le poids des nombres qu'il porte.
//!   Les cles sont courtes pour la meme raison ; la visionneuse reprend le tableau de
//!   correspondance a l'identique.
//!
//! Le fichier accepte tel quel : XML IPC-2581 (.xml, .cvg) ou archive ZIP en
//! contenant un -- les fabricants livrent souvent la seconde forme.
//!
//! Cle par cle, le dictionnaire produit :
//!
//!     unites      MILLIMETER ou INCH, tel que le declare le fichier
//!     epaisseur   epaisseur hors-tout de l'empilage
//!     contour     profil de la carte : {"o": [x,y,...], "t": [[x,y,...], ...]}
//!                 (o = outline, t = trous / decoupes)
//!     empilage    couches physiques, dans l'ordre de sequence ; « rug » la rugosite
//!                 du cuivre en micrometres (Rq, Hammerstad), ou « rug_huray »
//!                 {a (rayon des nodules, micrometres), sr} pour le modele de
//!                 Huray, quand le fichier la porte ; « rug »: 0, un cuivre que
//!                 le fichier declare lisse (absent : il ne dit rien)
//!     couches     noms de couches ; ailleurs, "c" est un index dans ce tableau
//!     nets        noms de nets   ; ailleurs, "n" est un index dans ce tableau
//!     classes_nets  {net: netClass} -- l'enumeration de la norme (GROUND, POWER,
//!                 SIGNAL, CLK) portee par <LogicalNet>
//!     regles      les regles de l'outil de CAO qui a ecrit le fichier, par NOM de net :
//!                   classes          {net: nom de classe}
//!                   regles_classes   {classe: {w, clr, via, drill, wL {couche: w},
//!                                    z, zTol, lMin, lMax, viasMax, couches,
//!                                    topo, ordre, stubMax, viaStubMax,
//!                                    etoileTol, cp}} -- millimetres, ohms, %
//!                   paires           [{nom, p, n, regle, w, wMin, wMax, gap,
//!                                    gapMin, gapMax, maxDecouple, zdiff, zTol}]
//!                   contraintes      {net: {z, zTol, lMin, lMax, viasMax, ...}}
//!                   groupes          [{nom, mode "mm"|"ps", tol, ref, nets}]
//!                   matrice          {"A|B": isolement en mm}
//!                 Vide quand le fichier n'en porte pas ; la visionneuse s'en
//!                 sert comme classement initial, devant la detection.
//!     pistes      {c, n, w (largeur), p (points a plat), f (remplissage),
//!                  b (bout : SQUARE ou NONE ; absent, ROUND)}
//!     arcs        {c, n, w, s (debut), e (fin), m (centre), h (horaire), b}
//!     plans       {c, n, f, g: [{o, t}]} -- zones de cuivre remplies
//!     textes      {c, x, y, r (rotation), m (miroir), t, h (hauteur),
//!                  b (boite [x1,y1,x2,y2])}
//!     percages    {x, y, d (diametre), p (metallise), ps (padstack), n, a (anneau),
//!                  a_sup (l'anneau vient d'une pastille devinee, pas du fichier),
//!                  sa / sb (portee : rangs des couches de depart et d'arrivee,
//!                  ABSENTS quand le fichier ne la declare pas), ss (d'ou elle
//!                  vient : "calque" ou "padstack"),
//!                  cp (contre-percage, voir `contre_percage`)}
//!     pads        pastilles libres : {x, y, r, m, ps, pin, n}
//!     composants  {ref, pkg, c, x, y, r, m, mnt, val, tol, pads, pins,
//!                  hauteur?, corps? [x0, y0, x1, y1...] (coordonnees du boitier)}
//!     padstacks   definitions : {trou, pad, pads: [{c, d, f (forme), a (antipad)}]}
//!     formes      primitives standard : cercle, rectangle, ovale, polygone...
//!     formesuser  formes du dictionnaire utilisateur (empreintes complexes)
//!     calques     {nom: {f (layerFunction), s (side)}} de chaque calque declare
//!     stats       comptages, pour l'entete de la visionneuse (dont
//!                 contre_percages et contre_percages_orphelins)

use crate::data::{Contour, Drill, IPCDesign, PadInstance, Point, ShapeDefinition, StackupLayer};
use crate::parser::{fonction_cuivre, IPC2581ParseError, IPC2581Parser};
use crate::py::{self, Json, Objet};
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use std::io::Read;

pub const FORMAT: &str = "cao-ipc2581-1";

// Assez pour que le micron soit exact en millimetres comme en pouces, sans
// trainer les 17 chiffres d'un float derriere chaque sommet de polygone.
pub const DECIMALES: usize = 6;

// Extensions rencontrees dans une archive de fabrication : IPC-2581 sort en
// .xml, parfois en .cvg (« CAD Viewable Geometry ») ou en .ipc.
pub const EXTENSIONS: [&str; 4] = [".xml", ".cvg", ".ipc", ".ipc2581"];

/// Ce que le pont Python levait.
#[derive(Debug, Clone, PartialEq)]
pub enum Erreur {
    /// IPC2581ParseError : le fichier est refusé, et le message est pour qui l'a envoyé.
    Parse(IPC2581ParseError),
    /// Toute autre exception de Python, « Type: message » (l'arrondi d'un infini né d'un
    /// débordement, une archive chiffrée...) : le serveur Python en faisait une erreur interne.
    Interne(String),
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Erreur::Parse(e) => e.fmt(f),
            Erreur::Interne(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for Erreur {}

impl From<IPC2581ParseError> for Erreur {
    fn from(e: IPC2581ParseError) -> Erreur {
        Erreur::Parse(e)
    }
}

fn refus(message: String) -> Erreur {
    Erreur::Parse(IPC2581ParseError(message))
}

thread_local! {
    // `_r` sur un non-fini levait en Python, et toute la traduction tombait : la première
    // de ces exceptions est gardée ici plutôt que de rendre chaque arrondi faillible.
    static NON_FINI: Cell<Option<&'static str>> = const { Cell::new(None) };
}

/// Un flottant de valeur entière, écrit en entier quelle que soit sa taille.
fn entier(f: f64) -> Json {
    if f.abs() < 9.0e18 {
        Json::Entier(f as i64)
    } else {
        Json::GrandEntier(format!("{:.0}", f))
    }
}

/// Arrondit un flottant, et rend un int quand la valeur est ronde.
fn r(v: f64) -> Json {
    let f = py::round(v, DECIMALES);
    if !f.is_finite() {
        let exception = if f.is_nan() {
            "ValueError: cannot convert float NaN to integer"
        } else {
            "OverflowError: cannot convert float infinity to integer"
        };
        NON_FINI.with(|c| c.set(c.get().or(Some(exception))));
        return Json::Entier(0);
    }
    if f != f.trunc() {
        Json::Flottant(f)
    } else {
        entier(f)
    }
}

/// Suite de Point -> tableau plat [x1, y1, x2, y2, ...].
fn pts(points: &[Point]) -> Json {
    let mut plat = Vec::with_capacity(points.len() * 2);
    for p in points {
        plat.push(r(p.x));
        plat.push(r(p.y));
    }
    Json::Liste(plat)
}

fn xy(p: Point) -> Json {
    Json::Liste(vec![r(p.x), r(p.y)])
}

fn objet<const N: usize>(paires: [(&str, Json); N]) -> Objet {
    paires.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

/// Contour -> {"o": [...], "t": [[...], ...]}, ou None.
fn contour(c: Option<&Contour>) -> Option<Json> {
    let c = c.filter(|c| !c.outline.is_empty())?;
    let mut out = objet([("o", pts(&c.outline))]);
    let trous: Vec<Json> = c.cutouts.iter().filter(|t| !t.is_empty()).map(|t| pts(t)).collect();
    if !trous.is_empty() {
        out.insert("t".into(), Json::Liste(trous));
    }
    Some(Json::Objet(out))
}

/// Tableau de noms sans doublon + acces O(1) au rang d'un nom.
///
/// Le rang, c'est ce qui part dans le JSON a la place du nom : une carte de
/// dix mille pistes porte dix mille fois le nom de sa couche, sinon.
#[derive(Default)]
struct Index {
    noms: Vec<String>,
    rangs: HashMap<String, usize>,
}

impl Index {
    /// Rang du nom (il est ajoute s'il est inconnu). Vide -> -1.
    fn rang(&mut self, nom: &str) -> i64 {
        let nom = py::strip(nom);
        if nom.is_empty() {
            return -1;
        }
        if let Some(i) = self.rangs.get(nom) {
            return *i as i64;
        }
        self.rangs.insert(nom.to_string(), self.noms.len());
        self.noms.push(nom.to_string());
        self.noms.len() as i64 - 1
    }

    /// Rang du nom, ou -1 -- SANS L'AJOUTER.
    ///
    /// POURQUOI LES DEUX EXISTENT. `rang` sert a enregistrer ce qu'on a vu :
    /// une couche de serigraphie rencontree dans les features doit entrer dans
    /// le tableau. Mais une REFERENCE vers une couche -- la portee d'un
    /// percage, par exemple -- ne cree pas de couche : si « TOP » n'est pas
    /// dans l'empilage, c'est que le fichier nomme ses couches autrement, et
    /// l'ajouter fabriquerait un calque fantome que la visionneuse afficherait
    /// vide. Rendre -1 laisse l'appelant dire « portee non resolue », ce qui
    /// est la verite.
    fn rang_connu(&self, nom: &str) -> i64 {
        self.rangs.get(py::strip(nom)).map_or(-1, |i| *i as i64)
    }

    fn en_json(&self) -> Json {
        Json::Liste(self.noms.iter().map(|n| Json::Texte(n.clone())).collect())
    }
}

/// ShapeDefinition -> dict, sans les champs a zero.
fn forme_en_dict(forme: &ShapeDefinition) -> Json {
    let mut out = objet([("t", forme.shape_type.as_str().into())]);
    for (cle, v) in [("d", forme.diameter), ("w", forme.width), ("h", forme.height), ("r", forme.radius), ("ch", forme.chamfer)]
    {
        if v != 0.0 {
            out.insert(cle.into(), r(v));
        }
    }
    if !forme.corners.is_empty() {
        out.insert("co".into(), Json::Liste(forme.corners.iter().map(|c| c.as_str().into()).collect()));
    }
    if !forme.points.is_empty() {
        out.insert("p".into(), pts(&forme.points));
    }
    if !forme.fill_property.is_empty() && forme.fill_property != "UNKNOWN" {
        out.insert("f".into(), forme.fill_property.as_str().into());
    }
    Json::Objet(out)
}

/// PadInstance -> dict. La couche n'y est pas : c'est le padstack qui la
/// porte, une pastille traversante existant sur plusieurs couches a la fois.
fn pad_en_dict(pad: &PadInstance, nets: &mut Index) -> Json {
    let mut out = objet([("x", r(pad.location.x)), ("y", r(pad.location.y)), ("ps", pad.padstack_ref.as_str().into())]);
    if pad.rotation != 0.0 {
        out.insert("r".into(), r(pad.rotation));
    }
    if pad.mirror {
        out.insert("m".into(), 1i64.into());
    }
    if !pad.pin_ref.is_empty() {
        out.insert("pin".into(), pad.pin_ref.as_str().into());
    }
    let rang = nets.rang(&pad.net_name);
    if rang >= 0 {
        out.insert("n".into(), rang.into());
    }
    Json::Objet(out)
}

/// Le moteur de motifs de la schématique (pattern_recognition.analyser_motifs_schema), que
/// l'appelant fournit s'il l'a : il reçoit {"components": {repère: {val, type}}, "nets":
/// {net: [{ref, pin}]}} et rend son résultat, ou None s'il échoue.
pub type Motifs<'a> = &'a dyn Fn(&Json) -> Option<Json>;

/// Nature de chaque net (Masse, Alimentation, Horloge, Rapide, RF,
/// Analogique) par le meme moteur que la schematique : noms de nets, motifs
/// (quartz, regulateurs...) et composants relies. Un echec n'empeche pas
/// l'import : la visionneuse retombe alors sur « Lent ». Sans moteur (WEB_ANTENNA
/// n'en a pas), le dictionnaire est vide.
fn natures_nets(composants: &[Json], noms_nets: &[String], motifs: Option<Motifs>) -> Json {
    let vide = Json::Objet(Objet::new());
    let Some(motifs) = motifs else { return vide };
    let champ = |c: &Json, cle: &str| c.get(cle).cloned().unwrap_or(Json::Nul);
    let mut comps = Objet::new();
    let mut nets: Objet = noms_nets.iter().filter(|n| !n.is_empty()).map(|n| (n.clone(), Json::Liste(Vec::new()))).collect();
    for c in composants {
        let nom = c.get("ref").and_then(Json::as_str).unwrap_or("");
        comps.insert(nom.to_string(), Json::Objet(objet([("val", champ(c, "val")), ("type", champ(c, "type"))])));
        for liste in ["pads", "pins"] {
            for p in c.get(liste).and_then(Json::as_liste).into_iter().flatten() {
                let Some(net) = p.get("n").and_then(Json::as_i64).and_then(|n| noms_nets.get(usize::try_from(n).ok()?)) else {
                    continue;
                };
                let pin = ["pin", "num"].iter().find_map(|k| p.get(k).filter(|v| v.vrai())).cloned().unwrap_or("".into());
                if let Some(Json::Liste(l)) = nets.get_mut(net) {
                    l.push(Json::Objet(objet([("ref", nom.into()), ("pin", pin)])));
                }
            }
        }
    }
    let entree = Json::Objet(objet([("components", Json::Objet(comps)), ("nets", Json::Objet(nets))]));
    let Some(res) = motifs(&entree) else { return vide };
    let mut bruyants: Vec<Json> = res.get("nets_bruyants").and_then(Json::as_liste).cloned().unwrap_or_default();
    bruyants.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    Json::Objet(objet([
        ("classes", champ(&res, "classes_suggerees")),
        ("raisons", champ(&res, "raisons_classes")),
        ("paires", champ(&res, "paires_diff")),
        ("bruyants", Json::Liste(bruyants)),
    ]))
}

/// Les regles de l'outil de CAO lues par le parseur (design.regles_nets),
/// telles quelles, moins le nom de la <Spec> qui a servi a les rassembler.
/// Une paire sans nom prend celui de son net P ; une paire a laquelle il
/// manque un net ne part pas.
fn regles(design: &IPCDesign) -> Json {
    let regles = &design.regles_nets;
    let sans_spec = |o: &Objet| -> Objet { o.iter().filter(|(k, _)| *k != "spec").map(|(k, v)| (k.clone(), v.clone())).collect() };
    let plein = |o: &Objet, cle: &str| o.get(cle).is_some_and(Json::vrai);
    let mut paires = Vec::new();
    for p in &regles.paires {
        if !plein(p, "p") || !plein(p, "n") {
            continue;
        }
        let mut q = sans_spec(p);
        if !plein(&q, "nom") {
            let net_p = q["p"].clone();
            q.insert("nom".into(), net_p);
        }
        paires.push(Json::Objet(q));
    }
    Json::Objet(objet([
        ("classes", Json::Objet(regles.classes.clone())),
        ("regles_classes", Json::Objet(regles.regles_classes.clone())),
        ("paires", Json::Liste(paires)),
        ("contraintes", Json::Objet(regles.contraintes.clone())),
        ("groupes", Json::Liste(regles.groupes.iter().map(|g| Json::Objet(sans_spec(g))).collect())),
        ("matrice", Json::Objet(regles.matrice.clone())),
    ]))
}

// Unites du fichier vers le millimetre : le moignon et la profondeur d'un
// contre-percage partent en mm, comme le serveur les attend.
fn mm(unites: &str) -> f64 {
    match unites {
        "MILLIMETER" | "MM" => 1.0,
        "MICRON" => 1e-3,
        "INCH" => 25.4,
        _ => 1.0,
    }
}

/// platingStatus de l'IPC-2581 (PLATED, NONPLATED, VIA) : un trou VIA est
/// metallise. Le JSON garde « VIA » (il distingue les vias des trous de
/// pastille) ; seuls les comptes et les lecteurs doivent le prendre pour
/// metallise.
fn metallise(plating: &str) -> bool {
    let p = plating.to_uppercase();
    p == "VIA" || p == "PLATED"
}

/// Le contre-percage d'un perçage, au format de la visionneuse, ou None.
///
///     cote  "dessous" | "dessus" : la face d'ou le foret repasse
///     de    rang de la couche de depart (la face) dans `couches`
///     g     rang de la couche a ne pas couper
///     res   moignon residuel admis, en mm (MAX_STUB_LENGTH)
///     d     diametre du foret, en unites du fichier (s'il a son trou)
///     prof  profondeur depuis la face, en mm, comptee dans l'empilage :
///           jusqu'a la couche gardee, moins le moignon residuel
///     spec  la <Spec> d'ou il vient ; src : "spec" ou "calque"
///
/// UNE FACE QUI N'EST PAS UNE FACE, une couche gardee hors de l'empilage ou
/// du mauvais cote : le champ part quand meme, sans `cote` ni `g` -- la fiche
/// dira ce que le fichier declare, et rien ne partira au serveur.
fn contre_percage(trou: &Drill, design: &IPCDesign, couches: &Index, cuivres: &[&str]) -> Option<Json> {
    let cp = trou.contre_percage.as_ref()?;
    let mut out = objet([("src", cp.source.as_str().into())]);
    if let Some(spec) = cp.spec.as_deref().filter(|s| !s.is_empty()) {
        out.insert("spec".into(), spec.into());
    }
    if let Some(moignon) = cp.moignon_mm {
        out.insert("res".into(), r(moignon));
    }
    if let Some(d) = cp.diametre.filter(|d| *d != 0.0) {
        out.insert("d".into(), r(d));
    }
    let depart = cp.depart.as_deref().unwrap_or("");
    let garde = cp.garde.as_deref().unwrap_or("");
    if !depart.is_empty() && couches.rang_connu(depart) >= 0 {
        out.insert("de".into(), couches.rang_connu(depart).into());
    }
    let (Some(haut), Some(bas)) = (cuivres.first(), cuivres.last()) else { return Some(Json::Objet(out)) };
    if depart != *haut && depart != *bas {
        return Some(Json::Objet(out));
    }
    let dessous = depart == *bas && cuivres.len() > 1;
    if let Some(ig) = cuivres.iter().position(|c| *c == garde) {
        // la couche gardee n'est pas la face percee
        if (dessous && ig < cuivres.len() - 1) || (!dessous && ig > 0) {
            out.insert("cote".into(), if dessous { "dessous" } else { "dessus" }.into());
            out.insert("g".into(), couches.rang_connu(garde).into());
            // LA PROFONDEUR SE COMPTE DANS L'EMPILAGE : de la face percee
            // jusqu'au cuivre garde (exclu), moins le moignon residuel. Une
            // epaisseur manquante et on ne la donne pas.
            let k = mm(&design.units.to_uppercase());
            let mut pile: Vec<&StackupLayer> = design.stackup.iter().collect();
            pile.sort_by(|a, b| a.sequence.total_cmp(&b.sequence));
            let rang = |nom: &str| pile.iter().position(|c| c.name == nom).unwrap_or(0);
            let (i0, i1, ic) = (rang(haut), rang(bas), rang(garde));
            let tranche = (if dessous { pile.get(ic + 1..i1 + 1) } else { pile.get(i0..ic) }).unwrap_or(&[]);
            if !tranche.is_empty() && tranche.iter().all(|c| c.thickness > 0.0) {
                let prof = py::sum(tranche.iter().map(|c| c.thickness)) * k - cp.moignon_mm.unwrap_or(0.0);
                if prof > 0.0 {
                    out.insert("prof".into(), r(py::round(prof, 4)));
                }
            }
        }
    }
    Some(Json::Objet(out))
}

/// os.path.basename
fn basename(chemin: &str) -> &str {
    chemin.rsplit(|c| c == '/' || (cfg!(windows) && (c == '\\' || c == ':'))).next().unwrap_or(chemin)
}

/// IPCDesign -> dictionnaire JSON pour la visionneuse (« natures_nets » vide : voir
/// `design_en_dict_avec`).
pub fn design_en_dict(design: &IPCDesign, fichier: &str) -> Result<Json, Erreur> {
    design_en_dict_avec(design, fichier, None)
}

/// IPCDesign -> dictionnaire JSON pour la visionneuse, « natures_nets » rempli par le moteur
/// de motifs de l'appelant.
pub fn design_en_dict_avec(design: &IPCDesign, fichier: &str, motifs: Option<Motifs>) -> Result<Json, Erreur> {
    NON_FINI.with(|c| c.set(None));
    let mut couches = Index::default();
    let mut nets = Index::default();

    // Les couches de l'empilage d'abord, et dans leur ordre : c'est celui de la
    // carte physique, et la visionneuse s'en sert pour l'ordre de dessin. Les
    // couches vues plus tard dans les features (serigraphie, masque, cotation)
    // viennent s'ajouter a la suite.
    let mut empilage = Vec::new();
    for couche in &design.stackup {
        couches.rang(&couche.name);
        let mut e = objet([
            ("nom", couche.name.as_str().into()),
            ("seq", entier(couche.sequence)),
            ("ep", r(couche.thickness)),
            ("mat", couche.material.as_str().into()),
            ("dk", couche.dk.as_str().into()),
            ("df", couche.df.as_str().into()),
            ("type", couche.layer_type.as_str().into()),
        ]);
        // La rugosite du cuivre, en MICROMETRES, quand le fichier la porte :
        // absente, la cle n'est pas ecrite.
        if couche.rugosite_um > 0.0 {
            e.insert("rug".into(), py::round(couche.rugosite_um, 4).into());
        } else if couche.rugosite_declaree {
            // declaree lisse : 0 ecrit, pas une absence
            e.insert("rug".into(), 0i64.into());
        }
        // Huray : rayon des nodules en micrometres et rapport
        // de surface ; a part de « rug », qui reste un Rq
        if let Some(h) = couche.rugosite_huray.filter(|h| h.a != 0.0 && h.sr != 0.0) {
            e.insert(
                "rug_huray".into(),
                Json::Objet(objet([("a", py::round(h.a, 4).into()), ("sr", py::round(h.sr, 4).into())])),
            );
        }
        empilage.push(Json::Objet(e));
    }

    let (mut pistes, mut arcs, mut plans) = (Vec::new(), Vec::new(), Vec::new());
    for net in design.nets.values() {
        let n = nets.rang(&net.name);
        for piste in &net.tracks {
            if piste.points.len() < 2 {
                continue;
            }
            let mut item = objet([
                ("c", couches.rang(&piste.layer_name).into()),
                ("n", n.into()),
                ("w", r(piste.width)),
                ("p", pts(&piste.points)),
            ]);
            if !piste.fill_property.is_empty() && piste.fill_property != "HOLLOW" {
                item.insert("f".into(), piste.fill_property.as_str().into());
            }
            if !piste.bout.is_empty() {
                item.insert("b".into(), piste.bout.as_str().into()); // lineEnd SQUARE ou NONE (ROUND : absent)
            }
            pistes.push(Json::Objet(item));
        }
        for arc in &net.arcs {
            let mut item = objet([
                ("c", couches.rang(&arc.layer_name).into()),
                ("n", n.into()),
                ("w", r(arc.width)),
                ("s", xy(arc.start)),
                ("e", xy(arc.end)),
                ("m", xy(arc.center)),
                ("h", (arc.clockwise as i64).into()),
            ]);
            if !arc.bout.is_empty() {
                item.insert("b".into(), arc.bout.as_str().into());
            }
            arcs.push(Json::Objet(item));
        }
        for plan in &net.copper_planes {
            let formes: Vec<Json> = plan.contours.iter().filter_map(|c| contour(Some(c))).collect();
            if formes.is_empty() {
                continue;
            }
            plans.push(Json::Objet(objet([
                ("c", couches.rang(&plan.layer_name).into()),
                ("n", n.into()),
                ("f", plan.fill_property.as_str().into()),
                ("g", Json::Liste(formes)),
            ])));
        }
    }

    let mut textes = Vec::new();
    for texte in &design.texts {
        let mut item = objet([
            ("c", couches.rang(&texte.layer_name).into()),
            ("t", texte.text.as_str().into()),
            ("x", r(texte.location.x)),
            ("y", r(texte.location.y)),
        ]);
        if texte.rotation != 0.0 {
            item.insert("r".into(), r(texte.rotation));
        }
        if texte.mirror {
            item.insert("m".into(), 1i64.into());
        }
        // taille et boite, quand le fichier les donne
        if texte.height > 0.0 {
            item.insert("h".into(), r(texte.height));
        }
        if let Some(boite) = texte.bbox {
            item.insert("b".into(), Json::Liste(boite.iter().map(|v| r(*v)).collect()));
        }
        textes.push(Json::Objet(item));
    }

    // Les cuivres de l'empilage, du dessus au dessous : la face d'un
    // contre-percage se juge contre eux.
    let mut pile: Vec<&StackupLayer> = design.stackup.iter().collect();
    pile.sort_by(|a, b| a.sequence.total_cmp(&b.sequence));
    let cuivres: Vec<&str> =
        pile.iter().filter(|c| fonction_cuivre(&c.layer_type.to_uppercase())).map(|c| c.name.as_str()).collect();
    let mut nb_cp = 0i64;
    let mut percages = Vec::new();
    for trou in &design.drills {
        let mut item = objet([
            ("x", r(trou.location.x)),
            ("y", r(trou.location.y)),
            ("d", r(trou.diameter)),
            ("p", if trou.plating.is_empty() { "UNKNOWN" } else { trou.plating.as_str() }.into()),
            ("ps", trou.padstack_ref.as_str().into()),
        ]);
        // une fente : sa longueur hors tout et l'angle de la longueur
        if trou.slot_length > trou.diameter {
            item.insert("l".into(), r(trou.slot_length));
            item.insert("r".into(), r(trou.slot_rotation));
        }
        let rang = nets.rang(&trou.net_name);
        if rang >= 0 {
            item.insert("n".into(), rang.into());
        }
        // LA PORTEE DU PERCAGE, EN RANGS DE COUCHE. Elle ne part QUE si les deux
        // bouts se resolvent sur des couches connues ET DIFFERENTES : une
        // portee a moitie resolue vaudrait moins que rien -- la visionneuse la
        // prendrait pour une declaration et bornerait le via sur une seule
        // couche, ce qui coupe un chemin de courant reel --, et une portee
        // d'une seule couche ne decrit aucun via. Absente, elle veut dire « le
        // fichier ne le dit pas », et l'outil suppose traversant EN LE DISANT.
        if trou.span_declaree() {
            let a = couches.rang_connu(&trou.span_from);
            let b = couches.rang_connu(&trou.span_to);
            if a >= 0 && b >= 0 && a != b {
                item.insert("sa".into(), a.into());
                item.insert("sb".into(), b.into());
                item.insert("ss".into(), trou.span_source.as_str().into());
            }
        }
        let anneau = trou.annular_ring();
        if anneau != 0.0 {
            item.insert("a".into(), r(anneau));
            // UN ANNEAU DEDUIT D'UNE PASTILLE DEVINEE N'EST PAS UNE MESURE.
            // Sans ce drapeau, la fiche affiche « Anneau 0,15 mm » avec le meme
            // aplomb qu'une cote lue dans le fichier -- alors que 0,15 est la
            // moitie du « + 0,3 » que le lecteur pose lui-meme faute de
            // padstack.
            if trou.annular_ring_supposee() {
                item.insert("a_sup".into(), true.into());
            }
        }
        if let Some(cp) = contre_percage(trou, design, &couches, &cuivres) {
            item.insert("cp".into(), cp);
            nb_cp += 1;
        }
        percages.push(Json::Objet(item));
    }

    let pads: Vec<Json> = design.standalone_pads.iter().map(|p| pad_en_dict(p, &mut nets)).collect();

    let mut composants = Vec::new();
    for comp in &design.components {
        let mut item = objet([
            ("ref", comp.ref_des.as_str().into()),
            ("pkg", comp.package_ref.as_str().into()),
            ("c", couches.rang(&comp.layer_ref).into()),
            ("x", r(comp.location.x)),
            ("y", r(comp.location.y)),
            ("r", r(comp.rotation)),
            ("m", (comp.mirror as i64).into()),
            ("mnt", if comp.mount_type.is_empty() { "UNKNOWN" } else { comp.mount_type.as_str() }.into()),
            ("val", comp.value.as_str().into()),
            ("tol", comp.tolerance.as_str().into()),
            ("type", comp.comp_type.as_str().into()),
            ("part", comp.part_number.as_str().into()),
            ("props", Json::Objet(comp.properties.iter().map(|(k, v)| (k.clone(), v.as_str().into())).collect())),
        ]);
        if !comp.populate {
            item.insert("nm".into(), 1i64.into()); // non monté (populate="false" dans la BOM)
        }
        if let Some(paquet) = &comp.package_obj {
            // les pastilles du boitier, sinon (aucune dans les calques) la
            // forme de ses broches : voir repli_broches du parseur
            let source = if paquet.pads.is_empty() { &comp.pads_repli } else { &paquet.pads };
            item.insert("pads".into(), Json::Liste(source.iter().map(|p| pad_en_dict(p, &mut nets)).collect()));
            let mut broches = Vec::new();
            for pin in &paquet.pins {
                let mut b = objet([("num", pin.number.as_str().into()), ("x", r(pin.x)), ("y", r(pin.y))]);
                // Le net d'une broche vient de <LogicalNet>, pas des <Pad> d'un
                // <Pin> -- ceux-la sont quasiment toujours absents d'un export
                // reel. comp.pin_nets est indexe par parse_logical_nets.
                if let Some(net_nom) = comp.pin_nets.get(&pin.number).filter(|n| !n.is_empty()) {
                    let rang = nets.rang(net_nom);
                    if rang >= 0 {
                        b.insert("n".into(), rang.into());
                    }
                }
                broches.push(Json::Objet(b));
            }
            item.insert("pins".into(), Json::Liste(broches));
            if let Some(h) = paquet.hauteur.filter(|h| *h != 0.0) {
                item.insert("hauteur".into(), r(h));
            }
            if !paquet.corps.is_empty() {
                item.insert("corps".into(), pts(&paquet.corps));
            }
        }
        composants.push(Json::Objet(item));
    }

    let mut padstacks = Objet::new();
    for (nom, pdef) in &design.padstacks {
        // Un padstack « inline » nait d'un <Pad> lu dans un <LayerFeature> : sa
        // couche est une vraie couche de features, meme si elle ne porte que des
        // pastilles (pate, masque). Absente de la table, la visionneuse prenait
        // la pastille pour un « ALL » et la posait sur tous les cuivres.
        if nom.starts_with("inline_pad_") {
            for p in &pdef.pads {
                couches.rang(&p.layer_ref);
            }
        }
        let mut item = objet([("trou", r(pdef.hole_diameter)), ("pad", r(pdef.pad_diameter))]);
        // « la pastille est supposee » -- absent quand elle est lue, pour
        // ne pas alourdir un document ou c'est le cas ordinaire.
        if pdef.pad_supposee {
            item.insert("pad_sup".into(), true.into());
        }
        item.insert(
            "pads".into(),
            Json::Liste(
                pdef.pads
                    .iter()
                    .map(|p| {
                        Json::Objet(objet([
                            ("c", p.layer_ref.as_str().into()),
                            ("d", r(p.pad_diameter)),
                            ("f", p.shape_ref.as_str().into()),
                            ("a", r(p.antipad_diameter)),
                        ]))
                    })
                    .collect(),
            ),
        );
        padstacks.insert(nom.clone(), Json::Objet(item));
    }

    let formes: Objet = design.shapes.iter().map(|(i, f)| (i.clone(), forme_en_dict(f))).collect();

    let mut formesuser = Objet::new();
    for (nom, forme) in &design.user_shapes {
        let mut gabarit = Objet::new();
        if !forme.tracks.is_empty() {
            gabarit.insert(
                "pistes".into(),
                Json::Liste(
                    forme
                        .tracks
                        .iter()
                        .filter(|t| t.points.len() >= 2)
                        .map(|t| Json::Objet(objet([("w", r(t.width)), ("p", pts(&t.points)), ("f", t.fill_property.as_str().into())])))
                        .collect(),
                ),
            );
        }
        if !forme.arcs.is_empty() {
            gabarit.insert(
                "arcs".into(),
                Json::Liste(
                    forme
                        .arcs
                        .iter()
                        .map(|a| {
                            Json::Objet(objet([
                                ("w", r(a.width)),
                                ("s", xy(a.start)),
                                ("e", xy(a.end)),
                                ("m", xy(a.center)),
                                ("h", (a.clockwise as i64).into()),
                            ]))
                        })
                        .collect(),
                ),
            );
        }
        if !forme.planes.is_empty() {
            gabarit.insert(
                "plans".into(),
                Json::Liste(forme.planes.iter().flat_map(|p| &p.contours).filter_map(|c| contour(Some(c))).collect()),
            );
        }
        if !forme.texts.is_empty() {
            gabarit.insert(
                "textes".into(),
                Json::Liste(
                    forme
                        .texts
                        .iter()
                        .map(|t| {
                            Json::Objet(objet([
                                ("t", t.text.as_str().into()),
                                ("x", r(t.location.x)),
                                ("y", r(t.location.y)),
                                ("r", r(t.rotation)),
                            ]))
                        })
                        .collect(),
                ),
            );
        }
        formesuser.insert(nom.clone(), Json::Objet(gabarit));
    }

    let longueur = py::sum(design.nets.values().filter_map(|net| net.longueur_pistes()));
    let metallises = design.drills.iter().filter(|d| metallise(&d.plating)).count();

    let classes_nets: Objet = design
        .nets
        .values()
        .filter(|net| !net.net_class.is_empty())
        .map(|net| (net.name.clone(), net.net_class.as_str().into()))
        .collect();

    let natures = natures_nets(&composants, &nets.noms, motifs);
    let stats = objet([
        ("couches", couches.noms.len().into()),
        ("empilage", empilage.len().into()),
        ("nets", nets.noms.len().into()),
        ("pistes", pistes.len().into()),
        ("arcs", arcs.len().into()),
        ("plans", plans.len().into()),
        ("textes", textes.len().into()),
        ("percages", percages.len().into()),
        ("percages_metallises", metallises.into()),
        ("pads", pads.len().into()),
        ("composants", composants.len().into()),
        ("padstacks", padstacks.len().into()),
        ("longueur_cuivre", r(longueur)),
        ("calques_ignores", design.ignored_layers.len().into()),
        ("contre_percages", nb_cp.into()),
        ("contre_percages_orphelins", design.contre_percages_orphelins.len().into()),
    ]);
    let modele = objet([
        ("format", FORMAT.into()),
        ("fichier", basename(fichier).into()),
        ("unites", design.units.as_str().into()),
        ("epaisseur", r(design.total_thickness)),
        ("contour", contour(design.board_outline.as_ref()).unwrap_or(Json::Nul)),
        ("empilage", Json::Liste(empilage)),
        ("couches", couches.en_json()),
        ("nets", nets.en_json()),
        ("classes_nets", Json::Objet(classes_nets)),
        ("regles", regles(design)),
        ("natures_nets", natures),
        ("pistes", Json::Liste(pistes)),
        ("arcs", Json::Liste(arcs)),
        ("plans", Json::Liste(plans)),
        ("textes", Json::Liste(textes)),
        ("percages", Json::Liste(percages)),
        ("pads", Json::Liste(pads)),
        ("composants", Json::Liste(composants)),
        ("padstacks", Json::Objet(padstacks)),
        ("formes", Json::Objet(formes)),
        ("formesuser", Json::Objet(formesuser)),
        // calques du fichier laisses de cote (nom -> layerFunction)
        ("ignores", Json::Objet(design.ignored_layers.iter().map(|(k, v)| (k.clone(), v.as_str().into())).collect())),
        // fonction (SILKSCREEN, SIGNAL...) et face (TOP, BOTTOM) de chaque
        // calque declare, par nom
        (
            "calques",
            Json::Objet(
                design
                    .layer_info
                    .iter()
                    .map(|(nom, i)| (nom.clone(), Json::Objet(objet([("f", i.fonction.as_str().into()), ("s", i.face.as_str().into())]))))
                    .collect(),
            ),
        ),
        ("stats", Json::Objet(stats)),
    ]);
    match NON_FINI.with(|c| c.take()) {
        Some(exception) => Err(Erreur::Interne(exception.to_string())),
        None => Ok(Json::Objet(modele)),
    }
}

/// Octets d'un fichier (XML ou ZIP) -> IPCDesign.
///
/// Les octets arrivent du navigateur, rien n'oblige a les poser sur le disque d'abord.
/// Rend Erreur::Parse comme le parseur levait IPC2581ParseError. tout_garder=false ne garde
/// que les calques utiles a une simulation (voir IPC2581Parser::role_calque).
pub fn charger_octets(data: &[u8], nom: &str, tout_garder: bool) -> Result<IPCDesign, Erreur> {
    if data.is_empty() {
        return Err(refus("Fichier vide.".into()));
    }

    let mut interne = String::new();
    let mut extrait: Option<Vec<u8>> = None;
    if data.starts_with(b"PK") {
        // shortcut: le texte de BadZipFile n'est repris que pour le cas courant (« File is not
        // a zip file ») ; une archive abîmée autrement porte ce même message.
        let illisible = |_| refus("Archive ZIP illisible : File is not a zip file".into());
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data)).map_err(illisible)?;
        // Le plus gros : une archive de fabrication contient parfois un petit
        // fichier d'index a cote du modele, et c'est le modele qu'on veut.
        let mut choix: Option<(usize, u64, String, bool)> = None;
        for i in 0..archive.len() {
            let f = archive.by_index_raw(i).map_err(illisible)?;
            let nom_f = f.name().to_string();
            let bas = nom_f.to_lowercase();
            if nom_f.ends_with('/') || !EXTENSIONS.iter().any(|e| bas.ends_with(e)) {
                continue;
            }
            if choix.as_ref().is_none_or(|c| f.size() > c.1) {
                choix = Some((i, f.size(), nom_f, f.encrypted()));
            }
        }
        let Some((rang, taille, nom_f, chiffre)) = choix else {
            return Err(refus(format!(
                "Aucun fichier IPC-2581 dans l'archive (extensions attendues : {}).",
                EXTENSIONS.join(", ")
            )));
        };
        const MAX_DECOMPRESSE: u64 = 200 * 1024 * 1024; // 200 Mo max
        if taille > MAX_DECOMPRESSE {
            return Err(refus(format!(
                "Le fichier IPC-2581 dans l'archive depasse la taille maximale autorisee ({} Mo).",
                MAX_DECOMPRESSE / (1024 * 1024)
            )));
        }
        if chiffre {
            return Err(Erreur::Interne(format!(
                "RuntimeError: File '{nom_f}' is encrypted, password required for extraction"
            )));
        }
        // Decompression bornee en streaming pour neutraliser les Zip Bombs
        let mut octets = Vec::new();
        let flux = archive.by_index(rang).map_err(|e| Erreur::Interne(format!("NotImplementedError: {e}")))?;
        flux.take(MAX_DECOMPRESSE + 1)
            .read_to_end(&mut octets)
            .map_err(|_| Erreur::Interne(format!("BadZipFile: Bad CRC-32 for file '{nom_f}'")))?;
        if octets.len() as u64 > MAX_DECOMPRESSE {
            return Err(refus(format!(
                "Decompression interrompue : le volume reel depasse {} Mo (Zip Bomb).",
                MAX_DECOMPRESSE / (1024 * 1024)
            )));
        }
        interne = nom_f;
        extrait = Some(octets);
    }

    let nom = [interne.as_str(), nom, "(flux)"].into_iter().find(|n| !n.is_empty()).unwrap_or("(flux)");
    Ok(IPC2581Parser::new(nom, tout_garder).parse(extrait.as_deref().unwrap_or(data))?)
}

/// Octets d'un fichier IPC-2581 (ou ZIP) -> dictionnaire JSON (« natures_nets » vide).
pub fn ipc2581_en_dict(data: &[u8], nom: &str, tout_garder: bool) -> Result<Json, Erreur> {
    ipc2581_en_dict_avec(data, nom, tout_garder, None)
}

/// Octets d'un fichier IPC-2581 (ou ZIP) -> dictionnaire JSON, « natures_nets » rempli par
/// le moteur de motifs de l'appelant.
pub fn ipc2581_en_dict_avec(data: &[u8], nom: &str, tout_garder: bool, motifs: Option<Motifs>) -> Result<Json, Erreur> {
    design_en_dict_avec(&charger_octets(data, nom, tout_garder)?, nom, motifs)
}
