//! Ce que Python faisait tout seul et que le JSON laisse voir : l'écriture de json.dumps
//! (« , » et « : » suivis d'une espace, flottants à la repr, entiers sans borne), float(),
//! round(), sum() et math.hypot. Sans cela, le modèle rendu aux pages ne serait plus, à l'octet,
//! celui du lecteur Python. Repris de lanceur/src/mcp/py.rs pour ce qui s'y trouvait déjà.

use indexmap::IndexMap;

/// Un dict de Python : l'ordre d'insertion est gardé, réécrire une clé ne la déplace pas.
pub type Objet = IndexMap<String, Json>;

/// Une valeur JSON telle que Python la tenait : un entier n'est pas un flottant (1 et 1.0
/// ne s'écrivent pas pareil), et un entier peut dépasser 64 bits (une coordonnée de 1e300
/// arrondie par `_r`).
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Nul,
    Bool(bool),
    Entier(i64),
    /// Un entier hors de i64, déjà écrit en chiffres.
    GrandEntier(String),
    Flottant(f64),
    Texte(String),
    Liste(Vec<Json>),
    Objet(Objet),
}

impl Json {
    /// v[cle] d'un objet ; None pour une clé absente ou une valeur qui n'est pas un objet.
    pub fn get(&self, cle: &str) -> Option<&Json> {
        match self {
            Json::Objet(o) => o.get(cle),
            _ => None,
        }
    }
    /// v[i] d'une liste.
    pub fn at(&self, i: usize) -> Option<&Json> {
        match self {
            Json::Liste(l) => l.get(i),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Texte(t) => Some(t),
            _ => None,
        }
    }
    /// Un nombre, entier ou flottant.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Entier(i) => Some(*i as f64),
            Json::Flottant(f) => Some(*f),
            Json::GrandEntier(t) => t.parse().ok(),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Json::Entier(i) => Some(*i),
            _ => None,
        }
    }
    pub fn as_liste(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Liste(l) => Some(l),
            _ => None,
        }
    }
    pub fn as_objet(&self) -> Option<&Objet> {
        match self {
            Json::Objet(o) => Some(o),
            _ => None,
        }
    }
    /// bool(v) de Python : None, False, 0, "", [] et {} sont faux.
    pub fn vrai(&self) -> bool {
        match self {
            Json::Nul => false,
            Json::Bool(b) => *b,
            Json::Entier(i) => *i != 0,
            Json::GrandEntier(_) => true,
            Json::Flottant(f) => *f != 0.0,
            Json::Texte(t) => !t.is_empty(),
            Json::Liste(l) => !l.is_empty(),
            Json::Objet(o) => !o.is_empty(),
        }
    }
}

/// v["cle"] : Nul pour une clé absente, comme dict.get.
impl std::ops::Index<&str> for Json {
    type Output = Json;
    fn index(&self, cle: &str) -> &Json {
        self.get(cle).unwrap_or(&Json::Nul)
    }
}

/// v[i] : Nul hors de la liste.
impl std::ops::Index<usize> for Json {
    type Output = Json;
    fn index(&self, i: usize) -> &Json {
        self.at(i).unwrap_or(&Json::Nul)
    }
}

impl From<&str> for Json {
    fn from(t: &str) -> Json {
        Json::Texte(t.to_string())
    }
}
impl From<String> for Json {
    fn from(t: String) -> Json {
        Json::Texte(t)
    }
}
impl From<f64> for Json {
    fn from(f: f64) -> Json {
        Json::Flottant(f)
    }
}
impl From<i64> for Json {
    fn from(i: i64) -> Json {
        Json::Entier(i)
    }
}
impl From<usize> for Json {
    fn from(i: usize) -> Json {
        Json::Entier(i as i64)
    }
}
impl From<bool> for Json {
    fn from(b: bool) -> Json {
        Json::Bool(b)
    }
}

/// json.dumps(v, ensure_ascii=False) : le texte rendu aux pages.
pub fn dumps(v: &Json) -> String {
    let mut s = String::new();
    ecrire(v, &mut s);
    s
}

