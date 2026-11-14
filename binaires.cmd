@echo off
rem Met les executables de WEB_SUITE dans bin\ (appele par les autres .cmd, depuis la racine).
rem Avec cargo sur le poste : compiles ici, toujours a jour avec les sources.
rem Sinon : telecharges depuis GitHub (publication "binaires", compilee par la CI a chaque
rem envoi sur main), une fois par version : bin\commit dit de quelle version ils viennent.
setlocal
rem archive de production (publication "binaires", sans les sources) : ses executables sont deja la
if not exist Cargo.toml if exist bin\web_suite.exe exit /b 0
if not exist bin mkdir bin
where cargo >nul 2>nul
if not errorlevel 1 (
  cargo build --release --bins || exit /b 1
  rem un executable en cours d'usage (le serveur MCP) ne se reecrit pas : il est garde tel quel
  copy /y target\release\*.exe bin\ >nul 2>nul
  exit /b 0
)
set ICI=
for /f %%c in ('git rev-parse HEAD 2^>nul') do set ICI=%%c
set LA=
if exist bin\commit set /p LA=<bin\commit
if exist bin\web_suite.exe if defined ICI if "%ICI%"=="%LA%" exit /b 0
echo   Telechargement des executables WEB_SUITE (windows-x86_64)...
curl -fsSL -o bin\.archive.zip https://github.com/pilou33620/web_suite/releases/download/binaires/web_suite-windows-x86_64.zip
if not errorlevel 1 tar -xf bin\.archive.zip -C bin
if not errorlevel 1 (
  del bin\.archive.zip >nul 2>nul
  exit /b 0
)
del bin\.archive.zip >nul 2>nul
if exist bin\web_suite.exe (
  echo   [!] Telechargement impossible : on garde les executables deja presents.
  exit /b 0
)
echo [X] Telechargement des executables impossible.
echo     Installer Rust (https://rustup.rs) puis relancer : ils seront compiles ici.
exit /b 1
