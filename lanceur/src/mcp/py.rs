//! Ce que Python faisait tout seul et que le protocole laisse voir : l'écriture JSON de
//! json.dumps (« , » et « : » suivis d'une espace, flottants à la repr), str() et repr() d'une
//! valeur dans un message, la véracité, urlencode. Sans cela, les tailles mesurées par resumer()
//! et les textes rendus à Claude ne seraient plus ceux du serveur Python.

use serde_json::Value;

pub type Objet = serde_json::Map<String, Value>;

/// json.dumps(v, ensure_ascii=False) : le texte rendu à Claude.
pub fn dumps(v: &Value) -> String {
    let mut s = String::new();
    ecrire(v, false, None, 0, &mut s);
    s
}

/// json.dumps(v) : ensure_ascii, pour les corps de requête et les lignes de commande.
pub fn dumps_ascii(v: &Value) -> String {
    let mut s = String::new();
    ecrire(v, true, None, 0, &mut s);
    s
}

/// json.dumps(v, ensure_ascii=False, indent=n, sort_keys=True).
pub fn dumps_indente_trie(v: &Value, indent: usize) -> String {
    let mut s = String::new();
    ecrire(&trier(v), false, Some(indent), 0, &mut s);
    s
}

fn trier(v: &Value) -> Value {
    match v {
        Value::Object(o) => {
            let mut cles: Vec<&String> = o.keys().collect();
            cles.sort(); // octets UTF-8 : l'ordre des points de code de Python
            Value::Object(cles.into_iter().map(|k| (k.clone(), trier(&o[k]))).collect())
        }
        Value::Array(l) => Value::Array(l.iter().map(trier).collect()),
        autre => autre.clone(),
    }
}

fn ecrire(v: &Value, ascii: bool, indent: Option<usize>, niveau: usize, s: &mut String) {
    let retour = |s: &mut String, niveau: usize| {
        if let Some(n) = indent {
            s.push('\n');
            s.push_str(&" ".repeat(n * niveau));
        }
    };
    let virgule = if indent.is_some() { "," } else { ", " };
    match v {
        Value::Null => s.push_str("null"),
        Value::Bool(b) => s.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => s.push_str(&nombre(n)),
        Value::String(t) => chaine(t, ascii, s),
        Value::Array(l) if l.is_empty() => s.push_str("[]"),
        Value::Array(l) => {
            s.push('[');
            for (i, e) in l.iter().enumerate() {
                if i > 0 {
                    s.push_str(virgule);
                }
                retour(s, niveau + 1);
                ecrire(e, ascii, indent, niveau + 1, s);
            }
            retour(s, niveau);
            s.push(']');
        }
        Value::Object(o) if o.is_empty() => s.push_str("{}"),
        Value::Object(o) => {
            s.push('{');
            for (i, (k, e)) in o.iter().enumerate() {
                if i > 0 {
                    s.push_str(virgule);
                }
                retour(s, niveau + 1);
                chaine(k, ascii, s);
                s.push_str(": ");
                ecrire(e, ascii, indent, niveau + 1, s);
            }
            retour(s, niveau);
            s.push('}');
        }
    }
}

fn chaine(t: &str, ascii: bool, s: &mut String) {
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
            c if ascii && !c.is_ascii() => {
                for u in c.encode_utf16(&mut [0; 2]) {
                    s.push_str(&format!("\\u{:04x}", u));
                }
            }
            c => s.push(c),
        }
    }
    s.push('"');
}

fn nombre(n: &serde_json::Number) -> String {
    if n.is_f64() {
        repr_flottant(n.as_f64().unwrap_or(0.0))
    } else {
        n.to_string()
    }
}

/// repr() d'un flottant : les chiffres les plus courts, en notation fixe de 1e-4 à 1e16.
pub fn repr_flottant(f: f64) -> String {
    let e = format!("{:e}", f.abs()); // « 1.2345e-5 », « 1e16 », « 0e0 »
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

/// str(v) de Python, pour une valeur venue du JSON.
pub fn py_str(v: &Value) -> String {
    match v {
        Value::String(t) => t.clone(),
        autre => py_repr(autre),
    }
}

/// repr(v) de Python.
pub fn py_repr(v: &Value) -> String {
    match v {
        Value::Null => "None".into(),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        Value::Number(n) => nombre(n),
        Value::String(t) => repr_chaine(t),
        Value::Array(l) => format!("[{}]", l.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Object(o) => format!(
            "{{{}}}",
            o.iter().map(|(k, e)| format!("{}: {}", repr_chaine(k), py_repr(e))).collect::<Vec<_>>().join(", ")
        ),
    }
}

fn repr_chaine(t: &str) -> String {
    let guillemet = if t.contains('\'') && !t.contains('"') { '"' } else { '\'' };
    let mut s = String::from(guillemet);
    for c in t.chars() {
        match c {
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c if c == guillemet => {
                s.push('\\');
                s.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => s.push_str(&format!("\\x{:02x}", c as u32)),
            c => s.push(c),
        }
    }
    s.push(guillemet);
    s
}

/// bool(v) de Python : None, False, 0, "", [] et {} sont faux.
pub fn vrai(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(t) => !t.is_empty(),
        Value::Array(l) => !l.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// « v or défaut » pour une clé d'objet : None quand la valeur est absente ou fausse.
pub fn ou_rien<'a>(o: &'a Objet, cle: &str) -> Option<&'a Value> {
    o.get(cle).filter(|v| vrai(v))
}

/// type(v) is int : un entier JSON, ni booléen ni flottant.
pub fn entier(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) if n.is_i64() => n.as_i64(),
        Value::Number(n) if n.is_u64() => Some(i64::MAX), // au-delà de i64 : toujours hors bornes ici
        _ => None,
    }
}

