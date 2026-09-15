# Registro Presenze — guida passo passo

Questa cartella contiene un solo progetto che produce:

- `Registro Presenze.app` e un file `.dmg` per macOS;
- un installer `.exe` per Windows.

L’app non usa la memoria di Chrome, Safari o Edge. Ogni modifica viene salvata in un database SQLite e copiata automaticamente nella cartella `Documenti/Registro Presenze/Backup`.

La versione 1.1.0 introduce gli aggiornamenti firmati. Dalla versione 1.3.0 l’app controlla automaticamente gli aggiornamenti all’avvio e periodicamente. Su Windows può installarli e riavviarsi; su macOS apre direttamente la pagina dell’ultima versione per scaricare il file `aarch64.dmg`, evitando il difetto di sostituzione dell’app presente nell’updater Tauri attuale. I dati personali non vengono inclusi negli aggiornamenti e restano sul computer.

La versione 1.4.0 nasconde gli importi all’avvio: il pulsante **Mostra saldi** li mostra o li nasconde insieme nell’app e nel report di stampa. Aggiunge inoltre il conteggio delle ferie godute nell’anno, consente di registrare in anticipo l’orario di uscita e forza il report mensile su una sola pagina A4 verticale. Su Windows l’app si avvia senza aprire una finestra del terminale.

La versione 1.4.1 ripristina l’orologio intelligente anche nei giorni ancora vuoti: clicca sulle ore, digita due cifre e vengono selezionati automaticamente i minuti.

## Fase A — preparare il Mac di Vigi

Questa preparazione si esegue una sola volta sul Mac usato per creare la versione Mac.

### Passo 1 — strumenti Apple

1. Apri **Terminale** dal Launchpad.
2. Incolla questo comando e premi Invio:

   ```bash
   xcode-select --install
   ```

3. Se appare una finestra, premi **Installa** e attendi la conclusione.

### Passo 2 — installare Rust

Nel Terminale incolla:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Quando compare la scelta, premi **1** e poi Invio. Al termine chiudi e riapri Terminale.

### Passo 3 — installare Node.js

1. Vai su <https://nodejs.org/it/download>.
2. Scarica la versione **LTS per macOS**.
3. Apri il pacchetto e completa l’installazione.

### Passo 4 — creare l’app Mac

1. Fai clic destro su `CREA-APP-MAC.command`.
2. Scegli **Apri**.
3. Conferma nuovamente **Apri** se macOS mostra un avviso.
4. Attendi: la prima compilazione può richiedere diversi minuti.
5. Quando richiesto, inserisci la password della chiave aggiornamenti.
6. Al termine si aprirà la cartella contenente il `.dmg`.

Il file `.dmg` è quello da conservare e passare ai colleghi con Mac.

## Fase B — preparare il PC Windows

Questa preparazione si esegue una sola volta su un PC Windows usato per creare la versione Windows.

### Passo 1 — Microsoft C++ Build Tools

1. Vai su <https://visualstudio.microsoft.com/visual-cpp-build-tools/>.
2. Scarica e avvia **Build Tools for Visual Studio**.
3. Seleziona **Sviluppo di applicazioni desktop con C++**.
4. Completa l’installazione e riavvia Windows.

### Passo 2 — Rust

1. Vai su <https://www.rust-lang.org/tools/install>.
2. Scarica ed esegui `rustup-init.exe`.
3. Accetta l’installazione predefinita.

### Passo 3 — Node.js

1. Vai su <https://nodejs.org/it/download>.
2. Scarica la versione **LTS per Windows**.
3. Completa l’installazione predefinita.

### Passo 4 — creare l’app Windows

1. Copia l’intera cartella `RegistroPresenze-Tauri` sul PC Windows.
2. Fai doppio clic su `CREA-APP-WINDOWS.bat`.
3. Attendi la compilazione.
4. Al termine si aprirà la cartella contenente l’installer `.exe`.

## Primo avvio dei colleghi

### Mac

1. Aprire il `.dmg`.
2. Trascinare **Registro Presenze** in **Applicazioni**.
3. Al primo avvio usare clic destro sull’app → **Apri** → **Apri**.

### Windows

1. Avviare l’installer `.exe`.
2. Se appare SmartScreen: **Ulteriori informazioni** → **Esegui comunque**.
3. Terminare l’installazione.

Questi avvisi possono comparire perché l’app è interna e non utilizza certificati commerciali a pagamento.

## Recuperare i dati del vecchio HTML

1. Apri il vecchio file HTML.
2. Premi **Scarica backup**.
3. Apri la nuova applicazione.
4. Vai in **Impostazioni** → **Importa backup**.
5. Seleziona il file `.json` scaricato.

Entrate, uscite, ferie, assenze, note e impostazioni verranno trasferite.

## Dove sono i backup

L’app crea automaticamente:

- `Documenti/Registro Presenze/Backup/registro-presenze-ultimo.json`;
- una copia giornaliera con la data;
- fino a 30 copie giornaliere precedenti.

Se l’archivio principale è danneggiato o assente, all’avvio l’app tenta automaticamente il recupero dall’ultimo backup valido.

## Aggiornamenti automatici

- L’app controlla automaticamente gli aggiornamenti dopo l’avvio.
- Il controllo manuale si trova in **Impostazioni → Aggiornamenti**.
- Quando trova una nuova versione, l’app la scarica e la verifica automaticamente in background.
- L’utente può scegliere **Più tardi** oppure **Installa e riavvia**.
- Database, impostazioni e backup non vengono sostituiti dall’aggiornamento.
- Gli aggiornamenti vengono accettati soltanto se firmati con la chiave privata originale.

La chiave privata attiva si trova sul Mac di Vigi in `~/.registro-presenze/registro-presenze-v2.key`. Non deve essere caricata nel repository o passata ai colleghi. Anche la password deve essere conservata separatamente.
