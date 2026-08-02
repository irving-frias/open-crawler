use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::crawler::engine::CrawlEngine;
use crate::crawler::fetcher::HtmlFetcher;
use crate::db::CrawlRepo;
use crate::error::AppError;
use crate::models::{
    CreateProjectRequest, CrawlConfig, CrawlProgress, IssueCount, PaginatedResults, RenameProjectRequest,
};
use crate::{AppState, CrawlState};

// ==================== PROJECT COMMANDS ====================

#[tauri::command]
pub fn is_mobile() -> bool {
    cfg!(mobile)
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, Arc<RwLock<AppState>>>,
    request: CreateProjectRequest,
) -> Result<crate::models::Project, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    let project = repo.create_project(&request.name)?;
    Ok(project)
}

#[tauri::command]
pub async fn list_projects(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<Vec<crate::models::Project>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    let projects = repo.list_projects()?;
    Ok(projects)
}

#[tauri::command]
pub async fn get_project(
    state: State<'_, Arc<RwLock<AppState>>>,
    id: String,
) -> Result<crate::models::Project, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    let project = repo
        .get_project(&id)?
        .ok_or_else(|| AppError::Crawl(format!("Project not found: {}", id)))?;
    Ok(project)
}

#[tauri::command]
pub async fn rename_project(
    state: State<'_, Arc<RwLock<AppState>>>,
    request: RenameProjectRequest,
) -> Result<(), AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    repo.rename_project(&request.id, &request.name)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, Arc<RwLock<AppState>>>,
    id: String,
) -> Result<(), AppError> {
    let state = state.inner().clone();

    // Stop crawl if running for this project
    {
        let state_write = state.write().await;
        let mut crawls = state_write.crawls.write().await;
        if let Some(crawl_state) = crawls.remove(&id) {
            crawl_state.cancellation.cancel();
            info!("Stopped crawl for deleted project: {}", id);
        }
    }

    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    repo.delete_project(&id)?;
    Ok(())
}

#[tauri::command]
pub async fn get_project_stats(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<serde_json::Value, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    let stats = repo.get_project_stats(&project_id)?;
    Ok(stats)
}

// ==================== CRAWL COMMANDS ====================

#[tauri::command]
pub async fn check_resumable_crawl(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Option<serde_json::Value>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);

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
}

#[tauri::command]
pub async fn start_crawl(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    config: CrawlConfig,
    project_id: String,
) -> Result<(), AppError> {
    info!(
        "start_crawl called for project: {} with config: {:?}",
        project_id, config
    );

    let state = state.inner().clone();

    // Check if already running for this project
    {
        let state_read = state.read().await;
        let crawls = state_read.crawls.read().await;
        if crawls.contains_key(&project_id) {
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
        project_id: project_id.clone(),
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
            project_id.clone(),
            CrawlState {
                cancellation: token.clone(),
                progress: progress.clone(),
            },
        );
    }

    // Set project_id on config
    let mut config = config;
    config.project_id = Some(project_id.clone());

    // Emit started event
    let _ = app.emit(
        "crawl-started",
        serde_json::json!({
            "project_id": &project_id,
            "seed_urls": &config.seed_urls,
        }),
    );

    // Start crawl in background
    let state_clone = state.clone();
    let app_handle = Arc::new(app.clone());
    let project_id_clone = project_id.clone();

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

#[tauri::command]
pub async fn get_page_detail(
    state: State<'_, Arc<RwLock<AppState>>>,
    page_id: String,
) -> Result<(crate::models::CrawlResult, Vec<crate::models::PageLink>), AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    let result = repo.get_page_detail(&page_id)?;
    Ok(result)
}

#[tauri::command]
pub async fn get_semantic_issue_counts(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Vec<IssueCount>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    let counts = repo.get_semantic_issue_counts(&project_id)?;
    Ok(counts)
}

#[tauri::command]
pub async fn get_page_html(
    state: State<'_, Arc<RwLock<AppState>>>,
    page_id: String,
) -> Result<Option<String>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    let html = repo.get_page_html(&page_id)?;
    Ok(html)
}

