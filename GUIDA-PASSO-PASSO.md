# Registro Presenze — guida passo passo

Questa cartella contiene un solo progetto che produce:

- `Registro Presenze.app` e un file `.dmg` per macOS;
- un installer `.exe` per Windows.

L’app non usa la memoria di Chrome, Safari o Edge. Ogni modifica viene salvata in un database SQLite e copiata automaticamente nella cartella `Documenti/Registro Presenze/Backup`.

La versione 1.1.0 introduce gli aggiornamenti firmati. Dalla versione 1.3.0 l’app controlla automaticamente gli aggiornamenti all’avvio e periodicamente. Su Windows può installarli e riavviarsi; su macOS apre direttamente la pagina dell’ultima versione per scaricare il file `aarch64.dmg`, evitando il difetto di sostituzione dell’app presente nell’updater Tauri attuale. I dati personali non vengono inclusi negli aggiornamenti e restano sul computer.

La versione 1.4.0 nasconde gli importi all’avvio: il pulsante **Mostra saldi** li mostra o li nasconde insieme nell’app e nel report di stampa. Aggiunge inoltre il conteggio delle ferie godute nell’anno, consente di registrare in anticipo l’orario di uscita e forza il report mensile su una sola pagina A4 verticale. Su Windows l’app si avvia senza aprire una finestra del terminale.

La versione 1.4.1 ripristina l’orologio intelligente anche nei giorni ancora vuoti: clicca sulle ore, digita due cifre e vengono selezionati automaticamente i minuti.

La versione 1.4.2 distribuisce il tabulato mensile su tutta l’altezza disponibile del foglio A4 verticale, mantenendo il report completo in una sola pagina e rendendo i testi più leggibili.

La versione 1.4.3 paga lo straordinario soltanto al completamento di ogni scaglione di 30 minuti: nei giorni feriali 17:01–17:29 non aggiunge nulla, 17:30–17:59 aggiunge 30 minuti e dalle 18:00 ne aggiunge 60; il sabato applica la stessa regola dopo le 13:00. Su Windows **Scarica e installa** esegue download, verifica, installazione e riavvio in un unico passaggio automatico.

La versione 1.4.4 corregge la pubblicazione dell’aggiornamento Windows. Al termine delle compilazioni, GitHub prepara nuovamente `latest.json` usando i collegamenti diretti agli installer e le firme abbinate ai rispettivi file. In questo modo **Scarica e installa** riceve il vero installer Windows, ne verifica la firma e lo installa automaticamente.

La versione 1.4.5 introduce un percorso di aggiornamento Windows indipendente dalla firma Tauri che ha dato errore nelle release precedenti. L’app scarica direttamente l’installer NSIS, confronta la sua impronta SHA-256 con quella calcolata da GitHub durante la pubblicazione e, se coincide, lo avvia con i parametri ufficiali di aggiornamento `/P /UPDATE /R`.

La versione 1.4.6 corregge i confini dell’uscita anticipata: timbrare esattamente alle 16:30 assegna già la mezz’ora 16:00–16:30 e produce 7h 45m retribuite; alle 16:00 risultano 7h 15m. Serve anche come prima prova reale dell’aggiornamento automatico dalla versione Windows 1.4.5.

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
- Quando trova una nuova versione, l’utente può scegliere **Più tardi** oppure **Scarica e installa**.
- Su Windows l’app scarica l’installer senza aprire il terminale, verifica l’impronta SHA-256 pubblicata da GitHub e avvia automaticamente l’installazione e il riavvio.
- Su macOS resta attiva la verifica con la chiave di firma dell’aggiornamento.
- Database, impostazioni e backup non vengono sostituiti dall’aggiornamento.

### Passaggio iniziale per Windows 1.4.5

Le versioni Windows fino alla 1.4.4 contengono ancora il vecchio percorso Tauri che termina con l’errore **The signature verification failed**. Perciò la 1.4.5 deve essere installata manualmente una sola volta scaricando dalla Release il file `Registro.Presenze_1.4.5_x64-setup.exe`. Dalla versione 1.4.5 in poi, gli aggiornamenti successivi useranno il nuovo pulsante automatico direttamente nell’app.

La chiave privata attiva si trova sul Mac di Vigi in `~/.registro-presenze/registro-presenze-v2.key`. Non deve essere caricata nel repository o passata ai colleghi. Anche la password deve essere conservata separatamente.
