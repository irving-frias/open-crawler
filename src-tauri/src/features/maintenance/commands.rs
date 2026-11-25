use std::collections::BTreeMap;
use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::AppState;

/// Delete rows whose parent is gone, across every project-keyed table
/// (T15). Returns per-table deleted counts.
#[tauri::command]
pub async fn cleanup_orphans(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<BTreeMap<String, u64>, AppError> {
    with_repo(&state, move |repo| repo.cleanup_orphans()).await
}