fn ecrire(v: &Json, s: &mut String) {
    match v {
        Json::Nul => s.push_str("null"),
        Json::Bool(b) => s.push_str(if *b { "true" } else { "false" }),
        Json::Entier(i) => s.push_str(&i.to_string()),
        Json::GrandEntier(t) => s.push_str(t),
        Json::Flottant(f) => s.push_str(&json_flottant(*f)),
        Json::Texte(t) => chaine(t, s),
        Json::Liste(l) => {
            s.push('[');
            for (i, e) in l.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                ecrire(e, s);
            }
            s.push(']');
        }
        Json::Objet(o) => {
            s.push('{');
            for (i, (k, e)) in o.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                chaine(k, s);
                s.push_str(": ");
                ecrire(e, s);
            }
            s.push('}');
        }
    }
}

fn chaine(t: &str, s: &mut String) {
    s.push('"');
    for c in t.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            '\u{8}' => s.push_str("\\b"),
            '\u{c}' => s.push_str("\\f"),
            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04x}", c as u32)),
            c => s.push(c),
        }
    }
    s.push('"');
}

/// Un flottant dans json.dumps : sa repr, et NaN / Infinity que Python écrit tels quels.
fn json_flottant(f: f64) -> String {
    if f.is_nan() {
        "NaN".into()
    } else if f.is_infinite() {
        if f > 0.0 { "Infinity" } else { "-Infinity" }.into()
    } else {
        repr_flottant(f)
    }
}

/// repr() d'un flottant : les chiffres les plus courts, en notation fixe de 1e-4 à 1e16.
pub fn repr_flottant(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf" } else { "-inf" }.into();
    }
    let mut e = format!("{:e}", f.abs()); // « 1.2345e-5 », « 1e16 », « 0e0 »
    // Deux écritures les plus courtes à égale distance (une moitié exacte au 17e chiffre) :
    // Python prend la paire, l'écriture la plus courte de Rust pas toujours. L'arrondi exact au
    // même nombre de chiffres tranche comme Python, s'il relit bien le même flottant.
    let chiffres = e.split('e').next().unwrap_or("").len().saturating_sub(2);
    let exact = format!("{:.*e}", chiffres, f.abs());
    if exact != e && exact.parse::<f64>().ok() == Some(f.abs()) {
        e = exact;
    }
    let (mantisse, exposant) = e.split_once('e').unwrap_or((&e, "0"));
    let exposant: i32 = exposant.parse().unwrap_or(0);
    let chiffres: String = mantisse.chars().filter(|c| *c != '.').collect();
    let signe = if f.is_sign_negative() { "-" } else { "" };
    if (-4..16).contains(&exposant) {
        let corps = if exposant < 0 {
            format!("0.{}{}", "0".repeat((-exposant - 1) as usize), chiffres)
        } else {
            let entiers = exposant as usize + 1;
            if chiffres.len() <= entiers {
                format!("{}{}.0", chiffres, "0".repeat(entiers - chiffres.len()))
            } else {
                format!("{}.{}", &chiffres[..entiers], &chiffres[entiers..])
            }
        };
        return format!("{signe}{corps}");
    }
    let mantisse = if chiffres.len() > 1 { format!("{}.{}", &chiffres[..1], &chiffres[1..]) } else { chiffres };
    format!("{signe}{mantisse}e{}{:02}", if exposant < 0 { '-' } else { '+' }, exposant.abs())
}

