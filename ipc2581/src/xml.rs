//! Le XML tel qu'expat le jugeait pour le lecteur Python : l'encodage du fichier, puis un
//! contrôle de bonne forme qui rend, à la première faute, le message d'expat et sa position
//! (« mismatched tag: line 12, column 4 »). L'arbre lui-même est bâti par roxmltree, qui ne
//! s'arrête ni aux mêmes fautes ni au même caractère : sans ce contrôle, un fichier abîmé
//! n'était plus refusé dans les mêmes mots, et une déclaration <?xml?> fautive passait.
//!
//! Ce qui n'est pas reproduit (voir `verifier`) : l'intérieur d'un <!DOCTYPE>, et les
//! classes de caractères d'expat pour un nom de balise hors ASCII.

/// Ce que la lecture du XML levait en Python.
pub(crate) enum Refus {
    /// ET.ParseError : le message d'expat (« XML invalide ou corrompu : fichier (message) »).
    Xml(String),
    /// Une autre exception (« Erreur lors du parsing de fichier : message »).
    Autre(String),
}

type Res<T> = Result<T, (&'static str, usize)>;

const INVALIDE: &str = "not well-formed (invalid token)";
// Un caractère qu'XML n'admet pas, là où expat regarde le caractère lui-même : la même faute,
// sauf pour celui que la fin du fichier coupe (« partial character »).
const CARACTERE: &str = "caractère";
const NON_CLOS: &str = "unclosed token";
const NS_XML: &str = "http://www.w3.org/XML/1998/namespace";
const NS_XMLNS: &str = "http://www.w3.org/2000/xmlns/";

// Un octet (ou une suite) que l'encodage ne sait pas lire : expat s'y arrêtait par « invalid
// token », à sa place dans le fichier. Il est rendu par ce non-caractère, que le contrôle
// refuse là où il le rencontre.
const ILLISIBLE: char = '\u{FFFF}';

/// « line L, column C » d'expat pour un rang d'octet du texte : la ligne compte de 1, la
/// colonne de 0 et en caractères ; « \r », « \n » et « \r\n » finissent chacun une ligne.
fn position(t: &str, rang: usize) -> String {
    let (mut ligne, mut colonne, mut apres_cr) = (1, 0, false);
    for c in t[..rang.min(t.len())].chars() {
        match c {
            '\r' => (ligne, colonne) = (ligne + 1, 0),
            '\n' if apres_cr => colonne = 0,
            '\n' => (ligne, colonne) = (ligne + 1, 0),
            _ => colonne += 1,
        }
        apres_cr = c == '\r';
    }
    format!("line {ligne}, column {colonne}")
}

fn espace(o: u8) -> bool {
    matches!(o, b' ' | b'\t' | b'\r' | b'\n')
}

// ----------------------------------------------------------------------
// L'encodage
// ----------------------------------------------------------------------

/// Le texte d'un fichier, tel qu'expat le parcourait.
pub(crate) struct Texte {
    /// Tout le fichier décodé, marque d'ordre comprise : expat la compte dans ses colonnes.
    pub xml: String,
    /// Le rang où le document commence, après la marque d'ordre.
    pub debut: usize,
    /// Le rang, dans `xml`, d'un caractère UTF-8 que la fin du fichier coupe.
    partiel: Option<usize>,
}

/// Les octets du fichier en texte, selon la marque d'ordre ou l'encodage déclaré.
/// Sont lus : UTF-8, UTF-16, ISO-8859-1, US-ASCII, windows-1252, ISO-8859-15.
/// shortcut: les autres encodages 8 bits que Python sait lire (cp437, iso-8859-2, koi8-r...)
/// sont refusés par « unknown encoding » ; à reprendre si un fichier en porte.
pub(crate) fn decoder(octets: &[u8]) -> Result<Texte, Refus> {
    let mal_formee = |t: &str, rang: usize| Refus::Xml(format!("XML declaration not well-formed: {}", position(t, rang)));
    let incorrect =
        |t: &str, rang: usize| Refus::Xml(format!("encoding specified in XML declaration is incorrect: {}", position(t, rang)));
    // UTF-16 : par sa marque d'ordre, ou par un « < » sur deux octets
    let gros = match octets {
        [0xFF, 0xFE, ..] | [0x3C, 0x00, ..] => Some(false),
        [0xFE, 0xFF, ..] | [0x00, 0x3C, ..] => Some(true),
        _ => None,
    };
    if let Some(gros) = gros {
        let unites = octets.chunks(2).map(|p| match p {
            [a, b] if gros => u16::from_be_bytes([*a, *b]),
            [a, b] => u16::from_le_bytes([*a, *b]),
            _ => 0xFFFF,
        });
        let xml: String = char::decode_utf16(unites).map(|c| c.unwrap_or(ILLISIBLE)).collect();
        let debut = if xml.starts_with('\u{FEFF}') { 3 } else { 0 };
        let corps = &xml.as_bytes()[debut..];
        if let Some(fin) = fin_declaration(corps).filter(|fin| xml[debut..debut + fin].chars().all(valide)) {
            match declaration(corps, fin) {
                Err(rang) => return Err(mal_formee(&xml, debut + rang)),
                Ok(Some((nom, rang))) => {
                    // un encodage qu'expat connaît et qui n'est pas celui du fichier
                    let exact = nom.to_ascii_uppercase();
                    if ["UTF-8", "ISO-8859-1", "US-ASCII"].contains(&exact.as_str()) {
                        return Err(incorrect(&xml, debut + rang));
                    }
                    codec(&nom)?;
                }
                Ok(None) => {}
            }
        }
        return Ok(Texte { xml, debut, partiel: None });
    }
    let marque = if octets.starts_with(&[0xEF, 0xBB, 0xBF]) { 3 } else { 0 };
    // la déclaration <?xml ... ?>, si elle ouvre le fichier et qu'elle est fermée (sinon le
    // contrôle dira « unclosed token »)
    let corps = &octets[marque..];
    let Some(fin) = fin_declaration(corps) else { return Ok(utf8(octets, marque)) };
    // un caractère qu'XML n'admet pas dans la déclaration : le contrôle le dira, à sa place
    if !std::str::from_utf8(&corps[..fin]).is_ok_and(|t| t.chars().all(valide)) {
        return Ok(utf8(octets, marque));
    }
    let en_tete = String::from_utf8_lossy(&octets[..marque + fin]);
    let (nom, rang) = match declaration(corps, fin) {
        Err(rang) => return Err(mal_formee(&en_tete, marque + rang)),
        Ok(None) => return Ok(utf8(octets, marque)),
        Ok(Some(encodage)) => encodage,
    };
    // Un encodage d'un octet par caractère : la marque d'ordre UTF-8, s'il y en a une, y fait
    // trois caractères -- et expat les compte ainsi dans ses colonnes.
    match codec(&nom)? {
        None if nom.to_ascii_uppercase().starts_with("UTF-16") => Err(incorrect(&en_tete, marque + rang)),
        None => Ok(utf8(octets, marque)),
        Some(table) => {
            let xml: String = octets.iter().map(|o| table(*o)).collect();
            let debut = xml.char_indices().nth(marque).map_or(xml.len(), |(i, _)| i);
            Ok(Texte { xml, debut, partiel: None })
        }
    }
}

/// La table d'un encodage d'un octet par caractère, par son nom ; None pour UTF-8 et UTF-16,
/// qu'expat lit lui-même. Un nom inconnu levait LookupError dans le lecteur Python.
fn codec(nom: &str) -> Result<Option<fn(u8) -> char>, Refus> {
    fn latin1(o: u8) -> char {
        o as char
    }
    fn ascii(o: u8) -> char {
        if o.is_ascii() {
            o as char
        } else {
            ILLISIBLE
        }
    }
    // les encodages qu'expat connaît lui-même, par leur nom exact
    match nom.to_ascii_uppercase().as_str() {
        "UTF-8" | "UTF-16" | "UTF-16BE" | "UTF-16LE" => return Ok(None),
        "ISO-8859-1" => return Ok(Some(latin1)),
        "US-ASCII" => return Ok(Some(ascii)),
        _ => {}
    }
    // les autres passaient par les codecs de Python, qui normalisent le nom
    // (minuscules, « - » et « _ » confondus, répétés ou au bord : « UTF- » est « utf »)
    let normal = nom.to_ascii_lowercase().split(['-', '_']).filter(|m| !m.is_empty()).collect::<Vec<_>>().join("_");
    match normal.as_str() {
        "latin_1" | "latin1" | "iso_8859_1" | "iso8859_1" | "8859" | "cp819" | "latin" | "l1" | "iso_ir_100"
        | "ibm819" | "iso8859" => Ok(Some(latin1)),
        // UTF-8 sous un autre nom que « UTF-8 » : Python donnait à expat une table d'un octet
        // par caractère, où tout octet au-delà de l'ASCII est illisible
        "ascii" | "us_ascii" | "646" | "iso646_us" | "us" | "cp367" | "ibm367" | "iso_ir_6" | "utf_8" | "utf8"
        | "u8" | "utf" | "cp65001" => Ok(Some(ascii)),
        "cp1252" | "windows_1252" | "1252" => Ok(Some(cp1252)),
        "iso8859_15" | "iso_8859_15" | "latin9" | "latin_9" | "l9" | "8859_15" => Ok(Some(latin9)),
        c if ["utf_16", "utf16", "u16", "utf_32", "utf32", "u32", "utf_7", "utf7", "big5", "gbk", "gb2312", "gb18030",
            "shift_jis", "sjis", "euc_jp", "euc_kr", "cp932", "cp936", "cp949", "cp950"]
            .iter()
            .any(|m| c.starts_with(m)) =>
        {
            Err(Refus::Autre("multi-byte encodings are not supported".into()))
        }
        _ => Err(Refus::Autre(format!("unknown encoding: {nom}"))),
    }
}

fn utf8(octets: &[u8], debut: usize) -> Texte {
    let mut xml = String::with_capacity(octets.len());
    let mut reste = octets;
    let mut partiel = None;
    loop {
        match std::str::from_utf8(reste) {
            Ok(t) => {
                xml.push_str(t);
                return Texte { xml, debut, partiel };
            }
            Err(e) => {
                let (bon, suite) = reste.split_at(e.valid_up_to());
                xml.push_str(std::str::from_utf8(bon).unwrap_or(""));
                // un caractère annoncé plus long que ce qui reste du fichier : expat le dit
                // « coupé », quels que soient les octets qui le suivent
                let attendu = match suite[0] {
                    0xC2..=0xDF => 2,
                    0xE0..=0xEF => 3,
                    0xF0..=0xF4 => 4,
                    _ => 1,
                };
                if suite.len() < attendu && partiel.is_none() {
                    partiel = Some(xml.len());
                }
                xml.push(ILLISIBLE);
                reste = &suite[e.error_len().unwrap_or(suite.len())..];
            }
        }
    }
}

fn cp1252(o: u8) -> char {
    const HAUT: [char; 32] = [
        '€', ILLISIBLE, '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', ILLISIBLE, 'Ž', ILLISIBLE, ILLISIBLE,
        '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', ILLISIBLE, 'ž', 'Ÿ',
    ];
    if (0x80..0xA0).contains(&o) {
        HAUT[(o - 0x80) as usize]
    } else {
        o as char
    }
}

fn latin9(o: u8) -> char {
    match o {
        0xA4 => '€',
        0xA6 => 'Š',
        0xA8 => 'š',
        0xB4 => 'Ž',
        0xB8 => 'ž',
        0xBC => 'Œ',
        0xBD => 'œ',
        0xBE => 'Ÿ',
        _ => o as char,
    }
}

/// Le texte s'ouvre-t-il sur la déclaration : « <?xml » puis un blanc, ou « ?> » aussitôt ?
/// (« <?xml-stylesheet » et « <?xml? » sont des instructions ordinaires, que le contrôle juge.)
fn ouvre_declaration(o: &[u8]) -> bool {
    o.starts_with(b"<?xml") && (o.get(5).is_none_or(|c| espace(*c)) || o[5..].starts_with(b"?>")) && o.len() > 5
}

/// Le rang du « ?> » qui ferme la déclaration <?xml ... ?> ouvrant le fichier, s'il y en a une.
fn fin_declaration(o: &[u8]) -> Option<usize> {
    if !ouvre_declaration(o) {
        return None;
    }
    o.windows(2).position(|p| p == b"?>")
}

/// Un pseudo-attribut de la déclaration : (nom, fin du nom, valeur, suite), None s'il n'y en
/// a plus ; en erreur, le rang de la faute. C'est parsePseudoAttribute d'expat.
#[allow(clippy::type_complexity)]
fn pseudo_attribut(o: &[u8], mut p: usize, fin: usize) -> Result<Option<(usize, usize, usize, usize)>, usize> {
    let c = |i: usize| if i < fin && o[i].is_ascii() { o[i] } else { 0 }; // 0 : la fin, ou hors ASCII
    if p == fin {
        return Ok(None);
    }
    if !espace(c(p)) {
        return Err(p);
    }
    while espace(c(p)) {
        p += 1;
    }
    if p == fin {
        return Ok(None);
    }
    let nom = p;
    let fin_nom;
    loop {
        match c(p) {
            0 => return Err(p),
            b'=' => {
                fin_nom = p;
                break;
            }
            b if espace(b) => {
                fin_nom = p;
                while espace(c(p)) {
                    p += 1;
                }
                if c(p) != b'=' {
                    return Err(p);
                }
                break;
            }
            _ => p += 1,
        }
    }
    if p == nom {
        return Err(p);
    }
    p += 1;
    while espace(c(p)) {
        p += 1;
    }
    let guillemet = c(p);
    if guillemet != b'"' && guillemet != b'\'' {
        return Err(p);
    }
    p += 1;
    let valeur = p;
    while c(p) != guillemet {
        if !(c(p).is_ascii_alphanumeric() || matches!(c(p), b'.' | b'-' | b'_')) {
            return Err(p);
        }
        p += 1;
    }
    Ok(Some((nom, fin_nom, valeur, p + 1)))
}

/// La déclaration <?xml version encoding? standalone? ?> : l'encodage déclaré et le rang de
/// sa valeur, ou le rang de la faute (doParseXmlDecl d'expat).
fn declaration(o: &[u8], fin: usize) -> Result<Option<(String, usize)>, usize> {
    let Some((mut nom, mut fin_nom, mut valeur, mut p)) = pseudo_attribut(o, 5, fin)? else { return Err(5) };
    if &o[nom..fin_nom] != b"version" {
        return Err(nom);
    }
    // la version : « 1. » suivi de chiffres
    let v = &o[valeur..p - 1];
    if !(v.len() > 2 && v.starts_with(b"1.") && v[2..].iter().all(u8::is_ascii_digit)) {
        return Err(valeur);
    }
    let Some(suivant) = pseudo_attribut(o, p, fin)? else { return Ok(None) };
    (nom, fin_nom, valeur, p) = suivant;
    let mut encodage = None;
    if &o[nom..fin_nom] == b"encoding" {
        if !o[valeur].is_ascii_alphabetic() {
            return Err(valeur);
        }
        encodage = Some((String::from_utf8_lossy(&o[valeur..p - 1]).into_owned(), valeur));
        let Some(suivant) = pseudo_attribut(o, p, fin)? else { return Ok(encodage) };
        (nom, fin_nom, valeur, p) = suivant;
    }
    if &o[nom..fin_nom] != b"standalone" {
        return Err(nom);
    }
    if !matches!(&o[valeur..p - 1], b"yes" | b"no") {
        return Err(valeur);
    }
    while p < fin && espace(o[p]) {
        p += 1;
    }
    if p != fin {
        return Err(p);
    }
    Ok(encodage)
}

// ----------------------------------------------------------------------
// La bonne forme
// ----------------------------------------------------------------------

/// Un caractère qu'XML admet (et qu'expat lit) : pas de caractère de commande hors
/// tabulation et fins de ligne, pas de non-caractère U+FFFE / U+FFFF.
fn valide(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r') || (c >= ' ' && c != '\u{FFFE}' && c != '\u{FFFF}')
}

/// Premier caractère d'un nom. shortcut: hors ASCII, les classes sont celles d'XML 1.0
/// 5e édition ; expat suit la 4e, plus étroite. Sans effet sur les balises d'IPC-2581.
fn debut_nom(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || matches!(c, '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{2FF}' | '\u{370}'..='\u{37D}'
            | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}'
            | '\u{3001}'..='\u{D7FF}')
}

