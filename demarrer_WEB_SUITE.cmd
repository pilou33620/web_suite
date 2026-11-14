@echo off
rem Lancement de WEB_SUITE, a double-cliquer : garde la fenetre ouverte en cas derreur.
cd /d "%~dp0"
rem Le lanceur sort avec le code 75 quand il vient de se mettre a jour (git pull) :
rem on reprend les executables de la nouvelle version, puis on le relance.
set WEB_SUITE_RELANCEUR=1
set WEB_SUITE_DEPOT=%CD%
:encore
call binaires.cmd
if errorlevel 1 goto echec
bin\web_suite.exe %*
if %errorlevel%==75 (
  set WEB_SUITE_DEJA_MAJ=1
  goto encore
)
if not errorlevel 1 exit /b 0
:echec
echo.
echo [X] Le demarrage a echoue. La raison est ecrite au-dessus.
echo.
pause
