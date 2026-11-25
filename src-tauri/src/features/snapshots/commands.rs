use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::models::{ComparePageResult, CrawlSnapshot};
use crate::AppState;

#[tauri::command]
pub async fn list_crawl_snapshots(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Vec<CrawlSnapshot>, AppError> {
    with_repo(&state, move |repo| repo.list_crawl_snapshots(&project_id)).await
}

/// Per-project snapshot retention (T15): how many snapshots to keep.
#[tauri::command]
pub async fn get_snapshot_retention(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<u32, AppError> {
    with_repo(&state, move |repo| repo.get_snapshot_retention(&project_id)).await
}

/// Set the retention (clamped 1..=100) and prune the old snapshots now.
#[tauri::command]
pub async fn set_snapshot_retention(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    retention: u32,
) -> Result<u32, AppError> {
    with_repo(&state, move |repo| {
        repo.set_snapshot_retention(&project_id, retention)
    })
    .await
}

#[tauri::command]
pub async fn compare_crawls_page(
    state: State<'_, Arc<RwLock<AppState>>>,
    snapshot_a: String,
    snapshot_b: String,
    section: String,
    page: u32,
    page_size: u32,
) -> Result<ComparePageResult, AppError> {
    with_repo(&state, move |repo| {
        repo.compare_crawl_snapshots_page(&snapshot_a, &snapshot_b, &section, page, page_size)
    })
    .await
}
