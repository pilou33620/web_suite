//! Canal de pilotage : le MCP de web_suite donne des ordres aux pages ouvertes d'un web tool.
//!
//! Spec : web_suite/lanceur/docs/SPEC-mcp-canal-pilote.md.
//!
//! Usage dans un serveur : `pilote.traiter(&mut requete)` en tête de chaque requête
//! (après le contrôle d'Origin du serveur pour un POST) ; `Some(réponse)` est à
//! renvoyer telle quelle, `None` laisse la requête suivre son cours.
//!
//! Les pages d'un même navigateur élisent un onglet relais : lui seul bat
//! (/battement), attend les commandes (/attendre, long-poll) et les transmet aux
//! autres onglets. La page visée répond (/reponse). Le MCP, sur ce poste
//! seulement, liste les onglets, dépose une commande et en suit l'état.
//!
//! Dans l'autre sens, le panneau « Claude » d'une page envoie une demande écrite
//! (/demande) et en lit la conversation (/conversation, long-poll). Le lanceur de
//! web_suite, sur ce poste seulement, les prend (/demandes, long-poll), les passe
//! à Claude Code et y écrit les réponses (/progres).

use std::collections::HashMap;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Duration;

use base64::Engine;
use serde_json::{json, Map, Value};

use crate::appairage::{deballer, horloge, mono};
use crate::serveur::{ErreurCorps, Reponse, Requete};

pub const PREFIXE: &str = "/api/pilote/";
pub const METHODES: &[(&str, &str)] = &[
    ("battement", "POST"),
    ("attendre", "GET"),
    ("reponse", "POST"),
    ("commande", "POST"),
    ("onglets", "GET"),
    ("suivre", "GET"),
    ("demande", "POST"),
    ("conversation", "GET"),
    ("demandes", "GET"),
    ("progres", "POST"),
    ("statut", "GET"),
];
/// le MCP et le lanceur, sur ce poste seulement
pub const ORDRES: &[&str] = &["commande", "onglets", "suivre", "demandes", "progres"];
pub const ETATS_PAGE: &[&str] = &["recue", "proposee", "appliquee", "refusee", "finie", "echouee"];
pub const FINAUX: &[&str] = &["appliquee", "refusee", "finie", "echouee", "expiree"];
pub const GARDE_COMMANDES: f64 = 3600.0; // s : une commande finie reste lisible par /suivre pendant une heure
pub const MAX_COMMANDES: usize = 500; // au-delà, les plus vieilles commandes finies sont oubliées
pub const MAX_ONGLETS: usize = 200; // tous navigateurs confondus : au-delà, un battement est refusé (429)
/// une réponse porte une capture, une commande un fichier (kicad_mod, Touchstone) ; les commandes ne viennent
/// que du MCP de ce poste (ORDRES). Battements et suivis restent petits.
pub const GROS: &[&str] = &["reponse", "commande"];
pub const MAX_TEXTE: usize = 8000; // caractères d'une demande écrite dans le panneau
pub const MAX_MESSAGE: usize = 20000; // caractères d'un message de Claude rendu au panneau
pub const MAX_MESSAGES: usize = 500; // messages gardés par demande
pub const PRISE: f64 = 20.0; // s : une demande que personne ne prend échoue (aucun lanceur sur ce poste)
pub const GENRES: &[&str] = &["texte", "outil", "erreur", "info"];
pub const MODELES: &[&str] = &["opus", "sonnet", "haiku"]; // alias de Claude Code (--model) : le dernier modèle de chaque famille
pub const LANCEUR_VIVANT: f64 = 35.0; // s : sans long-poll /demandes depuis, le lanceur est absent (poll : 25 s)
pub const TEST_PERIME: f64 = 30.0; // s : au-delà, l'ouverture d'un panneau fait retester Claude Code
/// Pièces jointes d'une demande (SPEC-claude-pages.md) : lues par Claude, jamais écrites sur disque
pub const TYPES_PIECE: &[&str] = &["image/png", "image/jpeg", "image/gif", "image/webp", "application/pdf"];
pub const MAX_PIECES: usize = 10;
pub const MAX_PIECES_OCTETS: usize = 20 * 1024 * 1024; // au total, après réduction des images par la page
pub const MAX_IMAGE: usize = 3932160; // 3,75 Mo : 5 Mo en base64, la limite de l'API par image
pub const MAX_PIECES_TEXTE: usize = 400000; // caractères de texte joint : ≈ 100 000 jetons du contexte
pub const MAX_EN_ATTENTE: usize = 64 * 1024 * 1024; // octets de pièces gardés en mémoire, toutes demandes confondues

/// (MAX_PIECES_OCTETS, MAX_EN_ATTENTE) ; le banc d'essai les abaisse.
fn plafonds() -> (usize, usize) {
    #[cfg(test)]
    return essais::plafonds();
    #[cfg(not(test))]
    (MAX_PIECES_OCTETS, MAX_EN_ATTENTE)
}

