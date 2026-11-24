use std::sync::Arc;

use tauri::{AppHandle, State};
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::alerts::service::{self, TestAlertResult};
use crate::features::with_repo;
use crate::models::AlertSettings;
use crate::AppState;

#[tauri::command]
pub async fn get_alert_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<AlertSettings, AppError> {
    with_repo(&state, move |repo| repo.get_alert_settings(&project_id)).await
}

#[tauri::command]
pub async fn save_alert_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    settings: AlertSettings,
) -> Result<AlertSettings, AppError> {
    with_repo(&state, move |repo| repo.save_alert_settings(&settings)).await
}

/// Sends a test notification on the project's (or, with no project, the
/// global) channels. Never errors when a channel fails: the returned flags
/// say which channels actually delivered.
#[tauri::command]
pub async fn test_alert_notification(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
) -> Result<TestAlertResult, AppError> {
    service::send_test_alert(&app, state.inner(), project_id.as_deref()).await
}