/// type(v) in (int, float) : un nombre JSON, pas un booléen. (NaN et l'infini n'existent pas en JSON.)
pub fn reel(v: &Value) -> Option<f64> {
    v.as_f64()
}

/// type(v).__name__
pub fn nom_type(v: &Value) -> &'static str {
    match v {
        Value::Null => "NoneType",
        Value::Bool(_) => "bool",
        Value::Number(n) if n.is_f64() => "float",
        Value::Number(_) => "int",
        Value::String(_) => "str",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}

/// len(texte) de Python : des caractères, pas des octets.
pub fn longueur(t: &str) -> usize {
    t.chars().count()
}

/// texte[:n]
pub fn debut(t: &str, n: usize) -> &str {
    t.char_indices().nth(n).map_or(t, |(i, _)| &t[..i])
}

/// texte[-n:]
pub fn fin(t: &str, n: usize) -> &str {
    let total = longueur(t);
    if total <= n {
        t
    } else {
        t.char_indices().nth(total - n).map_or(t, |(i, _)| &t[i..])
    }
}

/// str.strip() : les blancs de Python comptent aussi les séparateurs \x1c à \x1f.
pub fn strip(t: &str) -> &str {
    t.trim_matches(|c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c))
}

/// re.match(r"^[^\x00-\x1f]{mini,maxi}$") (et \x7f avec `del`) : « $ » tolère un « \n » final.
pub fn une_ligne(t: &str, mini: usize, maxi: usize, del: bool) -> bool {
    let t = t.strip_suffix('\n').unwrap_or(t);
    let n = longueur(t);
    mini <= n && n <= maxi && !t.chars().any(|c| (c as u32) < 0x20 || (del && c as u32 == 0x7f))
}

/// urllib.parse.quote(texte, safe) : tout sauf lettres, chiffres, « _.-~ » et `saufs` en %XX.
pub fn quote(t: &str, saufs: &str) -> String {
    let mut s = String::new();
    for o in t.bytes() {
        if o.is_ascii_alphanumeric() || b"_.-~".contains(&o) || saufs.as_bytes().contains(&o) {
            s.push(o as char);
        } else {
            s.push_str(&format!("%{:02X}", o));
        }
    }
    s
}

/// urllib.parse.urlencode : quote_plus de chaque valeur, l'espace en « + ».
pub fn urlencode(params: &[(&str, String)]) -> String {
    let plus = |t: &str| quote(t, " ").replace(' ', "+");
    params.iter().map(|(k, v)| format!("{}={}", plus(k), plus(v))).collect::<Vec<_>>().join("&")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ecriture_de_python() {
        let v = json!({"a": [1, 2.5, 1.0, true, null], "é": "x\"\n\u{1}", "v": {}, "l": []});
        assert_eq!(dumps(&v), "{\"a\": [1, 2.5, 1.0, true, null], \"é\": \"x\\\"\\n\\u0001\", \"v\": {}, \"l\": []}");
        assert_eq!(dumps_ascii(&json!("é😀")), "\"\\u00e9\\ud83d\\ude00\"");
        assert_eq!(dumps_indente_trie(&json!({"b": [1, {}], "a": {"c": []}}), 1),
                   "{\n \"a\": {\n  \"c\": []\n },\n \"b\": [\n  1,\n  {}\n ]\n}");
        for (f, attendu) in [(1e-6, "1e-06"), (1e16, "1e+16"), (1e15, "1000000000000000.0"), (0.0001, "0.0001"),
                             (-1000000.0, "-1000000.0"), (1.5e-7, "1.5e-07"), (0.1, "0.1"), (123.456, "123.456"),
                             (1.2345e22, "1.2345e+22"), (0.0, "0.0"), (20.5, "20.5")] {
            assert_eq!(repr_flottant(f), attendu);
        }
    }

    #[test]
    fn textes_de_python() {
        assert_eq!(py_str(&json!({"a": [1, "b", null, true], "c": "l'été"})), "{'a': [1, 'b', None, True], 'c': \"l'été\"}");
        assert_eq!(py_repr(&json!("../x")), "'../x'");
        assert!(une_ligne("abc\n", 1, 200, false) && !une_ligne("a\nb", 1, 200, false) && !une_ligne("", 1, 200, false));
        assert_eq!(urlencode(&[("chemin", "a b/c".into()), ("racine", "/r".into())]), "chemin=a+b%2Fc&racine=%2Fr");
        assert_eq!(quote("{\"a\": 1}/é", "/"), "%7B%22a%22%3A%201%7D/%C3%A9");
        assert_eq!((debut("éàü", 2), fin("éàü", 2), strip(" \u{1c}x\n")), ("éà", "àü", "x"));
    }
}
