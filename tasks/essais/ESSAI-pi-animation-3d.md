# Essai : l'animation 3D de `fabrication.zip` (Chrome du PC, puis Chromium du Pi)

Qui : les deux · Où : Chrome du PC Windows du boulot, puis Chromium du Pi · Durée : 15 min
chacun · Source : `WEB_CAO/A-FAIRE.md:158`

## Pourquoi

Avec la bascule **Fichier → Animation 3D dans le .zip**, **Fabrication .zip** joue la carte qui
s'assemble et l'enregistre dans le navigateur : `MediaRecorder` sur le canevas,
`captureStream(30)`, 1280 × 720, 8 Mbit/s (`editeur-pcb/js/41-animation-3d.js:354-388`) ; le
format est le premier accepté de `ANIM3D_FORMATS` (`41-animation-3d.js:342-344`) :
`video/mp4;codecs=avc1.640028` (H.264), puis `video/webm;codecs=vp9`, `vp8`, `video/webm` ; la
barre d'état donne le format et les images par seconde rendues (`41-animation-3d.js:415`,
`04-fabrication.js:1306-1310`).

Fait le 21/10/2026 dans Chromium headless sous Linux, rendu logiciel (SwiftShader) : WebM VP9,
20 s, **6 images/s** (`WEB_CAO/A-FAIRE.md:148-157`). Inconnus : le format et la cadence d'un
vrai navigateur avec un vrai GPU — le Chrome du PC d'abord (Google Chrome encode peut-être le
H.264), le Pi ensuite.

