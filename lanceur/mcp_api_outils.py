"""Outils MCP du module api-outils : ce que les serveurs des outils savent faire sans la page.

Spec : lanceur/docs/SPEC-mcp-api-outils.md. Chaque outil valide ses arguments, puis passe
par appeler_outil_web() du noyau : il n'y a pas d'outil « appeler une route », la table
OUTILS_API est la liste blanche.
"""
import json
import urllib.parse

import mcp_web_suite as noyau

PLAFOND = 100 * 1024   # octets de JSON rendus à Claude ; au-delà, un résumé
DOCS_CAO = ("schema", "pcb", "ipc2581")
TYPES_LIB = ("pcb", "schematique", "simulation")
# En-tête d'un projet WEB_ANTENNA : absent, le serveur le remet à vide (projet.py:438-446)
# Chiffrer sans lancer : jamais les routes « …/lancer » (le Pi calcule des heures)
CHIFFRAGES = {"modele": "/api/openems", "balayage": "/api/openems/balayage",
              "tableau_s": "/api/openems/tableau-s", "script": "/api/openems/script"}
TETE_ANTENNA = ("titre", "outil", "source", "fichier", "dessin", "antenne", "affichage")
SOLVEURS = {"simulation": "/api/simulation", "simulation_dc": "/api/simulation-dc",
            "crosstalk": "/api/crosstalk", "simulation_rf": "/api/simulation-rf", "oeil": "/api/oeil",
            "analyse_carte": "/api/analyse-carte", "ipc2581": "/api/ipc2581"}


def exiger(args, **types):
    """Les arguments nommés, présents et du bon type, dans l'ordre demandé ; sinon une
    erreur qui nomme le fautif. Un seul argument : rendu seul."""
    valeurs = [facultatif(args, nom, type_, None) for nom, type_ in types.items()]
    for nom, v in zip(types, valeurs):
        if v is None:
            raise noyau.ErreurOutil("Argument « %s » manquant." % nom)
    return valeurs[0] if len(valeurs) == 1 else valeurs


def facultatif(args, nom, type_, defaut):
    v = args.get(nom)
    if v is None:
        return defaut
    if not isinstance(v, type_) or (type_ is int and isinstance(v, bool)):
        raise noyau.ErreurOutil("Argument « %s » invalide : %s attendu." % (nom, type_.__name__))
    return v


def resumer(valeur, conseil):
    """Au-delà du plafond, la forme du JSON au lieu du contenu : Claude redemande une partie."""
    brut = json.dumps(valeur, ensure_ascii=False)
    if len(brut.encode("utf-8")) <= PLAFOND:
        return valeur

    def decrire(v):
        if len(json.dumps(v, ensure_ascii=False)) <= 200:
            return v
        if isinstance(v, (dict, list, str)):
            return {"type": type(v).__name__, "longueur": len(v)}
        return {"type": type(v).__name__}

    resume = {"resume": True, "taille_octets": len(brut.encode("utf-8")), "conseil": conseil}
    if isinstance(valeur, dict):
        resume["cles_total"] = len(valeur)
        resume["forme"] = {k: decrire(valeur[k]) for k in list(valeur)[:200]}   # 200 x 200 o au plus
    else:
        resume["forme"] = decrire(valeur)
    resume["extrait"] = brut[:2000]
    return resume


def route(base, **params):
    """Route plus paramètres encodés ; les valeurs None sont omises."""
    params = {k: v for k, v in params.items() if v is not None}
    return base + ("?" + urllib.parse.urlencode(params) if params else "")


def cao_projets(_args):
    return resumer(noyau.appeler_outil_web("web_cao", "/api/projets"), "Trop de projets : demande-les par racine.")


def cao_projet_lire(args):
    chemin = exiger(args, chemin=str)
    racine = facultatif(args, "racine", str, None)
    return resumer(noyau.appeler_outil_web("web_cao", route("/api/projet", chemin=chemin, racine=racine)),
                   "Projet trop gros : lis ses documents avec cao_doc_lire.")


