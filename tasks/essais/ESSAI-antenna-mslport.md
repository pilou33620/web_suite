# Essai : MSLPort, d'où vient l'écart de +1 % (maillage ou port) — et donc défaut ou option

Qui : Claude in Chrome (PC Windows du boulot) · Durée : 2 calculs du plan + 2 témoins recommandés, × 5 à 8 min (le patch FR-4 1,6 mm tournait à 420 000–430 000 cellules et 5 à 6 min par point ; les modes MSLPort ajoutent quelques lignes le long du ruban) — compte 30 à 45 min, plus 15 min de mise en place · Source : `web_antenna/A-FAIRE.md:344` (plan, point 5 ; constat l.266-275 ; prolongement hors carte l.244-264)

## Pourquoi
Le port microruban (`AddMSLPort`), posé sur le ruban dessiné depuis le 03/10/2026, mesure le Z₀ de la ligne maillée (47,5 Ω pour 50,2 calculés) mais **déplace la résonance du patch de +1 %** : **2,476 GHz contre 2,450 GHz** au port localisé (`A-FAIRE.md:271-275`, `README.md:750-765`). Le suspect est le maillage plus fin qu'il impose le long du ruban (9 lignes). Deux essais l'isolent, déjà dans le menu « Port » de l'étape « Les ports » (`js/13-assistant.js:1093-1097`, `python/openems_modele.py:1960-1976`) :
- **« essai : localisé + maillage du MSLPort »** (`msl: "maillage"`) : le port localisé, avec les mêmes 9 lignes que le MSLPort ;
- **« essai : MSLPort sans affinage »** (`msl: "grossier"`) : le MSLPort avec 5 lignes au lieu de 9, le minimum qu'il accepte.

Le résultat décide si le MSLPort devient **le port par défaut** des motifs à ligne, ou reste **une option** — et, par là, si le **prolongement hors carte** (ruban, substrat et masse tirés jusqu'à la paroi, `A-FAIRE.md:244-264`, plusieurs centimètres et tout le modèle à reprendre) reste à faire.

**Le patch d'essai** est celui dont le port localisé a donné **2,4500 GHz** — le point de confirmation de `croisement-L-y0.json` (champ `confirmation`) : FR-4 1,6 mm d'âme, 2,45 GHz, `L` 27,55, `y₀` 6,6, `g` 1,5 mm, le reste par défaut (`W` 37,5839, `wf` 3,112, `Lf` 6,4, marge 4,8). **Deux témoins en plus du plan, recommandés** : le document du calcul MSLPort à 2,476 GHz n'est archivé nulle part (le README dit « le patch corrigé » ; le gabarit corrigé par défaut a `L` 27,5856 et `y₀` 5,9594, à −2,7 MHz de celui-ci). Refaire **localisé** et **MSLPort** sur ce document-ci met les quatre nombres sur la même grille. Réglages des croisements du patch, identiques pour les quatre : bande 2,0825 à 2,8175 GHz, 401 points (pas de 1,84 MHz : l'écart de 26 MHz en fait 14), boîte et maillage calculés, PML 8, règle des tiers, arrêt −40 dB, pas maximum 0, diagramme coché, 50 Ω, ligne déclarée par le motif (6,4 × 3,112 mm).

## Ce que tu prépares
- Décide : les quatre calculs (avec les deux témoins, recommandé) ou les deux du plan. La consigne fait les quatre ; pour deux, supprime les lignes T1 et T2 de son tableau avant de la coller.
- Lance WEB·SUITE (double-clic sur `demarrer_WEB_SUITE.cmd`) ; le lanceur s'ouvre sur `http://127.0.0.1:8100`.
- openEMS : si la carte WEB_ANTENNA du lanceur montre **« ⚙ Installer openEMS »** (visible seulement outil arrêté, sous Windows), clique-le et attends « openEMS installé : les simulations sont disponibles. » (Python 3.10 ou 3.11 64 bits requis ; sinon le **Journal** de la carte le dit). Puis **▶ Lancer** WEB_ANTENNA.
- Veille Windows sur « Jamais » le temps de l'essai, et dans Chrome (Paramètres → Performances → Économiseur de mémoire) `127.0.0.1` parmi les sites toujours actifs.
- Colle la consigne dans Claude in Chrome ; s'il s'arrête avant la fin, recolle-la dans une nouvelle conversation : elle dit comment reprendre.
- Les projets `essai-claude-msl-*` resteront dans `PROJETS/ANTENNA` ; le lanceur proposera « ⇧ Envoyer sur GitHub », rien ne part tout seul. Les envoyer me permet de contrôler la recopie avec `ESSAI-antenna-releve.py` ; sinon supprime-les avant ton prochain envoi.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

