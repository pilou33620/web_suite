"""Claude depuis les pages : le panneau « Claude » d'un outil envoie une demande écrite,
le lanceur la passe à Claude Code, installé sur ce poste, et renvoie ses réponses à la page.

    python3 lanceur/claude_pages.py      vérification (banc d'essai, avec un faux claude)

Chaque outil lancé garde les demandes de ses pages (pilote.py : /api/pilote/demande). Un fil
par outil les prend (/api/pilote/demandes, long-poll en boucle locale) et lance pour chacune

    claude -p --output-format stream-json --tools Skill --strict-mcp-config
           --mcp-config <web_suite seul> --allowedTools mcp__web_suite --permission-mode dontAsk

dans le dépôt web_suite (son skill s'y charge). Claude n'a donc que les outils du MCP : ni
terminal, ni lecture ou écriture de fichiers du poste ; ses écritures dans la page arrivent en
cartes, comme d'ordinaire. Ses textes et les outils qu'il appelle repartent à la page
(/api/pilote/progres). La conversation continue d'une demande à l'autre (--resume), par outil
et par navigateur, jusqu'à « Nouvelle conversation ».
Spec : lanceur/docs/SPEC-claude-pages.md.
"""
import json
import os
import shutil
import subprocess
import sys
import threading
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp_web_suite as noyau   # noqa: E402  (ports des outils, appels en boucle locale)

ICI = os.path.dirname(os.path.abspath(__file__))
RACINE = os.path.dirname(ICI)             # le dépôt web_suite : Claude Code y charge le skill
OUTILS = ("web_cao", "web_antenna", "web_3d")
DUREE_MAX = 1800                          # s : au-delà, Claude est arrêté
ATTENTE_POLL = 40                         # s : un peu plus que le long-poll de pilote.py (25 s)
REPOS = 5                                 # s : outil arrêté, on réessaie
REPOS_ANCIEN = 60                         # s : outil sans /api/pilote/demandes (à mettre à jour)
MAX_MESSAGE = 20000                       # caractères : la limite de pilote.py

_sessions = {}                            # (outil, navigateur) -> session de Claude Code
_verrou = threading.Lock()


def trouver_claude():
    """Le claude de ce poste : le PATH, puis les emplacements de l'installeur (un lanceur
    démarré par le bureau n'a pas toujours ~/.local/bin dans son PATH)."""
    for chemin in (os.environ.get("WEB_SUITE_CLAUDE"), shutil.which("claude"),
                   os.path.expanduser("~/.local/bin/claude"), os.path.expanduser("~/.claude/local/claude")):
        if chemin and os.path.isfile(chemin) and os.access(chemin, os.X_OK):
            return chemin
    return None


def consigne(outil, d):
    return ("Tu es appelé depuis le panneau « Claude » d'une page de %s : page « %s », onglet %s, "
            "projet « %s », appareil %s. Agis avec les outils du MCP web_suite ; vise cet onglet "
            "(argument onglet) quand un outil le demande. Tes écritures dans la page arrivent en cartes "
            "que l'utilisateur valide. Ta réponse s'affiche dans ce panneau, petit : réponds en français, "
            "court, sans tableau." % (noyau.nom_outil(outil), d.get("page", ""), d.get("onglet", ""),
                                      d.get("projet", ""), d.get("appareil", "")))


def commande(claude, outil, d, session):
    mcp = {"mcpServers": {"web_suite": {"command": sys.executable, "args": [os.path.join(ICI, "mcp_web_suite.py")]}}}
    args = [claude, "-p", "--output-format", "stream-json", "--verbose",
            "--tools", "Skill", "--strict-mcp-config", "--mcp-config", json.dumps(mcp),
            "--allowedTools", "mcp__web_suite", "--permission-mode", "dontAsk",
            "--append-system-prompt", consigne(outil, d)]
    if session:
        args += ["--resume", session]
    return args


def messages(evenement):
    """Ce qu'un événement du flux stream-json montre à la page : ses textes et ses outils."""
    if evenement.get("type") != "assistant":
        return []
    rendu = []
    for bloc in (evenement.get("message") or {}).get("content") or []:
        if bloc.get("type") == "text" and (bloc.get("text") or "").strip():
            rendu.append({"genre": "texte", "texte": bloc["text"].strip()[:MAX_MESSAGE]})
        elif bloc.get("type") == "tool_use":
            nom = str(bloc.get("name") or "")
            rendu.append({"genre": "outil", "texte": nom.replace("mcp__web_suite__", "")[:200]})
    return rendu


def progres(outil, id_, message=None, etat=None):
    corps = {"id": id_}
    if message is not None:
        corps["message"] = message
    if etat is not None:
        corps["etat"] = etat
    try:
        noyau.appeler_outil_web(outil, "/api/pilote/progres", corps)
    except noyau.ErreurOutil as exc:
        noyau.journal("claude_pages : progrès perdu pour %s (%s)" % (outil, exc))


