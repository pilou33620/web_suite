# Essai : monopôle, croisement largeur du brin `wr` × longueur `Lm` (la réactance de +16j)

Qui : Claude in Chrome (PC Windows du boulot) · Durée : 12 points (un croisement de 9 + un balayage de 3) × 5 à 13 min (repère : le patch FR-4, 420 000 à 490 000 cellules par point), « ∑ Chiffrer la plage » fait foi — compte 1 h à 2 h 30, plus 15 min de mise en place · Source : `web_antenna/A-FAIRE.md:329` (plan, point 2 ; constat l.294 et l.310-311)

## Pourquoi
Corrigé le 03/10/2026 (brin × 1,29, masse à 0,8 × brin), le monopôle résonne à **2,435 GHz, −16,3 dB, mais avec une réactance de +16j qui reste** (`A-FAIRE.md:294`). Élargir le brin réduit l'inductance — et abaisse aussi la résonance, d'où le croisement avec `Lm`. Le résultat décide du **défaut de `wr`** : le rapport au défaut actuel où Im Z ≈ 0 à la résonance.

Défauts sur FR-4 1,53 mm à 2,45 GHz (`js/22-antennes.js:411-428`) : `Lm` 24,2416 · `wr` 5,952 (= 2 × `wf`) · `Lg` 19,3933 · `wf` 2,976 · `Lb` 24,312 · marge 4,59 mm. Les valeurs du plan :

| | ×0,6 | ×1 | ×1,5 | ×2 |
|---|---|---|---|---|
| `wr` | 3,5712 | 5,952 | 8,928 | 11,904 mm |

| | −3 % | 0 | +3 % |
|---|---|---|---|
| `Lm` | 23,5144 | 24,2416 | 24,9688 mm |

**Pourquoi deux lancements** : le balayage prend « de … à … par pas de … », et 0,6 / 1 / 1,5 / 2 ne sont pas à pas constant. On fait donc un croisement `wr` = ×1 / ×1,5 / ×2 (pas de 2,976 mm) × `Lm` (9 points), puis `wr` = ×0,6 posé à la main et un balayage de `Lm` seul (3 points). Le pas de maillage se fige sur le document au lancement (`python/openems_modele.py:5563`) ; le cuivre le plus étroit reste la ligne (2,976 mm) même à `wr` = 3,5712 : le pas est le même dans les deux lancements. `Lb` reste à 24,312 mm sur tous les points (à `wr` = 11,904 il reste 6,2 mm de substrat de chaque côté du brin) : seule la cote balayée change. Bande posée par le motif, 2,0825 à 2,8175 GHz (±15 %), 401 points ; le reste aux réglages par défaut des croisements du patch.

## Ce que tu prépares
- Lance WEB·SUITE (double-clic sur `demarrer_WEB_SUITE.cmd`) ; le lanceur s'ouvre sur `http://127.0.0.1:8100`.
- openEMS : si la carte WEB_ANTENNA du lanceur montre **« ⚙ Installer openEMS »** (visible seulement outil arrêté, sous Windows), clique-le et attends « openEMS installé : les simulations sont disponibles. » (Python 3.10 ou 3.11 64 bits requis ; sinon le **Journal** de la carte le dit). Puis **▶ Lancer** WEB_ANTENNA.
- Des heures de calcul : veille Windows sur « Jamais » le temps de l'essai, session ouverte, et dans Chrome (Paramètres → Performances → Économiseur de mémoire) `127.0.0.1` parmi les sites toujours actifs.
- N'utilise pas WEB_ANTENNA à côté pendant l'essai.
- Colle la consigne dans Claude in Chrome ; s'il s'arrête avant la fin, recolle-la dans une nouvelle conversation : elle dit comment reprendre.
- Les projets `essai-claude-monopole-wr-lm` et `essai-claude-monopole-wr06` resteront dans `PROJETS/ANTENNA` ; le lanceur proposera « ⇧ Envoyer sur GitHub », rien ne part tout seul. Les envoyer me permet de contrôler la recopie avec `ESSAI-antenna-releve.py` ; sinon supprime-les avant ton prochain envoi.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

