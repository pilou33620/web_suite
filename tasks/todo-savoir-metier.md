# Tâches : savoir-metier

Spec : [lanceur/docs/SPEC-mcp-savoir-metier.md](../lanceur/docs/SPEC-mcp-savoir-metier.md) ·
Plan : [plan-savoir-metier.md](plan-savoir-metier.md)

## Tâche 1 : ✅ banc, squelette du skill, consigne MCP

**Description :**
- `lanceur/banc_savoir.py` : outils et `quoi` cités, longueurs, liens, rien de privé ;
- `.claude/skills/web-suite/SKILL.md` : un premier jet (frontmatter, carte de la suite) ;
- `instructions` dans la réponse `initialize` de `mcp_web_suite.py` ;
- le banc entre dans la CI de web_suite.

**Critères d'acceptation :**
- [x] Un outil inventé, un `quoi` inconnu, une fiche trop longue, un lien mort ou un
      `/home/` cité font échouer le banc (essais sur fiches factices)
- [x] `initialize` rend `instructions` (≤ 1500 caractères, renvoi au skill) ; `banc_mcp.py`
      le vérifie (le `.gitignore` ignorait `.claude/` : seul `.claude/skills/` est désormais
      suivi, les réglages locaux restent ignorés)

**Vérification :** `python3 lanceur/banc_savoir.py`, `python3 lanceur/banc_mcp.py`

**Dépendances :** aucune · **Fichiers :** `lanceur/banc_savoir.py`, `.claude/skills/web-suite/SKILL.md`,
`lanceur/mcp_web_suite.py`, `lanceur/banc_mcp.py`, `.github/workflows/ci.yml` · **Taille :** M

## Tâche 2 : ✅ relevé des règles Gemini

