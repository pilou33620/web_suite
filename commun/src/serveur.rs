//! Petit serveur HTTP : un fil par connexion, une requête par connexion (comme
//! http.server de Python en HTTP/1.0, que ce serveur remplace). Le MÊME PORT sert
//! HTTPS et HTTP : un premier octet 0x16 est une poignée de main TLS.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use rustls::{ServerConfig, ServerConnection, StreamOwned};

const ENTETES_MAX: usize = 64 * 1024;
const MAX_CONNEXIONS: usize = 256;
/// Délai de lecture et d'écriture (s) sans `regler_delai` : un client muet ne garde pas son fil.
const DELAI_DEFAUT: u64 = 60;

/// Ce poste (127.x, ::1, ::ffff:127.x) n'entre pas dans le plafond de connexions.
fn hors_plafond(ip: IpAddr) -> bool {
    ip.to_canonical().is_loopback()
}

pub struct Requete<'a> {
    pub methode: String,
    /// La cible telle que reçue : chemin et, s'il y en a, « ?requête ».
    pub cible: String,
    pub entetes: Vec<(String, String)>,
    pub client: SocketAddr,
    pub local: SocketAddr,
    /// Cette connexion est chiffrée.
    pub https: bool,
    /// Le serveur sert aussi HTTPS sur ce port.
    pub tls_actif: bool,
    corps: &'a mut dyn Read,
    lu: bool,
    /// Garde::filtrer a laissé passer cette requête (condition d'un WebSocket).
    pub(crate) filtree: bool,
    bascule: Option<Box<dyn FnOnce(&mut dyn Duplex)>>,
}

/// La connexion rendue à l'outil après le 101 d'un WebSocket : elle se lit et s'écrit.
pub trait Duplex: Read + Write {
    /// Délai de chaque lecture (None : aucun). Une lecture qui le dépasse rend une erreur
    /// WouldBlock ou TimedOut (selon le système), et la connexion reste utilisable : en TLS, lire
    /// et écrire se font dans le même fil, en lisant avec un délai court.
    fn delai_lecture(&mut self, delai: Option<Duration>) -> io::Result<()>;
}

/// Clé fixe de la poignée de main WebSocket (RFC 6455).
const GUID_WEBSOCKET: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug)]
pub enum ErreurCorps {
    /// Content-Length n'est pas un entier positif.
    Invalide,
    TropGrand,
    Lecture(io::Error),
}

