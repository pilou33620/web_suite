# Essai : MIFA, croisement du développé visé `kd` × nombre de replis `n`

Qui : Claude in Chrome (PC Windows du boulot) · Durée : 12 points × 5 à 13 min (repère : le patch FR-4, 420 000 à 490 000 cellules par point) ; les points à `n` = 3 seront plus longs (brins à 0,16 mm) ; « ∑ Chiffrer la plage » fait foi — compte 1 h 30 à 3 h, plus 15 min de mise en place · Source : `web_antenna/A-FAIRE.md:321` (plan, point 1 ; constat l.304-311 ; limite README l.2038-2045)

## Pourquoi
Le MIFA résonne **au-dessus de 3,5 GHz pour 2,45 visés** (FR-4 1,53 mm, `A-FAIRE.md:297`). Son tracé pose un développé égal au quart d'onde, mais deux brins repliés trop proches se couplent et raccourcissent l'antenne électrique. Allonger l'empreinte efface le méandre (2,415 GHz à 17 mm, mais c'est un IFA droit). La bonne mesure : à empreinte fixe, allonger le **développé** (`kd`, en % du quart d'onde, 100 par défaut, `js/22-antennes.js:872` et `:902`), et voir combien il en faut selon l'écartement des brins.

Aux cotes par défaut (FR-4 1,53 mm, 2,45 GHz), d'après le tracé :

| `n` | pas du méandre | écartement des brins | écartement / largeur de bras (0,940 mm) | hauteur des dents à `kd` = 100 / 160 |
|---|---|---|---|---|
| 1 | 6,577 mm | 2,349 mm | 2,50 | 4,016 / 9,654 mm |
| 2 *(défaut)* | 3,289 mm | 0,705 mm | 0,75 | 2,008 / 4,827 mm |
| 3 | 2,192 mm | 0,157 mm | 0,17 | 1,339 / 3,218 mm |

`n` = 4 est exclu : écartement −0,117 mm (vérifié). Le résultat décide du **défaut de `kd`** : une loi `kd(écartement/largeur)` à la place de 100.

**Le seul écart aux relevés précédents, la bande** : le motif pose ±15 % (2,0825 à 2,8175 GHz), or le MIFA résonne au-dessus de 3,5 GHz et `kd` = 160 le fera descendre vers 1,5–2 GHz. La consigne pose **1,5 à 4,5 GHz, 601 points** (pas de 5 MHz), cible 2,45 GHz ; sans cela la moitié des points rendrait un minimum collé au bord de bande. Le reste est aux réglages par défaut des croisements du patch (boîte et maillage calculés, PML 8, règle des tiers, arrêt −40 dB, pas maximum 0, diagramme coché, port localisé 50 Ω). Le pas de maillage du balayage se fige sur le document **tel qu'il est au lancement** (`python/openems_modele.py:5563`) : on lance depuis les cotes par défaut.

## Ce que tu prépares
- Lance WEB·SUITE (double-clic sur `demarrer_WEB_SUITE.cmd`) ; le lanceur s'ouvre sur `http://127.0.0.1:8100`.
- openEMS : si la carte WEB_ANTENNA du lanceur montre **« ⚙ Installer openEMS »** (le bouton n'apparaît que quand l'outil est arrêté, et seulement sous Windows), clique-le et attends « openEMS installé : les simulations sont disponibles. » (il faut Python 3.10 ou 3.11 64 bits sur le poste ; sinon le **Journal** de la carte dit quoi faire). Puis **▶ Lancer** WEB_ANTENNA.
- Des heures de calcul : mets la veille Windows sur « Jamais » le temps de l'essai, laisse la session ouverte, et dans Chrome (Paramètres → Performances → Économiseur de mémoire) ajoute `127.0.0.1` aux sites toujours actifs : un onglet endormi perd le résultat.
- N'utilise pas WEB_ANTENNA à côté pendant l'essai (un seul calcul à la fois sur le poste).
- Colle la consigne dans Claude in Chrome. S'il s'arrête avant la fin (conversation trop longue), ouvre une nouvelle conversation et recolle la même consigne : elle dit comment reprendre.
- Le projet `essai-claude-mifa-kd-n` (et `-n3` s'il y a un prolongement) restera dans `PROJETS/ANTENNA` : la carte WEB_ANTENNA du lanceur proposera « ⇧ Envoyer sur GitHub ». Rien ne part tout seul (le lanceur ne pousse que des commits faits par ce bouton ou par « ☁ Enregistrer + GitHub »). À toi de choisir : l'envoyer me permet de contrôler la recopie avec `ESSAI-antenna-releve.py` (les `calculs/` ne partent pas, `.gitignore` de PROJETS) ; sinon supprime ces dossiers avant ton prochain envoi.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

ESSAI : ESSAI-antenna-mifa-kd-n.md — MIFA sur FR-4, croisement « développé visé » kd (100 à 160 %) × « nombre de replis » n (1 à 3) : 12 simulations openEMS, 1 h 30 à 3 h de calcul. Tu règles, tu lances, tu suis de loin, tu relèves les résultats dans la page.

RÈGLES DE CET ESSAI (en plus du fichier commun)
- Tout se passe dans UN onglet WEB_ANTENNA (http://127.0.0.1:8732, ou l'adresse de la carte WEB_ANTENNA du lanceur). Ne le recharge pas, ne le ferme pas, ne le fais pas naviguer ailleurs tant qu'un calcul tourne ou qu'un résultat n'est pas enregistré : le calcul continue sur le serveur, mais c'est la page qui reçoit son résultat. Pour le lanceur, prends un autre onglet.
- Projets : seulement essai-claude-mifa-kd-n (et essai-claude-mifa-kd-n3 si l'étape 11 le demande). Avant de nommer, regarde la liste « Reprendre un projet » du panneau « 📁 Projet » : si le nom y est déjà, ajoute « -2 » et dis-le. N'ouvre aucun autre projet.
- Interdit aussi : « ☁ Enregistrer + GitHub », « ⇧ Envoyer sur GitHub », « ■ Arrêter » et « ■ Arrêter la simulation », tout bouton « ⤓ » (téléchargements, « ⤓ Écrire le script Python »), le ✕ d'une couche, « Utiliser » et « 📂 » du « dossier de travail », les raccourcis de bande (868 MHz, 2,45 GHz…), « Exemple ».
- Nombres : tape les décimales avec un point. Après chaque réglage, J1 dit ce que la page a vraiment pris : c'est J1 qui fait foi.
- JavaScript : seulement J1 à J4 (en bas), copiées mot pour mot.
- Si une visite guidée (bulles « Suivant ») s'ouvre, ferme-la avec Échap.

ÉTAPES
1. Lanceur http://127.0.0.1:8100 : la carte WEB_ANTENNA dit « En cours sur … » (sinon « ▶ Lancer » et attends). Note les versions affichées. Ouvre WEB_ANTENNA dans un onglet neuf (« Ouvrir ↗ »).
   → attendu : en bas de la page, « openEMS : prêt ». Sinon arrête-toi (openEMS manque : c'est à l'utilisateur).
2. « ✏️ Concevoir » (barre du haut ; ou « ✏️ Concevoir une antenne » sur l'accueil). Bloc « Sur quelle carte ? » : « 2 couches », puis le modèle « 2 couches · FR-4 1,6 mm · 35 µm », puis « Empilage arrêté, dessiner ». (Si ce bloc n'apparaît pas, choisis ce modèle dans la liste « modèle d'usine ».) Ne retouche pas l'empilage.
3. Bloc « Partir d'un motif d'antenne » : « fréquence visée » 2.45, « en » GHz ; clique le motif « F inversé à méandres (MIFA) » (le recliquer referme sa fiche). Ne change aucune cote. Exécute J1.
   → attendu : motif "mifa", substrat h 1.53, er 4.3, df 0.02 ; cotes Lx 6.5772, n 2, wb 0.9396, kd 100 (les décimales au-delà peuvent différer d'un chiffre). Si ce n'est pas ça, ou si la fiche n'a pas de champ « kd développé visé », arrête-toi : WEB_ANTENNA n'est pas à jour.
4. « Dessiner ce motif ».
5. Assistant, étape « La bande » : « Début » 1.5, « Fin » 4.5 (« Unité » GHz), « Fréquence visée » 2.45, « Points de calcul » 601.
6. Étape « Le calcul » : coche « Balayer une cote » → « Cote » : « F inversé à méandres (MIFA) — développé visé » → « De » 100, « à » 160, « par pas de » 20. Coche « Croiser avec une seconde cote » → « Seconde cote » : « F inversé à méandres (MIFA) — nombre de replis » → « De » 1, « à » 3, « par pas de » 1. Laisse « Arrêt à l'énergie résiduelle » −40, « Pas de temps maximum » 0, « Calculer le diagramme de rayonnement » coché. Exécute J1.
   → attendu : bande f1 1500000000, f2 4500000000, n 601, fcible 2450000000 ; boite pml 8 ; maillage res_air 0, res_die 0, tiers true ; arret energie -40, nmax 0 ; diagramme true ; port R 50, msl false ; balayage de 100 a 160 pas 20, croise true, de2 1 a2 3 pas2 1, points 12. Le récapitulatif de la page dit « 12 simulations (4 × 3) ». Corrige tout écart et refais J1.
7. « 📁 Projet » (barre du haut) : « nom du projet » essai-claude-mifa-kd-n → « 💾 Enregistrer ».
   → attendu : avis « Projet « essai-claude-mifa-kd-n » enregistré ». Il faut l'avoir AVANT de lancer : c'est lui qui reçoit les dossiers de calcul.
8. Étape « Le calcul » : « ∑ Chiffrer la plage ».
   → attendu : pas de refus (tous les points sont vérifiés avant le premier). Recopie mot pour mot le récapitulatif (millions de cellules, durée, RAM, « le point le plus lourd : … cellules »). Un refus : recopie-le et arrête-toi.
9. « ▶ Lancer le balayage », puis tout de suite J2 : note l'« id » (dans tes notes et le rapport).
   → attendu : etat "calcule", point "1/12". Dans le panneau « Journal du solveur », une ligne par point fini : « → F inversé à méandres (MIFA) — développé visé = 100 × 1 % : resonance x.xxxx GHz, S11 -xx.xx dB ». À n fixé, la résonance doit descendre quand kd monte.
10. À la fin (J2 : etat "fini") : Ctrl+S tout de suite (J2 : non_enregistre false), puis J3, que tu recopies dans le rapport. Capture du panneau « Résultats », onglet « Balayage » (le tableau croisé).
11. Lis le relevé : pour chaque n, 2,45 GHz est-il entre deux kd (f0 au-dessus à un kd, au-dessous au suivant) ? Si à n = 3 même kd = 160 reste au-dessus de 2,45 GHz, prolonge :
    a. « 📁 Projet » : nom essai-claude-mifa-kd-n3 → « 💾 Enregistrer » (copie : le premier projet garde son résultat).
    b. Fiche du motif (panneau « Conception ») : « n nombre de replis » 3, rien d'autre → « Dessiner ce motif ». Étape « La bande » : remets 1.5 / 4.5 / 2.45 / 601 (dessiner remet ±15 %).
    c. Étape « Le calcul » : décoche « Croiser avec une seconde cote » ; « Cote » développé visé, « De » 180, « à » 220, « par pas de » 20. J1 → attendu : cotes n 3, bande comme à l'étape 6, balayage 180 à 220 pas 20, croise false, points 3.
    d. « ∑ Chiffrer la plage » (recopie), « ▶ Lancer le balayage », J2 (note l'id), suivi, à la fin Ctrl+S puis J3.

SUIVRE L'AVANCEMENT (des heures : ne reste pas devant)
- Toutes les 10 à 20 minutes : J2. etat "calcule" = ça tourne ; « point » dit lequel ; « faits » liste les points finis avec leur résonance. La même chose se lit dans l'assistant (« Avancement : … % », « Reste : ≈ … ») et dans le « Journal du solveur ».
- etat "fini" : Ctrl+S puis J3 (étape 10 ou 11d).
- etat "echoue" : recopie « detail » et la fin du « Journal du solveur », Ctrl+S, arrête-toi.
- etat "perdu" (« Suivi perdu » : la page n'a pas joint le serveur) : regarde la carte WEB_ANTENNA du lanceur dans un autre onglet. Si elle dit encore « En cours sur … », suis le calcul par J4 avec l'id noté. Sinon arrête-toi.
- Si tu ne peux pas attendre aussi longtemps, arrête-toi après le lancement en écrivant dans le rapport l'id, le projet et la prochaine étape : l'utilisateur recollera cette consigne dans une nouvelle conversation.

REPRENDRE DANS UNE NOUVELLE CONVERSATION
- Retrouve l'onglet WEB_ANTENNA déjà ouvert (ne le recharge pas, n'en ouvre pas un autre à sa place ; si tu ne l'atteins pas, demande à l'utilisateur de te le donner) et exécute J2 : « projet », « tache », et « essais » (« (résultats) » = un calcul enregistré dans ce projet) disent où tu en es. Reprends à l'étape qui suit.
- Un relevé J3 d'une conversation précédente que l'utilisateur ne t'a pas recopié : si J2 dit non_enregistre false, ouvre le projet concerné dans « Reprendre un projet » (son résultat revient avec lui) et refais J3. Avec non_enregistre true, Ctrl+S d'abord : sinon la page ouvre une fenêtre de confirmation qui te bloque.
- Onglet perdu (fermé, rechargé) : ne relance rien, le calcul continue sur le serveur. Ouvre un onglet neuf sur WEB_ANTENNA et suis-le par J4 avec l'id noté. Dis dans le rapport que ce résultat n'est pas dans le projet.

RAPPORT : le format commun, et en plus, avant CONSOLE :
IDENTIFIANTS : <projet> → <id> (pour chaque lancement)
DEVIS : <le récapitulatif de « ∑ Chiffrer la plage », mot pour mot, pour chaque lancement>
TABLEAU : une ligne par point, tirée de J3 : kd ; n ; f0 (GHz) ; S11 min (dB) ; S11 à 2,45 GHz (dB) ; Z au port (Re + j Im) ; rendement ; energie_fin_db ; diagnostic s'il y en a un
RELEVÉ :
--- kd x n (projet essai-claude-mifa-kd-n) ---
<la sortie de J3, recopiée telle quelle : pas de retouche, pas d'arrondi, pas de remise en forme>
--- kd n=3 prolonge (projet essai-claude-mifa-kd-n3) --- (seulement si l'étape 11 a eu lieu)
<idem>
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
- [ ] Les 12 points sont calculés : 12 points dans le relevé, aucun `diagnostic` (« aucune grandeur exploitable »).
- [ ] Chaque point a convergé : `energie_fin_db` ≤ −34 dB (seuil −40 dB ; le serveur ne s'en alarme qu'au-delà de 6 dB au-dessus, `python/openems_run.py:655`).
- [ ] Aucune résonance collée au bord de bande (f0 à plus de 2 % de 1,5 et de 4,5 GHz ; `bp_bord` jamais vrai).
- [ ] Pour au moins `n` = 1 et `n` = 2, 2,45 GHz est encadré par deux `kd`.
- [ ] Le relevé est dans le rapport, et `WEB_ANTENNA/croisement-mifa-kd-n.json` en est fait (moi, après).

## Ce qu'on en tirera
- Pour chaque `n`, une interpolation de `kd` où la résonance vaut 2,45 GHz : trois couples (écartement/largeur ; `kd`) = (2,50 ; ?), (0,75 ; ?), (0,17 ; ?).
- **Si `kd` croît quand l'écartement se resserre** (ce qu'on attend : plus de couplage, plus de développé à rattraper), la loi qui passe par ces trois points devient le défaut de `kd`, calculé depuis l'écartement — le défaut (`n` = 2) prend la valeur de 0,75.
- **Si les trois `kd` sont à peu près égaux**, ce n'est pas le couplage entre dents qui manque mais un facteur global (comme le bras × 1,018 de l'IFA) : `kd` devient une constante.
- Le rendement dira si un méandre serré (`n` = 3) vaut encore la peine : s'il tombe nettement sous celui de `n` = 1, la fiche du motif le dira.
- On ne lit pas la résonance d'un point dont le S₁₁ minimal reste au-dessus de −3 dB : ce n'est pas une résonance, je regarderai sa courbe.
- Si `n` = 3 diverge ou rend un court-circuit, c'est l'écart de 0,157 mm entre brins.

## Après
Pour Claude Code, à partir du rapport :
- Construire `web_antenna/croisement-mifa-kd-n.json` au format de `croisement-L-y0.json` (un cas) — ou, s'il y a eu le prolongement, une liste `cas` comme `croisement-patch-2026-10.json` (« kd x n » et « kd n=3 prolonge ») : `_note` = « MIFA kd x n, FR-4 1,53 mm, 2,45 GHz, bande 1,5-4,5 GHz 601 pts, reglages par defaut, arret -40 dB » ; `balayage` et `points` tels que J3 les rend ; `lignes` refaites depuis les points comme `_assembler` (`python/openems_run.py:973-987`). Si l'utilisateur a envoyé le projet sur GitHub, contrôler la recopie avec `tasks/essais/ESSAI-antenna-releve.py` sur `PROJETS/ANTENNA/essai-claude-mifa-kd-n/resultats.json`.
- [ ] `web_antenna/A-FAIRE.md:321-328` (plan, point 1) et `:304-311` (« Reste ouvert — le MIFA ») ; la ligne MIFA du tableau l.297 ; la limite l.372.
- [ ] `web_antenna/README.md:2038-2045` (« Un méandre serré sort du modèle »).
- [ ] `web_antenna/js/22-antennes.js:902` (`kd:100`) et `:872` (aide de la cote).
- [ ] `web_antenna/test/banc-interface.js:482-487` (le banc de `kd`).