fn suite_nom(c: char) -> bool {
    debut_nom(c)
        || c.is_ascii_digit()
        || matches!(c, '-' | '.' | '\u{B7}' | '\u{300}'..='\u{36F}' | '\u{203F}'..='\u{2040}')
}

/// Un jeton du prologue : un nom, un littéral entre guillemets, un signe, ou autre chose.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Jeton {
    Nom,
    Litteral,
    Signe(char),
    Autre,
}

struct Controle<'a> {
    t: &'a str,
    o: &'a [u8],
    i: usize,
    /// Un <!DOCTYPE> a été vu : les entités qu'il a pu déclarer ne sont pas connues d'ici.
    dtd: bool,
    /// Les préfixes d'espace de noms liés, du plus ancien au plus récent.
    liaisons: Vec<(&'a str, &'a str)>,
    /// Le rang d'un caractère UTF-8 que la fin du fichier coupe.
    partiel: Option<usize>,
    /// Le rang du jeton en cours de lecture.
    jeton: usize,
}

/// Contrôle la bonne forme du document comme expat ; en erreur, son message et sa position.
///
/// shortcut: les déclarations d'un <!DOCTYPE [...]> sont sautées sans être jugées, et les
/// références d'entité d'un document qui porte un <!DOCTYPE> sont laissées à roxmltree.
pub(crate) fn verifier(texte: &Texte) -> Result<(), String> {
    let xml = texte.xml.as_str();
    let mut c = Controle {
        t: xml,
        o: xml.as_bytes(),
        i: texte.debut,
        dtd: false,
        liaisons: vec![("xml", NS_XML)],
        partiel: texte.partiel,
        jeton: texte.debut,
    };
    c.document().map_err(|(message, rang)| {
        // le caractère coupé en bout de fichier : expat le dit, au début du jeton qui le porte
        match message {
            CARACTERE if c.partiel == Some(rang) => format!("partial character: {}", position(xml, c.jeton)),
            CARACTERE => format!("{INVALIDE}: {}", position(xml, rang)),
            _ => format!("{message}: {}", position(xml, rang)),
        }
    })
}