#[tauri::command]
pub async fn inline_assets(
    html: String,
    base_url: String,
) -> Result<String, AppError> {
    let url = url::Url::parse(&base_url)?;
    crate::crawler::assets::inline_page_assets(&html, &url).await
}

#[tauri::command]
pub async fn recrawl_page(
    state: State<'_, Arc<RwLock<AppState>>>,
    page_id: String,
) -> Result<crate::models::CrawlResult, AppError> {
    let state = state.inner().clone();

    // Get the original page data
    let (original, _links) = {
        let state_read = state.read().await;
        let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
        let repo = CrawlRepo::new(&db, None);
        repo.get_page_detail(&page_id)?
    };

    let url = url::Url::parse(&original.url)?;
    let project_id = original.project_id.clone();
    let config_id = original.config_id.clone();
    let depth = original.depth;

    info!("Re-crawling single page: {}", original.url);

    // Fetch the page
    let user_agent = crate::models::crawl_config::IMPLICIT_USER_AGENT;
    let fetcher = crate::crawler::fetcher::HttpFetcher::new(user_agent, 30000, Vec::new(), None)?;
    let response = fetcher.fetch(&url).await?;

    // Parse SEO data
    let parser = crate::crawler::parser::SeoParser::new();
    let (seo_data, _outgoing_urls) = parser.parse(&response.html, &url);

    // Inline assets for the HTML preview
    let html_body = match crate::crawler::assets::inline_page_assets(&response.html, &url).await {
        Ok(inlined) => Some(inlined),
        Err(_) => {
            let html = &response.html;
            let max_bytes = 100 * 1024;
            if html.len() > max_bytes {
                Some(html[..max_bytes].to_string())
            } else {
                Some(html.clone())
            }
        }
    };

    // Capture screenshot
    let screenshot_png = {
        let url_str = original.url.clone();
        tokio::task::spawn_blocking(move || crate::crawler::screenshot::capture_screenshot(&url_str))
            .await
            .map_err(|e| AppError::Crawl(e.to_string()))?
            .ok()
    };

    // Serialize
    let hreflang_json = if seo_data.hreflang_links.is_empty() {
        None
    } else {
        serde_json::to_string(&seo_data.hreflang_links).ok()
    };
    let semantic_issues_json = if seo_data.semantic_issues.is_empty() {
        None
    } else {
        serde_json::to_string(&seo_data.semantic_issues).ok()
    };

    let result = crate::models::CrawlResult {
        id: page_id.clone(),
        config_id,
        project_id,
        url: response.url.to_string(),
        status_code: Some(response.status),
        title: seo_data.title,
        meta_description: seo_data.meta_description,
        h1: seo_data.h1,
        canonical: seo_data.canonical,
        size_bytes: Some(response.size_bytes),
        load_time_ms: Some(response.load_time_ms),
        is_indexable: Some(
            !seo_data
                .meta_robots
                .as_deref()
                .map(|r| r.contains("noindex"))
                .unwrap_or(false),
        ),
        depth,
        parent_url: original.parent_url,
        crawl_timestamp: chrono::Utc::now().to_rfc3339(),
        links: Vec::new(),
        html_lang: seo_data.html_lang,
        hreflang_json,
        semantic_issues_json,
        html_body,
    };

    // Save to DB
    {
        let state_read = state.read().await;
        let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
        let repo = CrawlRepo::new(&db, None);
        repo.save_result(&result)?;
        if let Some(ref png) = screenshot_png {
            repo.save_screenshot(&page_id, png)?;
        }
    }

    info!("Re-crawl complete for: {}", original.url);
    Ok(result)
}

