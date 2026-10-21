# Essai : le tutoriel de la Gestion LIB dans le Chromium du Pi

Où : Pi, **Chromium du Pi avec son écran** (branché ou bureau à distance), pas en headless ·
Outils : WEB·SUITE (WEB_CAO lancé) · Durée : 15 min · Source : `A-FAIRE.md:71-72`

## Pourquoi

`A-FAIRE.md:71` : « son tutoriel ne passe pas en Chromium headless sur le Pi ». Le relevé
ne dit ni à quelle étape ça bloque, ni avec quel message. Il vient des essais du 19/10/2026
(`tasks/todo-composants.md`, tâche 7), où la Gestion LIB montait à environ 890 Mo au
chargement sur le Pi (500 Mo ensuite), jusqu'à faire tuer l'onglet d'essai par le noyau
(OOM). Ce point mémoire a été retiré de la liste le 21/10/2026.

L'essai dit si c'est la Gestion LIB ou le mode headless qui pèche. Si le tutoriel passe dans
un vrai Chromium du Pi, le défaut ne tient qu'au headless. Sinon, on sait où il casse.

Ce que fait le tutoriel (`WEB_CAO/commun/tuto.js:117-136`, chargé par
`gestion-lib/gestion-lib.html:412`) : neuf étapes, chacune une bulle posée sur un élément de
la page. Seules deux actions changent la page, et ce sont des clics d'onglet
(`faire: lib("composants")` ou `lib("pcb")`, l.47). Rien n'est écrit : l'étape
« Enregistrer » montre le bouton sans le presser. Pas besoin de copie de PROJETS, mais ne
clique pas toi-même **Enregistrer**.

## Avant de commencer

1. Mets à jour : démarre WEB·SUITE (`cd ~/Documents/web_suite && ./demarrer_WEB_SUITE.sh`),
   le terminal dit `WEB_CAO  installé · À jour.` (ou `Mis à jour …`).
2. **▶ Lancer** sur WEB_CAO, dans la page WEB·SUITE.
3. Dans le Chromium du Pi, ferme les autres onglets lourds. Ouvre le gestionnaire de tâches
   de Chromium (**Maj+Échap**) et laisse-le ouvert à côté. Dans un terminal : `free -m`, et
   note la ligne `Mem:`.

## Étapes

1. Ouvre `http://127.0.0.1:8000/` (l'accueil WEB_CAO), clique **🎓 Mode tuto** (il s'allume),
   puis la carte **Gestion LIB**.
   → attendu : un nouvel onglet `gestion-lib/gestion-lib.html`. Le catalogue se charge ; une
   bulle « Mode tuto · 4 / 5 », « Gestion LIB », « 9 étapes pour en faire le tour »,
   **Plus tard** / **Commencer ›**.
2. Pendant le chargement, regarde le gestionnaire de tâches.
   → attendu : l'onglet Gestion LIB monte puis se stabilise. **Note le pic et la valeur
   stable** (colonne « Empreinte mémoire »).
3. **Commencer ›**, puis **Suivant ›** (ou `→`) à chaque étape.
   → attendu, « étape i / 9 » en tête de bulle :
   1. « Bienvenue dans la gestion LIB » (bulle au centre)
   2. « Quatre vues » : les onglets entourés
   3. « Associations » : le bandeau des taux
   4. « Chercher » : le champ de recherche
   5. « L'inspecteur » : le panneau de droite
   6. « Empreintes » : la vue passe aux empreintes PCB
   7. « Importer JLCPCB / LCSC » : retour au catalogue, le bouton d'import entouré
   8. « Auto-associer »
   9. « Enregistrer », avec « Suite du parcours : Visionneuse IPC-2581. »
4. À chaque étape, regarde si la zone éclairée tombe sur le bon élément, et si la page répond
   encore (défilement, survol).
   → attendu : aucune étape sans cadre alors qu'elle en a un (seule la 1 est au centre),
   aucune bulle hors de l'écran, pas d'onglet tué (« Oh, non ! » / « Aïe, aïe, aïe » de Chromium).
5. Sur la dernière bulle, **Visionneuse IPC-2581 ›**.
   → attendu : la visionneuse s'ouvre avec sa propre proposition de visite.
6. Reviens à la Gestion LIB et recharge-la (sans `?tuto`). Le mode tuto est toujours allumé.
   → attendu : des pastilles **?** sur les grandes fonctions ; leur survol redonne
   l'explication de l'étape.
7. `free -m` de nouveau.

## C'est réussi si

- [ ] Les neuf étapes défilent jusqu'au bout, chacune sur le bon élément
- [ ] L'onglet n'est pas tué, la page reste utilisable pendant et après
- [ ] La dernière étape mène à la visionneuse
- [ ] Le pic et la mémoire stable de l'onglet sont notés

## Si ça rate, note

- L'étape (numéro et titre) où ça bloque, et ce qu'on voit : bulle absente, cadre au mauvais
  endroit, page figée, onglet tué. Une capture d'écran.
- La console (F12 → Console) : les lignes rouges, telles quelles.
- Le gestionnaire de tâches au moment du blocage (capture), et `free -m`.
- Si l'onglet a été tué : `journalctl -k --since "10 min ago" | grep -i -E "oom|killed process"`.
- Le modèle et la mémoire du Pi (`cat /proc/device-tree/model`, `free -h`), la version de
  Chromium (`chrome://version`).
- Le journal de WEB_CAO : bouton **Journal** de sa carte (`lanceur/journaux/web_cao.log`).

## Après

- Le tutoriel passe : `A-FAIRE.md:71-72`, écrire que dans un vrai Chromium du Pi il passe
  (date, mémoire notée), et que seul le headless pèche. Cocher si tu juges que le headless ne
  compte pas, sinon laisser la case pour le headless seul.
- Il casse : `A-FAIRE.md:71-72`, remplacer « ne passe pas en Chromium headless » par l'étape
  et le symptôme relevés. Le défaut est à corriger dans WEB_CAO : l'ajouter aussi à
  `WEB_CAO/A-FAIRE.md`, qui n'en parle pas encore.