impl<'a> Controle<'a> {
    fn car(&self, i: usize) -> Option<char> {
        self.t.get(i..).and_then(|s| s.chars().next())
    }

    fn blancs(&self, mut i: usize) -> usize {
        while self.o.get(i).is_some_and(|o| espace(*o)) {
            i += 1;
        }
        i
    }

    /// Le rang qui suit les caractères de nom à partir de `i`.
    fn fin_nom(&self, mut i: usize, deux_points: bool) -> usize {
        while let Some(c) = self.car(i) {
            if !(suite_nom(c) || (deux_points && c == ':')) {
                break;
            }
            i += c.len_utf8();
        }
        i
    }

    /// Avance jusqu'à `fin` (exclu) en refusant tout caractère qu'XML n'admet pas.
    fn texte(&self, debut: usize, fin: usize) -> Res<()> {
        match self.t[debut..fin].char_indices().find(|(_, c)| !valide(*c)) {
            Some((k, _)) => Err((CARACTERE, debut + k)),
            None => Ok(()),
        }
    }

    fn document(&mut self) -> Res<()> {
        let fin = self.o.len();
        // la déclaration, déjà jugée par `decoder` quand elle est fermée
        let debut = self.i;
        self.jeton = debut;
        if ouvre_declaration(&self.o[debut..]) {
            match self.t[debut..].find("?>") {
                Some(k) => {
                    self.texte(debut, debut + k)?;
                    self.i = debut + k + 2;
                }
                None => {
                    self.texte(debut, fin)?;
                    return Err((NON_CLOS, debut));
                }
            }
        }
        // le prologue, jusqu'à la balise racine
        loop {
            self.i = self.blancs(self.i);
            let d = self.i;
            self.jeton = d;
            match (self.o.get(d), self.o.get(d + 1)) {
                (None, _) => return Err(("no element found", fin)),
                (Some(b'<'), None) => return Err((NON_CLOS, d)),
                (Some(b'<'), Some(b'?')) => self.instruction(d)?,
                (Some(b'<'), Some(b'!')) if self.o.get(d + 2).is_none_or(|o| *o == b'-') => self.commentaire(d)?,
                (Some(b'<'), Some(b'!')) => {
                    let j = self.declaration_ouverte(d)?;
                    if &self.t[d..j] != "<!DOCTYPE" || self.dtd {
                        return Err(("syntax error", d));
                    }
                    self.doctype(j)?;
                }
                (Some(b'<'), Some(o)) if !o.is_ascii() || debut_nom(*o as char) => break,
                (Some(b'<'), _) => return Err((INVALIDE, d + 1)),
                _ => return Err(self.jeton(d).err().unwrap_or(("syntax error", d))),
            }
        }
        // le contenu
        let mut pile: Vec<(&str, usize)> = Vec::new();
        loop {
            let d = self.i;
            self.jeton = d;
            match (self.o.get(d), self.o.get(d + 1)) {
                // une fin de ligne « \r » en bout de fichier attend sa suite : expat reste devant
                (None, _) => return Err(("no element found", fin - self.t.ends_with('\r') as usize)),
                (Some(b'<'), None) => return Err((NON_CLOS, d)),
                (Some(b'<'), Some(b'/')) => {
                    let nom = self.fermante(d)?;
                    match pile.pop() {
                        Some((ouvert, liaisons)) if ouvert == nom => self.liaisons.truncate(liaisons),
                        _ => return Err(("mismatched tag", d + 2)), // expat désigne le nom
                    }
                }
                (Some(b'<'), Some(b'?')) => self.instruction(d)?,
                (Some(b'<'), Some(b'!')) => match self.o.get(d + 2) {
                    Some(b'[') => self.cdata(d)?,
                    None | Some(b'-') => self.commentaire(d)?,
                    _ => return Err((INVALIDE, d + 2)),
                },
                (Some(b'<'), _) if self.car(d + 1).is_some_and(debut_nom) => {
                    let liaisons = self.liaisons.len();
                    let (nom, vide) = self.ouvrante(d)?;
                    if vide {
                        self.liaisons.truncate(liaisons);
                    } else {
                        pile.push((nom, liaisons));
                    }
                }
                (Some(b'<'), _) => return Err((INVALIDE, d + 1)),
                (Some(b'&'), _) => {
                    self.i = self.reference(d, d)?;
                    self.entite(d, d)?;
                }
                (Some(b']'), _) if self.t[d..].starts_with("]]>") => return Err((INVALIDE, d + 2)),
                _ => {
                    let c = self.car(d).unwrap_or(ILLISIBLE);
                    if !valide(c) {
                        return Err((CARACTERE, d));
                    }
                    self.i = d + c.len_utf8();
                }
            }
            if pile.is_empty() {
                break;
            }
        }
        // l'épilogue : des blancs, des commentaires, des instructions, et rien d'autre
        let reste = "junk after document element";
        loop {
            self.i = self.blancs(self.i);
            let d = self.i;
            self.jeton = d;
            match (self.o.get(d), self.o.get(d + 1)) {
                (None, _) => return Ok(()),
                (Some(b'<'), None) => return Err((NON_CLOS, d)),
                (Some(b'<'), Some(b'?')) => self.instruction(d)?,
                (Some(b'<'), Some(b'!')) if self.o.get(d + 2).is_none_or(|o| *o == b'-') => self.commentaire(d)?,
                (Some(b'<'), Some(b'!')) => {
                    self.declaration_ouverte(d)?;
                    return Err((reste, d));
                }
                (Some(b'<'), Some(o)) if !o.is_ascii() || debut_nom(*o as char) => return Err((reste, d)),
                (Some(b'<'), _) => return Err((INVALIDE, d + 1)),
                _ => return Err(self.jeton(d).err().unwrap_or((reste, d))),
            }
        }
    }

