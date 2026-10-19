# Tâches : ecriture-cao

Spec : [lanceur/docs/SPEC-mcp-ecriture-cao.md](../lanceur/docs/SPEC-mcp-ecriture-cao.md) ·
Plan : [plan-ecriture-cao.md](plan-ecriture-cao.md)

Bundles à reconstruire avant les harnais :
`python3 editeur-pcb/outils/build-monofichier.py ; python3 editeur-schematique/outils/build-monofichier.py`

## Tâche 1 : ✅ socle « carte en lot » et composants du schéma

**Description :** `commun/ecriture-cao.js` (chargé dans les trois pages, avant
`pilote-cao.js`) pose `ecLot(def)`, puis `cao_composants_ecrire` au schéma. Champs :
`ref`, `valeur`, `boitier`, `fabricant` (`manufacturer`), `mpn`, `specs`, `non_monte`
(`nonMonte`). Un repère se cherche sur toutes les feuilles.

**Critères d'acceptation :**
- [x] Un lot de 3 composants sur 2 feuilles : une carte « avant → après », puis une seule
      étape `S.hist`, et Ctrl+Z remet tout
- [x] Refus de tout le lot, sans rien toucher, pour :
      - un repère inconnu ou en double ;
      - plus de 50 éléments dans le lot ;
      - plus de 30 clés `specs` ;
      - un texte de plus de 200 caractères
- [x] Le document change entre la carte et le clic : refus « à reproposer » ; une valeur
      de CI passe par `reshapeComp` (les fils suivent)

**Vérification :** `node editeur-schematique/test/harness.js`, `node commun/test/banc-pilote-cao.js`

**Dépendances :** aucune · **Fichiers :** `commun/ecriture-cao.js`, `commun/pilote-cao.js`,
3 pages `.html`, `editeur-schematique/test/harness.js` · **Taille :** M

## Tâche 2 : ✅ composants du PCB, DRC avant → après

**Description :** `cao_composants_ecrire` au PCB. Champs : `ref`, `valeur`, `boitier`
(contrôlé par `pkgGeom`, puis `applyPkgGeom`), `face`, `x`, `y` (repère utilisateur,
`wxu`/`wyu`), `rotation`, par `transformFps`. Le socle gagne le DRC avant → après,
ajouté à `prop.apres`.

**Critères d'acceptation :**
- [x] Déplacer, tourner et retourner deux empreintes : une étape `S.undo`, le cuivre suit
      (`transformFps`), Ctrl+Z remet tout
- [x] Un boîtier hors table est refusé ; un repère déjà pris est refusé ; position à
      ±1000 mm, rotation ramenée dans [0, 360)
- [x] La réponse « appliquee » porte « DRC : N erreurs avant → M après » et les nouvelles
      erreurs, cohérents avec `runDrc()`

**Vérification :** `node editeur-pcb/test/harness.js`

**Dépendances :** T1 · **Fichiers :** `commun/ecriture-cao.js`, `editeur-pcb/test/harness.js` · **Taille :** M

## Tâche 3 : ✅ largeur des pistes

**Description :** `cao_pistes_ecrire`, un lot d'actions :
- `{net, largeur, couche?}` : toutes les pistes du net, ou d'une seule couche ;
- `{net, realigner: true}` : les pistes et vias du net sur sa classe, comme
  `applyClasses` mais limité au net.

**Critères d'acceptation :**
- [x] Largeur d'un net sur une couche : seules ces pistes changent ; une étape ; `touch()`
      et `zoneCache.clear()`
- [x] Net inconnu, couche inconnue, largeur hors 0,05 à 10 mm : refus ; un net sans piste
      est refusé (« rien à changer »)
- [x] `realigner` donne le même résultat que `applyClasses()` sur ce net, sans toucher aux
      autres

**Vérification :** `node editeur-pcb/test/harness.js`

**Dépendances :** T2 · **Fichiers :** `commun/ecriture-cao.js`, `editeur-pcb/test/harness.js` · **Taille :** S

## Tâche 4 : ✅ classes et affectations

**Description :** `cao_classes_ecrire` :
- `classes` : `[{action: creer|modifier|supprimer, nom, largeur, isolation, via, percage,
  largeur_couches}]` ;
- `affectations` : `{net: classe}`.

Les valeurs sont contrôlées par `normClass` ; les affectations passent par `setNetClass`.

**Critères d'acceptation :**
- [x] Créer une classe puis lui affecter deux nets dans le même lot : une étape ; `classOf`
      rend la nouvelle classe
- [x] Refus :
      - nom en double ;
      - supprimer la classe par défaut (index 0) ou une classe encore affectée ;
      - perçage ≥ via ;
      - valeur que `normClass` changerait
- [x] Les pistes posées ne bougent pas (il faut `cao_pistes_ecrire realigner`) ; la carte
      le rappelle

