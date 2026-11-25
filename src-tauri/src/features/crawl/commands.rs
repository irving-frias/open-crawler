use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::crawler::engine::{CrawlEngine, CrawlSink};
use crate::error::AppError;
use crate::features::{with_repo, with_repo_arc};
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
    with_repo(&state, move |repo| {
        repo.get_latest_session_config(&project_id)
    })
    .await
}

#[tauri::command]
pub async fn start_crawl(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    config: CrawlConfig,
    project_id: String,
    resume: bool,
) -> Result<(), AppError> {
    start_crawl_internal(app, state.inner().clone(), config, &project_id, resume).await
}

/// Shared crawl-launch path used by the `start_crawl` command and the cron
/// scheduler. Registers the crawl in `AppState.crawls`, emits `crawl-started`
/// and spawns the engine on the background runtime. `resume` decides whether
/// an interrupted session is restored (`true`) or discarded for a fresh
/// start (`false`); the engine used to auto-resume unconditionally, so a
/// "fresh" start silently resumed the old queue.
pub(crate) async fn start_crawl_internal(
    app: AppHandle,
    state: Arc<RwLock<AppState>>,
    mut config: CrawlConfig,
    project_id: &str,
    resume: bool,
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
        errors_4xx: 0,
        errors_5xx: 0,
        errors_timeout: 0,
        current_depth: 0,
        last_error: None,
        retrying: None,
        retries_total: 0,
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
    let sink = Arc::new(CrawlSink::Tauri(Arc::new(app.clone())));
    let app_for_error = app.clone();
    let project_id_clone = project_id.to_string();
    let http_client = app.state::<reqwest::Client>().inner().clone();

    tokio::spawn(async move {
        let mut engine = CrawlEngine::new();
        engine.set_config(config);
        engine.set_http_client(http_client);

        let result = engine
            .start(sink, state_clone.clone(), token, &project_id_clone, resume)
            .await;

        if let Err(e) = result {
            error!("Crawl failed for project {}: {}", project_id_clone, e);
            let _ = app_for_error.emit(
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

/// Register a crawl in `AppState.crawls` (shared by the headless launchers).
/// Fails when a crawl is already running for the project. Sets
/// `config.project_id` and returns the cancellation token.
async fn register_crawl(
    state: &Arc<RwLock<AppState>>,
    config: &mut CrawlConfig,
    project_id: &str,
) -> Result<CancellationToken, AppError> {
    {
        let state_read = state.read().await;
        let crawls = state_read.crawls.read().await;
        if crawls.contains_key(project_id) {
            return Err(AppError::Crawl(format!(
                "Crawl already running for project: {project_id}"
            )));
        }
    }
    let token = CancellationToken::new();
    let progress = CrawlProgress {
        project_id: project_id.to_string(),
        urls_crawled: 0,
        urls_queued: (config.seed_urls.len() + config.url_list.len()) as u32,
        current_url: String::new(),
        errors: 0,
        elapsed_secs: 0,
        errors_4xx: 0,
        errors_5xx: 0,
        errors_timeout: 0,
        current_depth: 0,
        last_error: None,
        retrying: None,
        retries_total: 0,
    };
    {
        let state_write = state.write().await;
        let mut crawls = state_write.crawls.write().await;
        crawls.insert(
            project_id.to_string(),
            CrawlState {
                cancellation: token.clone(),
                progress,
            },
        );
    }
    config.project_id = Some(project_id.to_string());
    Ok(token)
}

async fn deregister_crawl(state: &Arc<RwLock<AppState>>, project_id: &str) {
    let state_write = state.write().await;
    let mut crawls = state_write.crawls.write().await;
    crawls.remove(project_id);
}

/// Headless crawl launch (MCP server): same engine path as
/// `start_crawl_internal`, but without Tauri events — progress goes to the
/// log via `CrawlSink::Headless`. Returns immediately after spawning; the
/// crawl stays visible in `AppState.crawls` until it finishes.
pub async fn start_crawl_headless(
    state: Arc<RwLock<AppState>>,
    mut config: CrawlConfig,
    project_id: &str,
    resume: bool,
) -> Result<(), AppError> {
    let token = register_crawl(&state, &mut config, project_id).await?;
    let state_clone = state.clone();
    let project_id_clone = project_id.to_string();
    let proxy = config.proxy.clone();
    tokio::spawn(async move {
        let mut engine = CrawlEngine::new();
        engine.set_config(config);
        // Headless crawls had no shared HTTP client, so the engine skipped
        // the site-resource audit (robots.txt / sitemap.xml / llms.txt were
        // silently absent from every MCP/CLI crawl). Build one here so
        // `fetch_and_parse` runs `audit_page_with_site`.
        if let Ok(builder) = crate::crawler::client_with_proxy(proxy.as_ref()) {
            if let Ok(http_client) = builder.build() {
                engine.set_http_client(http_client);
            }
        }
        let result = engine
            .start(
                Arc::new(CrawlSink::Headless),
                state_clone.clone(),
                token,
                &project_id_clone,
                resume,
            )
            .await;
        if let Err(e) = result {
            error!("Crawl failed for project {}: {}", project_id_clone, e);
        }
        deregister_crawl(&state_clone, &project_id_clone).await;
    });
    Ok(())
}

/// Headless crawl run to completion (CLI `--config` mode): drives the engine
/// inline instead of spawning, then returns `(pages_crawled, elapsed_secs)`.
pub async fn run_crawl_blocking(
    state: Arc<RwLock<AppState>>,
    mut config: CrawlConfig,
    project_id: &str,
    resume: bool,
) -> Result<(u32, u64), AppError> {
    let token = register_crawl(&state, &mut config, project_id).await?;
    let started = std::time::Instant::now();
    let proxy = config.proxy.clone();
    let mut engine = CrawlEngine::new();
    engine.set_config(config);
    // Same as start_crawl_headless: without a shared client the engine
    // skips the site-resource audit (see above).
    if let Ok(builder) = crate::crawler::client_with_proxy(proxy.as_ref()) {
        if let Ok(http_client) = builder.build() {
            engine.set_http_client(http_client);
        }
    }
    let result = engine
        .start(
            Arc::new(CrawlSink::Headless),
            state.clone(),
            token,
            project_id,
            resume,
        )
        .await;
    deregister_crawl(&state, project_id).await;
    result?;
    let project_id_owned = project_id.to_string();
    let pages = with_repo_arc(&state, move |repo| repo.count_pages(&project_id_owned)).await?;
    Ok((pages, started.elapsed().as_secs()))
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

/// Probes whether JavaScript rendering can run on this machine, without
/// launching a browser. The UI calls this to enable/disable the JS-rendering
/// toggle (and to show why it is unavailable).
#[tauri::command]
pub async fn check_js_rendering_available(
) -> Result<crate::crawler::fetcher::JsAvailability, AppError> {
    Ok(crate::crawler::fetcher::check_js_availability())
}