#[tauri::command]
pub async fn capture_page_screenshot(
    state: State<'_, Arc<RwLock<AppState>>>,
    page_id: String,
) -> Result<Option<String>, AppError> {
    let state = state.inner().clone();

    // Check if screenshot already exists
    {
        let state_read = state.read().await;
        let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
        let repo = CrawlRepo::new(&db, None);
        if let Some(png) = repo.get_screenshot(&page_id)? {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
            return Ok(Some(format!("data:image/png;base64,{}", b64)));
        }
    }

    // Get page URL
    let url = {
        let state_read = state.read().await;
        let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
        let repo = CrawlRepo::new(&db, None);
        let (detail, _) = repo.get_page_detail(&page_id)?;
        detail.url
    };

    info!("Capturing screenshot for: {}", url);

    // Capture screenshot
    let png_data = {
        let url_clone = url.clone();
        tokio::task::spawn_blocking(move || crate::crawler::screenshot::capture_screenshot(&url_clone))
            .await
            .map_err(|e| AppError::Crawl(e.to_string()))?
            .map_err(|e| AppError::Crawl(e.to_string()))?
    };

    // Save to DB
    {
        let state_read = state.read().await;
        let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
        let repo = CrawlRepo::new(&db, None);
        repo.save_screenshot(&page_id, &png_data)?;
    }

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Ok(Some(format!("data:image/png;base64,{}", b64)))
}

// ==================== RESULTS COMMANDS ====================

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn get_results(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    page: u32,
    page_size: u32,
    semantic_issue_type: Option<String>,
    search: Option<String>,
    status_filter: Option<Vec<u32>>,
    severity_filter: Option<Vec<String>>,
    domain_filter: Option<String>,
    depth_filter: Option<u32>,
    missing_title: Option<bool>,
    duplicate_title: Option<bool>,
    noindex_only: Option<bool>,
    is_404: Option<bool>,
) -> Result<PaginatedResults, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;

    let repo = CrawlRepo::new(&db, Some(state_read.results_cache.clone()));
    let (items, total) = repo.get_results(
        &project_id,
        page,
        page_size,
        semantic_issue_type.as_deref(),
        search.as_deref(),
        status_filter.as_deref(),
        severity_filter.as_deref(),
        domain_filter.as_deref(),
        depth_filter,
        missing_title.unwrap_or(false),
        duplicate_title.unwrap_or(false),
        noindex_only.unwrap_or(false),
        is_404.unwrap_or(false),
    )?;

    Ok(PaginatedResults {
        items,
        total,
        page,
        page_size,
    })
}

// ==================== SITE TREE COMMANDS ====================

#[tauri::command]
pub async fn get_site_tree(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    url: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<crate::models::SiteTreeNode>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;

    let repo = CrawlRepo::new(&db, None);
    repo.get_site_tree(&project_id, url.as_deref(), limit.unwrap_or(100))
}

// ==================== SETTINGS COMMANDS ====================

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<std::collections::HashMap<String, String>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    repo.get_all_settings()
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    settings: std::collections::HashMap<String, String>,
) -> Result<(), AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);
    for (key, value) in &settings {
        repo.set_setting(key, value)?;
    }
    Ok(())
}

// ==================== EXPORT COMMANDS ====================

const PAGE_BATCH_SIZE: u32 = 1000;
const LINK_BATCH_SIZE: u32 = 5000;

#[derive(serde::Serialize, Clone)]
struct ExportProgress {
    stage: &'static str,
    processed: u64,
    total: u64,
    percent: f32,
}

fn emit_export_progress(app: &AppHandle, stage: &'static str, processed: u64, total: u64) {
    let percent = if total == 0 {
        100.0
    } else {
        (processed as f32 / total as f32) * 100.0
    };
    let _ = app.emit(
        "export-progress",
        ExportProgress {
            stage,
            processed,
            total,
            percent,
        },
    );
}

#[tauri::command]
pub async fn export_full(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    file_path: String,
    format: String,
) -> Result<(), AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db, None);

    let total_pages = repo.count_pages(&project_id)?;
    if total_pages > 100_000 {
        warn!(
            "Export requested for {} pages (over 100K). File may be very large.",
            total_pages
        );
    }

    let (write_path, share_after) = export_target(&app, &file_path, &format)?;

    if format == "xlsx" {
        let total_links = repo.count_links(&project_id)?;
        let total_issues = repo.count_issues(&project_id)?;
        export_xlsx(
            &repo,
            &app,
            &project_id,
            &write_path,
            total_pages,
            total_links,
            total_issues,
        )?;
    } else {
        export_csv_single(&repo, &app, &project_id, &write_path, total_pages)?;
    }

    if share_after {
        share_export(&app, &write_path, &format);
    }

    Ok(())
}

