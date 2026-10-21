# Essai : les facteurs du dipôle, du monopôle et de l'IFA tiennent-ils hors de leur cas ?

Qui : Claude in Chrome (PC Windows du boulot) · Durée : 6 simulations seules × 5 à 15 min (repère : le patch FR-4 à 2,45 GHz, 420 000 à 490 000 cellules ; à 5,8 GHz la boîte est plus petite mais la maille plus fine), le bilan de chaque calcul fait foi — compte 1 h à 1 h 30, plus 20 min de mise en place · Source : `web_antenna/A-FAIRE.md:334` (plan, point 3 ; constat l.299-302)

## Pourquoi
Les trois facteurs mesurés le 03/10/2026 viennent d'**un seul cas chacun**, FR-4 1,53 mm à 2,45 GHz (`A-FAIRE.md:299-302`) :

| motif | facteur | confirmé à 2,45 GHz, FR-4 1,53 mm | code |
|---|---|---|---|
| dipôle | bras × 1,165 du quart d'onde | 2,461 GHz · −34,3 dB · 48 Ω | `js/22-antennes.js:1046-1052` |
| monopôle | brin × 1,29, masse à 0,8 × brin | 2,435 GHz · −16,3 dB · +16j | `js/22-antennes.js:415-425` |
| IFA | bras × 1,018, écart × 1,8 | 2,48 GHz · bande 2,405–2,575 GHz | `js/22-antennes.js:613-620` |

Deux confirmations par motif, **aux cotes par défaut, sans croisement** : 5,8 GHz sur le même FR-4 (la fréquence change), 2,45 GHz sur RO4003C 0,813 mm (le substrat change). Le résultat décide, motif par motif : facteur confirmé, ou croisement à mener à cette fréquence.

**Écart à ta consigne, je suis le plan** : tu m'as écrit « 5,8 GHz sur FR-4 et RO4003C » ; `A-FAIRE.md:335-336` dit **5,8 GHz sur FR-4 1,53 mm** et **2,45 GHz sur RO4003C 0,813 mm**. C'est ce que fait cette spec — dis-le-moi si tu voulais le RO4003C à 5,8 GHz.