    /// « <!MOT » avant ou après la racine : des lettres, puis un blanc ou « % ». Rend le rang
    /// qui suit le mot (scanDecl d'expat).
    fn declaration_ouverte(&self, d: usize) -> Res<usize> {
        let mut j = d + 2;
        match self.o.get(j) {
            None => return Err((NON_CLOS, d)),
            Some(b'[') => return Ok(j + 1),
            Some(o) if o.is_ascii_alphabetic() || *o == b'_' => {}
            Some(_) => return Err((INVALIDE, j)),
        }
        loop {
            j += 1;
            match self.o.get(j) {
                None => return Err((NON_CLOS, d)),
                Some(o) if o.is_ascii_alphabetic() || *o == b'_' => {}
                Some(o) if espace(*o) => return Ok(j),
                Some(b'%') => {
                    return match self.o.get(j + 1) {
                        None => Err((NON_CLOS, d)),
                        Some(o) if espace(*o) || *o == b'%' => Err((INVALIDE, j)),
                        Some(_) => Ok(j),
                    };
                }
                Some(_) => return Err((INVALIDE, j)),
            }
        }
    }

    /// Un jeton du prologue à partir de `d` -- un mot, un littéral, un signe de DTD -- et le
    /// rang qui le suit ; en erreur, la faute que le découpage d'expat y trouve (prologTok).
    /// `d` n'est ni un blanc ni un « < ».
    fn jeton(&self, d: usize) -> Res<(Jeton, usize)> {
        let Some(c) = self.car(d) else { return Err((NON_CLOS, d)) };
        let suivant = |j: usize| self.o.get(j).copied();
        match c {
            '"' | '\'' => {
                let Some(k) = self.t[d + 1..].find(c) else {
                    self.texte(d + 1, self.o.len())?;
                    return Err((NON_CLOS, d));
                };
                self.texte(d + 1, d + 1 + k)?;
                let j = d + 1 + k + 1;
                match suivant(j) {
                    None => Ok((Jeton::Litteral, j)),
                    Some(o) if espace(o) || matches!(o, b'>' | b'%' | b'[') => Ok((Jeton::Litteral, j)),
                    Some(_) => Err((INVALIDE, j)),
                }
            }
            '%' => match self.car(d + 1) {
                None => Err((NON_CLOS, d)),
                Some(s) if s.is_ascii() && (espace(s as u8) || s == '%') => Ok((Jeton::Signe('%'), d + 1)),
                Some(s) if debut_nom(s) => {
                    let j = self.fin_nom(d + 1, false);
                    match suivant(j) {
                        None => Err((NON_CLOS, d)),
                        Some(b';') => Ok((Jeton::Signe('%'), j + 1)),
                        Some(_) => Err((INVALIDE, j)),
                    }
                }
                Some(_) => Err((INVALIDE, d + 1)),
            },
            '#' => match self.car(d + 1) {
                None => Err((NON_CLOS, d)),
                Some(s) if debut_nom(s) => {
                    let j = self.fin_nom(d + 1, false);
                    match suivant(j) {
                        None => Ok((Jeton::Signe('#'), j)),
                        Some(o) if espace(o) || matches!(o, b')' | b'>' | b'%' | b'|') => Ok((Jeton::Signe('#'), j)),
                        Some(_) => Err((INVALIDE, j)),
                    }
                }
                Some(_) => Err((INVALIDE, d + 1)),
            },
            ')' => match suivant(d + 1) {
                None => Ok((Jeton::Signe(c), d + 1)),
                Some(b'*' | b'?' | b'+') => Ok((Jeton::Signe(c), d + 2)),
                Some(o) if espace(o) || matches!(o, b'>' | b',' | b'|' | b')') => Ok((Jeton::Signe(c), d + 1)),
                Some(_) => Err((INVALIDE, d + 1)),
            },
            '[' | ']' | '(' | '|' | ',' | '>' => Ok((Jeton::Signe(c), d + 1)),
            _ if suite_nom(c) || c == ':' => {
                // un nom (au plus un « : », suivi d'un caractère de nom), sinon un simple mot
                let mut nom = debut_nom(c);
                let mut deux_points = false;
                let mut j = d + c.len_utf8();
                loop {
                    match self.car(j) {
                        None => break,
                        Some(s) if suite_nom(s) => j += s.len_utf8(),
                        Some(':') => {
                            j += 1;
                            if nom && !deux_points {
                                deux_points = true;
                                match self.car(j) {
                                    None => return Err((NON_CLOS, d)),
                                    Some(s) if suite_nom(s) => {}
                                    Some(_) => nom = false,
                                }
                            } else {
                                nom = false;
                            }
                        }
                        Some(s) if s.is_ascii() && (espace(s as u8) || ">),|[%".contains(s)) => break,
                        Some('?' | '*' | '+') if nom => return Ok((Jeton::Autre, j + 1)),
                        Some(_) => return Err((INVALIDE, j)),
                    }
                }
                Ok((if nom { Jeton::Nom } else { Jeton::Autre }, j))
            }
            _ if !valide(c) => Err((CARACTERE, d)),
            _ => Err((INVALIDE, d)),
        }
    }

