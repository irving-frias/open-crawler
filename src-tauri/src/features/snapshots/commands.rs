use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::models::{CompareResult, CrawlSnapshot};
use crate::AppState;

#[tauri::command]
pub async fn list_crawl_snapshots(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Vec<CrawlSnapshot>, AppError> {
    with_repo(&state, move |repo| repo.list_crawl_snapshots(&project_id)).await
}

#[tauri::command]
pub async fn compare_crawls(
    state: State<'_, Arc<RwLock<AppState>>>,
    snapshot_a: String,
    snapshot_b: String,
) -> Result<CompareResult, AppError> {
    with_repo(&state, move |repo| {
        repo.compare_crawl_snapshots(&snapshot_a, &snapshot_b)
    })
    .await
}
