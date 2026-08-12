use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::models::LinkAnalysis;
use crate::AppState;

#[tauri::command]
pub async fn get_link_analysis(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<LinkAnalysis, AppError> {
    with_repo(&state, move |repo| repo.get_link_analysis(&project_id)).await
}

#[tauri::command]
pub async fn get_project_has_links(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<bool, AppError> {
    with_repo(&state, move |repo| repo.project_has_links(&project_id)).await
}