def traiter(outil, d):
    cle = (outil, d["navigateur"])
    claude = trouver_claude()
    if claude is None:
        progres(outil, d["id"], {"genre": "erreur", "texte": "Claude Code n'est pas installé sur ce poste (ou pas "
                                 "dans le PATH du lanceur) : installe-le et connecte-toi une fois avec « claude » "
                                 "dans un terminal, ou donne son chemin par WEB_SUITE_CLAUDE."}, "echouee")
        return
    with _verrou:
        if d.get("nouvelle"):
            _sessions.pop(cle, None)
        session = _sessions.get(cle)
    try:
        proc = subprocess.Popen(commande(claude, outil, d, session), cwd=RACINE, stdin=subprocess.PIPE,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding="utf-8",
                                errors="replace")
    except OSError as exc:
        progres(outil, d["id"], {"genre": "erreur", "texte": "Claude Code ne démarre pas : %s" % exc}, "echouee")
        return
    minuteur = threading.Timer(DUREE_MAX, proc.kill)
    minuteur.start()
    erreurs = []
    lecteur = threading.Thread(target=lambda: erreurs.append(proc.stderr.read()), daemon=True)
    lecteur.start()
    fin, resultat = None, None
    try:
        proc.stdin.write(d["texte"])
        proc.stdin.close()
        for ligne in proc.stdout:
            try:
                e = json.loads(ligne)
            except ValueError:
                continue
            if e.get("session_id"):
                with _verrou:
                    _sessions[cle] = e["session_id"]
            for m in messages(e):
                progres(outil, d["id"], m)
            if e.get("type") == "result":
                resultat = e
        proc.wait()
        lecteur.join(5)
    finally:
        minuteur.cancel()
    if resultat is not None and not resultat.get("is_error"):
        fin = "finie"
    else:
        texte = (resultat or {}).get("result") or "".join(erreurs).strip()[-1500:] or \
            "Claude Code s'est arrêté sans réponse (code %s)." % proc.returncode
        if proc.returncode is not None and proc.returncode < 0:
            texte = "Arrêté après %d min sans finir." % (DUREE_MAX // 60)
        if session and resultat is None:
            with _verrou:
                _sessions.pop(cle, None)        # session introuvable ou abîmée : la suivante repart de zéro
        progres(outil, d["id"], {"genre": "erreur", "texte": str(texte)[:MAX_MESSAGE]})
        fin = "echouee"
    progres(outil, d["id"], etat=fin)


def boucle(outil, arret):
    while not arret.is_set():
        try:
            r = noyau.appeler_outil_web(outil, "/api/pilote/demandes", delai=ATTENTE_POLL)
        except noyau.OutilArrete:
            arret.wait(REPOS)
            continue
        except noyau.ErreurOutil as exc:
            arret.wait(REPOS_ANCIEN if exc.code == 404 else REPOS)
            continue
        d = r.get("demande") if isinstance(r, dict) else None
        if isinstance(d, dict) and isinstance(d.get("id"), str) and isinstance(d.get("texte"), str):
            threading.Thread(target=traiter, args=(outil, d), daemon=True).start()


def demarrer():
    """Un fil par outil, qui vit avec le lanceur. Rend l'événement qui les arrête."""
    arret = threading.Event()
    for outil in OUTILS:
        threading.Thread(target=boucle, args=(outil, arret), daemon=True, name="claude-" + outil).start()
    return arret


if __name__ == "__main__":
    # Vérification : un vrai pilote.py derrière un serveur de boucle locale, et un faux claude
    # (script Python) qui écrit le flux stream-json de Claude Code et relit ses arguments.
    import http.server
    import tempfile

    import pilote

    P = pilote.Pilote(attente_poll=0.5)

    class H(http.server.BaseHTTPRequestHandler):
        def log_message(self, *a):
            pass

        def do_GET(self):
            if not P.traiter(self):
                self.send_error(404)

        do_POST = do_GET

    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H)
    srv.daemon_threads = True
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    os.environ["WEB_SUITE_MCP_PORTS"] = json.dumps({"web_suite": 1, "web_cao": srv.server_address[1],
                                                    "web_antenna": 2, "web_3d": 3})
    noyau._cache_ports = (0.0, None, None)

    dossier = tempfile.mkdtemp()
    journal_args = os.path.join(dossier, "args.json")
    faux = os.path.join(dossier, "claude")
    with open(faux, "w", encoding="utf-8") as f:
        f.write("#!%s\n" % sys.executable + "\n".join([
            "import json, sys",
            "args = sys.argv[1:]",
            "texte = sys.stdin.read()",
            "json.dump({'args': args, 'texte': texte}, open(%r, 'w'))" % journal_args,
            "if texte == 'plante':",
            "    sys.stderr.write('Erreur : pas connecte'); sys.exit(1)",
            "sid = 'sess-2' if '--resume' in args else 'sess-1'",
            "for e in ({'type': 'system', 'subtype': 'init', 'session_id': sid},",
            "          {'type': 'assistant', 'message': {'content': [{'type': 'text', 'text': 'Je lis la carte.'},",
            "           {'type': 'tool_use', 'name': 'mcp__web_suite__cao_doc_lire', 'input': {}}]}},",
            "          {'type': 'user', 'message': {'content': [{'type': 'tool_result', 'content': 'secret'}]}},",
            "          {'type': 'assistant', 'message': {'content': [{'type': 'text', 'text': 'Fait : 3 nets.'}]}},",
            "          {'type': 'result', 'subtype': 'success', 'is_error': False, 'result': 'Fait.', 'session_id': sid}):",
            "    print(json.dumps(e), flush=True)"]))
    os.chmod(faux, 0o755)
    os.environ["WEB_SUITE_CLAUDE"] = faux
    arret = demarrer()

    def requete(methode, route, corps=None):
        import http.client
        c = http.client.HTTPConnection("127.0.0.1", srv.server_address[1], timeout=10)
        c.request(methode, route, body=None if corps is None else json.dumps(corps),
                  headers={"Content-Type": "application/json"})
        r = c.getresponse()
        rendu = json.loads(r.read().decode())
        c.close()
        return r.status, rendu

    def conversation(texte, nouvelle=False):
        requete("POST", "/api/pilote/battement", {"navigateur": "ipad", "onglets": [
            {"id": "O1", "page": "pcb", "projet": "carte-usb", "focus_il_y_a": 0}]})
        code, d = requete("POST", "/api/pilote/demande", {"navigateur": "ipad", "onglet": "O1", "texte": texte,
                                                          "nouvelle": nouvelle})
        assert code == 200, d
        vus, fin = [], time.monotonic() + 15
        while time.monotonic() < fin:
            conv = requete("GET", "/api/pilote/conversation?navigateur=ipad&id=%s&depuis=%d" % (d["id"], len(vus)))[1]
            vus += conv["messages"]
            if conv["etat"] in ("finie", "echouee"):
                return conv["etat"], vus, json.load(open(journal_args))
        raise AssertionError("pas de fin : %r" % vus)

    etat, vus, recu = conversation("route USB_DP")
    assert etat == "finie", (etat, vus)
    assert vus == [{"genre": "texte", "texte": "Je lis la carte."}, {"genre": "outil", "texte": "cao_doc_lire"},
                   {"genre": "texte", "texte": "Fait : 3 nets."}], vus            # le résultat d'outil ne sort pas
    a = recu["args"]
    assert recu["texte"] == "route USB_DP" and "--resume" not in a, recu
    # Claude n'a que le MCP web_suite : ni terminal ni fichiers, rien d'autre à valider en douce
    assert a[a.index("--tools") + 1] == "Skill" and a[a.index("--allowedTools") + 1] == "mcp__web_suite", a
    assert a[a.index("--permission-mode") + 1] == "dontAsk" and "--strict-mcp-config" in a, a
    assert list(json.loads(a[a.index("--mcp-config") + 1])["mcpServers"]) == ["web_suite"], a
    assert "onglet O1" in a[a.index("--append-system-prompt") + 1] and "carte-usb" in a[a.index("--append-system-prompt") + 1]
    # La demande suivante continue la conversation ; « Nouvelle conversation » repart de zéro
    assert "--resume" in conversation("et USB_DM ?")[2]["args"]
    assert conversation("et USB_DM ?")[2]["args"][-1] == "sess-2"
    assert "--resume" not in conversation("autre chose", nouvelle=True)[2]["args"]
    # Claude qui échoue (non connecté) : la page le lit
    etat, vus, _ = conversation("plante")
    assert etat == "echouee" and "pas connecte" in vus[-1]["texte"], vus
    # Claude absent du poste : la page le dit
    os.environ["WEB_SUITE_CLAUDE"] = os.path.join(dossier, "absent")
    vrai_which = shutil.which
    shutil.which = lambda *_: None
    vrai_home = os.environ.get("HOME")
    os.environ["HOME"] = dossier
    etat, vus, _ = conversation("bonjour")
    assert etat == "echouee" and "pas installé" in vus[-1]["texte"], vus
    shutil.which = vrai_which
    if vrai_home is not None:
        os.environ["HOME"] = vrai_home
    arret.set()
    srv.shutdown()
    print("OK")