    /// Le jeton qui suit les blancs à partir de `i`, pour le <!DOCTYPE> : (genre, début, fin).
    /// La fin du fichier y est « no element found ».
    fn jeton_doctype(&self, i: usize) -> Res<(Jeton, usize, usize)> {
        let d = self.blancs(i);
        match self.o.get(d) {
            None => Err(("no element found", self.o.len())),
            Some(b'<') => Err((INVALIDE, d)),
            Some(_) => self.jeton(d).map(|(genre, fin)| (genre, d, fin)),
        }
    }

    /// <!DOCTYPE nom (SYSTEM "..." | PUBLIC "..." "...")? ([ ... ])? > ; `i` suit « <!DOCTYPE ».
    fn doctype(&mut self, i: usize) -> Res<()> {
        self.dtd = true;
        let faute = |d: usize| Err(("syntax error", d));
        let (genre, d, mut j) = self.jeton_doctype(i)?;
        if genre != Jeton::Nom {
            return faute(d);
        }
        let (mut genre, mut d, mut fin) = self.jeton_doctype(j)?;
        if genre == Jeton::Nom {
            // l'identifiant externe : un littéral après SYSTEM, deux après PUBLIC
            let litteraux = match &self.t[d..fin] {
                "SYSTEM" => 1,
                "PUBLIC" => 2,
                _ => return faute(d),
            };
            j = fin;
            for _ in 0..litteraux {
                let (g, d, f) = self.jeton_doctype(j)?;
                if g != Jeton::Litteral {
                    return faute(d);
                }
                j = f;
            }
            (genre, d, fin) = self.jeton_doctype(j)?;
        }
        if genre == Jeton::Signe('[') {
            j = self.sous_ensemble(fin)?;
            (genre, d, fin) = self.jeton_doctype(j)?;
        }
        if genre != Jeton::Signe('>') {
            return faute(d);
        }
        self.i = fin;
        Ok(())
    }

    /// Le sous-ensemble interne d'un <!DOCTYPE>, de « [ » à « ] » : des déclarations, des
    /// commentaires, des instructions. Rend le rang qui suit « ] ».
    /// shortcut: l'intérieur d'une déclaration (<!ELEMENT ...>) est sauté jusqu'à son « > »,
    /// guillemets compris, sans être jugé.
    fn sous_ensemble(&mut self, i: usize) -> Res<usize> {
        let mut j = i;
        loop {
            j = self.blancs(j);
            match (self.o.get(j), self.o.get(j + 1)) {
                (None, _) => return Err(("no element found", self.o.len())),
                (Some(b']'), _) => return Ok(j + 1),
                (Some(b'<'), None) => return Err((NON_CLOS, j)),
                (Some(b'<'), Some(b'?')) => {
                    self.instruction(j)?;
                    j = self.i;
                }
                (Some(b'<'), Some(b'!')) if self.o.get(j + 2).is_none_or(|o| *o == b'-') => {
                    self.commentaire(j)?;
                    j = self.i;
                }
                (Some(b'<'), Some(b'!')) => {
                    let d = j;
                    j = self.declaration_ouverte(d)?;
                    loop {
                        match self.o.get(j) {
                            None => return Err(("no element found", self.o.len())),
                            Some(b'>') => break,
                            Some(g @ (b'"' | b'\'')) => match self.o[j + 1..].iter().position(|o| o == g) {
                                Some(k) => j += k + 1,
                                None => return Err((NON_CLOS, j)),
                            },
                            Some(_) => {}
                        }
                        j += 1;
                    }
                    j += 1;
                }
                (Some(b'<'), _) => return Err((INVALIDE, j + 1)),
                (Some(b'%'), _) => j = self.jeton(j)?.1,
                _ => return Err(self.jeton(j).err().unwrap_or(("syntax error", j))),
            }
        }
    }

    /// <!-- ... --> ; `d` est le rang de « < », suivi de « ! ».
    fn commentaire(&mut self, d: usize) -> Res<()> {
        for k in [2, 3] {
            match self.o.get(d + k) {
                None => return Err((NON_CLOS, d)),
                Some(b'-') => {}
                Some(_) => return Err((INVALIDE, d + k)),
            }
        }
        let mut j = d + 4;
        loop {
            let Some(k) = self.t[j..].find("--") else {
                self.texte(j, self.o.len())?;
                return Err((NON_CLOS, d));
            };
            self.texte(j, j + k)?;
            j += k + 2;
            match self.o.get(j) {
                None => return Err((NON_CLOS, d)),
                Some(b'>') => {
                    self.i = j + 1;
                    return Ok(());
                }
                Some(_) => return Err((INVALIDE, j)),
            }
        }
    }

    /// <![CDATA[ ... ]]>
    fn cdata(&mut self, d: usize) -> Res<()> {
        let fin = self.o.len();
        if fin < d + 9 {
            return Err((NON_CLOS, d));
        }
        if let Some(k) = (0..6).find(|k| self.o[d + 3 + k] != b"CDATA["[*k]) {
            return Err((INVALIDE, d + 3 + k));
        }
        match self.t[d + 9..].find("]]>") {
            Some(k) => {
                self.texte(d + 9, d + 9 + k)?;
                self.i = d + 9 + k + 3;
                Ok(())
            }
            None => {
                self.texte(d + 9, fin)?;
                // un ou deux « ] » en bout de fichier attendent leur suite : expat reste devant
                let attente = self.o[d + 9..].iter().rev().take(2).take_while(|o| **o == b']').count();
                Err(("unclosed CDATA section", fin - attente))
            }
        }
    }

