"""L'aide des commandes de pilotage, pour la page : WEB_CAO/commun/pilote-aide.json.

Un script qui pilote une page depuis sa console (sans le MCP branché) ne voit pas les
descriptions ni les schémas d'arguments des outils : `Pilote.aide("cao_schema_dessiner")`
(WEB_CAO/commun/pilote-cao.js) les lit dans ce fichier. Il est TIRÉ des définitions du MCP
(OUTILS_MCP), une seule source : relancer ce script après avoir changé un outil cao_* ou lib_*.
lanceur/banc_mcp.py échoue tant que le fichier n'est pas à jour, et la CI de web_suite aussi
(sur son clone de WEB_CAO) : pousser WEB_CAO avant web_suite.

    python3 lanceur/aide_pilote.py
"""
import json
import os
import sys

ICI = os.path.dirname(os.path.abspath(__file__))
FICHIER = os.path.join(ICI, "..", "WEB_CAO", "commun", "pilote-aide.json")


def contenu():
    sys.path.insert(0, ICI)
    import mcp_web_suite
    outils = {n: {"description": o["description"], "arguments": o["inputSchema"]}
              for n, o in sorted(mcp_web_suite.OUTILS_MCP.items()) if n.startswith(("cao_", "lib_"))}
    return json.dumps({"source": "web_suite/lanceur/aide_pilote.py (définitions du MCP)", "outils": outils},
                      ensure_ascii=False, indent=1, sort_keys=True) + "\n"


if __name__ == "__main__":
    with open(FICHIER, "w", encoding="utf-8") as f:
        f.write(contenu())
    print("écrit :", os.path.normpath(FICHIER))
