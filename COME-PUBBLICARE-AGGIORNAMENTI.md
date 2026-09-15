# Come pubblicare un aggiornamento

Questa procedura verrà completata dopo il primo caricamento del progetto su GitHub.

## Regole fondamentali

1. Non caricare mai `registro-presenze.key` nel repository.
2. Non comunicare mai la password della chiave.
3. Aumentare sempre la versione in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` e `src/index.html`.
4. Pubblicare una versione soltanto dopo aver verificato che l’app si avvii e che i dati vengano salvati.

## Funzionamento

Il workflow GitHub compila automaticamente tre installer:

- macOS Apple Silicon, per Mac M1/M2/M3/M4 e successivi;
- macOS Intel;
- Windows a 64 bit.

Al termine crea una Release pubblica con il file `latest.json` e gli installer. Il workflow calcola anche l’impronta SHA-256 dell’installer Windows e la inserisce nel manifesto. Le copie già installate di Registro Presenze leggono quel file, mostrano l’avviso e installano l’aggiornamento soltanto dopo la conferma dell’utente.

La versione Windows 1.4.5 va installata manualmente una sola volta perché le versioni precedenti contengono ancora il vecchio updater. Dalla 1.4.5 in poi, **Scarica e installa** scarica l’installer, ne controlla l’impronta e lo avvia automaticamente senza aprire il terminale.

## Dati degli utenti

Gli aggiornamenti sostituiscono esclusivamente il programma. Il database SQLite e i backup nella cartella Documenti non fanno parte dell’installer e non vengono cancellati.
