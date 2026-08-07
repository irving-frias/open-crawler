use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::crawler::fetcher::HtmlFetcher;
use crate::error::AppError;
use crate::features::with_repo;
use crate::seo::SeoAuditResult;
use crate::AppState;

/// Returns the stored SEO audit for a page. Errors if the page was crawled
/// before the SEO audit feature existed (use `run_seo_audit` to backfill).
#[tauri::command]
pub async fn get_seo_audit(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    url: String,
) -> Result<SeoAuditResult, AppError> {
    let url_for_lookup = url.clone();
    let page_id = with_repo(&state, move |repo| {
        repo.find_page_id(&project_id, &url_for_lookup)
    })
    .await?
    .ok_or_else(|| AppError::Crawl(format!("Page not found: {url}")))?;

    let stored = with_repo(&state, move |repo| repo.get_seo_audit_json(&page_id)).await?;
    let json = stored
        .and_then(|stored| stored.json)
        .ok_or_else(|| AppError::Crawl("No stored SEO audit for this page yet".to_string()))?;

    serde_json::from_str::<SeoAuditResult>(&json)
        .map_err(|e| AppError::Crawl(format!("Stored SEO audit is invalid: {e}")))
}

/// Re-fetches a single page, runs the SEO audit on the fresh response and
/// persists the new score + JSON. Used to backfill pages crawled before the
/// SEO audit feature, or to re-audit after making changes.
#[tauri::command]
pub async fn run_seo_audit(
    state: State<'_, Arc<RwLock<AppState>>>,
    page_id: String,
) -> Result<SeoAuditResult, AppError> {
    let pid = page_id.clone();
    let original = with_repo(&state, move |repo| repo.get_page_detail(&pid))
        .await?
        .page;

    let url = url::Url::parse(&original.url)?;
    let project_id = original.project_id.clone();

    let project_id_cfg = project_id.clone();
    let config = with_repo(&state, move |repo| {
        repo.get_latest_session_config(&project_id_cfg)
    })
    .await?;
    let cookies = config.as_ref().map(|c| c.cookies.clone()).unwrap_or_default();
    let site_auth = config.and_then(|c| c.site_auth);

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

    let parser = crate::crawler::parser::SeoParser::new();
    let (seo_data, _) = parser.parse(&response.html, &url);

    let audit = crate::seo::audit::audit_page(
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

    let json = serde_json::to_string(&audit).ok();
    let score = audit.score;
    let pid = page_id.clone();
    with_repo(&state, move |repo| {
        repo.update_seo_audit(&pid, score, json.as_deref())
    })
    .await?;

    Ok(audit)
}

/// Project-wide SEO overview aggregated from stored audits.
#[tauri::command]
pub async fn get_seo_overview(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<crate::models::SeoOverview, AppError> {
    with_repo(&state, move |repo| repo.get_seo_overview(&project_id)).await
}
