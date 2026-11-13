#!/bin/sh
# Met les executables de WEB_SUITE dans bin/ (a inclure par « . ./binaires.sh » depuis la racine).
# Avec cargo sur le poste : compiles ici, toujours a jour avec les sources.
# Sinon : telecharges depuis GitHub (publication « binaires », compilee par la CI a chaque
# envoi sur main), une fois par version : bin/commit dit de quelle version ils viennent.
DEPOT_BINAIRES="https://github.com/pilou33620/web_suite/releases/download/binaires"

obtenir_binaires() {
  mkdir -p bin
  if command -v cargo >/dev/null 2>&1; then
    cargo build --release --bins || return 1
    for f in target/release/*; do
      if [ -f "$f" ] && [ -x "$f" ]; then
        # copie puis renommage : un executable en cours d'usage (le serveur MCP) ne se reecrit pas
        cp "$f" "bin/.neuf" && mv -f "bin/.neuf" "bin/$(basename "$f")"
      fi
    done
    return 0
  fi
  ici=$(git rev-parse HEAD 2>/dev/null)
  if [ -x bin/web_suite ] && [ -n "$ici" ] && [ "$(cat bin/commit 2>/dev/null)" = "$ici" ]; then
    return 0
  fi
  case "$(uname -s)" in Linux) os=linux ;; Darwin) os=macos ;; *) os=inconnu ;; esac
  case "$(uname -m)" in x86_64|amd64) cpu=x86_64 ;; aarch64|arm64) cpu=aarch64 ;; *) cpu=inconnu ;; esac
  echo "  Telechargement des executables WEB_SUITE ($os-$cpu)..."
  if curl -fsSL -o bin/.archive "$DEPOT_BINAIRES/web_suite-$os-$cpu.tar.gz" && tar -xzf bin/.archive -C bin; then
    rm -f bin/.archive
    return 0
  fi
  rm -f bin/.archive
  if [ -x bin/web_suite ]; then
    echo "  [!] Telechargement impossible : on garde les executables deja presents."
    return 0
  fi
  echo "[X] Pas d'executable pour ce poste ($os-$cpu) et telechargement impossible."
  echo "    Installer Rust (https://rustup.rs) puis relancer : ils seront compiles ici."
  return 1
}