#[derive(Debug)]
pub struct Refus {
    pub code: u16,
    pub detail: String,
}

type Rendu = Result<Value, Refus>;

fn refus<T>(code: u16, detail: impl Into<String>) -> Result<T, Refus> {
    Err(Refus { code, detail: detail.into() })
}

fn fini(etat: &str) -> bool {
    FINAUX.contains(&etat)
}

/// La valeur est une des chaînes de la liste.
fn parmi(liste: &[&'static str], v: Option<&Value>) -> Option<&'static str> {
    let v = v?.as_str()?;
    liste.iter().copied().find(|e| *e == v)
}

/// ^[A-Za-z0-9_-]{1,64}$
fn id(v: Option<&Value>) -> Option<&str> {
    let s = v?.as_str()?;
    let ok = (1..=64).contains(&s.len()) && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-');
    ok.then_some(s)
}

/// ^[a-z_][a-z0-9_]{0,63}$
fn nom_valide(s: &str) -> bool {
    let b = s.as_bytes();
    (1..=64).contains(&b.len())
        && !b[0].is_ascii_digit()
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
}

fn texte(v: Option<&Value>, nom: &str, maxi: usize, vide: bool) -> Result<String, Refus> {
    match v.and_then(Value::as_str) {
        Some(s) if s.chars().count() <= maxi && (vide || !s.is_empty()) => Ok(s.to_string()),
        _ => refus(400, format!("Champ « {} » invalide.", nom)),
    }
}

/// Un nombre fini et positif (ni booléen ni chaîne).
fn positif(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64().filter(|x| x.is_finite() && *x >= 0.0),
        _ => None,
    }
}

/// Les pièces jointes d'une demande, vérifiées : (liste, octets). Une image ou un PDF porte
/// ses octets en base64 (« donnees »), un fichier texte son contenu (« texte »).
fn pieces(v: Option<Value>) -> Result<(Vec<Value>, usize), Refus> {
    use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
    let liste = match v {
        None | Some(Value::Null) => return Ok((Vec::new(), 0)),
        Some(Value::Array(l)) if l.len() <= MAX_PIECES => l,
        _ => return refus(400, format!("Au plus {} pièces jointes.", MAX_PIECES)),
    };
    // comme base64.b64decode(validate=True) de Python : les bits de fin non nuls passent
    let b64 = GeneralPurpose::new(&base64::alphabet::STANDARD, GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true));
    let (mut rendu, mut octets, mut caracteres) = (Vec::new(), 0, 0);
    for p in liste {
        let Value::Object(mut p) = p else { return refus(400, "Pièce jointe invalide.") };
        let nom = texte(p.get("nom"), "nom", 200, false)?;
        if nom.chars().any(|c| (c as u32) < 32) {
            return refus(400, "Nom de pièce jointe invalide.");
        }
        if let Some(t) = p.remove("texte") {
            let contenu = texte(Some(&t), "texte", MAX_PIECES_TEXTE, true)?;
            caracteres += contenu.chars().count();
            octets += contenu.len();
            rendu.push(json!({"nom": nom, "texte": contenu}));
            continue;
        }
        let (Some(type_), Some(Value::String(donnees))) = (parmi(TYPES_PIECE, p.get("type")), p.remove("donnees")) else {
            return refus(400, format!("« {} » : type de fichier non lu par Claude.", nom));
        };
        let Ok(taille) = b64.decode(&donnees).map(|o| o.len()) else {
            return refus(400, format!("« {} » : contenu illisible.", nom));
        };
        if type_ != "application/pdf" && taille > MAX_IMAGE {
            return refus(413, format!("« {} » : image de plus de 3,75 Mo.", nom));
        }
        octets += taille;
        rendu.push(json!({"nom": nom, "type": type_, "donnees": donnees}));
    }
    if octets > plafonds().0 {
        return refus(413, "Pièces jointes de plus de 20 Mo au total.");
    }
    if caracteres > MAX_PIECES_TEXTE {
        return refus(413, format!("Plus de {} caractères de texte joint.", MAX_PIECES_TEXTE));
    }
    Ok((rendu, octets))
}

fn jeton_hex() -> String {
    let mut b = [0u8; 16];
    getrandom::getrandom(&mut b).expect("générateur aléatoire du système");
    b.iter().map(|o| format!("{:02x}", o)).collect()
}

/// La requête vient-elle de ce poste (boucle locale) ?
pub fn ce_poste(r: &Requete) -> bool {
    #[cfg(test)]
    if essais::DISTANT.load(std::sync::atomic::Ordering::SeqCst) {
        return false;
    }
    deballer(r.client.ip()).is_loopback()
}

