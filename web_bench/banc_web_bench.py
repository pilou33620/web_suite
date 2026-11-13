"""Banc de web_bench lui-même, sans les vrais outils.

    python web_bench/banc_web_bench.py

L'exécutable web_bench (Rust) est piloté de l'extérieur : un sous-processus, et des requêtes HTTP.
Un faux WEB_CAO et un faux WEB_3D (de petits serveurs qui acceptent les options de web_bench)
sont créés dans une fausse racine de web_suite ; le faux WEB_CAO a deux bancs, l'un réussit,
l'autre échoue. On vérifie le relais (la page reçoit la sonde, un POST passe avec son corps et
ses en-têtes), le passage d'un outil à l'autre, les gardes (Host étranger, POST de l'API sans
en-tête), la découverte et le lancement des bancs, l'enregistrement d'un rapport, et l'arrêt
(outils arrêtés, dossiers temporaires effacés).
Affiche OK si tout passe. Les fonctions internes ont leurs essais dans le code : cargo test -p web_bench
"""
import http.client
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time

ICI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(ICI), "lanceur"))
from banc_bin import binaire  # noqa: E402

FAUX_CAO = r'''
import argparse, http.server, json, os
ap = argparse.ArgumentParser()
ap.add_argument("--port", type=int); ap.add_argument("--projets"); ap.add_argument("--lib")
a, _ = ap.parse_known_args()
NOM = os.path.basename(os.path.dirname(os.path.abspath(__file__))).encode()
class G(http.server.BaseHTTPRequestHandler):
    def log_message(self, *x): print("requete", self.path, flush=True)
    def _r(self, code, corps, type_):
        self.send_response(code); self.send_header("Content-Type", type_)
        self.send_header("Content-Length", str(len(corps))); self.end_headers(); self.wfile.write(corps)
    def do_GET(self):
        if self.path.endswith(".html") or self.path == "/":
            return self._r(200, b"<!DOCTYPE html><html><head><title>t</title></head><body>" + NOM + b"</body></html>", "text/html")
        self._r(404, b"non", "text/plain")
    def do_POST(self):
        n = int(self.headers.get("Content-Length") or 0)
        rendu = {"corps": self.rfile.read(n).decode(), "host": self.headers.get("Host"),
                 "entete": self.headers.get("X-WEB_CAO"), "lib": a.lib, "projets": a.projets}
        self._r(200, json.dumps(rendu).encode(), "application/json")
http.server.ThreadingHTTPServer(("127.0.0.1", a.port), G).serve_forever()
'''


def requete(port, methode, chemin, corps=None, entetes=None):
    c = http.client.HTTPConnection("127.0.0.1", port, timeout=20)
    c.request(methode, chemin, body=corps, headers=entetes or {})
    r = c.getresponse()
    return r.status, r.read()


def ecrire(chemin, texte):
    os.makedirs(os.path.dirname(chemin), exist_ok=True)
    with open(chemin, "w", encoding="utf-8") as f:
        f.write(texte)


