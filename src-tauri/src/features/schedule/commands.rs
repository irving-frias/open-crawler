use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::models::{CreateScheduledJobRequest, ScheduledJob, UpdateScheduledJobRequest};
use crate::AppState;

#[tauri::command]
pub async fn list_scheduled_jobs(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<Vec<ScheduledJob>, AppError> {
    with_repo(&state, |repo| repo.list_scheduled_jobs()).await
}

#[tauri::command]
pub async fn create_scheduled_job(
    state: State<'_, Arc<RwLock<AppState>>>,
    req: CreateScheduledJobRequest,
) -> Result<ScheduledJob, AppError> {
    with_repo(&state, move |repo| repo.create_scheduled_job(&req)).await
}

#[tauri::command]
pub async fn update_scheduled_job(
    state: State<'_, Arc<RwLock<AppState>>>,
    req: UpdateScheduledJobRequest,
) -> Result<Option<ScheduledJob>, AppError> {
    with_repo(&state, move |repo| repo.update_scheduled_job(&req)).await
}

#[tauri::command]
pub async fn delete_scheduled_job(
    state: State<'_, Arc<RwLock<AppState>>>,
    id: String,
) -> Result<(), AppError> {
    with_repo(&state, move |repo| repo.delete_scheduled_job(&id)).await
}
