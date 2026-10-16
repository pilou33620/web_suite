# Spec : mcp-noyau

Module 1 de la [carte MCP web_suite](SPEC-mcp.md). Statut : **validée le 2026-10-16**.

## Objectif

Un serveur MCP stdio que Claude Code lance lui-même, et qui sert de socle à tous les autres
modules. À lui seul, il ne fait qu'une chose utile : dire quels outils de WEB·SUITE tournent.
Tout le reste (calculs, pilotage, composants, routage) s'y branche ensuite comme des outils
MCP supplémentaires, sans toucher au protocole.

Utilisateur : toi, depuis Claude Code, sur le poste qui fait tourner les outils (Pi, PC) ou
depuis une autre machine par SSH.

**Critères d'acceptation**

1. Sur le poste : `claude mcp add web_suite -- python3 <web_suite>/lanceur/mcp_web_suite.py`,
   puis `/mcp` dans Claude Code montre `web_suite` **connecté**.
2. Depuis une autre machine : `claude mcp add web_suite -- ssh <poste> python3 <web_suite>/lanceur/mcp_web_suite.py`
   donne le même résultat, sans aucun port ouvert en plus de SSH.
3. L'outil MCP `etat` rend, pour le lanceur WEB·SUITE, WEB_CAO, WEB_ANTENNA et WEB_3D :
   lancé ou non, port, adresse.
   Les ports viennent de `lanceur/outils.py` (`CATALOGUE`), jamais recopiés.
4. Un outil qui ne tourne pas donne un résultat d'erreur lisible (« WEB_CAO ne tourne pas :
   lance-le depuis WEB·SUITE »), jamais une exception ni une coupure du serveur.
5. Le noyau fournit aux autres modules **un seul point d'appel HTTP** vers les outils, limité à
   `127.0.0.1` et aux ports du catalogue, qui **refuse** `/api/github/*`, `/api/ia/*` et, partout, `/api/envoyer`, `/api/identite`,
   `/api/appairer` (le lanceur publie sur GitHub par `/api/envoyer`). Le lanceur est en
   **lecture seule** (GET) : ses POST installent, lancent, arrêtent ou publient.
6. Ligne JSON illisible : erreur JSON-RPC `-32700`, le serveur continue. Méthode inconnue :
   `-32601`. Outil MCP inconnu ou arguments invalides : résultat `isError`.
7. Rien d'autre que le protocole sur stdout ; les journaux vont sur stderr.

## Hors périmètre (modules suivants)

Les outils de calcul, de projet et de LIB (`api-outils`), le canal vers la page
(`canal-pilote`), tout ce qui écrit. Le démarrage des outils.

## Pile technique

- Python 3.8+, bibliothèque standard seulement : `json`, `sys`, `urllib.request`.
- MCP sur stdio : JSON-RPC 2.0, un message par ligne. Méthodes traitées : `initialize`,
  `notifications/initialized`, `ping`, `tools/list`, `tools/call`. Version de protocole :
  celle du client si elle est connue, sinon `2025-06-18` (la version déjà utilisée par
  `WEB_CAO/python/passerelle_mcp.py`).
- **Pas de jeton** : le MCP tourne sur le poste des outils (directement ou par SSH) et les
  appelle en boucle locale. Or `Appairage.ce_poste()` (`lanceur/appairage.py:223`) laisse
  déjà passer la boucle locale sans jeton. Le MCP ne lit donc jamais
  `~/.web_tools/jeton-reseau.txt`.

## Commandes

```
Banc :              python lanceur/banc_mcp.py
Lancer à la main :  python3 lanceur/mcp_web_suite.py        (attend du JSON-RPC sur stdin)
Ajouter (poste) :   claude mcp add web_suite -- python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py
Ajouter (distant) : claude mcp add web_suite -- ssh pilou@<poste> python3 /home/pilou/Documents/web_suite/lanceur/mcp_web_suite.py
Windows :           claude mcp add web_suite -- py -3 C:\...\web_suite\lanceur\mcp_web_suite.py
```

## Structure

