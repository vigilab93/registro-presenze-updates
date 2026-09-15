use chrono::{Local, SecondsFormat};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_updater::{Update, UpdaterExt};

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppData {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    #[serde(default = "empty_object")]
    entries: Value,
    #[serde(default = "empty_object")]
    config: Value,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveResult {
    saved: bool,
    backup_path: String,
    warning: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInfo {
    version: String,
    notes: Option<String>,
    date: Option<String>,
}

struct PendingUpdate(Mutex<Option<Update>>);

struct StoragePaths {
    database: PathBuf,
    backup_dir: PathBuf,
    export_dir: PathBuf,
}

fn default_schema_version() -> u32 {
    SCHEMA_VERSION
}

fn empty_object() -> Value {
    Value::Object(Map::new())
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            entries: empty_object(),
            config: empty_object(),
            updated_at: None,
        }
    }
}

fn storage_paths(app: &tauri::AppHandle) -> Result<StoragePaths, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Cartella dati non disponibile: {error}"))?;
    let documents = app
        .path()
        .document_dir()
        .map_err(|error| format!("Cartella Documenti non disponibile: {error}"))?;
    let user_dir = documents.join("Registro Presenze");

    Ok(StoragePaths {
        database: data_dir.join("registro-presenze.sqlite"),
        backup_dir: user_dir.join("Backup"),
        export_dir: user_dir.join("Esportazioni"),
    })
}

fn open_database(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Impossibile creare la cartella dati: {error}"))?;
    }

    let connection = Connection::open(path)
        .map_err(|error| format!("Impossibile aprire il database: {error}"))?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| format!("Impossibile attivare la protezione WAL: {error}"))?;
    connection
        .pragma_update(None, "synchronous", "FULL")
        .map_err(|error| format!("Impossibile configurare il salvataggio sicuro: {error}"))?;
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS app_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                payload TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|error| format!("Impossibile preparare il database: {error}"))?;
    Ok(connection)
}

fn open_database_with_recovery(path: &Path) -> Result<Connection, String> {
    match open_database(path) {
        Ok(connection) => Ok(connection),
        Err(first_error) => {
            if path.exists() {
                let corrupt_name = format!(
                    "registro-presenze-danneggiato-{}.sqlite",
                    Local::now().format("%Y%m%d-%H%M%S")
                );
                let corrupt_path = path.with_file_name(corrupt_name);
                fs::rename(path, &corrupt_path).map_err(|rename_error| {
                    format!(
                        "{first_error}. Impossibile mettere al sicuro il database danneggiato: {rename_error}"
                    )
                })?;
                let wal = path.with_extension("sqlite-wal");
                let shm = path.with_extension("sqlite-shm");
                let _ = fs::remove_file(wal);
                let _ = fs::remove_file(shm);
            }
            open_database(path)
        }
    }
}

fn normalize(mut data: AppData) -> AppData {
    if !data.entries.is_object() {
        data.entries = empty_object();
    }
    if !data.config.is_object() {
        data.config = empty_object();
    }
    data.schema_version = SCHEMA_VERSION;
    data
}

fn parse_data(raw: &str) -> Result<AppData, String> {
    serde_json::from_str::<AppData>(raw)
        .map(normalize)
        .map_err(|error| format!("Archivio dati non valido: {error}"))
}

fn newest_valid_backup(backup_dir: &Path) -> Option<AppData> {
    let mut candidates: Vec<PathBuf> = fs::read_dir(backup_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect();
    candidates.sort();
    candidates.reverse();

    for path in candidates {
        if let Ok(raw) = fs::read_to_string(path) {
            if let Ok(data) = parse_data(&raw) {
                return Some(data);
            }
        }
    }
    None
}

fn write_database(connection: &mut Connection, data: &AppData) -> Result<(), String> {
    let payload = serde_json::to_string(data)
        .map_err(|error| format!("Impossibile preparare i dati: {error}"))?;
    let timestamp = data.updated_at.clone().unwrap_or_else(now_iso);
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Impossibile iniziare il salvataggio: {error}"))?;
    transaction
        .execute(
            "INSERT INTO app_state (id, payload, updated_at) VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at",
            params![payload, timestamp],
        )
        .map_err(|error| format!("Impossibile salvare i dati: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Impossibile completare il salvataggio: {error}"))
}

fn now_iso() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}

fn write_backups(backup_dir: &Path, data: &AppData) -> Result<String, String> {
    fs::create_dir_all(backup_dir)
        .map_err(|error| format!("Impossibile creare la cartella backup: {error}"))?;
    let pretty = serde_json::to_vec_pretty(data)
        .map_err(|error| format!("Impossibile creare il backup: {error}"))?;
    let current = backup_dir.join("registro-presenze-ultimo.json");
    let daily = backup_dir.join(format!(
        "registro-presenze-{}.json",
        Local::now().format("%Y-%m-%d")
    ));
    fs::write(&current, &pretty)
        .map_err(|error| format!("Impossibile aggiornare il backup principale: {error}"))?;
    fs::write(&daily, &pretty)
        .map_err(|error| format!("Impossibile aggiornare il backup giornaliero: {error}"))?;
    prune_daily_backups(backup_dir, 30);
    Ok(current.to_string_lossy().into_owned())
}

