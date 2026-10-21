# Essai : patch FR-4 2,45 GHz, balayage fin de l'encastrement `y₀` (le reste réactif au pied)

Qui : Claude in Chrome (PC Windows du boulot) · Durée : 7 points × 5 à 6 min (le même document a tourné à 420 000–430 000 cellules, 46 et 56 min pour 9 points le 02/10/2026), compte 40 min à 1 h, plus 15 min de mise en place · Source : `web_antenna/A-FAIRE.md:339` (plan, point 4 ; origine des 15 Ω l.164-183 ; strate l.205-215)

> **À trancher avec toi avant de lancer : la plage.** La consigne porte une ligne `STATUT` : tant qu'elle ne dit pas `VALIDÉE`, Claude in Chrome prépare et vérifie le document mais **ne lance rien**. Pour valider, remplace dans la consigne (ici, et pousse) `STATUT : À VALIDER` par `STATUT : VALIDÉE le JJ/MM`, et `PLAGE : à choisir` par `PLAGE : proposée` ou `PLAGE : plan`.

## Pourquoi
La dixième simulation du 02/10/2026 (gabarit d'alors, `g` 1,49, `y₀` 8,51) laissait au pied du patch **57,9 + 15,5 j Ω** : un reste inductif « qu'un encastrement un peu plus court compenserait » (`A-FAIRE.md:173-183`). Le plan : balayer `y₀` finement sous le défaut corrigé, lire où Im Z au pied s'annule, et reporter le recul sur la strate FR-4 de `patchCorrRbord` (0,173 aujourd'hui, `js/22-antennes.js:231-238`).

**Ce que les relevés disent déjà, et qui change la plage** :
- Depuis le 03/10/2026, le gabarit corrigé pose **`y₀` = 5,9594 mm** (FR-4 1,6 mm, 2,45 GHz ; `L` 27,5856, `g` 1,4938). Les 15 Ω inductifs étaient ceux de `y₀` = 8,51 : ils sont déjà corrigés, et au-delà.
- Dans `croisement-L-y0.json`, Im Z **au pied** vaut −7,8 j à `y₀` = 6 et +4,0 j à `y₀` = 7 (`L` 27,7) ; −5,0 j et +6,4 j à `L` 27,2. Le point de confirmation (`L` 27,55, `y₀` 6,6) donne **+0,4 j**. Im Z s'annule donc vers **6,6 mm, au-dessus du défaut** : au défaut, le pied doit être **capacitif d'environ 8 Ω**, pas inductif.
- Le plan arrêté (`y₀` de −1,5 à 0 mm sous le défaut, soit 4,4594 → 5,9594) ne contiendrait donc **aucun** zéro : sept points tous capacitifs.
- **Je propose la même finesse, décalée** : de −0,5 à +1,0 mm autour du défaut, **5,4594 → 6,9594 par 0,25** (7 points), qui encadre 6,6. Si tu préfères tenir le plan, les valeurs sont 4,4594 → 5,9594 par 0,25 — la spec marche pareil.
- 6,6 mm correspond à une strate de **0,196** (calcul fait depuis le `G1` du gabarit) : c'est déjà la valeur à 2,45 GHz du tableau (0,151 / 0,196 / 0,173 à 868 MHz / 2,45 / 5,8 GHz, `A-FAIRE.md:205-207`). Ce que l'essai ajoute vraiment : **la mesure au défaut lui-même**, jamais simulé tel quel, et la pente de Im Z en ohms par millimètre autour de lui.

Document de départ, **le même que `croisement-patch-2026-10.json`** : FR-4 1,6 mm, **âme retouchée de 1,530 à 1,600 mm** (le « FR-4 1,6 mm d'âme » des relevés ; sans cette retouche les tableaux ne se lisent pas ensemble, `A-FAIRE.md:95-97`), 2,45 GHz, patch aux cotes par défaut. Réglages des croisements du patch : bande du motif 2,0825 à 2,8175 GHz, 401 points, boîte et maillage calculés, PML 8, règle des tiers, arrêt −40 dB, pas maximum 0, diagramme coché, port localisé 50 Ω avec la ligne déclarée par le motif (6,4 × 3,112 mm : c'est elle qui donne Z au pied).

