use std::path::Path;
use std::sync::Arc;

use futures::StreamExt;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::warn;

use crate::error::AppError;
use crate::features::with_repo;
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

#[tauri::command]
pub async fn export_package(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    project_ids: Option<Vec<String>>,
    file_path: String,
    lightweight: Option<bool>,
    include_credentials: Option<bool>,
    share_after: Option<bool>,
) -> Result<ExportPackageInfo, AppError> {
    let lightweight = lightweight.unwrap_or(false);
    let include_credentials = include_credentials.unwrap_or(false);
    let share_after = share_after.unwrap_or(false);

    let (write_path, will_share) = package_target(&app, &file_path)?;

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

    if will_share || share_after {
        share_package(&app, &write_path);
    }

    Ok(info)
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
fn share_package(app: &AppHandle, path: &str) {
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
}

#[cfg(not(mobile))]
fn share_package(_app: &AppHandle, _path: &str) {
    warn!("Share sheet requested but not available on this platform");
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
