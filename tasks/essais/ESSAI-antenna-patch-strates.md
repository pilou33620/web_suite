# Essai : patch, deux strates de plus pour `patchCorrRbord` (FR-4 haute fréquence, RO4003C 1,524 mm)

Qui : Claude in Chrome (PC Windows du boulot) · Durée : 2 × (7 points + 1 confirmation) = 16 calculs × 5 à 8 min (le patch FR-4 1,6 mm tournait à 420 000–430 000 cellules et 5 à 6 min par point) — compte 1 h 30 à 2 h 30, plus 9 points par stratifié si le croisement de secours est nécessaire, plus 20 min de mise en place · Source : `web_antenna/A-FAIRE.md:213` (« À mesurer » ; strates l.205-212 ; limite l.371)

> **Plan proposé, à valider avec toi.** `A-FAIRE.md` dit quoi mesurer (l.213-215), pas comment : aucun plan n'est arrêté. Celui-ci est calqué sur la mesure RO4350B déjà faite, allégé de son croisement `L` × `y₀` puisque la loi de longueur tient désormais à 0,2 %. La consigne porte une ligne `STATUT` : tant qu'elle ne dit pas `VALIDÉE`, Claude in Chrome prépare et vérifie les deux documents mais **ne lance rien**. Pour valider, remplace dans la consigne (ici, et pousse) `STATUT : À VALIDER` par `STATUT : VALIDÉE le JJ/MM`.

## Pourquoi
L'encastrement du patch sort de la résistance de bord, que le gabarit corrige par un rapport mesuré à la conductance de fente : **0,173 sur FR-4** (tan δ 0,02, trois fréquences, ±13 %) et **0,722 sur RO4350B** (tan δ 0,0037, 0,762 mm). Entre les deux, `patchCorrRbord` **interpole en tan δ** et se borne aux deux valeurs (`js/22-antennes.js:231-238`) : « une hypothèse à deux points » (`A-FAIRE.md:211-212`), que l'assistant signale sur tout autre stratifié (`js/31-cartes-antenna.js:1174-1195`). Deux mesures la trancheraient :
- **FR-4 haute fréquence** (tan δ 0,012, εr 4,20) : le gabarit y prend **0,4424** (milieu de la droite). La mesure dit si la droite tient.
- **RO4003C 1,524 mm** (tan δ 0,0027, borné à 0,722) : même famille de pertes que le RO4350B, **deux fois plus épais**. La mesure sépare les pertes de l'épaisseur.

Ce que le gabarit pose (calculé le 21/10/2026 par le code du dépôt), et où tomberait `y₀` selon la strate :

| stratifié | `L` | `W` | `wf` | `g` | `y₀` posé | `y₀` si 0,173 (FR-4) | si 0,196 | si 0,722 (RO) |
|---|---|---|---|---|---|---|---|---|
| FR-4 HF, âme 1,600 mm | 27,908 | 37,9435 | 3,167 | 1,5202 | **9,3907** | 5,92 | 6,58 | 10,45 |
| RO4003C, âme 1,524 mm | 30,3882 | 40,5634 | 3,409 | 1,6363 | **11,0947** | 5,55 | 6,40 | 11,09 |

Ce que je propose de garder du RO4350B (`croisement-patch-2026-10.json`, cas 3) : le gabarit à ses cotes (`g` = 0,48 × `wf`), un **balayage de `y₀` au pas de 0,8 mm** à `L` fixé, la lecture de Im Z au pied. Ce que j'enlève : le croisement `L` × `y₀` 3 × 3 du début — il servait à régler `L`, que `patchCorrLongueur` règle maintenant à 0,2 % près sur quatre cas. Il revient en secours si la résonance tombe à plus de 1 % de 2,45 GHz. Plage : 7 points (le RO4350B en avait 5, prolongés), qui couvrent **les deux bornes** de la strate pour le FR-4 HF, et ±2,4 mm autour du posé pour le RO4003C. FR-4 HF : **âme retouchée de 1,530 à 1,600 mm** (comme les relevés FR-4 : seul le stratifié change, pas l'épaisseur) ; RO4003C tel quel. Réglages des croisements du patch : bande du motif 2,0825 à 2,8175 GHz, 401 points, boîte et maillage calculés, PML 8, règle des tiers, arrêt −40 dB, pas maximum 0, diagramme coché, port localisé 50 Ω, ligne déclarée par le motif. L'assistant affichera « L'encastrement du patch est interpolé, pas mesuré » : c'est normal sur ces deux stratifiés, c'est précisément ce qu'on mesure.