ESSAI : ESSAI-antenna-monopole-wr-lm.md — monopôle imprimé sur FR-4 à 2,45 GHz : croisement largeur du brin wr × longueur du brin Lm (9 simulations), puis wr = 3,5712 × Lm (3 simulations). 1 h à 2 h 30 de calcul. Tu règles, tu lances, tu suis de loin, tu relèves les résultats dans la page.

RÈGLES DE CET ESSAI (en plus du fichier commun)
- Tout se passe dans UN onglet WEB_ANTENNA (http://127.0.0.1:8732, ou l'adresse de la carte WEB_ANTENNA du lanceur). Ne le recharge pas, ne le ferme pas, ne le fais pas naviguer ailleurs tant qu'un calcul tourne ou qu'un résultat n'est pas enregistré : le calcul continue sur le serveur, mais c'est la page qui reçoit son résultat. Pour le lanceur, prends un autre onglet.
- Projets : seulement essai-claude-monopole-wr-lm et essai-claude-monopole-wr06. Avant de nommer, regarde la liste « Reprendre un projet » du panneau « 📁 Projet » : si le nom y est déjà, ajoute « -2 » et dis-le. N'ouvre aucun autre projet.
- Interdit aussi : « ☁ Enregistrer + GitHub », « ⇧ Envoyer sur GitHub », « ■ Arrêter » et « ■ Arrêter la simulation », tout bouton « ⤓ » (téléchargements, « ⤓ Écrire le script Python »), le ✕ d'une couche, « Utiliser » et « 📂 » du « dossier de travail », les raccourcis de bande (868 MHz, 2,45 GHz…), « Exemple ».
- Nombres : tape les décimales avec un point. Après chaque réglage, J1 dit ce que la page a vraiment pris : c'est J1 qui fait foi. (Un champ peut se colorer pour une valeur hors de sa grille de 0,05 : la valeur est quand même prise, J1 le montre.)
- JavaScript : seulement J1 à J4 (en bas), copiées mot pour mot.
- Si une visite guidée (bulles « Suivant ») s'ouvre, ferme-la avec Échap.

ÉTAPES
1. Lanceur http://127.0.0.1:8100 : la carte WEB_ANTENNA dit « En cours sur … » (sinon « ▶ Lancer » et attends). Note les versions affichées. Ouvre WEB_ANTENNA dans un onglet neuf (« Ouvrir ↗ »).
   → attendu : en bas de la page, « openEMS : prêt ». Sinon arrête-toi (openEMS manque : c'est à l'utilisateur).
2. « ✏️ Concevoir » (barre du haut ; ou « ✏️ Concevoir une antenne » sur l'accueil). Bloc « Sur quelle carte ? » : « 2 couches », puis le modèle « 2 couches · FR-4 1,6 mm · 35 µm », puis « Empilage arrêté, dessiner ». (Si ce bloc n'apparaît pas, choisis ce modèle dans la liste « modèle d'usine ».) Ne retouche pas l'empilage.
3. Bloc « Partir d'un motif d'antenne » : « fréquence visée » 2.45, « en » GHz ; clique le motif « Monopôle imprimé » (le recliquer referme sa fiche). Ne change aucune cote. « Dessiner ce motif ». Exécute J1.
   → attendu : motif "monopole", substrat h 1.53, er 4.3, df 0.02 ; cotes Lm 24.2416, wr 5.952, Lg 19.3933, wf 2.976, Lb 24.312, marge 4.59 (à un chiffre près sur la dernière décimale) ; bande f1 2082500000, f2 2817500000, n 401, fcible 2450000000 ; boite pml 8 ; maillage res_air 0, res_die 0, tiers true ; arret energie -40, nmax 0 ; diagramme true ; port R 50, msl false. Un écart sur les cotes : arrête-toi et dis-le. Sur les réglages (« Points de calcul » de l'étape « La bande », « Arrêt à l'énergie résiduelle » et « Pas de temps maximum » de l'étape « Le calcul », « Calculer le diagramme de rayonnement ») : corrige et refais J1.
4. « 📁 Projet » (barre du haut) : « nom du projet » essai-claude-monopole-wr-lm → « 💾 Enregistrer ».
   → attendu : avis « Projet « essai-claude-monopole-wr-lm » enregistré ». Il faut l'avoir AVANT de lancer : c'est lui qui reçoit les dossiers de calcul.
5. Assistant, étape « Le calcul » : coche « Balayer une cote » → « Cote » : « Monopôle imprimé — largeur du brin » → « De » 5.952, « à » 11.904, « par pas de » 2.976. Coche « Croiser avec une seconde cote » → « Seconde cote » : « Monopôle imprimé — longueur du brin » → « De » 23.5144, « à » 24.9688, « par pas de » 0.7272. Exécute J1.
   → attendu : balayage de 5.952 a 11.904 pas 2.976, croise true, de2 23.5144 a2 24.9688 pas2 0.7272, points 9 ; récapitulatif « 9 simulations (3 × 3) ».
6. « ∑ Chiffrer la plage ».
   → attendu : pas de refus. Recopie mot pour mot le récapitulatif (millions de cellules, durée, RAM, « le point le plus lourd : … cellules »). Un refus : recopie-le et arrête-toi.
7. « ▶ Lancer le balayage », puis tout de suite J2 : note l'« id ».
   → attendu : etat "calcule", point "1/9". Une ligne par point fini dans le « Journal du solveur » (« → Monopôle imprimé — largeur du brin = … »). À Lm fixé, la résonance doit descendre quand wr monte.
8. À la fin (J2 : etat "fini") : Ctrl+S tout de suite (J2 : non_enregistre false), puis J3, que tu recopies dans le rapport. Capture du panneau « Résultats », onglet « Balayage ».
9. Second lancement. « 📁 Projet » : nom essai-claude-monopole-wr06 → « 💾 Enregistrer » (copie : le premier projet garde son résultat). Panneau « Conception », fiche du motif : « wr largeur du brin » 3.5712, rien d'autre → « Dessiner ce motif ». Étape « Le calcul » : décoche « Croiser avec une seconde cote » ; « Cote » : « Monopôle imprimé — longueur du brin » → « De » 23.5144, « à » 24.9688, « par pas de » 0.7272. Exécute J1.
   → attendu : projet "essai-claude-monopole-wr06" ; cotes wr 3.5712, Lm 24.2416, Lb 24.312 ; bande et réglages comme à l'étape 3 ; balayage cote « Monopôle imprimé — longueur du brin », de 23.5144 a 24.9688 pas 0.7272, croise false, points 3. Si la liste « Cote » ne propose plus « Monopôle imprimé — … », le motif a été retouché à la main : redessine-le depuis sa fiche.
10. « ∑ Chiffrer la plage » (recopie), « ▶ Lancer le balayage », J2 (note l'id), suivi ; à la fin Ctrl+S puis J3.

SUIVRE L'AVANCEMENT (des heures : ne reste pas devant)
- Toutes les 10 à 20 minutes : J2. etat "calcule" = ça tourne ; « point » dit lequel ; « faits » liste les points finis avec leur résonance. La même chose se lit dans l'assistant (« Avancement : … % », « Reste : ≈ … ») et dans le « Journal du solveur ».
- etat "fini" : Ctrl+S puis J3 (étape 8 ou 10).
- etat "echoue" : recopie « detail » et la fin du « Journal du solveur », Ctrl+S, arrête-toi.
- etat "perdu" (« Suivi perdu » : la page n'a pas joint le serveur) : regarde la carte WEB_ANTENNA du lanceur dans un autre onglet. Si elle dit encore « En cours sur … », suis le calcul par J4 avec l'id noté. Sinon arrête-toi.
- Si tu ne peux pas attendre aussi longtemps, arrête-toi après un lancement en écrivant dans le rapport l'id, le projet et la prochaine étape : l'utilisateur recollera cette consigne dans une nouvelle conversation.

REPRENDRE DANS UNE NOUVELLE CONVERSATION
- Retrouve l'onglet WEB_ANTENNA déjà ouvert (ne le recharge pas, n'en ouvre pas un autre à sa place ; si tu ne l'atteins pas, demande à l'utilisateur de te le donner) et exécute J2 : « projet », « tache », et « essais » (« (résultats) » = un calcul enregistré dans ce projet) disent où tu en es. Reprends à l'étape qui suit.
- Un relevé J3 d'une conversation précédente que l'utilisateur ne t'a pas recopié : si J2 dit non_enregistre false, ouvre le projet concerné dans « Reprendre un projet » (son résultat revient avec lui) et refais J3. Avec non_enregistre true, Ctrl+S d'abord : sinon la page ouvre une fenêtre de confirmation qui te bloque.
- Onglet perdu (fermé, rechargé) : ne relance rien, le calcul continue sur le serveur. Ouvre un onglet neuf sur WEB_ANTENNA et suis-le par J4 avec l'id noté. Dis dans le rapport que ce résultat n'est pas dans le projet.

RAPPORT : le format commun, et en plus, avant CONSOLE :
IDENTIFIANTS : <projet> → <id> (pour chaque lancement)
DEVIS : <le récapitulatif de « ∑ Chiffrer la plage », mot pour mot, pour chaque lancement>
TABLEAU : une ligne par point (12), tirée de J3 : wr ; Lm ; f0 (GHz) ; S11 min (dB) ; S11 à 2,45 GHz (dB) ; Z au port (Re + j Im) ; Z au pied (Re + j Im) ; energie_fin_db ; diagnostic s'il y en a un
RELEVÉ :
--- wr x1 a x2 croise Lm (projet essai-claude-monopole-wr-lm) ---
<la sortie de J3, recopiée telle quelle : pas de retouche, pas d'arrondi, pas de remise en forme>
--- wr x0,6 (3,5712 mm), Lm seul (projet essai-claude-monopole-wr06) ---
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
- [ ] Les 12 points sont calculés (9 + 3), aucun `diagnostic` (« aucune grandeur exploitable »).
- [ ] Convergence partout : `energie_fin_db` ≤ −34 dB (seuil −40 dB ; le serveur ne s'en alarme qu'au-delà de 6 dB au-dessus, `python/openems_run.py:655`).
- [ ] Le point `wr` 5,952 × `Lm` 24,2416 retombe près du relevé du 03/10/2026 (2,435 GHz, +16j) : c'est le témoin du document.
- [ ] Im Z (au port) change de signe entre deux `wr` sur au moins une ligne de `Lm` (sinon la plage ne l'encadre pas).
- [ ] Les deux relevés sont dans le rapport, et `WEB_ANTENNA/croisement-monopole-wr-Lm.json` en est fait (moi, après).

## Ce qu'on en tirera
- Im Z **au port** à la résonance, où le +16j a été lu ; le relevé garde aussi Z au pied, je lirai les deux.
- **Si Im Z passe par zéro entre ×1 et ×2** : on retient le rapport interpolé (par exemple ×1,4), et `wr` par défaut passe de `2·wf` à ce rapport × `2·wf`. Le `Lm` qui garde 2,45 GHz à ce `wr` se lit sur la même ligne, et donne la nouvelle correction de longueur (1,29 descendra, puisque le brin large résonne plus bas).
- **Si Im Z reste positif même à ×2** : la largeur ne suffit pas, l'inductance vient d'ailleurs (la ligne, le raccord brin-masse) ; le défaut de `wr` ne bouge pas, et c'est `Lg` ou la marge qu'il faudrait croiser.
- **Si ×0,6 est déjà le meilleur** : le sens est l'inverse de celui attendu, et je relirai les courbes avant de toucher au gabarit.

## Après
Pour Claude Code, à partir du rapport :
- Construire `web_antenna/croisement-monopole-wr-Lm.json`, liste `cas` comme `croisement-patch-2026-10.json` : `_note` = « Monopole wr x Lm, FR-4 1,53 mm, 2,45 GHz, reglages par defaut, arret -40 dB. wr x1/x1,5/x2 croise, wr x0,6 balaye seul » ; deux cas, « wr x1 a x2 croise Lm » et « wr x0,6 (3,5712 mm), Lm seul », chacun avec `balayage` et `points` tels que J3 les rend et `lignes` refaites depuis les points comme `_assembler` (`python/openems_run.py:973-987`). Si l'utilisateur a envoyé les projets sur GitHub, contrôler la recopie avec `tasks/essais/ESSAI-antenna-releve.py` sur leurs `resultats.json`.
- [ ] `web_antenna/A-FAIRE.md:329-333` (plan, point 2), `:294` (ligne monopôle du tableau), `:310-311`.
- [ ] `web_antenna/js/22-antennes.js:413` (`wr`) et `:415-422` (correction de `Lm`, `ponytail:`).
- [ ] `web_antenna/test/banc-interface.js` (section 4, les motifs) si le défaut change.