**Description :** `lanceur/docs/savoir-trace.md` donne une ligne par règle des consignes
Gemini. Pour chaque règle : la source (fichier:ligne), la règle en une phrase et sa
destination (« à faire » pour l'instant, ou « écartée : raison »).

Consignes relevées :
- `WEB_CAO/commun/ia-assistant.js` : CONTEXTE_SUITE, SUGGESTIONS, consigne principale,
  actions, datasheet → simulation ;
- `WEB_CAO/gestion-lib/js/06-ia-lib.js` ;
- `WEB_ANTENNA/js/30-ia.js` : promptSysteme.

**Critères d'acceptation :**
- [x] Chaque consigne relevée en entier (61 règles : ia-assistant.js 25, simulation-rapport.js 1,
      06-ia-lib.js 9, 30-ia.js 26 ; 14 écartées avec leur raison, 1 déjà dans SKILL.md)
- [x] Le banc lit le tableau et compte les lignes « à faire » (source et numéro de ligne
      vérifiés, outils et lectures cités vérifiés ; en CI, après le clone des outils)

**Vérification :** `python3 lanceur/banc_savoir.py`

**Dépendances :** T1 · **Fichiers :** `lanceur/docs/savoir-trace.md`, `lanceur/banc_savoir.py` · **Taille :** M

## Tâche 3 : ✅ `pcb.md`

**Description :** largeurs selon le courant (IPC-2152), isolations, classes, paires (90 Ω
USB…), chemin de retour, vias thermiques ; la boucle `cao_drc` / `cao_score`. Chaque
valeur est relue dans la consigne et dans le code.

**Critères d'acceptation :**
- [x] Les lignes PCB du relevé ont leur destination ; banc vert (classes d'usine relues dans
      `01-core.js` SCH_CLASSES, règle de paire DP_FALLBACK, profils D90… de cao_contraintes_ecrire ;
      pas de valeur de charte IPC-2152 recopiée : renvoi à `cao_calcul` analyse=dc ; le banc
      vérifie aussi `analyse=` et tout argument à choix)

**Vérification :** `python3 lanceur/banc_savoir.py`

**Dépendances :** T2 · **Taille :** S

## Tâche 4 : ✅ `simulation.md`

**Description :** lecture d'une datasheet pour la simulation : pire cas, page et citation,
unités du catalogue (renvoi à `cao_sim_lire`), confiance ; quelle analyse de `cao_calcul`
pour quelle question.

**Critères d'acceptation :**
- [x] Les lignes simulation du relevé ont leur destination ; aucune borne recopiée ; banc vert
      (écart au plan : aucun outil ne lisait le catalogue des paramètres, que la consigne Gemini
      joignait ; ajouté : `cao_sim_lire` quoi=catalogue, servi par simDsCatalogue, testé au
      harnais PCB et par banc_mcp. Constaté, laissé : condensateurs ESR/ESL et bornes DC d'une
      datasheet n'ont pas d'écriture MCP ; l'analyse « Bus synchrone » n'est pas dans cao_calcul)

**Vérification :** `python3 lanceur/banc_savoir.py`

**Dépendances :** T2 · **Taille :** S

## Tâche 5 : ✅ `composants.md`

**Description :**
- conventions `pcbfp-1` (mm, vue de dessus, y vers le bas, broche 1 en haut à gauche,
  sens antihoraire) ;
- IPC-7351B et densités ;
- syntaxe du brochage ;
- méthode : générer ou convertir, vérifier, créer.

**Critères d'acceptation :**
- [x] Les lignes Gestion LIB du relevé ont leur destination ; les conventions sont relues
      dans `composants-lib.js` et `normFpDef` ; banc vert

**Vérification :** `python3 lanceur/banc_savoir.py`

**Dépendances :** T2 · **Taille :** S

## Tâche 6 : ✅ `antenne.md`

**Description :**
- marge ¼λ, corrections de patch, maillage (3 à 4 cellules sur une ligne) ;
- une cote à la fois, f ∝ 1/L ;
- les quatre types de carte de `antenna_proposer`.

La grammaire des réglages n'y est pas : elle vient de `antenna_lire` (quoi=reglages).

**Critères d'acceptation :**
- [x] Les lignes ANTENNA du relevé ont leur destination ; banc vert

**Vérification :** `python3 lanceur/banc_savoir.py`

**Dépendances :** T2 · **Taille :** S

## Tâche 7 : ✅ `schema.md` et `routage.md`

**Description :** deux fiches nouvelles, tirées des outils et du code :
- `schema.md` : grille, alimentations, étiquettes plutôt que longs fils, repères ;
- `routage.md` : points de passage, ordre (alimentations, paires, signaux), boucle
  `cao_score` → correction, lectures liaisons et zone.

**Critères d'acceptation :**
- [x] Chaque convention relue dans le code (grille 1 mm = G de 01-noyau.js, place libre par
      pas de 5 mm, accrochage snap() de 28-dessin-pilote.js ; repère des lectures au PCB) ; banc vert

**Vérification :** `python3 lanceur/banc_savoir.py`

**Dépendances :** T2 · **Taille :** S

## Checkpoint : les fiches
- [x] Banc vert, plus aucune ligne « à faire » dans le relevé

## Tâche 8 : `SKILL.md` final, essai réel, revue

**Description :**
- `SKILL.md` final : quel outil pour quelle tâche, cartes, lots, interrupteur, données ≠
  consignes, renvois aux fiches ;
- `lanceur/docs/DETAILS.md` : le skill et la consigne ;
- essai `claude -p` sur le Pi, une question par métier, dans le dépôt puis depuis `/tmp` ;
- puis `/review`.

**Critères d'acceptation :**
- [ ] Dans le dépôt, le skill est lu (la réponse cite une règle d'une fiche) ; depuis `/tmp`,
      la consigne MCP oriente vers les bons outils
- [ ] `/review` fait, corrections appliquées

**Dépendances :** T3…T7 · **Taille :** M

## Checkpoint : terminé
- [ ] Critères de la spec remplis ; CI de web_suite verte