#[derive(Clone, Debug)]
pub struct Reglages {
    pub attente: f64,        // s : un appel du MCP attend la réponse de la page
    pub attente_poll: f64,   // s : un long-poll du relais
    pub oubli: f64,          // s : un onglet sans nouvelles est oublié (> 90 s de pilote.js)
    pub redistribution: f64, // s : livrée mais pas accusée, la commande repart
    pub max_corps: usize,    // octets : une capture d'écran, un fichier dans une commande
    pub max_petit: usize,    // octets : battement, suivi, progrès
    pub max_demande: usize,  // octets : une demande et ses pièces jointes (20 Mo en base64)
    pub prise: f64,          // s : une demande écrite attend le lanceur
    pub attente_test: f64,   // s : la page attend le test de Claude Code par le lanceur
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages {
            attente: 120.0,
            attente_poll: 25.0,
            oubli: 120.0,
            redistribution: 10.0,
            max_corps: 8 * 1024 * 1024,
            max_petit: 64 * 1024,
            max_demande: 28 * 1024 * 1024,
            prise: PRISE,
            attente_test: 8.0,
        }
    }
}

#[derive(Clone)]
struct Onglet {
    id: String,
    navigateur: String,
    page: String,
    projet: String,
    interrupteur: bool,
    focus: f64, // horloge du serveur
    appareil: String,
    vu: f64,
}

struct Commande {
    id: String,
    onglet: String,
    navigateur: String,
    nom: String,
    args: Value,
    ecriture: bool,
    etat: &'static str,
    resultat: Value,
    cree: f64,
    livree: f64,
    fin: f64,
    cible: Onglet,
}

struct Demande {
    id: String,
    navigateur: String,
    onglet: String,
    texte: String,
    pieces: Vec<Value>,
    poids: usize,
    modele: &'static str,
    nouvelle: bool,
    etat: &'static str,
    messages: Vec<Value>,
    cree: f64,
    vu: f64,
    fin: f64,
    page: String,
    projet: String,
    appareil: String,
}

/// Les listes gardent l'ordre d'arrivée, comme les dict de Python.
struct Etat {
    onglets: Vec<Onglet>,
    commandes: Vec<Commande>,
    polls: HashMap<String, u64>, // navigateur -> jeton du long-poll en cours
    jeton: u64,
    demandes: Vec<Demande>, // écrites dans un panneau « Claude »
    prise: f64,
    attente_test: f64,
    lanceur_vu: f64, // dernier long-poll /demandes du lanceur
    claude: Value,   // état de Claude Code au dernier test du lanceur
    claude_t: f64,
    tester: bool, // une page attend un nouveau test
}

impl Etat {
    fn onglet(&self, id: &str) -> Option<&Onglet> {
        self.onglets.iter().find(|o| o.id == id)
    }

    fn commande(&mut self, id: &str) -> Option<&mut Commande> {
        self.commandes.iter_mut().find(|c| c.id == id)
    }

    fn demande(&mut self, id: &str) -> Option<&mut Demande> {
        self.demandes.iter_mut().find(|d| d.id == id)
    }

    /// Oublie les onglets muets ; une commande encore en cours pour eux expire.
    /// Vrai s'il faut réveiller ceux qui attendent.
    fn menage(&mut self, oubli: f64) -> bool {
        let maintenant = mono();
        let mut reveil = false;
        self.onglets.retain(|o| maintenant - o.vu <= oubli);
        let onglets = &self.onglets;
        self.commandes.retain_mut(|c| {
            let expire = |c: &mut Commande, resultat: &str| {
                c.etat = "expiree";
                c.resultat = json!(resultat);
                c.fin = mono();
            };
            if !fini(c.etat) && !onglets.iter().any(|o| o.id == c.onglet) {
                expire(c, "L'onglet visé a été fermé avant de répondre.");
                reveil = true;
            } else if !fini(c.etat) && maintenant - c.cree > GARDE_COMMANDES {
                expire(c, "La page n'a jamais répondu.");
                reveil = true;
            } else if fini(c.etat) && maintenant - c.fin > GARDE_COMMANDES {
                return false;
            }
            true
        });
        let trop = self.commandes.len().saturating_sub(MAX_COMMANDES);
        if trop > 0 {
            let mut finies: Vec<(f64, String)> = self.commandes.iter().filter(|c| fini(c.etat)).map(|c| (c.fin, c.id.clone())).collect();
            finies.sort_by(|a, b| a.0.total_cmp(&b.0));
            finies.truncate(trop);
            self.commandes.retain(|c| !finies.iter().any(|(_, i)| *i == c.id));
        }
        let prise = self.prise;
        self.demandes.retain_mut(|d| {
            if d.etat == "en_attente" && maintenant - d.cree > prise {
                d.messages.push(json!({"genre": "erreur", "texte": "Aucun lanceur WEB·SUITE ne répond sur ce poste : \
                    démarre l'outil depuis WEB·SUITE pour écrire à Claude."}));
                (d.etat, d.fin, d.pieces, d.poids) = ("echouee", maintenant, Vec::new(), 0);
                reveil = true;
            } else if d.etat == "prise" && maintenant - d.vu > GARDE_COMMANDES {
                (d.etat, d.fin) = ("echouee", maintenant);
            } else if fini(d.etat) && maintenant - d.fin > GARDE_COMMANDES {
                return false;
            }
            true
        });
        reveil
    }