## Ce que tu prépares
- **Valide le plan** (encadré ci-dessus) : sans cela Claude in Chrome s'arrête avant de lancer.
- Lance WEB·SUITE (double-clic sur `demarrer_WEB_SUITE.cmd`) ; le lanceur s'ouvre sur `http://127.0.0.1:8100`.
- openEMS : si la carte WEB_ANTENNA du lanceur montre **« ⚙ Installer openEMS »** (visible seulement outil arrêté, sous Windows), clique-le et attends « openEMS installé : les simulations sont disponibles. » (Python 3.10 ou 3.11 64 bits requis ; sinon le **Journal** de la carte le dit). Puis **▶ Lancer** WEB_ANTENNA.
- Des heures de calcul : veille Windows sur « Jamais » le temps de l'essai, session ouverte, et dans Chrome (Paramètres → Performances → Économiseur de mémoire) `127.0.0.1` parmi les sites toujours actifs.
- N'utilise pas WEB_ANTENNA à côté pendant l'essai.
- Colle la consigne dans Claude in Chrome ; s'il s'arrête avant la fin, recolle-la dans une nouvelle conversation : elle dit comment reprendre.
- Les projets `essai-claude-strate-*` resteront dans `PROJETS/ANTENNA` ; le lanceur proposera « ⇧ Envoyer sur GitHub », rien ne part tout seul. Les envoyer me permet de contrôler la recopie avec `ESSAI-antenna-releve.py` ; sinon supprime-les avant ton prochain envoi.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

STATUT : À VALIDER
(Si la ligne STATUT ne dit pas « VALIDÉE », fais seulement les étapes 1 et 2 pour les deux stratifiés — préparation et vérification des cotes, aucun projet, aucun calcul — puis arrête-toi et écris dans le rapport « spec non validée, rien lancé ».)

ESSAI : ESSAI-antenna-patch-strates.md — patch à 2,45 GHz sur deux stratifiés (FR-4 haute fréquence, puis RO4003C 1,524 mm) : pour chacun, un balayage de l'encastrement y₀ en 7 points, puis une simulation de confirmation. 16 calculs, 1 h 30 à 2 h 30. Tu règles, tu lances, tu suis de loin, tu relèves les résultats dans la page.

