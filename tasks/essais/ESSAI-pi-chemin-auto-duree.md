# Essai : `cao_chemin` sur le Pi, durée d'un appel de 20 liaisons

Où : Pi, avec la page PCB ouverte **dans le Chromium du Pi** (écran branché ou bureau à
distance) · Outils : WEB_CAO lancé sur une copie de PROJETS, Claude Code avec le MCP ·
Durée : 30 min · Source : `tasks/todo-chemin-auto.md:25-28`

## Pourquoi

`cao_chemin` cherche ses chemins (A* sur grille) **dans la page** : le code est
`WEB_CAO/editeur-pcb/js/40-chemin-auto.js`. La durée dépend donc du navigateur qui affiche la
page. Pour mesurer le Pi, la page doit tourner dans le Chromium du Pi : ouverte sur l'iPad,
c'est l'iPad qu'on mesurerait.

Le budget est de **8 s par appel** pour toute la recherche (`SPEC-mcp-chemin-auto.md`,
décision 7 ; `CHEMIN.BUDGET_MS: 8000`, `40-chemin-auto.js:19`). Au-delà, chaque liaison
restante est refusée avec « temps épuisé (8 s pour l'appel) ».

Déjà fait le 21/10/2026 sous Linux, dans la vraie page (`A-FAIRE.md:20`) : exemple 2,
20 liaisons en **0,5 s**, 19 trouvées, 19 posées par `cao_router`. Le routeur, en finissant
la route U3.1 → U3.3 (+5V) que `cao_chemin` fait contourner U3.2, coupe en diagonale de
(13,73 ; 9,93) à (12,90 ; 10,75) et entre de **0,13 mm** dans la pastille U3.2 (GND). La carte
l'a dit : « DRC : 0 erreur avant → 1 après » (`WEB_CAO/A-FAIRE.md:136-140`). L'essai
note si ça se reproduit sur le Pi. Le routeur (`11-pns-*` à `13-pns-shove.js`) ne se change
pas sans ton accord.

## Avant de commencer

1. Mets à jour :
   ```
   cd ~/Documents/web_suite
   git pull --ff-only
   git -C WEB_CAO pull --ff-only
   git -C WEB_CAO merge-base --is-ancestor 6c50425 HEAD && echo "via à 10 mm : ok"
   ```
   → attendu : `via à 10 mm : ok` (le coût de via revu le 21/10/2026).
2. Dans WEB·SUITE, arrête WEB_CAO s'il tourne (**■ Arrêter**).
3. Copie PROJETS, note son empreinte, puis lance WEB_CAO sur la copie, en local :
   ```
   rm -rf /tmp/essai-PROJETS && cp -a ~/Documents/web_suite/PROJETS /tmp/essai-PROJETS
   find ~/Documents/web_suite/PROJETS -type f -not -path '*/.git/*' -exec md5sum {} + | sort -k2 | md5sum
   cd ~/Documents/web_suite/WEB_CAO && nohup python3 web_CAO.py --port 8000 --local --sans-navigateur --sans-pause --sans-maj --projets /tmp/essai-PROJETS/CAO --lib /tmp/essai-PROJETS/LIB_CAO > /tmp/essai-cao.log 2>&1 &
   ```
4. Dans le Chromium du Pi : `http://127.0.0.1:8000/editeur-pcb/editeur-pcb.html`.
   **Fichier → Exemples…** → « Interface IoT & Bus Haute Vitesse — 4 couches » → **Ouvrir
   cet exemple**. C'est la carte de l'essai Linux. Garde cet onglet seul et au premier plan.
5. Dérouter toute la carte : **Ctrl+A**, puis **U** (Édition → Dérouter).
   → attendu : la barre d'état dit « N segments et M vias supprimés · … empreinte(s) laissée(s)
   en place ». Les zones de cuivre restent.
6. Claude Code sur le Pi, dans `~/Documents/web_suite`, avec le MCP
   (`ESSAI-pi-mcp-claude-code.md`).

## Étapes

1. À Claude : « Lis les liaisons de la carte PCB ouverte (cao_pcb_lire, liaisons) et écris les
   20 premières, au format des liaisons de cao_chemin ([{"depart": "U1.3", "arrivee": "C2.1"}, …],
   rien d'autre), dans /tmp/liaisons-20.json. N'appelle pas encore cao_chemin. »
   → attendu : le fichier existe, avec 20 liaisons. Note si U3.1 → U3.3 en fait partie.
2. Mesure un appel, dans un terminal du Pi (copie le bloc tel quel, il commence en colonne 0) :

```
cd ~/Documents/web_suite && python3 - <<'EOF'
import json, subprocess, time
liaisons = json.load(open("/tmp/liaisons-20.json"))
p = subprocess.Popen(["python3", "lanceur/mcp_web_suite.py"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
def envoyer(m):
    p.stdin.write(json.dumps(m) + "\n"); p.stdin.flush()
    return json.loads(p.stdout.readline())
envoyer({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "essai", "version": "0"}}})
t = time.time()
rep = envoyer({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "cao_chemin", "arguments": {"liaisons": liaisons}}})
duree = time.time() - t
p.stdin.close(); p.wait()
texte = rep["result"]["content"][0]["text"]
print("liaisons envoyées : %d · durée de l'appel : %.2f s" % (len(liaisons), duree))
try:
    r = json.loads(texte)
    print("pas : %s mm · trouvées : %d / %d" % (r["pas_mm"], sum(1 for x in r["resultats"] if x.get("trouve")), len(r["resultats"])))
    for x in r["resultats"]:
        if not x.get("trouve"):
            print("  liaison %d : %s" % (x["liaison"], x.get("raison")))
except (ValueError, KeyError):
    print(texte)
EOF
```
   → attendu : `durée de l'appel : … s` **sous 8 s** (0,5 s sous Linux) ; `pas : 0.25 mm` ;
   19 ou 20 trouvées ; aucune raison « temps épuisé ». Lance-le trois fois et garde les trois
   durées : `cao_chemin` n'écrit rien, on peut le rejouer.
3. À Claude : « Appelle cao_chemin sur les liaisons de /tmp/liaisons-20.json, puis donne ses
   routes à cao_router. »
   → attendu : dans le panneau **Claude** de la page, une carte de routage avec l'aperçu en
   pointillés et le DRC avant → après. Touche **Appliquer**.
4. Compte les routes posées, puis : « Lance le DRC (cao_drc) et dis-moi s'il y a une erreur
   d'isolation sur U3.2. »
   → attendu : Claude donne le nombre de routes posées (19 sous Linux) et les erreurs du DRC.
   Sous Linux : une seule, la route +5V qui entre de 0,13 mm dans U3.2.
5. Si l'erreur U3.2 revient : « Cadre U3.2 et capture la vue » (`cao_vue`, `pilote_capture`).
   → attendu : une image de la diagonale qui mord la pastille.
6. Ferme : `pkill -f "projets /tmp/essai-PROJETS"`, puis l'empreinte de PROJETS (la commande du
   point 3 d'« Avant de commencer »).
   → attendu : la même qu'avant.

## C'est réussi si

- [ ] Un appel de 20 liaisons dure moins de 8 s sur le Pi (trois durées notées)
- [ ] Aucune liaison refusée pour « temps épuisé »
- [ ] Les routes trouvées sont posées par `cao_router` (nombre noté, 19 sous Linux)
- [ ] L'entrée du routeur dans U3.2 est notée : reproduite (avec sa profondeur) ou non
- [ ] PROJETS a la même empreinte avant et après

## Si ça rate, note

- La sortie complète du script de l'étape 2 (les trois fois).
- Le contenu de `/tmp/liaisons-20.json`.
- La réponse de `cao_router` (texte du résultat d'outil) et une capture de sa carte.
- Le modèle du Pi et sa mémoire libre pendant l'appel : `cat /proc/device-tree/model ; free -m`.
- Le journal de WEB_CAO : `/tmp/essai-cao.log`.
- Si la page ne répond pas (« en_attente », ou rien) : la sortie de `pilote_onglets`. L'onglet
  doit être là, avec l'appareil « ce poste ».

## Après

En cas de réussite :
- `tasks/todo-chemin-auto.md:25-28` : `[~]` → `[x]`, avec la durée sur le Pi
- `A-FAIRE.md:11` : retirer « Reste : l'essai réel sur le Pi » ; `:21` : « Reste la durée sur
  le Pi. » → la durée mesurée ; `:36-37` : retirer « l'essai réel sur le Pi reste… »
- `WEB_CAO/A-FAIRE.md:141` et `:48` : retirer « la durée de `cao_chemin` sur le Pi »
- `WEB_CAO/A-FAIRE.md:136-140` (routeur dans une pastille) : **ne pas cocher**,
  ajouter ce que le Pi a montré (reproduit ou non) ; la correction se décide avec toi
