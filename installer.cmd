@echo off
rem Telechargement des outils WEB_SUITE : demande lequel installer (ou les trois).
cd /d "%~dp0"
set LANCEUR=py
where py >nul 2>nul || set LANCEUR=python
%LANCEUR% installer.py %*
echo.
pause
