//! Appairage des web tools : un code à 6 chiffres ouvre l'accès réseau à un appareil.
//!
//! Un serveur qui écoute sur le réseau local exige des autres appareils un jeton.
//! Le serveur affiche dans son terminal un code à 6 chiffres, que l'on saisit sur
//! la tablette. Le code ne remplace pas le jeton, il le DONNE : bon code, la
//! tablette reçoit le cookie du jeton. Ce poste lui-même (boucle locale) n'a
//! jamais besoin de code.
//!
//! UN SEUL JETON, UN SEUL COOKIE POUR TOUS LES TOOLS D'UN POSTE : le jeton est dans
//! ~/.web_tools/jeton-reseau.txt, et un navigateur envoie un cookie à tous les
//! ports d'une même adresse.
//!
//! HTTPS EN MODE RÉSEAU : une autorité (CA) propre à ce poste, créée une fois dans
//! ~/.web_tools/tls/ par la commande openssl, signe le certificat du serveur. Le
//! MÊME PORT sert HTTPS et HTTP (serveur.rs). La boucle locale garde HTTP ; un
//! autre appareil arrivé en HTTP est renvoyé vers https://. Sans openssl, ou avec
//! WEB_TOOLS_HTTP=1, tout reste en HTTP et le terminal le dit.
//!
//! Usage : `garde.filtrer(&mut requete)` en tête de chaque requête ; `Some(réponse)`
//! est à renvoyer telle quelle, `None` laisse la requête suivre son cours.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine;
use rustls::ServerConfig;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::serveur::{encoder_url, params, Reponse, Requete, Serveur};

pub const BISCUIT: &str = "websuite_jeton"; // le nom posé par WEB·SUITE depuis toujours : ses appareils restent appairés
pub const PAGE: &str = "/appairage"; // la page du code (sa propre adresse : Safari ressortait « / » de son cache)
pub const ROUTE: &str = "/api/appairer"; // la seule route ouverte sans jeton, en POST
pub const ENTETE: &str = "X-Appairage"; // en-tête maison : un autre site ne peut pas l'ajouter sans pré-vol CORS
pub const MARQUE: &str = "[connexion]"; // WEB·SUITE recopie ces lignes du journal d'un outil dans son terminal
pub const CA: &str = "/appairage/ca.pem"; // le certificat de la CA du poste, à installer sur la tablette
const PAGE_HTML: &str = include_str!("appairage_page.html");

pub type Afficher = Arc<dyn Fn(&str) + Send + Sync>;

fn console() -> Afficher {
    Arc::new(|texte| {
        println!("{}", texte);
        let _ = std::io::stdout().flush();
    })
}

fn mono() -> f64 {
    static DEBUT: OnceLock<Instant> = OnceLock::new();
    DEBUT.get_or_init(Instant::now).elapsed().as_secs_f64() + 1.0
}

fn horloge() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

pub fn heure() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

fn alea(n: usize) -> Vec<u8> {
    let mut b = vec![0u8; n];
    getrandom::getrandom(&mut b).expect("générateur aléatoire du système");
    b
}

pub fn dossier_personnel() -> PathBuf {
    PathBuf::from(std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).unwrap_or_default())
}

pub fn fichier_jeton() -> PathBuf {
    match std::env::var_os("WEB_TOOLS_JETON_FICHIER") {
        Some(f) if !f.is_empty() => PathBuf::from(f),
        _ => dossier_personnel().join(".web_tools").join("jeton-reseau.txt"),
    }
}

fn lire(chemin: &Path) -> Option<String> {
    let j = fs::read_to_string(chemin).ok()?.trim().to_string();
    (j.len() >= 16).then_some(j)
}

fn creer_prive(chemin: &Path, exclusif: bool) -> std::io::Result<fs::File> {
    let mut o = fs::OpenOptions::new();
    o.write(true);
    if exclusif {
        o.create_new(true);
    } else {
        o.create(true).truncate(true);
    }
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut o, 0o600);
    o.open(chemin)
}

