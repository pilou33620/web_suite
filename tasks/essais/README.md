# Essais à faire toi-même

Ce que Claude ne peut pas faire seul : il faut le Pi, le PC Windows, l'iPad, une vraie pièce
ou une vraie carte, une norme, ou des heures de calcul openEMS. Une spec par essai : pourquoi,
ce qu'il faut, les étapes avec ce que tu dois voir, ce qui fait la réussite, quoi me renvoyer
si ça rate, et les cases à cocher après. Écrites le 21/10/2026.

Pour un essai qui écrit, toujours une **copie** de PROJETS (chaque spec dit comment).
Les renvois `fichier:ligne` valent au 21/10/2026 : ils glissent au prochain changement.

## Sur le Pi

| Essai | Durée |
|---|---|
| [Claude Code relancé avec le MCP](ESSAI-pi-mcp-claude-code.md) (à faire en premier : les autres s'en servent) | 20 min |
| [`cao_chemin`, durée d'un appel de 20 liaisons](ESSAI-pi-chemin-auto-duree.md) | 30 min |
| [Animation 3D de `fabrication.zip`](ESSAI-pi-animation-3d.md) | — |
| [Tutoriel de la Gestion LIB dans le Chromium du Pi](ESSAI-pi-tutoriel-gestion-lib.md) | 15 min |

## Depuis le PC Windows

| Essai | Durée |
|---|---|
| [Le MCP par SSH vers le Pi (skill absent, `savoir_lire`)](ESSAI-windows-ssh-mcp.md) | 30 min |

## Sur l'iPad

| Essai | Durée |
|---|---|
| [Le canal de pilotage depuis l'iPad appairé](ESSAI-ipad-canal-pilotage.md) | 45 min |
| [Navigation tactile de WEB_3D](ESSAI-ipad-tactile-web3d.md) | 15 min |

## WEB_ANTENNA : calculs openEMS

Chaque spec finit par un relevé JSON fait par [`ESSAI-antenna-releve.py`](ESSAI-antenna-releve.py) :
donne-le-moi, je remplis A-FAIRE et corrige le gabarit.

| Essai | Durée | À trancher avant |
|---|---|---|
| [MIFA, `kd` × `n`](ESSAI-antenna-mifa-kd-n.md) | 12 points | bande élargie à 1,5–4,5 GHz (le MIFA résonne au-dessus de 3,5 GHz) |
| [Monopôle, `wr` × `Lm`](ESSAI-antenna-monopole-wr-lm.md) | 12 points | — |
| [Facteurs dipôle, monopôle, IFA hors de leur cas](ESSAI-antenna-facteurs-5g8.md) | 1 h à 1 h 30 | — |
| [Patch, balayage fin de `y₀`](ESSAI-antenna-patch-y0-fin.md) | 40 min à 1 h | **oui** : la plage du plan ne trouvera pas le zéro avec le gabarit corrigé ; plage proposée 5,46 → 6,96 mm |
| [MSLPort, maillage ou port](ESSAI-antenna-mslport.md) | 30 à 45 min | — (décide ensuite : MSLPort par défaut ou en option) |
| [Patch, deux strates de plus pour `patchCorrRbord`](ESSAI-antenna-patch-strates.md) | 1 h 30 à 2 h 30 | **oui** : plan proposé, pas encore arrêté |

## WEB_CAO : pièces, cartes, fichiers

| Essai | Durée |
|---|---|
| [Broche 1 du support Yamaichi IC51-0484-806](ESSAI-cao-yamaichi-broche1.md) | 15 min |
| [Empreinte SOT-89-5 (Torex)](ESSAI-cao-sot89-5.md) | 30 min |
| [LED RVB, sens et couleurs](ESSAI-cao-led-rvb.md) | 20 min |
| [eCADSTAR : `<Package><Outline>` et `height` ?](ESSAI-cao-ecadstar-package.md) | 20 min |
| [Carte TRD1_N en « ETH 100 » et la règle `retour`](ESSAI-cao-trd1n-eth100.md) | 20 min |
| [Simulation RF de P01x274 : tes réponses et les fichiers](ESSAI-cao-rf-p01x274.md) | 30 min |
| [Dossier méca : télécharger et importer le DXF dans une CAO méca](ESSAI-cao-dossier-meca-import.md) | 30 min |
| [Dossier méca : une IA lit le JSON (critère 6)](ESSAI-cao-dossier-meca-ia.md) | 20 min |

## WEB_CAO : normes à relire

| Essai | Durée |
|---|---|
| [USB 2.0 Template 2, figure 7-15](ESSAI-cao-norme-usb2-template2.md) (A-FAIRE se contredit : « recoupé » ici, « à relire » là) | 20 min |
| [Rugosité de Huray, a et SR](ESSAI-cao-huray.md) | 1 h |
| [IBIS, sections d'un boîtier](ESSAI-cao-norme-ibis-sections.md) | 20 min |
| [IPC-7351B, constantes des puces](ESSAI-cao-norme-ipc7351b-puces.md) (norme payante) | 20 min |
| [HDMI 1.4b, masque à TP2](ESSAI-cao-norme-hdmi-tp2.md) (accès adoptant) | 15 min |

## WEB_CAO : relectures de code

| Essai | Durée |
|---|---|
| [Classes d'interface (USB, Ethernet, LVDS, CAN, RS-485)](ESSAI-cao-relecture-classes-interface.md) | 1 h à 1 h 30 |
| [Dossier mécanique](ESSAI-cao-relecture-dossier-meca.md) | 1 h |
