@echo off
rem Telechargement des outils WEB_SUITE : demande lequel installer (ou les trois).
cd /d "%~dp0"
set WEB_SUITE_DEPOT=%CD%
call binaires.cmd
if not errorlevel 1 bin\installer.exe %*
echo.
pause
