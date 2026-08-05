use std::path::Path;
use std::sync::Arc;

use futures::StreamExt;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::warn;
#[cfg(mobile)]
use uuid::Uuid;

use crate::error::AppError;
use crate::features::with_repo;
use crate::transfer::obex;
use crate::transfer::package::{
    export_package as package_export, import_package as package_import, ExportPackageInfo,
    ImportMode, ImportSummary,
};
use crate::transfer::server::{self, TransferInfo};
use crate::AppState;

/// Resolves where a package should be written.
///
/// On desktop the user picks the destination via a save dialog and the package
/// is written straight to it. On mobile a SAF `content://` URI is written
/// through the fs plugin's mobile bridge; any other name is written into the
/// app data dir and the native share sheet is opened afterwards.
fn package_target(app: &AppHandle, file_path: &str) -> Result<(String, bool), AppError> {
    if !cfg!(mobile) {
        return Ok((file_path.to_string(), false));
    }

    if file_path.starts_with("content://") {
        return Ok((file_path.to_string(), false));
    }

    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty() && !n.starts_with("content:"))
        .map(str::to_string)
        .unwrap_or_else(|| "open-crawler.ocproj".to_string());

    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Crawl(e.to_string()))?
        .join("exports");
    std::fs::create_dir_all(&dir)?;
    Ok((dir.join(&file_name).to_string_lossy().into_owned(), true))
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn export_package(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    project_ids: Option<Vec<String>>,
    file_path: String,
    lightweight: Option<bool>,
    include_credentials: Option<bool>,
    share_after: Option<bool>,
    silent: Option<bool>,
) -> Result<ExportPackageInfo, AppError> {
    let lightweight = lightweight.unwrap_or(false);
    let include_credentials = include_credentials.unwrap_or(false);
    let share_after = share_after.unwrap_or(false);
    let silent = silent.unwrap_or(false);

    // Silent mode writes into a managed transfers dir (no save dialog on
    // desktop, no share sheet on mobile) — used by the direct-share flow
    // (WiFi/Bluetooth/P2P) where export happens automatically as step 1.
    let (write_path, will_share) = if silent {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Crawl(e.to_string()))?
            .join("transfers");
        std::fs::create_dir_all(&dir)?;
        let file_name = Path::new(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|n| !n.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| "open-crawler.ocproj".to_string());
        (dir.join(&file_name).to_string_lossy().into_owned(), false)
    } else {
        package_target(&app, &file_path)?
    };

    let content_uri = if cfg!(mobile) && write_path.starts_with("content://") {
        Some(write_path.clone())
    } else {
        None
    };
    let tmp_path: Option<std::path::PathBuf> = if content_uri.is_some() {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Crawl(e.to_string()))?
            .join("exports");
        std::fs::create_dir_all(&dir)?;
        Some(dir.join(format!("package-tmp-{}.ocproj", std::process::id())))
    } else {
        None
    };

    let dest_path = tmp_path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| write_path.clone());

    let app_export = app.clone();
    let dest = dest_path.clone();
    let info = with_repo(&state, move |repo| {
        package_export(
            repo,
            &app_export,
            project_ids,
            lightweight,
            include_credentials,
            Path::new(&dest),
        )
    })
    .await?;

    if let (Some(uri), Some(tmp)) = (content_uri, tmp_path) {
        crate::features::export::commands::copy_to_content_uri(&app, &uri, &tmp)?;
        let _ = std::fs::remove_file(&tmp);
    }

    if !silent && (will_share || share_after) {
        if let Err(e) = share_package(&app, &write_path) {
            warn!("Could not open share sheet: {e}");
        }
    }

    Ok(info)
}

/// Opens the native share sheet (AirDrop on macOS, system share UI on mobile)
/// for an already-exported package file. Used by the direct-share / Bluetooth
/// flow after a silent export.
#[tauri::command]
pub fn open_share_sheet(app: AppHandle, file_path: String) -> Result<(), AppError> {
    share_package(&app, &file_path)
}

