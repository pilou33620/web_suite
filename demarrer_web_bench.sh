#!/bin/sh
# Banc d'essai web_bench (WEB_CAO, WEB_ANTENNA, WEB_3D) : garde le terminal ouvert en cas d'erreur.
cd "$(dirname "$0")" || exit 1
if ! python3 web_bench/web_bench.py "$@"; then
  echo
  echo "[X] Le banc s'est arrete sur une erreur. La raison est ecrite au-dessus."
  echo
  printf "Appuyez sur Entree pour fermer..."; read -r _
fi
