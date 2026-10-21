# Essai : un IPC-2581 d'eCADSTAR porte-t-il `<Package><Outline>` et `height` ?

Qui : les deux (toi : l'export eCADSTAR et le dépôt du fichier ; Claude in Chrome : la lecture du XML et la visionneuse) · Où : PC Windows du boulot (eCADSTAR et WEB·SUITE) · Durée : 10 min pour toi, 20 min pour Claude in Chrome · Source : `web_cao/A-FAIRE.md:338` (rappel l.47 ; risque noté `tasks/plan-dossier-meca.md:81`)

## Pourquoi
Le dossier mécanique de la visionneuse tire le corps et la hauteur de chaque composant du fichier IPC-2581 :
- `python/ipc2581_parser.py:2157-2172` (`_process_package`) lit l'attribut `height` du `<Package>` (dans les unités du fichier) et le premier `<Polygon>` **directement sous** `<Package><Outline>` ;
- `visionneuse-ipc2581/js/08-dossier-meca.js:30-38` (`mecaEntreeIpc`) : sans ce polygone (au moins 3 points), le corps devient la boîte des pastilles, marqué `corps_estime` ; l.58 : sans `height`, la hauteur vient de la LIB (« Maximun Height ») ou de ta saisie ;
- `commun/dossier-meca.js` (`mecaDonnees`) écrit alors l'avertissement « Encombrement estimé d'après les pastilles (pas de contour de boîtier dans le fichier) ».

On ne sait pas ce qu'eCADSTAR écrit. S'il met le contour ailleurs (`<AssemblyDrawing>`, `<SilkScreen>`, un `<Polyline>`) ou la hauteur dans un attribut non standard, tous les corps sont estimés trop petits et le boîtier dessiné d'après le dossier peut toucher des composants.

Claude in Chrome n'a pas eCADSTAR et ne lit pas un fichier du disque : tu exportes et tu déposes le fichier là où WEB_CAO le sert ; il lit ensuite le XML par la page et l'ouvre dans la visionneuse.

## Ce que tu prépares
1. Dans eCADSTAR (PCB Editor), sur une carte réelle avec des composants de hauteurs variées (connecteur, électrolytique, CI), exporte en IPC-2581 : la révision la plus haute proposée (C, sinon B), avec tout le contenu (composants, assemblage, packages). Si l'export propose une option de hauteur ou de contour d'assemblage / de placement, coche-la.
   → à noter pour le rapport : version d'eCADSTAR, révision IPC-2581, options cochées (copie d'écran de la fenêtre d'export).
2. Renomme le fichier exactement `essai-ecadstar.xml` et copie-le, avec l'explorateur Windows, dans le dossier `IPC2581_Exemple` de WEB_CAO : `<dossier de WEB·SUITE>\WEB_CAO\IPC2581_Exemple\` (à côté de `demarrer_WEB_SUITE.cmd` se trouve le dossier `WEB_CAO` ; crée `IPC2581_Exemple` s'il manque). Ce dossier est ignoré par git : le fichier ne part jamais sur GitHub. **Ne le mets pas dans PROJETS**, qui est envoyé sur GitHub.
3. WEB·SUITE lancé (WEB_CAO sur `http://127.0.0.1:8000`, avec sa LIB), Claude in Chrome ouvert. Donne-lui la consigne avec ta note de l'étape 1.

