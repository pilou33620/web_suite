# Essai : MSLPort, d'où vient l'écart de +1 % (maillage ou port) — et donc défaut ou option

Où : poste avec openEMS (WEB_ANTENNA lancé par WEB·SUITE) · Durée : 2 calculs du plan + 2 témoins recommandés, × 5 à 8 min (le patch FR-4 1,6 mm tournait à 420 000–430 000 cellules et 5 à 6 min par point ; les modes MSLPort ajoutent quelques lignes le long du ruban) — compte 30 à 45 min · Source : `web_antenna/A-FAIRE.md:344` (plan, point 5 ; constat l.266-275 ; prolongement hors carte l.244-264)

## Pourquoi
Le port microruban (`AddMSLPort`), posé sur le ruban dessiné depuis le 03/10/2026, mesure le Z₀ de la ligne maillée (47,5 Ω pour 50,2 calculés) mais **déplace la résonance du patch de +1 %** : **2,476 GHz contre 2,450 GHz** au port localisé (`A-FAIRE.md:271-275`, `README.md:750-765`). Le suspect est le maillage plus fin qu'il impose le long du ruban (9 lignes). Deux essais l'isolent, déjà dans le menu du port (`js/13-assistant.js:1093-1097`, `python/openems_modele.py:1960-1976`) :
- **« essai : localisé + maillage du MSLPort »** (`msl: "maillage"`) : le port localisé, avec les mêmes 9 lignes que le MSLPort ;
- **« essai : MSLPort sans affinage »** (`msl: "grossier"`) : le MSLPort avec 5 lignes au lieu de 9, le minimum qu'il accepte.