## Ce que tu prépares
- **Valide la spec** (encadré ci-dessus) : sans cela Claude in Chrome s'arrête avant de lancer.
- Lance WEB·SUITE (double-clic sur `demarrer_WEB_SUITE.cmd`) ; le lanceur s'ouvre sur `http://127.0.0.1:8100`.
- openEMS : si la carte WEB_ANTENNA du lanceur montre **« ⚙ Installer openEMS »** (visible seulement outil arrêté, sous Windows), clique-le et attends « openEMS installé : les simulations sont disponibles. » (Python 3.10 ou 3.11 64 bits requis ; sinon le **Journal** de la carte le dit). Puis **▶ Lancer** WEB_ANTENNA.
- Veille Windows sur « Jamais » le temps de l'essai, et dans Chrome (Paramètres → Performances → Économiseur de mémoire) `127.0.0.1` parmi les sites toujours actifs.
- Colle la consigne dans Claude in Chrome ; s'il s'arrête avant la fin, recolle-la dans une nouvelle conversation : elle dit comment reprendre.
- Le projet `essai-claude-patch-y0-fin` restera dans `PROJETS/ANTENNA` ; le lanceur proposera « ⇧ Envoyer sur GitHub », rien ne part tout seul. L'envoyer me permet de contrôler la recopie avec `ESSAI-antenna-releve.py` ; sinon supprime-le avant ton prochain envoi.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