def cao_doc_lire(args):
    chemin, doc = exiger(args, chemin=str, doc=str)
    racine = facultatif(args, "racine", str, None)
    parties = lire_parties(args)
    if doc not in DOCS_CAO:
        raise noyau.ErreurOutil("Argument « doc » invalide : %s attendu." % ", ".join(DOCS_CAO))
    rep = noyau.appeler_outil_web("web_cao", route("/api/projet/doc", chemin=chemin, racine=racine, doc=doc))
    document = rep.get("document") if isinstance(rep, dict) else None
    if not isinstance(document, dict):
        return resumer(rep, "Document trop gros.")
    rep["document"] = garder_parties(document, parties, rep)
    return rep


def lire_parties(args):
    parties = facultatif(args, "parties", list, None)
    if parties is not None and not all(isinstance(p, str) for p in parties):
        raise noyau.ErreurOutil("Argument « parties » invalide : liste de noms de clés attendue.")
    return parties


def garder_parties(objet, parties, rep):
    """Les seules clés demandées (les absentes notées dans rep), puis le résumé si c'est encore gros."""
    if parties is not None:
        absentes = [p for p in parties if p not in objet]
        objet = {p: objet[p] for p in parties if p in objet}
        if absentes:
            rep["parties_absentes"] = absentes
    return resumer(objet, "Trop gros : redemande avec « parties » (clés de premier niveau listées dans « forme »).")


def cao_solveurs(_args):
    """Les sept GET d'état en un appel ; un solveur en panne n'empêche pas les autres."""
    etat = {}
    for nom, chemin in SOLVEURS.items():
        try:
            etat[nom] = noyau.appeler_outil_web("web_cao", chemin)
        except noyau.OutilArrete:
            raise                        # WEB_CAO arrêté : un seul message, pas sept
        except noyau.ErreurOutil as exc:
            etat[nom] = {"erreur": str(exc)}
    oeil = etat["oeil"]
    if isinstance(oeil, dict) and isinstance(oeil.get("gabarits"), list):
        # Les polygones des masques pèsent des dizaines de Ko : l'id et le nom suffisent pour choisir
        oeil["gabarits"] = [{"id": g.get("id"), "nom": g.get("nom")} for g in oeil["gabarits"] if isinstance(g, dict)]
    return resumer(etat, "")


def lire(id_outil, chemin, conseil="", **params):
    return resumer(noyau.appeler_outil_web(id_outil, route(chemin, **params)), conseil)


def lib_fichier_lire(args):
    type_, nom = exiger(args, type=str, nom=str)
    if type_ not in TYPES_LIB:
        raise noyau.ErreurOutil("Argument « type » invalide : %s attendu." % ", ".join(TYPES_LIB))
    return lire("web_cao", "/api/lib/fichier", "Fichier trop gros.", type=type_, nom=nom)


def lib_composants(args):
    """Le catalogue entier est relu, puis filtré et paginé ici : WEB_CAO ne sait pas le faire."""
    filtre = facultatif(args, "filtre", str, "").lower()
    colonnes = facultatif(args, "colonnes", list, None)
    debut = facultatif(args, "debut", int, 0)
    nombre = facultatif(args, "nombre", int, 50)
    if debut < 0:
        raise noyau.ErreurOutil("Argument « debut » invalide : 0 ou plus.")
    if not 1 <= nombre <= 500:
        raise noyau.ErreurOutil("Argument « nombre » invalide : de 1 à 500.")
    if colonnes is not None and not all(isinstance(c, str) for c in colonnes):
        raise noyau.ErreurOutil("Argument « colonnes » invalide : liste de noms de colonnes attendue.")
    # shortcut: catalogue entier relu à chaque appel ; mettre en cache si c'est lent sur le Pi
    rep = noyau.appeler_outil_web("web_cao", "/api/lib/composants")
    lignes = [c for c in rep.get("composants") or [] if isinstance(c, dict)]
    if filtre:
        lignes = [c for c in lignes if any(filtre in str(v).lower() for k, v in c.items() if k != "_id")]
    page = lignes[debut:debut + nombre]
    if colonnes is not None:
        page = [{k: v for k, v in c.items() if k in colonnes or k == "_id"} for c in page]
    return resumer({"colonnes": rep.get("colonnes"), "total": rep.get("total"), "trouves": len(lignes),
                    "debut": debut, "composants": page},
                   "Page trop grosse : baisse « nombre » ou choisis des « colonnes ».")


