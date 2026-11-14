//! Serveur : page Banc sous /__banc/, tout le reste relayé à l'outil actif.

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};
use web_tools_commun::serveur::{params, Reponse, Requete};

use crate::bancs::Bancs;
use crate::outils::{trouver_chromium, which, Outil, CATALOGUE, DOSSIERS_IGNORES};

pub const PREFIXE: &str = "/__banc";
const ENTETE_BANC: &str = "X-Web-Bench";
const METHODES: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

/// Requêtes arrêtées par le relais pour tous les outils, quel que soit le bouton qui les envoie :
/// rien ne part sur GitHub, rien n'est demandé à Claude Code.
/// (l'ancienne expression : ^/api/github/envoyer|^/api/pilote/demande)
const ROUTES_BLOQUEES: &[&str] = &["/api/github/envoyer", "/api/pilote/demande"];

/// en-têtes propres à une connexion : ils ne passent pas le relais
const SAUTS: &[&str] = &[
    "connection", "keep-alive", "proxy-authenticate", "proxy-authorization", "te", "trailer", "transfer-encoding",
    "upgrade", "content-length", "accept-encoding",
];
const SONDE: &[u8] = b"<script src=\"/__banc/sonde.js\"></script>";

fn saute(entete: &str) -> bool {
    SAUTS.contains(&entete.to_ascii_lowercase().as_str())
}

/// L'état partagé du serveur : les outils, leurs bancs, l'outil affiché.
pub struct Banc {
    pub outils: Vec<Arc<Outil>>,
    pub bancs: Vec<Arc<Bancs>>,
    actif: Mutex<&'static str>,
    dossier_page: PathBuf,
    dossier_rapports: PathBuf,
}

impl Banc {
    /// `ici` : le dossier web_bench/ du dépôt, où vivent page/ et rapports/.
    pub fn new(outils: Vec<Arc<Outil>>, actif: &'static str, ici: &Path) -> Banc {
        Banc {
            bancs: outils.iter().map(|o| Bancs::new(o.clone())).collect(),
            outils,
            actif: Mutex::new(actif),
            dossier_page: ici.join("page"),
            dossier_rapports: ici.join("rapports"),
        }
    }