RÈGLES DE CET ESSAI (en plus du fichier commun)
- Tout se passe dans UN onglet WEB_ANTENNA (http://127.0.0.1:8732, ou l'adresse de la carte WEB_ANTENNA du lanceur). Ne le recharge pas, ne le ferme pas, ne le fais pas naviguer ailleurs tant qu'un calcul tourne ou qu'un résultat n'est pas enregistré : le calcul continue sur le serveur, mais c'est la page qui reçoit son résultat. Pour le lanceur, prends un autre onglet.
- Projets : seulement essai-claude-strate-fr4hf, essai-claude-strate-fr4hf-conf, essai-claude-strate-ro4003, essai-claude-strate-ro4003-conf (et, si les étapes le demandent, essai-claude-strate-fr4hf-plus, essai-claude-strate-ro4003-plus, essai-claude-strate-fr4hf-LY, essai-claude-strate-ro4003-LY). Avant de nommer, regarde la liste « Reprendre un projet » du panneau « 📁 Projet » : si le nom y est déjà, ajoute « -2 » et dis-le. N'ouvre aucun autre projet.
- Interdit aussi : « ☁ Enregistrer + GitHub », « ⇧ Envoyer sur GitHub », « ■ Arrêter » et « ■ Arrêter la simulation », tout bouton « ⤓ » (téléchargements, « ⤓ Écrire le script Python »), le ✕ d'une couche, « Utiliser » et « 📂 » du « dossier de travail », les raccourcis de bande, « Exemple ».
- Nombres : tape les décimales avec un point. Après chaque réglage, J1 dit ce que la page a vraiment pris : c'est J1 qui fait foi.
- JavaScript : seulement J1 à J4 (en bas), copiées mot pour mot.
- Si une visite guidée (bulles « Suivant ») s'ouvre, ferme-la avec Échap.

LES DEUX STRATIFIÉS
| | FR-4 HF | RO4003C |
| modèle d'usine | « 2 couches · FR-4 haute fréquence 1,6 mm » | « 2 couches · RO4003C 1,524 mm (60 mil) » |
| âme (« Substrat ») | épaisseur → 1.6 | ne pas toucher |
| substrat attendu dans J1 | h 1.6, er 4.2, df 0.012 | h 1.524, er 3.55, df 0.0027 |
| cotes attendues dans J1 | L 27.908, W 37.9435, wf 3.167, g 1.5202, y0 9.3907 | L 30.3882, W 40.5634, wf 3.409, g 1.6363, y0 11.0947 |
| balayage de y₀ | De 5.8, à 10.6, par pas de 0.8 (5.8 · 6.6 · 7.4 · 8.2 · 9 · 9.8 · 10.6) | De 8.7, à 13.5, par pas de 0.8 (8.7 · 9.5 · 10.3 · 11.1 · 11.9 · 12.7 · 13.5) |
| projets | essai-claude-strate-fr4hf, puis essai-claude-strate-fr4hf-conf | essai-claude-strate-ro4003, puis essai-claude-strate-ro4003-conf |
| secours L (étape 7) | De 27.1824, à 28.6336, par pas de 0.7256 | De 29.5981, à 31.1783, par pas de 0.7901 |

ÉTAPES (FR-4 HF d'abord, puis tout de nouveau pour le RO4003C à partir de l'étape 2)
1. Lanceur http://127.0.0.1:8100 : la carte WEB_ANTENNA dit « En cours sur … » (sinon « ▶ Lancer » et attends). Note les versions affichées. Ouvre WEB_ANTENNA dans un onglet neuf (« Ouvrir ↗ »).
   → attendu : en bas de la page, « openEMS : prêt ». Sinon arrête-toi (openEMS manque : c'est à l'utilisateur).
2. Le document. Première fois : « ✏️ Concevoir » (barre du haut ; ou « ✏️ Concevoir une antenne » sur l'accueil), bloc « Sur quelle carte ? » : « 2 couches », puis le modèle du stratifié, puis « Empilage arrêté, dessiner ». Pour le RO4003C (ou si ce bloc n'apparaît pas) : panneau « Conception », bloc « L'empilage et les matériaux », liste « modèle d'usine » → le modèle du stratifié. FR-4 HF seulement : dans le tableau de l'empilage, clique la ligne « Substrat » ; sa fiche « Diélectrique » s'ouvre ; champ « épaisseur » → 1.6, puis Tab (ne touche ni εr ni tan δ). Puis bloc « Partir d'un motif d'antenne » : « fréquence visée » 2.45, « en » GHz ; clique le motif « Patch rectangulaire » (le recliquer referme sa fiche ; s'il est déjà ouvert, clique plutôt « ↻ cotes du calcul » dans sa fiche : y₀ reprend la valeur calculée sur ce stratifié). Ne change aucune cote. « Dessiner ce motif ». Exécute J1.
   → attendu : motif "patch" ; le substrat et les cotes du tableau (à un chiffre près sur la dernière décimale) ; bande f1 2082500000, f2 2817500000, n 401, fcible 2450000000 ; boite pml 8 ; maillage res_air 0, res_die 0, tiers true ; arret energie -40, nmax 0 ; diagramme true ; port R 50, msl false, ligne_d et ligne_w non nuls. Un écart sur le substrat ou les cotes : arrête-toi et dis-le. Sur les réglages : corrige (« Points de calcul » de l'étape « La bande », « Arrêt à l'énergie résiduelle » et « Pas de temps maximum » de l'étape « Le calcul », menu « Port » de l'étape « Les ports » sur « localisé ») et refais J1.
   — Spec non validée : fais cette étape pour les deux stratifiés, puis arrête-toi.
3. « 📁 Projet » (barre du haut) : « nom du projet » = premier projet du stratifié → « 💾 Enregistrer ».
   → attendu : avis « Projet « … » enregistré ». Il faut l'avoir AVANT de lancer : c'est lui qui reçoit les dossiers de calcul.
4. Assistant, étape « Le calcul » : coche « Balayer une cote » → « Cote » : « Patch rectangulaire — encastrement » → De / à / par pas de du tableau. « Croiser avec une seconde cote » décoché. J1 → attendu : balayage de, a, pas du tableau, croise false, points 7. « ∑ Chiffrer la plage » (recopie le récapitulatif mot pour mot ; un refus : recopie-le et arrête-toi). « ▶ Lancer le balayage », puis tout de suite J2 : note l'« id ».
   → attendu : résonances vers 2,44 GHz, un S11 qui passe par un minimum, Im Z au pied qui croît avec y₀.
5. À la fin (J2 : etat "fini") : Ctrl+S tout de suite (J2 : non_enregistre false), puis J3, que tu recopies dans le rapport. Dans J3, cherche deux points voisins (par y₀ croissant) où z0_pied_im change de signe : y1, X1 puis y2, X2. Le zéro : y* = y1 − X1 × (y2 − y1) / (X2 − X1), arrondi à 0.05. Écris le calcul dans le rapport.
   — Pas de changement de signe : l'optimum est au bord. Prolonge de 2 points du côté qui manque (tous négatifs : au-dessus du dernier y₀ ; tous positifs : au-dessous du premier), au même pas de 0.8 : projet « …-plus » (nom changé puis « 💾 Enregistrer » = copie), balayage sur ces 2 valeurs, J1 (points 2), « ∑ Chiffrer la plage », « ▶ Lancer le balayage », J2, à la fin Ctrl+S puis J3 ; puis refais le calcul de y* sur les 9 points.
6. Confirmation. « 📁 Projet » : nom = projet « …-conf » du stratifié → « 💾 Enregistrer » (copie). Panneau « Conception », fiche du motif : « y₀ encastrement » = y* → « Dessiner ce motif ». Étape « Le calcul » : décoche « Balayer une cote ». J1 → attendu : cotes y0 = y*, les autres comme à l'étape 2 ; balayage "non". « ▶ Lancer openEMS », puis J2 (note l'id). À la fin : Ctrl+S puis J3.
   → attendu : S11 sous −25 dB à la résonance (s11_min_db), f0 entre 2,4255 et 2,4745 GHz (1 % de 2,45).
7. Secours, seulement si la f0 de la confirmation est hors de 2,4255–2,4745 GHz : « 📁 Projet » : nom « …-LY » → « 💾 Enregistrer » (copie). Étape « Le calcul » : coche « Balayer une cote » → « Cote » : « Patch rectangulaire — longueur du patch », De / à / par pas de de la ligne « secours L » du tableau ; coche « Croiser avec une seconde cote » → « Seconde cote » : « Patch rectangulaire — encastrement », De y* − 1.5, à y* + 1.5, par pas de 1.5. J1 → attendu : points 9. « ∑ Chiffrer la plage », « ▶ Lancer le balayage », J2, à la fin Ctrl+S puis J3.
8. Passe au RO4003C : étape 2 (le modèle d'usine change sous le dessin en place ; « ↻ cotes du calcul » puis « Dessiner ce motif » le refont).

SUIVRE L'AVANCEMENT (des heures : ne reste pas devant)
- Toutes les 10 à 20 minutes : J2. etat "calcule" = ça tourne ; « point » dit lequel ; « faits » liste les points finis avec leur résonance. La même chose se lit dans l'assistant (« Avancement : … % », « Reste : ≈ … ») et dans le « Journal du solveur ».
- etat "fini" : Ctrl+S puis J3 (étapes 5, 6 ou 7).
- etat "echoue" : recopie « detail » et la fin du « Journal du solveur », Ctrl+S, arrête-toi.
- etat "perdu" (« Suivi perdu » : la page n'a pas joint le serveur) : regarde la carte WEB_ANTENNA du lanceur dans un autre onglet. Si elle dit encore « En cours sur … », suis le calcul par J4 avec l'id noté. Sinon arrête-toi.
- Si tu ne peux pas attendre aussi longtemps, arrête-toi après un lancement en écrivant dans le rapport l'id, le projet, y* s'il est calculé, et la prochaine étape : l'utilisateur recollera cette consigne dans une nouvelle conversation.

REPRENDRE DANS UNE NOUVELLE CONVERSATION
- Retrouve l'onglet WEB_ANTENNA déjà ouvert (ne le recharge pas, n'en ouvre pas un autre à sa place ; si tu ne l'atteins pas, demande à l'utilisateur de te le donner) et exécute J2 : « projet », « tache », et « essais » (« (résultats) » = un calcul enregistré dans ce projet) disent où tu en es. Reprends à l'étape qui suit, en vérifiant par J1 le stratifié et les cotes.
- Les relevés J3 d'une conversation précédente que l'utilisateur ne t'a pas recopiés : pour chaque projet fait, si J2 dit non_enregistre false, ouvre-le dans « Reprendre un projet » (son résultat revient avec lui) et refais J3. Avec non_enregistre true, Ctrl+S d'abord : sinon la page ouvre une fenêtre de confirmation qui te bloque.
- Onglet perdu (fermé, rechargé) : ne relance rien, le calcul continue sur le serveur. Ouvre un onglet neuf sur WEB_ANTENNA et suis-le par J4 avec l'id noté. Dis dans le rapport que ce résultat n'est pas dans le projet.

RAPPORT : le format commun, et en plus, avant CONSOLE :
STATUT : <la ligne telle que tu l'as lue>
IDENTIFIANTS : <projet> → <id> (pour chaque lancement)
DEVIS : <le récapitulatif de « ∑ Chiffrer la plage », mot pour mot, pour chaque balayage>
ZÉRO DE Im Z AU PIED : <pour chaque stratifié, les deux points encadrants et le calcul de y*>
TABLEAU : une ligne par point, tirée de J3 : stratifié ; y₀ (et L pour le secours) ; f0 (GHz) ; S11 min (dB) ; S11 à 2,45 GHz (dB) ; Z au port (Re + j Im) ; Z au pied (Re + j Im) ; energie_fin_db ; diagnostic s'il y en a un
RELEVÉ :
--- FR-4 HF 1,6 mm, y0 (projet essai-claude-strate-fr4hf) ---
<la sortie de J3, recopiée telle quelle : pas de retouche, pas d'arrondi, pas de remise en forme>
--- FR-4 HF 1,6 mm, confirmation (projet essai-claude-strate-fr4hf-conf) ---
--- RO4003C 1,524 mm, y0 (projet essai-claude-strate-ro4003) ---
--- RO4003C 1,524 mm, confirmation (projet essai-claude-strate-ro4003-conf) ---
(chacun suivi de sa sortie de J3 ; ajoute de même les blocs « …, y0 prolonge » et « …, secours L x y0 » s'il y en a eu)
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
- [ ] La spec était validée (ligne `STATUT`).
- [ ] Les deux balayages ont 7 points calculés, convergés : `energie_fin_db` ≤ −34 dB (seuil −40 dB, `python/openems_run.py:655`), aucun `diagnostic`.
- [ ] Sur chacun, Im Z au pied change de signe entre deux points (sinon l'optimum est au bord : prolongé de 2 points du côté qui manque, comme le RO4350B).
- [ ] Les deux confirmations tombent à moins de 1 % de 2,45 GHz, ou le croisement de secours est fait.
- [ ] Les relevés sont dans le rapport, et `WEB_ANTENNA/balayage-patch-strates.json` en est fait (moi, après).

## Ce qu'on en tirera
- Pour chaque stratifié, le rapport mesuré `k = 2·G1·50 / cos²(π·y₀/L)` au `y₀` réglé (je le calcule), à placer sur le graphe k(tan δ) à côté de FR-4 (0,02 ; 0,173) et RO4350B (0,0037 ; 0,722).
- **FR-4 HF près de 0,44** : la droite en tan δ tient, `patchCorrRbord` reste tel quel et son `ponytail:` passe à trois points. **Près de 0,17–0,20** : ce n'est pas la perte qui fait la strate mais la famille de résine (ou l'εr) ; la loi devient un tableau par matériau. **Près de 0,72** : la strate tombe vite dès qu'on quitte le FR-4 standard ; la droite devient une courbe.
- **RO4003C près de 0,72** : l'épaisseur ne compte pas, la strate est une affaire de pertes. **Nettement ailleurs** : `k` dépend aussi de h/λ₀, et la loi prend une seconde variable — sur deux points, je ne la poserai pas sans un troisième.
- La loi de longueur se vérifie en passant : la résonance des confirmations doit tomber à 0,2 % près si `patchCorrLongueur` tient sur ces deux stratifiés.

## Après
Pour Claude Code, à partir du rapport :
- Construire `web_antenna/balayage-patch-strates.json` (aucun nom n'est arrêté), liste `cas` comme `croisement-patch-2026-10.json` : `_note` = « Strates de patchCorrRbord. Gabarit patch corrige, 2,45 GHz, balayage y0 au pas 0,8 puis confirmation. FR-4 HF ame 1,600 mm ; RO4003C 1,524 mm. Bande +-15 %, 401 pts, arret -40 dB » ; quatre cas (plus les prolongements et secours) nommés comme les blocs du RELEVÉ ; pour les balayages, `lignes` refaites depuis les points comme `_assembler` (`python/openems_run.py:973-987`). Vérifier le calcul de y* de Claude in Chrome. Si l'utilisateur a envoyé les projets sur GitHub, contrôler la recopie avec `tasks/essais/ESSAI-antenna-releve.py` sur leurs `resultats.json`.
- [ ] `web_antenna/A-FAIRE.md:205-215` (les strates), le tableau de la correction l.192-197, la limite l.371.
- [ ] `web_antenna/js/22-antennes.js:214-238` (`patchCorrRbord`, son commentaire et son `ponytail:`).
- [ ] `web_antenna/js/31-cartes-antenna.js:1174-1195` (l'avertissement « interpolé, pas mesuré » et sa tolérance sur tan δ).
- [ ] `web_antenna/README.md:2029-2037` (la limite « La correction du patch n'est mesurée que sur deux stratifiés »).