def deja_la(nom, existants):
    """Le nom figure-t-il déjà parmi les existants ? Sans casse : sous Windows, R0603 et
    r0603 sont le même fichier ; mieux vaut refuser un nom de trop qu'écraser."""
    return nom.lower() in {str(e).lower() for e in existants}


def cao_doc_ecrire(args):
    chemin, doc, document, modifie = exiger(args, chemin=str, doc=str, document=dict, modifie=int)
    racine = facultatif(args, "racine", str, None)
    if doc not in DOCS_CAO:
        raise noyau.ErreurOutil("Argument « doc » invalide : %s attendu." % ", ".join(DOCS_CAO))
    # « modifie » obligatoire : sans lui, WEB_CAO écrase sans contrôle de conflit
    try:
        return noyau.appeler_outil_web("web_cao", route("/api/projet/doc", chemin=chemin, racine=racine, doc=doc,
                                                         modifie=modifie), document, methode="PUT")
    except noyau.ErreurOutil as exc:
        if exc.code == 409:
            raise noyau.ErreurOutil("Le fichier a changé depuis ta lecture (la page l'a peut-être "
                                    "enregistré) : relis-le avec cao_doc_lire avant de réécrire.")
        raise


def cao_profil_ecrire(args):
    nom, profil = exiger(args, nom=str, profil=dict)
    nom = " ".join(nom.split())   # comme nom_profil() de WEB_CAO
    existants = noyau.appeler_outil_web("web_cao", "/api/profils").get("profils") or []
    if deja_la(nom, [p.get("nom") for p in existants if isinstance(p, dict)]):
        raise noyau.ErreurOutil("Le profil « %s » existe déjà : le MCP n'écrase pas. Choisis un autre nom." % nom)
    return noyau.appeler_outil_web("web_cao", route("/api/profil", nom=nom), profil, methode="PUT")


def lib_fichier_ecrire(args):
    type_, nom = exiger(args, type=str, nom=str)
    if type_ not in TYPES_LIB:
        raise noyau.ErreurOutil("Argument « type » invalide : %s attendu." % ", ".join(TYPES_LIB))
    data, contenu = args.get("data"), facultatif(args, "contenu", str, None)
    if (data is None) == (contenu is None) or not isinstance(data, (dict, list, type(None))):
        raise noyau.ErreurOutil("Donne soit « data » (objet ou liste JSON), soit « contenu » (texte), pas les deux.")
    base = nom.strip().replace("\\", "/").rsplit("/", 1)[-1]   # WEB_CAO ne garde que le nom de base
    existants = noyau.appeler_outil_web("web_cao", "/api/lib/fichiers").get(type_) or []
    if deja_la(base, existants):
        raise noyau.ErreurOutil("« %s » existe déjà dans la LIB (%s) : le MCP n'écrase pas." % (base, type_))
    corps = {"type": type_, "nom": nom}
    corps.update({"data": data} if data is not None else {"contenu": contenu})
    return noyau.appeler_outil_web("web_cao", "/api/lib/fichier", corps)


def pcbparts_appeler(args):
    nom = exiger(args, name=str)
    arguments = facultatif(args, "arguments", dict, {})
    # La liste blanche est celle de WEB_CAO (passerelle_mcp.ALLOWED_TOOLS), relue à chaque appel
    permis = noyau.appeler_outil_web("web_cao", "/api/tools").get("allowed") or []
    if nom not in permis:
        raise noyau.ErreurOutil("Outil pcbparts non permis : %s. Permis : %s." % (nom, ", ".join(map(str, permis))))
    return resumer(noyau.appeler_outil_web("web_cao", "/api/tool", {"name": nom, "arguments": arguments}),
                   "Réponse pcbparts trop grosse : affine la recherche.")


