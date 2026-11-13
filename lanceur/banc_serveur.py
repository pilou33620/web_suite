"""Banc du serveur du lanceur : Host, jeton du mode réseau, en-tête X-WebSuite.

    python lanceur/banc_serveur.py

Pilote l'exécutable Rust web_suite en boîte noire, sur 127.0.0.1, dans un dossier
temporaire sans aucun outil (rien n'est lancé, cloné ni poussé : un dépôt nu local
joue GitHub). L'appareil distant est joué par WEB_TOOLS_BANC_DISTANT=1 : le serveur
ne fait alors plus confiance à la boucle locale. Affiche OK si tout passe.

Ce que l'ancien banc vérifiait en touchant l'intérieur du serveur est devenu des
tests Rust (cargo test) :
  - client d'Internet refusé (8.8.8.8, 2a01:…, ::ffff:8.8.8.8), fin de la pause après
    cinq erreurs, code expiré refusé et remplacé : commun/src/appairage/essais.rs ;
  - journal tourné en .log.1, pas de bouton openEMS hors Windows, archive openEMS à
    l'empreinte fausse refusée, code d'appairage lu dans le journal d'un outil :
    lanceur/src/outils/essais.rs.
"""
import json
import os
import ssl
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from banc_lanceur import Poste  # noqa: E402


def verifier(nom, obtenu, attendu):
    assert obtenu == attendu, "%s : attendu %r, obtenu %r" % (nom, attendu, obtenu)


def arret_propre(lanceur):
    if os.name != "nt":                                   # sous Windows, terminate() tue sans prévenir
        verifier("SIGTERM : arrêt propre", (lanceur.arreter(), "Arrêt." in lanceur.sortie()), (0, True))


