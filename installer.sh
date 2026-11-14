#!/bin/sh
# Telechargement des outils WEB_SUITE (Linux, Raspberry Pi) : demande lequel installer (ou les trois).
cd "$(dirname "$0")" || exit 1
. ./binaires.sh
export WEB_SUITE_DEPOT="$PWD"
obtenir_binaires && bin/installer "$@"
echo
printf "Appuyez sur Entree pour fermer..."; read -r _