def antenna_ouvert():
    """Nom du projet ouvert dans WEB_ANTENNA ; refus s'il n'y en a pas."""
    ouvert = noyau.appeler_outil_web("web_antenna", "/api/projet").get("ouvert")
    if not isinstance(ouvert, dict) or not ouvert.get("nom"):
        raise noyau.ErreurOutil("Aucun projet ouvert dans WEB_ANTENNA : ouvre-le depuis la page. "
                                "Le MCP ne change pas le projet courant.")
    return ouvert["nom"]


def antenna_charge(nom):
    # Rouvrir le projet déjà ouvert ne change rien côté serveur (même nom, même dossier)
    return noyau.appeler_outil_web("web_antenna", route("/api/projet/ouvrir", nom=nom))


def antenna_projet_lire(args):
    parties = lire_parties(args)
    notes = {}
    return dict(garder_parties(antenna_charge(antenna_ouvert()), parties, notes), **notes)


def antenna_projet_enregistrer(args):
    projet = dict(exiger(args, projet=dict))
    nuls = sorted(k for k, v in projet.items() if v is None)
    if nuls:
        # null supprime carte.json ou resultats.json côté serveur : jamais par Claude
        raise noyau.ErreurOutil("Valeur null refusée pour : %s. Omets la clé pour la laisser inchangée." % ", ".join(nuls))
    nom = antenna_ouvert()
    if projet.setdefault("nom", nom) != nom:
        raise noyau.ErreurOutil("Seul le projet ouvert (« %s ») peut être enregistré par le MCP." % nom)
    absents = [k for k in TETE_ANTENNA if k not in projet]
    if absents:
        actuel = antenna_charge(nom)
        projet.update((k, actuel[k]) for k in absents if k in actuel)
    return noyau.appeler_outil_web("web_antenna", "/api/projet/enregistrer", projet)


def antenna_openems_chiffrer(args):
    quoi, document = exiger(args, quoi=str, document=dict)
    if quoi not in CHIFFRAGES:
        raise noyau.ErreurOutil("Argument « quoi » invalide : %s attendu. Le MCP ne lance pas de calcul."
                                % ", ".join(CHIFFRAGES))
    return resumer(noyau.appeler_outil_web("web_antenna", CHIFFRAGES[quoi], document), "Réponse trop grosse.")


def antenna_openems_suivre(args):
    id_ = exiger(args, id=str)
    depuis = facultatif(args, "depuis", int, None)
    return lire("web_antenna", "/api/openems/journal", "Journal trop gros : passe « depuis ».", id=id_, depuis=depuis)


CHEMIN = {"type": "string", "description": "Chemin du projet, tel que rendu par cao_projets."}
RACINE = {"type": "string", "description": "Racine du projet (champ « racine » de cao_projets) ; facultatif."}