/// Les blancs de str.strip() : ceux d'Unicode, et les séparateurs \x1c à \x1f.
pub fn blanc(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// str.strip()
pub fn strip(t: &str) -> &str {
    t.trim_matches(blanc)
}

// Le zéro de chaque suite de dix chiffres décimaux d'Unicode 15.1 (catégorie Nd), tel que
// le connaît Python 3.13 : float() les lit tous, « ١٢ » vaut 12.
const ZEROS: [u32; 68] = [
    0x30, 0x660, 0x6F0, 0x7C0, 0x966, 0x9E6, 0xA66, 0xAE6, 0xB66, 0xBE6, 0xC66, 0xCE6, 0xD66, 0xDE6, 0xE50, 0xED0,
    0xF20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80, 0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620,
    0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0, 0xFF10, 0x104A0, 0x10D30, 0x11066, 0x110F0, 0x11136, 0x111D0,
    0x112F0, 0x11450, 0x114D0, 0x11650, 0x116C0, 0x11730, 0x118E0, 0x11950, 0x11C50, 0x11D50, 0x11DA0, 0x11F50,
    0x16A60, 0x16AC0, 0x16B50, 0x1D7CE, 0x1D7D8, 0x1D7E2, 0x1D7EC, 0x1D7F6, 0x1E140, 0x1E2F0, 0x1E4F0, 0x1E950,
    0x1FBF0,
];

/// float(texte) : None là où Python lève ValueError. Les blancs autour sont admis, « 1_000 »
/// aussi, les chiffres d'autres écritures de même, « inf » et « nan » encore (à l'appelant
/// de les refuser) ; « 1e400 » rend l'infini.
pub fn float(t: &str) -> Option<f64> {
    let t = strip(t);
    let propre;
    let t = if t.is_ascii() {
        t
    } else {
        // un chiffre décimal d'Unicode vaut son chiffre ASCII ; tout autre caractère est refusé
        let mut ascii = String::with_capacity(t.len());
        for c in t.chars() {
            let n = c as u32;
            ascii.push(match ZEROS.iter().find(|z| (**z..**z + 10).contains(&n)) {
                Some(z) => (b'0' + (n - z) as u8) as char,
                None if c.is_ascii() => c,
                None => return None,
            });
        }
        propre = ascii;
        propre.as_str()
    };
    let sans_tirets;
    let t = if t.contains('_') {
        // un tiret bas ne se place qu'entre deux chiffres
        let o = t.as_bytes();
        for (i, c) in o.iter().enumerate() {
            if *c == b'_' && !(i > 0 && i + 1 < o.len() && o[i - 1].is_ascii_digit() && o[i + 1].is_ascii_digit()) {
                return None;
            }
        }
        sans_tirets = t.replace('_', "");
        sans_tirets.as_str()
    } else {
        t
    };
    t.parse::<f64>().ok()
}

/// round(x, n) : l'arrondi décimal exact de la valeur binaire, les moitiés vers le pair ;
/// un non-fini est rendu tel quel.
pub fn round(x: f64, n: usize) -> f64 {
    if !x.is_finite() {
        return x;
    }
    format!("{:.*}", n, x).parse().unwrap_or(x)
}

/// sum() d'une suite de flottants (0 pour une suite vide). Depuis Python 3.12, la somme
/// est compensée (Neumaier) : une addition naïve ne rend pas le même dernier chiffre.
pub fn sum(valeurs: impl IntoIterator<Item = f64>) -> f64 {
    let mut it = valeurs.into_iter();
    let Some(premier) = it.next() else { return 0.0 };
    let mut total = 0.0 + premier; // sum() part de l'entier 0 : -0.0 devient 0.0
    let mut c = 0.0f64;
    for x in it {
        let t = total + x;
        if total.abs() >= x.abs() {
            c += (total - t) + x;
        } else {
            c += (x - t) + total;
        }
        total = t;
    }
    if c != 0.0 && c.is_finite() {
        total += c;
    }
    total
}

/// math.hypot(x, y), par l'algorithme de CPython (vector_norm) et non celui de la libm :
/// les deux sont à un demi-chiffre près, pas toujours du même côté.
pub fn hypot(x: f64, y: f64) -> f64 {
    let (x, y) = (x.abs(), y.abs());
    if x.is_infinite() || y.is_infinite() {
        return f64::INFINITY;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    norme([x, y], x.max(y))
}

fn norme(mut v: [f64; 2], max: f64) -> f64 {
    if max == 0.0 {
        return max;
    }
    let max_e = exposant(max);
    if max_e < -1023 {
        // ldexp(1.0, -max_e) déborderait
        v[0] /= f64::MIN_POSITIVE;
        v[1] /= f64::MIN_POSITIVE;
        return f64::MIN_POSITIVE * norme(v, max / f64::MIN_POSITIVE);
    }
    // ldexp(1.0, -max_e), exact jusque dans les dénormalisés (2^-1024)
    let echelle = if -max_e >= -1022 {
        f64::from_bits(((1023 - max_e) as u64) << 52)
    } else {
        f64::from_bits(1u64 << (1074 - max_e))
    };
    // [csum, frac1, frac2] : la somme des carrés, et ce que chaque addition a perdu
    let mut s = [1.0f64, 0.0, 0.0];
    fn ajouter(x: f64, y: f64, s: &mut [f64; 3]) {
        let hi = x * y;
        let lo = x.mul_add(y, -hi); // produit sans perte
        let somme = s[0] + hi;
        s[2] += (s[0] - somme) + hi;
        s[0] = somme;
        s[1] += lo;
    }
    for x in v {
        let x = x * echelle;
        ajouter(x, x, &mut s);
    }
    let mut h = (s[0] - 1.0 + (s[1] + s[2])).sqrt();
    ajouter(-h, h, &mut s);
    let x = s[0] - 1.0 + (s[1] + s[2]);
    h += x / (2.0 * h); // correction différentielle
    h / echelle
}

/// L'exposant de frexp : x = m * 2^e, 0,5 <= m < 1.
fn exposant(x: f64) -> i32 {
    let bits = x.to_bits();
    let e = ((bits >> 52) & 0x7ff) as i32;
    if e == 0 {
        // dénormalisé : le rang du premier bit de la mantisse
        let m = bits & ((1u64 << 52) - 1);
        return -1074 + (64 - m.leading_zeros() as i32);
    }
    e - 1022
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecriture_de_python() {
        let mut o = Objet::new();
        o.insert("a".into(), Json::Liste(vec![1i64.into(), 2.5.into(), 1.0.into(), true.into(), Json::Nul]));
        o.insert("é".into(), "x\"\n\u{1}".into());
        o.insert("v".into(), Json::Objet(Objet::new()));
        o.insert("l".into(), Json::Liste(vec![]));
        assert_eq!(dumps(&Json::Objet(o)),
                   "{\"a\": [1, 2.5, 1.0, true, null], \"é\": \"x\\\"\\n\\u0001\", \"v\": {}, \"l\": []}");
        for (f, attendu) in [(1e-6, "1e-06"), (1e16, "1e+16"), (1e15, "1000000000000000.0"), (0.0001, "0.0001"),
                             (-1000000.0, "-1000000.0"), (1.5e-7, "1.5e-07"), (0.1, "0.1"), (123.456, "123.456"),
                             (1.2345e22, "1.2345e+22"), (0.0, "0.0"), (20.5, "20.5")] {
            assert_eq!(repr_flottant(f), attendu);
        }
        assert_eq!(dumps(&Json::Liste(vec![f64::NAN.into(), f64::INFINITY.into(), f64::NEG_INFINITY.into()])),
                   "[NaN, Infinity, -Infinity]");
    }

    #[test]
    fn nombres_de_python() {
        assert_eq!(float(" 1_000.5 "), Some(1000.5));
        assert_eq!(float("1e400"), Some(f64::INFINITY));
        assert_eq!(float(".5"), Some(0.5));
        assert_eq!(float("١٢.٥"), Some(12.5));
        assert_eq!(float("5."), Some(5.0));
        assert!(float("nan").is_some_and(f64::is_nan));
        for refuse in ["", "abc", "1,5", "_1", "1__0", "0x10", "1e", "+", "1 2", "½", "é"] {
            assert_eq!(float(refuse), None, "{refuse}");
        }
        // les moitiés exactes vont au pair ; 2.675 est en dessous de sa moitié
        assert_eq!((round(0.5, 0), round(1.5, 0), round(2.5, 0)), (0.0, 2.0, 2.0));
        assert_eq!((round(0.125, 2), round(2.675, 2), round(1e-7 / 2.0, 6)), (0.12, 2.67, 0.0));
        assert_eq!(round(-1e-9, 6).to_bits(), (-0.0f64).to_bits());
        // relevés sur Python 3.13 : sum() compensée, math.hypot
        assert_eq!(sum([0.1; 10]), 1.0);
        assert_eq!(sum([1e16, 1.0, -1e16]), 1.0);
        assert_eq!(sum(std::iter::empty()), 0.0);
        assert_eq!(hypot(3.0, 4.0), 5.0);
        assert_eq!(hypot(1e308, 1e308), 1.4142135623730951e308);
        assert_eq!(hypot(1e308, 48.0), 1e308);
        assert_eq!(sum([-0.0]).to_bits(), 0.0f64.to_bits());
        assert_eq!(repr_flottant(99201984633078.625), "99201984633078.62");
        assert_eq!(hypot(0.0, -0.0), 0.0);
    }
}