    /// <?cible ... ?>
    fn instruction(&mut self, d: usize) -> Res<()> {
        match self.car(d + 2) {
            None => return Err((NON_CLOS, d)),
            Some(c) if debut_nom(c) => {}
            Some(_) => return Err((INVALIDE, d + 2)),
        }
        let j = self.fin_nom(d + 2, false);
        let cible = &self.t[d + 2..j];
        let reservee = cible.eq_ignore_ascii_case("xml");
        match self.o.get(j) {
            None => Err((NON_CLOS, d)),
            Some(o) if espace(*o) || *o == b'?' => {
                if reservee && cible != "xml" {
                    return Err((INVALIDE, j));
                }
                if *o == b'?' {
                    match self.o.get(j + 1) {
                        None => return Err((NON_CLOS, d)),
                        Some(b'>') => {}
                        Some(_) => return Err((INVALIDE, j + 1)),
                    }
                }
                let Some(k) = self.t[j..].find("?>") else {
                    self.texte(j, self.o.len())?;
                    return Err((NON_CLOS, d));
                };
                self.texte(j, j + k)?;
                if reservee {
                    return Err(("XML or text declaration not at start of entity", d));
                }
                self.i = j + k + 2;
                Ok(())
            }
            Some(_) => Err((INVALIDE, j)),
        }
    }

    /// Un nom qualifié à partir de `i` (dont le premier caractère est déjà jugé) : au plus un
    /// « : », suivi d'un début de nom. Rend le rang qui le suit ; `d` est le début de la balise.
    fn nom_qualifie(&self, i: usize, d: usize) -> Res<usize> {
        let mut j = self.fin_nom(i, false);
        if self.o.get(j) == Some(&b':') {
            match self.car(j + 1) {
                None => return Err((NON_CLOS, d)),
                Some(c) if debut_nom(c) => {}
                Some(_) => return Err((INVALIDE, j + 1)),
            }
            j = self.fin_nom(j + 1, false);
            if self.o.get(j) == Some(&b':') {
                return Err((INVALIDE, j));
            }
        }
        Ok(j)
    }

    /// Une référence « &...; » à partir du rang `e` : sa forme seulement. Rend le rang qui
    /// la suit ; `d` est le début du jeton qui la porte (elle-même, ou la balise).
    fn reference(&self, e: usize, d: usize) -> Res<usize> {
        let fin = |j: usize| match self.o.get(j) {
            None => Err((NON_CLOS, d)),
            Some(b';') => Ok(j + 1),
            Some(_) => Err((INVALIDE, j)),
        };
        let chiffres = |mut j: usize, hexa: bool| {
            let bon = |o: &u8| if hexa { o.is_ascii_hexdigit() } else { o.is_ascii_digit() };
            match self.o.get(j) {
                None => return Err((NON_CLOS, d)),
                Some(o) if bon(o) => {}
                Some(_) => return Err((INVALIDE, j)),
            }
            while self.o.get(j).is_some_and(bon) {
                j += 1;
            }
            fin(j)
        };
        match (self.o.get(e + 1), self.o.get(e + 2)) {
            (None, _) | (Some(b'#'), None) => Err((NON_CLOS, d)),
            (Some(b'#'), Some(b'x')) => chiffres(e + 3, true),
            (Some(b'#'), _) => chiffres(e + 2, false),
            _ if self.car(e + 1).is_some_and(debut_nom) => fin(self.fin_nom(e + 1, false)),
            _ => Err((INVALIDE, e + 1)),
        }
    }

    /// Le sens d'une référence bien formée au rang `e` : un numéro de caractère qu'XML admet,
    /// une entité connue. `d` : où expat signale une entité inconnue.
    fn entite(&self, e: usize, d: usize) -> Res<()> {
        let fin = e + self.t[e..].find(';').unwrap_or(0);
        let corps = &self.t[e + 1..fin];
        if let Some(numero) = corps.strip_prefix('#') {
            let n = match numero.strip_prefix('x') {
                Some(h) => u32::from_str_radix(h, 16),
                None => numero.parse::<u32>(),
            };
            return match n.ok().and_then(char::from_u32) {
                Some(c) if valide(c) => Ok(()),
                _ => Err(("reference to invalid character number", e)),
            };
        }
        if self.dtd || ["lt", "gt", "amp", "apos", "quot"].contains(&corps) {
            Ok(())
        } else {
            Err(("undefined entity", d))
        }
    }