impl Requete<'_> {
    pub fn entete(&self, nom: &str) -> Option<&str> {
        self.entetes.iter().find(|(k, _)| k.eq_ignore_ascii_case(nom)).map(|(_, v)| v.as_str())
    }

    pub fn chemin(&self) -> &str {
        let sans_fragment = self.cible.split('#').next().unwrap_or("");
        sans_fragment.split('?').next().unwrap_or("")
    }

    pub fn requete(&self) -> &str {
        let sans_fragment = self.cible.split('#').next().unwrap_or("");
        sans_fragment.split_once('?').map(|(_, q)| q).unwrap_or("")
    }

    /// Première valeur non vide du paramètre (parse_qs de Python).
    pub fn param(&self, nom: &str) -> Option<String> {
        params(self.requete()).into_iter().find(|(k, v)| k == nom && !v.is_empty()).map(|(_, v)| v)
    }

    pub fn lecture(&self) -> bool {
        self.methode == "GET" || self.methode == "HEAD"
    }

    /// Le corps brut, à lire soi-même par blocs (envoi d'un gros fichier) : s'en tenir
    /// à Content-Length, rien ne borne la lecture ici.
    pub fn flux_corps(&mut self) -> &mut dyn Read {
        self.lu = true;
        self.corps
    }

    /// Le corps, s'il tient dans `max` octets. Un corps refusé n'est pas lu.
    pub fn lire_corps(&mut self, max: usize) -> Result<Vec<u8>, ErreurCorps> {
        let n: usize = match self.entete("Content-Length") {
            None | Some("") => 0,
            Some(v) => v.trim().parse().map_err(|_| ErreurCorps::Invalide)?,
        };
        if n > max {
            return Err(ErreurCorps::TropGrand);
        }
        if self.lu {
            return Ok(Vec::new());
        }
        self.lu = true;
        // le tampon grandit avec ce qui arrive : un Content-Length annoncé ne réserve rien
        let mut corps = Vec::new();
        (&mut self.corps).take(n as u64).read_to_end(&mut corps).map_err(ErreurCorps::Lecture)?;
        if corps.len() < n {
            return Err(ErreurCorps::Lecture(io::ErrorKind::UnexpectedEof.into()));
        }
        Ok(corps)
    }

    /// WebSocket (RFC 6455) : rend le 101 de la poignée de main et garde `suite`, que le serveur
    /// appelle avec la connexion une fois le 101 envoyé, dans le fil de cette connexion ; elle se
    /// ferme au retour de `suite`. Refus : 403 si Garde::filtrer n'a pas laissé passer la requête,
    /// ou si Origin est un autre site ; 400 si ce n'est pas une demande de WebSocket.
    /// `protocoles` : les sous-protocoles acceptés (le premier que le client propose est rendu).
    pub fn websocket(&mut self, protocoles: &[&str], suite: impl FnOnce(&mut dyn Duplex) + 'static) -> Reponse {
        if !self.filtree {
            return Reponse::texte(403, "Requête refusée");
        }
        if let Some(origine) = self.entete("Origin") {
            // un WebSocket ignore la règle de même origine : une page d'un autre site ne s'y connecte pas
            let hote = origine.split_once("://").map(|(_, h)| h).unwrap_or("").split('/').next().unwrap_or("");
            if hote.is_empty() || !hote.eq_ignore_ascii_case(self.entete("Host").unwrap_or("")) {
                return Reponse::texte(403, "Requête refusée (autre site)");
            }
        }
        let upgrade = self.entete("Upgrade").is_some_and(|u| u.eq_ignore_ascii_case("websocket"));
        let Some(cle) = self.entete("Sec-WebSocket-Key").filter(|c| upgrade && !c.is_empty()) else {
            return Reponse::texte(400, "WebSocket attendu");
        };
        use base64::Engine;
        use sha1::{Digest, Sha1};
        let accept = base64::engine::general_purpose::STANDARD.encode(Sha1::digest(format!("{}{}", cle, GUID_WEBSOCKET)));
        // Connection exactement « Upgrade » : les anciens Safari n'acceptent rien d'autre
        let mut rep = Reponse { code: 101, entetes: Vec::new(), corps: Vec::new(), suite: None }
            .entete("Upgrade", "websocket")
            .entete("Connection", "Upgrade")
            .entete("Sec-WebSocket-Accept", &accept);
        let offerts = self.entete("Sec-WebSocket-Protocol").unwrap_or("");
        if let Some(p) = offerts.split(',').map(str::trim).find(|p| protocoles.contains(p)) {
            rep = rep.entete("Sec-WebSocket-Protocol", p);
        }
        self.bascule = Some(Box::new(suite));
        rep
    }
}

pub struct Reponse {
    pub code: u16,
    pub entetes: Vec<(String, String)>,
    pub corps: Vec<u8>,
    /// Envoyé après `corps`, par blocs : un modèle de 200 Mo ne passe pas en mémoire.
    /// Content-Length est alors à donner dans `entetes`.
    pub suite: Option<Box<dyn Read + Send>>,
}

impl Reponse {
    pub fn new(code: u16, type_: &str, corps: impl Into<Vec<u8>>) -> Self {
        Reponse { code, entetes: vec![("Content-Type".into(), type_.into())], corps: corps.into(), suite: None }
            .entete("Cache-Control", "no-store")
    }

    pub fn texte(code: u16, corps: &str) -> Self {
        Self::new(code, "text/plain; charset=utf-8", corps)
    }

    pub fn json(code: u16, donnees: &serde_json::Value) -> Self {
        Self::new(code, "application/json; charset=utf-8", donnees.to_string())
    }

    pub fn redirection(code: u16, ou: &str) -> Self {
        Reponse { code, entetes: Vec::new(), corps: Vec::new(), suite: None }
            .entete("Location", ou)
            .entete("Cache-Control", "no-store")
    }

    /// Un fichier (ou tout flux) de `taille` octets, envoyé par blocs.
    pub fn flux(code: u16, type_: &str, taille: u64, suite: impl Read + Send + 'static) -> Self {
        let mut r = Self::new(code, type_, Vec::new()).entete("Content-Length", &taille.to_string());
        r.suite = Some(Box::new(suite));
        r
    }

