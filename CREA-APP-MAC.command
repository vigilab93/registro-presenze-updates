#!/bin/zsh
set -e

PROJECT_DIR="${0:A:h}"
cd "$PROJECT_DIR"

if ! command -v node >/dev/null 2>&1; then
  osascript -e 'display dialog "Manca Node.js. Installa la versione LTS da nodejs.org, poi riapri questo file." buttons {"OK"} default button "OK" with icon caution'
  open "https://nodejs.org/it/download"
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  osascript -e 'display dialog "Manca Rust. Apri il file GUIDA-PASSO-PASSO.md e completa il Passo 2, poi riapri questo file." buttons {"OK"} default button "OK" with icon caution'
  exit 1
fi

if ! xcode-select -p >/dev/null 2>&1; then
  osascript -e 'display dialog "Mancano gli strumenti Apple. Premi Installa nella finestra che apparirà, poi riapri questo file." buttons {"OK"} default button "OK" with icon caution'
  xcode-select --install
  exit 1
fi

SIGNING_KEY_PATH="$HOME/.registro-presenze/registro-presenze.key"
if [[ ! -f "$SIGNING_KEY_PATH" ]]; then
  osascript -e 'display dialog "Manca la chiave privata degli aggiornamenti. Non è possibile creare una versione aggiornabile." buttons {"OK"} default button "OK" with icon caution'
  exit 1
fi

SIGNING_PASSWORD=$(osascript -e 'text returned of (display dialog "Inserisci la password della chiave aggiornamenti. Non verrà mostrata né salvata." default answer "" with hidden answer buttons {"Continua"} default button "Continua" with icon note)')
export TAURI_SIGNING_PRIVATE_KEY_PATH="$SIGNING_KEY_PATH"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$SIGNING_PASSWORD"
trap 'unset TAURI_SIGNING_PRIVATE_KEY_PATH TAURI_SIGNING_PRIVATE_KEY_PASSWORD SIGNING_PASSWORD' EXIT

npm install
npm run build -- --bundles app,dmg

DMG_DIR="$PROJECT_DIR/src-tauri/target/release/bundle/dmg"
APP_DIR="$PROJECT_DIR/src-tauri/target/release/bundle/macos"

osascript -e 'display dialog "Versione Mac creata correttamente. Si aprirà la cartella con il file DMG." buttons {"Apri cartella"} default button "Apri cartella" with icon note'
if [[ -d "$DMG_DIR" ]]; then
  open "$DMG_DIR"
else
  open "$APP_DIR"
fi