STATUT : À VALIDER
PLAGE : à choisir
(Si la ligne STATUT ne dit pas « VALIDÉE », fais seulement les étapes 1 à 3 — préparation et vérification, aucun projet, aucun calcul — puis arrête-toi et écris dans le rapport « spec non validée, rien lancé ». PLAGE « proposée » : de 5.4594 à 6.9594 ; « plan » : de 4.4594 à 5.9594 ; pas 0.25 dans les deux cas. Toute autre valeur : arrête-toi avant l'étape 4.)

ESSAI : ESSAI-antenna-patch-y0-fin.md — patch FR-4 à 2,45 GHz, balayage de l'encastrement y₀ en 7 points de 0,25 mm. 40 min à 1 h de calcul. Tu règles, tu lances, tu suis de loin, tu relèves les résultats dans la page.

RÈGLES DE CET ESSAI (en plus du fichier commun)
- Tout se passe dans UN onglet WEB_ANTENNA (http://127.0.0.1:8732, ou l'adresse de la carte WEB_ANTENNA du lanceur). Ne le recharge pas, ne le ferme pas, ne le fais pas naviguer ailleurs tant qu'un calcul tourne ou qu'un résultat n'est pas enregistré : le calcul continue sur le serveur, mais c'est la page qui reçoit son résultat. Pour le lanceur, prends un autre onglet.
- Projet : seulement essai-claude-patch-y0-fin. Avant de nommer, regarde la liste « Reprendre un projet » du panneau « 📁 Projet » : si le nom y est déjà, ajoute « -2 » et dis-le. N'ouvre aucun autre projet.
- Interdit aussi : « ☁ Enregistrer + GitHub », « ⇧ Envoyer sur GitHub », « ■ Arrêter » et « ■ Arrêter la simulation », tout bouton « ⤓ » (téléchargements, « ⤓ Écrire le script Python »), le ✕ d'une couche, « Utiliser » et « 📂 » du « dossier de travail », les raccourcis de bande, « Exemple ».
- Nombres : tape les décimales avec un point. Après chaque réglage, J1 dit ce que la page a vraiment pris : c'est J1 qui fait foi.
- JavaScript : seulement J1 à J4 (en bas), copiées mot pour mot.
- Si une visite guidée (bulles « Suivant ») s'ouvre, ferme-la avec Échap.

ÉTAPES
1. Lanceur http://127.0.0.1:8100 : la carte WEB_ANTENNA dit « En cours sur … » (sinon « ▶ Lancer » et attends). Note les versions affichées. Ouvre WEB_ANTENNA dans un onglet neuf (« Ouvrir ↗ »).
   → attendu : en bas de la page, « openEMS : prêt ». Sinon arrête-toi (openEMS manque : c'est à l'utilisateur).
2. « ✏️ Concevoir » (barre du haut ; ou « ✏️ Concevoir une antenne » sur l'accueil). Bloc « Sur quelle carte ? » : « 2 couches », puis le modèle « 2 couches · FR-4 1,6 mm · 35 µm », puis « Empilage arrêté, dessiner ». (Si ce bloc n'apparaît pas, choisis ce modèle dans la liste « modèle d'usine ».) Puis l'âme : dans le tableau de l'empilage, clique la ligne « Substrat » ; sa fiche « Diélectrique » s'ouvre ; champ « épaisseur » → 1.6, puis Tab. Ne touche ni εr ni tan δ.
3. Bloc « Partir d'un motif d'antenne » : « fréquence visée » 2.45, « en » GHz ; clique le motif « Patch rectangulaire » (le recliquer referme sa fiche). Ne change aucune cote. « Dessiner ce motif ». Exécute J1.
   → attendu : motif "patch" ; substrat h 1.6, er 4.3, df 0.02 ; cotes L 27.5856, W 37.5839, y0 5.9594, wf 3.112, g 1.4938, Lf 6.4, marge 4.8 (à un chiffre près sur la dernière décimale) ; bande f1 2082500000, f2 2817500000, n 401, fcible 2450000000 ; boite pml 8 ; maillage res_air 0, res_die 0, tiers true ; arret energie -40, nmax 0 ; diagramme true ; port R 50, msl false, ligne_d 6.4, ligne_w 3.112. Un écart sur les cotes : arrête-toi et dis-le. Sur les réglages (« Points de calcul » de l'étape « La bande », « Arrêt à l'énergie résiduelle » et « Pas de temps maximum » de l'étape « Le calcul », menu « Port » de l'étape « Les ports » sur « localisé ») : corrige et refais J1.
   — Spec non validée : arrête-toi ici.
4. « 📁 Projet » (barre du haut) : « nom du projet » essai-claude-patch-y0-fin → « 💾 Enregistrer ».
   → attendu : avis « Projet « essai-claude-patch-y0-fin » enregistré ». Il faut l'avoir AVANT de lancer : c'est lui qui reçoit les dossiers de calcul.
5. Assistant, étape « Le calcul » : coche « Balayer une cote » → « Cote » : « Patch rectangulaire — encastrement » → « De » et « à » selon la ligne PLAGE, « par pas de » 0.25. « Croiser avec une seconde cote » décoché. Exécute J1.
   → attendu : balayage cote « Patch rectangulaire — encastrement », de et a de la PLAGE, pas 0.25, croise false, points 7.
6. « ∑ Chiffrer la plage ».
   → attendu : pas de refus ; le point le plus lourd vers 420 000 cellules. Recopie mot pour mot le récapitulatif. Un refus : recopie-le et arrête-toi.
7. « ▶ Lancer le balayage », puis tout de suite J2 : note l'« id ».
   → attendu : etat "calcule", point "1/7". Une ligne par point fini dans le « Journal du solveur » ; la résonance bouge à peine (quelques MHz : c'est L qui fait la fréquence), le S11 passe par un minimum, et Im Z au pied croît avec y₀.
8. À la fin (J2 : etat "fini") : Ctrl+S tout de suite (J2 : non_enregistre false), puis J3, que tu recopies dans le rapport. Capture du panneau « Résultats », onglet « Balayage ». Si z0_pied_re et z0_pied_im manquent dans J3, la ligne n'était pas déclarée au port : dis-le.

SUIVRE L'AVANCEMENT
- Toutes les 10 minutes : J2. etat "calcule" = ça tourne ; « point » dit lequel ; « faits » liste les points finis avec leur résonance. La même chose se lit dans l'assistant (« Avancement : … % », « Reste : ≈ … ») et dans le « Journal du solveur ».
- etat "fini" : Ctrl+S puis J3 (étape 8).
- etat "echoue" : recopie « detail » et la fin du « Journal du solveur », Ctrl+S, arrête-toi.
- etat "perdu" (« Suivi perdu » : la page n'a pas joint le serveur) : regarde la carte WEB_ANTENNA du lanceur dans un autre onglet. Si elle dit encore « En cours sur … », suis le calcul par J4 avec l'id noté. Sinon arrête-toi.

REPRENDRE DANS UNE NOUVELLE CONVERSATION
- Retrouve l'onglet WEB_ANTENNA déjà ouvert (ne le recharge pas, n'en ouvre pas un autre à sa place ; si tu ne l'atteins pas, demande à l'utilisateur de te le donner) et exécute J2 : « projet », « tache », et « essais » (« (résultats) » = un calcul enregistré dans ce projet) disent où tu en es. Reprends à l'étape qui suit.
- Un relevé J3 d'une conversation précédente que l'utilisateur ne t'a pas recopié : si J2 dit non_enregistre false, ouvre essai-claude-patch-y0-fin dans « Reprendre un projet » (son résultat revient avec lui) et refais J3. Avec non_enregistre true, Ctrl+S d'abord : sinon la page ouvre une fenêtre de confirmation qui te bloque.
- Onglet perdu (fermé, rechargé) : ne relance rien, le calcul continue sur le serveur. Ouvre un onglet neuf sur WEB_ANTENNA et suis-le par J4 avec l'id noté. Dis dans le rapport que ce résultat n'est pas dans le projet.

RAPPORT : le format commun, et en plus, avant CONSOLE :
STATUT ET PLAGE : <les deux lignes telles que tu les as lues>
IDENTIFIANTS : essai-claude-patch-y0-fin → <id>
DEVIS : <le récapitulatif de « ∑ Chiffrer la plage », mot pour mot>
TABLEAU : une ligne par point (7), tirée de J3 : y₀ ; f0 (GHz) ; S11 min (dB) ; S11 à 2,45 GHz (dB) ; Z au port (Re + j Im) ; Z au pied (Re + j Im) ; energie_fin_db ; diagnostic s'il y en a un
RELEVÉ :
--- y0 fin (projet essai-claude-patch-y0-fin) ---
<la sortie de J3, recopiée telle quelle : pas de retouche, pas d'arrondi, pas de remise en forme>
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
- [ ] La spec était validée (ligne `STATUT`) et la plage lancée est celle de la ligne `PLAGE`.
- [ ] Les 7 points sont calculés, convergés : `energie_fin_db` ≤ −34 dB (seuil −40 dB, `python/openems_run.py:655`), aucun `diagnostic`.
- [ ] Le point `y₀` = 5,9594 (le défaut) est dans la plage, et Im Z au pied y est lu.
- [ ] Avec la plage proposée : Im Z au pied change de signe entre deux points.
- [ ] Le point le plus proche de 6,6 mm retombe près de la confirmation du 02/10/2026 (2,450 GHz, −39,9 dB, 50,9 + 0,4 j au pied) : c'est le témoin du document.
- [ ] Le relevé est dans le rapport, et `WEB_ANTENNA/balayage-patch-y0-fin.json` en est fait (moi, après), 7 points.

## Ce qu'on en tirera
- Le `y₀` où Im Z au pied s'annule, interpolé entre deux points, et ce qu'il donne de strate : `k = 2·G1·50 / cos²(π·y₀/L)` (je le calcule).
- **Si le zéro tombe vers 6,6 mm** (k ≈ 0,196) : la mesure confirme que la strate FR-4 moyenne (0,173) laisse ~8 Ω capacitifs au pied à 2,45 GHz. Monter la strate à 0,196 corrigerait 2,45 GHz mais dégraderait 868 MHz (0,151) : la décision est soit de garder la moyenne et d'écrire l'écart dans l'aide de la cote, soit de faire dépendre la strate de h/λ₀. Je ne corrige pas la formule sur ce seul relevé.
- **Si le zéro tombe ailleurs** (le défaut n'est pas où le croisement l'interpolait) : c'est `g` ou `L` du gabarit corrigé qui a déplacé l'adaptation, et je le noterai à côté de la loi de longueur.
- La pente de Im Z (Ω/mm) dit quelle tolérance tenir sur `y₀` en fabrication.

## Après
Pour Claude Code, à partir du rapport :
- Construire `web_antenna/balayage-patch-y0-fin.json` au format de `croisement-L-y0.json` (un cas) : `_note` = « Patch gabarit corrige, FR-4 1,6 mm d'ame, 2,45 GHz, y0 de 5,4594 a 6,9594 par 0,25 (ou plage du plan, selon la ligne PLAGE), memes reglages que croisement-patch-2026-10.json » ; `balayage` et `points` tels que J3 les rend ; `lignes` refaites depuis les points comme `_assembler` (`python/openems_run.py:973-987`). Si l'utilisateur a envoyé le projet sur GitHub, contrôler la recopie avec `tasks/essais/ESSAI-antenna-releve.py` sur `PROJETS/ANTENNA/essai-claude-patch-y0-fin/resultats.json`.
- [ ] `web_antenna/A-FAIRE.md:339-343` (plan, point 4 ; la plage s'y corrige si la proposée a été validée), `:164-183` (les 15 Ω), `:205-215` (les strates), la limite l.371.
- [ ] `web_antenna/js/22-antennes.js:214-238` (`patchCorrRbord` et son commentaire) et l'aide de `y₀` l.251-255.