/// Sends an already-exported package to a Bluetooth device over OBEX Object
/// Push (Windows/Linux). Progress is reported through the `transfer-progress`
/// event with stage `"bluetooth"`.
///
/// NOTE: experimental — RFCOMM transports have not been verified against real
/// hardware yet. macOS/iOS fall back to the system share sheet instead.
#[tauri::command]
pub async fn bt_send(app: AppHandle, addr: String, file_path: String) -> Result<(), AppError> {
    let path = std::path::PathBuf::from(&file_path);
    let app = app.clone();

    tokio::task::spawn_blocking(move || {
        obex::send_file(&addr, &path, |sent, total| {
            emit_transfer_progress(&app, "bluetooth", sent, total);
        })
    })
    .await
    .map_err(|e| AppError::Crawl(format!("Bluetooth task failed: {e}")))?
}

#[tauri::command]
pub async fn import_package(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    file_path: String,
    mode: String,
) -> Result<ImportSummary, AppError> {
    let mobile_uri = cfg!(mobile) && file_path.starts_with("content://");

    let (import_path, tmp_path) = if mobile_uri {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Crawl(e.to_string()))?
            .join("transfers");
        std::fs::create_dir_all(&dir)?;
        let tmp = dir.join(format!("import-tmp-{}.ocproj", std::process::id()));
        crate::features::export::commands::copy_from_content_uri(&app, &file_path, &tmp)?;
        (tmp.to_string_lossy().into_owned(), Some(tmp))
    } else {
        (file_path.clone(), None)
    };

    let app_import = app.clone();
    let path = import_path.clone();
    let result = with_repo(&state, move |repo| {
        package_import(repo, &app_import, Path::new(&path), ImportMode::parse(&mode))
    })
    .await;

    if let Some(tmp) = tmp_path {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

#[cfg(mobile)]
fn share_package(app: &AppHandle, path: &str) -> Result<(), AppError> {
    use tauri_plugin_share::ShareExt;

    let app = app.clone();
    let path = path.to_string();

    std::thread::spawn(move || {
        let _ = app.share_file().share_file(tauri_plugin_share::ShareRequest {
            path: Some(path),
            mime: Some("application/octet-stream".to_string()),
            group: None,
        });
    });
    Ok(())
}

#[cfg(not(mobile))]
fn share_package(app: &AppHandle, path: &str) -> Result<(), AppError> {
    crate::transfer::desktop_share::share_file(app, path).map_err(AppError::Crawl)
}

#[derive(serde::Serialize, Clone)]
struct TransferProgress {
    stage: &'static str,
    processed: u64,
    total: u64,
    percent: f32,
}

fn emit_transfer_progress(app: &AppHandle, stage: &'static str, processed: u64, total: u64) {
    let percent = if total == 0 {
        100.0
    } else {
        (processed as f32 / total as f32) * 100.0
    };
    let _ = app.emit(
        "transfer-progress",
        TransferProgress {
            stage,
            processed,
            total,
            percent,
        },
    );
}

/// Starts the LAN WiFi server so other devices can download a package file
/// from the phone/desktop's local address (shown as a QR code / link).
#[tauri::command]
pub async fn start_transfer_server(
    state: State<'_, Arc<RwLock<AppState>>>,
    file_path: String,
    minutes: Option<u64>,
) -> Result<TransferInfo, AppError> {
    let state_read = state.read().await;
    server::start_transfer_server(&state_read, Path::new(&file_path), minutes.unwrap_or(0))
}

#[tauri::command]
pub async fn stop_transfer_server(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<(), AppError> {
    let state_read = state.read().await;
    server::stop_transfer_server(&state_read)
}

#[tauri::command]
pub async fn get_active_transfer(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<Option<TransferInfo>, AppError> {
    let state_read = state.read().await;
    Ok(server::active_transfer(&state_read))
}

/// Downloads a package from a transfer URL (typed or scanned from a QR code)
/// into `dest`, reporting progress via the `transfer-progress` event.
#[tauri::command]
pub async fn download_transfer(
    app: AppHandle,
    url: String,
    dest: String,
) -> Result<(), AppError> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Crawl(format!("failed to build HTTP client: {e}")))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Crawl(format!("download failed: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::Crawl(format!(
            "download failed with HTTP {}",
            response.status()
        )));
    }

    let total = response.content_length().unwrap_or(0);
    if let Some(parent) = Path::new(&dest).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = tokio::fs::File::create(&dest).await?;
    let mut processed: u64 = 0;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Crawl(format!("download error: {e}")))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| AppError::Crawl(format!("write error: {e}")))?;
        processed += chunk.len() as u64;
        emit_transfer_progress(&app, "download", processed, total);
    }
    emit_transfer_progress(&app, "download", processed, processed);
    Ok(())
}

