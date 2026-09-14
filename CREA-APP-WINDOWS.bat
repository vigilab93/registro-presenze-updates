@echo off
setlocal
cd /d "%~dp0"

where node >nul 2>&1
if errorlevel 1 (
  echo ERRORE: Node.js non e installato.
  echo Installa la versione LTS da https://nodejs.org/it/download
  pause
  exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
  echo ERRORE: Rust non e installato.
  echo Apri GUIDA-PASSO-PASSO.md e completa il Passo 2 per Windows.
  pause
  exit /b 1
)

echo Installazione componenti del progetto...
call npm install
if errorlevel 1 goto :errore

echo Creazione applicazione Windows...
call npm run build -- --bundles nsis
if errorlevel 1 goto :errore

echo.
echo VERSIONE WINDOWS CREATA CORRETTAMENTE.
start "" "%~dp0src-tauri\target\release\bundle\nsis"
pause
exit /b 0

:errore
echo.
echo La compilazione non e riuscita. Fotografa questa finestra e inviala a Codex.
pause
exit /b 1