    pub fn entete(mut self, nom: &str, valeur: &str) -> Self {
        self.entetes.push((nom.into(), valeur.into()));
        self
    }
}

fn raison(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        206 => "Partial Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        411 => "Length Required",
        413 => "Request Entity Too Large",
        415 => "Unsupported Media Type",
        416 => "Requested Range Not Satisfiable",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "",
    }
}

pub fn params(requete: &str) -> Vec<(String, String)> {
    requete
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (decoder_url(k), decoder_url(v))
        })
        .collect()
}

/// unquote_plus : %XX et « + ». Pour les paramètres seulement : un chemin passe par decoder_chemin.
pub fn decoder_url(s: &str) -> String {
    decoder(s, true)
}

/// unquote : %XX seulement. « + » reste « + » dans un chemin (/js/a+b.js).
pub fn decoder_chemin(s: &str) -> String {
    decoder(s, false)
}

fn decoder(s: &str, plus_en_espace: bool) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        match b[i] {
            b'+' if plus_en_espace => out.push(b' '),
            b'%' if i + 2 < b.len() && hex(b[i + 1]).is_some() && hex(b[i + 2]).is_some() => {
                out.push((hex(b[i + 1]).unwrap() * 16 + hex(b[i + 2]).unwrap()) as u8);
                i += 2;
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// quote_plus.
pub fn encoder_url(s: &str) -> String {
    let mut out = String::new();
    for c in s.bytes() {
        match c {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b'-' | b'~' => out.push(c as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", c)),
        }
    }
    out
}

enum Flux {
    Clair(TcpStream),
    Tls(Box<StreamOwned<ServerConnection, TcpStream>>),
}

impl Read for Flux {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Flux::Clair(s) => s.read(buf),
            Flux::Tls(s) => s.read(buf),
        }
    }
}

impl Write for Flux {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Flux::Clair(s) => s.write(buf),
            Flux::Tls(s) => s.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Flux::Clair(s) => s.flush(),
            Flux::Tls(s) => s.flush(),
        }
    }
}

/// La connexion d'un WebSocket : ce qui reste à lire du tampon, puis le flux (clair ou TLS).
struct Connexion<'a> {
    lecteur: &'a mut BufReader<Flux>,
    tcp: &'a TcpStream,
}

impl Read for Connexion<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lecteur.read(buf)
    }
}

impl Write for Connexion<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lecteur.get_mut().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.lecteur.get_mut().flush()
    }
}

impl Duplex for Connexion<'_> {
    fn delai_lecture(&mut self, delai: Option<Duration>) -> io::Result<()> {
        self.tcp.set_read_timeout(delai)
    }
}

pub type Service = dyn Fn(&mut Requete) -> Reponse + Send + Sync;

pub struct Serveur {
    ecoute: TcpListener,
    tls: RwLock<Option<Arc<ServerConfig>>>,
    arret: AtomicBool,
    delai: std::sync::atomic::AtomicU64,
}

fn ecouter(adresse: SocketAddr, double_pile: bool) -> io::Result<TcpListener> {
    use socket2::{Domain, Socket, Type};
    let s = Socket::new(Domain::for_address(adresse), Type::STREAM, None)?;
    if double_pile {
        s.set_only_v6(false)?;
    }
    #[cfg(not(windows))]
    s.set_reuse_address(true)?;
    s.bind(&adresse.into())?;
    s.listen(128)?;
    Ok(s.into())
}

/// Sous Windows, une socket en double pile se lie au port même si un autre programme
/// tient déjà son IPv4, et les clients IPv4 iraient chez lui : on le demande avant.
fn ipv4_libre(port: u16) -> bool {
    !cfg!(windows) || TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).is_ok()
}

