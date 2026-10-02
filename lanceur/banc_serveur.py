"""Banc du serveur du lanceur : Host, jeton du mode réseau, en-tête X-WebSuite.

    python lanceur/banc_serveur.py

Démarre le serveur en mémoire sur 127.0.0.1 avec un gestionnaire factice (rien
n'est lancé, cloné ni poussé) et joue un appareil distant en masquant la boucle
locale. Affiche OK si tout passe.
"""
import http.client
import os
import sys
import tempfile
import threading

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import web_suite  # noqa: E402


class Factice:
    racine = tempfile.gettempdir()

    def etat(self):
        return []

    def arreter(self, oid):
        Factice.arrete = oid

    installer = lancer = installer_openems = arreter


class Distant(web_suite.Lanceur):
    distant = False

    def _de_ce_poste(self):
        return not Distant.distant and super()._de_ce_poste()


Distant.gestionnaire = Factice()
serveur = web_suite.Serveur(("127.0.0.1", 0), Distant)
threading.Thread(target=serveur.serve_forever, daemon=True).start()
PORT = serveur.server_address[1]


def req(chemin, methode="GET", hote=None, entetes=None, corps=b""):
    c = http.client.HTTPConnection("127.0.0.1", PORT, timeout=10)
    c.putrequest(methode, chemin, skip_host=True)
    c.putheader("Host", hote or "127.0.0.1:%d" % PORT)
    for k, v in (entetes or {}).items():
        c.putheader(k, v)
    if methode == "POST":
        c.putheader("Content-Length", (entetes or {}).get("Content-Length") or str(len(corps)))
    c.endheaders(corps if methode == "POST" and corps else None)
    r = c.getresponse()
    r.read()
    c.close()
    return r


def verifier(nom, obtenu, attendu):
    assert obtenu == attendu, "%s : attendu %r, obtenu %r" % (nom, attendu, obtenu)


# Écoute locale : pas de jeton, mais Host contrôlé
verifier("page locale", req("/").status, 200)
verifier("Host étranger", req("/", hote="evil.example:8100").status, 403)
verifier("Host étranger en POST", req("/api/arreter", "POST", hote="evil.example",
                                      entetes={"X-WebSuite": "1"}, corps=b'{"id":"web_3d"}').status, 403)
verifier("sans X-WebSuite", req("/api/arreter", "POST", corps=b'{"id":"web_3d"}').status, 403)
verifier("corps géant", req("/api/arreter", "POST", entetes={"X-WebSuite": "1",
                            "Content-Length": str(10 ** 9)}).status, 413)
verifier("JSON non objet", req("/api/arreter", "POST", entetes={"X-WebSuite": "1"}, corps=b"[]").status, 400)

# Mode réseau, appareil distant
Distant.jeton = "j" * 22
Distant.distant = True
verifier("distant sans jeton", req("/").status, 403)
verifier("distant, faux jeton", req("/?jeton=faux").status, 403)
verifier("distant, faux biscuit", req("/api/etat", entetes={"Cookie": "websuite_jeton=faux"}).status, 403)
r = req("/?jeton=" + Distant.jeton)
verifier("bon jeton -> redirection", r.status, 303)
verifier("redirection sans jeton", r.getheader("Location"), "/")
biscuit = r.getheader("Set-Cookie").split(";")[0]
verifier("biscuit -> page", req("/", entetes={"Cookie": biscuit}).status, 200)
verifier("biscuit -> action", req("/api/arreter", "POST", entetes={"Cookie": biscuit, "X-WebSuite": "1"},
                                  corps=b'{"id":"web_3d"}').status, 200)
verifier("action reçue", Factice.arrete, "web_3d")
verifier("biscuit illisible", req("/", entetes={"Cookie": '";;=='}).status, 403)
Distant.distant = False
verifier("ce poste sans jeton", req("/").status, 200)

serveur.shutdown()

# Journaux : un journal trop gros passe en .log.1 au lancement suivant.
outils = web_suite.outils
with tempfile.TemporaryDirectory() as tmp:
    journal = os.path.join(tmp, "web_3d.log")
    with open(journal, "w") as f:
        f.write("a" * (outils.JOURNAL_MAX + 1))
    outils._tourner(journal)
    verifier("journal tourné", (os.path.exists(journal), os.path.exists(journal + ".1")), (False, True))
    with open(journal, "w") as f:
        f.write("petit")
    outils._tourner(journal)
    verifier("petit journal gardé", os.path.exists(journal), True)

# Hors Windows (Pi), l'installation d'openEMS est refusée : pas de bouton.
windows, outils.WINDOWS = outils.WINDOWS, False
verifier("pas de bouton openEMS hors Windows", outils.openems_pret(outils.PAR_ID["web_antenna"]), None)
outils.WINDOWS = windows

# openEMS : une archive dont l'empreinte n'est pas la bonne n'est pas extraite.
import io            # noqa: E402
import zipfile       # noqa: E402
fausse = io.BytesIO()
with zipfile.ZipFile(fausse, "w") as z:
    z.writestr("openEMS/CSXCAD.dll", b"pas la vraie")


class Reponse(io.BytesIO):
    headers = {}

    def __enter__(self):
        return self

    def __exit__(self, *_):
        return False


vraie_urlopen, outils.urllib.request.urlopen = outils.urllib.request.urlopen, lambda *a, **k: Reponse(fausse.getvalue())
with tempfile.TemporaryDirectory() as tmp:
    cible = os.path.join(tmp, "openEMS")
    verifier("archive refusée", outils._telecharger_openems(cible, lambda t: None), False)
    verifier("rien d'extrait ni de .part laissé", sorted(os.listdir(tmp)), [])
outils.urllib.request.urlopen = vraie_urlopen
print("OK")