    fn actif(&self) -> &'static str {
        *self.actif.lock().unwrap()
    }

    fn par_id(&self, id: &str) -> Option<&Arc<Outil>> {
        self.outils.iter().find(|o| o.conf.id == id)
    }

    fn outil(&self) -> &Arc<Outil> {
        self.par_id(self.actif()).expect("l'outil actif est installé")
    }

    pub fn arreter(&self) {
        for b in &self.bancs {
            b.arreter();
        }
        for o in &self.outils {
            o.arreter();
        }
    }

    pub fn servir(&self, r: &mut Requete) -> Reponse {
        if !METHODES.contains(&r.methode.as_str()) {
            return Reponse::texte(501, &format!("Unsupported method ('{}')", r.methode));
        }
        // garde : seulement la boucle locale, sous un nom de la boucle locale
        if !hote_local(r.entete("Host").unwrap_or("")) {
            return Reponse::texte(403, "Host refuse");
        }
        let chemin = r.chemin().to_string();
        if let Some(suite) = chemin.strip_prefix(PREFIXE).filter(|s| s.is_empty() || s.starts_with('/')) {
            return self.page_banc(r, if suite.is_empty() { "/" } else { suite });
        }
        // tout le reste, « / » compris, est à l'outil actif : ses pages y renvoient
        self.relayer(r)
    }

    // ---- la page Banc et son API
    fn page_banc(&self, r: &mut Requete, chemin: &str) -> Reponse {
        if let Some(route) = chemin.strip_prefix("/api/") {
            if r.methode == "POST" && r.entete(ENTETE_BANC) != Some("1") {
                return rendre_json(403, &json!({"erreur": format!("en-tête {} manquant", ENTETE_BANC)}));
            }
            return self.api(r, route).unwrap_or_else(|refus| refus);
        }
        let fichier = if chemin == "/" { "index.html" } else { chemin.trim_start_matches('/') };
        let introuvable = || Reponse::texte(404, "introuvable");
        let (Ok(racine), Ok(cible)) = (fs::canonicalize(&self.dossier_page), fs::canonicalize(self.dossier_page.join(fichier)))
        else {
            return introuvable();
        };
        if cible == racine || !cible.starts_with(&racine) || !cible.is_file() {
            return introuvable();
        }
        let Ok(corps) = fs::read(&cible) else { return introuvable() };
        let type_ = match cible.extension().and_then(|e| e.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("js") => "text/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("json") => "application/json",
            Some("svg") => "image/svg+xml",
            _ => "application/octet-stream",
        };
        Reponse::new(200, type_, corps)
    }

    /// Le corps JSON d'un POST. Python laissait tomber la connexion sur un corps illisible ;
    /// ici la réponse est un 400.
    fn corps_json(r: &mut Requete) -> Result<Value, Reponse> {
        let refus = |quoi: &str| rendre_json(400, &json!({"erreur": quoi}));
        let brut = r.lire_corps(20 * 1024 * 1024).map_err(|_| refus("corps trop gros"))?;
        let corps = if brut.is_empty() { json!({}) } else { serde_json::from_slice(&brut).map_err(|_| refus("JSON invalide"))? };
        if corps.is_object() {
            Ok(corps)
        } else {
            Err(refus("JSON invalide"))
        }
    }

    fn bancs_de(&self, params: &Params, corps: Option<&Value>) -> Option<&Arc<Bancs>> {
        let id = corps
            .and_then(|c| c["outil"].as_str())
            .filter(|i| !i.is_empty())
            .or_else(|| params.get("outil"))
            .unwrap_or_else(|| self.actif());
        self.outils.iter().position(|o| o.conf.id == id).map(|i| &self.bancs[i])
    }

    fn api(&self, r: &mut Requete, route: &str) -> Result<Reponse, Reponse> {
        let params = Params(params(r.requete()));
        let o = self.outil();
        let post = r.methode == "POST";
        let ok = |v: Value| Ok(rendre_json(200, &v));
        let erreur = |code: u16, quoi: &str| Ok(rendre_json(code, &json!({"erreur": quoi})));
        match route {
            "etat" => {
                let outils: Vec<Value> = CATALOGUE
                    .iter()
                    .map(|c| {
                        let o = self.par_id(c.id);
                        json!({"id": c.id, "nom": c.nom, "installe": o.is_some(),
                               "en_marche": o.is_some_and(|o| o.en_marche()),
                               "port": o.and_then(|o| o.port()),
                               "dossier": o.map(|o| o.dossier.to_string_lossy()),
                               "version": o.map(|o| &o.version),
                               "noire": c.noire})
                    })
                    .collect();
                ok(json!({"actif": self.actif(), "chromium": trouver_chromium(), "node": which("node"), "outils": outils}))
            }
            "outil/choisir" if post => {
                let corps = Self::corps_json(r)?;
                let Some(choisi) = corps["id"].as_str().and_then(|i| self.par_id(i)) else {
                    let id = match &corps["id"] {
                        Value::String(s) => s.clone(),
                        Value::Null => "None".into(),
                        autre => autre.to_string(),
                    };
                    return erreur(404, &format!("outil non installé : {}", id));
                };
                if let Err(exc) = choisi.demarrer() {
                    return erreur(500, &exc);
                }
                *self.actif.lock().unwrap() = choisi.conf.id;
                ok(json!({"ok": true, "actif": choisi.conf.id}))
            }
            "outil/redemarrer" if post => match o.redemarrer() {
                Ok(()) => ok(json!({"ok": true, "port": o.port()})),
                Err(exc) => erreur(500, &exc),
            },
            "pages" => ok(json!({"outil": self.actif(), "pages": pages_de(&o.dossier)})),
            "journal" => {
                let depuis = params.get("depuis").and_then(|d| d.trim().parse().ok()).unwrap_or(0);
                let (lignes, fin) = o.lignes_depuis(depuis);
                ok(json!({"outil": self.actif(), "lignes": lignes, "fin": fin}))
            }
            "bancs" => match self.bancs_de(&params, None) {
                Some(b) => ok(b.etat()),
                None => erreur(404, "outil inconnu"),
            },
            "bancs/sortie" => match self.bancs_de(&params, None).and_then(|b| b.sortie(params.get("id").unwrap_or(""))) {
                Some(s) => ok(json!({"sortie": s})),
                None => erreur(404, "banc inconnu"),
            },
            "bancs/lancer" if post => {
                let corps = Self::corps_json(r)?;
                let Some(b) = self.bancs_de(&params, Some(&corps)) else { return erreur(404, "outil inconnu") };
                let ids: Vec<String> = match corps["ids"].as_array().filter(|ids| !ids.is_empty()) {
                    Some(ids) => ids.iter().filter_map(|i| i.as_str().map(str::to_string)).collect(),
                    None => b.ids_ci(),
                };
                ok(json!({"ok": b.lancer(&ids)}))
            }
            "bancs/arreter" if post => {
                let corps = Self::corps_json(r)?;
                if let Some(b) = self.bancs_de(&params, Some(&corps)) {
                    b.arreter();
                }
                ok(json!({"ok": true}))
            }
            "rapports" if r.methode == "GET" => ok(json!({"rapports": self.lister_rapports()})),
            "rapports" if post => {
                let corps = Self::corps_json(r)?;
                match self.enregistrer_rapport(corps) {
                    Ok(nom) => ok(json!({"nom": nom})),
                    Err(e) => erreur(500, &e.to_string()),
                }
            }
            "rapport.md" => {
                let nom = nom_de_base(params.get("nom").unwrap_or(""));
                match nom.strip_suffix(".json").and_then(|n| fs::read(self.dossier_rapports.join(format!("{}.md", n))).ok()) {
                    Some(md) => Ok(Reponse::new(200, "text/markdown; charset=utf-8", md)),
                    None => Ok(Reponse::texte(404, "pas de .md pour ce rapport")),
                }
            }
            "rapport" => {
                let nom = nom_de_base(params.get("nom").unwrap_or(""));
                let lu = if nom.ends_with(".json") { fs::read(self.dossier_rapports.join(nom)).ok() } else { None };
                match lu.and_then(|brut| serde_json::from_slice::<Value>(&brut).ok()) {
                    Some(rapport) => ok(rapport),
                    None => erreur(404, "rapport inconnu"),
                }
            }
            _ => erreur(404, "route inconnue"),
        }
    }

    // ---- rapports du clic-partout
    fn lister_rapports(&self) -> Vec<Value> {
        let Ok(entrees) = fs::read_dir(&self.dossier_rapports) else { return Vec::new() };
        let mut noms: Vec<String> = entrees.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        noms.sort();
        let mut rendu = Vec::new();
        for f in noms.iter().rev().filter(|f| f.ends_with(".json")) {
            let lu = fs::read(self.dossier_rapports.join(f)).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok());
            if let Some(r) = lu {
                let md = self.dossier_rapports.join(format!("{}.md", &f[..f.len() - 5])).is_file();
                rendu.push(json!({"nom": f, "date": r.get("date"), "outil": r.get("outil"), "bilan": r.get("bilan"), "md": md}));
            }
        }
        rendu
    }

    /// Le rapport complet en .json, et sa version lisible en .md à côté : celle-là se donne
    /// telle quelle à Claude pour les corrections.
    fn enregistrer_rapport(&self, mut rapport: Value) -> io::Result<String> {
        fs::create_dir_all(&self.dossier_rapports)?;
        let outil = match &rapport["outil"] {
            Value::String(s) if !s.is_empty() => s.clone(),
            Value::Null | Value::String(_) | Value::Bool(false) => "outil".into(),
            autre => autre.to_string(),
        };
        let outil: String = outil.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(20).collect();
        let nom = format!("{}_{}.json", chrono::Local::now().format("%Y-%m-%d_%H%M%S"), outil);
        let markdown = rapport.as_object_mut().and_then(|r| r.remove("markdown"));
        // comme json.dump(indent=1)
        let mut texte = Vec::new();
        let mut ecrivain = serde_json::Serializer::with_formatter(&mut texte, serde_json::ser::PrettyFormatter::with_indent(b" "));
        rapport.serialize(&mut ecrivain)?;
        fs::write(self.dossier_rapports.join(&nom), texte)?;
        if let Some(md) = markdown.as_ref().and_then(Value::as_str).filter(|md| !md.is_empty()) {
            fs::write(self.dossier_rapports.join(format!("{}.md", &nom[..nom.len() - 5])), md)?;
        }
        Ok(nom)
    }

    // ---- le relais vers l'outil actif
    fn relayer(&self, r: &mut Requete) -> Reponse {
        let corps = match r.lire_corps(256 * 1024 * 1024) {
            Ok(c) => c,
            Err(web_tools_commun::serveur::ErreurCorps::TropGrand) => return Reponse::texte(413, "Corps trop gros"),
            Err(_) => return Reponse::texte(400, "Content-Length invalide"),
        };
        let o = self.outil();
        let chemin = r.chemin().to_string();
        let bloquee = ROUTES_BLOQUEES.iter().any(|p| chemin.starts_with(p)) || o.conf.routes.is_some_and(|f| f(&chemin));
        if r.methode != "GET" && bloquee {
            o.noter(format!("[web_bench] requête arrêtée par le banc : {} {}", r.methode, chemin));
            return Reponse {
                code: 403,
                entetes: vec![("Content-Type".into(), "application/json".into()), ("X-Web-Bench-Bloque".into(), "1".into())],
                corps: format!("{{\"detail\": \"{}\"}}", ascii("Arrêté par web_bench (route protégée pendant les essais)"))
                    .into_bytes(),
                suite: None,
            };
        }
        let entetes: Vec<(String, String)> = r.entetes.iter().filter(|(k, _)| !saute(k)).cloned().collect();
        let corps = (!corps.is_empty()).then_some(corps);
        let rep = match demander(o.port().unwrap_or(0), &r.methode, &r.cible, &entetes, corps.as_deref()) {
            Ok(rep) => rep,
            Err(exc) => return Reponse::texte(502, &format!("{} injoignable : {}", o.nom(), exc)),
        };
        let de_amont = |nom: &str| rep.entetes.iter().find(|(k, _)| k.eq_ignore_ascii_case(nom)).map(|(_, v)| v.clone());
        let mut entetes: Vec<(String, String)> = rep.entetes.iter().filter(|(k, _)| !saute(k)).cloned().collect();
        let html = de_amont("Content-Type").is_some_and(|t| t.starts_with("text/html"));
        let corps = if html && r.methode == "GET" { injecter_sonde(&rep.corps) } else { rep.corps };
        // HEAD : la longueur annoncée par l'outil ; sinon le serveur compte le corps
        if let (true, Some(longueur)) = (r.methode == "HEAD", de_amont("Content-Length")) {
            entetes.push(("Content-Length".into(), longueur));
        }
        Reponse { code: rep.code, entetes, corps, suite: None }
    }
}

