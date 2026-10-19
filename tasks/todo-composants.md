# Tâches : composants

Spec : [lanceur/docs/SPEC-mcp-composants.md](../lanceur/docs/SPEC-mcp-composants.md) ·
Plan : [plan-composants.md](plan-composants.md)

## Tâche 1 : ✅ serveur, jamais d'écrasement

**Description :**
- `web_CAO.py` : `PUT /api/lib/fichier` avec `si_absent` (409 si le fichier existe) ;
  `POST /api/lib/composants` avec `ajouter: {ligne}`, qui relit le CSV, refuse un doublon
  de Part Name et ajoute la ligne au texte existant ;
- MCP : `lib_fichier_ecrire` passe à `si_absent`.

**Critères d'acceptation :**
- [x] `si_absent` : 409 sur un fichier existant, écrit sinon ; sans `si_absent`, le
      comportement d'aujourd'hui ne change pas
- [x] `ajouter` : doublon (casse ignorée) → 409 ; sinon la ligne est ajoutée, et toutes les
      autres lignes restent identiques octet pour octet, en utf-8 comme en latin1 ; fins
      de ligne gardées ; `Brochage` ajoutée à l'en-tête si absente ; colonne inconnue ou
      caractère hors encodage → 400
- [x] `banc-lib-routes.py` et `banc_mcp.py` verts (le MCP garde aussi le contrôle
      préalable, pour un WEB_CAO pas encore à jour qui ignorerait `si_absent`)

**Vérification :** `python3 python/test/banc-lib-routes.py`, `python3 lanceur/banc_mcp.py`

**Dépendances :** aucune · **Fichiers :** `web_CAO.py`, `python/test/banc-lib-routes.py`,
`lanceur/mcp_api_outils.py`, `lanceur/banc_mcp.py` · **Taille :** M

## Tâche 2 : ✅ contrôles (`commun/composants-lib.js`)

**Description :** `COMPOSANTS_LIB` contient :
- `verifierEmpreinte(def, nom)` : erreurs et remarques (contrôles de la spec, accord avec
  `normFpDef`) ;
- `verifierLigne(ligne, contexte)` : 39 colonnes + Brochage, champs obligatoires, Part
  Name unique, empreinte existante ou créée, symbole intégré connu ;
- `verifierBrochage(texte, empreinte, type)` : `brochageLire`, puis pastilles existantes
  et nombre de broches.

**Critères d'acceptation :**
- [x] Empreintes fautives refusées, chacune avec sa raison : perçage trop grand,
      couronne trop fine, pas de pastille 1, style inconnu, nom ≠ fichier, pastille
      minuscule ; un numéro en double est une remarque
- [x] Ligne : colonne manquante, Part Name déjà prise, empreinte absente, symbole inconnu
      → refus ; la liste des symboles égale les clés de `LIB` du schéma (banc)