## Consigne pour Claude in Chrome
```text
Lis d'abord https://github.com/pilou33620/web_suite/blob/main/tasks/essais/CLAUDE-IN-CHROME.md et respecte ses règles.

Essai : ESSAI-cao-ecadstar-package.md. But : dire où un IPC-2581 exporté par eCADSTAR met le
contour et la hauteur des boîtiers, et ce que le dossier méca de la visionneuse en tire.
L'utilisateur a déposé le fichier ; il est servi en lecture à
http://127.0.0.1:8000/IPC2581_Exemple/essai-ecadstar.xml
Tu ne l'enregistres nulle part : pas de « ☁ Enregistrer », pas de Ctrl+S, pas de
« Exporter .json », pas de « ⬇ Télécharger .zip ».

1. Ouvre la visionneuse dans un onglet neuf :
   http://127.0.0.1:8000/visionneuse-ipc2581/visionneuse-ipc2581.html
   et exécute (lecture du XML brut, rien n'est modifié) :
   (async()=>{const r=await fetch("/IPC2581_Exemple/essai-ecadstar.xml");if(!r.ok)return "HTTP "+r.status+" : fichier absent";const d=new DOMParser().parseFromString(await r.text(),"application/xml");if(d.getElementsByTagName("parsererror").length)return "XML illisible";const q=(e,n)=>[...e.getElementsByTagNameNS("*",n)];const kids=e=>[...e.children].map(c=>c.localName);const P=q(d,"Package");const J=q(d,"Component").find(c=>/^(J|CN|X|P)\d/i.test(c.getAttribute("refDes")||""));const pj=J&&P.find(p=>p.getAttribute("name")===J.getAttribute("packageRef"));const H=[...d.getElementsByTagName("*")].filter(e=>e.localName!=="Package"&&[...e.attributes].some(a=>/height/i.test(a.name)||/^height$/i.test(a.value)));return JSON.stringify({racine:d.documentElement.localName,revision:d.documentElement.getAttribute("revision"),logiciel:q(d,"SoftwarePackage").map(e=>e.outerHTML.slice(0,400)),unites:q(d,"CadHeader").map(e=>e.getAttribute("units")),packages:P.length,avecHeight:P.filter(p=>p.hasAttribute("height")).length,exemplesHeight:P.filter(p=>p.hasAttribute("height")).slice(0,5).map(p=>p.getAttribute("name")+" height="+p.getAttribute("height")),avecOutlinePolygon:P.filter(p=>[...p.children].some(c=>c.localName==="Outline"&&kids(c).includes("Polygon"))).length,enfantsDePackage:[...new Set(P.flatMap(kids))],enfantsDeOutline:[...new Set(P.flatMap(p=>[...p.children].filter(c=>c.localName==="Outline").flatMap(kids)))],heightAilleurs:[...new Set(H.map(e=>e.localName+" : "+e.outerHTML.slice(0,160)))].slice(0,15),connecteur:J?J.getAttribute("refDes")+" → "+J.getAttribute("packageRef"):"aucun",packageConnecteur:pj?pj.outerHTML.slice(0,6000):"introuvable"},null,1)})()
   → attendu : `logiciel` nomme eCADSTAR / Zuken ; sinon arrête-toi (mauvais fichier).
   → à relever : `revision`, `unites` (MILLIMETER, INCH ou MICRON) ; `packages` ; `avecHeight` et
     des exemples (la valeur est-elle plausible pour ce boîtier, dans ces unités ?) ;
     `avecOutlinePolygon` ; la liste des enfants d'un <Package> et d'un <Outline> ; `heightAilleurs`
     (où et sous quel nom une hauteur apparaît hors de <Package>, ex. NonstandardAttribute) ; le
     <Package> complet du connecteur (`packageConnecteur`), à recopier tel quel dans le rapport.
   → si `avecOutlinePolygon` vaut 0 : regarde dans `packageConnecteur` où est le contour
     (AssemblyDrawing, SilkScreen, Polyline, autre) et note l'élément exact.
2. Même onglet, ouvre le fichier dans la visionneuse :
   (async()=>{const n="essai-ecadstar.xml";const r=await fetch("/IPC2581_Exemple/"+n);if(!r.ok)return "HTTP "+r.status;await charger(new File([await r.blob()],n));return V.fichier+" : "+V.modele.composants.length+" composants";})()
   → attendu : la carte s'affiche ; une fenêtre de classement des nets s'ouvre : clique « Passer ».
   Capture d'écran.
3. Bouton « Dossier méca » de la barre. Attends que le tableau se remplisse, puis exécute :
   (()=>{const J=mecaIpcDossier();const n=s=>J.composants.filter(c=>c.source_hauteur===s).length;return JSON.stringify({composants:J.composants.length,corpsEstimes:J.composants.filter(c=>c.corps_estime).length,hauteurs:{fichier:n("fichier"),lib:n("lib"),saisie:n("saisie"),inconnue:n("inconnue")},avertissements:J.avertissements},null,1)})()
   → à relever : M composants, N corps estimés, K hauteurs « fichier », et les avertissements
     (dont « Encombrement estimé d'après les pastilles … N composant(s) »). Capture de la fenêtre.
   Ferme la fenêtre (✕). Ne clique pas « ⬇ Télécharger .zip ».

Rapport, en plus du format commun :
- EXPORT : ce que l'utilisateur t'a donné (version eCADSTAR, révision, options).
- XML : le résultat brut de l'étape 1 et, en clair : `height` présent / absent / ailleurs (nom
  exact et élément) ; `<Outline><Polygon>` présent / absent / contour ailleurs (élément exact).
- VISIONNEUSE : N corps estimés sur M composants, K hauteurs « fichier ».
```

## C'est réussi si
- [ ] `<Package height>` : présent / absent / ailleurs (nom exact et élément).
- [ ] `<Package><Outline><Polygon>` : présent / absent / contour ailleurs (élément exact).
- [ ] Unités du fichier.
- [ ] Visionneuse : N corps estimés sur M composants, K hauteurs « fichier ».
- [ ] Un `<Package>` complet d'un connecteur, recopié dans le rapport.
- [ ] Version d'eCADSTAR et options d'export notées.

## Ce qui reste à toi
- L'export depuis eCADSTAR et le dépôt du fichier (ci-dessus) : Claude in Chrome n'a ni eCADSTAR ni accès au disque.
- Si le rapport montre une option d'export non essayée qui pourrait écrire hauteur ou contour : refais l'export avec, sous le même nom, et redonne la consigne.
- Après l'essai, tu peux effacer `essai-ecadstar.xml` du dossier `IPC2581_Exemple` (il n'est pas suivi par git).

## Après
Pour Claude Code, avec le rapport :
- Les deux présents et lus (aucun corps estimé à l'étape 3) :
  - [ ] cocher `web_cao/A-FAIRE.md:338` et la mention `web_cao/A-FAIRE.md:47`.
- Contour ou hauteur ailleurs :
  - [ ] ajouter ce chemin au parseur (`web_cao/python/ipc2581_parser.py:2157-2172`, puis `ipc2581_json.py`, recopie dans WEB_ANTENNA) ;
  - [ ] faire de l'extrait `<Package>` du rapport un essai de `web_cao/visionneuse-ipc2581/test/banc-essai.py`, puis cocher.
- Absents :
  - [ ] noter dans `web_cao/A-FAIRE.md:338` qu'eCADSTAR ne les écrit pas (version et options essayées), et cocher : le repli actuel est le bon.