struct Params(Vec<(String, String)>);

impl Params {
    /// Comme dict(parse_qsl()) : la dernière valeur non vide du paramètre.
    fn get(&self, nom: &str) -> Option<&str> {
        self.0.iter().rev().find(|(k, v)| k == nom && !v.is_empty()).map(|(_, v)| v.as_str())
    }
}

/// Les accents échappés (ê : barre oblique inverse, u00ea), comme json.dumps les écrivait par défaut.
fn ascii(texte: &str) -> String {
    texte.chars().map(|c| if c.is_ascii() { c.to_string() } else { format!("{}u{:04x}", '\\', c as u32) }).collect()
}

fn nom_de_base(nom: &str) -> &str {
    nom.rsplit(|c| c == '/' || (cfg!(windows) && c == '\\')).next().unwrap_or("")
}

fn hote_local(host: &str) -> bool {
    let host = host.to_lowercase();
    let hote = host.rsplit_once(':').map_or(host.as_str(), |(h, _)| h).trim_matches(['[', ']']);
    ["127.0.0.1", "localhost", "::1"].contains(&hote)
}

/// json.dumps(ensure_ascii=False) : « , » et « : » suivis d'une espace.
struct CommePython;

impl serde_json::ser::Formatter for CommePython {
    fn begin_array_value<W: ?Sized + Write>(&mut self, w: &mut W, premier: bool) -> io::Result<()> {
        w.write_all(if premier { b"" } else { b", " })
    }
    fn begin_object_key<W: ?Sized + Write>(&mut self, w: &mut W, premier: bool) -> io::Result<()> {
        w.write_all(if premier { b"" } else { b", " })
    }
    fn begin_object_value<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        w.write_all(b": ")
    }
}

