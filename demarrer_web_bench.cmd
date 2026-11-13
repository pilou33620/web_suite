@echo off
rem Banc d'essai web_bench (WEB_CAO, WEB_ANTENNA, WEB_3D), a double-cliquer.
cd /d "%~dp0"
set WEB_SUITE_DEPOT=%CD%
call binaires.cmd
if not errorlevel 1 bin\web_bench.exe %*
if errorlevel 1 (
  echo.
  echo [X] Le banc s'est arrete sur une erreur. La raison est ecrite au-dessus.
  echo.
  pause
)
