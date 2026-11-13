#!/bin/sh
# Lancement de WEB_SUITE (Linux, Raspberry Pi) : garde le terminal ouvert en cas d'erreur.
cd "$(dirname "$0")" || exit 1
. ./binaires.sh
# Le lanceur sort avec le code 75 quand il vient de se mettre a jour (git pull) :
# on reprend les executables de la nouvelle version, puis on le relance.
export WEB_SUITE_RELANCEUR=1 WEB_SUITE_DEPOT="$PWD"
while :; do
  obtenir_binaires && bin/web_suite "$@"
  code=$?
  [ "$code" -eq 75 ] || break
  export WEB_SUITE_DEJA_MAJ=1
done
if [ "$code" -ne 0 ]; then
  echo
  echo "[X] Le demarrage a echoue. La raison est ecrite au-dessus."
  echo
  printf "Appuyez sur Entree pour fermer..."; read -r _
fi