impl Serveur {
    /// `reseau` : toutes les interfaces, en double pile (IPv4 et IPv6 sur la même
    /// socket : Tailscale fd7a::, ULA) si le système le permet ; sinon 127.0.0.1.
    /// Essaie `essais` ports à partir de `port` (0 : un port libre quelconque).
    pub fn ouvrir(port: u16, reseau: bool, essais: u16) -> io::Result<Arc<Serveur>> {
        for p in port..=port.saturating_add(essais.saturating_sub(1)) {
            if reseau && p != 0 && !ipv4_libre(p) {
                continue;
            }
            let piles: &[(IpAddr, bool)] = if reseau {
                &[(IpAddr::V6(Ipv6Addr::UNSPECIFIED), true), (IpAddr::V4(Ipv4Addr::UNSPECIFIED), false)]
            } else {
                &[(IpAddr::V4(Ipv4Addr::LOCALHOST), false)]
            };
            for &(ip, double) in piles {
                if let Ok(ecoute) = ecouter(SocketAddr::new(ip, p), double) {
                    return Ok(Serveur::neuf(ecoute));
                }
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            format!("aucun port libre entre {} et {}", port, port.saturating_add(essais.saturating_sub(1))),
        ))
    }

    /// Écoute sur la seule adresse `ip` (option --host) : 127.0.0.1, ::1, une adresse du poste
    /// (192.168.1.20, fd7a:…), ou 0.0.0.0 / :: pour toutes les interfaces (:: en double pile).
    /// Mêmes `essais` ports à partir de `port` et même tri HTTP/HTTPS sur le port qu'`ouvrir`.
    /// Seul un port pris fait passer au suivant : une adresse absente du poste est refusée
    /// d'emblée (AddrNotAvailable), comme une IPv6 de lien local (fe80::, il y faudrait l'interface).
    /// Une adresse qui n'est pas une boucle locale est le mode réseau : la Garde doit le savoir,
    /// `Garde::new(nom, !ip.to_canonical().is_loopback(), …)`, sinon le réseau entre sans jeton.
    pub fn ouvrir_sur(ip: IpAddr, port: u16, essais: u16) -> io::Result<Arc<Serveur>> {
        let ip = ip.to_canonical(); // ::ffff:192.168.1.20 est 192.168.1.20
        let double = ip == IpAddr::V6(Ipv6Addr::UNSPECIFIED);
        let dernier = port.saturating_add(essais.saturating_sub(1));
        let mut cause = String::from("port déjà pris");
        for p in port..=dernier {
            if double && p != 0 && !ipv4_libre(p) {
                continue;
            }
            match ecouter(SocketAddr::new(ip, p), double) {
                Ok(ecoute) => return Ok(Serveur::neuf(ecoute)),
                // Windows rend PermissionDenied pour un port réservé ; Linux, pour un port < 1024
                Err(e) if matches!(e.kind(), io::ErrorKind::AddrInUse | io::ErrorKind::PermissionDenied) => cause = e.to_string(),
                Err(e) => return Err(io::Error::new(e.kind(), format!("écoute impossible sur {} : {}", ip, e))),
            }
        }
        Err(io::Error::new(io::ErrorKind::AddrInUse, format!("aucun port libre sur {} entre {} et {} ({})", ip, port, dernier, cause)))
    }

    fn neuf(ecoute: TcpListener) -> Arc<Serveur> {
        Arc::new(Serveur { ecoute, tls: RwLock::new(None), arret: AtomicBool::new(false), delai: DELAI_DEFAUT.into() })
    }

    /// Délai de chaque lecture du socket (0 : aucun) : un envoi coupé sans fermeture (Wi-Fi
    /// perdu) ne tient pas un fil indéfiniment. Un envoi lent mais vivant n'est pas coupé.
    pub fn regler_delai(&self, secondes: u64) {
        self.delai.store(secondes, Ordering::SeqCst);
    }

    pub fn port(&self) -> u16 {
        self.ecoute.local_addr().map(|a| a.port()).unwrap_or(0)
    }

    /// Le port sert aussi HTTPS à partir de maintenant.
    pub fn brancher_tls(&self, cfg: Arc<ServerConfig>) {
        *self.tls.write().unwrap() = Some(cfg);
    }

    pub fn tls_actif(&self) -> bool {
        self.tls.read().unwrap().is_some()
    }