/// Payload extracted from a received Android share intent.
#[cfg(any(mobile, test))]
#[derive(Debug, PartialEq, Eq)]
enum IntentPayload {
    /// A file URI (`android.intent.extra.STREAM`) pointing at the shared file.
    Stream(String),
    /// Plain text (`android.intent.extra.TEXT`) — a transfer URL to download.
    Text(String),
}

/// Parses the raw Android intent string produced by `tauri-plugin-mobile-sharetarget`
/// (an `#Intent;...` URI) into a usable payload, if any.
#[cfg(any(mobile, test))]
fn parse_intent_payload(raw: &str) -> Option<IntentPayload> {
    const STREAM: &str = "S.android.intent.extra.STREAM=";
    const TEXT: &str = "S.android.intent.extra.TEXT=";

    for part in raw.split(';') {
        if let Some(v) = part.strip_prefix(STREAM) {
            return Some(IntentPayload::Stream(url_decode(v)));
        }
    }
    for part in raw.split(';') {
        if let Some(v) = part.strip_prefix(TEXT) {
            return Some(IntentPayload::Text(url_decode(v)));
        }
    }
    None
}

#[cfg(any(mobile, test))]
fn url_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8()
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| s.to_string())
}

/// Pops the incoming Android share intent (if any) and imports the received
/// `.ocproj` package. A `STREAM` extra is treated as a `content://` file URI;
/// a `TEXT` extra is treated as a transfer URL that gets downloaded first.
/// Returns `None` when the queue is empty or the payload isn't usable.
#[tauri::command]
pub async fn import_shared_intent(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    mode: String,
) -> Result<Option<ImportSummary>, AppError> {
    #[cfg(mobile)]
    {
        use tauri_plugin_mobile_sharetarget::MobileSharetargetExt;

        let raw = app
            .mobile_sharetarget()
            .get_latest_intent()
            .map_err(|e| AppError::Crawl(format!("share target error: {e}")))?;
        let Some(raw) = raw else {
            return Ok(None);
        };

        match parse_intent_payload(&raw) {
            Some(IntentPayload::Stream(uri)) => {
                let summary = import_package(app, state, uri, mode).await?;
                Ok(Some(summary))
            }
            Some(IntentPayload::Text(url)) => {
                let dir = app
                    .path()
                    .app_data_dir()
                    .map_err(|e| AppError::Crawl(e.to_string()))?
                    .join("transfers");
                std::fs::create_dir_all(&dir)?;
                let dest = dir.join(format!("shared-{}.ocproj", Uuid::new_v4().simple()));
                let url_dl = url.clone();
                let app_dl = app.clone();
                let dest_dl = dest.clone();
                download_transfer(app_dl, url_dl, dest_dl.to_string_lossy().into_owned())
                    .await?;
                let summary = import_package(
                    app,
                    state,
                    dest.to_string_lossy().into_owned(),
                    mode,
                )
                .await?;
                let _ = std::fs::remove_file(&dest);
                Ok(Some(summary))
            }
            None => Ok(None),
        }
    }

    #[cfg(not(mobile))]
    {
        let _ = (app, state, mode);
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stream_extra() {
        let raw = r"#Intent;action=android.intent.action.SEND;type=application/octet-stream;launchFlags=0x13400000;component=com.tauri.dev\/.MainActivity;S.android.intent.extra.STREAM=content%3A%2F%2Fmedia%2Fexternal%2Fdownloads%2F1;end";
        assert_eq!(
            parse_intent_payload(raw),
            Some(IntentPayload::Stream(
                "content://media/external/downloads/1".to_string()
            ))
        );
    }

    #[test]
    fn parses_text_extra() {
        let raw = r"#Intent;action=android.intent.action.SEND;type=text\/plain;component=com.tauri.dev\/.MainActivity;S.android.intent.extra.TEXT=http%3A%2F%2F192.168.1.5%3A45231%2Fdl%2Fabc%2Fpkg.ocproj;end";
        assert_eq!(
            parse_intent_payload(raw),
            Some(IntentPayload::Text(
                "http://192.168.1.5:45231/dl/abc/pkg.ocproj".to_string()
            ))
        );
    }

    #[test]
    fn returns_none_when_no_payload() {
        assert_eq!(parse_intent_payload("#Intent;action=android.intent.action.VIEW;end"), None);
    }
}
