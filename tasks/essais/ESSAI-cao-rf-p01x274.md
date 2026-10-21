# Essai : simulation RF S21 de P01x274, tes réponses (questions 2 à 5) et les fichiers

Où : PC avec la datasheet SX1261, le design de référence Semtech, le firmware de la carte et, si tu l'as, un analyseur de réseau · Durée : 30 min de réponses ; la mesure de l'antenne en plus si elle est à faire · Source : `web_cao/A-FAIRE.md:246` (contexte l.238-245 ; rappel l.44)

## Pourquoi
Le S21 TX de `IPC2581_Exemple/P01x274PCB-C.xml` (U400 SX1261 → antenne, 868 MHz) est prêt côté code : chaîne décrite `A-FAIRE.md:241`, Touchstone Skyworks des deux AS179 liés (l.243, question 1 réglée). Il manque ce que toi seul peux décider ou fournir :
- **Port 1** sur U400.23 (RFO). Son impédance se saisit dans le panneau S21 (`commun/simulation-em.js:17260-17274` : partie réelle, signe ± j, partie imaginaire ; l'aide dit « impédance de *sortie* de la puce »). Le bon choix est le conjugué de la Zopt du PA — absente du datasheet (§ 13.1.14.1 : elle dépend de `paDutyCycle`) — sinon 50 Ω, qui ne donne que la perte d'insertion (l.244).
- **Port 2** : A400.1 (alimentation de l'IFA) ou J400.3 (U.FL, comme au banc) (l.240).
- **Branche RX** : C431 et C414 sont à exclure tant qu'on ne simule que la TX ; un S21 RX sur RFI_P / RFI_N demanderait un port différentiel, que le code n'a pas (l.242, l.245).
- **Fichiers** : Murata, impédance de l'IFA, Zopt (l.246).

Un mauvais port 1 ou une mauvaise charge en port 2 donne un S21 qui a l'air juste et ne dit rien de la puissance rayonnée.

## Ce qu'il te faut
- La datasheet `IPC2581_Exemple/DS_SX1261-2_V2_1_EN.pdf`.
- Le design de référence Semtech SX1261 à 868 MHz (kit SX1261MB2BAS — vérifie le nom sur semtech.com), avec sa note sur l'adaptation du PA si elle donne Zopt.
- Le firmware de la carte : l'appel `SetPaConfig` (paDutyCycle, hpMax, deviceSel, paLut) et la puissance `SetTxParams`.
- La nomenclature de P01x274 (références fabricant de L404 à L407, C411 à C423).
- Si tu as un VNA et la carte : de quoi mesurer l'antenne A400 sur J400 (ou un .s1p tiré d'une simulation EM, par exemple de WEB_ANTENNA, qui exporte du `.s1p`).

## Étapes
1. **Question 2 — port 1.** Dans le firmware, relève les paramètres de `SetPaConfig` et la puissance TX. Cherche dans le design de référence ou la note Semtech la Zopt pour cette configuration.
   → à rapporter : « Zopt » = R + jX Ω à 868 MHz et sa source (document, page) ; ou « prends 50 Ω ».
2. **Question 3 — port 2.** Dis où tu veux le port 2.
   → à rapporter : « antenne (A400.1) », « U.FL (J400.3) » ou « les deux » ; et quelle variante de montage est la vraie (R416 0 Ω ou R425 / J401, `A-FAIRE.md:242`).
3. **Question 4 — branche RX.**
   → à rapporter : « exclue » (C431, C414 retirés de la marche), « gardée » (sa charge sur la TX comptée) ou « S21 RX plus tard » (port différentiel à ajouter, nouvelle case).
4. **Question 5 — fichiers.** Pour chaque self et condensateur de la chaîne TX (L405, L406, L407, C411, C412, C413, C416, C418, C419, C423, C433, et L404 / C406 / C407 pour VR_PA), relève la référence fabricant de la nomenclature.
   → à rapporter : la liste, et pour chaque référence Murata son `.s2p` (Murata SimSurfing) ; dépose-les dans `lib_simulation` ou lie-les depuis la Gestion LIB (fiche → « Modèles de simulation » → « 📎 Lier un fichier »).
5. **Impédance de l'IFA** (si port 2 = antenne). Mesure S11 de l'antenne sur J400, ou exporte le `.s1p` d'une simulation EM.
   → à rapporter : le `.s1p` (de 800 à 950 MHz au moins), ou au minimum Z = R + jX Ω à 868 MHz et comment tu l'as obtenue.
6. **Zopt** : si l'étape 1 l'a donnée, pas d'autre fichier ; sinon dis si tu préfères attendre un document Semtech ou partir sur 50 Ω.

## C'est réussi si / La réponse
- [ ] (2) Port 1 : Zopt = … + j… Ω (source) ou 50 Ω.
- [ ] (3) Port 2 : antenne / U.FL / les deux ; variante de montage.
- [ ] (4) Branche RX : exclue / gardée / plus tard.
- [ ] (5) Références de la chaîne TX ; `.s2p` Murata fournis (liste des fichiers) ; `.s1p` de l'IFA ou Z à 868 MHz ; Zopt (déjà en 2).

## Après
- Je pose les ports dans le panneau S21 de la carte (port 1 = conjugué de Zopt, saisi avec le signe opposé ; port 2 selon ta réponse), j'applique les exclusions de ta réponse 4 et je lie les fichiers à leurs références (`banc-composants.js` pour la liaison).
- Je lance le S21, je te donne la courbe et le Touchstone, et je coche `web_cao/A-FAIRE.md:246` et la ligne de rappel l.44.
- Si tu choisis « S21 RX plus tard », j'ouvre une case « port différentiel » sous l.245.
