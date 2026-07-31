use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
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
pub async fn create_project(
    state: State<'_, Arc<RwLock<AppState>>>,
    request: CreateProjectRequest,
) -> Result<crate::models::Project, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db);
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
    let repo = CrawlRepo::new(&db);
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
    let repo = CrawlRepo::new(&db);
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
    let repo = CrawlRepo::new(&db);
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
    let repo = CrawlRepo::new(&db);
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
    let repo = CrawlRepo::new(&db);
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
    let repo = CrawlRepo::new(&db);

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
    let repo = CrawlRepo::new(&db);
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
    let repo = CrawlRepo::new(&db);
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
    let repo = CrawlRepo::new(&db);
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
        let repo = CrawlRepo::new(&db);
        repo.get_page_detail(&page_id)?
    };

    let url = url::Url::parse(&original.url)?;
    let project_id = original.project_id.clone();
    let config_id = original.config_id.clone();
    let depth = original.depth;

    info!("Re-crawling single page: {}", original.url);

    // Fetch the page
    let user_agent = crate::models::crawl_config::IMPLICIT_USER_AGENT;
    let fetcher = crate::crawler::fetcher::HttpFetcher::new(user_agent)?;
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
        let repo = CrawlRepo::new(&db);
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
        let repo = CrawlRepo::new(&db);
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
        let repo = CrawlRepo::new(&db);
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
        let repo = CrawlRepo::new(&db);
        repo.save_screenshot(&page_id, &png_data)?;
    }

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Ok(Some(format!("data:image/png;base64,{}", b64)))
}

// ==================== RESULTS COMMANDS ====================