    fn vue(&self, c: &Commande) -> Value {
        let o = self.onglet(&c.onglet).unwrap_or(&c.cible);
        json!({"id": c.id, "nom": c.nom, "etat": c.etat, "resultat": c.resultat,
               "onglet": {"id": o.id, "page": o.page, "projet": o.projet, "appareil": o.appareil}})
    }
}

fn vue_onglet(o: &Onglet) -> Value {
    json!({"id": o.id, "page": o.page, "projet": o.projet, "appareil": o.appareil,
           "navigateur": o.navigateur, "interrupteur": o.interrupteur,
           "focus_il_y_a_s": ((horloge() - o.focus) * 10.0).round() / 10.0})
}

pub struct Pilote {
    pub reglages: Reglages,
    etat: Mutex<Etat>,
    cond: Condvar,
}

impl Default for Pilote {
    fn default() -> Self {
        Self::new(Reglages::default())
    }
}

impl Pilote {
    pub fn new(reglages: Reglages) -> Self {
        let etat = Etat {
            onglets: Vec::new(),
            commandes: Vec::new(),
            polls: HashMap::new(),
            jeton: 0,
            demandes: Vec::new(),
            prise: reglages.prise,
            attente_test: reglages.attente_test,
            lanceur_vu: f64::NEG_INFINITY,
            claude: Value::Null,
            claude_t: f64::NEG_INFINITY,
            tester: false,
        };
        Pilote { reglages, etat: Mutex::new(etat), cond: Condvar::new() }
    }

    // -- entrée -------------------------------------------------------------
    /// `Some(réponse)` si la route était au pilote, `None` sinon.
    pub fn traiter(&self, r: &mut Requete) -> Option<Reponse> {
        let route = r.chemin().strip_prefix(PREFIXE)?.to_string();
        Some(match self.router(r, &route) {
            Ok(rendu) => Reponse::json(200, &rendu),
            Err(x) => Reponse::json(x.code, &json!({"detail": x.detail})),
        })
    }

    fn router(&self, r: &mut Requete, route: &str) -> Rendu {
        let Some(&(_, methode)) = METHODES.iter().find(|(n, _)| *n == route) else {
            return refus(404, format!("Route du pilote inconnue : {}", route));
        };
        if r.methode != methode {
            return refus(405, format!("Méthode {} refusée sur {}.", r.methode, route));
        }
        let origine = r.entete("Origin").filter(|o| !o.is_empty()).map(str::to_lowercase);
        let hote = r.entete("Host").unwrap_or("").to_lowercase();
        // HTTP ou HTTPS : le mode réseau sert HTTPS sur le même port (appairage.rs)
        if r.methode == "POST" && origine.as_ref().is_some_and(|o| *o != format!("http://{}", hote) && *o != format!("https://{}", hote)) {
            // Un autre site ouvert dans le navigateur : refusé ici, même si le serveur hôte
            // ne contrôle pas l'Origin (WEB_3D n'a pas de garde CSRF)
            return refus(403, "Origine refusée.");
        }
        // Une page du poste est aussi en boucle locale : mais un navigateur envoie Origin ou
        // Sec-Fetch-Site, que le MCP n'envoie jamais
        if ORDRES.contains(&route) && (!ce_poste(r) || origine.is_some() || r.entete("Sec-Fetch-Site").is_some_and(|v| !v.is_empty())) {
            return refus(403, "Seul le MCP de ce poste donne des ordres aux pages.");
        }
        let reg = &self.reglages;
        let maxi = if GROS.contains(&route) {
            reg.max_corps
        } else if route == "demande" {
            reg.max_demande
        } else {
            reg.max_petit
        };
        let corps = if r.methode == "POST" { Self::corps(r, maxi)? } else { Map::new() };
        match route {
            "battement" => self.battement(r, &corps),
            "attendre" => self.attendre(r),
            "reponse" => self.reponse(corps),
            "commande" => self.commande(corps),
            "onglets" => self.onglets(),
            "suivre" => self.suivre(r),
            "demande" => self.demande(corps),
            "conversation" => self.conversation(r),
            "demandes" => self.demandes(r),
            "progres" => self.progres(&corps),
            _ => self.statut(),
        }
    }