    /// Sert jusqu'à `arreter()`.
    pub fn servir(self: &Arc<Self>, service: Arc<Service>) {
        let places = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        for flux in self.ecoute.incoming() {
            if self.arret.load(Ordering::SeqCst) {
                break;
            }
            let Ok(flux) = flux else { continue };
            // Un autre appareil ne tient pas plus de MAX_CONNEXIONS fils à la fois (connexions
            // muettes) : la suivante est fermée d'emblée. Ce poste n'est jamais compté.
            let comptee = !flux.peer_addr().is_ok_and(|a| hors_plafond(a.ip()));
            if comptee && places.fetch_add(1, Ordering::SeqCst) >= MAX_CONNEXIONS {
                places.fetch_sub(1, Ordering::SeqCst);
                continue;
            }
            let (serveur, service, places) = (self.clone(), service.clone(), places.clone());
            std::thread::spawn(move || {
                // Le navigateur a fermé la connexion avant la réponse : rien à dire.
                let _ = serveur.connexion(flux, &*service);
                if comptee {
                    places.fetch_sub(1, Ordering::SeqCst);
                }
            });
        }
    }

    pub fn servir_en_fond(self: &Arc<Self>, service: Arc<Service>) {
        let s = self.clone();
        std::thread::spawn(move || s.servir(service));
    }

    pub fn arreter(&self) {
        self.arret.store(true, Ordering::SeqCst);
        // réveille accept() : par l'adresse d'écoute si elle est précise (ouvrir_sur), sinon 127.0.0.1
        let ip = match self.ecoute.local_addr() {
            Ok(a) if !a.ip().is_unspecified() => a.ip(),
            _ => IpAddr::V4(Ipv4Addr::LOCALHOST),
        };
        let _ = TcpStream::connect((ip, self.port()));
    }

    fn connexion(&self, tcp: TcpStream, service: &Service) -> io::Result<()> {
        let client = tcp.peer_addr()?;
        let local = tcp.local_addr()?;
        let tls = self.tls.read().unwrap().clone();
        let mut flux = Flux::Clair(tcp.try_clone()?);
        let mut https = false;
        if let Some(cfg) = &tls {
            tcp.set_read_timeout(Some(Duration::from_secs(15)))?; // un client muet ne garde pas le fil
            let mut premier = [0u8; 1];
            if tcp.peek(&mut premier)? == 1 && premier[0] == 0x16 {
                let conn = ServerConnection::new(cfg.clone()).map_err(io::Error::other)?;
                flux = Flux::Tls(Box::new(StreamOwned::new(conn, tcp.try_clone()?)));
                https = true;
            }
            tcp.set_read_timeout(None)?;
        }
        let delai = self.delai.load(Ordering::SeqCst);
        if delai > 0 {
            tcp.set_read_timeout(Some(Duration::from_secs(delai)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(delai)))?;
        }
        let mut lecteur = BufReader::new(flux);
        let mut total = 0;
        let mut ligne = |lecteur: &mut BufReader<Flux>| -> io::Result<String> {
            let mut l = Vec::new();
            lecteur.take((ENTETES_MAX - total) as u64 + 1).read_until(b'\n', &mut l)?;
            total += l.len();
            if total > ENTETES_MAX || !l.ends_with(b"\n") {
                return Err(io::ErrorKind::InvalidData.into());
            }
            Ok(String::from_utf8_lossy(&l).trim_end_matches(['\r', '\n']).to_string())
        };
        let premiere = ligne(&mut lecteur)?;
        let mut mots = premiere.split(' ');
        let (Some(methode), Some(cible)) = (mots.next(), mots.next()) else {
            return Err(io::ErrorKind::InvalidData.into());
        };
        let mut entetes = Vec::new();
        loop {
            let l = ligne(&mut lecteur)?;
            if l.is_empty() {
                break;
            }
            if let Some((k, v)) = l.split_once(':') {
                entetes.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
        let tete = methode == "HEAD";
        let mut requete = Requete {
            methode: methode.to_string(),
            cible: cible.to_string(),
            entetes,
            client,
            local,
            https,
            tls_actif: tls.is_some(),
            corps: &mut lecteur,
            lu: false,
            filtree: false,
            bascule: None,
        };
        let reponse = service(&mut requete);
        let bascule = if reponse.code == 101 { requete.bascule.take() } else { None };
        drop(requete);
        if let Some(suite) = bascule {
            // WebSocket : 101 en HTTP/1.1 sans Content-Length ni « Connection: close », puis l'outil
            let mut tete = String::from("HTTP/1.1 101 Switching Protocols\r\n");
            for (k, v) in &reponse.entetes {
                tete += &format!("{}: {}\r\n", k, v);
            }
            tete += "\r\n";
            lecteur.get_mut().write_all(tete.as_bytes())?;
            lecteur.get_mut().flush()?;
            let mut connexion = Connexion { lecteur: &mut lecteur, tcp: &tcp };
            // une panique de l'outil ferme cette connexion, sans tuer le fil qui la compte
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| suite(&mut connexion)));
            return Serveur::fermer(lecteur.get_mut(), &tcp);
        }
        let mut sortie = Vec::with_capacity(reponse.corps.len() + 256);
        write!(sortie, "HTTP/1.0 {} {}\r\n", reponse.code, raison(reponse.code))?;
        for (k, v) in &reponse.entetes {
            write!(sortie, "{}: {}\r\n", k, v)?;
        }
        if !reponse.entetes.iter().any(|(k, _)| k.eq_ignore_ascii_case("Content-Length")) {
            write!(sortie, "Content-Length: {}\r\n", reponse.corps.len())?;
        }
        // pas de cadre chez une autre origine : une page piégée ne détourne pas les clics
        sortie.extend_from_slice(b"X-Frame-Options: SAMEORIGIN\r\nConnection: close\r\n\r\n");
        if !tete {
            sortie.extend_from_slice(&reponse.corps);
        }
        let flux = lecteur.get_mut();
        flux.write_all(&sortie)?;
        if let (Some(mut suite), false) = (reponse.suite, tete) {
            std::io::copy(&mut suite, flux)?;
        }
        Serveur::fermer(flux, &tcp)
    }