/// Le jeton partagé par les tools de ce poste, gardé d'un démarrage à l'autre
/// (les favoris de la tablette restent bons). Supprimer le fichier en crée un neuf.
///
/// ancien : fichier d'un jeton propre à un tool (lanceur/jeton-reseau.txt de
/// WEB·SUITE) ; repris s'il n'y a pas encore de jeton partagé.
pub fn jeton(ancien: Option<&Path>) -> String {
    let chemin = fichier_jeton();
    if let Some(j) = lire(&chemin) {
        return j;
    }
    let neuf = ancien
        .and_then(lire)
        .unwrap_or_else(|| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(alea(16)));
    if let Some(d) = chemin.parent() {
        let _ = fs::create_dir_all(d);
    }
    // create_new : deux tools qui démarrent ensemble ne s'écrasent pas leur jeton
    match creer_prive(&chemin, true) {
        Ok(mut f) => {
            let _ = writeln!(f, "{}", neuf);
            neuf
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            for _ in 0..50 {
                // l'autre tool est en train de l'écrire
                if let Some(j) = lire(&chemin) {
                    return j;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            // Toujours vide ou trop court (démarrage coupé en pleine écriture) : on le remplace.
            let temp = PathBuf::from(format!("{}.{}.tmp", chemin.display(), std::process::id()));
            let ecrit = creer_prive(&temp, false)
                .and_then(|mut f| writeln!(f, "{}", neuf))
                .and_then(|_| fs::rename(&temp, &chemin));
            if ecrit.is_err() {
                let _ = fs::remove_file(&temp);
                println!("  {} illisible et impossible à remplacer : jeton de cette session seulement.", chemin.display());
                return neuf;
            }
            println!("  {} était vide ou abîmé : jeton neuf écrit, les appareils redonneront le code.", chemin.display());
            // shortcut: deux tools qui le remplacent au même instant gardent le dernier écrit ; un verrou si ça arrive
            lire(&chemin).unwrap_or(neuf)
        }
        // Dossier personnel en lecture seule : un jeton pour cette session seulement.
        Err(_) => neuf,
    }
}

/// Une ligne de journal sans le jeton (« GET /?jeton=... » avant la redirection 303).
pub fn masquer(texte: &str) -> String {
    let mut out = String::new();
    let mut reste = texte;
    while let Some(i) = reste.find("jeton=") {
        let apres = &reste[i + 6..];
        let fin = apres.find(|c: char| c == '&' || c == '"' || c.is_whitespace()).unwrap_or(apres.len());
        if fin == 0 {
            out.push_str(&reste[..i + 6]);
        } else {
            out.push_str(&reste[..i]);
            out.push_str("jeton=***");
        }
        reste = &apres[fin..];
    }
    out + reste
}

// -- adresses -----------------------------------------------------------------

/// `ip` est-elle dans le réseau « adresse/préfixe » ?
pub fn dans(ip: IpAddr, reseau: &str) -> bool {
    let Some((base, prefixe)) = reseau.split_once('/') else { return false };
    let (Ok(base), Ok(prefixe)) = (base.parse::<IpAddr>(), prefixe.parse::<u32>()) else { return false };
    match (ip, base) {
        (IpAddr::V4(a), IpAddr::V4(b)) => prefixe == 0 || (u32::from(a) ^ u32::from(b)) >> (32 - prefixe) == 0,
        (IpAddr::V6(a), IpAddr::V6(b)) => prefixe == 0 || (u128::from(a) ^ u128::from(b)) >> (128 - prefixe) == 0,
        _ => false,
    }
}

/// « 192.168.1.2 », « fe80::1%eth0 », « ::ffff:192.168.1.2 » -> l'adresse, l'IPv4 enveloppée déballée.
pub fn lire_ip(texte: &str) -> Option<IpAddr> {
    texte.split('%').next()?.parse::<IpAddr>().ok().map(deballer)
}

pub fn deballer(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip),
        v4 => v4,
    }
}

/// Adresse publique d'Internet (ipaddress.is_global de Python) : ni privée, ni
/// partagée (100.64/10), ni réservée.
pub fn globale(ip: IpAddr) -> bool {
    const RESERVEES: &[&str] = &[
        "0.0.0.0/8", "10.0.0.0/8", "100.64.0.0/10", "127.0.0.0/8", "169.254.0.0/16", "172.16.0.0/12",
        "192.0.0.0/24", "192.0.2.0/24", "192.168.0.0/16", "198.18.0.0/15", "198.51.100.0/24",
        "203.0.113.0/24", "240.0.0.0/4",
        "::/127", "::ffff:0:0/96", "64:ff9b:1::/48", "100::/64", "2001::/23", "2001:db8::/32",
        "2002::/16", "3fff::/20", "fc00::/7", "fe80::/10",
    ];
    !RESERVEES.iter().any(|r| dans(ip, r))
}