    fn corps(r: &mut Requete, maxi: usize) -> Result<Map<String, Value>, Refus> {
        let brut = match r.lire_corps(maxi) {
            Ok(b) => b,
            Err(ErreurCorps::Invalide) => return refus(400, "Content-Length invalide."),
            Err(ErreurCorps::TropGrand) => return refus(413, "Corps trop gros."), // le corps n'est pas lu
            Err(ErreurCorps::Lecture(_)) => return refus(400, "JSON illisible."),
        };
        let brut: &[u8] = if brut.is_empty() { b"null" } else { &brut };
        match serde_json::from_slice(brut) {
            Ok(Value::Object(corps)) => Ok(corps),
            Ok(_) => refus(400, "Objet JSON attendu."),
            Err(_) => refus(400, "JSON illisible."), // dont [[[[… très imbriqué
        }
    }

    // -- état (sous le verrou) ------------------------------------------------
    fn verrou(&self) -> MutexGuard<'_, Etat> {
        self.etat.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn dormir<'a>(&self, e: MutexGuard<'a, Etat>, secondes: f64) -> MutexGuard<'a, Etat> {
        let duree = Duration::from_secs_f64(secondes.max(0.0));
        self.cond.wait_timeout(e, duree).unwrap_or_else(|e| e.into_inner()).0
    }

    fn menage(&self, e: &mut Etat) {
        if e.menage(self.reglages.oubli) {
            self.cond.notify_all();
        }
    }

    // -- routes des pages ---------------------------------------------------
    fn battement(&self, r: &Requete, corps: &Map<String, Value>) -> Rendu {
        let Some(nav) = id(corps.get("navigateur")) else { return refus(400, "Champ « navigateur » invalide.") };
        let liste = match corps.get("onglets") {
            Some(Value::Array(l)) if l.len() <= 50 => l,
            _ => return refus(400, "Champ « onglets » invalide."),
        };
        let appareil = if ce_poste(r) { "ce poste".to_string() } else { r.client.ip().to_string() };
        let mut neufs: Vec<Onglet> = Vec::new();
        for o in liste {
            let Some(i) = id(o.get("id")).filter(|_| o.is_object()) else { return refus(400, "Onglet invalide.") };
            let Some(il_y_a) = o.get("focus_il_y_a").map_or(Some(0.0), positif) else {
                return refus(400, "Champ « focus_il_y_a » invalide.");
            };
            let page = texte(o.get("page"), "page", 40, false)?;
            // « o.get("projet") or "" » de Python : nul, faux, zéro et vide valent ""
            let projet = match o.get("projet") {
                None | Some(Value::Null) | Some(Value::Bool(false)) => String::new(),
                Some(Value::Number(n)) if n.as_f64() == Some(0.0) => String::new(),
                Some(Value::Array(a)) if a.is_empty() => String::new(),
                Some(Value::Object(m)) if m.is_empty() => String::new(),
                v => texte(v, "projet", 200, true)?,
            };
            let neuf = Onglet {
                id: i.to_string(),
                navigateur: nav.to_string(),
                page,
                projet,
                interrupteur: o.get("interrupteur") == Some(&Value::Bool(true)),
                // Le focus est daté par le serveur : les horloges des appareils diffèrent
                focus: horloge() - il_y_a / 1000.0,
                appareil: appareil.clone(),
                vu: mono(),
            };
            match neufs.iter_mut().find(|n| n.id == neuf.id) {
                Some(n) => *n = neuf,
                None => neufs.push(neuf),
            }
        }
        let mut e = self.verrou();
        // Un id d'onglet tenu par un autre navigateur ne se reprend pas (onglet volé)
        if neufs.iter().any(|n| e.onglet(&n.id).is_some_and(|o| o.navigateur != nav)) {
            return refus(409, "Onglet déjà tenu par un autre navigateur.");
        }
        let autres = e.onglets.iter().filter(|o| o.navigateur != nav).count();
        if autres + neufs.len() > MAX_ONGLETS {
            return refus(429, "Trop d'onglets branchés.");
        }
        // La liste du relais fait foi : un onglet absent est fermé
        e.onglets.retain(|o| o.navigateur != nav);
        e.onglets.extend(neufs);
        self.menage(&mut e);
        Ok(json!({"ok": true}))
    }

    fn attendre(&self, r: &Requete) -> Rendu {
        let nav = r.param("navigateur").map(Value::String);
        let Some(nav) = id(nav.as_ref()) else { return refus(400, "Paramètre « navigateur » invalide.") };
        let reg = &self.reglages;
        let fin = mono() + reg.attente_poll;
        let mut e = self.verrou();
        // Un nouveau long-poll du même navigateur remplace l'ancien, dont le client est
        // peut-être parti (relais fermé ou rétrogradé) : l'ancien rend vide et s'arrête
        e.jeton += 1;
        let jeton = e.jeton;
        e.polls.insert(nav.to_string(), jeton);
        self.cond.notify_all();
        let maintenant = loop {
            let maintenant = mono();
            if e.polls.get(nav) != Some(&jeton) {
                return Ok(json!({"commandes": []}));
            }
            for o in e.onglets.iter_mut().filter(|o| o.navigateur == nav) {
                o.vu = maintenant;
            }
            self.menage(&mut e);
            let pret = e.commandes.iter().any(|c| Self::a_livrer(c, nav, maintenant, reg.redistribution));
            if pret || maintenant >= fin {
                break maintenant;
            }
            e = self.dormir(e, (fin - maintenant).min(reg.redistribution));
        };
        let mut prets: Vec<&mut Commande> =
            e.commandes.iter_mut().filter(|c| Self::a_livrer(c, nav, maintenant, reg.redistribution)).collect();
        prets.sort_by(|a, b| a.cree.total_cmp(&b.cree));
        let mut commandes = Vec::new();
        for c in prets {
            (c.etat, c.livree) = ("livree", maintenant);
            commandes.push(json!({"id": c.id, "onglet": c.onglet, "nom": c.nom, "args": c.args, "ecriture": c.ecriture}));
        }
        Ok(json!({"commandes": commandes}))
    }

    fn a_livrer(c: &Commande, nav: &str, maintenant: f64, redistribution: f64) -> bool {
        c.navigateur == nav && (c.etat == "a_livrer" || (c.etat == "livree" && maintenant - c.livree > redistribution))
    }

    fn reponse(&self, mut corps: Map<String, Value>) -> Rendu {
        let (Some(etat), Some(Value::String(i))) = (parmi(ETATS_PAGE, corps.get("etat")), corps.get("id").cloned()) else {
            return refus(400, "Champ « etat » ou « id » invalide.");
        };
        let mut e = self.verrou();
        // Seul le navigateur de la commande répond : un autre ne forge pas ce que Claude lit
        let Some(c) = e.commande(&i).filter(|c| corps.get("navigateur").and_then(Value::as_str) == Some(&c.navigateur)) else {
            return refus(404, "Commande inconnue.");
        };
        // Un état final ne change plus, sauf « expiree », qui n'était qu'une supposition du serveur
        if !fini(c.etat) || (c.etat == "expiree" && fini(etat)) {
            if fini(etat) {
                (c.etat, c.resultat, c.fin) = (etat, corps.remove("resultat").unwrap_or(Value::Null), mono());
                self.cond.notify_all();
            } else if etat == "proposee" || c.etat == "a_livrer" || c.etat == "livree" {
                // « recue » ne recule pas
                c.etat = etat;
                self.cond.notify_all();
            }
        }
        Ok(json!({"ok": true}))
    }

    fn demande(&self, mut corps: Map<String, Value>) -> Rendu {
        let (Some(nav), Some(onglet)) = (id(corps.get("navigateur")), id(corps.get("onglet"))) else {
            return refus(400, "Champ « navigateur » ou « onglet » invalide.");
        };
        let (nav, onglet) = (nav.to_string(), onglet.to_string());
        let texte = texte(corps.get("texte"), "texte", MAX_TEXTE, false)?.trim().to_string();
        if texte.is_empty() {
            return refus(400, "Champ « texte » vide.");
        }
        let modele = match corps.get("modele") {
            None => MODELES[0],
            v => match parmi(MODELES, v) {
                Some(m) => m,
                None => return refus(400, "Champ « modele » invalide."),
            },
        };
        let (pieces, poids) = pieces(corps.remove("pieces"))?;
        let mut e = self.verrou();
        self.menage(&mut e);
        let Some(o) = e.onglet(&onglet).filter(|o| o.navigateur == nav) else {
            return refus(404, "Onglet pas encore connu du serveur : réessaie dans quelques secondes.");
        };
        if e.demandes.iter().any(|d| d.navigateur == nav && !fini(d.etat)) {
            return refus(409, "Claude travaille déjà sur ta demande précédente.");
        }
        if poids > 0 && e.demandes.iter().map(|d| d.poids).sum::<usize>() + poids > plafonds().1 {
            return refus(429, "Trop de fichiers en attente sur le poste : réessaie dans un instant.");
        }
        let d = Demande {
            id: jeton_hex(),
            page: o.page.clone(),
            projet: o.projet.clone(),
            appareil: o.appareil.clone(),
            navigateur: nav,
            onglet,
            texte,
            pieces,
            poids,
            modele,
            nouvelle: corps.get("nouvelle") == Some(&Value::Bool(true)),
            etat: "en_attente",
            messages: Vec::new(),
            cree: mono(),
            vu: mono(),
            fin: 0.0,
        };
        let rendu = json!({"id": d.id});
        e.demandes.push(d);
        self.cond.notify_all();
        Ok(rendu)
    }

    fn conversation(&self, r: &Requete) -> Rendu {
        let depuis = r.param("depuis").filter(|d| d.bytes().all(|c| c.is_ascii_digit()));
        let depuis: usize = depuis.map_or(0, |d| d.parse().unwrap_or(usize::MAX));
        let (i, nav) = (r.param("id").unwrap_or_default(), r.param("navigateur"));
        let fin = mono() + self.reglages.attente_poll;
        let mut e = self.verrou();
        loop {
            // Seul le navigateur de la demande la lit : la conversation peut citer le projet
            let Some(d) = e.demande(&i).filter(|d| nav.as_deref() == Some(&d.navigateur)) else {
                return refus(404, "Demande inconnue.");
            };
            if d.messages.len() > depuis || fini(d.etat) || mono() >= fin {
                let suite = &d.messages[depuis.min(d.messages.len())..];
                return Ok(json!({"etat": d.etat, "messages": suite, "total": d.messages.len()}));
            }
            e = self.dormir(e, fin - mono());
            self.menage(&mut e);
        }
    }

    // -- routes du MCP et du lanceur (ce poste) -------------------------------
    fn commande(&self, mut corps: Map<String, Value>) -> Rendu {
        let pages: Option<Vec<String>> = match corps.get("page") {
            Some(Value::String(p)) => Some(vec![p.clone()]),
            Some(Value::Array(l)) if !l.is_empty() => l.iter().map(|p| p.as_str().map(str::to_string)).collect(),
            _ => None,
        };
        let Some(pages) = pages else { return refus(400, "Champ « page » invalide.") };
        let ou = pages.join(" ou ");
        let Some(nom) = corps.get("nom").and_then(Value::as_str).filter(|n| nom_valide(n)).map(str::to_string) else {
            return refus(400, "Champ « nom » invalide.");
        };
        let args = match corps.remove("args") {
            None => json!({}),
            Some(a @ Value::Object(_)) => a,
            _ => return refus(400, "Champ « args » invalide : objet attendu."),
        };
        let attente = corps.get("attente").and_then(positif).map_or(self.reglages.attente, |a| a.min(self.reglages.attente));
        let mut e = self.verrou();
        self.menage(&mut e);
        let o = match corps.get("onglet") {
            None | Some(Value::Null) => {
                let candidats: Vec<&Onglet> = e.onglets.iter().filter(|o| pages.contains(&o.page)).collect();
                if candidats.is_empty() {
                    return refus(404, format!("Aucun onglet {} ouvert.", ou));
                }
                let mut appareils: Vec<&str> = candidats.iter().map(|o| o.appareil.as_str()).collect();
                appareils.sort();
                appareils.dedup();
                if appareils.len() > 1 {
                    // Sur plusieurs appareils, « le plus récent » se falsifie : Claude nomme l'onglet
                    return refus(409, format!("Onglets {} ouverts sur plusieurs appareils ({}) : précise l'onglet \
                        (voir pilote_onglets).", ou, appareils.join(", ")));
                }
                candidats.into_iter().reduce(|a, b| if b.focus > a.focus { b } else { a }).unwrap()
            }
            Some(Value::String(voulu)) => {
                let Some(o) = e.onglet(voulu) else {
                    return refus(404, format!("Onglet {} inconnu ou fermé : voir pilote_onglets.", voulu));
                };
                if !pages.contains(&o.page) {
                    return refus(404, format!("L'onglet {} est une page {}, pas {}.", voulu, o.page, ou));
                }
                o
            }
            _ => return refus(400, "Champ « onglet » invalide."),
        };
        let c = Commande {
            id: jeton_hex(),
            onglet: o.id.clone(),
            navigateur: o.navigateur.clone(),
            nom,
            args,
            ecriture: corps.get("ecriture") == Some(&Value::Bool(true)),
            etat: "a_livrer",
            resultat: Value::Null,
            cree: mono(),
            livree: 0.0,
            fin: 0.0,
            cible: o.clone(),
        };
        let i = c.id.clone();
        e.commandes.push(c);
        self.cond.notify_all();
        let fin = mono() + attente;
        while e.commande(&i).is_some_and(|c| !fini(c.etat)) && mono() < fin {
            e = self.dormir(e, fin - mono());
        }
        // shortcut: une commande oubliée par le ménage pendant l'attente rend 404 (Python garde sa
        // référence) ; il faudrait plus de 500 commandes en cours, à revoir si MAX_COMMANDES baisse
        let Some(p) = e.commandes.iter().position(|c| c.id == i) else {
            return refus(404, "Commande inconnue (ou finie depuis plus d'une heure).");
        };
        let vue = e.vue(&e.commandes[p]);
        let c = &mut e.commandes[p];
        if c.nom == "capture" && fini(c.etat) {
            c.resultat = Value::Null; // l'image est rendue : ne pas garder des Mo une heure
        }
        Ok(vue)
    }

    fn onglets(&self) -> Rendu {
        let mut e = self.verrou();
        self.menage(&mut e);
        let mut liste: Vec<&Onglet> = e.onglets.iter().collect();
        liste.sort_by(|a, b| b.focus.total_cmp(&a.focus));
        Ok(json!({"onglets": liste.into_iter().map(vue_onglet).collect::<Vec<_>>()}))
    }

    fn suivre(&self, r: &Requete) -> Rendu {
        let mut e = self.verrou();
        self.menage(&mut e);
        let i = r.param("id").unwrap_or_default();
        match e.commandes.iter().find(|c| c.id == i) {
            Some(c) => Ok(e.vue(c)),
            None => refus(404, "Commande inconnue (ou finie depuis plus d'une heure)."),
        }
    }

    /// Le lanceur prend la plus ancienne demande en attente (long-poll). Il y joint, juste après
    /// un test, l'état de Claude Code (?claude=) ; il rend {"tester": true} quand une page en attend un.
    fn demandes(&self, r: &Requete) -> Rendu {
        let mut etat = None;
        if let Some(brut) = r.param("claude") {
            let lu: Value = serde_json::from_str(&brut).unwrap_or(Value::Null);
            let Some(pret) = lu.as_object().and_then(|o| o.get("pret")).and_then(Value::as_bool) else {
                return refus(400, "Champ « claude » invalide.");
            };
            let message = texte(lu.get("message"), "message", 500, false)?;
            let version = texte(lu.get("version").or(Some(&json!(""))), "version", 40, true)?;
            etat = Some(json!({"pret": pret, "message": message, "version": version}));
        }
        let fin = mono() + self.reglages.attente_poll;
        let mut e = self.verrou();
        e.lanceur_vu = mono();
        if let Some(etat) = etat {
            (e.claude, e.claude_t) = (etat, mono());
            self.cond.notify_all();
        }
        loop {
            self.menage(&mut e);
            let pret = e.demandes.iter().any(|d| d.etat == "en_attente");
            if pret || e.tester || mono() >= fin {
                break;
            }
            e = self.dormir(e, fin - mono());
        }
        let attente = e.demandes.iter_mut().filter(|d| d.etat == "en_attente");
        let Some(d) = attente.reduce(|a, b| if b.cree < a.cree { b } else { a }) else {
            // Rendu une seule fois : un lanceur d'avant le test reviendrait sans fin
            let tester = std::mem::take(&mut e.tester);
            return Ok(if tester { json!({"demande": null, "tester": true}) } else { json!({"demande": null}) });
        };
        let pieces = std::mem::take(&mut d.pieces); // le lanceur les a : oubliées ici
        (d.etat, d.vu, d.poids) = ("prise", mono(), 0);
        let rendu = json!({"demande": {"id": d.id, "navigateur": d.navigateur, "onglet": d.onglet,
                                       "texte": d.texte, "pieces": pieces, "modele": d.modele, "nouvelle": d.nouvelle,
                                       "page": d.page, "projet": d.projet, "appareil": d.appareil}});
        self.cond.notify_all();
        Ok(rendu)
    }

    /// Le panneau s'ouvre : le lanceur écoute-t-il, et Claude Code est-il prêt ? Un état trop
    /// vieux fait retester (sans requête au modèle) ; on attend le nouveau quelques secondes.
    fn statut(&self) -> Rendu {
        let mut e = self.verrou();
        let fin = mono() + e.attente_test;
        let lanceur = mono() - e.lanceur_vu < LANCEUR_VIVANT;
        if lanceur && mono() - e.claude_t > TEST_PERIME {
            let avant = e.claude_t;
            e.tester = true;
            self.cond.notify_all();
            while e.claude_t == avant && mono() < fin {
                e = self.dormir(e, fin - mono());
            }
        }
        Ok(json!({"lanceur": lanceur, "claude": e.claude}))
    }

    fn progres(&self, corps: &Map<String, Value>) -> Rendu {
        let etat = match corps.get("etat") {
            None | Some(Value::Null) => None,
            v => match parmi(&["finie", "echouee"], v) {
                Some(e) => Some(e),
                None => return refus(400, "Champ « etat » invalide."),
            },
        };
        let message = match corps.get("message") {
            None | Some(Value::Null) => None,
            Some(m) => {
                let Some(genre) = parmi(GENRES, m.as_object().and_then(|m| m.get("genre"))) else {
                    return refus(400, "Champ « message » invalide.");
                };
                Some(json!({"genre": genre, "texte": texte(m.get("texte"), "texte", MAX_MESSAGE, false)?}))
            }
        };
        let mut e = self.verrou();
        let i = corps.get("id").and_then(Value::as_str).unwrap_or("");
        let Some(d) = e.demande(i).filter(|d| d.etat == "prise") else {
            return refus(404, "Demande inconnue ou déjà finie.");
        };
        d.vu = mono();
        if let Some(m) = message.filter(|_| d.messages.len() < MAX_MESSAGES) {
            d.messages.push(m);
        }
        if let Some(etat) = etat {
            (d.etat, d.fin) = (etat, mono());
        }
        self.cond.notify_all();
        Ok(json!({"ok": true}))
    }
}

#[cfg(test)]
mod essais;