Les deux empilages : **A** = « 2 couches · FR-4 1,6 mm · 35 µm » tel quel (âme 1,530 mm, εr 4,30, tan δ 0,020), 5,8 GHz ; **B** = « 2 couches · RO4003C 1,524 mm (60 mil) », **âme retouchée à 0,813 mm** (aucun modèle d'usine ne la donne ; εr 3,55, tan δ 0,0027 restent), 2,45 GHz. Réglages identiques aux mesures du 03/10/2026 : la bande que pose le motif (±15 % : 4,93 à 6,67 GHz pour A, 2,0825 à 2,8175 GHz pour B, 401 points), boîte et maillage calculés, PML 8, règle des tiers, arrêt −40 dB, pas maximum 0, diagramme coché, port localisé 50 Ω. Les cotes attendues (calculées le 21/10/2026 par le code du dépôt) :

| point | motif | cotes par défaut | résonance annoncée par la fiche |
|---|---|---|---|
| A1 | Dipôle imprimé | `La` 9,2477 · `wd` 1 · `ov` 1,53 · marge 6,12 | 5,8000 GHz |
| A2 | Monopôle imprimé | `Lm` 10,24 · `wr` 5,952 · `Lg` **10** · `wf` 2,976 · `Lb` 24,312 | 5,8000 GHz |
| A3 | F inversé (IFA) | `La` 8,877 · `ha` 2,58 · `wb` 0,6 · `d` 2,322 · `Lg` 25 · `Lb` 30 | 5,7981 GHz |
| B1 | Dipôle imprimé | `La` 23,6282 · `wd` 1,6901 · `ov` 0,813 | 2,4500 GHz |
| B2 | Monopôle imprimé | `Lm` 26,1634 · `wr` 3,638 · `Lg` 20,9307 · `wf` 1,819 · `Lb` 20 | 2,4500 GHz |
| B3 | F inversé (IFA) | `La` 22,009 · `ha` 6,12 · `wb` 1,22 · `d` 5,508 · `Lg` 30,59 · `Lb` 30 | 2,4496 GHz |

## Ce que tu prépares
- Lance WEB·SUITE (double-clic sur `demarrer_WEB_SUITE.cmd`) ; le lanceur s'ouvre sur `http://127.0.0.1:8100`.
- openEMS : si la carte WEB_ANTENNA du lanceur montre **« ⚙ Installer openEMS »** (visible seulement outil arrêté, sous Windows), clique-le et attends « openEMS installé : les simulations sont disponibles. » (Python 3.10 ou 3.11 64 bits requis ; sinon le **Journal** de la carte le dit). Puis **▶ Lancer** WEB_ANTENNA.
- Plus d'une heure de calcul : veille Windows sur « Jamais » le temps de l'essai, session ouverte, et dans Chrome (Paramètres → Performances → Économiseur de mémoire) `127.0.0.1` parmi les sites toujours actifs.
- N'utilise pas WEB_ANTENNA à côté pendant l'essai.
- Colle la consigne dans Claude in Chrome ; s'il s'arrête avant la fin, recolle-la dans une nouvelle conversation : elle dit comment reprendre.
- Les six projets `essai-claude-facteurs-*` resteront dans `PROJETS/ANTENNA` ; le lanceur proposera « ⇧ Envoyer sur GitHub », rien ne part tout seul. Les envoyer me permet de contrôler la recopie avec `ESSAI-antenna-releve.py` ; sinon supprime-les avant ton prochain envoi.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

ESSAI : ESSAI-antenna-facteurs-5g8.md — six simulations openEMS seules (dipôle, monopôle, IFA ; sur FR-4 à 5,8 GHz puis sur RO4003C 0,813 mm à 2,45 GHz), aux cotes par défaut, un projet par simulation. 1 h à 1 h 30 de calcul. Tu règles, tu lances, tu suis de loin, tu relèves chaque résultat dans la page avant de lancer le suivant.

RÈGLES DE CET ESSAI (en plus du fichier commun)
- Tout se passe dans UN onglet WEB_ANTENNA (http://127.0.0.1:8732, ou l'adresse de la carte WEB_ANTENNA du lanceur). Ne le recharge pas, ne le ferme pas, ne le fais pas naviguer ailleurs tant qu'un calcul tourne ou qu'un résultat n'est pas enregistré : le calcul continue sur le serveur, mais c'est la page qui reçoit son résultat. Pour le lanceur, prends un autre onglet.
- Projets : seulement les six nommés au tableau ci-dessous. Avant de nommer, regarde la liste « Reprendre un projet » du panneau « 📁 Projet » : si le nom y est déjà, ajoute « -2 » et dis-le. N'ouvre aucun autre projet.
- Interdit aussi : « ☁ Enregistrer + GitHub », « ⇧ Envoyer sur GitHub », « ■ Arrêter » et « ■ Arrêter la simulation », tout bouton « ⤓ » (téléchargements, « ⤓ Écrire le script Python »), le ✕ d'une couche, « Utiliser » et « 📂 » du « dossier de travail », les raccourcis de bande (868 MHz, 5,8 GHz…), « Exemple », « Balayer une cote » (il doit rester décoché).
- Nombres : tape les décimales avec un point. Après chaque réglage, J1 dit ce que la page a vraiment pris : c'est J1 qui fait foi.
- JavaScript : seulement J1 à J4 (en bas), copiées mot pour mot.
- Si une visite guidée (bulles « Suivant ») s'ouvre, ferme-la avec Échap.

LES SIX CALCULS (dans cet ordre)
| cas | empilage | fréquence visée | motif | projet | cotes attendues dans J1 (à un chiffre près sur la dernière décimale) |
| A1 | A | 5.8 | « Dipôle imprimé » | essai-claude-facteurs-dipole-5g8 | La 9.2477, wd 1, ov 1.53, marge 6.12 |
| A2 | A | 5.8 | « Monopôle imprimé » | essai-claude-facteurs-monopole-5g8 | Lm 10.24, wr 5.952, Lg 10, wf 2.976, Lb 24.312 |
| A3 | A | 5.8 | « F inversé (IFA) » | essai-claude-facteurs-ifa-5g8 | La 8.877, ha 2.58, wb 0.6, d 2.322, Lg 25, Lb 30 |
| B1 | B | 2.45 | « Dipôle imprimé » | essai-claude-facteurs-dipole-ro4003 | La 23.6282, wd 1.6901, ov 0.813 |
| B2 | B | 2.45 | « Monopôle imprimé » | essai-claude-facteurs-monopole-ro4003 | Lm 26.1634, wr 3.638, Lg 20.9307, wf 1.819, Lb 20 |
| B3 | B | 2.45 | « F inversé (IFA) » | essai-claude-facteurs-ifa-ro4003 | La 22.009, ha 6.12, wb 1.22, d 5.508, Lg 30.59, Lb 30 |
Empilage A : substrat attendu dans J1 h 1.53, er 4.3, df 0.02. Empilage B : h 0.813, er 3.55, df 0.0027.

ÉTAPES
1. Lanceur http://127.0.0.1:8100 : la carte WEB_ANTENNA dit « En cours sur … » (sinon « ▶ Lancer » et attends). Note les versions affichées. Ouvre WEB_ANTENNA dans un onglet neuf (« Ouvrir ↗ »).
   → attendu : en bas de la page, « openEMS : prêt ». Sinon arrête-toi (openEMS manque : c'est à l'utilisateur).
2. Empilage A : « ✏️ Concevoir » (barre du haut ; ou « ✏️ Concevoir une antenne » sur l'accueil). Bloc « Sur quelle carte ? » : « 2 couches », puis le modèle « 2 couches · FR-4 1,6 mm · 35 µm », puis « Empilage arrêté, dessiner ». (Si ce bloc n'apparaît pas, choisis ce modèle dans la liste « modèle d'usine ».)
3. Pour chaque cas, dans l'ordre du tableau :
   a. Bloc « Partir d'un motif d'antenne » : « fréquence visée » du tableau, « en » GHz ; clique le motif (s'il est déjà ouvert, sa fiche est là ; le recliquer la referme). Ne change aucune cote. Exécute J1.
      → attendu : le substrat de l'empilage et les cotes du tableau. Un écart sur une cote : arrête-toi et dis-le.
   b. « Dessiner ce motif ». Exécute J1.
      → attendu : bande f1 = 0,85 × et f2 = 1,15 × la fréquence visée (A : 4930000000 et 6670000000 ; B : 2082500000 et 2817500000), n 401, fcible = la fréquence visée ; boite pml 8 ; maillage res_air 0, res_die 0, tiers true ; arret energie -40, nmax 0 ; diagramme true ; port R 50, msl false ; balayage "non". Sinon corrige (« Points de calcul » de l'étape « La bande », « Arrêt à l'énergie résiduelle » et « Pas de temps maximum » de l'étape « Le calcul », « Calculer le diagramme de rayonnement ») et refais J1.
   c. « 📁 Projet » (barre du haut) : « nom du projet » = le projet du cas → « 💾 Enregistrer ».
      → attendu : avis « Projet « … » enregistré ». Il faut l'avoir AVANT de lancer : c'est lui qui reçoit le dossier de calcul.
   d. Assistant, étape « Le calcul » : « ▶ Lancer openEMS », puis tout de suite J2 : note l'« id ». S'il y a un refus avant le lancement, ou une alerte « Ce calcul prendra un moment », recopie-les mot pour mot.
   e. Suivi (voir plus bas). À la fin (J2 : etat "fini") : Ctrl+S tout de suite (J2 : non_enregistre false), puis J3, que tu recopies dans le rapport sous le nom du cas.
      → attendu : une résonance lisible, S11 minimal sous −6 dB, loin du bord de bande. Si c'est l'IFA et qu'il n'y a aucune résonance avec un rendement autour de 25 % (0.25), fais une capture de la vue 2D autour du port.
4. Avant B1 : passe à l'empilage B. Panneau « Conception », bloc « L'empilage et les matériaux » : liste « modèle d'usine » → « 2 couches · RO4003C 1,524 mm (60 mil) ». Dans le tableau de l'empilage, clique la ligne « Substrat » : sa fiche « Diélectrique » s'ouvre ; champ « épaisseur » → 0.813, puis Tab. Ne touche ni εr ni tan δ. Puis reprends l'étape 3 pour B1, B2, B3 (J1 doit montrer h 0.813, er 3.55, df 0.0027).

SUIVRE L'AVANCEMENT
- Toutes les 5 à 10 minutes : J2. etat "calcule" = ça tourne. La même chose se lit dans l'assistant (« Avancement : … % », « Reste : ≈ … ») et dans le « Journal du solveur ».
- etat "fini" : Ctrl+S puis J3 (étape 3e), puis cas suivant.
- etat "echoue" : recopie « detail » et la fin du « Journal du solveur » (diagnostic d'énergie compris), Ctrl+S, et passe au cas suivant.
- etat "perdu" (« Suivi perdu » : la page n'a pas joint le serveur) : regarde la carte WEB_ANTENNA du lanceur dans un autre onglet. Si elle dit encore « En cours sur … », suis le calcul par J4 avec l'id noté. Sinon arrête-toi.
- Si tu ne peux pas aller jusqu'au bout, arrête-toi après un relevé en écrivant dans le rapport le dernier cas fait et l'id du calcul en cours s'il y en a un : l'utilisateur recollera cette consigne dans une nouvelle conversation.

REPRENDRE DANS UNE NOUVELLE CONVERSATION
- Retrouve l'onglet WEB_ANTENNA déjà ouvert (ne le recharge pas, n'en ouvre pas un autre à sa place ; si tu ne l'atteins pas, demande à l'utilisateur de te le donner) et exécute J2 : « projet », « tache », et « essais » (« (résultats) » = un calcul enregistré dans ce projet) disent quels cas sont faits. Reprends au cas suivant.
- Les relevés J3 d'une conversation précédente que l'utilisateur ne t'a pas recopiés : pour chaque cas fait, si J2 dit non_enregistre false, ouvre son projet dans « Reprendre un projet » (son résultat revient avec lui) et refais J3. Avec non_enregistre true, Ctrl+S d'abord : sinon la page ouvre une fenêtre de confirmation qui te bloque. Puis, pour le cas suivant, pars de l'étape 3a (et, pour un cas B, vérifie d'abord par J1 que le substrat est h 0.813, er 3.55 ; sinon refais l'étape 4).
- Onglet perdu (fermé, rechargé) : ne relance rien, le calcul continue sur le serveur. Ouvre un onglet neuf sur WEB_ANTENNA et suis-le par J4 avec l'id noté. Dis dans le rapport que ce résultat n'est pas dans le projet.

RAPPORT : le format commun, et en plus, avant CONSOLE :
IDENTIFIANTS : <projet> → <id> (pour chaque cas)
TABLEAU : une ligne par cas, tirée de J3 : cas ; f0 (GHz) ; écart à la cible (%) ; S11 min (dB) ; S11 à la cible (dB) ; Z au port (Re + j Im) ; bande −10 dB (bp_f1 – bp_f2, bp_bord) ; rendement ; energie_fin_db ; diagnostic s'il y en a un
RELEVÉ :
--- dipole FR-4 1,53 mm 5,8 GHz (projet essai-claude-facteurs-dipole-5g8) ---
<la sortie de J3, recopiée telle quelle : pas de retouche, pas d'arrondi, pas de remise en forme>
--- monopole FR-4 1,53 mm 5,8 GHz (projet essai-claude-facteurs-monopole-5g8) ---
--- IFA FR-4 1,53 mm 5,8 GHz (projet essai-claude-facteurs-ifa-5g8) ---
--- dipole RO4003C 0,813 mm 2,45 GHz (projet essai-claude-facteurs-dipole-ro4003) ---
--- monopole RO4003C 0,813 mm 2,45 GHz (projet essai-claude-facteurs-monopole-ro4003) ---
--- IFA RO4003C 0,813 mm 2,45 GHz (projet essai-claude-facteurs-ifa-ro4003) ---
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
   etat "fini" : relevé par J3 en remplaçant sa fin (ANT.resultat,ANT.bande.fcible) par (window.RELEVE,5.8e9) pour un cas A, (window.RELEVE,2.45e9) pour un cas B. « Cette simulation n'existe plus » : le serveur a redémarré ou le calcul est fini depuis plus de 12 h ; arrête-toi.
```

## C'est réussi si
- [ ] Les six calculs ont rendu une résonance (aucun `diagnostic`, S₁₁ min sous −6 dB), convergés : `energie_fin_db` ≤ −34 dB (seuil −40 dB, `python/openems_run.py:655`).
- [ ] Aucune résonance collée au bord de bande (f0 à plus de 2 % des bords ; si oui, c'est déjà un écart de plus de 15 % : noté, pas relancé).
- [ ] Les cotes vues au motif sont celles du tableau.
- [ ] Les six relevés sont dans le rapport, et `WEB_ANTENNA/validation-facteurs.json` en est fait (moi, après), 6 cas.

## Ce qu'on en tirera
- Écart de résonance à la cible, motif par motif : **sous 2 %** (±116 MHz à 5,8 GHz, ±49 MHz à 2,45 GHz), le facteur est confirmé et son `ponytail:` « UN SEUL cas mesuré » devient « trois cas » ; **au-delà**, on mène le croisement « cote de résonance × cote d'adaptation » de ce motif à cette fréquence, comme le 03/10/2026 (pour le monopôle, c'était brin × masse, `Lm` × `Lg`).
- **Une prévision à vérifier sur A2** : à 5,8 GHz, la masse du monopôle n'est pas à 0,8 × brin (8,19 mm) mais au plancher de **10 mm** (`js/22-antennes.js:425`). Si la loi mesurée `Lm + 0,43·Lg ≈ cste` suit la longueur d'onde, l'antenne est électriquement 5,6 % trop longue et résonnera vers **5,49 GHz** (−5,4 %). Si c'est ce qu'on voit, ce n'est pas le facteur 1,29 qui est en cause mais le plancher : la correction portera sur `Lm` en fonction de `Lg`, pas sur le 1,29.
- Une dérive qui suit le **substrat** (B) et pas la fréquence (A) dira que le facteur dépend de l'εr, donc de `eeffAir = (εr+1)/2` : c'est elle qu'il faudrait corriger, pour les quatre motifs à la fois.
- L'adaptation (S₁₁, Z) se lit en second : le plan ne juge que la résonance.
- Pour l'IFA : aucune résonance et un rendement autour de 25 %, c'est le court-circuit d'alimentation du 03/10/2026 qui revient (la capture de la vue 2D autour du port le dira).

## Après
Pour Claude Code, à partir du rapport :
- Construire `web_antenna/validation-facteurs.json`, liste `cas` comme `croisement-patch-2026-10.json` : `_note` = « Facteurs dipole/monopole/IFA aux cotes par defaut, sans croisement. A = FR-4 1,53 mm 5,8 GHz, B = RO4003C 0,813 mm 2,45 GHz. Bande +-15 %, 401 pts, arret -40 dB » ; six cas nommés comme les blocs du RELEVÉ, chacun avec les nombres de J3. Si l'utilisateur a envoyé les projets sur GitHub, contrôler la recopie avec `tasks/essais/ESSAI-antenna-releve.py` sur leurs `resultats.json`.
- [ ] `web_antenna/A-FAIRE.md:334-338` (plan, point 3), `:299-302` (« des cas uniques »), le tableau l.292-297 et la limite l.370.
- [ ] `web_antenna/js/22-antennes.js:1046-1052` (dipôle), `:415-425` (monopôle), `:613-620` (IFA), chacun avec son `ponytail:`.