fn prune_daily_backups(backup_dir: &Path, keep: usize) {
    let mut daily: Vec<PathBuf> = match fs::read_dir(backup_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| {
                        name.starts_with("registro-presenze-20") && name.ends_with(".json")
                    })
            })
            .collect(),
        Err(_) => return,
    };
    daily.sort();
    let remove_count = daily.len().saturating_sub(keep);
    for path in daily.into_iter().take(remove_count) {
        let _ = fs::remove_file(path);
    }
}

#[tauri::command]
fn load_app_data(app: tauri::AppHandle) -> Result<AppData, String> {
    let paths = storage_paths(&app)?;
    let mut connection = open_database_with_recovery(&paths.database)?;

    let stored: Option<String> = connection
        .query_row("SELECT payload FROM app_state WHERE id = 1", [], |row| row.get(0))
        .optional()
        .map_err(|error| format!("Impossibile leggere il database: {error}"))?;

    if let Some(raw) = stored {
        if let Ok(data) = parse_data(&raw) {
            return Ok(data);
        }
    }

    if let Some(mut recovered) = newest_valid_backup(&paths.backup_dir) {
        recovered.updated_at = Some(now_iso());
        write_database(&mut connection, &recovered)?;
        return Ok(recovered);
    }

    Ok(AppData::default())
}

#[tauri::command]
fn save_app_data(mut data: AppData, app: tauri::AppHandle) -> Result<SaveResult, String> {
    data = normalize(data);
    data.updated_at = Some(now_iso());
    let paths = storage_paths(&app)?;
    let mut connection = open_database(&paths.database)?;
    write_database(&mut connection, &data)?;

    match write_backups(&paths.backup_dir, &data) {
        Ok(path) => Ok(SaveResult {
            saved: true,
            backup_path: path,
            warning: None,
        }),
        Err(error) => Ok(SaveResult {
            saved: true,
            backup_path: paths.backup_dir.to_string_lossy().into_owned(),
            warning: Some(format!("Dati salvati, ma il backup non è stato aggiornato: {error}")),
        }),
    }
}

#[tauri::command]
fn save_export_file(file_name: String, contents: String, app: tauri::AppHandle) -> Result<String, String> {
    let paths = storage_paths(&app)?;
    fs::create_dir_all(&paths.export_dir)
        .map_err(|error| format!("Impossibile creare la cartella esportazioni: {error}"))?;
    let safe_name: String = file_name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_'))
        .collect();
    if safe_name.is_empty() {
        return Err("Nome del file non valido".into());
    }
    let destination = paths.export_dir.join(safe_name);
    fs::write(&destination, contents.as_bytes())
        .map_err(|error| format!("Impossibile salvare l'esportazione: {error}"))?;
    Ok(destination.to_string_lossy().into_owned())
}

#[tauri::command]
fn print_report(window: tauri::WebviewWindow) -> Result<(), String> {
    window
        .print()
        .map_err(|error| format!("Impossibile aprire la stampa: {error}"))
}

#[tauri::command]
fn open_update_page() -> Result<(), String> {
    const RELEASES_URL: &str =
        "https://github.com/vigilab93/registro-presenze-updates/releases/latest";

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(RELEASES_URL).spawn();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(["/C", "start", "", RELEASES_URL])
        .spawn();

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let result = Command::new("xdg-open").arg(RELEASES_URL).spawn();

    result
        .map(|_| ())
        .map_err(|error| format!("Impossibile aprire la pagina degli aggiornamenti: {error}"))
}

#[tauri::command]
async fn check_for_update(
    app: tauri::AppHandle,
    pending_update: tauri::State<'_, PendingUpdate>,
) -> Result<Option<UpdateInfo>, String> {
    let update = app
        .updater()
        .map_err(|error| format!("Servizio aggiornamenti non disponibile: {error}"))?
        .check()
        .await
        .map_err(|error| format!("Controllo aggiornamenti non riuscito: {error}"))?;

    let info = update.as_ref().map(|available| UpdateInfo {
        version: available.version.clone(),
        notes: available.body.clone(),
        date: available.date.as_ref().map(ToString::to_string),
    });

    let mut pending = pending_update
        .0
        .lock()
        .map_err(|_| "Impossibile preparare l'aggiornamento".to_string())?;
    *pending = update;
    Ok(info)
}