**Vérification :** `node editeur-pcb/test/harness.js`

**Dépendances :** T2 · **Fichiers :** `commun/ecriture-cao.js`, `editeur-pcb/test/harness.js` · **Taille :** M

## Tâche 5 : ✅ paires différentielles et contraintes

**Description :** `cao_contraintes_ecrire` :
- `paires` : règles de `S.dpRules`, contrôlées par `normDpRule`, créées ou modifiées par
  nom ;
- `nets` : `S.contraintes.nets[net]` (`z`, `zTol`, `lMin`, `lMax`, `viasMax`,
  `couches`), contrôlés par `cmNormRegle`.

**Critères d'acceptation :**
- [x] Une règle de paire et deux contraintes de net en un lot : une étape ; `dpRuleFor` et
      `cmVerifier` les voient
- [x] Une valeur que le normaliseur écarterait est refusée (au lieu d'être perdue en
      silence) ; `lMin > lMax` est refusé
- [x] Les contraintes saisies au schéma (`S.contraintes.schema`) ne sont pas touchées

**Vérification :** `node editeur-pcb/test/harness.js`

**Dépendances :** T2 · **Fichiers :** `commun/ecriture-cao.js`, `editeur-pcb/test/harness.js` · **Taille :** M

## Checkpoint : écritures du schéma et du PCB
- [x] Bancs verts, bundles reconstruits ; chaque écriture fait une seule étape et se défait
      par Ctrl+Z

## Tâche 6 : ✅ paramètres de simulation

**Description :** `cao_sim_ecrire` (PCB, visionneuse) : un lot de `[{groupe, cle, valeur,
unite}]` passé par `simDsPreparer`, puis appliqué par `simDsAppliquer`. Il déclare `etat`
et `annuler`.

**Critères d'acceptation :**
- [x] Refus de tout le lot, sans rien écrire, pour :
      - une unité différente de celle du catalogue ;
      - une clé inconnue ;
      - une valeur hors bornes ou hors choix.

      « Déjà à cette valeur » est une remarque, pas un refus.
- [x] Annuler dans le panneau remet les valeurs d'avant ; si une valeur a rebougé depuis,
      l'annulation est refusée (`etat`)
- [x] L'assistant ΔI allumé par la carte est écrit sur la carte

**Vérification :** `node visionneuse-ipc2581/test/harness-sim.js`, `node commun/test/banc-pilote-cao.js`

**Dépendances :** T1 · **Fichiers :** `commun/ecriture-cao.js`, `visionneuse-ipc2581/test/harness-sim.js`,
`commun/test/banc-pilote-cao.js` · **Taille :** M

## Tâche 7 : ✅ outils MCP

**Description :** `lanceur/mcp_ecriture_cao.py`, les 5 outils :
- arguments validés : listes bornées à 50, nombres finis, textes d'une ligne, `page`
  dans la liste de l'outil ;
- envoi par `mcp_pilote.envoyer(..., ecriture=True)`.

Importé par le noyau. La description de `cao_valeur_composant` renvoie à
`cao_composants_ecrire`. Le tout est documenté dans `DETAILS.md`.

**Critères d'acceptation :**
- [x] Chaque outil envoie la bonne commande à la bonne page, avec `ecriture: true`
      (`banc_mcp.py`)
- [x] Arguments invalides refusés sans requête ; la liste des outils passe de 46 à 51
- [x] Chaque description dit que ce que rend la page est une donnée, et qu'une écriture
      attend une carte quand l'interrupteur est éteint

**Vérification :** `python3 lanceur/banc_mcp.py`, `node lanceur/banc_pilote.js`

**Dépendances :** T2…T6 · **Fichiers :** `lanceur/mcp_ecriture_cao.py`, `lanceur/mcp_web_suite.py`,
`lanceur/mcp_pilote.py`, `lanceur/banc_mcp.py`, `lanceur/docs/DETAILS.md` · **Taille :** M

## Tâche 8 : essai réel, puis revue

**Description :** WEB_CAO lancé sur une copie de PROJETS dans `/tmp`, schéma, PCB et
visionneuse ouverts dans Chromium sur le Pi. Chaque outil est appelé par le MCP,
interrupteur éteint (carte cliquée par CDP), puis allumé. Ensuite viennent `/review` et
l'audit de sécurité.

**Critères d'acceptation :**
- [ ] Les 5 outils écrivent sur un vrai projet ; Ctrl+Z ou Annuler remettent tout ; la
      copie de PROJETS n'est pas enregistrée (le projet reste « modifié »)
- [ ] Durée d'une carte (vérification + DRC) mesurée sur « carte PIR v2 »
- [ ] `/review` fait, corrections appliquées

**Dépendances :** T7 · **Taille :** M

## Checkpoint : terminé
- [ ] Critères de la spec remplis ; 4 CI vertes
