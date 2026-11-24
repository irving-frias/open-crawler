use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::crawler::extraction::{extract, extract_one, validate, CustomExtraction};
use crate::crawler::fetcher::{HtmlFetcher, HttpFetcher};
use crate::error::AppError;
use crate::features::with_repo;
use crate::AppState;

/// The project's custom extraction rules, in display order.
#[tauri::command]
pub async fn get_custom_extractions(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Vec<CustomExtraction>, AppError> {
    with_repo(&state, move |repo| repo.get_custom_extractions(&project_id)).await
}

/// Replaces the project's rules with the given list (validated first, so a
/// bad selector never wipes the stored rules). Returns the stored rules,
/// with fresh ids for new ones and `position` following the list order.
#[tauri::command]
pub async fn save_custom_extractions(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    rules: Vec<CustomExtraction>,
) -> Result<Vec<CustomExtraction>, AppError> {
    with_repo(&state, move |repo| {
        repo.save_custom_extractions(&project_id, &rules)
    })
    .await
}

/// Applies one rule to a URL fetched on the spot and returns the value it
/// extracts (or `null` when nothing matches). Lets the user test a selector
/// before saving it. The rule is validated before any fetch happens.
#[tauri::command]
pub async fn test_custom_extraction(
    http: State<'_, reqwest::Client>,
    rule: CustomExtraction,
    url: String,
) -> Result<serde_json::Value, AppError> {
    validate(&rule).map_err(AppError::InvalidConfig)?;
    let parsed = url::Url::parse(&url)?;
    let fetcher = HttpFetcher::new(
        Some(http.inner().clone()),
        crate::models::crawl_config::IMPLICIT_USER_AGENT,
        30_000,
        Vec::new(),
        Vec::new(),
        None,
        None,
    )?;
    let response = fetcher.fetch(&parsed).await?;
    extract_one(&response.html, &rule).map_err(AppError::InvalidConfig)
}

/// Recomputes the extraction values for every page of the project that still
/// has its HTML stored, using the project's current rules, and persists them
/// in one transaction. Pages without stored HTML are skipped (the SEO
/// re-audit re-fetches those). Returns the number of pages updated.
#[tauri::command]
pub async fn recompute_custom_extractions(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<u32, AppError> {
    let pid = project_id.clone();
    let rules = with_repo(&state, move |repo| repo.get_custom_extractions(&pid)).await?;
    if rules.is_empty() {
        return Ok(0);
    }
    let pid = project_id.clone();
    let pages = with_repo(&state, move |repo| repo.list_pages_for_seo_audit(&pid)).await?;

    // CPU-bound parsing runs off the async runtime.
    let rows = tokio::task::spawn_blocking(move || {
        pages
            .into_iter()
            .filter_map(|stored| {
                stored.html_body.filter(|h| !h.is_empty()).map(|html| {
                    let values = extract(&html, &rules);
                    let json = serde_json::to_string(&values).ok();
                    (stored.page_id, json)
                })
            })
            .collect::<Vec<(String, Option<String>)>>()
    })
    .await
    .map_err(|e| AppError::Crawl(e.to_string()))?;

    let updated = rows.len() as u32;
    with_repo(&state, move |repo| {
        repo.update_custom_extractions_batch(&rows)
    })
    .await?;
    Ok(updated)
}