Claude in Chrome ne peut pas ouvrir le .zip téléchargé : il relève donc dans la page ce qui part
dedans. Avant le clic, une commande enveloppe `fabLivrer` (`04-fabrication.js:1300`, qui reçoit
les fichiers de l'animation et range le .zip) pour en garder une copie ; la vidéo est ensuite
relue dans la page (taille, durée, cinq images), et la page HTML ouverte dans un cadre
par-dessus l'éditeur. Rien n'est écrit dans PROJETS : l'exemple est chargé en mémoire, le
dossier part en téléchargement.

Rejoué le 21/10/2026 dans Chromium headless (Playwright) avec ces commandes mot pour mot : WebM
VP9, 19 s, 8 images/s, 25 fichiers, vidéo 1280 × 720 de 18,1 s, page HTML sans script externe,
qui tourne au glisser.

## Ce que tu prépares

1. Double-clic sur `demarrer_WEB_SUITE.cmd` : le lanceur s'ouvre sur `http://127.0.0.1:8100`.
2. Ouvre Claude in Chrome et colle la consigne. Le .zip arrive dans tes Téléchargements : tu
   peux l'ouvrir dans l'Explorateur et lire la vidéo, si tu veux voir de tes yeux.

## Consigne pour Claude in Chrome

```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-pi-animation-3d.md. L'animation 3D du dossier de fabrication de l'éditeur PCB :
format vidéo retenu par ce Chrome, images par seconde, et contenu du .zip, relevé dans la page.
Le téléchargement de fabrication.zip est voulu ici (ce n'est pas le « ⇩ Télécharger » du
lanceur). Les commandes JavaScript se collent telles quelles dans l'outil JavaScript, dans
l'onglet de l'éditeur PCB.

1. Ouvre http://127.0.0.1:8100 et relève les versions des outils. Si la carte WEB_CAO dit
   qu'il est arrêté, clique ▶ Lancer sur elle.
   → attendu : WEB_CAO lancé, port 8000 (sinon, prends le port de sa carte).
2. Ouvre un onglet neuf sur http://127.0.0.1:8000/editeur-pcb/editeur-pcb.html. Il doit rester
   au premier plan pendant tout l'essai : n'ouvre pas d'autre onglet avant l'étape 9.
3. Formats que ce Chrome sait enregistrer :

["video/mp4;codecs=avc1.640028", "video/webm;codecs=vp9", "video/webm;codecs=vp8", "video/webm"]
  .map(t => t + " : " + MediaRecorder.isTypeSupported(t)).join("\n")

   → attendu : quatre lignes true / false ; la première à true est le format de l'animation.
4. Le rendu 3D :

(() => {
  const gl = document.createElement("canvas").getContext("webgl");
  if (!gl) return "pas de WebGL";
  const e = gl.getExtension("WEBGL_debug_renderer_info");
  return e ? gl.getParameter(e.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER);
})()

   → attendu : la carte graphique du PC (« ANGLE (Intel… / NVIDIA… / AMD…, Direct3D11…) »).
   « SwiftShader » voudrait dire un rendu logiciel : note-le.
5. Charge l'exemple 2 et prépare le relevé du .zip :

(() => {
  loadDoc(exemple2()); fit();
  if (!globalThis.ESSAI_FAB_ORIG) {
    globalThis.ESSAI_FAB_ORIG = fabLivrer;
    fabLivrer = function (extra, note) {   // relève ce qui part dans le .zip, puis le livre comme avant
      const files = ESSAI_FAB_ORIG(extra, note);
      globalThis.ESSAI_FAB = {extra, note, noms: files.map(f => f.name)};
      return files;
    };
  }
  globalThis.ESSAI_FAB = null;
  return "exemple chargé : " + S.fps.length + " empreintes ; relevé du .zip prêt";
})()

   → attendu : « exemple chargé : 11 empreintes ; relevé du .zip prêt ».
6. Menu Fichier → « Animation 3D dans le .zip » (🎬). Puis lis la barre d'état :

document.getElementById("fHint").textContent

   → attendu : le bouton allumé ; « Fabrication .zip : l'animation 3D (vidéo et page HTML) sera
   ajoutée au dossier. »
7. Menu Fichier (rouvre-le s'il s'est fermé) → « Fabrication .zip » (🏭). Ne touche plus à rien
   pendant 30 s ; fais une capture vers la 5e seconde.
   → attendu : un voile noir, la carte qui s'assemble (substrat, cuivre, vernis, sérigraphie,
   composants, puis un tour complet) et dessous « Animation 3D : enregistrement t / N s — gardez
   cet onglet au premier plan », N vers 19 ou 20. Si une alerte « Animation 3D impossible : … »
   s'ouvre, ou si Chrome demande où enregistrer le fichier, arrête-toi : demande à
   l'utilisateur de la fermer et recopie son texte.
8. Le voile parti, relis la barre d'état (commande de l'étape 6).
   → attendu : « 25 fichier(s) exportés dans fabrication.zip — 35 trou(s), 3 outil(s) de
   perçage, 11 empreinte(s). Animation 3D : vidéo <format>, <N> s, rendue à <X> images/s. »
   Recopie-la telle quelle.
9. Contrôle ce qui est parti dans le .zip et relis la vidéo dans la page (jusqu'à 30 s ; si la
   réponse est vide ou une promesse, relance une fois) :

(async () => {
  const F = globalThis.ESSAI_FAB;
  if (!F) return "le dossier n'est pas encore parti";
  const out = {note: F.note, fichiers: F.noms};
  const v = F.extra.find(f => f.data), h = F.extra.find(f => f.text);
  out.page_html = h ? h.name + " : " + h.text.length + " caractères ; script externe : " + /<script[^>]*[ ]src=/i.test(h.text) : "absente";
  if (v) {
    const el = document.createElement("video"); el.muted = true; el.preload = "auto";
    el.src = URL.createObjectURL(new Blob([v.data], {type: /\.mp4$/.test(v.name) ? "video/mp4" : "video/webm"}));
    const attendre = ev => new Promise((ok, ko) => {
      const t = setTimeout(() => ko(new Error(ev + " : rien en 10 s")), 10000);
      el.addEventListener(ev, () => { clearTimeout(t); ok(); }, {once: true});
      el.addEventListener("error", () => { clearTimeout(t); ko(new Error("vidéo illisible, code " + (el.error && el.error.code))); }, {once: true});
    });
    await attendre("loadedmetadata");
    // une vidéo de MediaRecorder n'annonce pas toujours sa durée : un saut à la fin la donne
    if (!isFinite(el.duration)) { el.currentTime = 1e6; await attendre("seeked"); }
    const D = isFinite(el.duration) ? el.duration : el.currentTime;
    out.video = v.name + " : " + v.data.length + " octets, " + el.videoWidth + " x " + el.videoHeight + ", " + Math.round(D * 10) / 10 + " s";
    const cv = document.createElement("canvas"); cv.width = 160; cv.height = 90;
    const g = cv.getContext("2d", {willReadFrequently: true});
    out.images = [];
    for (const f of [0.1, 0.3, 0.5, 0.7, 0.9]) {
      el.currentTime = D * f; await attendre("seeked");
      g.drawImage(el, 0, 0, 160, 90);
      const d = g.getImageData(0, 0, 160, 90).data;
      let s = 0, e = 0;
      for (let i = 0; i < d.length; i += 4) { s += d[i] + d[i + 1] + d[i + 2]; e = (e * 31 + d[i] + d[i + 1]) >>> 0; }
      out.images.push(Math.round(f * 100) + " % : luminosité " + Math.round(s / (d.length / 4) / 3) + ", empreinte " + e.toString(16));
    }
    URL.revokeObjectURL(el.src);
  }
  return JSON.stringify(out, null, 1);
})()

   → attendu : « fichiers » contient carte-ASSEMBLAGE-3D.html et carte-ASSEMBLAGE-3D.mp4 (ou
   .webm) ; « script externe : false » (la page s'ouvre hors ligne) ; « video » en 1280 x 720,
   à une seconde près de N ; cinq images de luminosité non nulle (le fond est noir : 10 à 30)
   et d'empreintes toutes différentes (ni écran noir, ni image figée).
10. Ouvre la page HTML de l'animation dans un cadre par-dessus l'éditeur :

(() => {
  const h = globalThis.ESSAI_FAB.extra.find(f => f.text);
  const fr = document.createElement("iframe");
  fr.id = "essaiAnim";
  fr.style.cssText = "position:fixed;left:5vw;top:5vh;width:90vw;height:90vh;z-index:99999;border:2px solid #888;background:#000";
  fr.srcdoc = h.text;
  document.body.appendChild(fr);
  return "page de l'animation ouverte par-dessus l'éditeur";
})()

   → attendu : l'animation en boucle, « Glisser : tourner · molette : zoom · double-clic : vue de
   l'animation » en bas. Capture ; puis glisse dans le cadre (capture : la carte a tourné),
   molette (capture : zoom), double-clic (la vue de l'animation revient).
11. Ferme le cadre :

document.getElementById("essaiAnim").remove()

Rapport, en plus du format commun : les quatre lignes de l'étape 3 ; le texte de l'étape 4 ; la
ligne de la barre d'état de l'étape 8, telle quelle ; le JSON de l'étape 9 ; ce qu'ont fait le
glisser, la molette et le double-clic à l'étape 10.
```

## C'est réussi si

- [ ] PC : la barre d'état donne un format et une cadence ; les deux sont notés, avec le rendu (GPU ou SwiftShader)
- [ ] PC : le .zip contient la vidéo et la page HTML
- [ ] PC : la vidéo se relit, en 1280 × 720, sur toute sa durée, sans écran noir ni image figée
- [ ] PC : la page HTML n'appelle aucun script extérieur et se tourne à la souris
- [ ] Pi : format et cadence notés dans le Chromium du Pi (« Ce qui reste à toi »)

## Ce qui reste à toi

La même chose dans le **Chromium du Pi** (écran branché ou bureau à distance), à la maison.

1. Démarre WEB·SUITE sur le Pi, **▶ Lancer** sur WEB_CAO ; l'animation doit avoir sa dernière
   correction : `git -C ~/Documents/web_suite/WEB_CAO merge-base --is-ancestor 058580a HEAD && echo ok`.
2. Chromium du Pi : `http://127.0.0.1:8000/editeur-pcb/editeur-pcb.html` ; note `chrome://version`
   et la ligne « WebGL » de `chrome://gpu`.
3. F12 → Console (`allow pasting` au premier collage), puis les étapes 3 à 11 de la consigne,
   commandes comprises, en comparant aux attendus.
4. Facultatif, la vidéo hors du navigateur :
   `cd ~/Téléchargements 2>/dev/null || cd ~/Downloads; unzip -l "$(ls -t *fabrication*.zip | head -1)" | grep -i assemblage`,
   puis la vidéo dans VLC ; avec `ffprobe` :
   `ffprobe -v error -show_entries stream=codec_name,width,height,avg_frame_rate -show_entries format=duration <fichier>`.

Si ça rate, note : `chrome://version`, `cat /proc/device-tree/model`, les sorties des commandes,
le texte d'une alerte « Animation 3D impossible : … » (le dossier part alors sans elle :
`04-fabrication.js:1292-1295`), les lignes rouges de la console. Une cadence proche de
6 images/s dit un rendu logiciel, comme SwiftShader.

## Après

Pour Claude Code :
- [ ] PC fait : `WEB_CAO/A-FAIRE.md:148-157`, ajouter le format et la cadence du Chrome du PC
      (référence avec GPU)
- [ ] Pi fait : `WEB_CAO/A-FAIRE.md:158` cocher « Sur le Pi : format retenu par son Chromium et
      images par seconde », avec les valeurs ; `:49` et `:41` : retirer l'essai sur le Pi
- [ ] Un enregistrement en MP4 H.264, ou une cadence très basse : le dire dans
      `WEB_CAO/editeur-pcb/README.md:1407` (« Animation 3D dans le dossier de fabrication »)