OUTILS_API = {
    "cao_projets": {
        "description": "Liste les projets WEB_CAO (nom, chemin, racine, date). Point de départ avant "
                       "de lire un projet.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": cao_projets,
    },
    "cao_projet_lire": {
        "description": "Ouvre un projet WEB_CAO en lecture : son projet.cao.json et l'état de ses "
                       "documents (schéma, PCB, IPC-2581 : présent, date de modification).",
        "inputSchema": {"type": "object", "properties": {"chemin": CHEMIN, "racine": RACINE},
                        "required": ["chemin"]},
        "fn": cao_projet_lire,
    },
    "cao_doc_lire": {
        "description": "Lit un document d'un projet WEB_CAO (schema, pcb ou ipc2581). Rend aussi "
                       "« modifie », à renvoyer pour écrire. Au-delà de 100 Ko, rend un résumé : "
                       "redemande alors seulement les clés utiles avec « parties ».",
        "inputSchema": {"type": "object", "properties": {
            "chemin": CHEMIN, "racine": RACINE,
            "doc": {"type": "string", "enum": list(DOCS_CAO)},
            "parties": {"type": "array", "items": {"type": "string"},
                        "description": "Clés de premier niveau du document à rendre ; facultatif."}},
            "required": ["chemin", "doc"]},
        "fn": cao_doc_lire,
    },
    "cao_solveurs": {
        "description": "État des solveurs WEB_CAO (EM, DC, crosstalk, RF, œil avec ses gabarits, "
                       "analyse de carte, import IPC-2581) : disponibles ou non, version, conseil. "
                       "Lancer un calcul se fait depuis la page, pas d'ici.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": cao_solveurs,
    },
    "cao_profils": {
        "description": "Liste les profils WEB_CAO (empilages et règles de fabricant enregistrés).",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda _a: lire("web_cao", "/api/profils"),
    },
    "cao_profil_lire": {
        "description": "Lit un profil WEB_CAO par son nom (rendu par cao_profils).",
        "inputSchema": {"type": "object", "properties": {"nom": {"type": "string"}}, "required": ["nom"]},
        "fn": lambda a: lire("web_cao", "/api/profil", nom=exiger(a, nom=str)),
    },
    "lib_composants": {
        "description": "Cherche dans le catalogue de composants de la LIB (LIB_composants.csv) : "
                       "« filtre » (texte, sans casse, sur toutes les colonnes), « colonnes » à "
                       "garder, pagination « debut »/« nombre » (50 par défaut, 500 au plus).",
        "inputSchema": {"type": "object", "properties": {
            "filtre": {"type": "string"},
            "colonnes": {"type": "array", "items": {"type": "string"}},
            "debut": {"type": "integer", "minimum": 0},
            "nombre": {"type": "integer", "minimum": 1, "maximum": 500}}},
        "fn": lib_composants,
    },
    "lib_fichiers": {
        "description": "Liste les fichiers de la LIB : empreintes PCB, symboles, modèles de simulation.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda _a: lire("web_cao", "/api/lib/fichiers"),
    },
    "lib_fichier_lire": {
        "description": "Lit un fichier de la LIB (empreinte, symbole ou modèle) par type et nom.",
        "inputSchema": {"type": "object", "properties": {
            "type": {"type": "string", "enum": list(TYPES_LIB)}, "nom": {"type": "string"}},
            "required": ["type", "nom"]},
        "fn": lib_fichier_lire,
    },
    "lib_parasites": {
        "description": "Parasites des condensateurs et selfs de la LIB (C/ESR/ESL, L/DCR), tirés des "
                       "modèles fabricant : utiles pour le découplage et le PDN.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda _a: lire("web_cao", "/api/lib/parasites"),
    },
    "web3d_projets": {
        "description": "Liste les fichiers 3D (STEP, STL…) rangés dans les projets WEB_3D.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda _a: lire("web_3d", "/api/projets"),
    },
    "cao_doc_ecrire": {
        "description": "Écrit un document d'un projet WEB_CAO (schema, pcb ou ipc2581). Exige le "
                       "« modifie » rendu par cao_doc_lire : si le fichier a changé depuis, refus. "
                       "Si la page a ce document ouvert, elle demandera avant d'écraser ta version.",
        "inputSchema": {"type": "object", "properties": {
            "chemin": CHEMIN, "racine": RACINE,
            "doc": {"type": "string", "enum": list(DOCS_CAO)},
            "document": {"type": "object", "description": "Le document entier."},
            "modifie": {"type": "integer", "description": "« modifie » rendu par cao_doc_lire."}},
            "required": ["chemin", "doc", "document", "modifie"]},
        "fn": cao_doc_ecrire,
    },
    "cao_profil_ecrire": {
        "description": "Crée un profil WEB_CAO (format cao-profil-1). Refusé si le nom existe déjà : "
                       "jamais d'écrasement.",
        "inputSchema": {"type": "object", "properties": {
            "nom": {"type": "string"}, "profil": {"type": "object"}}, "required": ["nom", "profil"]},
        "fn": cao_profil_ecrire,
    },
    "lib_fichier_ecrire": {
        "description": "Ajoute un fichier à la LIB (empreinte pcb, symbole schematique, modèle "
                       "simulation) : « data » pour du JSON, « contenu » pour du texte (.sub, .lib…). "
                       "Refusé si le fichier existe déjà : jamais d'écrasement.",
        "inputSchema": {"type": "object", "properties": {
            "type": {"type": "string", "enum": list(TYPES_LIB)}, "nom": {"type": "string"},
            "data": {"type": ["object", "array"]}, "contenu": {"type": "string"}},
            "required": ["type", "nom"]},
        "fn": lib_fichier_ecrire,
    },
    "pcbparts_outils": {
        "description": "Liste les outils pcbparts.dev (recherche JLCPCB, Mouser, Digi-Key, brochages, "
                       "règles de fabrication…) que WEB_CAO relaie, avec leurs schémas d'arguments. "
                       "Passe par Internet.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda _a: lire("web_cao", "/api/tools"),
    },
    "pcbparts_appeler": {
        "description": "Appelle un outil pcbparts.dev par le relais de WEB_CAO (seulement ceux de sa "
                       "liste « allowed », voir pcbparts_outils). Passe par Internet.",
        "inputSchema": {"type": "object", "properties": {
            "name": {"type": "string"}, "arguments": {"type": "object"}}, "required": ["name"]},
        "fn": pcbparts_appeler,
    },
    "antenna_projets": {
        "description": "État des projets WEB_ANTENNA : racine, liste des projets, projet ouvert.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda _a: lire("web_antenna", "/api/projet"),
    },
    "antenna_projet_lire": {
        "description": "Lit le projet ouvert dans WEB_ANTENNA (en-tête, dessin, antenne, carte, "
                       "résultats). Seulement celui-là : le MCP ne change pas le projet courant. "
                       "Au-delà de 100 Ko, résumé : redemande avec « parties ».",
        "inputSchema": {"type": "object", "properties": {
            "parties": {"type": "array", "items": {"type": "string"},
                        "description": "Clés de premier niveau à rendre ; facultatif."}}},
        "fn": antenna_projet_lire,
    },
    "antenna_projet_enregistrer": {
        "description": "Enregistre le projet ouvert de WEB_ANTENNA. Ne donne que ce qui change : "
                       "l'en-tête absent est repris du projet ouvert, une « carte » ou des "
                       "« resultats » absents restent inchangés. null refusé (il supprimerait).",
        "inputSchema": {"type": "object", "properties": {"projet": {"type": "object"}}, "required": ["projet"]},
        "fn": antenna_projet_enregistrer,
    },
    "antenna_calculs": {
        "description": "Liste les calculs openEMS rangés dans le projet ouvert de WEB_ANTENNA.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda _a: lire("web_antenna", "/api/projet/calculs"),
    },
    "antenna_openems_etat": {
        "description": "État d'openEMS dans WEB_ANTENNA : préparer, lancer, script, visionneuse "
                       "disponibles ou non, fils de calcul, solveur.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda _a: lire("web_antenna", "/api/openems"),
    },
    "antenna_openems_chiffrer": {
        "description": "Vérifie et chiffre un modèle d'antenne openEMS sans rien lancer : « modele » "
                       "(modèle complété, estimation, avis), « balayage » (points d'un balayage "
                       "paramétrique), « tableau_s » (matrice S complète), « script » (script Python "
                       "autonome). Lancer reste à faire depuis la page.",
        "inputSchema": {"type": "object", "properties": {
            "quoi": {"type": "string", "enum": list(CHIFFRAGES)},
            "document": {"type": "object", "description": "Document antenne (unite, empilage, cuivre, "
                                                          "ports, bande…), tel que dans le projet."}},
            "required": ["quoi", "document"]},
        "fn": antenna_openems_chiffrer,
    },
    "antenna_openems_suivre": {
        "description": "Suit un calcul openEMS lancé depuis la page : état (prepare, calcule, fini, "
                       "echoue, arrete), avancement, nouvelles lignes du journal depuis « depuis », "
                       "et le résultat quand il est fini.",
        "inputSchema": {"type": "object", "properties": {
            "id": {"type": "string", "description": "Identifiant de la tâche."},
            "depuis": {"type": "integer", "minimum": 0, "description": "Lignes déjà lues (« n »)."}},
            "required": ["id"]},
        "fn": antenna_openems_suivre,
    },
    "antenna_openems_champs": {
        "description": "Inventaire des champs (.vtr) d'un calcul openEMS fini : séries disponibles.",
        "inputSchema": {"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]},
        "fn": lambda a: lire("web_antenna", "/api/openems/champs", id=exiger(a, id=str)),
    },
}
