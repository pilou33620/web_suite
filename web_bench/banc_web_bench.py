"""Banc de web_bench lui-même, sans les vrais outils.

    python web_bench/banc_web_bench.py

Un faux WEB_CAO et un faux WEB_3D (de petits serveurs qui acceptent les options de web_bench)
sont créés dans une fausse racine de web_suite ; le faux WEB_CAO a deux bancs, l'un réussit,
l'autre échoue. On vérifie le relais (la page reçoit la sonde, un POST passe avec son corps et
ses en-têtes), le passage d'un outil à l'autre, les gardes (Host étranger, POST de l'API sans
en-tête), la découverte et le lancement des bancs, et l'enregistrement d'un rapport.
Affiche OK si tout passe.
"""
import http.client
import json
import os
import shutil
import sys
import tempfile
import threading
import time

ICI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, ICI)
import web_bench  # noqa: E402

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


def main():
    temp = tempfile.mkdtemp(prefix="banc_web_bench_")
    rapports = web_bench.DOSSIER_RAPPORTS
    web_bench.DOSSIER_RAPPORTS = os.path.join(temp, "rapports")
    try:
        cao = os.path.join(temp, "WEB_CAO")
        os.makedirs(os.path.join(cao, "test"))
        os.makedirs(os.path.join(cao, "graphify-out"))
        with open(os.path.join(cao, "web_CAO.py"), "w") as f:
            f.write(FAUX_CAO)
        os.makedirs(os.path.join(temp, "WEB_3D"))
        with open(os.path.join(temp, "WEB_3D", "web_3D.py"), "w") as f:
            f.write(FAUX_CAO)
        with open(os.path.join(temp, "WEB_3D", "conception.html"), "w") as f:
            f.write("<html></html>")
        # la vraie LIB de WEB_CAO, que le banc copie sans y toucher
        os.makedirs(os.path.join(temp, "PROJETS", "LIB_CAO"))
        with open(os.path.join(temp, "PROJETS", "LIB_CAO", "LIB_composants.csv"), "w") as f:
            f.write("ref;valeur\n")
        with open(os.path.join(cao, "index.html"), "w") as f:
            f.write("<html></html>")
        with open(os.path.join(cao, "test", "banc-bon.py"), "w") as f:
            f.write("print('OK')\n")
        with open(os.path.join(cao, "test", "banc-mauvais.py"), "w") as f:
            f.write("import sys\nprint('raté')\nsys.exit(3)\n")
        with open(os.path.join(cao, "graphify-out", "banc-ignore.py"), "w") as f:
            f.write("raise SystemExit(1)\n")

        outils = web_bench.trouver_outils(temp)
        assert sorted(outils) == ["3d", "cao"], outils
        outil = outils["cao"]
        outil.demarrer()
        banc = web_bench.Banc(outils, "cao")
        web_bench.Gestionnaire.banc = banc
        serveur = web_bench.Serveur(("127.0.0.1", 0), web_bench.Gestionnaire)
        port = serveur.server_address[1]
        threading.Thread(target=serveur.serve_forever, daemon=True).start()
        hote = {"Host": "127.0.0.1:%d" % port}

        # la page relayée porte la sonde en tête
        code, page = requete(port, "GET", "/index.html", entetes=hote)
        assert code == 200 and b'<head><script src="/__banc/sonde.js"></script><title>' in page, page
        assert b"WEB_CAO" in page, page

        # un POST passe avec son corps, son Host et l'en-tête maison
        code, rep = requete(port, "POST", "/api/projet", b'{"a":1}',
                            dict(hote, **{"X-WEB_CAO": "1", "Content-Type": "application/json"}))
        rep = json.loads(rep)
        assert code == 200 and rep["corps"] == '{"a":1}' and rep["entete"] == "1", rep
        assert rep["host"] == "127.0.0.1:%d" % port, rep
        # l'outil tourne sur des dossiers temporaires, pas sur ceux de l'utilisateur
        assert rep["lib"].startswith(outil.temp) and rep["projets"].startswith(outil.temp), rep
        assert os.path.isfile(os.path.join(rep["lib"], "LIB_composants.csv")), "LIB non copiée"

        # routes protégées : arrêtées par le relais, l'outil ne les reçoit jamais
        avant = len(outil.journal)
        code, rep = requete(port, "POST", "/api/github/envoyer", b"{}", dict(hote, **{"X-WEB_CAO": "1"}))
        assert code == 403 and b"web_bench" in rep, (code, rep)
        assert not any("github" in l for l in outil.journal[avant:] if l.startswith("requete")), outil.journal
        assert any("arrêtée par le banc" in l for l in outil.journal), outil.journal

        # gardes
        code, _ = requete(port, "GET", "/index.html", entetes={"Host": "piege.exemple:%d" % port})
        assert code == 403, code
        code, _ = requete(port, "POST", "/__banc/api/bancs/lancer", b"{}", hote)
        assert code == 403, code
        code, _ = requete(port, "GET", "/__banc/../web_bench.py", entetes=hote)
        assert code == 404, code

        # page Banc et sonde servies
        for f in ("/__banc/", "/__banc/banc.js", "/__banc/sonde.js", "/__banc/banc.css"):
            code, _ = requete(port, "GET", f, entetes=hote)
            assert code == 200, f

        # bancs : graphify-out ignoré, les deux autres lancés
        code, e = requete(port, "GET", "/__banc/api/bancs", entetes=hote)
        ids = [b["id"] for b in json.loads(e)["bancs"]]
        assert ids == ["test/banc-bon.py", "test/banc-mauvais.py"], ids
        code, _ = requete(port, "POST", "/__banc/api/bancs/lancer", json.dumps({"ids": ids}).encode(),
                          dict(hote, **{"X-Web-Bench": "1"}))
        assert code == 200
        for _ in range(100):
            e = json.loads(requete(port, "GET", "/__banc/api/bancs", entetes=hote)[1])
            if not e["en_cours"]:
                break
            time.sleep(0.1)
        etats = {b["id"]: (b["etat"], b["code"]) for b in e["bancs"]}
        assert etats == {"test/banc-bon.py": ("ok", 0), "test/banc-mauvais.py": ("echec", 3)}, etats
        sortie = json.loads(requete(port, "GET", "/__banc/api/bancs/sortie?id=test/banc-mauvais.py", entetes=hote)[1])
        assert "raté" in sortie["sortie"], sortie

        # journal de l'outil
        j = json.loads(requete(port, "GET", "/__banc/api/journal?depuis=0", entetes=hote)[1])
        assert any("requete" in l for l in j["lignes"]), j

        # rapport enregistré puis relu
        r = {"date": "2026-11-09T10:00:00Z", "outil": "WEB_CAO", "bilan": {"essais": 1}, "resultats": []}
        code, rep = requete(port, "POST", "/__banc/api/rapports", json.dumps(r).encode(), dict(hote, **{"X-Web-Bench": "1"}))
        nom = json.loads(rep)["nom"]
        liste = json.loads(requete(port, "GET", "/__banc/api/rapports", entetes=hote)[1])["rapports"]
        assert [x["nom"] for x in liste] == [nom], liste
        relu = json.loads(requete(port, "GET", "/__banc/api/rapport?nom=" + nom, entetes=hote)[1])
        assert relu["bilan"] == {"essais": 1}, relu

        # passer à WEB_3D : il démarre à la demande, le relais le suit, ses pages sont listées
        code, _ = requete(port, "POST", "/__banc/api/outil/choisir", json.dumps({"id": "3d"}).encode(),
                          dict(hote, **{"X-Web-Bench": "1"}))
        assert code == 200 and outils["3d"].en_marche()
        code, page = requete(port, "GET", "/index.html", entetes=hote)
        assert b"WEB_3D" in page, page
        pages = json.loads(requete(port, "GET", "/__banc/api/pages", entetes=hote)[1])["pages"]
        assert [p["url"] for p in pages] == ["/conception.html"], pages
        code, _ = requete(port, "POST", "/__banc/api/outil/choisir", json.dumps({"id": "antenna"}).encode(),
                          dict(hote, **{"X-Web-Bench": "1"}))
        assert code == 404, code
        etat = json.loads(requete(port, "GET", "/__banc/api/etat", entetes=hote)[1])
        assert etat["actif"] == "3d" and [o["installe"] for o in etat["outils"]] == [True, False, True], etat

        serveur.shutdown()
        temp_outil = outil.temp
        banc.arreter()
        assert not os.path.exists(temp_outil), "dossiers temporaires de l'outil non effacés"
        print("OK")
    finally:
        web_bench.DOSSIER_RAPPORTS = rapports
        shutil.rmtree(temp, ignore_errors=True)


if __name__ == "__main__":
    main()