fn rendre_json(code: u16, donnees: &Value) -> Reponse {
    let mut texte = Vec::new();
    donnees.serialize(&mut serde_json::Serializer::with_formatter(&mut texte, CommePython)).expect("écriture en mémoire");
    Reponse::new(code, "application/json", texte)
}

/// Les pages de l'outil à essayer : l'accueil, les autres pages à la racine, et chaque
/// <dossier>/<dossier>.html (les sous-outils de WEB_CAO).
fn pages_de(dossier: &Path) -> Vec<Value> {
    let mut noms: Vec<String> =
        fs::read_dir(dossier).into_iter().flatten().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    noms.sort();
    let mut pages = Vec::new();
    if noms.iter().any(|f| f == "index.html") {
        pages.push(json!({"url": "/index.html", "nom": "Accueil"}));
    }
    for f in noms.iter().filter(|f| f.ends_with(".html") && *f != "index.html") {
        pages.push(json!({"url": format!("/{}", f), "nom": &f[..f.len() - 5]}));
    }
    for d in &noms {
        if DOSSIERS_IGNORES.contains(&d.as_str()) || d.starts_with('.') {
            continue;
        }
        if dossier.join(d).join(format!("{}.html", d)).is_file() {
            pages.push(json!({"url": format!("/{}/{}.html", d, d), "nom": d}));
        }
    }
    pages
}