def main():
    exe = binaire("web_bench")
    temp = tempfile.mkdtemp(prefix="banc_web_bench_")
    proc = None
    try:
        # une fausse racine de web_suite : les pages du banc, et les rapports, y vivent aussi
        shutil.copytree(os.path.join(ICI, "page"), os.path.join(temp, "web_bench", "page"))
        cao = os.path.join(temp, "WEB_CAO")
        ecrire(os.path.join(cao, "web_CAO.py"), FAUX_CAO)
        ecrire(os.path.join(temp, "WEB_3D", "web_3D.py"), FAUX_CAO)
        ecrire(os.path.join(temp, "WEB_3D", "conception.html"), "<html></html>")
        # la vraie LIB de WEB_CAO, que le banc copie sans y toucher
        ecrire(os.path.join(temp, "PROJETS", "LIB_CAO", "LIB_composants.csv"), "ref;valeur\n")
        # ce que la CI fait en plus des banc-* (catalogue de web_bench) : un essai sous un autre
        # nom, sa préparation, et un banc qui exige la LIB privée (il voit la LIB par WEB_CAO_LIB)
        harness, prep, banc_lib = ("editeur-pcb/test/harness.js", "editeur-pcb/outils/build-monofichier.py",
                                   "python/test/banc-lib-routes.py")
        ecrire(os.path.join(cao, harness), "console.log('harness ok')\n")
        ecrire(os.path.join(cao, prep), "open('prepare.txt', 'w').write('fait')\nprint('préparé')\n")
        ecrire(os.path.join(cao, banc_lib),
               "import os, sys\nprint(os.environ.get('WEB_CAO_LIB'))\n"
               "sys.exit(0 if os.path.isfile(os.path.join(os.environ['WEB_CAO_LIB'], 'LIB_composants.csv')) else 4)\n")
        ecrire(os.path.join(cao, "index.html"), "<html></html>")
        ecrire(os.path.join(cao, "test", "banc-bon.py"), "print('OK')\n")
        ecrire(os.path.join(cao, "test", "banc-mauvais.py"), "import sys\nprint('raté')\nsys.exit(3)\n")
        ecrire(os.path.join(cao, "graphify-out", "banc-ignore.py"), "raise SystemExit(1)\n")

        # options refusées et aide, sans rien démarrer
        r = subprocess.run([exe, "--outil", "kicad"], capture_output=True)
        assert r.returncode == 2 and b"--outil" in r.stderr, r
        r = subprocess.run([exe, "--help"], capture_output=True)
        assert r.returncode == 0 and b"--racine" in r.stdout and b"--sans-navigateur" in r.stdout, r
        vide = os.path.join(temp, "vide")
        os.makedirs(vide)
        r = subprocess.run([exe, "--racine", vide], capture_output=True)
        assert r.returncode == 2 and "Aucun outil trouvé".encode() in r.stdout, r

        env = {k: v for k, v in os.environ.items() if k != "WEB_CAO_LIB"}
        env["WEB_SUITE_DEPOT"] = temp
        proc = subprocess.Popen([exe, "--racine", temp, "--outil", "cao", "--port", "0", "--sans-navigateur"],
                                env=env, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if os.name == "nt" else 0)
        debut = b""
        while b"Page Banc" not in debut:
            ligne = proc.stdout.readline()
            assert ligne, "web_bench s'est arrêté au démarrage :\n" + debut.decode("utf-8", "replace")
            debut += ligne
        debut = debut.decode("utf-8")
        assert "[*] Outils trouvés : WEB_CAO, WEB_3D" in debut, debut
        assert "[*] Démarrage de WEB_CAO (dossiers temporaires)..." in debut, debut
        assert "[*] Bancs trouvés : WEB_CAO 4, WEB_3D 0" in debut, debut
        port = int(re.search(r"Page Banc : http://127\.0\.0\.1:(\d+)/__banc/   \(Ctrl\+C pour arrêter\)", debut).group(1))
        hote = {"Host": "127.0.0.1:%d" % port}
        maison = dict(hote, **{"X-Web-Bench": "1"})

        def journal():
            return json.loads(requete(port, "GET", "/__banc/api/journal?depuis=0", entetes=hote)[1])["lignes"]

        def attendre_bancs():
            for _ in range(300):
                e = json.loads(requete(port, "GET", "/__banc/api/bancs", entetes=hote)[1])
                if not e["en_cours"]:
                    return e
                time.sleep(0.1)
            raise AssertionError("les bancs ne finissent pas")

        # la page relayée porte la sonde en tête
        code, page = requete(port, "GET", "/index.html", entetes=hote)
        assert code == 200 and b'<head><script src="/__banc/sonde.js"></script><title>' in page, page
        assert b"WEB_CAO" in page, page
        # ce qui n'est pas une page passe tel quel, avec son code
        code, rep = requete(port, "GET", "/absent.txt", entetes=hote)
        assert (code, rep) == (404, b"non"), (code, rep)

        # un POST passe avec son corps, son Host et l'en-tête maison
        code, rep = requete(port, "POST", "/api/projet", b'{"a":1}',
                            dict(hote, **{"X-WEB_CAO": "1", "Content-Type": "application/json"}))
        rep = json.loads(rep)
        assert code == 200 and rep["corps"] == '{"a":1}' and rep["entete"] == "1", rep
        assert rep["host"] == "127.0.0.1:%d" % port, rep
        # l'outil tourne sur des dossiers temporaires, pas sur ceux de l'utilisateur
        temp_outil = os.path.dirname(rep["lib"])
        assert os.path.basename(temp_outil).startswith("web_bench_cao_") and temp not in temp_outil, rep
        assert rep["projets"] == os.path.join(temp_outil, "projets"), rep
        assert os.path.isfile(os.path.join(rep["lib"], "LIB_composants.csv")), "LIB non copiée"

        # routes protégées : arrêtées par le relais, l'outil ne les reçoit jamais
        avant = len(journal())
        code, rep = requete(port, "POST", "/api/github/envoyer", b"{}", dict(hote, **{"X-WEB_CAO": "1"}))
        assert code == 403 and b"web_bench" in rep, (code, rep)
        assert json.loads(rep) == {"detail": "Arrêté par web_bench (route protégée pendant les essais)"}, rep
        time.sleep(0.3)
        lignes = journal()
        assert not any("github" in l for l in lignes[avant:] if l.startswith("requete")), lignes
        assert "[web_bench] requête arrêtée par le banc : POST /api/github/envoyer" in lignes, lignes

        # gardes
        code, _ = requete(port, "GET", "/index.html", entetes={"Host": "piege.exemple:%d" % port})
        assert code == 403, code
        code, rep = requete(port, "POST", "/__banc/api/bancs/lancer", b"{}", hote)
        assert code == 403 and json.loads(rep) == {"erreur": "en-tête X-Web-Bench manquant"}, (code, rep)
        code, _ = requete(port, "GET", "/__banc/../../WEB_CAO/web_CAO.py", entetes=hote)
        assert code == 404, code

        # page Banc et sonde servies
        for f in ("/__banc/", "/__banc/banc.js", "/__banc/sonde.js", "/__banc/banc.css"):
            code, _ = requete(port, "GET", f, entetes=hote)
            assert code == 200, f

        # bancs : graphify-out ignoré, l'essai harness.js ajouté, tous lancés
        code, e = requete(port, "GET", "/__banc/api/bancs", entetes=hote)
        ids = [b["id"] for b in json.loads(e)["bancs"]]
        assert ids == [harness, banc_lib, "test/banc-bon.py", "test/banc-mauvais.py"], ids
        code, rep = requete(port, "POST", "/__banc/api/bancs/lancer", json.dumps({"ids": ids}).encode(), maison)
        assert code == 200 and json.loads(rep) == {"ok": True}, (code, rep)
        e = attendre_bancs()
        etats = {b["id"]: (b["etat"], b["code"]) for b in e["bancs"]}
        node = shutil.which("node")
        attendu = {"test/banc-bon.py": ("ok", 0), banc_lib: ("ok", 0), "test/banc-mauvais.py": ("echec", 3),
                   harness: ("ok", 0) if node else ("ignore", None)}
        assert etats == attendu, etats
        if node:
            sortie = json.loads(requete(port, "GET", "/__banc/api/bancs/sortie?id=" + harness, entetes=hote)[1])["sortie"]
            assert "préparé" in sortie and "(préparation)" in sortie and "harness ok" in sortie, sortie
            assert os.path.isfile(os.path.join(cao, "prepare.txt")), "préparation non lancée"
        # sans la LIB privée : le banc qui l'exige est sauté, comme dans la CI
        shutil.rmtree(os.path.join(temp, "PROJETS"))
        requete(port, "POST", "/__banc/api/bancs/lancer", json.dumps({"ids": [banc_lib]}).encode(), maison)
        e = attendre_bancs()
        assert {b["id"]: b["etat"] for b in e["bancs"]}[banc_lib] == "ignore", e
        sortie = json.loads(requete(port, "GET", "/__banc/api/bancs/sortie?id=test/banc-mauvais.py", entetes=hote)[1])
        assert "raté" in sortie["sortie"], sortie

        # journal de l'outil
        assert any("requete" in l for l in journal())

        # rapport enregistré puis relu
        r = {"date": "2026-11-09T10:00:00Z", "outil": "WEB_CAO", "bilan": {"essais": 1}, "resultats": [],
             "markdown": "# Rapport\n"}
        code, rep = requete(port, "POST", "/__banc/api/rapports", json.dumps(r).encode(), maison)
        nom = json.loads(rep)["nom"]
        assert re.fullmatch(r"\d{4}-\d\d-\d\d_\d{6}_WEB_CAO\.json", nom), nom
        liste = json.loads(requete(port, "GET", "/__banc/api/rapports", entetes=hote)[1])["rapports"]
        assert [(x["nom"], x["md"], x["outil"]) for x in liste] == [(nom, True, "WEB_CAO")], liste
        relu = json.loads(requete(port, "GET", "/__banc/api/rapport?nom=" + nom, entetes=hote)[1])
        assert relu["bilan"] == {"essais": 1} and "markdown" not in relu, relu
        assert requete(port, "GET", "/__banc/api/rapport.md?nom=" + nom, entetes=hote) == (200, b"# Rapport\n")
        assert os.path.isfile(os.path.join(temp, "web_bench", "rapports", nom)), "rapport hors du dépôt"

        # passer à WEB_3D : il démarre à la demande, le relais le suit, ses pages sont listées
        code, _ = requete(port, "POST", "/__banc/api/outil/choisir", json.dumps({"id": "3d"}).encode(), maison)
        assert code == 200, code
        code, page = requete(port, "GET", "/index.html", entetes=hote)
        assert b"WEB_3D" in page, page
        pages = json.loads(requete(port, "GET", "/__banc/api/pages", entetes=hote)[1])["pages"]
        assert [p["url"] for p in pages] == ["/conception.html"], pages
        code, rep = requete(port, "POST", "/__banc/api/outil/choisir", json.dumps({"id": "antenna"}).encode(), maison)
        assert code == 404 and json.loads(rep) == {"erreur": "outil non installé : antenna"}, (code, rep)
        etat = json.loads(requete(port, "GET", "/__banc/api/etat", entetes=hote)[1])
        assert etat["actif"] == "3d" and [o["installe"] for o in etat["outils"]] == [True, False, True], etat
        assert [o["en_marche"] for o in etat["outils"]] == [True, False, True], etat
        ports = [o["port"] for o in etat["outils"] if o["port"]]
        code, _ = requete(port, "GET", "/__banc/api/inconnue", entetes=hote)
        assert code == 404, code

        # arrêt demandé de l'extérieur : les outils s'arrêtent, leurs dossiers temporaires partent
        proc.send_signal(signal.CTRL_BREAK_EVENT if os.name == "nt" else signal.SIGTERM)
        fin = proc.communicate(timeout=30)[0].decode("utf-8")
        assert proc.returncode == 0 and "[*] Arrêt." in fin, (proc.returncode, fin)
        assert not os.path.exists(temp_outil), "dossiers temporaires de l'outil non effacés"
        for p in ports:
            try:
                requete(p, "GET", "/")
                raise AssertionError("un outil tourne encore sur le port %d" % p)
            except OSError:
                pass
        print("OK")
    finally:
        if proc and proc.poll() is None:
            proc.kill()
        shutil.rmtree(temp, ignore_errors=True)


if __name__ == "__main__":
    main()