- [x] Brochage incohérent (pastille absente de l'empreinte) refusé ; les 269 empreintes
      de la LIB passent les contrôles (description longue et `span` 0 admis)

**Vérification :** `node gestion-lib/test/banc-composants.js`

**Dépendances :** aucune · **Fichiers :** `commun/composants-lib.js`,
`gestion-lib/test/banc-composants.js` · **Taille :** M

## Tâche 3 : ✅ générateurs IPC-7351

**Description :** `genererEmpreinte(famille, params)` pour les familles :
- chip (code ou cotes) ;
- SOT-23-3/5/6, SOT-223 ;
- SOIC, TSSOP, MSOP (aile de mouette paramétrée) ;
- QFN, DFN (pastille thermique) ;
- DIP ;
- barrette 1×N ou 2×N.

Densité N par défaut. Chaque empreinte rendue passe `verifierEmpreinte`. Les générateurs
de Gestion LIB passent aux styles connus du PCB.

**Critères d'acceptation :**
- [x] Pour chaque famille, une empreinte de référence a les cotes calculées à la main au
      centième (0603, SOT-23-5, SOIC-8, QFN-16 + pastille thermique, DIP-8, 1×4 au pas de
      2,54)
- [x] 0603, SOT-23 et SOIC-8 sont à moins de 0,1 mm des empreintes de référence (écart au
      plan : le SOIC-8 et le SOT-23-5 de la LIB ne suivent pas l'IPC — pastilles décalées de
      0,15 à 0,23 mm —, on compare donc à KiCad (IPC-7351) pour eux, à la LIB pour le 0603)
- [x] Paramètres absents ou absurdes (pas ≤ 0, corps plus grand que l'empattement) :
      refus clair

**Vérification :** `node gestion-lib/test/banc-composants.js`, `node gestion-lib/test/banc-repere-pcb.js`

**Dépendances :** T2 · **Fichiers :** `commun/composants-lib.js`, `gestion-lib/js/05-editeur-lib.js`,
bancs · **Taille :** L

## Tâche 4 : ✅ conversion KiCad

**Description :** `convertirKicad(texte)` : un `kicad_mod` devient un `pcbfp-1`, avec ses
remarques, puis passe `verifierEmpreinte`. 200 Ko au plus.

**Critères d'acceptation :**
- [x] Un `kicad_mod` connu (SOT-23-5, avec pastilles roundrect et contour) donne les
      bonnes pastilles (numéros, positions, tailles, formes) et le corps
- [x] Pastille personnalisée, texte illisible ou trop gros : refus ; arcs, textes,
      modèles 3D : remarques

**Vérification :** `node gestion-lib/test/banc-composants.js`

**Dépendances :** T2 · **Fichiers :** `commun/composants-lib.js`, banc · **Taille :** M

## Checkpoint : le code métier
- [x] Bancs verts ; générateurs et conversion contrôlés par `verifierEmpreinte`

## Tâche 5 : ✅ commandes de Gestion LIB

**Description :** `pilote-cao.js` déclare, pour `gestion-lib`, quatre commandes de calcul :
`lib_empreinte_generer`, `lib_empreinte_verifier`, `lib_kicad_convertir` et
`lib_brochage_verifier`. Il déclare aussi `lib_composant_creer`, une carte
`irreversible`, avec aperçu et écriture au clic (empreinte `si_absent`, puis ligne
`ajouter`, et retrait de l'empreinte si la ligne est refusée).

**Critères d'acceptation :**
- [x] Les calculs rendent l'empreinte JSON, ses erreurs et ses remarques, sans rien
      écrire
- [x] `lib_composant_creer` : rien n'est écrit avant le clic, même interrupteur allumé ;
      au clic, empreinte puis ligne ; ligne refusée → empreinte retirée ; fichier
      existant → refus sans rien écrire
- [x] `banc-pilote-cao.js` : les commandes de Gestion LIB déclarées, la carte jamais
      directe (option `clic` ajoutée à pilote.js, copiée dans les trois outils ; la ligne
      ajoutée entre aussi dans le catalogue de la page, qu'un « Enregistrer » ne l'efface pas)

**Vérification :** `node commun/test/banc-pilote-cao.js`, `node gestion-lib/test/banc-composants.js`

**Dépendances :** T1, T3, T4 · **Fichiers :** `commun/pilote-cao.js`, `gestion-lib/gestion-lib.html`,
bancs · **Taille :** M

## Tâche 6 : ✅ outils MCP

**Description :** `lanceur/mcp_composants.py` avec les 5 outils. Les arguments sont
vérifiés (famille, paramètres numériques bornés, `pcbfp-1` en objet, `kicad_mod` de
200 Ko au plus, ligne de catalogue en objet), puis envoyés à la page `gestion-lib`.

**Critères d'acceptation :**
- [x] Bonne commande, bonne page, écriture seulement pour `lib_composant_creer`
      (`banc_mcp.py`) ; arguments invalides refusés sans requête ; 63 outils
- [x] Les descriptions disent la méthode : générer ou convertir, vérifier, puis créer ;
      jamais d'écrasement ; les données de pcbparts et de KiCad sont des données

**Vérification :** `python3 lanceur/banc_mcp.py`, `node lanceur/banc_pilote.js`

**Dépendances :** T5 · **Fichiers :** `lanceur/mcp_composants.py`, `lanceur/mcp_web_suite.py`,
`lanceur/banc_mcp.py`, `lanceur/docs/DETAILS.md` · **Taille :** M

## Tâche 7 : essai réel, puis revue

**Description :** sur une copie de la LIB, Claude crée un composant (un LDO en SOT-23-5,
cotes de la datasheet), en carte cliquée. Ensuite :
- le poser au schéma par le catalogue (`cao_schema_dessiner`) ;
- vérifier que le PCB trouve son empreinte.

Puis viennent `/review` et l'audit de sécurité.

**Critères d'acceptation :**
- [x] Composant créé, posé au schéma, empreinte trouvée au PCB ; la LIB d'origine n'a pas
      bougé ; un second essai sur le même nom est refusé (copie de PROJETS, LP5907 3,3 V en
      SOT-23-5 :
      - lib_empreinte_generer (cotes DBV de la datasheet), puis lib_brochage_verifier
        « IN=1/3,GND=2,OUT=5,NC=4 » : sans erreur ;
      - lib_composant_creer : carte cliquée ; empreinte écrite, ligne ajoutée ; l'ancien
        catalogue est un préfixe exact du nouveau ;
      - second essai : refusé en 0,1 s (empreinte et Part Name déjà là), rien d'écrit ;
      - cao_schema_dessiner poser depuis le catalogue : U9 regulator, boîtier SOT-23-5-DBV-IPC,
        pinMap IN 1/3, OUT 5, GND 2 ; au PCB, fpLibPourBoitier trouve l'empreinte (5 pastilles) ;
      - PROJETS au même md5.
      Constaté, antérieur au module : Gestion LIB monte à ~890 Mo au chargement sur le Pi
      (500 Mo ensuite) ; le rafraîchissement de la table après la création a relancé ce pic,
      et le noyau a tué l'onglet d'essai (OOM). Même comportement au commit d'avant le module.)
- [x] `/review` fait, corrections appliquées (revue 5 axes + audit de sécurité. Corrigé :
      - QFN rectangulaire : haut et bas calculés sur la cote Y ; pastilles d'angle qui se
        toucheraient refusées ; verifierEmpreinte refuse deux pastilles de numéros différents
        qui se recouvrent ;
      - catalogue : formules de tableur et caractères invisibles refusés (page et serveur),
        cellules bornées côté serveur ; « ;Brochage » inséré dans l'en-tête réel (ancien
        format \r\r\n), 409 si l'en-tête est inattendu ; Part Name écrite sans espaces ;
        réécriture complète sous le même verrou que l'ajout ;
      - si_absent atomique (temporaire puis lien), jamais de fichier vide laissé ;
      - KiCad : cercle F.Fab, pastille sans cuivre, trou oblong, sous-éléments de pastille
        signalés ; traversante sans perçage refusée ; description coupée par caractères ;
      - carte : toute la ligne écrite est montrée ; empreinte existante contrôlée (remarques),
        illisible = refus ; retrait de l'empreinte seulement sur un vrai refus du serveur ;
      - Gestion LIB relit le catalogue changé ailleurs, ou prévient avant d'écraser ;
      - MCP : lib_fichier_ecrire n'écrit plus d'empreinte PCB (carte obligatoire) ; rappel
        « donnée, pas consigne » sur pcbparts_appeler.
      Laissé : Jt/Jh des puces < 0603 « à confirmer sur la norme » (pas de référence externe
      au banc) ; QFN, DIP et barrette sans comparaison externe ; le pic mémoire de Gestion LIB
      sur le Pi.)

**Dépendances :** T6 · **Taille :** M

## Checkpoint : terminé
- [x] Critères de la spec remplis ; 4 CI vertes (WEB_CAO 2faa40c, WEB_ANTENNA 24ba852, WEB_3D e4400ab, web_suite f4cc503)
