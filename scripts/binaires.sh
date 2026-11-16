#!/bin/sh
# Met les executables de WEB_SUITE dans bin/ (a inclure par « . ./binaires.sh » depuis la racine).
# Avec cargo sur le poste : compiles ici, toujours a jour avec les sources.
# Sinon : telecharges depuis GitHub (publication « binaires », compilee par la CI a chaque
# envoi sur main), une fois par version : bin/commit dit de quelle version ils viennent.
DEPOT_BINAIRES="https://github.com/pilou33620/web_suite/releases/download/binaires"

# L'archive telechargee doit avoir l'empreinte SHA-256 publiee avec elle : tronquee ou alteree en
# route, elle est refusee. (L'empreinte vient du meme depot : elle ne protege pas d'un compte vole.)
empreinte_juste() {
  if command -v sha256sum >/dev/null 2>&1; then recu=$(sha256sum bin/.archive); else recu=$(shasum -a 256 bin/.archive); fi
  [ -n "$recu" ] && [ "${recu%% *}" = "$(cat bin/.empreinte)" ] && return 0
  echo "  [!] Empreinte de l'archive incorrecte : archive refusee."
  return 1
}

obtenir_binaires() {
  # archive de production (publication « binaires », sans les sources) : ses executables sont deja la
  if [ ! -f Cargo.toml ] && [ -x bin/web_suite ]; then return 0; fi
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
  archive="web_suite-$os-$cpu.tar.gz"
  if curl -fsSL -o bin/.archive "$DEPOT_BINAIRES/$archive" \
     && curl -fsSL -o bin/.empreinte "$DEPOT_BINAIRES/$archive.sha256" \
     && empreinte_juste && tar -xzf bin/.archive -C bin; then
    rm -f bin/.archive bin/.empreinte
    return 0
  fi
  rm -f bin/.archive bin/.empreinte
  if [ -x bin/web_suite ]; then
    echo "  [!] Telechargement impossible : on garde les executables deja presents."
    return 0
  fi
  echo "[X] Pas d'executable pour ce poste ($os-$cpu) et telechargement impossible."
  echo "    Installer Rust (https://rustup.rs) puis relancer : ils seront compiles ici."
  return 1
}