#[tauri::command]
async fn install_update(
    app: tauri::AppHandle,
    pending_update: tauri::State<'_, PendingUpdate>,
) -> Result<(), String> {
    let update = {
        let pending = pending_update
            .0
            .lock()
            .map_err(|_| "Impossibile aprire l'aggiornamento".to_string())?;
        pending
            .as_ref()
            .cloned()
            .ok_or_else(|| "Nessun aggiornamento pronto da installare".to_string())?
    };

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let download_url = update.download_url.to_string();
        let platform = update
            .raw_json
            .get("platforms")
            .and_then(Value::as_object)
            .and_then(|platforms| {
                platforms.get(update.target.as_str()).or_else(|| {
                    platforms.values().find(|candidate| {
                        candidate.get("url").and_then(Value::as_str)
                            == Some(download_url.as_str())
                    })
                })
            })
            .ok_or_else(|| "Dati dell'aggiornamento Windows incompleti".to_string())?;
        let expected_sha256 = platform
            .get("sha256")
            .and_then(Value::as_str)
            .filter(|hash| hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()))
            .ok_or_else(|| "Controllo di integrità SHA-256 mancante".to_string())?
            .to_ascii_lowercase();
        let safe_version: String = update
            .version
            .chars()
            .filter(|character| character.is_ascii_alphanumeric() || *character == '.')
            .collect();
        let installer_path = std::env::temp_dir().join(format!(
            "Registro-Presenze-{safe_version}-aggiornamento.exe"
        ));
        let powershell_script = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest -UseBasicParsing -Uri $env:RP_UPDATE_URL -OutFile $env:RP_UPDATE_PATH
$actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $env:RP_UPDATE_PATH).Hash.ToLowerInvariant()
$expected = $env:RP_UPDATE_SHA256.ToLowerInvariant()
if ($actual -ne $expected) {
  Remove-Item -LiteralPath $env:RP_UPDATE_PATH -Force -ErrorAction SilentlyContinue
  throw "Il file scaricato non supera il controllo SHA-256"
}
"#;

        let download_result = Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                powershell_script,
            ])
            .env("RP_UPDATE_URL", &download_url)
            .env("RP_UPDATE_PATH", &installer_path)
            .env("RP_UPDATE_SHA256", &expected_sha256)
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match download_result {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let details = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let mut pending = pending_update.0.lock().map_err(|_| {
                    "Download non riuscito e aggiornamento non recuperabile".to_string()
                })?;
                *pending = Some(update);
                return Err(if details.is_empty() {
                    "Download o controllo di integrità non riuscito".to_string()
                } else {
                    format!("Download o controllo di integrità non riuscito: {details}")
                });
            }
            Err(error) => {
                let mut pending = pending_update.0.lock().map_err(|_| {
                    "Download non riuscito e aggiornamento non recuperabile".to_string()
                })?;
                *pending = Some(update);
                return Err(format!("Impossibile avviare il download Windows: {error}"));
            }
        }

        if !installer_path.is_file() {
            let mut pending = pending_update.0.lock().map_err(|_| {
                "Installer non trovato e aggiornamento non recuperabile".to_string()
            })?;
            *pending = Some(update);
            return Err("Il download è terminato ma l'installer non è stato trovato".to_string());
        }

        let launch_result = Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "Start-Process -FilePath $env:RP_UPDATE_PATH -ArgumentList @('/P','/UPDATE','/R')",
            ])
            .env("RP_UPDATE_PATH", &installer_path)
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match launch_result {
            Ok(output) if output.status.success() => {
                app.exit(0);
                Ok(())
            }
            Ok(output) => {
                let details = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let mut pending = pending_update.0.lock().map_err(|_| {
                    "Installer non avviato e aggiornamento non recuperabile".to_string()
                })?;
                *pending = Some(update);
                Err(if details.is_empty() {
                    "Windows non ha avviato l'installer dell'aggiornamento".to_string()
                } else {
                    format!("Windows non ha avviato l'installer: {details}")
                })
            }
            Err(error) => {
                let mut pending = pending_update.0.lock().map_err(|_| {
                    "Installer non avviato e aggiornamento non recuperabile".to_string()
                })?;
                *pending = Some(update);
                Err(format!("Impossibile avviare PowerShell per l'aggiornamento: {error}"))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let install_result = update.download_and_install(|_, _| {}, || {}).await;

        if let Err(error) = install_result {
            let mut pending = pending_update
                .0
                .lock()
                .map_err(|_| format!("Installazione non riuscita: {error}"))?;
            *pending = Some(update);
            return Err(format!("Installazione non riuscita: {error}"));
        }

        app.restart();
        #[allow(unreachable_code)]
        Ok(())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(PendingUpdate(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            load_app_data,
            save_app_data,
            save_export_file,
            print_report,
            open_update_page,
            check_for_update,
            install_update
        ])
        .run(tauri::generate_context!())
        .expect("errore durante l'avvio di Registro Presenze");
}
