#!/bin/sh
# Banc d'essai web_bench (WEB_CAO, WEB_ANTENNA, WEB_3D) : garde le terminal ouvert en cas d'erreur.
cd "$(dirname "$0")" || exit 1
. ./binaires.sh
export WEB_SUITE_DEPOT="$PWD"
if ! { obtenir_binaires && bin/web_bench "$@"; }; then
  echo
  echo "[X] Le banc s'est arrete sur une erreur. La raison est ecrite au-dessus."
  echo
  printf "Appuyez sur Entree pour fermer..."; read -r _
fi
