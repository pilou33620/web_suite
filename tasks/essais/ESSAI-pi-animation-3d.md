# Essai : l'animation 3D de `fabrication.zip` dans le Chromium du Pi

Où : Pi, page PCB ouverte **dans le Chromium du Pi** (écran branché ou bureau à distance) ·
Outils : WEB·SUITE (WEB_CAO lancé), un lecteur vidéo (VLC, ou Chromium lui-même) · Durée :
15 min · Source : `WEB_CAO/A-FAIRE.md:158`

## Pourquoi

Avec la bascule **Fichier → Animation 3D dans le .zip**, **Fabrication .zip** joue la carte qui
s'assemble et l'enregistre dans le navigateur :
- `MediaRecorder` sur le canevas, `captureStream(30)`, 1280 × 720, 8 Mbit/s
  (`editeur-pcb/js/41-animation-3d.js:354-388`) ;
- le format est le premier que le navigateur accepte, dans cet ordre
  (`41-animation-3d.js:342-347`) : `video/mp4;codecs=avc1.640028` (H.264), puis
  `video/webm;codecs=vp9`, `video/webm;codecs=vp8`, `video/webm` ;
- la barre d'état donne le format retenu et les images par seconde réellement rendues
  (`41-animation-3d.js:415`, `04-fabrication.js:1306-1310`).

Déjà fait le 21/10/2026 dans Chromium headless sous Linux, en rendu logiciel (SwiftShader) :
WebM VP9, 20 s, **6 images/s** (`WEB_CAO/A-FAIRE.md:148-157`). Ce qu'on ne sait pas :
le format que retient le Chromium du Pi, et sa cadence avec le vrai GPU du Pi.

L'essai n'écrit rien dans PROJETS : le dossier part en téléchargement. Pas besoin de copie,
mais n'enregistre pas la carte.

## Avant de commencer

1. Mets à jour : démarre WEB·SUITE (`cd ~/Documents/web_suite && ./demarrer_WEB_SUITE.sh`) ;
   le terminal dit `WEB_CAO  installé · À jour.` ou `Mis à jour (n commit(s)).` Vérifie que
   l'animation y est, avec sa dernière correction (un enregistrement à la fois) :
   `git -C ~/Documents/web_suite/WEB_CAO merge-base --is-ancestor 058580a HEAD && echo ok`.
2. Dans la page WEB·SUITE, **▶ Lancer** sur WEB_CAO.
3. Dans le Chromium **du Pi** : `http://127.0.0.1:8000/editeur-pcb/editeur-pcb.html`.
   Note sa version (`chrome://version`, première ligne).
4. **Fichier → Exemples…** → « Interface IoT & Bus Haute Vitesse — 4 couches » → **Ouvrir cet
   exemple**.

## Étapes

1. Console du navigateur (F12 → Console), colle :
   ```
   ["video/mp4;codecs=avc1.640028","video/webm;codecs=vp9","video/webm;codecs=vp8","video/webm"].map(t=>t+" : "+MediaRecorder.isTypeSupported(t)).join("\n")
   ```
   → attendu : quatre lignes `true` / `false`. La première à `true` est le format que prendra
   l'animation.
2. **Fichier → 🎬 Animation 3D dans le .zip**.
   → attendu : le bouton s'allume ; barre d'état « Fabrication .zip : l'animation 3D (vidéo et
   page HTML) sera ajoutée au dossier. »
3. **Fichier → 🏭 Fabrication .zip**, puis ne touche plus à rien : l'onglet doit rester au
   premier plan.
   → attendu : un voile noir, la carte qui s'assemble (substrat, cuivre, vernis, sérigraphie,
   composants du plus gros au plus petit, puis un tour complet), et dessous « Animation 3D :
   enregistrement t / N s — gardez cet onglet au premier plan ». N est d'une vingtaine de
   secondes.
4. À la fin, le voile disparaît et le .zip se télécharge.
   → attendu : barre d'état « … fichier(s) exportés dans …fabrication….zip — … Animation 3D :
   vidéo <format>, <N> s, rendue à <X> images/s. » **Recopie cette ligne telle quelle.**
5. Regarde le contenu du .zip :
   ```
   cd ~/Téléchargements 2>/dev/null || cd ~/Downloads
   unzip -l "$(ls -t *fabrication*.zip | head -1)" | grep -i assemblage
   ```
   → attendu : `<nom>-ASSEMBLAGE-3D.html` et `<nom>-ASSEMBLAGE-3D.mp4` (ou `.webm`).
6. Sors la vidéo et lis-la (VLC, ou glisse-la dans un onglet de Chromium).
   → attendu : 1280 × 720, environ N s, la même animation, sans écran noir ni image figée.
   Si `ffprobe` est installé :
   `ffprobe -v error -show_entries stream=codec_name,width,height,avg_frame_rate -show_entries format=duration <fichier>`.
7. Ouvre la page `…-ASSEMBLAGE-3D.html` dans Chromium, Wi-Fi coupé si tu veux le vérifier.
   → attendu : l'animation tourne en boucle ; glisser la fait tourner, la molette zoome,
   un double-clic revient à la vue de l'animation.

## C'est réussi si

- [ ] La barre d'état donne un format et une cadence : les deux sont notés
- [ ] Le .zip contient la vidéo et la page HTML
- [ ] La vidéo se lit, à la bonne taille et sur toute sa durée
- [ ] La page HTML s'ouvre hors ligne et se tourne à la souris

## Si ça rate, note

- La version de Chromium (`chrome://version`) et le modèle du Pi (`cat /proc/device-tree/model`).
- Les quatre lignes de l'étape 1.
- La ligne exacte de la barre d'état, ou le message de l'alerte « Animation 3D impossible : … »
  (le dossier part alors sans elle : `04-fabrication.js:1292-1295`).
- La console du navigateur (F12) : les lignes rouges, en capture.
- `chrome://gpu` : la ligne « WebGL » (accéléré ou logiciel). Une cadence proche de 6 images/s
  dit que le Pi rend en logiciel, comme SwiftShader.
- Si la vidéo ne se lit pas : la sortie de `ffprobe` (ou le message du lecteur).

## Après

En cas de réussite :
- `WEB_CAO/A-FAIRE.md:158` : cocher « Sur le Pi : format retenu par son Chromium et
  images par seconde », en écrivant le format et la cadence relevés
- `WEB_CAO/A-FAIRE.md:49` et `:41` (synthèse) : retirer l'essai sur le Pi
- Si le Pi enregistre en MP4 H.264 ou à une cadence très basse, le dire aussi dans le README de
  l'éditeur, section « Animation 3D dans le dossier de fabrication » (`WEB_CAO/editeur-pcb/README.md:1407`)