/// La sonde en tout premier dans <head> : elle voit les erreurs dès le premier script.
/// (l'ancienne expression : <head[^>]*>, sans égard à la casse)
fn injecter_sonde(page: &[u8]) -> Vec<u8> {
    let fin = page
        .windows(5)
        .position(|f| f.eq_ignore_ascii_case(b"<head"))
        .and_then(|debut| page[debut..].iter().position(|&c| c == b'>').map(|i| debut + i + 1))
        .unwrap_or(0);
    [&page[..fin], SONDE, &page[fin..]].concat()
}

struct Amont {
    code: u16,
    entetes: Vec<(String, String)>,
    corps: Vec<u8>,
}

fn invalide(quoi: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, quoi.to_string())
}

fn ligne(lecteur: &mut impl BufRead) -> io::Result<String> {
    let mut l = Vec::new();
    lecteur.read_until(b'\n', &mut l)?;
    Ok(String::from_utf8_lossy(&l).trim_end_matches(['\r', '\n']).to_string())
}

/// Une requête HTTP/1.1 à l'outil, telle que http.client l'envoyait : les en-têtes du
/// navigateur (Host compris) passent tels quels, aucune redirection n'est suivie, et la
/// réponse revient entière, quel que soit son code.
// shortcut: la réponse est lue en entier avant d'être rendue (le serveur commun n'écrit pas
// au fil de l'eau) ; à revoir si un outil se met à répondre en flux (text/event-stream).
fn demander(port: u16, methode: &str, cible: &str, entetes: &[(String, String)], corps: Option<&[u8]>) -> io::Result<Amont> {
    let mut flux = TcpStream::connect(("127.0.0.1", port))?;
    flux.set_read_timeout(Some(Duration::from_secs(3600)))?;
    flux.set_write_timeout(Some(Duration::from_secs(3600)))?;
    let mut requete = format!("{} {} HTTP/1.1\r\nAccept-Encoding: identity\r\n", methode, cible).into_bytes();
    match corps {
        Some(c) => write!(requete, "Content-Length: {}\r\n", c.len())?,
        None if ["PATCH", "POST", "PUT"].contains(&methode) => requete.extend_from_slice(b"Content-Length: 0\r\n"),
        None => {}
    }
    for (k, v) in entetes {
        write!(requete, "{}: {}\r\n", k, v)?;
    }
    requete.extend_from_slice(b"\r\n");
    requete.extend_from_slice(corps.unwrap_or(b""));
    flux.write_all(&requete)?;
    let mut lecteur = BufReader::new(flux);
    lire_reponse(&mut lecteur, methode)
}