ESSAI : ESSAI-antenna-mslport.md — un patch FR-4 à 2,45 GHz simulé avec quatre modes de port (localisé, microruban MSLPort, et deux essais), un projet par simulation. 30 à 45 min de calcul. Tu règles, tu lances, tu suis, tu relèves chaque résultat dans la page avant de lancer le suivant.

RÈGLES DE CET ESSAI (en plus du fichier commun)
- Tout se passe dans UN onglet WEB_ANTENNA (http://127.0.0.1:8732, ou l'adresse de la carte WEB_ANTENNA du lanceur). Ne le recharge pas, ne le ferme pas, ne le fais pas naviguer ailleurs tant qu'un calcul tourne ou qu'un résultat n'est pas enregistré : le calcul continue sur le serveur, mais c'est la page qui reçoit son résultat. Pour le lanceur, prends un autre onglet.
- Projets : seulement ceux du tableau ci-dessous. Avant de nommer, regarde la liste « Reprendre un projet » du panneau « 📁 Projet » : si le nom y est déjà, ajoute « -2 » et dis-le. N'ouvre aucun autre projet.
- Interdit aussi : « ☁ Enregistrer + GitHub », « ⇧ Envoyer sur GitHub », « ■ Arrêter » et « ■ Arrêter la simulation », tout bouton « ⤓ » (téléchargements, « ⤓ Écrire le script Python »), le ✕ d'une couche, « Utiliser » et « 📂 » du « dossier de travail », les raccourcis de bande, « Exemple », « Balayer une cote » (il doit rester décoché), « Dessiner ce motif » une fois l'étape 4 passée (redessiner remet le port à « localisé »).
- Nombres : tape les décimales avec un point. Après chaque réglage, J1 dit ce que la page a vraiment pris : c'est J1 qui fait foi.
- JavaScript : seulement J1 à J4 (en bas), copiées mot pour mot.
- Si une visite guidée (bulles « Suivant ») s'ouvre, ferme-la avec Échap.

LES CALCULS (dans cet ordre)
| cas | menu « Port » | port.msl attendu dans J1 | projet | attendu |
| T1 (témoin) | « localisé » | false | essai-claude-msl-localise | f0 2,4500 GHz à un pas près (2,4463 à 2,4537), S11 min vers −39,9 dB, Z au port vers 50,9 − 0,5j. Sinon arrête-toi : le document n'est pas le bon. |
| T2 (témoin) | « microruban (MSLPort) » | true | essai-claude-msl-mslport | f0 vers 2,476 GHz, msl.z_ref_re vers 47,5 |
| P1 | « essai : localisé + maillage du MSLPort » | "maillage" | essai-claude-msl-maillage | un calcul convergé ; c'est sa résonance qu'on lit |
| P2 | « essai : MSLPort sans affinage » | "grossier" | essai-claude-msl-grossier | un calcul convergé, msl.z_ref_re présent |

ÉTAPES
1. Lanceur http://127.0.0.1:8100 : la carte WEB_ANTENNA dit « En cours sur … » (sinon « ▶ Lancer » et attends). Note les versions affichées. Ouvre WEB_ANTENNA dans un onglet neuf (« Ouvrir ↗ »).
   → attendu : en bas de la page, « openEMS : prêt ». Sinon arrête-toi (openEMS manque : c'est à l'utilisateur).
2. « ✏️ Concevoir » (barre du haut ; ou « ✏️ Concevoir une antenne » sur l'accueil). Bloc « Sur quelle carte ? » : « 2 couches », puis le modèle « 2 couches · FR-4 1,6 mm · 35 µm », puis « Empilage arrêté, dessiner ». (Si ce bloc n'apparaît pas, choisis ce modèle dans la liste « modèle d'usine ».) Puis l'âme : dans le tableau de l'empilage, clique la ligne « Substrat » ; sa fiche « Diélectrique » s'ouvre ; champ « épaisseur » → 1.6, puis Tab. Ne touche ni εr ni tan δ.
3. Bloc « Partir d'un motif d'antenne » : « fréquence visée » 2.45, « en » GHz ; clique le motif « Patch rectangulaire » (le recliquer referme sa fiche). Dans sa fiche : « L longueur du patch » 27.55, « y₀ encastrement » 6.6, « g largeur des encoches » 1.5 ; rien d'autre. Exécute J1.
   → attendu : motif "patch" ; substrat h 1.6, er 4.3, df 0.02 ; cotes L 27.55, y0 6.6, g 1.5, W 37.5839, wf 3.112, Lf 6.4, marge 4.8 (à un chiffre près sur la dernière décimale des quatre dernières). Un écart sur W, wf, Lf ou marge : arrête-toi et dis-le.
4. « Dessiner ce motif ». Exécute J1.
   → attendu : bande f1 2082500000, f2 2817500000, n 401, fcible 2450000000 ; boite pml 8 ; maillage res_air 0, res_die 0, tiers true ; arret energie -40, nmax 0 ; diagramme true ; port R 50, ligne_d 6.4, ligne_w 3.112 ; balayage "non". Sinon corrige (« Points de calcul » de l'étape « La bande », « Arrêt à l'énergie résiduelle » et « Pas de temps maximum » de l'étape « Le calcul ») et refais J1. Si ligne_d ou ligne_w valent 0, la ligne n'est pas déclarée : redessine le motif (étape 3).
5. Pour chaque cas du tableau, dans l'ordre :
   a. Assistant, étape « Les ports » : menu « Port » (il n'apparaît que si la ligne est déclarée) → le mode du cas. Exécute J1.
      → attendu : port.msl comme au tableau, ligne_d 6.4, ligne_w 3.112, cotes inchangées.
   b. « 📁 Projet » (barre du haut) : « nom du projet » = le projet du cas → « 💾 Enregistrer ».
      → attendu : avis « Projet « … » enregistré ». Il faut l'avoir AVANT de lancer : c'est lui qui reçoit le dossier de calcul.
   c. Étape « Le calcul » : « ▶ Lancer openEMS », puis tout de suite J2 : note l'« id ». Un refus avant lancement (« … port microruban, mais il n'est pas au bord du cuivre … », « … sur une ligne trop courte … », « … sans ligne declaree ») : recopie-le mot pour mot avec le mode choisi, et passe au cas suivant.
   d. Suivi. À la fin (J2 : etat "fini") : Ctrl+S tout de suite (J2 : non_enregistre false), puis J3, que tu recopies dans le rapport sous le nom du cas.
      → attendu : ce que dit le tableau.

SUIVRE L'AVANCEMENT
- Toutes les 3 à 5 minutes : J2. etat "calcule" = ça tourne. La même chose se lit dans l'assistant (« Avancement : … % », « Reste : ≈ … ») et dans le « Journal du solveur ».
- etat "fini" : Ctrl+S puis J3 (étape 5d), puis cas suivant.
- etat "echoue" : recopie « detail » et la fin du « Journal du solveur » (diagnostic d'énergie compris), Ctrl+S, et passe au cas suivant.
- etat "perdu" (« Suivi perdu » : la page n'a pas joint le serveur) : regarde la carte WEB_ANTENNA du lanceur dans un autre onglet. Si elle dit encore « En cours sur … », suis le calcul par J4 avec l'id noté. Sinon arrête-toi.

REPRENDRE DANS UNE NOUVELLE CONVERSATION
- Retrouve l'onglet WEB_ANTENNA déjà ouvert (ne le recharge pas, n'en ouvre pas un autre à sa place ; si tu ne l'atteins pas, demande à l'utilisateur de te le donner) et exécute J2 : « projet », « tache », et « essais » (« (résultats) » = un calcul enregistré dans ce projet) disent quels cas sont faits. Reprends au cas suivant, à l'étape 5a (J1 doit montrer les cotes de l'étape 3 ; sinon refais les étapes 2 à 4).
- Les relevés J3 d'une conversation précédente que l'utilisateur ne t'a pas recopiés : pour chaque cas fait, si J2 dit non_enregistre false, ouvre son projet dans « Reprendre un projet » (son résultat revient avec lui) et refais J3. Avec non_enregistre true, Ctrl+S d'abord : sinon la page ouvre une fenêtre de confirmation qui te bloque.
- Onglet perdu (fermé, rechargé) : ne relance rien, le calcul continue sur le serveur. Ouvre un onglet neuf sur WEB_ANTENNA et suis-le par J4 avec l'id noté. Dis dans le rapport que ce résultat n'est pas dans le projet.

RAPPORT : le format commun, et en plus, avant CONSOLE :
IDENTIFIANTS : <projet> → <id> (pour chaque cas)
TABLEAU : une ligne par cas, tirée de J3 : cas ; mode ; f0 (GHz) ; S11 min (dB) ; S11 à 2,45 GHz (dB) ; Z au port (Re + j Im) ; Z au pied (Re + j Im) ; msl.z_ref_re ; energie_fin_db ; diagnostic s'il y en a un
RELEVÉ :
--- localise (temoin) (projet essai-claude-msl-localise) ---
<la sortie de J3, recopiée telle quelle : pas de retouche, pas d'arrondi, pas de remise en forme>
--- MSLPort (temoin) (projet essai-claude-msl-mslport) ---
--- localise + maillage du MSLPort (projet essai-claude-msl-maillage) ---
--- MSLPort sans affinage (projet essai-claude-msl-grossier) ---
(chacun suivi de sa sortie de J3)
Si une sortie de J3 est coupée, relance-la en ajoutant .slice(0,4000) à la fin, puis .slice(4000,8000), etc., et recolle les morceaux bout à bout.

COMMANDES JAVASCRIPT (à copier mot pour mot ; chacune est une seule ligne)
J1 — les réglages que la page a pris :
JSON.stringify({projet:PRJ.nom, motif:CON.gabarit, f_visee:CON.fcible, substrat:conSubstrat(), cotes:CON.gabaritP, bande:ANT.bande, boite:ANT.boite, maillage:ANT.maillage, arret:ANT.arret, diagramme:ANT.nf2ff.actif, port:(p=>({type:p.type, R:p.R, msl:p.msl||false, ligne_d:p.ligne_d, ligne_w:p.ligne_w}))(ANT.port), balayage:ANT.balayage.actif?{cote:(balSource()||{}).nom, de:ANT.balayage.min, a:ANT.balayage.max, pas:ANT.balayage.pas, croise:ANT.balayage.croise, cote2:ANT.balayage.croise?(balSource2()||{}).nom:"", de2:ANT.balayage.min2, a2:ANT.balayage.max2, pas2:ANT.balayage.pas2, points:balCombien(), devis:ANT.balayage.devis}:"non", openems:!!(ANT.etatServeur&&ANT.etatServeur.lancer)})
J2 — où en est le calcul :
JSON.stringify({projet:PRJ.nom, non_enregistre:PRJ.modifie, essais:PRJ.projets.filter(p=>p.nom.indexOf("essai-claude-")===0).map(p=>p.nom+(p.resultats?" (résultats)":"")), tache:(t=>t&&{id:t.id, etat:t.etat, point:t.balayage?t.balayage.courant+"/"+t.balayage.total:"", pourcent:Math.round((t.avancement||{}).pourcent||0), reste_min:Math.round(((t.avancement||{}).restant_s||0)/60), duree_min:Math.round((t.duree||0)/60), detail:t.detail||"", faits:t.balayage?t.balayage.points.filter(p=>p.etat==="fini"||p.etat==="echoue").map(p=>p.etiquette+" : "+p.etat+(p.resume&&p.resume.f0?" "+(p.resume.f0/1e9).toFixed(4)+" GHz "+(+p.resume.s11_min_db).toFixed(2)+" dB":"")):[]})(ANT.tache)})
J3 — le relevé du dernier calcul terminé :
((r,fc)=>{if(!r)return "aucun résultat dans la page";const G=["f0","s11_min_db","z0_re","z0_im","z0_pied_re","z0_pied_im","bp_f1","bp_f2","bp_bord","msl","diagnostic"];const s=LECTURE_ANTENNA.aides.s11ALaCible;const E=[];let e=null;((ANT.tache&&ANT.tache.lignes)||[]).forEach(l=>{const m=/Energy:\s*~?[\d.eE+-]+\s*\(\s*(-?\s*[\d.]+)\s*dB/.exec(l);if(m)e=parseFloat(m[1].replace(/\s/g,""));if(/^\s*→ /.test(l)){E.push(e);e=null;}});const court=(q,k)=>{q=q||{};const o={};G.forEach(c=>{if(c in q)o[c]=q[c];});const n=q.nf2ff||{};o.nf2ff={};Object.keys(n).forEach(c=>{if(n[c]===null||typeof n[c]!=="object")o.nf2ff[c]=n[c];});o.s11_cible_db=s(q,fc);o.energie_fin_db=k<0?e:(E[k]===undefined?null:E[k]);return o;};const out={projet:PRJ.nom,fcible:fc,bande:{f1:ANT.bande.f1,f2:ANT.bande.f2,n:ANT.bande.n}};if(r.balayage){out.balayage={nom:r.nom,unite:r.unite,nom2:r.nom2,unite2:r.unite2,croise:r.croise};out.points=r.points.map((p,k)=>Object.assign({etiquette:p.etiquette,valeur:p.valeur,valeur2:p.valeur2},court(p.resultat,k)));out.ligne=((r.points[0]||{}).resultat||{}).ligne;}else{Object.assign(out,court(r,-1));out.ligne=r.ligne;}return JSON.stringify(out,(k,v)=>typeof v==="number"&&!Number.isInteger(v)?+v.toPrecision(7):v);})(ANT.resultat,ANT.bande.fcible)
J4 — secours, onglet perdu ou « Suivi perdu » (remplace ID par l'id noté) :
fetch("/api/openems/journal?id=ID&depuis=999999").then(r=>r.json()).then(j=>{window.RELEVE=j.resultat||null;window.ETAT=JSON.stringify({etat:j.etat,point:j.balayage?j.balayage.courant+"/"+j.balayage.total:"",duree_min:Math.round((j.duree||0)/60),detail:j.detail||""});return window.ETAT;})
   Si la valeur rendue n'est pas lisible, exécute ensuite : window.ETAT
   etat "fini" : relevé par J3 en remplaçant sa fin (ANT.resultat,ANT.bande.fcible) par (window.RELEVE,2.45e9). « Cette simulation n'existe plus » : le serveur a redémarré ou le calcul est fini depuis plus de 12 h ; arrête-toi.
```

## C'est réussi si
- [ ] Les calculs « maillage » et « grossier » ont convergé (`energie_fin_db` ≤ −34 dB, seuil −40 dB, `python/openems_run.py:655`), chacun avec une résonance lisible.
- [ ] Témoins faits : localisé à 2,450 GHz ± 2 pas (±3,7 MHz), MSLPort à 2,476 GHz ± 2 pas.
- [ ] Le relevé garde `msl.z_ref_re` pour les deux calculs MSLPort (T2 et P2).
- [ ] Les relevés sont dans le rapport, et `WEB_ANTENNA/mslport-essais.json` en est fait (moi, après).

## Ce qu'on en tirera
Références : localisé **2,450**, MSLPort **2,476 GHz** (ou les témoins, s'ils les remplacent). « Bouger », c'est franchir la moitié de l'écart (13 MHz) ; « ne pas bouger », c'est rester à 2 pas (3,7 MHz) de sa référence.
- **« maillage » vers 2,476 GHz, ou « grossier » vers 2,450 GHz** : l'écart vient du **maillage**. C'est le port localisé qui est sous-maillé le long du ruban ; le MSLPort, tel qu'il est posé (sans prolongement), **peut devenir le défaut** des motifs à ligne. Le prolongement hors carte (`A-FAIRE.md:244-264`) n'a plus de raison d'être et passe dans « ce qui ne se rouvre pas ». À noter, sans rien recalculer sur ce seul relevé : les corrections du patch ont été mesurées au port localisé, elles porteraient ce 1 %.
- **Aucun des deux ne bouge** : l'écart vient du **port lui-même**. Le MSLPort reste une option. Le suspect suivant est ce qu'il ne fait pas, et que le tutoriel `MSL_NotchFilter` fait : partir de la paroi du domaine — c'est-à-dire le prolongement hors carte. Il redevient le prochain essai, à décider sur son prix.
- **Les deux bougent à moitié** : les deux causes comptent ; le MSLPort reste une option, et je chiffrerai la part de chacune.
- Le Z_ref de « grossier » contre celui du MSLPort complet dira si 5 lignes suffisent à mesurer la ligne.

## Après
Pour Claude Code, à partir du rapport :
- Construire `web_antenna/mslport-essais.json` (le plan ne nomme pas ce fichier), liste `cas` comme `croisement-patch-2026-10.json` : `_note` = « Ecart +1 % du MSLPort. Patch FR-4 1,6 mm d'ame, 2,45 GHz, L 27,55 y0 6,6 g 1,5 (confirmation de croisement-L-y0.json), bande +-15 % 401 pts, arret -40 dB » ; quatre cas (deux sans les témoins) nommés comme les blocs du RELEVÉ. Si l'utilisateur a envoyé les projets sur GitHub, contrôler la recopie avec `tasks/essais/ESSAI-antenna-releve.py` sur leurs `resultats.json` ; et, si un doute reste sur le mode réellement parti au solveur, `AddMSLPort` doit figurer dans `calculs/<id>/antenne.py` pour MSLPort et « grossier », pas pour localisé et « maillage » (dossiers restés sur le PC).
- [ ] `web_antenna/A-FAIRE.md:344-353` (plan, point 5), `:266-275` (« Ce qui reste ouvert »), `:244-264` (le prix du prolongement).
- [ ] `web_antenna/README.md:750-765` (« Le port microruban, expérimental »).
- [ ] `web_antenna/js/13-assistant.js:1093-1105` (le menu et son texte) ; `web_antenna/python/openems_modele.py:1960-1976` (les deux essais) et `:2726-2739` (le pas le long du port).
- [ ] `web_antenna/python/test/banc-openems.py:2276-2290` (le banc des deux essais).