    /// </nom> ; rend le nom.
    fn fermante(&mut self, d: usize) -> Res<&'a str> {
        match self.car(d + 2) {
            None => return Err((NON_CLOS, d)),
            Some(c) if debut_nom(c) => {}
            Some(_) => return Err((INVALIDE, d + 2)),
        }
        let j = self.fin_nom(d + 2, true);
        let k = self.blancs(j);
        match self.o.get(k) {
            None => Err((NON_CLOS, d)),
            Some(b'>') => {
                self.i = k + 1;
                Ok(&self.t[d + 2..j])
            }
            Some(_) => Err((INVALIDE, k)),
        }
    }

    /// <nom attr="valeur" ...> ou <nom .../> ; rend le nom et « la balise est vide ».
    fn ouvrante(&mut self, d: usize) -> Res<(&'a str, bool)> {
        let t = self.t;
        let mut j = self.nom_qualifie(d + 1, d)?;
        let nom = &t[d + 1..j];
        // (rang du nom, nom, valeur brute, rang de la valeur)
        let mut attributs: Vec<(usize, &str, &str, usize)> = Vec::new();
        let vide = loop {
            match self.o.get(j) {
                None => return Err((NON_CLOS, d)),
                Some(b'>') => {
                    j += 1;
                    break false;
                }
                Some(b'/') => match self.o.get(j + 1) {
                    None => return Err((NON_CLOS, d)),
                    Some(b'>') => {
                        j += 2;
                        break true;
                    }
                    Some(_) => return Err((INVALIDE, j + 1)),
                },
                Some(o) if espace(*o) => {
                    j = self.blancs(j);
                    match self.car(j) {
                        None => return Err((NON_CLOS, d)),
                        Some('>' | '/') => continue,
                        Some(c) if debut_nom(c) => {}
                        Some(_) => return Err((INVALIDE, j)),
                    }
                    let rang_nom = j;
                    j = self.nom_qualifie(j, d)?;
                    let nom_attribut = &t[rang_nom..j];
                    j = self.blancs(j);
                    match self.o.get(j) {
                        None => return Err((NON_CLOS, d)),
                        Some(b'=') => j = self.blancs(j + 1),
                        Some(_) => return Err((INVALIDE, j)),
                    }
                    let guillemet = match self.o.get(j) {
                        None => return Err((NON_CLOS, d)),
                        Some(g @ (b'"' | b'\'')) => *g,
                        Some(_) => return Err((INVALIDE, j)),
                    };
                    j += 1;
                    let rang_valeur = j;
                    loop {
                        match self.o.get(j) {
                            None => return Err((NON_CLOS, d)),
                            Some(g) if *g == guillemet => break,
                            Some(b'<') => return Err((INVALIDE, j)),
                            Some(b'&') => j = self.reference(j, d)?,
                            Some(_) => {
                                let c = self.car(j).unwrap_or(ILLISIBLE);
                                if !valide(c) {
                                    return Err((CARACTERE, j));
                                }
                                j += c.len_utf8();
                            }
                        }
                    }
                    attributs.push((rang_nom, nom_attribut, &t[rang_valeur..j], rang_valeur));
                    j += 1;
                    match self.o.get(j) {
                        None => return Err((NON_CLOS, d)),
                        Some(o) if espace(*o) || *o == b'/' || *o == b'>' => {}
                        Some(_) => return Err((INVALIDE, j)),
                    }
                }
                Some(_) => return Err((INVALIDE, j)),
            }
        };
        // la balise est bien formée : ses attributs, un par un, dans l'ordre d'expat
        for (k, (rang_nom, nom_attribut, valeur, rang_valeur)) in attributs.iter().enumerate() {
            if attributs[..k].iter().any(|a| a.1 == *nom_attribut) {
                return Err(("duplicate attribute", *rang_nom));
            }
            for (e, _) in valeur.match_indices('&') {
                self.entite(rang_valeur + e, d)?;
            }
            let prefixe = match nom_attribut.strip_prefix("xmlns") {
                Some("") => "",
                Some(p) if p.starts_with(':') => &p[1..],
                _ => continue,
            };
            if !prefixe.is_empty() {
                if valeur.is_empty() {
                    return Err(("must not undeclare prefix", d));
                }
                if prefixe == "xmlns" {
                    return Err(("reserved prefix (xmlns) must not be declared or undeclared", d));
                }
                if prefixe == "xml" && *valeur != NS_XML {
                    return Err(("reserved prefix (xml) must not be undeclared or bound to another namespace name", d));
                }
            }
            if (*valeur == NS_XML && prefixe != "xml") || *valeur == NS_XMLNS {
                return Err(("prefix must not be bound to one of the reserved namespace names", d));
            }
            self.liaisons.push((prefixe, valeur));
        }
        let espace_de = |prefixe: &str| self.liaisons.iter().rev().find(|l| l.0 == prefixe).map(|l| l.1);
        let mut etendus: Vec<(&str, &str)> = Vec::new();
        for (_, nom_attribut, _, _) in &attributs {
            let Some((prefixe, local)) = nom_attribut.split_once(':') else { continue };
            if prefixe == "xmlns" {
                continue;
            }
            let Some(uri) = espace_de(prefixe) else { return Err(("unbound prefix", d)) };
            if etendus.contains(&(uri, local)) {
                return Err(("duplicate attribute", d));
            }
            etendus.push((uri, local));
        }
        if let Some((prefixe, _)) = nom.split_once(':') {
            if espace_de(prefixe).is_none() {
                return Err(("unbound prefix", d));
            }
        }
        self.i = j;
        Ok((nom, vide))
    }
}

