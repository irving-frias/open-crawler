use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::AppState;

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<HashMap<String, String>, AppError> {
    with_repo(&state, |repo| repo.get_all_settings()).await
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    settings: HashMap<String, String>,
) -> Result<(), AppError> {
    with_repo(&state, move |repo| {
        for (key, value) in &settings {
            repo.set_setting(key, value)?;
        }
        Ok(())
    })
    .await
}
