# Essais

Ce que Claude Code ne peut pas vérifier seul. La plupart se donnent à **Claude in Chrome** sur
le PC Windows du boulot, WEB·SUITE lancé par double-clic : lis d'abord
[CLAUDE-IN-CHROME.md](CLAUDE-IN-CHROME.md) (règles, adresses, format du rapport). Chaque spec
dit en tête **qui** fait quoi ; quand Claude in Chrome a une part, la spec a un bloc
« Consigne pour Claude in Chrome » à lui coller tel quel. Son rapport, tu me le redonnes.

Les consignes renvoient aux fichiers de `main` sur GitHub : ils doivent y être (et le PC du
boulot à jour) avant le premier essai. Les renvois `fichier:ligne` valent au 21/10/2026.

Depuis le 22/10/2026, chaque outil (web_suite, WEB_CAO, WEB_ANTENNA, WEB_3D) liste ses essais
ouverts dans son `TESTS-A-FAIRE.md` : un essai fait s'y coche, et ce qu'il demande à coder va
dans son `A-FAIRE.md`. Les renvois `A-FAIRE.md:ligne` des specs visent les A-FAIRE d'avant cette
séparation, gardés tels quels dans `HISTORIQUE.md` (web_suite, WEB_ANTENNA) et à la fin de
`docs/HISTORIQUE_DEVELOPPEMENT.md` (WEB_CAO).

## Claude in Chrome seul

| Essai | Note |
|---|---|
| [MIFA, `kd` × `n`](ESSAI-antenna-mifa-kd-n.md) | calcul openEMS, 12 points |
| [Monopôle, `wr` × `Lm`](ESSAI-antenna-monopole-wr-lm.md) | calcul openEMS, 12 points |
| [Facteurs dipôle, monopôle, IFA hors de leur cas](ESSAI-antenna-facteurs-5g8.md) | calcul openEMS, 6 points |
| [MSLPort, maillage ou port](ESSAI-antenna-mslport.md) | calcul openEMS ; ensuite tu décides : par défaut ou en option |
| [Patch, balayage fin de `y₀`](ESSAI-antenna-patch-y0-fin.md) | **à valider avant** (plage à choisir) : sinon Claude in Chrome ne lance rien |
| [Patch, deux strates de plus](ESSAI-antenna-patch-strates.md) | **à valider avant** (plan proposé) |
| ~~[IBIS, sections d'un boîtier](ESSAI-cao-norme-ibis-sections.md)~~ | **fait le 22/10/2026** : sens puce → broche confirmé |
| ~~[Rugosité de Huray, a et SR](ESSAI-cao-huray.md)~~ | **fait le 22/10/2026** : passé à Simonovich |
| [Dossier méca : une IA lit le JSON (critère 6)](ESSAI-cao-dossier-meca-ia.md) | dans claude.ai ; contrôle final par toi |
| [Relecture : classes d'interface](ESSAI-cao-relecture-classes-interface.md) | code lu sur github.com ; le verdict est le tien |
| [Relecture : dossier mécanique](ESSAI-cao-relecture-dossier-meca.md) | code lu sur github.com ; le verdict est le tien |

Pour les calculs openEMS : un seul onglet, jamais rechargé ; la veille de Windows sur
« Jamais » ; `127.0.0.1` dans les sites toujours actifs de l'économiseur de mémoire de Chrome.

## Claude in Chrome et toi

| Essai | Ta part |
|---|---|
| [Yamaichi IC51-0484-806, broche 1](ESSAI-cao-yamaichi-broche1.md) | le support en main (documents lus le 22/10/2026) |
| [Empreinte SOT-89-5 (Torex)](ESSAI-cao-sot89-5.md) | la carte en main (empreinte refaite le 22/10/2026) |
| [LED RVB, sens et couleurs](ESSAI-cao-led-rvb.md) | allumer la LED (datasheet lue le 22/10/2026) |
| [eCADSTAR : `<Package><Outline>` et `height` ?](ESSAI-cao-ecadstar-package.md) | l'export eCADSTAR, déposé dans `WEB_CAO\IPC2581_Exemple\` |
| [Carte TRD1_N en « ETH 100 »](ESSAI-cao-trd1n-eth100.md) | déposer les fichiers de la carte |
| ~~[USB 2.0 Template 2, figure 7-15](ESSAI-cao-norme-usb2-template2.md)~~ | **fait le 22/10/2026** : figure 7-14, points justes |
| [Simulation RF de P01x274](ESSAI-cao-rf-p01x274.md) | les réponses aux questions 2 à 5, les fichiers, la mesure |
| [Dossier méca : télécharger et importer le DXF](ESSAI-cao-dossier-meca-import.md) | ouvrir le zip, importer le DXF dans la CAO méca |
| [`cao_chemin`, durée de 20 liaisons](ESSAI-pi-chemin-auto-duree.md) | la même mesure sur le Pi (PC fait le 22/10/2026) |
| [Animation 3D de `fabrication.zip`](ESSAI-pi-animation-3d.md) | la même chose dans le Chromium du Pi |
| [Tutoriel de la Gestion LIB](ESSAI-pi-tutoriel-gestion-lib.md) | « Empreinte mémoire » sur le PC, puis le Chromium du Pi (PC fait le 22/10/2026) |

## Toi seul

| Essai | Pourquoi pas Claude in Chrome |
|---|---|
| [Claude Code relancé avec le MCP, sur le Pi](ESSAI-pi-mcp-claude-code.md) | Claude Code et le MCP, pas un navigateur |
| [Le MCP depuis Windows par SSH](ESSAI-windows-ssh-mcp.md) | terminal et SSH |
| [Le canal de pilotage depuis l'iPad](ESSAI-ipad-canal-pilotage.md) | les commandes viennent du MCP ; l'iPad |
| [Navigation tactile de WEB_3D](ESSAI-ipad-tactile-web3d.md) | un vrai toucher sur l'iPad |
| [HDMI 1.4b, masque à TP2](ESSAI-cao-norme-hdmi-tp2.md) | norme réservée aux adoptants |
| [IPC-7351B, constantes des puces](ESSAI-cao-norme-ipc7351b-puces.md) | norme payante |

[`ESSAI-antenna-releve.py`](ESSAI-antenna-releve.py) : pour contrôler au nombre près un relevé
recopié par Claude in Chrome, si les projets d'essai sont envoyés sur GitHub (terminal requis).
