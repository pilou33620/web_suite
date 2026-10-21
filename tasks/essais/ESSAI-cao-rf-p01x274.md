# Essai : simulation RF S21 de P01x274, tes réponses (questions 2 à 5) et les fichiers

Qui : les deux (Claude in Chrome : relever la chaîne TX dans la visionneuse et chercher Zopt et les `.s2p` sur le web ; toi : les décisions, le firmware, les fichiers à lier, la mesure de l'antenne) · Où : PC Windows du boulot (et le banc si l'antenne est à mesurer) · Durée : 30 min pour Claude in Chrome, 30 min pour toi ; la mesure de l'antenne en plus · Source : `web_cao/A-FAIRE.md:246` (contexte l.238-245 ; rappel l.44)

## Pourquoi
Le S21 TX de `IPC2581_Exemple/P01x274PCB-C.xml` (U400 SX1261 → antenne, 868 MHz) est prêt côté code : chaîne décrite `A-FAIRE.md:241`, Touchstone Skyworks des deux AS179 liés (l.243, question 1 réglée). Il manque ce que toi seul peux décider ou fournir :
- **Port 1** sur U400.23 (RFO). Son impédance se saisit dans le panneau S21 (`commun/simulation-em.js:17260-17274` : partie réelle, signe ± j, partie imaginaire ; l'aide dit « impédance de *sortie* de la puce »). Le bon choix est le conjugué de la Zopt du PA — absente du datasheet (§ 13.1.14.1 : elle dépend de `paDutyCycle`) — sinon 50 Ω, qui ne donne que la perte d'insertion (l.244).
- **Port 2** : A400.1 (alimentation de l'IFA) ou J400.3 (U.FL, comme au banc) (l.240).
- **Branche RX** : C431 et C414 sont à exclure tant qu'on ne simule que la TX ; un S21 RX sur RFI_P / RFI_N demanderait un port différentiel, que le code n'a pas (l.242, l.245).
- **Fichiers** : Murata, impédance de l'IFA, Zopt (l.246).

Un mauvais port 1 ou une mauvaise charge en port 2 donne un S21 qui a l'air juste et ne dit rien de la puissance rayonnée.

Ce que Claude in Chrome peut faire : lire les références des composants de la chaîne dans la carte ouverte dans la visionneuse, et chercher dans les documents publics (Semtech, Murata) ce qui éclaire tes réponses. Il ne décide pas : les questions 2 à 4 restent les tiennes, et lier un fichier passe par une boîte de choix de fichier qu'il ne peut pas ouvrir.

## Ce que tu prépares
1. Vérifie que la carte est servie par WEB_CAO : `http://127.0.0.1:8000/IPC2581_Exemple/P01x274PCB-C.xml` doit s'ouvrir (ou se télécharger) dans Chrome. Sinon copie `P01x274PCB-C.xml` (et, si tu l'as, `DS_SX1261-2_V2_1_EN.pdf`) avec l'explorateur Windows dans `<dossier de WEB·SUITE>\WEB_CAO\IPC2581_Exemple\` (dossier ignoré par git ; pas dans PROJETS).
2. Le firmware de la carte : note l'appel `SetPaConfig` (paDutyCycle, hpMax, deviceSel, paLut) et la puissance de `SetTxParams`. Donne ces valeurs à Claude in Chrome en tête de la consigne (« FIRMWARE : … ») ; sans elles il cherchera la Zopt de toutes les configurations publiées.
3. WEB·SUITE lancé (WEB_CAO sur `http://127.0.0.1:8000`).

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-rf-p01x274.md. But : préparer les réponses de l'utilisateur pour la simulation
S21 TX de la carte P01x274 (SX1261, 868 MHz) : les références de la chaîne TX, la Zopt du PA
d'après Semtech, et l'adresse des modèles .s2p Murata. Tu ne décides rien, tu ne poses aucun
port, tu ne lies aucun fichier, tu n'enregistres rien (pas de « ☁ Enregistrer », pas de Ctrl+S,
pas de « 📎 Lier un fichier », pas de « 💾 Enregistrer » dans la Gestion LIB).
Si l'utilisateur t'a donné une ligne « FIRMWARE : … », garde-la pour l'étape 3.

1. Onglet neuf : http://127.0.0.1:8000/visionneuse-ipc2581/visionneuse-ipc2581.html, puis :
   (async()=>{const n="P01x274PCB-C.xml";const r=await fetch("/IPC2581_Exemple/"+n);if(!r.ok)return "HTTP "+r.status+" : fichier absent";await charger(new File([await r.blob()],n));return V.fichier+" : "+V.modele.composants.length+" composants";})()
   → la fenêtre de classement des nets s'ouvre : clique « Passer ».
   Si la réponse est « HTTP 404 » : arrête l'étape 1 et 2, note-le, passe à l'étape 3.
2. Références de la chaîne :
   (()=>{const R=["U400","L404","C406","C407","C411","L405","C412","C413","L406","C433","U401","C416","C418","L407","C419","U402","C423","R416","R425","J400","J401","A400","C414","C431"];return JSON.stringify(V.modele.composants.filter(c=>R.includes(c.ref)).map(c=>({ref:c.ref,val:c.val,pkg:c.pkg,part:c.part,props:c.props})),null,1)})()
   → à relever : pour chaque repère, valeur, boîtier, référence fabricant (champ `part` ou
     `props`, ex. MPN / PART_NAME). Note les repères absents et ceux sans référence fabricant.
