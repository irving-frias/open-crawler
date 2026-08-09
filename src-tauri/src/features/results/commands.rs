use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use crate::crawler::fetcher::HtmlFetcher;
use crate::error::AppError;
use crate::features::with_repo;
use crate::models::{
    CrawlResult, IssueCount, PageDetail, PaginatedResults, SiteTreeFullNode, SiteTreeNode,
};
use crate::AppState;

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
    let (items, total) = with_repo(&state, move |repo| {
        repo.get_results(
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
        )
    })
    .await?;

    Ok(PaginatedResults {
        items,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub async fn get_site_tree(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    url: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<SiteTreeNode>, AppError> {
    with_repo(&state, move |repo| {
        repo.get_site_tree(&project_id, url.as_deref(), limit.unwrap_or(100))
    })
    .await
}

#[tauri::command]
pub async fn get_site_tree_full(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Vec<SiteTreeFullNode>, AppError> {
    with_repo(&state, move |repo| repo.get_site_tree_full(&project_id)).await
}

#[tauri::command]
pub async fn get_page_detail(
    state: State<'_, Arc<RwLock<AppState>>>,
    page_id: String,
) -> Result<PageDetail, AppError> {
    with_repo(&state, move |repo| repo.get_page_detail(&page_id)).await
}

#[tauri::command]
pub async fn get_semantic_issue_counts(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Vec<IssueCount>, AppError> {
    with_repo(&state, move |repo| {
        repo.get_semantic_issue_counts(&project_id)
    })
    .await
}

#[tauri::command]
pub async fn get_page_html(
    state: State<'_, Arc<RwLock<AppState>>>,
    page_id: String,
) -> Result<Option<String>, AppError> {
    with_repo(&state, move |repo| repo.get_page_html(&page_id)).await
}

#[tauri::command]
pub async fn inline_assets(html: String, base_url: String) -> Result<String, AppError> {
    let url = url::Url::parse(&base_url)?;
    crate::crawler::assets::inline_page_assets(&html, &url).await
}

#[tauri::command]
pub async fn recrawl_page(
    state: State<'_, Arc<RwLock<AppState>>>,
    page_id: String,
) -> Result<CrawlResult, AppError> {
    // Get the original page data
    let pid = page_id.clone();
    let original = with_repo(&state, move |repo| repo.get_page_detail(&pid))
        .await?
        .page;

    let url = url::Url::parse(&original.url)?;
    let project_id = original.project_id.clone();
    let config_id = original.config_id.clone();
    let depth = original.depth;

    info!("Re-crawling single page: {}", original.url);

    // Reuse the project's cookies and Basic Auth so authenticated pages stay reachable.
    let project_id_cfg = project_id.clone();
    let config = with_repo(&state, move |repo| {
        repo.get_latest_session_config(&project_id_cfg)
    })
    .await?;
    let cookies = config
        .as_ref()
        .map(|c| c.cookies.clone())
        .unwrap_or_default();
    let site_auth = config.and_then(|c| c.site_auth);

    // Fetch the page
    let user_agent = crate::models::crawl_config::IMPLICIT_USER_AGENT;
    let fetcher = crate::crawler::fetcher::HttpFetcher::new(
        user_agent,
        30000,
        Vec::new(),
        cookies,
        site_auth,
        None,
    )?;
    let response = fetcher.fetch(&url).await?;

    // Parse SEO data
    let parser = crate::crawler::parser::SeoParser::new();
    let (seo_data, _outgoing_urls) = parser.parse(&response.html, &url);

    // Re-run the SEO audit for the fresh response.
    let seo_audit = crate::seo::audit::audit_page(
        &seo_data,
        &response.html,
        &crate::seo::AuditContext {
            url: response.url.to_string(),
            status_code: response.status,
            size_bytes: response.size_bytes,
            load_time_ms: response.load_time_ms,
            pagespeed_score: None,
        },
    );
    let seo_audit_json = serde_json::to_string(&seo_audit).ok();
    let seo_score = Some(seo_audit.score);

    // Inline assets for the HTML preview
    let html_body = match crate::crawler::assets::inline_page_assets(&response.html, &url).await {
        Ok(inlined) => Some(inlined),
        Err(_) => {
            let html = &response.html;
            let max_bytes = 100 * 1024;
            if html.len() > max_bytes {
                Some(crate::crawler::truncate_bytes(html, max_bytes))
            } else {
                Some(html.clone())
            }
        }
    };

    // Capture screenshot
    let screenshot_png = {
        let url_str = original.url.clone();
        tokio::task::spawn_blocking(move || {
            crate::crawler::screenshot::capture_screenshot(&url_str)
        })
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
    let keywords_json = if seo_data.keywords.is_empty() {
        None
    } else {
        serde_json::to_string(&seo_data.keywords).ok()
    };
    let og_json = if seo_data.og_meta.is_empty() {
        None
    } else {
        serde_json::to_string(&seo_data.og_meta).ok()
    };

    let result = crate::models::CrawlResult {
        id: page_id.clone(),
        config_id,
        project_id,
        url: response.url.to_string(),
        status_code: Some(response.status),
        blocked: response.blocked,
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
        readability_score: seo_data.readability_score,
        content_hash: seo_data.content_hash,
        duplicate_group_id: None,
        keywords_json,
        og_json,
        pagespeed_score: None,
        pagespeed_json: None,
        seo_score,
        seo_audit_json,
    };

    // Save to DB
    let result_for_save = result.clone();
    with_repo(&state, move |repo| {
        repo.save_result(&result_for_save)?;
        if let Some(ref png) = screenshot_png {
            repo.save_screenshot(&page_id, png)?;
        }
        Ok(())
    })
    .await?;

    info!("Re-crawl complete for: {}", original.url);
    Ok(result)
}

#[tauri::command]
pub async fn capture_page_screenshot(
    state: State<'_, Arc<RwLock<AppState>>>,
    page_id: String,
) -> Result<Option<String>, AppError> {
    // Check if screenshot already exists
    let pid = page_id.clone();
    let existing = with_repo(&state, move |repo| repo.get_screenshot(&pid)).await?;
    if let Some(png) = existing {
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
        return Ok(Some(format!("data:image/png;base64,{}", b64)));
    }

    // Get page URL
    let pid = page_id.clone();
    let url = with_repo(&state, move |repo| {
        let detail = repo.get_page_detail(&pid)?;
        Ok(detail.page.url)
    })
    .await?;

    info!("Capturing screenshot for: {}", url);

    // Capture screenshot
    let png_data = {
        let url_clone = url.clone();
        tokio::task::spawn_blocking(move || {
            crate::crawler::screenshot::capture_screenshot(&url_clone)
        })
        .await
        .map_err(|e| AppError::Crawl(e.to_string()))?
        .map_err(|e| AppError::Crawl(e.to_string()))?
    };

    // Save to DB
    let png_for_save = png_data.clone();
    with_repo(&state, move |repo| {
        repo.save_screenshot(&page_id, &png_for_save)
    })
    .await?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Ok(Some(format!("data:image/png;base64,{}", b64)))
}