Le résultat décide si le MSLPort devient **le port par défaut** des motifs à ligne, ou reste **une option** — et, par là, si le **prolongement hors carte** (ruban, substrat et masse tirés jusqu'à la paroi, `A-FAIRE.md:244-264`, plusieurs centimètres et tout le modèle à reprendre) reste à faire.

## Avant de commencer
- WEB_ANTENNA au moins au commit `0decae2` (les deux essais du menu) ; spec écrite sur `c95f9b6` du 21/10/2026. Dans WEB_ANTENNA, `python python/test/banc-openems.py` → **1053 vérifications, toutes passées** (dont « « maillage » : les lignes du MSLPort sous un port localisé » et « « grossier » : le MSLPort reste, avec ses cinq lignes au moins », `python/test/banc-openems.py:2280`, `:2286`).
- **Le patch d'essai** : celui dont le port localisé a donné **2,4500 GHz** — le point de confirmation de `croisement-L-y0.json` (champ `confirmation`) : **✏️ Concevoir** → **« 2 couches · FR-4 1,6 mm · 35 µm »**, **âme retouchée à 1,600 mm** → **2,45 GHz** → **« Patch rectangulaire »** → cotes **`L` = 27,55**, **`y₀` = 6,6**, **`g` = 1,5** mm, le reste par défaut (`W` 37,5839, `wf` 3,112, `Lf` 6,4, marge 4,8) → **Dessiner ce motif**.
- **Deux témoins, en plus du plan, et je te les recommande** : le document du calcul MSLPort à 2,476 GHz n'est archivé nulle part (le README dit « le patch corrigé » ; le gabarit corrigé par défaut a `L` 27,5856 et `y₀` 5,9594, à −2,7 MHz de celui-ci). Refaire **localisé** et **MSLPort** sur ce document-ci met les quatre nombres sur la même grille ; sans eux, la comparaison repose sur deux références d'origine incertaine. Si tu ne veux que le plan, saute les calculs 1 et 2.
- Réglages des croisements du patch, identiques pour les quatre : bande **2,0825 à 2,8175 GHz, 401 points** (pas de 1,84 MHz : l'écart de 26 MHz en fait 14), boîte et maillage calculés (0), PML 8, règle des tiers, arrêt **−40 dB**, pas maximum 0, diagramme coché, 50 Ω, ligne déclarée par le motif (6,4 × 3,112 mm).
- Un projet par calcul (`resultats.json` ne garde que le dernier) : changer le nom dans **« 📁 Projet »** puis **Ctrl+S** fait une copie.

## Étapes
Pour chaque calcul : étape « Les ports » → panneau du port → menu **« Port »** (il n'apparaît que si la ligne est déclarée) → le mode indiqué ; « 📁 Projet » → le nom indiqué, **Ctrl+S** ; étape « Le calcul » → **▶ Lancer openEMS** ; à la fin, **Ctrl+S**. Vérifie le menu avant chaque lancement : redessiner le motif peut le remettre à « localisé ».

1. *(témoin)* Port **« localisé »** → projet `essai-msl-localise`.
   → attendu : **2,4500 GHz** à un pas près, −39,9 dB, 50,9 − 0,5 j au port (la confirmation du 02/10/2026). Sinon, arrête-toi : le document n'est pas le bon.
2. *(témoin)* Port **« microruban (MSLPort) »** → projet `essai-msl-mslport`.
   → attendu : vers **2,476 GHz**, Z_ref mesuré vers 47,5 Ω (dans le relevé : `msl.z_ref_re`).
3. Port **« essai : localisé + maillage du MSLPort »** → projet `essai-msl-maillage`.
   → attendu : un calcul convergé ; c'est sa résonance qu'on lit.
4. Port **« essai : MSLPort sans affinage »** → projet `essai-msl-grossier`.
   → attendu : un calcul convergé, Z_ref mesuré présent.
5. Enregistrer le relevé sous `mslport-essais.json` à la racine de WEB_ANTENNA (le plan ne nomme pas ce fichier ; format des précédents, une liste de cas) — depuis le dossier de WEB·SUITE :
   ```
   python tasks/essais/ESSAI-antenna-releve.py WEB_ANTENNA/mslport-essais.json "Ecart +1 % du MSLPort. Patch FR-4 1,6 mm d'ame, 2,45 GHz, L 27,55 y0 6,6 g 1,5 (confirmation de croisement-L-y0.json), bande +-15 % 401 pts, arret -40 dB" "localise (temoin)=PROJETS/ANTENNA/essai-msl-localise/resultats.json" "MSLPort (temoin)=PROJETS/ANTENNA/essai-msl-mslport/resultats.json" "localise + maillage du MSLPort=PROJETS/ANTENNA/essai-msl-maillage/resultats.json" "MSLPort sans affinage=PROJETS/ANTENNA/essai-msl-grossier/resultats.json"
   ```
   → attendu : « … : 4 cas, 4 points » (2 cas si tu n'as fait que le plan : retire les deux témoins de la commande).

## C'est réussi si
- [ ] Les calculs « maillage » et « grossier » ont convergé à −40 dB, chacun avec une résonance lisible.
- [ ] Témoins faits : localisé à 2,450 GHz ± 2 pas (±3,7 MHz), MSLPort à 2,476 GHz ± 2 pas.
- [ ] Le relevé garde `msl.z_ref_re` pour les deux calculs MSLPort.
- [ ] `WEB_ANTENNA/mslport-essais.json` existe.

## Ce qu'on en tirera
Références : localisé **2,450**, MSLPort **2,476 GHz** (ou les témoins, s'ils les remplacent). « Bouger », c'est franchir la moitié de l'écart (13 MHz) ; « ne pas bouger », c'est rester à 2 pas (3,7 MHz) de sa référence.
- **« maillage » vers 2,476 GHz, ou « grossier » vers 2,450 GHz** : l'écart vient du **maillage**. C'est le port localisé qui est sous-maillé le long du ruban ; le MSLPort, tel qu'il est posé (sans prolongement), **peut devenir le défaut** des motifs à ligne. Le prolongement hors carte (`A-FAIRE.md:244-264`) n'a plus de raison d'être et passe dans « ce qui ne se rouvre pas ». À noter, sans rien recalculer sur ce seul relevé : les corrections du patch ont été mesurées au port localisé, elles porteraient ce 1 %.
- **Aucun des deux ne bouge** : l'écart vient du **port lui-même**. Le MSLPort reste une option. Le suspect suivant est ce qu'il ne fait pas, et que le tutoriel `MSL_NotchFilter` fait : partir de la paroi du domaine — c'est-à-dire le prolongement hors carte. Il redevient le prochain essai, à décider sur son prix.
- **Les deux bougent à moitié** : les deux causes comptent ; le MSLPort reste une option, et je chiffrerai la part de chacune.
- Le Z_ref de « grossier » contre celui du MSLPort complet dira si 5 lignes suffisent à mesurer la ligne.

## Si ça rate, note
- Le refus avant lancement, mot pour mot : « … port microruban, mais il n'est pas au bord du cuivre … », « … sur une ligne trop courte … » ou « … sans ligne declaree » (`python/openems_modele.py:1845`, `:1855`, `:1978`) — et le mode choisi.
- Le journal : panneau **« Journal du solveur »** de la page (pas écrit sur disque) — la fin, diagnostic d'énergie compris.
- Le dossier du calcul : `PROJETS/ANTENNA/<projet>/calculs/<identifiant>/antenne.py` — cherche `AddMSLPort` dedans : présent pour MSLPort et « grossier », absent pour localisé et « maillage ».

## Après
Donne-moi le JSON : je remplis `A-FAIRE.md`, je tranche défaut ou option avec toi, et selon le cas je retire les deux essais du menu ou j'ouvre le prolongement hors carte. Cases concernées :
- `web_antenna/A-FAIRE.md:344-353` (plan, point 5), `:266-275` (« Ce qui reste ouvert »), `:244-264` (le prix du prolongement).
- `web_antenna/README.md:750-765` (« Le port microruban, expérimental »).
- `web_antenna/js/13-assistant.js:1093-1105` (le menu et son texte) ; `web_antenna/python/openems_modele.py:1960-1976` (les deux essais) et `:2726-2739` (le pas le long du port).
- `web_antenna/python/test/banc-openems.py:2276-2290` (le banc des deux essais).