/// Resolves the destination file for an export.
///
/// On desktop the user picks a path via the save dialog and the file is
/// written straight to it. On mobile that dialog returns a `content://` URI
/// that `std::fs` cannot open (os error 2 / ENOENT), so the file is written to
/// the app data dir instead and the native share sheet is opened afterwards.
fn export_target(
    app: &AppHandle,
    file_path: &str,
    format: &str,
) -> Result<(String, bool), AppError> {
    if !cfg!(mobile) {
        return Ok((file_path.to_string(), false));
    }

    let ext = if format == "xlsx" { "xlsx" } else { "csv" };
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty() && !n.starts_with("content:"))
        .map(str::to_string)
        .unwrap_or_else(|| format!("crawl-results.{ext}"));

    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Crawl(e.to_string()))?
        .join("exports");
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(&file_name);

    Ok((dest.to_string_lossy().into_owned(), true))
}

#[cfg(mobile)]
fn share_export(app: &AppHandle, path: &str, format: &str) {
    use tauri_plugin_share::ShareExt;

    let mime = if format == "xlsx" {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    } else {
        "text/csv"
    };

    let app = app.clone();
    let path = path.to_string();
    let mime = mime.to_string();

    // The Android share plugin never resolves its invoke after launching the
    // share sheet, so run it on a detached thread to avoid blocking.
    std::thread::spawn(move || {
        let _ = app.share_file().share_file(tauri_plugin_share::ShareRequest {
            path: Some(path),
            mime: Some(mime),
            group: None,
        });
    });
}

#[cfg(not(mobile))]
fn share_export(_app: &AppHandle, _path: &str, _format: &str) {}

fn export_csv_single(
    repo: &CrawlRepo,
    app: &AppHandle,
    project_id: &str,
    file_path: &str,
    total_pages: u32,
) -> Result<(), AppError> {
    let mut wtr = csv::Writer::from_path(file_path)?;

    wtr.write_record([
        "URL",
        "Status Code",
        "Title",
        "Meta Description",
        "H1",
        "Canonical",
        "HTML Lang",
        "Indexable",
        "Depth",
        "Parent URL",
        "Size (bytes)",
        "Load Time (ms)",
        "Semantic Issues",
    ])?;

    let mut last_timestamp: Option<String> = None;
    let mut last_id: Option<String> = None;
    let mut processed: u64 = 0;

    loop {
        let batch = repo.get_result_batch(
            project_id,
            last_timestamp.as_deref(),
            last_id.as_deref(),
            PAGE_BATCH_SIZE,
        )?;
        if batch.is_empty() {
            break;
        }
        for item in &batch {
            let indexable = item
                .is_indexable
                .map(|i| if i { "Yes".to_string() } else { "No".to_string() })
                .unwrap_or_else(|| "Unknown".to_string());
            let issues_str = item
                .semantic_issues_json
                .clone()
                .unwrap_or_else(|| "[]".to_string());
            wtr.write_record([
                &item.url,
                &item.status_code.map(|s| s.to_string()).unwrap_or_default(),
                &item.title.clone().unwrap_or_default(),
                &item.meta_description.clone().unwrap_or_default(),
                &item.h1.clone().unwrap_or_default(),
                &item.canonical.clone().unwrap_or_default(),
                &item.html_lang.clone().unwrap_or_default(),
                &indexable,
                &item.depth.to_string(),
                &item.parent_url.clone().unwrap_or_default(),
                &item.size_bytes.map(|s| s.to_string()).unwrap_or_default(),
                &item.load_time_ms.map(|l| l.to_string()).unwrap_or_default(),
                &issues_str,
            ])?;
        }
        processed += batch.len() as u64;
        emit_export_progress(app, "pages", processed, total_pages as u64);
        let last = batch.last().expect("batch is not empty");
        last_timestamp = Some(last.crawl_timestamp.clone());
        last_id = Some(last.id.clone());
    }

    wtr.flush()?;
    emit_export_progress(app, "pages", total_pages as u64, total_pages as u64);
    Ok(())
}

fn xlsx_str(ws: &mut rust_xlsxwriter::Worksheet, row: u32, col: u16, val: &str, fmt: Option<&rust_xlsxwriter::Format>) -> Result<(), AppError> {
    let res = match fmt {
        Some(f) => ws.write_string_with_format(row, col, val.to_string(), f),
        None => ws.write_string(row, col, val.to_string()),
    };
    res.map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}