with Poste() as poste:
    # Écoute locale : pas de jeton, mais Host contrôlé
    with poste.lanceur("--local") as l:
        req = l.req
        verifier("page locale", req("/").status, 200)
        verifier("Host étranger", req("/", hote="evil.example:8100").status, 403)
        verifier("Host étranger en POST", req("/api/arreter", "POST", hote="evil.example",
                                              entetes={"X-WebSuite": "1"}, corps=b'{"id":"web_3d"}').status, 403)
        verifier("sans X-WebSuite", req("/api/arreter", "POST", corps=b'{"id":"web_3d"}').status, 403)
        verifier("corps géant", req("/api/arreter", "POST", entetes={"X-WebSuite": "1",
                                    "Content-Length": str(10 ** 9)}).status, 413)
        verifier("JSON non objet", req("/api/arreter", "POST", entetes={"X-WebSuite": "1"}, corps=b"[]").status, 400)
        arret_propre(l)

    # Mode réseau, appareil distant
    with poste.lanceur("--reseau", WEB_TOOLS_BANC_DISTANT="1", WEB_TOOLS_HTTP="1") as l:
        req = l.req
        jeton = poste.jeton()
        r = req("/")
        verifier("distant sans jeton : renvoyé vers la page du code", (r.status, r.getheader("Location")), (303, "/appairage"))
        r = req("/appairage")
        verifier("distant sans jeton : page du code", (r.status, "appairer" in r.corps.decode()), (200, True))
        verifier("page du code jamais en cache", r.getheader("Cache-Control"), "no-store")
        verifier("le code s'affiche dans le terminal", len(l.codes()), 1)
        verifier("distant sans jeton : pas l'état", req("/api/etat").status, 403)
        verifier("distant sans jeton : pas l'intro", req("/websuite-intro.html").status, 403)
        verifier("distant sans jeton : pas d'action", req("/api/arreter", "POST", entetes={"X-WebSuite": "1"},
                                                        corps=b'{"id":"web_3d"}').status, 403)
        verifier("distant, faux jeton", req("/?jeton=faux").getheader("Location"), "/appairage")
        verifier("distant, faux biscuit", req("/api/etat", entetes={"Cookie": "websuite_jeton=faux"}).status, 403)
        r = req("/?jeton=" + jeton)
        verifier("bon jeton -> redirection", r.status, 303)
        verifier("redirection sans jeton", r.getheader("Location"), "/")
        biscuit = r.getheader("Set-Cookie").split(";")[0]
        verifier("biscuit -> page", req("/", entetes={"Cookie": biscuit}).status, 200)
        r = req("/api/arreter", "POST", entetes={"Cookie": biscuit, "X-WebSuite": "1"}, corps=b'{"id":"web_3d"}')
        verifier("biscuit -> action", (r.status, json.loads(r.corps)["ok"]), (200, True))
        l.attendre("arrête WEB_3D")                         # action reçue, et dite dans le terminal
        verifier("biscuit illisible -> page du code", req("/", entetes={"Cookie": '";;=='}).getheader("Location"), "/appairage")

        # Code d'appairage : le bon donne le cookie du jeton, une seule fois.
        def appairer(code, x=True):
            return req("/api/appairer", "POST", entetes={"X-WebSuite": "1"} if x else {},
                       corps=json.dumps({"code": code}).encode())

        code = l.codes()[-1]
        verifier("code à 6 chiffres", (len(code), code.isdigit()), (6, True))
        verifier("appairer sans X-WebSuite", appairer(code, x=False).status, 403)
        faux = "%06d" % ((int(code) + 1) % 10 ** 6)
        r = appairer(faux)
        verifier("faux code refusé", (r.status, r.getheader("Set-Cookie")), (403, None))
        r = appairer(code[:3] + " " + code[3:])                   # tel qu'affiché, espace compris
        verifier("bon code accepté", r.status, 200)
        biscuit = r.getheader("Set-Cookie").split(";")[0]
        verifier("le cookie est celui du jeton", biscuit, "websuite_jeton=" + jeton)
        verifier("cookie du code -> outils", req("/api/etat", entetes={"Cookie": biscuit}).status, 200)
        verifier("code à usage unique", appairer(code).status, 403)
        # Le cas de l'iPad : code accepté, puis la page renvoie le code (copie en cache,
        # double validation). L'appareil est déjà autorisé : pas d'erreur 404.
        r = req("/api/appairer", "POST", entetes={"X-WebSuite": "1", "Cookie": biscuit},
                corps=json.dumps({"code": "999999"}).encode())
        verifier("déjà autorisé : le code renvoyé passe", (r.status, json.loads(r.corps)["ok"]), (200, True))
        r = req("/appairage", entetes={"Cookie": biscuit})
        verifier("déjà autorisé : la page du code renvoie au lanceur", (r.status, r.getheader("Location")), (303, "/"))
        r = req("/", entetes={"Cookie": biscuit})
        verifier("le lanceur jamais en cache", (r.status, r.getheader("Cache-Control")), (200, "no-store"))
        req("/appairage")                                         # un nouveau code pour la suite
        verifier("nouveau code affiché après usage", len(l.codes()) >= 2, True)
        # Cinq erreurs brûlent le code, et l'on attend avant le suivant.
        code = l.codes()[-1]
        for _ in range(5):                                        # Appairage::ESSAIS
            r = appairer("000000" if code != "000000" else "111111")
        verifier("cinq erreurs : code annulé", "code annulé" in json.loads(r.corps)["message"], True)
        verifier("pendant la pause, même le bon code échoue", appairer(code).status, 403)
        verifier("la pause est annoncée", "attendez" in json.loads(appairer(code).corps)["message"], True)
        verifier("pendant la pause, pas de nouveau code", (req("/appairage").status, l.codes()[-1]), (200, code))
        arret_propre(l)

    # HTTPS en réseau : même port, la boucle locale garde HTTP, un autre appareil en
    # HTTP est renvoyé vers https://, le cookie y est Secure.
    with poste.lanceur("--reseau") as l:
        verifier("ce poste sans jeton", l.req("/").status, 200)
        https = "HTTPS : la première fois" in l.sortie()
        if https:
            ca = os.path.join(os.path.dirname(poste.fichier_jeton), "tls", "ca.pem")
            client = ssl.create_default_context(cafile=ca)
            verifier("ce poste en HTTP", l.req("/").status, 200)
            verifier("ce poste en HTTPS", l.req("/", contexte=client).status, 200)
        else:
            verifier("pas de HTTPS : le terminal dit pourquoi", "HTTP en clair sur le réseau" in l.sortie()
                     or "adresse IP du poste introuvable" in l.sortie(), True)
            print("  (openssl absent : HTTPS non vérifié)")
    if https:
        with poste.lanceur("--reseau", WEB_TOOLS_BANC_DISTANT="1") as l:
            r = l.req("/api/etat")
            verifier("distant en HTTP : vers https", (r.status, r.getheader("Location")),
                     (307, "https://127.0.0.1:%d/api/etat" % l.port))
            verifier("la CA, sans jeton", l.req("/appairage/ca.pem").corps.startswith(b"-----BEGIN"), True)
            r = l.req("/?jeton=" + poste.jeton(), contexte=client)
            verifier("cookie Secure", (r.status, "; Secure" in r.getheader("Set-Cookie")), (303, True))
            verifier("page du code en HTTPS", 'id="ca"' in l.req("/appairage", contexte=client).corps.decode(), True)

    # Mise à jour de WEB_SUITE au démarrage : avec un script relanceur (WEB_SUITE_RELANCEUR=1)
    # le lanceur sort en 75 ; sinon il se relance lui-même et le bilan suit.
    def git(cwd, *args):
        subprocess.run(["git", "-c", "user.name=banc", "-c", "user.email=banc@example.com", *args], cwd=cwd,
                       env=poste.env, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def publier(n):                                           # un commit de plus sur le « GitHub » de WEB_SUITE
        with open(os.path.join(origine, "lanceur", "index.html"), "w", encoding="utf-8") as f:
            f.write("version %d" % n)
        git(origine, "add", "-A")
        git(origine, "commit", "-m", "version %d" % n)

    origine, suite = os.path.join(poste.tmp, "suite-origine"), os.path.join(poste.tmp, "suite")
    os.makedirs(os.path.join(origine, "lanceur"))
    git(origine, "init")
    publier(1)
    git(poste.tmp, "clone", origine, suite)
    publier(2)
    code, sortie = poste.lancer("web_suite", "--port", "0", "--racine", poste.racine, "--sans-navigateur", "--local",
                                WEB_SUITE_DEPOT=suite, WEB_SUITE_RELANCEUR="1")
    verifier("mise à jour avec relanceur : code 75", (code, "[*] WEB_SUITE mis à jour, redémarrage." in sortie), (75, True))
    if os.name != "nt":                                       # sous Windows le relancé est un enfant, que terminate() laisserait
        publier(3)
        with poste.lanceur("--local", maj=True, WEB_SUITE_DEPOT=suite) as l:
            verifier("mise à jour sans relanceur : relancé, bilan transmis",
                     "WEB_SUITE    Mis à jour (1 commit(s))." in l.sortie(), True)
            verifier("le relancé sert la nouvelle page", l.req("/").corps, b"version 3")
            arret_propre(l)
print("OK")
