//! Les essais du lecteur IPC-2581.
//!
//! 1. LA NON-RÉGRESSION CONTRE LE LECTEUR PYTHON. `essais/` garde les entrées que les bancs
//!    Python donnaient au lecteur (visionneuse-ipc2581/test/banc-essai.py et
//!    editeur-pcb/test/banc-ipc2581-export.py de WEB_CAO, interceptées à leur appel), les
//!    fichiers d'exemple des deux dépôts, et une carte « tout.xml » qui porte chaque balise
//!    que le lecteur connaît. À côté de chacune, ce que rendait
//!    `json.dumps(ipc2581_json.ipc2581_en_dict(octets, nom, tout_garder), ensure_ascii=False)`
//!    du lecteur Python 1.84 / 1.5.0 (Python 3.13, sans pattern_recognition : « natures_nets »
//!    vide), ou « ERREUR: message » pour une IPC2581ParseError, ou « EXCEPTION: Type: message »
//!    pour toute autre exception. Le Rust doit rendre la même chose, à l'octet.
//!      - manifeste.tsv : fichier, tout_garder, nom, origine ; la sortie est dans <cas>.json ;
//!      - derives.tsv : nom, tout_garder, SHA-256 de la sortie, puis des paires (texte,
//!        remplacement) à appliquer à tout.xml -- unités, espaces de noms, et chaque écriture
//!        de nombre (« 1e400 », « ١٢ », « 1_0.5 »...) à chaque endroit où le lecteur lit un
//!        nombre ;
//!      - petits.tsv : nom, tout_garder, entrée en hexadécimal, sortie -- XML abîmé,
//!        encodages, archives.
//! 2. LES CAS DES BANCS PYTHON qui interrogent le lecteur lui-même (l'IPCDesign, le découpage
//!    d'un arc, le type d'un composant), portés avec leurs assertions. Ceux qui ne regardent
//!    que le modèle JSON sont tenus par la comparaison ci-dessus, qui porte sur leurs entrées.

use crate::*;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn dossier() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("essais")
}

fn lire(nom: &str) -> Vec<u8> {
    std::fs::read(dossier().join(nom)).unwrap_or_else(|e| panic!("essais/{nom} : {e}"))
}

fn lignes(nom: &str) -> Vec<Vec<String>> {
    String::from_utf8(lire(nom)).unwrap().lines().map(|l| l.split('\t').map(str::to_string).collect()).collect()
}

/// Ce que le script de référence Python écrivait pour une entrée.
fn sortie(octets: &[u8], nom: &str, tout_garder: bool) -> String {
    match ipc2581_en_dict(octets, nom, tout_garder) {
        Ok(modele) => dumps(&modele),
        Err(Erreur::Parse(e)) => format!("ERREUR: {e}"),
        Err(Erreur::Interne(e)) => format!("EXCEPTION: {e}"),
    }
}

/// « \\t », « \\n », « \\r » et « \\\\ » des tables.
fn clair(t: &str) -> String {
    let mut s = String::new();
    let mut it = t.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            s.push(c);
            continue;
        }
        s.push(match it.next() {
            Some('t') => '\t',
            Some('n') => '\n',
            Some('r') => '\r',
            _ => '\\',
        });
    }
    s
}