fn xlsx_num(ws: &mut rust_xlsxwriter::Worksheet, row: u32, col: u16, val: f64, fmt: Option<&rust_xlsxwriter::Format>) -> Result<(), AppError> {
    let res = match fmt {
        Some(f) => ws.write_number_with_format(row, col, val, f),
        None => ws.write_number(row, col, val),
    };
    res.map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}

const MAX_ROWS_PER_SHEET: u32 = 1_048_575;

fn sheet_name(base: &str, index: usize, total: usize) -> String {
    if total <= 1 {
        base.to_string()
    } else {
        format!("{} {}", base, index + 1)
    }
}

fn sheet_count(rows: u32) -> usize {
    rows.div_ceil(MAX_ROWS_PER_SHEET) as usize
}

fn write_page_row(ws: &mut rust_xlsxwriter::Worksheet, row: u32, p: &crate::models::CrawlResult, alt: &rust_xlsxwriter::Format, wrap: &rust_xlsxwriter::Format) -> Result<(), AppError> {
    let f = if row.is_multiple_of(2) { Some(alt) } else { None };
    xlsx_str(ws, row, 0, &p.url, f)?;
    xlsx_num(ws, row, 1, p.status_code.unwrap_or(0) as f64, f)?;
    xlsx_str(ws, row, 2, p.title.as_deref().unwrap_or(""), Some(wrap))?;
    xlsx_str(ws, row, 3, p.meta_description.as_deref().unwrap_or(""), Some(wrap))?;
    xlsx_str(ws, row, 4, p.h1.as_deref().unwrap_or(""), f)?;
    xlsx_str(ws, row, 5, p.canonical.as_deref().unwrap_or(""), f)?;
    xlsx_str(ws, row, 6, p.html_lang.as_deref().unwrap_or(""), f)?;
    let idx = p.is_indexable.map(|v| if v {"Yes"} else {"No"}).unwrap_or("Unknown");
    xlsx_str(ws, row, 7, idx, f)?;
    xlsx_num(ws, row, 8, p.depth as f64, f)?;
    xlsx_str(ws, row, 9, p.parent_url.as_deref().unwrap_or(""), f)?;
    xlsx_num(ws, row, 10, p.size_bytes.unwrap_or(0) as f64, f)?;
    xlsx_num(ws, row, 11, p.load_time_ms.unwrap_or(0) as f64, f)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_issue_row(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    url: &str,
    iss: &serde_json::Value,
    wrap: &rust_xlsxwriter::Format,
    err_fmt: &rust_xlsxwriter::Format,
    warn_fmt: &rust_xlsxwriter::Format,
    info_fmt: &rust_xlsxwriter::Format,
) -> Result<(), AppError> {
    xlsx_str(ws, row, 0, url, None)?;
    xlsx_str(ws, row, 1, iss.get("issue_type").and_then(|v| v.as_str()).unwrap_or(""), None)?;
    let sev = iss.get("severity").and_then(|v| v.as_str()).unwrap_or("");
    let sev_fmt = match sev {
        "error" => Some(err_fmt),
        "warning" => Some(warn_fmt),
        "info" => Some(info_fmt),
        _ => None,
    };
    xlsx_str(ws, row, 2, sev, sev_fmt)?;
    xlsx_str(ws, row, 3, iss.get("message").and_then(|v| v.as_str()).unwrap_or(""), Some(wrap))?;
    xlsx_str(ws, row, 4, iss.get("element").and_then(|v| v.as_str()).unwrap_or(""), None)?;
    xlsx_str(ws, row, 5, iss.get("css_selector").and_then(|v| v.as_str()).unwrap_or(""), None)?;
    xlsx_str(ws, row, 6, iss.get("xpath").and_then(|v| v.as_str()).unwrap_or(""), None)?;
    Ok(())
}

fn finalize_sheet(ws: &mut rust_xlsxwriter::Worksheet, data_rows: u32, num_cols: u16) -> Result<(), AppError> {
    ws.set_column_width(0, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    if num_cols > 2 { ws.set_column_width(2, 40.0).map_err(|e| AppError::Crawl(e.to_string()))?; }
    if num_cols > 3 { ws.set_column_width(3, 40.0).map_err(|e| AppError::Crawl(e.to_string()))?; }
    if data_rows > 0 {
        ws.autofilter(0, 0, data_rows, num_cols - 1).map_err(|e| AppError::Crawl(e.to_string()))?;
    }
    ws.set_freeze_panes(1, 0).map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}

const PAGE_HEADERS: [&str; 12] = ["URL","Status Code","Title","Meta Description","H1","Canonical","HTML Lang","Indexable","Depth","Parent URL","Size (bytes)","Load Time (ms)"];
const ISSUE_HEADERS: [&str; 7] = ["URL","Issue Type","Severity","Message","Element","Selector","XPath"];
const LINK_HEADERS: [&str; 5] = ["From URL","To URL","Link Type","Anchor Text","Follow"];

fn export_xlsx(
    repo: &CrawlRepo,
    app: &AppHandle,
    project_id: &str,
    file_path: &str,
    total_pages: u32,
    total_links: u32,
    total_issues: u32,
) -> Result<(), AppError> {
    use rust_xlsxwriter::{Format, FormatAlign, Workbook};

    let mut workbook = Workbook::new();
    let header_fmt = Format::new()
        .set_bold()
        .set_font_color(0xFFFFFF)
        .set_background_color(0x2E3440)
        .set_align(FormatAlign::Center);
    let alt = Format::new().set_background_color(0xF8F9FA);
    let wrap = Format::new().set_text_wrap();
    let err_fmt = Format::new().set_background_color(0xFFE0E0).set_font_color(0x721C24);
    let warn_fmt = Format::new().set_background_color(0xFFF3CD).set_font_color(0x856404);
    let info_fmt = Format::new().set_background_color(0xD1ECF1).set_font_color(0x0C5460);

    let total = total_pages as u64 + total_issues as u64 + total_links as u64;
    let mut processed: u64 = 0;

    // === Pass 1: Pages sheets (streamed by batch, split if > 1,048,575 pages) ===
    if total_pages > 0 {
        let num_sheets = sheet_count(total_pages);
        let mut sheet_idx: usize = 0;
        let mut sheet_rows: u32 = 1;
        let mut ws = workbook.add_worksheet_with_constant_memory();
        ws.set_name(sheet_name("Pages", 0, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
        for (col, h) in PAGE_HEADERS.iter().enumerate() {
            xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
        }

        let mut last_timestamp: Option<String> = None;
        let mut last_id: Option<String> = None;
        loop {
            let batch = repo.get_result_batch(
                project_id,
                last_timestamp.as_deref(),
                last_id.as_deref(),
                PAGE_BATCH_SIZE,
            )?;
            if batch.is_empty() {
                break;
            }
            for p in &batch {
                if sheet_rows > MAX_ROWS_PER_SHEET {
                    finalize_sheet(ws, MAX_ROWS_PER_SHEET, 12)?;
                    sheet_idx += 1;
                    ws = workbook.add_worksheet_with_constant_memory();
                    ws.set_name(sheet_name("Pages", sheet_idx, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
                    for (col, h) in PAGE_HEADERS.iter().enumerate() {
                        xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
                    }
                    sheet_rows = 1;
                }
                write_page_row(ws, sheet_rows, p, &alt, &wrap)?;
                sheet_rows += 1;
            }
            processed += batch.len() as u64;
            emit_export_progress(app, "pages", processed, total);
            let last = batch.last().expect("batch is not empty");
            last_timestamp = Some(last.crawl_timestamp.clone());
            last_id = Some(last.id.clone());
        }
        finalize_sheet(ws, sheet_rows.saturating_sub(1), 12)?;
    }

    // === Pass 2: Issues sheets (re-reads pages by batch, split if > 1,048,575 issues) ===
    if total_issues > 0 {
        let num_sheets = sheet_count(total_issues);
        let mut sheet_idx: usize = 0;
        let mut sheet_rows: u32 = 1;
        let mut ws = workbook.add_worksheet_with_constant_memory();
        ws.set_name(sheet_name("Issues", 0, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
        for (col, h) in ISSUE_HEADERS.iter().enumerate() {
            xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
        }

        let mut last_timestamp: Option<String> = None;
        let mut last_id: Option<String> = None;
        loop {
            let batch = repo.get_result_batch(
                project_id,
                last_timestamp.as_deref(),
                last_id.as_deref(),
                PAGE_BATCH_SIZE,
            )?;
            if batch.is_empty() {
                break;
            }
            let mut issues_written: u64 = 0;
            for p in &batch {
                if let Some(ref js) = p.semantic_issues_json {
                    if let Ok(issues) = serde_json::from_str::<Vec<serde_json::Value>>(js) {
                        for iss in &issues {
                            if sheet_rows > MAX_ROWS_PER_SHEET {
                                finalize_sheet(ws, MAX_ROWS_PER_SHEET, 7)?;
                                ws.set_column_width(3, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
                                ws.set_column_width(5, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
                                ws.set_column_width(6, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
                                sheet_idx += 1;
                                ws = workbook.add_worksheet_with_constant_memory();
                                ws.set_name(sheet_name("Issues", sheet_idx, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
                                for (col, h) in ISSUE_HEADERS.iter().enumerate() {
                                    xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
                                }
                                sheet_rows = 1;
                            }
                            write_issue_row(ws, sheet_rows, &p.url, iss, &wrap, &err_fmt, &warn_fmt, &info_fmt)?;
                            sheet_rows += 1;
                            issues_written += 1;
                        }
                    }
                }
            }
            processed += issues_written;
            emit_export_progress(app, "issues", processed, total);
            let last = batch.last().expect("batch is not empty");
            last_timestamp = Some(last.crawl_timestamp.clone());
            last_id = Some(last.id.clone());
        }
        finalize_sheet(ws, sheet_rows.saturating_sub(1), 7)?;
        ws.set_column_width(3, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
        ws.set_column_width(5, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
        ws.set_column_width(6, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    }

    // === Pass 3: Links sheets (streamed by rowid batch, split if > 1,048,575 links) ===
    if total_links > 0 {
        let num_sheets = sheet_count(total_links);
        let mut sheet_idx: usize = 0;
        let mut sheet_rows: u32 = 1;
        let mut ws = workbook.add_worksheet_with_constant_memory();
        ws.set_name(sheet_name("Links", 0, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
        for (col, h) in LINK_HEADERS.iter().enumerate() {
            xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
        }

        let mut last_rowid: Option<i64> = None;
        loop {
            let batch = repo.get_links_batch(project_id, last_rowid, LINK_BATCH_SIZE)?;
            if batch.is_empty() {
                break;
            }
            for (_, lk) in &batch {
                if sheet_rows > MAX_ROWS_PER_SHEET {
                    finalize_sheet(ws, MAX_ROWS_PER_SHEET, 5)?;
                    ws.set_column_width(1, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
                    sheet_idx += 1;
                    ws = workbook.add_worksheet_with_constant_memory();
                    ws.set_name(sheet_name("Links", sheet_idx, num_sheets)).map_err(|e| AppError::Crawl(e.to_string()))?;
                    for (col, h) in LINK_HEADERS.iter().enumerate() {
                        xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
                    }
                    sheet_rows = 1;
                }
                let r = sheet_rows;
                let f = if r.is_multiple_of(2) { Some(&alt) } else { None };
                xlsx_str(ws, r, 0, &lk.from_url, f)?;
                xlsx_str(ws, r, 1, &lk.to_url, f)?;
                xlsx_str(ws, r, 2, &lk.link_type, f)?;
                xlsx_str(ws, r, 3, lk.anchor_text.as_deref().unwrap_or(""), f)?;
                xlsx_str(ws, r, 4, if lk.is_follow { "Yes" } else { "No" }, f)?;
                sheet_rows += 1;
            }
            processed += batch.len() as u64;
            emit_export_progress(app, "links", processed, total);
            last_rowid = Some(batch.last().expect("batch is not empty").0);
        }
        finalize_sheet(ws, sheet_rows.saturating_sub(1), 5)?;
        ws.set_column_width(1, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    }

    workbook.save(file_path).map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}
