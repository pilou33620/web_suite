@echo off
rem Banc d'essai web_bench (WEB_CAO, WEB_ANTENNA, WEB_3D), a double-cliquer.
cd /d "%~dp0"
set LANCEUR=py
where py >nul 2>nul || set LANCEUR=python
%LANCEUR% web_bench\web_bench.py %*
if errorlevel 1 (
  echo.
  echo [X] Le banc s'est arrete sur une erreur. La raison est ecrite au-dessus.
  echo.
  pause
)
