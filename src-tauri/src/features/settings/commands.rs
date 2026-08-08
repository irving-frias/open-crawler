use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::secrets;
use crate::AppState;

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<HashMap<String, String>, AppError> {
    with_repo(&state, |repo| {
        let mut settings = repo.get_all_settings()?;
        for key in secrets::SECRET_KEYS {
            let flag = format!("{key}_set");
            settings.remove(key);
            settings.insert(flag, secrets::has(repo, key)?.to_string());
        }
        Ok(settings)
    })
    .await
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    settings: HashMap<String, String>,
) -> Result<(), AppError> {
    with_repo(&state, move |repo| {
        for (key, value) in &settings {
            if secrets::is_secret_key(key) {
                secrets::set(repo, key, value)?;
            } else {
                repo.set_setting(key, value)?;
            }
        }
        Ok(())
    })
    .await
}