fn lire_reponse(lecteur: &mut impl BufRead, methode: &str) -> io::Result<Amont> {
    let (code, entetes) = loop {
        let statut = ligne(lecteur)?;
        let code: u16 =
            statut.split(' ').nth(1).and_then(|c| c.parse().ok()).ok_or_else(|| invalide("ligne de statut illisible"))?;
        let mut entetes = Vec::new();
        loop {
            let l = ligne(lecteur)?;
            if l.is_empty() {
                break;
            }
            if let Some((k, v)) = l.split_once(':') {
                entetes.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
        if code != 100 {
            break (code, entetes); // « 100 Continue » : la vraie réponse suit
        }
    };
    let de = |nom: &str| entetes.iter().find(|(k, _)| k.eq_ignore_ascii_case(nom)).map(|(_, v)| v.as_str());
    let mut corps = Vec::new();
    if methode == "HEAD" || code < 200 || code == 204 || code == 304 {
    } else if de("Transfer-Encoding").is_some_and(|t| t.to_ascii_lowercase().contains("chunked")) {
        loop {
            let taille = ligne(lecteur)?;
            let n = usize::from_str_radix(taille.split(';').next().unwrap_or("").trim(), 16)
                .map_err(|_| invalide("taille de bloc illisible"))?;
            if n == 0 {
                break;
            }
            let debut = corps.len();
            corps.resize(debut + n, 0);
            lecteur.read_exact(&mut corps[debut..])?;
            ligne(lecteur)?;
        }
    } else if let Some(n) = de("Content-Length").and_then(|n| n.trim().parse::<usize>().ok()) {
        corps.resize(n, 0);
        lecteur.read_exact(&mut corps)?;
    } else {
        lecteur.read_to_end(&mut corps)?;
    }
    Ok(Amont { code, entetes, corps })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sonde_en_tete() {
        assert_eq!(
            injecter_sonde(b"<html><head><title>t</title></head></html>"),
            b"<html><head><script src=\"/__banc/sonde.js\"></script><title>t</title></head></html>"
        );
        assert_eq!(
            injecter_sonde(b"<HEAD lang=\"fr\"><meta>"),
            b"<HEAD lang=\"fr\"><script src=\"/__banc/sonde.js\"></script><meta>"
        );
        assert_eq!(injecter_sonde(b"<p>sans tete</p>"), b"<script src=\"/__banc/sonde.js\"></script><p>sans tete</p>");
        assert_eq!(injecter_sonde(b"<head"), b"<script src=\"/__banc/sonde.js\"></script><head");
    }

    #[test]
    fn garde_du_nom() {
        for h in ["127.0.0.1:8200", "localhost", "LOCALHOST:1", "[::1]:8200", "127.0.0.1"] {
            assert!(hote_local(h), "{}", h);
        }
        for h in ["", "piege.exemple:8200", "192.168.1.2:8200", "127.0.0.1.piege.exemple", "localhost.piege:80"] {
            assert!(!hote_local(h), "{}", h);
        }
    }

    #[test]
    fn json_comme_python() {
        let v = json!({"erreur": "en-tête manquant", "liste": [1, null, true], "vide": {}, "n": 0.5});
        let r = rendre_json(403, &v);
        assert_eq!(String::from_utf8(r.corps).unwrap(), r#"{"erreur": "en-tête manquant", "liste": [1, null, true], "vide": {}, "n": 0.5}"#);
        assert_eq!(r.entetes[0], ("Content-Type".to_string(), "application/json".to_string()));
    }

    #[test]
    fn parametres_et_noms() {
        let p = Params(params("nom=a.json&vide=&nom=b.json&id=test%2Fbanc-bon.py"));
        assert_eq!(p.get("nom"), Some("b.json"));
        assert_eq!(p.get("vide"), None);
        assert_eq!(p.get("id"), Some("test/banc-bon.py"));
        assert_eq!(nom_de_base("../../etc/passwd"), "passwd");
        assert_eq!(nom_de_base("a/.."), "..");
        assert_eq!(ascii("Arrêté"), ["Arr", "u00eat", "u00e9"].join(&'\\'.to_string()));
        assert!(saute("Content-Length") && saute("accept-encoding") && !saute("Content-Type"));
    }

    #[test]
    fn reponses_de_l_outil() {
        let lire = |brut: &[u8], methode: &str| lire_reponse(&mut BufReader::new(brut), methode).unwrap();
        let r = lire(b"HTTP/1.0 404 Not Found\r\nContent-Type: text/plain\r\nSet-Cookie: a=1\r\nSet-Cookie: b=2\r\n\r\nnon", "GET");
        assert_eq!((r.code, r.corps.as_slice(), r.entetes.len()), (404, &b"non"[..], 3));
        let r = lire(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nabcd", "GET");
        assert_eq!(r.corps, b"ab");
        let r = lire(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\nA;x=1\r\n0123456789\r\n0\r\n\r\n", "GET");
        assert_eq!(r.corps, b"abc0123456789");
        let r = lire(b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 302 Found\r\nLocation: /ailleurs\r\nContent-Length: 0\r\n\r\n", "POST");
        assert_eq!((r.code, r.entetes[0].1.as_str()), (302, "/ailleurs"));
        let r = lire(b"HTTP/1.0 200 OK\r\nContent-Length: 12\r\n\r\n", "HEAD");
        assert!(r.corps.is_empty());
        assert!(lire_reponse(&mut BufReader::new(&b"n'importe quoi\r\n\r\n"[..]), "GET").is_err());
    }
}
