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
}

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
        let mut corps = vec![0; n];
        self.corps.read_exact(&mut corps).map_err(ErreurCorps::Lecture)?;
        Ok(corps)
    }
}

pub struct Reponse {
    pub code: u16,
    pub entetes: Vec<(String, String)>,
    pub corps: Vec<u8>,
}

impl Reponse {
    pub fn new(code: u16, type_: &str, corps: impl Into<Vec<u8>>) -> Self {
        Reponse { code, entetes: vec![("Content-Type".into(), type_.into())], corps: corps.into() }
            .entete("Cache-Control", "no-store")
    }

    pub fn texte(code: u16, corps: &str) -> Self {
        Self::new(code, "text/plain; charset=utf-8", corps)
    }

    pub fn json(code: u16, donnees: &serde_json::Value) -> Self {
        Self::new(code, "application/json; charset=utf-8", donnees.to_string())
    }

    pub fn redirection(code: u16, ou: &str) -> Self {
        Reponse { code, entetes: Vec::new(), corps: Vec::new() }
            .entete("Location", ou)
            .entete("Cache-Control", "no-store")
    }

    pub fn entete(mut self, nom: &str, valeur: &str) -> Self {
        self.entetes.push((nom.into(), valeur.into()));
        self
    }
}

fn raison(code: u16) -> &'static str {
    match code {
        200 => "OK",
        204 => "No Content",
        206 => "Partial Content",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Request Entity Too Large",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
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

/// unquote_plus : %XX et « + ».
pub fn decoder_url(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        match b[i] {
            b'+' => out.push(b' '),
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

pub type Service = dyn Fn(&mut Requete) -> Reponse + Send + Sync;

pub struct Serveur {
    ecoute: TcpListener,
    tls: RwLock<Option<Arc<ServerConfig>>>,
    arret: AtomicBool,
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
                    return Ok(Arc::new(Serveur { ecoute, tls: RwLock::new(None), arret: AtomicBool::new(false) }));
                }
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            format!("aucun port libre entre {} et {}", port, port.saturating_add(essais.saturating_sub(1))),
        ))
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
        for flux in self.ecoute.incoming() {
            if self.arret.load(Ordering::SeqCst) {
                break;
            }
            let Ok(flux) = flux else { continue };
            let (serveur, service) = (self.clone(), service.clone());
            std::thread::spawn(move || {
                // Le navigateur a fermé la connexion avant la réponse : rien à dire.
                let _ = serveur.connexion(flux, &*service);
            });
        }
    }

    pub fn servir_en_fond(self: &Arc<Self>, service: Arc<Service>) {
        let s = self.clone();
        std::thread::spawn(move || s.servir(service));
    }

    pub fn arreter(&self) {
        self.arret.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect((Ipv4Addr::LOCALHOST, self.port())); // réveille accept()
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
        };
        let reponse = service(&mut requete);
        let mut sortie = Vec::with_capacity(reponse.corps.len() + 256);
        write!(sortie, "HTTP/1.0 {} {}\r\n", reponse.code, raison(reponse.code))?;
        for (k, v) in &reponse.entetes {
            write!(sortie, "{}: {}\r\n", k, v)?;
        }
        if !reponse.entetes.iter().any(|(k, _)| k.eq_ignore_ascii_case("Content-Length")) {
            write!(sortie, "Content-Length: {}\r\n", reponse.corps.len())?;
        }
        sortie.extend_from_slice(b"Connection: close\r\n\r\n");
        if !tete {
            sortie.extend_from_slice(&reponse.corps);
        }
        let flux = lecteur.get_mut();
        flux.write_all(&sortie)?;
        flux.flush()?;
        if let Flux::Tls(s) = flux {
            s.conn.send_close_notify();
            let _ = s.flush();
        }
        let _ = tcp.shutdown(std::net::Shutdown::Write);
        Ok(())
    }
}
