use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;
use tracing::info;

use crate::error::AppError;
use crate::features::with_repo;
use crate::models::{CreateProjectRequest, Project, RenameProjectRequest};
use crate::AppState;

/// Broadcasts a `projects-changed` event to every window so the launcher and
/// the per-project windows stay in sync (rename / create / delete).
fn projects_changed(app: &AppHandle) {
    let _ = app.emit("projects-changed", ());
}

#[tauri::command]
pub async fn create_project(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    request: CreateProjectRequest,
) -> Result<Project, AppError> {
    let project = with_repo(&state, move |repo| repo.create_project(&request.name)).await?;
    projects_changed(&app);
    Ok(project)
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
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    request: RenameProjectRequest,
) -> Result<(), AppError> {
    with_repo(&state, move |repo| {
        repo.rename_project(&request.id, &request.name)
    })
    .await?;
    projects_changed(&app);
    Ok(())
}

#[tauri::command]
pub async fn delete_project(
    app: AppHandle,
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

    // Stop SEO audit if running for this project
    {
        let state_write = state_inner.write().await;
        let mut audits = state_write.seo_audits.write().await;
        if let Some(audit) = audits.remove(&id) {
            audit.cancellation.cancel();
            info!("Stopped SEO audit for deleted project: {}", id);
        }
    }

    let label = format!("project-{id}");
    let result = with_repo(&state, move |repo| repo.delete_project(&id)).await;

    if result.is_ok() {
        projects_changed(&app);
    }

    // A deleted project must never leave an orphaned project window behind.
    // This runs regardless of which window triggered the delete (launcher or
    // the project window itself). Desktop only — on Android there are no
    // project windows, so the project is simply removed from the in-app list.
    if result.is_ok() {
        #[cfg(not(mobile))]
        {
            let app_for_main = app.clone();
            app.run_on_main_thread(move || {
                if let Some(win) = app_for_main.get_webview_window(&label) {
                    let _ = win.close();
                }
            })
            .map_err(|e| AppError::Crawl(format!("Failed to close project window: {e}")))?;
        }
    }

    result
}

#[tauri::command]
pub async fn get_project_stats(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<serde_json::Value, AppError> {
    with_repo(&state, move |repo| repo.get_project_stats(&project_id)).await
}