#[tauri::command]
pub async fn get_results(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    page: u32,
    page_size: u32,
    semantic_issue_type: Option<String>,
) -> Result<PaginatedResults, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;

    let repo = CrawlRepo::new(&db);
    let (items, total) = repo.get_results(&project_id, page, page_size, semantic_issue_type.as_deref())?;

    Ok(PaginatedResults {
        items,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub async fn export_csv(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    file_path: String,
) -> Result<(), AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;

    let repo = CrawlRepo::new(&db);
    let (items, _) = repo.get_results(&project_id, 1, 10000, None)?;

    let mut wtr = csv::Writer::from_path(&file_path)?;

    wtr.write_record([
        "URL",
        "Status Code",
        "Title",
        "Meta Description",
        "H1",
        "Canonical",
        "Size (bytes)",
        "Load Time (ms)",
    ])?;

    for item in &items {
        wtr.write_record([
            &item.url,
            &item.status_code
                .map(|s| s.to_string())
                .unwrap_or_default(),
            &item.title.clone().unwrap_or_default(),
            &item.meta_description.clone().unwrap_or_default(),
            &item.h1.clone().unwrap_or_default(),
            &item.canonical.clone().unwrap_or_default(),
            &item.size_bytes
                .map(|s| s.to_string())
                .unwrap_or_default(),
            &item.load_time_ms
                .map(|l| l.to_string())
                .unwrap_or_default(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

// ==================== SETTINGS COMMANDS ====================

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> Result<std::collections::HashMap<String, String>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db);
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
    let repo = CrawlRepo::new(&db);
    for (key, value) in &settings {
        repo.set_setting(key, value)?;
    }
    Ok(())
}

// ==================== EXPORT COMMANDS ====================

#[tauri::command]
pub async fn export_full(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    file_path: String,
    format: String,
) -> Result<(), AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let db = state_read.db.lock().map_err(|e| AppError::Crawl(e.to_string()))?;
    let repo = CrawlRepo::new(&db);

    let pages = repo.get_all_results(&project_id)?;
    let links = repo.get_links_for_project(&project_id)?;

    if pages.len() > 100_000 {
        warn!("Export requested for {} pages (over 100K). File may be very large.", pages.len());
    }

    if format == "xlsx" {
        export_xlsx(&pages, &links, &file_path)?;
    } else {
        export_csv_files(&pages, &links, &file_path)?;
    }

    Ok(())
}

fn export_csv_files(
    pages: &[crate::models::CrawlResult],
    links: &[crate::models::PageLink],
    file_path: &str,
) -> Result<(), AppError> {
    let base = std::path::Path::new(file_path)
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("export");
    let dir = std::path::Path::new(file_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("csv");

    // Pages CSV
    let pages_path = dir.join(format!("{}_pages.{}", base, ext));
    {
        let mut wtr = csv::Writer::from_path(&pages_path)?;
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
        for item in pages {
            let indexable = item.is_indexable.map(|i| if i { "Yes".to_string() } else { "No".to_string() }).unwrap_or_else(|| "Unknown".to_string());
            let issues_str = item.semantic_issues_json.clone().unwrap_or_else(|| "[]".to_string());
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
        wtr.flush()?;
    }

    // Issues CSV
    let issues_path = dir.join(format!("{}_issues.{}", base, ext));
    {
        let mut wtr = csv::Writer::from_path(&issues_path)?;
        wtr.write_record([
            "URL",
            "Issue Type",
            "Message",
            "Element",
            "Selector",
            "XPath",
        ])?;
        for page in pages {
            if let Some(ref json_str) = page.semantic_issues_json {
                if let Ok(issues) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                    for issue in &issues {
                        let issue_type = issue.get("issue_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let message = issue.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let element = issue.get("element").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let selector = issue.get("css_selector").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let xpath = issue.get("xpath").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        wtr.write_record([
                            &page.url,
                            &issue_type,
                            &message,
                            &element,
                            &selector,
                            &xpath,
                        ])?;
                    }
                }
            }
        }
        wtr.flush()?;
    }

    // Links CSV
    let links_path = dir.join(format!("{}_links.{}", base, ext));
    {
        let mut wtr = csv::Writer::from_path(&links_path)?;
        wtr.write_record([
            "From URL",
            "To URL",
            "Link Type",
            "Anchor Text",
            "Follow",
        ])?;
        for link in links {
            wtr.write_record([
                &link.from_url,
                &link.to_url,
                &link.link_type,
                &link.anchor_text.clone().unwrap_or_default(),
                &if link.is_follow { "Yes" } else { "No" }.to_string(),
            ])?;
        }
        wtr.flush()?;
    }

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

fn export_xlsx(
    pages: &[crate::models::CrawlResult],
    links: &[crate::models::PageLink],
    file_path: &str,
) -> Result<(), AppError> {
    use rust_xlsxwriter::{Workbook, Format, FormatAlign};

    let mut workbook = Workbook::new();
    let header_fmt = Format::new()
        .set_bold()
        .set_font_color(0xFFFFFF)
        .set_background_color(0x2E3440)
        .set_align(FormatAlign::Center);
    let alt = Format::new().set_background_color(0xF8F9FA);
    let wrap = Format::new().set_text_wrap();

    // Pages sheet
    let ws = workbook.add_worksheet();
    ws.set_name("Pages").map_err(|e| AppError::Crawl(e.to_string()))?;
    for (col, h) in ["URL","Status Code","Title","Meta Description","H1","Canonical","HTML Lang","Indexable","Depth","Parent URL","Size (bytes)","Load Time (ms)"].iter().enumerate() {
        xlsx_str(ws, 0, col as u16, h, Some(&header_fmt))?;
    }
    for (i, p) in pages.iter().enumerate() {
        let r = (i + 1) as u32;
        let f = if i % 2 == 1 { Some(&alt) } else { None };
        xlsx_str(ws, r, 0, &p.url, f)?;
        xlsx_num(ws, r, 1, p.status_code.unwrap_or(0) as f64, f)?;
        xlsx_str(ws, r, 2, p.title.as_deref().unwrap_or(""), Some(&wrap))?;
        xlsx_str(ws, r, 3, p.meta_description.as_deref().unwrap_or(""), Some(&wrap))?;
        xlsx_str(ws, r, 4, p.h1.as_deref().unwrap_or(""), f)?;
        xlsx_str(ws, r, 5, p.canonical.as_deref().unwrap_or(""), f)?;
        xlsx_str(ws, r, 6, p.html_lang.as_deref().unwrap_or(""), f)?;
        let idx = p.is_indexable.map(|v| if v {"Yes"} else {"No"}).unwrap_or("Unknown");
        xlsx_str(ws, r, 7, idx, f)?;
        xlsx_num(ws, r, 8, p.depth as f64, f)?;
        xlsx_str(ws, r, 9, p.parent_url.as_deref().unwrap_or(""), f)?;
        xlsx_num(ws, r, 10, p.size_bytes.unwrap_or(0) as f64, f)?;
        xlsx_num(ws, r, 11, p.load_time_ms.unwrap_or(0) as f64, f)?;
    }
    ws.set_column_width(0, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    ws.set_column_width(2, 40.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    ws.set_column_width(3, 40.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    ws.autofilter(0, 0, (pages.len() as u32).max(1), 11).map_err(|e| AppError::Crawl(e.to_string()))?;
    ws.set_freeze_panes(1, 0).map_err(|e| AppError::Crawl(e.to_string()))?;

    // Issues sheet
    let wi = workbook.add_worksheet();
    wi.set_name("Issues").map_err(|e| AppError::Crawl(e.to_string()))?;
    for (col, h) in ["URL","Issue Type","Message","Element","Selector","XPath"].iter().enumerate() {
        xlsx_str(wi, 0, col as u16, h, Some(&header_fmt))?;
    }
    let mut ir: u32 = 1;
    for p in pages {
        if let Some(ref js) = p.semantic_issues_json {
            if let Ok(issues) = serde_json::from_str::<Vec<serde_json::Value>>(js) {
                for iss in &issues {
                    let f = if ir % 2 == 0 { Some(&alt) } else { None };
                    xlsx_str(wi, ir, 0, &p.url, f)?;
                    xlsx_str(wi, ir, 1, iss.get("issue_type").and_then(|v| v.as_str()).unwrap_or(""), f)?;
                    xlsx_str(wi, ir, 2, iss.get("message").and_then(|v| v.as_str()).unwrap_or(""), Some(&wrap))?;
                    xlsx_str(wi, ir, 3, iss.get("element").and_then(|v| v.as_str()).unwrap_or(""), f)?;
                    xlsx_str(wi, ir, 4, iss.get("css_selector").and_then(|v| v.as_str()).unwrap_or(""), f)?;
                    xlsx_str(wi, ir, 5, iss.get("xpath").and_then(|v| v.as_str()).unwrap_or(""), f)?;
                    ir += 1;
                }
            }
        }
    }
    wi.set_column_width(0, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    wi.set_column_width(2, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    wi.set_column_width(4, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    wi.set_column_width(5, 50.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    if ir > 1 { wi.autofilter(0, 0, ir - 1, 5).map_err(|e| AppError::Crawl(e.to_string()))?; }
    wi.set_freeze_panes(1, 0).map_err(|e| AppError::Crawl(e.to_string()))?;

    // Links sheet
    let wl = workbook.add_worksheet();
    wl.set_name("Links").map_err(|e| AppError::Crawl(e.to_string()))?;
    for (col, h) in ["From URL","To URL","Link Type","Anchor Text","Follow"].iter().enumerate() {
        xlsx_str(wl, 0, col as u16, h, Some(&header_fmt))?;
    }
    for (i, lk) in links.iter().enumerate() {
        let r = (i + 1) as u32;
        let f = if i % 2 == 1 { Some(&alt) } else { None };
        xlsx_str(wl, r, 0, &lk.from_url, f)?;
        xlsx_str(wl, r, 1, &lk.to_url, f)?;
        xlsx_str(wl, r, 2, &lk.link_type, f)?;
        xlsx_str(wl, r, 3, lk.anchor_text.as_deref().unwrap_or(""), f)?;
        xlsx_str(wl, r, 4, if lk.is_follow {"Yes"} else {"No"}, f)?;
    }
    wl.set_column_width(0, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    wl.set_column_width(1, 60.0).map_err(|e| AppError::Crawl(e.to_string()))?;
    if !links.is_empty() { wl.autofilter(0, 0, links.len() as u32, 4).map_err(|e| AppError::Crawl(e.to_string()))?; }
    wl.set_freeze_panes(1, 0).map_err(|e| AppError::Crawl(e.to_string()))?;

    workbook.save(file_path).map_err(|e| AppError::Crawl(e.to_string()))?;
    Ok(())
}
