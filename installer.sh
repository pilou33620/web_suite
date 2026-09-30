#!/bin/sh
# Telechargement des outils WEB_SUITE (Linux, Raspberry Pi) : demande lequel installer (ou les trois).
cd "$(dirname "$0")" || exit 1
python3 lanceur/installer.py "$@"
echo
printf "Appuyez sur Entree pour fermer..."; read -r _