    fn fermer(flux: &mut Flux, tcp: &TcpStream) -> io::Result<()> {
        flux.flush()?;
        if let Flux::Tls(s) = flux {
            s.conn.send_close_notify();
            let _ = s.flush();
        }
        let _ = tcp.shutdown(std::net::Shutdown::Write);
        Ok(())
    }
}

#[cfg(test)]
mod essais_plafond {
    use super::*;

    #[test]
    fn seul_ce_poste_est_hors_plafond() {
        for a in ["127.0.0.1", "::1", "::ffff:127.0.0.1"] {
            assert!(hors_plafond(a.parse().unwrap()), "{a}");
        }
        for a in ["192.168.1.20", "fd7a:115c:a1e0::1"] {
            assert!(!hors_plafond(a.parse().unwrap()), "{a}");
        }
    }

    #[test]
    fn plus_garde_dans_le_chemin_espace_dans_les_parametres() {
        assert_eq!(decoder_chemin("/js/a+b.js"), "/js/a+b.js");
        assert_eq!(decoder_chemin("/js/a%2Bb.js"), "/js/a+b.js");
        assert_eq!(decoder_chemin("/js/a%20b.js"), "/js/a b.js");
        assert_eq!(params("nom=a+b&x=c%2Bd"), vec![("nom".to_string(), "a b".to_string()), ("x".to_string(), "c+d".to_string())]);
        assert_eq!(decoder_url("a+b"), "a b");
    }
}

#[cfg(test)]
mod essais_ecoute {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn libelles_de_statut() {
        assert_eq!(raison(501), "Not Implemented");
        assert_eq!(raison(422), "Unprocessable Entity");
        for c in [200, 201, 204, 206, 301, 302, 303, 304, 400, 401, 403, 404, 405, 409, 411, 413, 415, 416, 422, 429, 500, 501, 502, 503, 504] {
            assert!(!raison(c).is_empty(), "{c}");
        }
    }

    #[test]
    fn ouvrir_sur_une_adresse_precise() {
        let local = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let srv = Serveur::ouvrir_sur(local, 0, 1).unwrap();
        assert_eq!(srv.ecoute.local_addr().unwrap().ip(), local);
        srv.servir_en_fond(Arc::new(|_: &mut Requete| Reponse::texte(501, "non")));
        let mut c = TcpStream::connect((local, srv.port())).unwrap();
        c.write_all(b"GET / HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n").unwrap();
        let mut brut = String::new();
        c.read_to_string(&mut brut).unwrap();
        assert!(brut.starts_with("HTTP/1.0 501 Not Implemented\r\n"), "{brut}");
        srv.arreter();
        // 192.0.2.1 (RFC 5737, réservée à la documentation) n'est sur aucun poste
        let e = Serveur::ouvrir_sur("192.0.2.1".parse().unwrap(), 8000, 10).err().expect("refus attendu");
        assert_eq!(e.kind(), io::ErrorKind::AddrNotAvailable, "{e}");
    }
}