/// Le message d'expat le plus proche d'une erreur de roxmltree, pour un document que le
/// contrôle a laissé passer (un <!DOCTYPE> surtout) : le genre de la faute dans les mots
/// d'expat, la position du lecteur d'ici.
pub(crate) fn erreur_roxmltree(xml: &str, e: &roxmltree::Error) -> String {
    use roxmltree::Error as E;
    let p = e.pos();
    let ici = format!("line {}, column {}", p.row, p.col.saturating_sub(1));
    match e {
        E::NoRootNode | E::UnclosedRootNode | E::UnexpectedEndOfStream => {
            format!("no element found: {}", position(xml, xml.len()))
        }
        E::UnexpectedCloseTag(..) => format!("mismatched tag: {ici}"),
        E::UnknownNamespace(..) => format!("unbound prefix: {ici}"),
        E::DuplicatedAttribute(..) => format!("duplicate attribute: {ici}"),
        E::UnknownEntityReference(..) => format!("undefined entity: {ici}"),
        E::EntityReferenceLoop(..) => format!("recursive entity reference: {ici}"),
        _ => format!("{INVALIDE}: {ici}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expat(octets: &[u8]) -> String {
        match decoder(octets) {
            Err(Refus::Xml(m)) | Err(Refus::Autre(m)) => m,
            Ok(t) => verifier(&t).err().unwrap_or_else(|| "OK".into()),
        }
    }

    fn utf16(t: &str) -> Vec<u8> {
        [0xFF, 0xFE].into_iter().chain(t.encode_utf16().flat_map(u16::to_le_bytes)).collect()
    }

    /// Relevés sur Python 3.13 (xml.etree.ElementTree.fromstring), un par genre de faute.
    #[test]
    fn messages_d_expat() {
        for (xml, attendu) in [
            ("<a><b></a>", "mismatched tag: line 1, column 8"),
            ("<a>\n <b>\n</a>", "mismatched tag: line 3, column 2"),
            ("<a>\r\n<b>\r\n</a>", "mismatched tag: line 3, column 2"),
            ("<é>\n   é</b>", "mismatched tag: line 2, column 6"),
            ("</a>", "not well-formed (invalid token): line 1, column 1"),
            ("<a/></a>", "not well-formed (invalid token): line 1, column 5"),
            ("<a><b/>", "no element found: line 1, column 7"),
            ("<a b=\"x", "unclosed token: line 1, column 0"),
            ("<a><!-- x", "unclosed token: line 1, column 3"),
            ("<a><![CDATA[ x", "unclosed CDATA section: line 1, column 14"),
            ("<a>&amp", "unclosed token: line 1, column 3"),
            ("", "no element found: line 1, column 0"),
            ("<a/>x", "junk after document element: line 1, column 4"),
            ("<a/><b/>", "junk after document element: line 1, column 4"),
            ("<a/><!-- c -->\n", "OK"),
            ("x<a/>", "not well-formed (invalid token): line 1, column 1"),
            ("bonjour", "syntax error: line 1, column 0"),
            ("<a b=1/>", "not well-formed (invalid token): line 1, column 5"),
            ("<a b='1' b='2'/>", "duplicate attribute: line 1, column 9"),
            ("<a b=\"<\"/>", "not well-formed (invalid token): line 1, column 6"),
            ("<a b=\"&x;\"/>", "undefined entity: line 1, column 0"),
            ("<a>&x;</a>", "undefined entity: line 1, column 3"),
            ("<a b=\"&#0;\"/>", "reference to invalid character number: line 1, column 6"),
            ("<x:a/>", "unbound prefix: line 1, column 0"),
            ("<a xmlns:x=\"\"/>", "must not undeclare prefix: line 1, column 0"),
            ("<a xmlns:p=\"u\" xmlns:q=\"u\" p:c=\"1\" q:c=\"2\"/>", "duplicate attribute: line 1, column 0"),
            ("<a>\u{1}</a>", "not well-formed (invalid token): line 1, column 3"),
            ("<a>]]></a>", "not well-formed (invalid token): line 1, column 5"),
            ("<a><!-- a -- b --></a>", "not well-formed (invalid token): line 1, column 12"),
            ("<a><![CDAT[x]]></a>", "not well-formed (invalid token): line 1, column 10"),
            ("<a:b:c xmlns:a=\"u\"/>", "not well-formed (invalid token): line 1, column 4"),
            ("<a><?xml version=\"1.0\"?></a>", "XML or text declaration not at start of entity: line 1, column 3"),
            ("<a><?XmL x?></a>", "not well-formed (invalid token): line 1, column 8"),
            ("<?xml version=\"2.0\"?><a/>", "XML declaration not well-formed: line 1, column 15"),
            ("<?xml version=\"1.10\"?><a/>", "OK"),
            ("<?xml?><a/>", "XML declaration not well-formed: line 1, column 5"),
            ("<?xml encoding=\"UTF-8\"?><a/>", "XML declaration not well-formed: line 1, column 6"),
            ("<?xml version=\"1.0\" encoding=\"a b\"?><a/>", "XML declaration not well-formed: line 1, column 31"),
            ("<?xml version=\"1.0\" standalone=\"yes\" encoding=\"UTF-8\"?><a/>", "XML declaration not well-formed: line 1, column 37"),
            ("<?xml version = \"1.0\" encoding='UTF-8' standalone=\"no\" ?><a/>", "OK"),
            ("<?xml version=\"1.0\" encoding=\"UTF-16\"?><a/>", "encoding specified in XML declaration is incorrect: line 1, column 30"),
            ("<?xml version=\"1.0\" encoding=\"klingon\"?><a/>", "unknown encoding: klingon"),
            ("<?xml version=\"1.0\" encoding=\"big5\"?><a/>", "multi-byte encodings are not supported"),
            ("<?xml version=\"1.0\"", "unclosed token: line 1, column 0"),
            ("<!DOCTYPE a [<!ELEMENT a EMPTY>]><a/>", "OK"),
            ("<!DOCTYPE a [", "no element found: line 1, column 13"),
            ("<!DOCTYPE a><a/>", "OK"),
            ("<!DOCTYPE a SYSTEM \"a.dtd\" [ <!-- c --> <!ATTLIST a b CDATA \">\"> ]>\n<a/>", "OK"),
            ("<!DOCTYPE a?><a/>", "syntax error: line 1, column 10"),
            ("<!DOCTYPE a|<a/>", "syntax error: line 1, column 11"),
            ("<!DOCTYPE a<a/>", "not well-formed (invalid token): line 1, column 11"),
            ("<!DOCTYPEs a><a/>", "syntax error: line 1, column 0"),
            ("<!DOC", "unclosed token: line 1, column 0"),
            ("5?<a/>", "not well-formed (invalid token): line 1, column 1"),
            (")<a/>", "not well-formed (invalid token): line 1, column 1"),
            (":a", "syntax error: line 1, column 0"),
            ("<?xml? version=\"1.0\"?><a/>", "not well-formed (invalid token): line 1, column 6"),
            ("#i:a <a/>", "not well-formed (invalid token): line 1, column 2"),
            ("<a/><!/- c -->", "not well-formed (invalid token): line 1, column 6"),
            ("<a/><\u{1F600}fin?>", "junk after document element: line 1, column 4"),
            ("<a><![>", "unclosed token: line 1, column 3"),
            ("<a><![CDATA[x]]", "unclosed CDATA section: line 1, column 13"),
            ("<a>\r\n<c/>\r", "no element found: line 2, column 4"),
            ("<?xml version=\"1.1\" encoding=\"utf-\" ?>\n<a/>", "OK"),
            ("<?xml version=\"1.0\" ></a\u{1}>", "not well-formed (invalid token): line 1, column 24"),
            ("<a xml:lang=\"fr\">&lt;&#65;&#x41;<b c = \"1\" /><![CDATA[<]]></a >\n<?p?>", "OK"),
        ] {
            assert_eq!(expat(xml.as_bytes()), attendu, "{xml:?}");
        }
        assert_eq!(expat(b"<a b='\xff'/>"), "not well-formed (invalid token): line 1, column 6");
        assert_eq!(expat(b"<?xml version=\"1.0\" encoding=\"us-ascii\"?><a>\xe9</a>"), "not well-formed (invalid token): line 1, column 44");
        assert_eq!(expat(b"<?xml version=\"1.0\" encoding=\"iso-8859-1\"?><a b=\"\xe9\"></b>"), "mismatched tag: line 1, column 54");
        assert_eq!(expat(b"\xef\xbb\xbf<a/>"), "OK");
        // la marque d'ordre compte pour une colonne (trois, lue dans un encodage d'un octet)
        assert_eq!(expat(b"\xef\xbb\xbf<a><b></a>"), "mismatched tag: line 1, column 9");
        assert_eq!(expat(b"\xef\xbb\xbf<a>\n<b></a>"), "mismatched tag: line 2, column 5");
        assert_eq!(
            expat(b"\xef\xbb\xbf<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><a b=\"\xc3\xa9\"></b>"),
            "mismatched tag: line 1, column 58"
        );
        assert_eq!(expat(b"\xef\xbb\xbf"), "no element found: line 1, column 1");
        assert_eq!(expat(&utf16("<a>\n<b></a>")), "mismatched tag: line 2, column 5");
        assert_eq!(expat(b"\xff\xfe<\x00"), "unclosed token: line 1, column 1");
        assert_eq!(expat(b"<\x00a\x00/\x00>\x00"), "OK");
        assert_eq!(
            expat(&utf16("<?xml version=\"1.0\" encoding=\"UTF-8\"?><a/>")),
            "encoding specified in XML declaration is incorrect: line 1, column 31"
        );
        assert_eq!(expat(b"<a>\xc3</a>"), "not well-formed (invalid token): line 1, column 3");
        assert_eq!(expat(b"<a>\xe2\x82"), "partial character: line 1, column 3");
        assert_eq!(expat(b"<?xml versio<!\xe8"), "partial character: line 1, column 0");
        assert_eq!(expat(b"\xef\xbb\xbf\xf4"), "partial character: line 1, column 1");
        assert_eq!(expat(b"<?xml version=\"1.0\" e></a\xf0>\n"), "partial character: line 1, column 0");
        assert_eq!(expat(b"<a b=\xd3"), "not well-formed (invalid token): line 1, column 5");
        assert_eq!(expat(b"<a><!-- \xff --></a>"), "not well-formed (invalid token): line 1, column 8");
    }
}
