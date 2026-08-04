use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use crate::error::AppError;
use crate::features::with_repo;
use crate::models::{CreateProjectRequest, Project, RenameProjectRequest};
use crate::AppState;

#[tauri::command]
pub async fn create_project(
    state: State<'_, Arc<RwLock<AppState>>>,
    request: CreateProjectRequest,
) -> Result<Project, AppError> {
    with_repo(&state, move |repo| repo.create_project(&request.name)).await
}

#[tauri::command]
pub async fn list_projects(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<Vec<Project>, AppError> {
    with_repo(&state, |repo| repo.list_projects()).await
}

#[tauri::command]
pub async fn get_project(
    state: State<'_, Arc<RwLock<AppState>>>,
    id: String,
) -> Result<Project, AppError> {
    with_repo(&state, move |repo| {
        repo.get_project(&id)?
            .ok_or_else(|| AppError::Crawl(format!("Project not found: {}", id)))
    })
    .await
}

#[tauri::command]
pub async fn rename_project(
    state: State<'_, Arc<RwLock<AppState>>>,
    request: RenameProjectRequest,
) -> Result<(), AppError> {
    with_repo(&state, move |repo| repo.rename_project(&request.id, &request.name)).await
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, Arc<RwLock<AppState>>>,
    id: String,
) -> Result<(), AppError> {
    let state_inner = state.inner().clone();

    // Stop crawl if running for this project
    {
        let state_write = state_inner.write().await;
        let mut crawls = state_write.crawls.write().await;
        if let Some(crawl_state) = crawls.remove(&id) {
            crawl_state.cancellation.cancel();
            info!("Stopped crawl for deleted project: {}", id);
        }
    }

    with_repo(&state, move |repo| repo.delete_project(&id)).await
}

#[tauri::command]
pub async fn get_project_stats(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<serde_json::Value, AppError> {
    with_repo(&state, move |repo| repo.get_project_stats(&project_id)).await
}