fn ip_par_route(cible: &str) -> Option<IpAddr> {
    let cible: IpAddr = cible.parse().ok()?;
    let s = UdpSocket::bind(if cible.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" }).ok()?;
    s.connect(SocketAddr::new(cible, 1)).ok()?; // aucun paquet n'est envoyé
    Some(s.local_addr().ok()?.ip())
}

/// Adresse Tailscale (100.64/10) de ce poste. 100.100.100.100 est l'adresse
/// interne de Tailscale : sa route passe par tailscale0.
pub fn ip_tailscale() -> Option<IpAddr> {
    ip_par_route("100.100.100.100").filter(|ip| dans(*ip, "100.64.0.0/10"))
}

/// L'adresse IPv4 du poste sur le réseau local.
pub fn adresse_reseau() -> Option<IpAddr> {
    ip_par_route("192.168.255.255").filter(|ip| !ip.is_loopback()).or_else(|| {
        (nom_poste().as_str(), 0).to_socket_addrs().ok()?.map(|a| a.ip()).find(|ip| ip.is_ipv4() && !ip.is_loopback())
    })
}

pub fn nom_poste() -> String {
    gethostname::gethostname().to_string_lossy().into_owned()
}

/// Le nom MagicDNS exact de CE poste (pilou.tailxxxx.ts.net), "" sans Tailscale.
pub fn nom_tailscale() -> String {
    // shortcut: gardé jusqu'au redémarrage de l'outil, à revoir si le poste change de nom Tailscale
    static NOM: OnceLock<String> = OnceLock::new();
    NOM.get_or_init(|| {
        let sortie = Command::new("tailscale").args(["status", "--json", "--peers=false"]).stdin(Stdio::null()).output();
        // shortcut: pas de délai de 3 s comme en Python, « status --peers=false » répond aussitôt
        sortie
            .ok()
            .and_then(|r| serde_json::from_slice::<Value>(&r.stdout).ok())
            .and_then(|v| v["Self"]["DNSName"].as_str().map(|n| n.to_lowercase().trim_end_matches('.').to_string()))
            .unwrap_or_default()
    })
    .clone()
}

/// L'en-tête Host désigne-t-il ce poste ? (parade au DNS rebinding)
///
/// Une page piégée qui fait résoudre son nom vers 127.0.0.1 devient « de même
/// origine » ; elle envoie alors SON nom dans Host. Une IP littérale ne se
/// rebranche pas : on accepte toutes les IP, localhost, le nom du poste et son nom
/// Tailscale exact (jamais tout *.ts.net : n'importe qui peut en avoir un).
pub fn hote_permis(entete: Option<&str>) -> bool {
    hote_permis_avec(entete, &nom_tailscale)
}

fn hote_permis_avec(entete: Option<&str>, tailscale: &dyn Fn() -> String) -> bool {
    let h = entete.unwrap_or("").trim().to_lowercase();
    let mut h = h.trim_end_matches('.');
    if let Some(reste) = h.strip_prefix('[') {
        let Some((dedans, _)) = reste.split_once(']') else { return false }; // « [ » seul : rien à reconnaître
        h = dedans;
    } else if h.matches(':').count() == 1 {
        h = h.split(':').next().unwrap_or("");
    }
    let nom = nom_poste().to_lowercase();
    if h.is_empty() || h == "localhost" || h == nom || h == format!("{}.local", nom) {
        return true;
    }
    if h.ends_with(".ts.net") {
        return h == tailscale();
    }
    h.split('%').next().unwrap_or("").parse::<IpAddr>().is_ok()
}

// -- HTTPS --------------------------------------------------------------------
// La CA ne peut signer QUE des adresses privées et des noms locaux (nameConstraints) :
// installée sur une tablette, elle ne pourrait pas servir à imiter un site d'Internet,
// même si sa clé quittait ce poste.
// fc00::/7 (ULA) contient le fd7a:115c:a1e0::/48 de Tailscale. Changer PLAGES refait la CA
// (ca.json) : chaque appareil doit alors la réinstaller.
pub const PLAGES: &[&str] = &["10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16", "100.64.0.0/10", "127.0.0.0/8", "fc00::/7"];
const TLS_JOURS: u64 = 397; // en dessous de la limite d'Apple (398) ; renouvelé à 30 jours

pub fn dossier_tls() -> PathBuf {
    fichier_jeton().parent().unwrap_or(Path::new(".")).join("tls")
}

fn dans_le_path(nom: &str) -> Option<PathBuf> {
    let exe = if cfg!(windows) { format!("{}.exe", nom) } else { nom.to_string() };
    std::env::split_paths(&std::env::var_os("PATH")?).map(|d| d.join(&exe)).find(|p| p.is_file())
}

pub fn openssl() -> Option<PathBuf> {
    dans_le_path("openssl").or_else(|| {
        [r"C:\Program Files\Git\usr\bin\openssl.exe", r"C:\Program Files\Git\mingw64\bin\openssl.exe"]
            .iter()
            .map(PathBuf::from)
            .find(|p| p.exists())
    })
}

fn nom_local() -> String {
    let nom = nom_poste().split('.').next().unwrap_or("").to_lowercase();
    let ok = (1..=63).contains(&nom.len()) && nom.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-');
    if ok { nom } else { String::new() }
}

/// Les adresses privées de ce poste (réseau local, Tailscale, boucle locale), IPv4 d'abord.
pub fn adresses_poste() -> Vec<String> {
    let mut ips: Vec<IpAddr> = vec!["127.0.0.1".parse().unwrap()];
    // fd7a:115c:a1e0::53 : le DNS de Tailscale ; fdff:… : une ULA du réseau local, s'il en a
    for cible in ["192.168.255.255", "10.255.255.255", "172.31.255.255", "fd7a:115c:a1e0::53", "fdff:ffff:ffff:ffff::1"] {
        ips.extend(ip_par_route(cible));
    }
    if let Ok(adresses) = (nom_poste().as_str(), 0).to_socket_addrs() {
        ips.extend(adresses.map(|a| a.ip()));
    }
    ips.extend(ip_tailscale());
    let mut textes: Vec<String> =
        ips.into_iter().filter(|ip| PLAGES.iter().any(|p| dans(*ip, p))).map(|ip| ip.to_string()).collect();
    textes.sort_by_key(|i| (i.contains(':'), i.clone()));
    textes.dedup();
    textes
}

/// « adresse/masque » comme openssl l'attend dans nameConstraints.
fn masque(reseau: &str) -> String {
    let (base, prefixe) = reseau.split_once('/').unwrap();
    let n: u32 = prefixe.parse().unwrap();
    match base.parse::<IpAddr>().unwrap() {
        IpAddr::V4(_) => format!("{}/{}", base, std::net::Ipv4Addr::from(u32::MAX.checked_shl(32 - n).unwrap_or(0))),
        IpAddr::V6(_) => format!("{}/{}", base, Ipv6Addr::from(u128::MAX.checked_shl(128 - n).unwrap_or(0))),
    }
}

fn erreur(texte: String) -> std::io::Error {
    std::io::Error::other(texte)
}

/// Crée la CA (si ca_neuve) puis le certificat du serveur, par la commande openssl
/// (fichiers de configuration : la même syntaxe pour OpenSSL et LibreSSL).
fn generer(dossier: &Path, openssl: &Path, adresses: &[String], ca_neuve: bool) -> std::io::Result<()> {
    let nom = nom_local();
    let tmp = dossier.join(format!("tmp{}", alea(6).iter().map(|b| format!("{:02x}", b)).collect::<String>()));
    fs::create_dir(&tmp)?;
    let resultat = (|| {
        let lancer = |args: &[&str]| -> std::io::Result<()> {
            let r = Command::new(openssl).args(args).current_dir(&tmp).stdin(Stdio::null()).output()?;
            if !r.status.success() {
                let texte = String::from_utf8_lossy(if r.stderr.is_empty() { &r.stdout } else { &r.stderr }).trim().to_string();
                let fin: String = texte.chars().rev().take(300).collect::<Vec<_>>().into_iter().rev().collect();
                return Err(erreur(format!("openssl {} : {}", args[0], fin)));
            }
            Ok(())
        };
        if ca_neuve {
            let mut contraintes: Vec<String> = PLAGES.iter().map(|p| format!("permitted;IP:{}", masque(p))).collect();
            contraintes.push("permitted;DNS:localhost".into());
            contraintes.push("permitted;DNS:local".into());
            if !nom.is_empty() {
                contraintes.push(format!("permitted;DNS:{}", nom));
            }
            fs::write(
                tmp.join("ca.cnf"),
                format!(
                    "[req]\ndistinguished_name=dn\nprompt=no\nx509_extensions=v3\n\
                     [dn]\nCN=WEB tools {} (CA locale)\n[v3]\n\
                     basicConstraints=critical,CA:TRUE,pathlen:0\n\
                     keyUsage=critical,keyCertSign,cRLSign\nsubjectKeyIdentifier=hash\n\
                     nameConstraints=critical,{}\n",
                    if nom.is_empty() { "poste" } else { &nom },
                    contraintes.join(",")
                ),
            )?;
            lancer(&["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-sha256", "-days", "3650", "-config", "ca.cnf", "-keyout", "ca.key", "-out", "ca.pem"])?;
            fs::write(tmp.join("ca.json"), json!({ "plages": PLAGES }).to_string())?;
        } else {
            for f in ["ca.key", "ca.pem"] {
                fs::copy(dossier.join(f), tmp.join(f))?;
            }
        }
        let mut noms = vec!["localhost".to_string()];
        if !nom.is_empty() {
            noms.push(nom.clone());
            noms.push(format!("{}.local", nom));
        }
        fs::write(
            tmp.join("req.cnf"),
            format!("[req]\ndistinguished_name=dn\nprompt=no\n[dn]\nCN={}\n", if nom.is_empty() { "localhost" } else { &nom }),
        )?;
        let san: Vec<String> =
            noms.iter().map(|n| format!("DNS:{}", n)).chain(adresses.iter().map(|i| format!("IP:{}", i))).collect();
        fs::write(
            tmp.join("ext.cnf"),
            format!(
                "[v3]\nbasicConstraints=critical,CA:FALSE\n\
                 keyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n\
                 subjectAltName={}\n",
                san.join(",")
            ),
        )?;
        lancer(&["req", "-new", "-newkey", "rsa:2048", "-nodes", "-config", "req.cnf", "-keyout", "serveur.key", "-out", "serveur.csr"])?;
        let serie = (u64::from_le_bytes(alea(8).try_into().unwrap()) >> 1).to_string();
        lancer(&[
            "x509", "-req", "-sha256", "-days", &TLS_JOURS.to_string(), "-in", "serveur.csr", "-CA", "ca.pem", "-CAkey", "ca.key",
            "-set_serial", &serie, "-extfile", "ext.cnf", "-extensions", "v3", "-out", "serveur.pem",
        ])?;
        fs::write(
            tmp.join("serveur.json"),
            json!({ "adresses": adresses, "noms": noms, "expire": horloge() + (TLS_JOURS * 86400) as f64 }).to_string(),
        )?;
        #[cfg(unix)]
        for f in ["ca.key", "serveur.key"] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(tmp.join(f), fs::Permissions::from_mode(0o600))?;
        }
        // la CA d'abord : un tool qui lit entre deux trouve au pire un certificat à refaire
        let ca: &[&str] = if ca_neuve { &["ca.key", "ca.pem", "ca.json"] } else { &[] };
        for f in ca.iter().chain(&["serveur.key", "serveur.pem", "serveur.json"]) {
            fs::rename(tmp.join(f), dossier.join(f))?;
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(&tmp);
    resultat
}

fn lire_json(chemin: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(chemin).ok()?).ok()
}

/// CA absente, ou faite pour d'autres PLAGES (une CA d'avant IPv6 n'a pas de ca.json) :
/// ses contraintes de nom refuseraient le certificat du serveur.
fn ca_a_refaire(dossier: &Path) -> bool {
    match lire_json(&dossier.join("ca.json")) {
        Some(d) => d["plages"] != json!(PLAGES) || !dossier.join("ca.pem").exists() || !dossier.join("ca.key").exists(),
        None => true,
    }
}

fn a_refaire(dossier: &Path, adresses: &[String]) -> bool {
    let Some(d) = lire_json(&dossier.join("serveur.json")) else { return true };
    let (Some(connues), Some(expire)) = (d["adresses"].as_array(), d["expire"].as_f64()) else { return true };
    ca_a_refaire(dossier)
        || !adresses.iter().all(|a| connues.iter().any(|c| c.as_str() == Some(a)))
        || expire - horloge() < 30.0 * 86400.0
        || !dossier.join("serveur.pem").exists()
}

/// SHA-256 du certificat de la CA, comme iOS l'affiche (« Plus de détails »).
pub fn empreinte_ca() -> std::io::Result<String> {
    let pem = fs::read(dossier_tls().join("ca.pem"))?;
    let der = rustls_pemfile::certs(&mut &pem[..]).next().ok_or_else(|| erreur("ca.pem sans certificat".into()))??;
    Ok(Sha256::digest(&der).iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "))
}

/// Ok(configuration TLS du serveur) ou Err(raison de rester en HTTP).
pub fn contexte_tls() -> Result<Arc<ServerConfig>, String> {
    if std::env::var("WEB_TOOLS_HTTP").as_deref() == Ok("1") {
        return Err("WEB_TOOLS_HTTP=1".into());
    }
    let dossier = dossier_tls();
    let adresses = adresses_poste();
    let verrou = dossier.join("verrou");
    let charger = || -> std::io::Result<Result<Arc<ServerConfig>, String>> {
        fs::create_dir_all(&dossier)?;
        if a_refaire(&dossier, &adresses) {
            let Some(openssl) = openssl() else { return Ok(Err("la commande openssl est introuvable".into())) };
            // Deux tools qui démarrent ensemble : un seul crée la CA, l'autre attend.
            let mut tenu = false;
            for _ in 0..300 {
                match creer_prive(&verrou, true) {
                    Ok(_) => {
                        tenu = true;
                        break;
                    }
                    Err(_) => {
                        let vieux = fs::metadata(&verrou).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok());
                        if vieux.is_some_and(|d| d > Duration::from_secs(120)) {
                            let _ = fs::remove_file(&verrou); // laissé par un tool tué en route
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
            if !tenu {
                return Ok(Err(format!("verrou {} tenu trop longtemps", verrou.display())));
            }
            let fait = if a_refaire(&dossier, &adresses) {
                generer(&dossier, &openssl, &adresses, ca_a_refaire(&dossier))
            } else {
                Ok(())
            };
            let _ = fs::remove_file(&verrou);
            fait?;
        }
        let pem = fs::read(dossier.join("serveur.pem"))?;
        let certificats = rustls_pemfile::certs(&mut &pem[..]).collect::<Result<Vec<_>, _>>()?;
        let cle = rustls_pemfile::private_key(&mut &fs::read(dossier.join("serveur.key"))?[..])?
            .ok_or_else(|| erreur("serveur.key sans clé".into()))?;
        let cfg = ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions() // TLS 1.2 au minimum
            .map_err(|e| erreur(e.to_string()))?
            .with_no_client_auth()
            .with_single_cert(certificats, cle)
            .map_err(|e| erreur(e.to_string()))?;
        Ok(Ok(Arc::new(cfg)))
    };
    charger().unwrap_or_else(|e| Err(format!("certificat impossible : {}", e)))
}

/// Une IPv6 littérale va entre crochets dans une adresse : https://[fd7a::1]:8080/.
pub fn hote_url(h: &str) -> String {
    if h.contains(':') && !h.starts_with('[') { format!("[{}]", h) } else { h.to_string() }
}

/// Ce que le terminal affiche à côté de l'adresse réseau.
pub fn ligne_https(nom_ou_ip: &str, port: u16) -> String {
    format!(
        "  HTTPS : la première fois, sur chaque appareil, installez la CA du poste :\n\
         \x20         https://{}:{}{}\n\
         \x20         (iPad : Réglages › Profil téléchargé › Installer, puis Général ›\n\
         \x20          Informations › Réglages des certificats › activer)\n\
         \x20         empreinte SHA-256 : {}",
        hote_url(nom_ou_ip),
        port,
        CA,
        empreinte_ca().unwrap_or_default()
    )
}

/// None : la requête suit son cours. Sert la CA du poste (sans jeton : elle est
/// publique), et renvoie vers https:// un autre appareil arrivé en HTTP.
pub fn filtrer_tls(r: &Requete) -> Option<Reponse> {
    if !r.tls_actif {
        return None;
    }
    if r.lecture() && r.chemin() == CA {
        return Some(match fs::read(dossier_tls().join("ca.pem")) {
            // ce type-là : Safari propose d'installer le profil
            Ok(corps) => Reponse::new(200, "application/x-x509-ca-cert", corps)
                .entete("Content-Disposition", "attachment; filename=\"web-tools-ca.pem\""),
            Err(_) => Reponse::texte(404, "CA introuvable"),
        });
    }
    if r.https || ce_poste(r) {
        return None;
    }
    Some(Reponse::texte(307, "HTTPS").entete("Location", &format!("https://{}{}", r.entete("Host").unwrap_or(""), r.cible)))
}

// -- le code ------------------------------------------------------------------

struct EtatCode {
    code: Option<String>,
    expire: f64,
    echecs: u32,
    bloque: f64,
    grilles: u32, // codes brûlés d'affilée
}

/// Le code court. Deviner : un code ne vit que DUREE secondes, sert une fois, et
/// ESSAIS erreurs le brûlent, avec PAUSE secondes avant le suivant. Chaque nouveau
/// code s'affiche dans le terminal : une série d'essais se voit.
pub struct Appairage {
    afficher: Afficher,
    nom: String,
    etat: Mutex<EtatCode>,
}

impl Appairage {
    pub const DUREE: f64 = 600.0;
    pub const ESSAIS: u32 = 5;
    pub const PAUSE: f64 = 60.0;
    pub const PAUSE_MAX: f64 = 3600.0;

    pub fn new(afficher: Option<Afficher>, nom: &str) -> Self {
        Appairage {
            afficher: afficher.unwrap_or_else(console),
            nom: nom.to_string(),
            etat: Mutex::new(EtatCode { code: None, expire: 0.0, echecs: 0, bloque: 0.0, grilles: 0 }),
        }
    }

    pub fn lisible(code: &str) -> String {
        format!("{} {}", &code[..3], &code[3..])
    }

    fn neuf(&self, e: &mut EtatCode, maintenant: f64) {
        // tirage sans biais : 2^32 n'est pas un multiple de 10^6
        let n = loop {
            let n = u32::from_le_bytes(alea(4).try_into().unwrap());
            if n < 4_294_000_000 {
                break n % 1_000_000;
            }
        };
        let code = format!("{:06}", n);
        e.expire = maintenant + Self::DUREE;
        // echecs n'est PAS remis à zéro ici : sinon 4 essais par code, en attendant
        // chaque expiration, contournaient la pause (576 essais par jour)
        (self.afficher)(&format!(
            "  Code d'appairage{} : {}   (à saisir sur la tablette, valable {} min)",
            if self.nom.is_empty() { String::new() } else { format!(" {}", self.nom) },
            Self::lisible(&code),
            Self::DUREE as u32 / 60
        ));
        e.code = Some(code);
    }

    /// Un code valable existe (en créer un, et l'afficher, s'il le faut).
    /// Renvoie les secondes d'attente si les essais sont suspendus, sinon 0.
    pub fn preparer(&self) -> u64 {
        let mut e = self.etat.lock().unwrap();
        let maintenant = mono();
        if maintenant < e.bloque {
            return (e.bloque - maintenant) as u64 + 1;
        }
        if e.code.is_none() || maintenant >= e.expire {
            self.neuf(&mut e, maintenant);
        }
        0
    }

    /// Renvoie (ok, message).
    pub fn essayer(&self, saisi: &str) -> (bool, String) {
        let saisi: String = saisi.chars().filter(|c| c.is_ascii_digit()).collect();
        let mut e = self.etat.lock().unwrap();
        let maintenant = mono();
        if maintenant < e.bloque {
            return (false, format!("Trop d'essais : attendez {} s, un nouveau code s'affichera sur le serveur.", (e.bloque - maintenant) as u64 + 1));
        }
        if e.code.is_none() || maintenant >= e.expire {
            self.neuf(&mut e, maintenant);
            return (false, "Code expiré : un nouveau code est affiché sur le serveur.".into());
        }
        if saisi.len() == 6 && egal(&saisi, e.code.as_deref().unwrap_or("")) {
            e.code = None; // usage unique
            e.grilles = 0;
            e.echecs = 0;
            return (true, "Appareil autorisé.".into());
        }
        e.echecs += 1;
        if e.echecs >= Self::ESSAIS {
            // pause doublée à chaque code brûlé d'affilée (plafond 1 h) : à pause fixe,
            // 7 200 essais par jour suffisaient à deviner un code de 6 chiffres en quelques semaines
            let pause = (Self::PAUSE * 2f64.powi(e.grilles as i32)).min(Self::PAUSE_MAX);
            e.grilles += 1;
            e.echecs = 0;
            e.code = None;
            e.bloque = maintenant + pause;
            return (false, format!("Trop d'essais : code annulé. Dans {} s, rechargez la page : un nouveau code s'affichera sur le serveur.", pause as u64));
        }
        (false, format!("Code incorrect ({} essai(s) restant(s)).", Self::ESSAIS - e.echecs))
    }
}

/// Comparaison en temps constant (hmac.compare_digest).
pub fn egal(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
}

/// Les appareils qui utilisent un serveur, affichés dans son terminal : une ligne
/// quand une IP arrive, ou revient après INACTIF secondes sans requête.
pub struct Visites {
    nom: String,
    afficher: Afficher,
    vues: Mutex<BTreeMap<String, f64>>, // ip -> dernière requête
}

impl Visites {
    pub const INACTIF: f64 = 600.0;

    pub fn new(nom: &str, afficher: Option<Afficher>) -> Self {
        Visites { nom: nom.to_string(), afficher: afficher.unwrap_or_else(console), vues: Mutex::new(BTreeMap::new()) }
    }

    pub fn noter(&self, ip: IpAddr) {
        let ip = deballer(ip).to_string();
        let maintenant = mono();
        let actifs: Vec<String> = {
            let mut vues = self.vues.lock().unwrap();
            let arrivee = vues.get(&ip).is_none_or(|t| maintenant - t > Self::INACTIF);
            vues.retain(|_, t| maintenant - *t <= Self::INACTIF);
            vues.insert(ip.clone(), maintenant);
            if !arrivee {
                return;
            }
            vues.keys().cloned().collect()
        };
        (self.afficher)(&format!(
            "  {} {}  {:<11} {}  ->  {} appareil(s) actif(s) : {}",
            MARQUE, heure(), self.nom, ip, actifs.len(), actifs.join(", ")
        ));
    }
}

// -- le contrôle d'accès --------------------------------------------------------

/// La requête vient-elle de ce poste (boucle locale) ?
/// WEB_TOOLS_BANC_DISTANT=1 (bancs d'essai) : ce poste est traité comme un autre
/// appareil ; la variable ne peut que retirer des droits.
pub fn ce_poste(r: &Requete) -> bool {
    static DISTANT: OnceLock<bool> = OnceLock::new();
    let distant = *DISTANT.get_or_init(|| std::env::var("WEB_TOOLS_BANC_DISTANT").as_deref() == Ok("1"));
    #[cfg(test)]
    if essais::DISTANT.load(std::sync::atomic::Ordering::SeqCst) {
        return false;
    }
    !distant && deballer(r.client.ip()).is_loopback()
}

/// Le client a-t-il une adresse publique ? (IPv6 de la box ouverte, redirection de port)
/// Réseau local, lien local et Tailscale (100.64/10, fd7a::) ne sont pas publics, ni une
/// IPv6 globale du /64 de l'IPv6 globale du poste où elle arrive : un appareil du réseau
/// local qui a trouvé pilou.local par mDNS (la box distribue ce /64 au seul réseau local).
pub fn internet(client: IpAddr, ici: IpAddr) -> bool {
    let ip = deballer(client);
    if !globale(ip) {
        return false;
    }
    let (IpAddr::V6(ip), IpAddr::V6(ici)) = (ip, ici) else { return true };
    let meme_64 = u128::from(ip) >> 64 == u128::from(ici) >> 64;
    !(globale(IpAddr::V6(ici)) && ici.to_ipv4_mapped().is_none() && meme_64)
}

/// La valeur du cookie `nom` dans l'en-tête Cookie.
pub fn biscuit_recu(r: &Requete, nom: &str) -> Option<String> {
    r.entete("Cookie")?.split(';').find_map(|morceau| {
        let (k, v) = morceau.trim().split_once('=')?;
        (k == nom).then(|| v.trim_matches('"').to_string())
    })
}

/// Le jeton est-il dans l'adresse (?jeton=) ou dans le cookie ?
pub fn jeton_recu(r: &Requete, jeton: &str) -> bool {
    [r.param("jeton"), biscuit_recu(r, BISCUIT)].into_iter().flatten().any(|recu| !recu.is_empty() && egal(&recu, jeton))
}

/// Secure en HTTPS : le navigateur ne le renverra plus jamais en clair.
pub fn biscuit(jeton: &str, https: bool) -> String {
    format!("{}={}; Path=/; Max-Age=31536000; HttpOnly; SameSite=Strict{}", BISCUIT, jeton, if https { "; Secure" } else { "" })
}

fn html(texte: &str) -> String {
    texte.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Le contrôle d'accès d'un serveur, à appeler par filtrer() en tête de chaque requête.
///
/// actif=false (le serveur n'écoute que sur 127.0.0.1) : seul l'en-tête Host est vérifié.
pub struct Garde {
    pub nom: String,
    pub actif: bool,
    pub jeton: Option<String>,
    pub code: Option<Appairage>,
    pub visites: Visites,
    https: OnceLock<bool>, // en mode réseau, après brancher()
    port: OnceLock<u16>,
    sans_tls: OnceLock<String>, // … ou pourquoi tout reste en HTTP
}

impl Garde {
    pub fn new(nom: &str, actif: bool, afficher: Option<Afficher>, ancien: Option<&Path>) -> Self {
        Garde {
            nom: nom.to_string(),
            actif,
            jeton: actif.then(|| jeton(ancien)),
            code: actif.then(|| Appairage::new(afficher.clone(), nom)),
            visites: Visites::new(nom, afficher),
            https: OnceLock::new(),
            port: OnceLock::new(),
            sans_tls: OnceLock::new(),
        }
    }

    /// En mode réseau : HTTPS sur le port du serveur. Rend le schéma des adresses.
    pub fn brancher(&self, serveur: &Serveur) -> &'static str {
        let _ = self.port.set(serveur.port());
        if self.actif {
            match contexte_tls() {
                Ok(cfg) => {
                    serveur.brancher_tls(cfg);
                    let _ = self.https.set(true);
                }
                Err(raison) => {
                    let _ = self.sans_tls.set(raison);
                }
            }
        }
        self.schema()
    }

    pub fn schema(&self) -> &'static str {
        if self.https.get().copied().unwrap_or(false) { "https" } else { "http" }
    }

    pub fn adresse(&self, hote: &str, port: u16) -> String {
        format!("{}://{}:{}/", self.schema(), hote_url(hote), port)
    }

    /// Au démarrage, en mode réseau : affiche le code dans le terminal.
    pub fn annoncer(&self) {
        let Some(code) = &self.code else { return };
        if self.schema() == "https" {
            let hors = adresses_poste().into_iter().find(|i| !["127.", "100.", "fd7a:115c:a1e0:"].iter().any(|p| i.starts_with(p)));
            println!("{}", ligne_https(hors.as_deref().unwrap_or("127.0.0.1"), self.port.get().copied().unwrap_or(0)));
        } else if let Some(raison) = self.sans_tls.get() {
            println!("  HTTP en clair sur le réseau ({}) : le jeton peut être lu sur le Wi-Fi.", raison);
        }
        code.preparer();
    }

    pub fn autorise(&self, r: &Requete) -> bool {
        !self.actif || ce_poste(r) || jeton_recu(r, self.jeton.as_deref().unwrap_or(""))
    }

    pub fn biscuit(&self, r: &Requete) -> String {
        biscuit(self.jeton.as_deref().unwrap_or(""), r.https)
    }

    /// None : la requête suit son cours. Some : la réponse à envoyer.
    pub fn filtrer(&self, r: &mut Requete) -> Option<Reponse> {
        if !hote_permis(r.entete("Host")) {
            return Some(Reponse::texte(403, "Host non autorisé (protection DNS rebinding)"));
        }
        if internet(r.client.ip(), r.local.ip()) {
            return Some(Reponse::texte(403, "Accès depuis Internet refusé : passez par le réseau local ou Tailscale"));
        }
        if let Some(reponse) = filtrer_tls(r) {
            return Some(reponse);
        }
        if !self.actif {
            self.visites.noter(r.client.ip());
            return None;
        }
        let code = self.code.as_ref().expect("garde actif");
        let chemin = r.chemin().to_string();
        let lecture = r.lecture();
        if self.autorise(r) {
            self.visites.noter(r.client.ip());
            if lecture && r.param("jeton").is_some() {
                // Le jeton passe dans un cookie et quitte l'adresse : il ne reste
                // ni dans l'historique, ni dans un lien recopié.
                let reste: Vec<String> = params(r.requete())
                    .iter()
                    .filter(|(k, v)| k != "jeton" && !v.is_empty())
                    .map(|(k, v)| format!("{}={}", encoder_url(k), encoder_url(v)))
                    .collect();
                let ou = if reste.is_empty() { chemin } else { format!("{}?{}", chemin, reste.join("&")) };
                return Some(Reponse::texte(303, "").entete("Set-Cookie", &self.biscuit(r)).entete("Location", &ou));
            }
            if lecture && chemin == PAGE {
                return Some(Reponse::texte(303, "").entete("Location", "/"));
            }
            if r.methode == "POST" && chemin == ROUTE {
                return Some(Reponse::json(200, &json!({"ok": true, "message": "Cet appareil est déjà autorisé."})));
            }
            return None;
        }
        if lecture && chemin == PAGE {
            code.preparer();
            return Some(Reponse::new(200, "text/html; charset=utf-8", PAGE_HTML.replace("__NOM__", &html(&self.nom))));
        }
        if r.methode == "POST" && chemin == ROUTE && r.entete(ENTETE) == Some("1") {
            let corps: Value = r.lire_corps(4096).ok().and_then(|c| serde_json::from_slice(&c).ok()).unwrap_or(Value::Null);
            let saisi = match &corps["code"] {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => String::new(),
            };
            let (ok, texte) = code.essayer(&saisi);
            let reponse = Reponse::json(if ok { 200 } else { 403 }, &json!({"ok": ok, "message": texte}));
            return Some(if ok { reponse.entete("Set-Cookie", &self.biscuit(r)) } else { reponse });
        }
        if lecture && (chemin == "/" || chemin == "/index.html" || r.entete("Accept").unwrap_or("").contains("text/html")) {
            // une page : vers celle du code (le code s'affiche dans le terminal du serveur)
            return Some(Reponse::texte(303, "").entete("Location", PAGE));
        }
        Some(Reponse::json(403, &json!({"detail": "Appareil non appairé : ouvrez la page de ce serveur et saisissez le code d'appairage."})))
    }
}

#[cfg(test)]
mod essais;