```
lanceur/mcp_web_suite.py   → serveur MCP : boucle stdio, OUTILS_MCP, appeler_outil_web()
lanceur/banc_mcp.py        → banc : lance le serveur en sous-processus, faux outil HTTP
lanceur/docs/SPEC-mcp*.md  → carte et specs
.github/workflows/ci.yml   → une étape de plus : python lanceur/banc_mcp.py
README.md, lanceur/docs/DETAILS.md → section « Claude Code (MCP) »
```

Les modules suivants ajoutent leurs outils dans `OUTILS_MCP` (ou dans un fichier voisin
importé par lui), avec le même format.

## Style de code

Celui du lanceur : noms en français, docstrings courtes, commentaires sur le *pourquoi*.

```python
# Outils MCP : nom -> description, schéma d'entrée, fonction. Les modules suivants
# ajoutent leurs entrées ici ; le protocole ne change pas.
OUTILS_MCP = {
    "etat": {
        "description": "Quels outils de WEB·SUITE tournent sur ce poste, et sur quel port.",
        "inputSchema": {"type": "object", "properties": {}},
        "fn": lambda args: etat_outils(),
    },
}

ROUTES_INTERDITES = ("/api/github", "/api/ia")   # publier sur GitHub, clé d'IA : jamais par Claude


def appeler_outil_web(id_outil, route, corps=None):
    """Seule porte vers les outils : boucle locale, port du catalogue, routes filtrées."""
    if route.startswith(ROUTES_INTERDITES):
        raise ErreurOutil("Route refusée au MCP : " + route)
    ...
```

## Stratégie de test

`lanceur/banc_mcp.py`, sur le modèle de `banc_serveur.py` (fonction `verifier`, sortie
non nulle en cas d'échec), lancé par la CI :

- lance `mcp_web_suite.py` en sous-processus et lui parle en JSON-RPC sur stdin/stdout ;
- démarre un **faux outil** HTTP sur un port libre ; pour le banc, une variable
  d'environnement remplace les ports du catalogue (même idée que `WEB_TOOLS_JETON_FICHIER`) ;
- vérifie : `initialize` puis `tools/list` contient `etat` ; `etat` voit le faux outil lancé
  et les autres arrêtés ; une ligne illisible donne `-32700` et le serveur répond encore ;
  `/api/github/envoyer` et `/api/ia/cle` sont refusés **sans qu'aucune requête n'atteigne**
  le faux outil ; stdout ne contient que du JSON-RPC.

Vérification à la main, une fois : `/mcp` dans Claude Code, sur le Pi puis par SSH.

## Limites

- **Toujours** : bibliothèque standard seulement ; appels en boucle locale seulement ;
  passer par `appeler_outil_web()` ; journaux sur stderr ; banc vert avant chaque commit.
- **Demander d'abord** : ajouter une dépendance ou un port ; changer `outils.py` ou
  `appairage.py` ; changer la CI au-delà de l'étape du banc.
- **Jamais** : exposer `/api/github/*`, `/api/ia/*`, `/api/envoyer`, `/api/identite`,
  `/api/appairer`, ni un POST du lanceur ; lire ou afficher le jeton ; appeler
  une autre adresse que `127.0.0.1` ; écrire autre chose que du JSON-RPC sur stdout.

## Critères de réussite

- `python lanceur/banc_mcp.py` vert en local et en CI (3.12, comme les autres bancs) ;
  aucune syntaxe postérieure à Python 3.8.
- `/mcp` montre `web_suite` connecté, sur le Pi et par SSH depuis une autre machine.
- Demander à Claude « quels outils tournent ? » rend la bonne réponse, avec un outil
  arrêté et deux lancés.

## Questions tranchées (2026-10-16)

1. Ports du catalogue **seulement** : un outil lancé à la main sur un autre port n'est pas
   trouvé.
2. `etat` dit aussi si le **lanceur WEB·SUITE** tourne (port `PORT_DEFAUT` de
   `lanceur/web_suite.py`, lu et non recopié).
3. SSH **depuis Windows** est utilisé : vérifier qu'OpenSSH (client) est présent sur le PC,
   et documenter la commande `claude mcp add` côté Windows avec `ssh`.

Statut : **validée**.
