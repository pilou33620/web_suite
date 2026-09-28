@echo off
rem Lancement de WEB_SUITE, a double-cliquer : garde la fenetre ouverte en cas derreur.
cd /d "%~dp0"
set LANCEUR=py
where py >nul 2>nul || set LANCEUR=python
%LANCEUR% lanceur\web_suite.py %*
if errorlevel 1 (
  echo.
  echo [X] Le demarrage a echoue. La raison est ecrite au-dessus.
  echo.
  pause
)
