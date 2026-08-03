use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::models::{DashboardStats, DuplicateGroup, KeywordAggregate};
use crate::AppState;

#[tauri::command]
pub async fn get_dashboard_stats(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<DashboardStats, AppError> {
    with_repo(&state, |repo| repo.get_dashboard_stats(&project_id)).await
}

#[tauri::command]
pub async fn get_duplicate_groups(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Vec<DuplicateGroup>, AppError> {
    with_repo(&state, |repo| repo.get_duplicate_groups(&project_id)).await
}

#[tauri::command]
pub async fn get_project_keywords(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    limit: Option<u32>,
) -> Result<Vec<KeywordAggregate>, AppError> {
    with_repo(&state, |repo| repo.get_keywords(&project_id, limit.unwrap_or(100))).await
}