fn hexa(t: &str) -> Vec<u8> {
    (0..t.len() / 2).map(|i| u8::from_str_radix(&t[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn sha256(t: &str) -> String {
    Sha256::digest(t.as_bytes()).iter().map(|o| format!("{o:02x}")).collect()
}

#[test]
fn les_sorties_sont_celles_du_lecteur_python() {
    let cas = lignes("manifeste.tsv");
    assert!(cas.len() >= 60, "{} cas", cas.len());
    for c in &cas {
        let (fichier, tout, nom) = (&c[0], c[1] == "1", &c[2]);
        let attendu = String::from_utf8(lire(&format!("{}.json", fichier.rsplit_once('.').unwrap().0))).unwrap();
        let vu = sortie(&lire(fichier), nom, tout) + "\n";
        assert!(vu == attendu, "{fichier} ({nom}, {}) : la sortie n'est plus celle du lecteur Python", c[3]);
    }
}

#[test]
fn les_derives_de_tout_xml_aussi() {
    let base = String::from_utf8(lire("tout.xml")).unwrap();
    let cas = lignes("derives.tsv");
    assert!(cas.len() >= 600, "{} cas", cas.len());
    let mut ecarts = Vec::new();
    for c in &cas {
        let mut xml = base.clone();
        for paire in c[3..].chunks(2) {
            let (cible, remplacement) = (clair(&paire[0]), clair(&paire[1]));
            assert!(xml.contains(&cible), "{} : « {cible} » n'est pas dans tout.xml", c[0]);
            xml = xml.replace(&cible, &remplacement);
        }
        if sha256(&(sortie(xml.as_bytes(), &c[0], c[1] == "1") + "\n")) != c[2] {
            ecarts.push(c[0].as_str());
        }
    }
    assert!(ecarts.is_empty(), "sorties qui ne sont plus celles du lecteur Python : {ecarts:?}");
}

#[test]
fn le_xml_abime_les_encodages_et_les_archives_aussi() {
    let cas = lignes("petits.tsv");
    assert!(cas.len() >= 130, "{} cas", cas.len());
    for c in &cas {
        assert_eq!(sortie(&hexa(&c[2]), &c[0], c[1] == "1"), clair(&c[3]), "{} (tout_garder={})", c[0], c[1]);
    }
}

// =============================================================================
// La carte d'essai de visionneuse-ipc2581/test/banc-essai.py
// -----------------------------------------------------------------------------
// Deux couches de cuivre separees par 0,2 mm de dielectrique, un contour de
// 40 x 20, une piste coudee de 20 + 10 mm sur le dessus, la pastille et le via
// ou elle s'arrete, une resistance et son boitier.
// =============================================================================

fn carte() -> String {
    String::from_utf8(lire("carte-essai.xml")).unwrap()
}

/// La carte d'essai, une substitution faite (la première occurrence, qui doit exister).
fn variante(cible: &str, remplacement: &str) -> String {
    let c = carte();
    assert!(c.contains(cible), "la carte variante n'a pas ete construite");
    c.replacen(cible, remplacement, 1)
}

fn design(xml: &str, nom: &str) -> IPCDesign {
    charger_octets(xml.as_bytes(), nom, true).unwrap()
}

fn modele(xml: &str, nom: &str) -> Json {
    ipc2581_en_dict(xml.as_bytes(), nom, true).unwrap()
}

fn proche(vu: f64, attendu: f64) {
    assert!((vu - attendu).abs() <= 1e-6, "{attendu} attendu, {vu} vu");
}

fn couche<'a>(d: &'a IPCDesign, nom: &str) -> &'a StackupLayer {
    d.stackup.iter().find(|c| c.name == nom).unwrap_or_else(|| panic!("couche {nom} absente de l'empilage"))
}

const CALQUE_PERCAGE: &str = r#"<Layer name="Hole1-2" layerFunction="DRILL" side="ALL"/>"#;

#[test]
fn unites_et_empilage() {
    let d = design(&carte(), "carte-essai.xml");
    // les unites viennent de <CadHeader>
    assert_eq!(d.units, "MILLIMETER");
    // l'empilage a ses trois couches, dans l'ordre des sequences
    assert_eq!(d.stackup.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["Conductor-1", "DielectricLayer-1-2", "Conductor-2"]);
    proche(couche(&d, "DielectricLayer-1-2").thickness, 0.2);
    // l'epaisseur totale vient de <Stackup overallThickness>
    proche(d.total_thickness, 0.27);
    // layerFunction distingue le conducteur, le plan et le dielectrique
    assert_eq!(
        ["Conductor-1", "DielectricLayer-1-2", "Conductor-2"].map(|n| couche(&d, n).layer_type.as_str()),
        ["SIGNAL", "DIELPREG", "PLANE"]
    );
    // le Dk et le Df se lisent dans une <Spec> pointee par <SpecRef> (forme type+Property)
    assert_eq!(couche(&d, "DielectricLayer-1-2").dk, "4.37");
    assert_eq!(couche(&d, "DielectricLayer-1-2").df, "0.022");
    // une couche sans <SpecRef> n'invente pas de permittivite
    assert_eq!(couche(&d, "Conductor-1").dk, "");
}

#[test]
fn contour_pistes_pastilles_et_percages() {
    let d = design(&carte(), "carte-essai.xml");
    // le contour de carte est lu depuis <Profile>, et fait bien 40 x 20
    let contour = d.board_outline.as_ref().expect("aucun contour de carte");
    assert_eq!(
        contour.outline[..4].iter().map(|p| (py::round(p.x, 3), py::round(p.y, 3))).collect::<Vec<_>>(),
        [(0.0, 0.0), (40.0, 0.0), (40.0, 20.0), (0.0, 20.0)]
    );
    // les deux pistes de la carte sont lues
    let pistes: Vec<&Track> = d.nets.values().flat_map(|n| &n.tracks).collect();
    assert_eq!(pistes.len(), 2);
    let mut largeurs: Vec<f64> = pistes.iter().map(|p| p.width).collect();
    largeurs.sort_by(f64::total_cmp);
    // un <LineDesc> sans enfant donne quand meme sa largeur
    proche(largeurs[0], 0.2);
    // un <LineDescRef> va chercher sa largeur dans le dictionnaire
    proche(largeurs[1], 0.25);
    // les pistes portent leur net et leur couche
    assert!(pistes.iter().all(|p| (p.net_name.as_str(), p.layer_name.as_str()) == ("SIG_A", "Conductor-1")));
    // la pastille du <Set> est lue
    assert_eq!(d.standalone_pads.len(), 1);
    // le percage est lu, avec son diametre et sa metallisation
    assert_eq!(d.drills.iter().map(|t| (py::round(t.diameter, 3), t.plating.as_str())).collect::<Vec<_>>(), [(0.25, "PLATED")]);
    // un percage marque padUsage=VIA est reconnu comme via
    assert!(d.drills.iter().any(|t| !t.padstack_ref.is_empty()), "le percage n'a pas de padstack de via");
    // le via est a l'endroit ou la piste s'arrete
    assert_eq!((d.drills[0].location.x, d.drills[0].location.y), (25.0, 15.0));
}

#[test]
fn la_pastille_d_un_via_lue_ou_devinee() {
    let d = design(&carte(), "carte-essai.xml");
    let m = design_en_dict(&d, "carte-essai.xml").unwrap();
    // la pastille d'un via sans padstack declare est marquee comme supposee
    let v25 = d.padstacks.get("v25").expect("v25");
    assert!(v25.pad_supposee, "la pastille inventee passe pour une cote du fichier");
    // et elle vaut exactement percage + 0,3 : c'est la convention, pas une mesure
    proche(v25.pad_diameter, 0.25 + 0.3);
    // l'anneau deduit d'une pastille inventee s'avoue jusqu'a la page
    proche(d.drills[0].annular_ring(), 0.15);
    assert!(d.drills[0].annular_ring_supposee(), "l'anneau ne s'avoue pas suppose");
    assert_eq!(m["percages"][0]["a_sup"], Json::Bool(true));
    assert_eq!(m["padstacks"]["v25"]["pad_sup"], Json::Bool(true));

    // LE REVERS : une pastille declaree dans le fichier n'est PAS annoncee supposee
    let declare = variante(
        "  <CadData>",
        r#"  <CadData>
   <PadstackDef name="v25" padUsage="VIA">
    <PadstackHoleDef name="R0.125" diameter="0.25"/>
    <PadstackPadDef layerRef="TOP" padUse="REGULAR">
     <Circle diameter="0.70"/>
    </PadstackPadDef>
   </PadstackDef>"#,
    );
    let d2 = design(&declare, "carte-declaree.xml");
    let ps = d2.padstacks.get("v25").expect("le padstack declare n'a pas ete lu");
    assert!(!ps.pad_supposee, "une pastille DECLAREE est quand meme annoncee supposee");
    proche(ps.pad_diameter, 0.70);
    // Et l'anneau qui en decoule est une vraie cote : (0,70 - 0,25) / 2.
    proche(d2.drills[0].annular_ring(), 0.225);
    assert!(!d2.drills[0].annular_ring_supposee(), "un anneau lu dans le fichier s'annonce suppose");
    let m2 = design_en_dict(&d2, "carte-declaree.xml").unwrap();
    assert!(m2["padstacks"]["v25"].get("pad_sup").is_none(), "le modele marque suppose un padstack declare");
    assert!(m2["percages"][0].get("a_sup").is_none(), "le modele marque suppose un anneau lu");
}

#[test]
fn la_portee_d_un_percage() {
    // un percage sans <Span> ne se voit pas attribuer de portee
    let d = design(&carte(), "carte-essai.xml");
    let m = design_en_dict(&d, "carte-essai.xml").unwrap();
    assert!(!d.drills[0].span_declaree() && m["percages"][0].get("sa").is_none(), "une portee a ete inventee");

    // la portee declaree sur le calque DRILL est lue et part en rangs
    let avec = |span: &str| variante(CALQUE_PERCAGE, &format!(r#"<Layer name="Hole1-2" layerFunction="DRILL" side="ALL">{span}</Layer>"#));
    let d = design(&avec(r#"<Span fromLayer="Conductor-1" toLayer="Conductor-2"/>"#), "portee.xml");
    let trou = &d.drills[0];
    assert!(trou.span_declaree(), "la portee n'a pas ete lue");
    assert_eq!([&trou.span_from, &trou.span_to, &trou.span_source], ["Conductor-1", "Conductor-2", "calque"]);
    let p = &design_en_dict(&d, "portee.xml").unwrap()["percages"][0];
    assert_eq!((p["sa"].as_i64(), p["sb"].as_i64(), p["ss"].as_str()), (Some(0), Some(2), Some("calque")));

    // la portee se retrouve aussi par le padstack du percage
    let par_padstack = variante(
        CALQUE_PERCAGE,
        &format!(
            r#"{CALQUE_PERCAGE}<Layer name="Hole-enterre" layerFunction="DRILL" side="ALL"><Span fromLayer="Conductor-2" toLayer="Conductor-1"/></Layer>"#
        ),
    )
    .replacen(
        "  <CadData>",
        r#"  <CadData>
   <PadstackDef name="v25" padUsage="VIA">
    <PadstackHoleDef name="R0.125" diameter="0.25" layerRef="Hole-enterre"/>
   </PadstackDef>"#,
        1,
    );
    let d = design(&par_padstack, "portee2.xml");
    assert_eq!(d.padstacks["v25"].hole_layer_ref, "Hole-enterre");
    let trou = &d.drills[0];
    assert_eq!([&trou.span_from, &trou.span_to, &trou.span_source], ["Conductor-2", "Conductor-1", "padstack"]);

    // une portee dont les couches sont inconnues ne part pas, et n'invente rien
    let d = design(&avec(r#"<Span fromLayer="TOP" toLayer="BOTTOM"/>"#), "portee3.xml");
    assert!(d.drills[0].span_declaree(), "la portee brute n'a pas ete lue");
    let m = design_en_dict(&d, "portee3.xml").unwrap();
    assert!(m["percages"][0].get("sa").is_none() && m["percages"][0].get("sb").is_none());
    let couches: Vec<&str> = m["couches"].as_liste().unwrap().iter().filter_map(Json::as_str).collect();
    assert!(!couches.contains(&"TOP") && !couches.contains(&"BOTTOM"), "un calque fantome est entre dans la table");

    // une portee qui tient sur une seule couche ne part pas non plus
    let m = modele(&avec(r#"<Span fromLayer="Conductor-1" toLayer="Conductor-1"/>"#), "portee4.xml");
    assert!(m["percages"][0].get("sa").is_none(), "une portee d'une seule couche est partie");
}

#[test]
fn composants_et_boitiers() {
    let d = design(&carte(), "carte-essai.xml");
    // le composant est lu avec son repere et son boitier
    assert_eq!(d.components.iter().map(|c| (c.ref_des.as_str(), c.package_ref.as_str())).collect::<Vec<_>>(), [("R1", "R0603")]);
    proche(d.components[0].rotation, 90.0);
    assert_eq!(d.packages["R0603"].pins.len(), 2);
    // le net d'une broche vient de <LogicalNet>, pas des <Pad> du <Pin>
    assert_eq!(d.components[0].pin_nets.iter().collect::<Vec<_>>(), [(&"1".to_string(), &"SIG_A".to_string())]);
    // la valeur et le type de R1 viennent de part='RES-10K'
    assert_eq!((d.components[0].value.as_str(), d.components[0].comp_type.as_str()), ("10K", "RESISTOR"));

    // les passifs R, C, L sont lus avec leur type, valeur et tolérance, depuis <Property> et <BomItem>
    let passifs = variante(
        "</Component>",
        r#" <Property name="Value" value="10k"/>
 <Property name="Tolerance" value="1%"/>
</Component>
<Component refDes="C1" packageRef="R0603" layerRef="Component-1-A">
 <Property name="Value" value="100nF"/>
 <Property name="Voltage" value="50V"/>
</Component>
<Component refDes="L1" packageRef="R0603" layerRef="Component-1-A">
 <Property name="Value" value="10uH"/>
 <Property name="Tolerance" value="10%"/>
</Component>"#,
    )
    .replacen(
        "</Ecad>",
        r#" <Bom>
  <BomHeader>
   <BomItem OEMDesignNumberRef="IND_10uH_0603">
    <RefDes name="L1"/>
   </BomItem>
  </BomHeader>
 </Bom>
</Ecad>"#,
        1,
    );
    let d = design(&passifs, "passifs.xml");
    let comp = |r: &str| d.components.iter().find(|c| c.ref_des == r).unwrap_or_else(|| panic!("{r} manque"));
    let (r1, c1, l1) = (comp("R1"), comp("C1"), comp("L1"));
    assert_eq!([&r1.value, &r1.tolerance, &r1.comp_type], ["10k", "1%", "RESISTOR"]);
    assert_eq!([&c1.value, &c1.comp_type, &c1.properties["Voltage"]], ["100nF", "CAPACITOR", "50V"]);
    assert_eq!([&l1.value, &l1.tolerance, &l1.comp_type], ["10uH", "10%", "INDUCTOR"]);

    // les composants actifs (D, Q, U) et connecteurs (J) sont qualifiés par leur type
    let actifs = variante(
        "</Component>",
        r#"</Component>
<Component refDes="D1" packageRef="SOD123" layerRef="Component-1-A">
 <Property name="Value" value="BATS54"/>
</Component>
<Component refDes="Q1" packageRef="SOT23" layerRef="Component-1-A">
 <Property name="Value" value="2N7002"/>
</Component>
<Component refDes="U1" packageRef="SOIC8" layerRef="Component-1-A">
 <Property name="Part" value="NE555"/>
</Component>
<Component refDes="J1" packageRef="HDR_1X4" layerRef="Component-1-A">
 <Property name="Value" value="CONN_4P"/>
</Component>"#,
    );
    let d = design(&actifs, "actifs.xml");
    let genre = |r: &str| d.components.iter().find(|c| c.ref_des == r).unwrap().comp_type.as_str();
    assert_eq!(["D1", "Q1", "U1", "J1"].map(genre), ["DIODE", "TRANSISTOR", "IC", "CONNECTOR"]);
}

/// banc-analyse-carte.py : le type que l'analyse lit vient du parseur IPC-2581 : des mots
/// entiers, pas des sous-chaînes (Resonators n'est pas RES, CERAMIC pas IC).
#[test]
fn type_composant_ipc() {
    let q = |ref_des: &str, cle: &str, valeur: &str| {
        let props: indexmap::IndexMap<String, String> = [(cle.to_string(), valeur.to_string())].into_iter().collect();
        IPC2581Parser::qualifier_type_composant(ref_des, "", Some(&props))
    };
    assert_eq!(q("Y1", "CATEGORY", "Crystals, Oscillators, Resonators"), "CRYSTAL");
    assert_eq!(q("C5", "DEVICE", "C_0603_CERAMIC"), "CAPACITOR");
    assert_ne!(q("T1", "TYPE", "Transformer"), "TRANSISTOR");
    assert_eq!(q("R1", "TYPE", "Resistor"), "RESISTOR");
    assert_eq!(q("U1", "CATEGORY", "Integrated Circuits (ICs)"), "IC");
    assert_eq!(q("Q1", "CATEGORY", "Transistors"), "TRANSISTOR");
    assert_eq!(IPC2581Parser::extraire_val_tol("C0603_100nF_+-10%_X7R"), ("100nF".to_string(), "+-10".to_string()));
    assert_eq!(IPC2581Parser::extraire_val_tol("RES_0603_10K_1%"), ("10K".to_string(), "1%".to_string()));
    assert_eq!(IPC2581Parser::extraire_val_tol("RES-4R7"), ("4R7".to_string(), String::new()));
}

#[test]
fn le_modele_de_la_visionneuse() {
    let d = design(&carte(), "carte-essai.xml");
    let m = design_en_dict(&d, "carte-essai.xml").unwrap();
    assert_eq!(m["format"].as_str(), Some(FORMAT));
    // la classe de net (netClass) est lue et exportee dans classes_nets
    assert_eq!((m["classes_nets"]["SIG_A"].as_str(), m["classes_nets"]["GND"].as_str()), (Some("SIGNAL"), Some("GROUND")));
    // les couches deviennent un index, l'empilage d'abord
    assert_eq!(
        m["couches"].as_liste().unwrap()[..3].iter().map(|c| c.as_str().unwrap()).collect::<Vec<_>>(),
        ["Conductor-1", "DielectricLayer-1-2", "Conductor-2"]
    );
    // une piste designe sa couche et son net par leur rang
    assert!(m["pistes"].as_liste().unwrap().iter().all(|p| p["c"].as_i64().is_some() && p["n"].as_i64().is_some()));
    // dans le modele, la broche 1 de R1 porte le rang de SIG_A ; la broche 2 ressort sans net
    let broches = m["composants"][0]["pins"].as_liste().unwrap();
    assert_eq!(m["nets"][broches[0]["n"].as_i64().unwrap() as usize].as_str(), Some("SIG_A"));
    assert!(broches[1].get("n").is_none());
    // le cuivre total est la somme des deux pistes (20 + 10 mm)
    proche(m["stats"]["longueur_cuivre"].as_f64().unwrap(), 30.0);
    assert_eq!(["pistes", "percages", "composants", "empilage", "percages_metallises"].map(|k| m["stats"][k].as_i64().unwrap()), [2, 1, 1, 3, 1]);
    assert!(m["contour"].vrai(), "le contour ne ressort pas dans le modele");
    // l'empilage garde le Dk pour le calcul d'impedance de la page
    assert_eq!(m["empilage"].as_liste().unwrap().iter().map(|c| c["dk"].as_str().unwrap()).collect::<Vec<_>>(), ["", "4.37", ""]);
    // le nom du fichier suit le modele, sans son chemin
    assert_eq!(design_en_dict(&d, "d/carte.xml").unwrap()["fichier"].as_str(), Some("carte.xml"));
    // la carte d'essai ordinaire ne porte pas de contre-percage
    assert_eq!(m["stats"]["contre_percages"].as_i64(), Some(0));

    // platingStatus="VIA" compte comme un trou metallise ; le JSON garde VIA
    let via = modele(&carte().replace(r#"platingStatus="PLATED""#, r#"platingStatus="VIA""#), "via.xml");
    assert_eq!((via["stats"]["percages_metallises"].as_i64(), via["percages"][0]["p"].as_str()), (Some(1), Some("VIA")));
}

/// La nature des nets vient du moteur de motifs de la schematique, que l'appelant fournit :
/// il reçoit les composants et les nets, et ce qu'il rend part dans « natures_nets ».
#[test]
fn natures_des_nets_par_le_moteur_de_l_appelant() {
    let d = design(&carte(), "carte-essai.xml");
    let recu = std::cell::RefCell::new(String::new());
    let moteur = |entree: &Json| {
        *recu.borrow_mut() = dumps(entree);
        let mut classes = Objet::new();
        classes.insert("GND".into(), "Masse".into());
        let mut res = Objet::new();
        res.insert("classes_suggerees".into(), Json::Objet(classes));
        res.insert("raisons_classes".into(), Json::Objet(Objet::new()));
        res.insert("paires_diff".into(), Json::Liste(vec![]));
        res.insert("nets_bruyants".into(), Json::Liste(vec!["SIG_B".into(), "SIG_A".into()]));
        Some(Json::Objet(res))
    };
    let m = design_en_dict_avec(&d, "carte-essai.xml", Some(&moteur)).unwrap();
    // ce que pattern_recognition.analyser_motifs_schema recevait du pont Python pour cette carte
    assert_eq!(
        *recu.borrow(),
        r#"{"components": {"R1": {"val": "10K", "type": "RESISTOR"}}, "nets": {"SIG_A": [{"ref": "R1", "pin": "1"}], "GND": []}}"#
    );
    assert_eq!(
        dumps(&m["natures_nets"]),
        r#"{"classes": {"GND": "Masse"}, "raisons": {}, "paires": [], "bruyants": ["SIG_A", "SIG_B"]}"#
    );
    // un moteur qui échoue n'empêche pas l'import ; sans moteur, le dictionnaire est vide
    let en_panne = |_: &Json| None;
    assert_eq!(dumps(&design_en_dict_avec(&d, "", Some(&en_panne)).unwrap()["natures_nets"]), "{}");
    assert_eq!(dumps(&design_en_dict(&d, "").unwrap()["natures_nets"]), "{}");
}

#[test]
fn archives() {
    // une archive ZIP contenant la carte se lit comme la carte
    assert_eq!(charger_octets(&lire("dossier.zip"), "dossier.zip", true).unwrap().components.len(), 1);
    // dans une archive, c'est le plus gros IPC-2581 qui est pris
    assert_eq!(charger_octets(&lire("dossier-2.zip"), "dossier.zip", true).unwrap().stackup.len(), 3);
    // une archive sans IPC-2581 est refusee en le disant
    match charger_octets(&lire("dossier-3.zip"), "dossier.zip", true) {
        Err(Erreur::Parse(e)) => assert_eq!(
            e.to_string(),
            "Aucun fichier IPC-2581 dans l'archive (extensions attendues : .xml, .cvg, .ipc, .ipc2581)."
        ),
        autre => panic!("une archive sans IPC-2581 a ete acceptee : {autre:?}"),
    }
}

#[test]
fn refus() {
    let refuse = |data: &[u8], quoi: &str| match charger_octets(data, quoi, true) {
        Err(Erreur::Parse(e)) => e.to_string(),
        Err(Erreur::Interne(e)) => panic!("{quoi} : {e} au lieu d'IPC2581ParseError"),
        Ok(_) => panic!("{quoi} : accepte alors qu'il devrait etre refuse"),
    };
    assert_eq!(refuse(b"", "fichier vide"), "Fichier vide.");
    assert_eq!(refuse(b"<IPC-2581><oups>", "XML tronque"), "XML invalide ou corrompu : XML tronque (no element found: line 1, column 16)");
    assert_eq!(
        refuse(b"ceci n'est pas un fichier IPC-2581", "texte quelconque"),
        "XML invalide ou corrompu : texte quelconque (syntax error: line 1, column 0)"
    );
    assert_eq!(refuse(b"PK\x03\x04 pas vraiment une archive", "ZIP tronque"), "Archive ZIP illisible : File is not a zip file");
    // une entite XML declaree apres 64 Ko de commentaire est refusee
    let mut tardive = b"<?xml version=\"1.0\"?><!--".to_vec();
    tardive.extend(std::iter::repeat_n(b'x', 70000));
    tardive.extend(b"--><!DOCTYPE IPC-2581 [<!ENTITY a \"aaaa\">]><IPC-2581>&a;</IPC-2581>");
    assert_eq!(refuse(&tardive, "entite tardive"), "Declaration d'entite XML interdite (<!ENTITY).");
    // banc-serveur-routes.py : protection XML Entity Expansion (Billion Laughs / DTD)
    // en UTF-16 les octets bruts ne montrent pas la déclaration : elle est cherchée dans le texte décodé
    let mut utf16 = vec![0xFF, 0xFE];
    utf16.extend("<!DOCTYPE IPC-2581 [<!ENTITY a \"aaaa\">]><IPC-2581>&a;</IPC-2581>".encode_utf16().flat_map(u16::to_le_bytes));
    assert_eq!(refuse(&utf16, "entite en UTF-16"), "Declaration d'entite XML interdite (<!ENTITY).");
    let xxe = b"<?xml version=\"1.0\"?>\n<!DOCTYPE lolz [\n <!ENTITY lol \"lol\">\n]>\n<IPC-2581></IPC-2581>";
    assert!(IPC2581Parser::new("(flux)", true).parse(xxe).unwrap_err().to_string().contains("interdite"));
    assert_eq!(
        refuse(b"<Autre/>", "autre.xml"),
        "Ce n'est pas un fichier IPC-2581 : la racine est <Autre>, il faut <IPC-2581>."
    );
    assert_eq!(
        IPC2581Parser::new("/nulle/part/carte.xml", true).parse_fichier().unwrap_err().to_string(),
        "Fichier introuvable : /nulle/part/carte.xml"
    );
}

#[test]
fn tri_des_calques_et_textes() {
    let m0 = modele(&carte(), "carte-essai.xml");
    // tri des calques : la serigraphie reste lue par defaut, ecartee sur demande
    let serigraphie = variante(
        "    <LayerFeature layerRef=\"Hole1-2\">",
        r#"    <LayerFeature layerRef="Symbol-A">
     <Set>
      <Features>
       <Line startX="0" startY="0" endX="10" endY="0">
        <LineDesc lineEnd="ROUND" lineWidth="0.15"/>
       </Line>
      </Features>
     </Set>
    </LayerFeature>
    <LayerFeature layerRef="Hole1-2">"#,
    );
    let tout = ipc2581_en_dict(serigraphie.as_bytes(), "seri.xml", true).unwrap();
    let tri = ipc2581_en_dict(serigraphie.as_bytes(), "seri.xml", false).unwrap();
    let pistes = |m: &Json| m["stats"]["pistes"].as_i64().unwrap();
    assert_eq!(pistes(&tout), pistes(&m0) + 1);
    assert_eq!(dumps(&tout["ignores"]), "{}");
    assert_eq!(pistes(&tri), pistes(&m0));
    assert_eq!(dumps(&tri["ignores"]), r#"{"Symbol-A": "SILKSCREEN"}"#);
    assert_eq!(tri["stats"]["calques_ignores"].as_i64(), Some(1));

    // textes : textString, taille par la boite ou fontSize ; face des calques
    let seri = variante(
        "    <LayerFeature layerRef=\"Hole1-2\">",
        r#"    <LayerFeature layerRef="Symbol-A">
     <Set>
      <Features>
       <Text textString="R1" fontSize="1">
        <Xform rotation="90"/>
        <BoundingBox lowerLeftX="4" lowerLeftY="2" upperRightX="6" upperRightY="3.2"/>
        <Location x="4" y="2"/>
       </Text>
       <Text text="ANCIEN">
        <Location x="1" y="1"/>
       </Text>
       <Text textString="SANS BOITE" fontSize="0.8"/>
      </Features>
     </Set>
    </LayerFeature>
    <LayerFeature layerRef="Hole1-2">"#,
    );
    let m = modele(&seri, "textes.xml");
    let texte = |t: &str| m["textes"].as_liste().unwrap().iter().find(|x| x["t"].as_str() == Some(t)).unwrap_or_else(|| panic!("{t} manque"));
    proche(texte("R1")["h"].as_f64().unwrap(), 1.2);
    assert_eq!(dumps(&texte("R1")["b"]), "[4, 2, 6, 3.2]");
    assert_eq!(texte("R1")["r"].as_i64(), Some(90));
    proche(texte("SANS BOITE")["h"].as_f64().unwrap(), 0.8);
    assert!(texte("ANCIEN").get("h").is_none(), "sans taille, pas de cle");
    assert_eq!(dumps(&m["calques"]["Symbol-A"]), r#"{"f": "SILKSCREEN", "s": "TOP"}"#);
    assert_eq!(m["calques"]["Conductor-2"]["s"].as_str(), Some("BOTTOM"));
}

fn polygone(lecteur: &IPC2581Parser, xml: &str) -> Vec<Point> {
    let doc = roxmltree::Document::parse(xml).unwrap();
    lecteur.parse_polygon(doc.root_element()).unwrap()
}

#[test]
fn arcs_de_polygone() {
    // un PolyStepCurve qui revient a son debut est un cercle, pas un point
    let pts = polygone(
        &IPC2581Parser::new("x.xml", true),
        r#"<Polygon><PolyBegin x="1" y="0"/><PolyStepCurve x="1" y="0" centerX="0" centerY="0" clockwise="false"/></Polygon>"#,
    );
    let aire = (0..pts.len()).map(|i| (pts[i], pts[(i + 1) % pts.len()])).map(|(a, b)| a.x * b.y - b.x * a.y).sum::<f64>().abs() / 2.0;
    assert!((aire - std::f64::consts::PI).abs() <= 0.01, "aire d'un cercle de rayon 1 (fin = debut) : {aire}");

    // un arc de polygone se decoupe tous les 0,05 mm, meme en pouces :
    // un quart de cercle de 1 pouce, 40 mm d'arc, une corde par 0,05 mm (360 au plus)
    let mut lecteur = IPC2581Parser::new("x.xml", true);
    lecteur.design.units = "INCH".into();
    let quart = r#"<Polygon><PolyBegin x="1" y="0"/><PolyStepCurve x="0" y="1" centerX="0" centerY="0" clockwise="false"/></Polygon>"#;
    assert!(polygone(&lecteur, quart).len() > 300, "arc en pouces decoupe finement");
}

/// La carte minimale de banc-essai.py (_mini) : un sujet par carte.
fn mini(step: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<IPC-2581 revision="C" xmlns="http://webstds.ipc.org/2581">
 <Content roleRef="owner">
  <DictionaryLineDesc units="MILLIMETER">
   <EntryLineDesc id="l2"><LineDesc lineEnd="SQUARE" lineWidth="0.2"/></EntryLineDesc>
  </DictionaryLineDesc>

 </Content>
 <Ecad name="mini">
  <CadHeader units="MILLIMETER"/>
  <CadData>
   <Layer name="TOP" layerFunction="SIGNAL" side="TOP"/>
   <Step name="MINI">{step}</Step>
  </CadData>
 </Ecad>
</IPC-2581>"#
    )
}

#[test]
fn nombres_non_finis() {
    // une coordonnee inf ou nan fait ignorer la forme, pas tomber l'import
    let ligne = |x: &str| format!(r#"<Line startX="{x}" startY="0" endX="1" endY="0"><LineDesc lineWidth="0.2"/></Line>"#);
    for mauvais in ["inf", "nan", "1e400"] {
        let arc = format!(
            r#"<Arc startX="0" startY="0" endX="2" endY="0" centerX="{mauvais}" centerY="0" clockwise="true"><LineDesc lineWidth="0.2"/></Arc>"#
        );
        let m = modele(
            &mini(&format!(
                r#"
    <LayerFeature layerRef="TOP"><Set net="A"><Features>{}{}{arc}</Features></Set></LayerFeature>"#,
                ligne("0"),
                ligne(mauvais)
            )),
            "mini.xml",
        );
        assert_eq!((m["stats"]["pistes"].as_i64(), m["stats"]["arcs"].as_i64()), (Some(1), Some(0)), "{mauvais}");
    }
    // une hauteur non finie (1e400) ne fait pas tomber la lecture
    let m = modele(&carte().replace(r#"type="CHIP" height="0.45""#, r#"type="CHIP" height="1e400""#), "inf.xml");
    assert!(m["composants"][0].get("hauteur").is_none());
    // un débordement né d'un calcul (un rayon de 1e308) levait, hors de toute IPC2581ParseError
    let deborde = mini(
        r#"<Profile><Polygon><PolyBegin x="1e308" y="0"/><PolyStepCurve x="0" y="1e308" centerX="-1e308" centerY="-1e308"/><PolyStepSegment x="0" y="0"/></Polygon></Profile>"#,
    );
    assert_eq!(
        ipc2581_en_dict(deborde.as_bytes(), "mini.xml", true),
        Err(Erreur::Parse(IPC2581ParseError(
            "Erreur lors du parsing de mini.xml : cannot convert float infinity to integer".into()
        )))
    );
}
