use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::crawler::engine::CrawlEngine;
use crate::error::AppError;
use crate::features::with_repo;
use crate::models::{CrawlConfig, CrawlProgress};
use crate::{AppState, CrawlState};

#[tauri::command]
pub async fn check_resumable_crawl(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Option<serde_json::Value>, AppError> {
    with_repo(&state, move |repo| {
        match repo.get_interrupted_session(&project_id)? {
            Some(session) => {
                let queue_count = repo.load_queue(&session.id)?.len() as u32;
                Ok(Some(serde_json::json!({
                    "session_id": session.id,
                    "pages_crawled": session.pages_crawled,
                    "errors": session.errors,
                    "elapsed_secs": session.elapsed_secs,
                    "queue_remaining": queue_count,
                })))
            }
            None => Ok(None),
        }
    })
    .await
}

#[tauri::command]
pub async fn get_last_crawl_config(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Option<CrawlConfig>, AppError> {
    with_repo(&state, move |repo| repo.get_latest_session_config(&project_id)).await
}

#[tauri::command]
pub async fn start_crawl(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    config: CrawlConfig,
    project_id: String,
) -> Result<(), AppError> {
    start_crawl_internal(app, state.inner().clone(), config, &project_id).await
}

/// Shared crawl-launch path used by the `start_crawl` command and the cron
/// scheduler. Registers the crawl in `AppState.crawls`, emits `crawl-started`
/// and spawns the engine on the background runtime.
pub(crate) async fn start_crawl_internal(
    app: AppHandle,
    state: Arc<RwLock<AppState>>,
    config: CrawlConfig,
    project_id: &str,
) -> Result<(), AppError> {
    info!(
        "start_crawl called for project: {} with config: {:?}",
        project_id, config
    );

    // Check if already running for this project
    {
        let state_read = state.read().await;
        let crawls = state_read.crawls.read().await;
        if crawls.contains_key(project_id) {
            return Err(AppError::Crawl(format!(
                "Crawl already running for project: {}",
                project_id
            )));
        }
    }

    // Create cancellation token
    let token = tokio_util::sync::CancellationToken::new();

    // Initial progress
    let progress = CrawlProgress {
        project_id: project_id.to_string(),
        urls_crawled: 0,
        urls_queued: config.seed_urls.len() as u32,
        current_url: String::new(),
        errors: 0,
        elapsed_secs: 0,
    };

    // Store crawl state
    {
        let state_write = state.write().await;
        let mut crawls = state_write.crawls.write().await;
        crawls.insert(
            project_id.to_string(),
            CrawlState {
                cancellation: token.clone(),
                progress: progress.clone(),
            },
        );
    }

    // Set project_id on config
    let mut config = config;
    config.project_id = Some(project_id.to_string());

    // Emit started event
    let _ = app.emit(
        "crawl-started",
        serde_json::json!({
            "project_id": project_id,
            "seed_urls": &config.seed_urls,
        }),
    );

    // Start crawl in background
    let state_clone = state.clone();
    let app_handle = Arc::new(app.clone());
    let project_id_clone = project_id.to_string();

    tokio::spawn(async move {
        let mut engine = CrawlEngine::new();
        engine.set_config(config);

        let result = engine
            .start(app_handle.clone(), state_clone.clone(), token, &project_id_clone)
            .await;

        if let Err(e) = result {
            error!("Crawl failed for project {}: {}", project_id_clone, e);
            let _ = app_handle.emit(
                "crawl-error",
                serde_json::json!({
                    "project_id": &project_id_clone,
                    "error": e.to_string(),
                }),
            );
        }

        // Remove crawl state
        {
            let state_write = state_clone.write().await;
            let mut crawls = state_write.crawls.write().await;
            crawls.remove(&project_id_clone);
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_crawl(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<(), AppError> {
    info!("stop_crawl called for project: {}", project_id);

    let state = state.inner().clone();
    let state_write = state.write().await;
    let mut crawls = state_write.crawls.write().await;

    if let Some(crawl_state) = crawls.remove(&project_id) {
        drop(crawls);
        drop(state_write);
        crawl_state.cancellation.cancel();
        info!("Crawl cancellation signal sent for project: {}", project_id);
        let _ = app.emit(
            "crawl-stopped",
            serde_json::json!({ "project_id": &project_id }),
        );
    } else {
        warn!("No crawl running for project: {}", project_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_crawl_status(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Option<CrawlProgress>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let crawls = state_read.crawls.read().await;
    let progress = crawls.get(&project_id).map(|s| s.progress.clone());
    Ok(progress)
}

#[tauri::command]
pub async fn get_running_crawls(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<Vec<String>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let crawls = state_read.crawls.read().await;
    Ok(crawls.keys().cloned().collect())
}
