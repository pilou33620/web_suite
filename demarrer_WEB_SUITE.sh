#!/bin/sh
# Lancement de WEB_SUITE (Linux, Raspberry Pi) : garde le terminal ouvert en cas d'erreur.
cd "$(dirname "$0")" || exit 1
if ! python3 lanceur/web_suite.py "$@"; then
  echo
  echo "[X] Le demarrage a echoue. La raison est ecrite au-dessus."
  echo
  printf "Appuyez sur Entree pour fermer..."; read -r _
fi
