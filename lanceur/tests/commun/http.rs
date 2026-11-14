//! Client HTTP des bancs : une requête, une réponse, sur la boucle locale.
//! Sans dépendance (web_bench/tests le reprend par #[path]).
#![allow(dead_code)]
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct Rep {
    pub code: u16,
    pub entetes: Vec<(String, String)>,
    pub corps: Vec<u8>,
}

impl Rep {
    pub fn entete(&self, nom: &str) -> Option<&str> {
        self.entetes.iter().find(|(k, _)| k.eq_ignore_ascii_case(nom)).map(|(_, v)| v.as_str())
    }
    pub fn texte(&self) -> String {
        String::from_utf8_lossy(&self.corps).into_owned()
    }
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.corps).unwrap_or_else(|e| panic!("JSON attendu ({e}) : {}", self.texte()))
    }
}

/// Écrit la requête sur `flux` (TCP ou TLS) et lit la réponse. Host et Content-Length sont
/// ajoutés s'ils ne sont pas dans `entetes` (Content-Length : seulement avec un corps ou en POST).
pub fn echanger<F: Read + Write>(
    flux: &mut F, port: u16, methode: &str, chemin: &str, entetes: &[(&str, &str)], corps: &[u8],
) -> io::Result<Rep> {
    let a = |nom: &str| entetes.iter().any(|(k, _)| k.eq_ignore_ascii_case(nom));
    let mut d = format!("{methode} {chemin} HTTP/1.1\r\n");
    if !a("Host") {
        d += &format!("Host: 127.0.0.1:{port}\r\n");
    }
    for (k, v) in entetes {
        d += &format!("{k}: {v}\r\n");
    }
    if !a("Content-Length") && (!corps.is_empty() || methode == "POST") {
        d += &format!("Content-Length: {}\r\n", corps.len());
    }
    // l'envoi peut être coupé par un serveur qui a déjà refusé (corps annoncé géant) : on lit quand même
    let _ = flux.write_all(d.as_bytes()).and_then(|_| flux.write_all(b"\r\n")).and_then(|_| flux.write_all(corps));
    let _ = flux.flush();

    let (mut brut, mut bloc) = (Vec::new(), [0u8; 65536]);
    let coupe = loop {
        if let Some(i) = brut.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        match flux.read(&mut bloc)? {
            0 => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "réponse HTTP sans en-têtes")),
            n => brut.extend_from_slice(&bloc[..n]),
        }
    };
    let tete = String::from_utf8_lossy(&brut[..coupe]).into_owned();
    let mut lignes = tete.split("\r\n");
    let code = lignes.next().and_then(|l| l.split(' ').nth(1)).and_then(|c| c.parse().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "ligne d'état HTTP illisible"))?;
    let entetes: Vec<(String, String)> =
        lignes.filter_map(|l| l.split_once(':')).map(|(k, v)| (k.to_string(), v.trim().to_string())).collect();
    let mut rep = Rep { code, entetes, corps: brut[coupe + 4..].to_vec() };
    let attendu = if methode == "HEAD" || code == 204 || code == 304 {
        Some(0)
    } else {
        rep.entete("Content-Length").and_then(|n| n.parse::<usize>().ok())
    };
    // une fin de flux (ou une fermeture TLS sans adieu) termine le corps
    while attendu.is_none_or(|n| rep.corps.len() < n) {
        match flux.read(&mut bloc) {
            Ok(0) | Err(_) => break,
            Ok(n) => rep.corps.extend_from_slice(&bloc[..n]),
        }
    }
    if let Some(n) = attendu {
        rep.corps.truncate(n);
    }
    Ok(rep)
}

pub fn connecter(port: u16) -> io::Result<TcpStream> {
    let c = TcpStream::connect(("127.0.0.1", port))?;
    c.set_read_timeout(Some(Duration::from_secs(20)))?;
    Ok(c)
}

pub fn requete(port: u16, methode: &str, chemin: &str, entetes: &[(&str, &str)], corps: &[u8]) -> io::Result<Rep> {
    echanger(&mut connecter(port)?, port, methode, chemin, entetes, corps)
}