3. Zopt du PA. Sur semtech.com : la datasheet SX1261/2 (DS_SX1261-2, dernière révision) et le
   kit de référence 868 MHz du SX1261 (vérifie son nom exact, ex. SX1261MB2BAS / SX1261MB1BAS ;
   schéma, nomenclature, note d'application sur l'adaptation du PA si elle existe).
   → à relever, avec URL, page, figure ou tableau : le tableau des réglages optimaux du PA
     (paDutyCycle, hpMax, deviceSel, paLut par puissance) ; toute impédance de charge optimale
     du PA (« optimal load », « Zopt », « load impedance », R + jX à 868 MHz) et la configuration
     à laquelle elle se rapporte ; le réseau d'adaptation du kit de référence (valeurs de
     chaque self et condensateur entre RFO et l'antenne).
   → si une ligne FIRMWARE t'a été donnée : la Zopt qui correspond à CETTE configuration, ou
     « non publiée pour cette configuration ».
   Source PRIMAIRE = document Semtech ; forum, blog, article de distributeur = SECONDAIRE.
   N'invente aucune valeur : « non trouvé » est une réponse.
4. Modèles Murata. Pour chaque référence fabricant Murata de l'étape 2 (selfs LQG / LQW,
   condensateurs GRM / GJM…), cherche sur murata.com la page du produit ou l'outil SimSurfing.
   → à relever : la référence, l'URL de la page du produit, l'URL (ou le chemin de clics) du
     fichier de paramètres S (.s2p) ; ne télécharge rien, l'utilisateur le fera.
   Pour une référence non Murata : le fabricant, et s'il publie un .s2p (URL) ou non.

Rapport, en plus du format commun :
- CHAÎNE TX : le tableau repère / valeur / boîtier / référence fabricant.
- ZOPT : ce que disent les documents Semtech (valeur, configuration, URL, page), ou « non publiée ».
- RÉSEAU DE RÉFÉRENCE : les valeurs du kit Semtech, à côté de celles de la carte (même repère
  fonctionnel si possible).
- MODÈLES : une ligne par référence (URL du .s2p ou « pas de modèle publié »).
- Ce que tu n'as pas pu ouvrir, avec le message exact.
```

## C'est réussi si
- [ ] La chaîne TX relevée (références fabricant de L405, L406, L407, C411, C412, C413, C416, C418, C419, C423, C433, et L404 / C406 / C407 pour VR_PA).
- [ ] Zopt : valeur et source primaire, ou « non publiée » dit clairement.
- [ ] Une adresse de `.s2p` par référence Murata, ou « pas de modèle ».
- [ ] Tes réponses (ci-dessous) : (2) port 1 ; (3) port 2 et variante de montage ; (4) branche RX ; (5) fichiers liés, `.s1p` de l'IFA ou Z à 868 MHz.

## Ce qui reste à toi
Avec le rapport de Claude in Chrome sous les yeux :
1. **Question 2 — port 1.** « Zopt = R + jX Ω à 868 MHz (source) » ou « prends 50 Ω ». Si Semtech ne publie pas la Zopt de ta configuration, dis si tu préfères attendre un document ou partir sur 50 Ω.
2. **Question 3 — port 2.** « antenne (A400.1) », « U.FL (J400.3) » ou « les deux » ; et la variante de montage qui est la vraie (R416 0 Ω ou R425 / J401, `A-FAIRE.md:242`).
3. **Question 4 — branche RX.** « exclue » (C431, C414 retirés de la marche), « gardée » (sa charge sur la TX comptée) ou « S21 RX plus tard » (port différentiel à ajouter, nouvelle case).
4. **Question 5 — fichiers.** Télécharge les `.s2p` aux adresses du rapport et lie-les dans la Gestion LIB : fiche de la référence → « Modèles de simulation » → « 📎 Lier un fichier… » (la boîte de choix de fichier est un geste à toi).
5. **Impédance de l'IFA** (si port 2 = antenne) : mesure S11 de l'antenne sur J400 au VNA, ou exporte le `.s1p` d'une simulation EM (WEB_ANTENNA exporte du `.s1p`), de 800 à 950 MHz au moins ; à défaut, Z = R + jX Ω à 868 MHz et comment tu l'as obtenue.

## Après
Pour Claude Code, avec le rapport et tes réponses :
- [ ] poser les ports dans le panneau S21 de la carte (port 1 = conjugué de Zopt, saisi avec le signe opposé, `web_cao/commun/simulation-em.js:17260-17274` ; port 2 selon la réponse 3) ;
- [ ] appliquer les exclusions de la réponse 4 et vérifier la liaison des fichiers à leurs références (`banc-composants.js`) ;
- [ ] lancer le S21, donner la courbe et le Touchstone, cocher `web_cao/A-FAIRE.md:246` et la ligne de rappel `web_cao/A-FAIRE.md:44` ;
- [ ] si « S21 RX plus tard » : ouvrir une case « port différentiel » sous `web_cao/A-FAIRE.md:245`.
