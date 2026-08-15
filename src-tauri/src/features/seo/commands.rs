use std::sync::Arc;

use futures::StreamExt;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::crawler::fetcher::HtmlFetcher;
use crate::crawler::fetcher::HttpFetcher;
use crate::crawler::parser::SeoParser;
use crate::error::AppError;
use crate::features::with_repo;
use crate::seo::audit::audit_page_with_site;
use crate::seo::AuditContext;
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
    http: State<'_, reqwest::Client>,
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
    let cookies = config
        .as_ref()
        .map(|c| c.cookies.clone())
        .unwrap_or_default();
    let site_auth = config.and_then(|c| c.site_auth);

    let user_agent = crate::models::crawl_config::IMPLICIT_USER_AGENT;
    let fetcher = crate::crawler::fetcher::HttpFetcher::new(
        Some(http.inner().clone()),
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

    let audit = crate::seo::audit::audit_page_with_site(
        &seo_data,
        &response.html,
        &crate::seo::AuditContext {
            url: response.url.to_string(),
            status_code: response.status,
            size_bytes: response.size_bytes,
            load_time_ms: response.load_time_ms,
            pagespeed_score: None,
            response_headers: response.headers.clone(),
            site_resources: None,
        },
        http.inner(),
    )
    .await;

    let json = serde_json::to_string(&audit).ok();
    let score = audit.score;
    let pid = page_id.clone();
    let pid_project = original.project_id.clone();
    let response_headers_json = if response.headers.is_empty() {
        None
    } else {
        serde_json::to_string(&response.headers).ok()
    };
    with_repo(&state, move |repo| {
        repo.update_seo_audit(
            &pid_project,
            &pid,
            score,
            json.as_deref(),
            response_headers_json.as_deref(),
        )
    })
    .await?;

    Ok(audit)
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FixSuggestion {
    pub suggestion: String,
    pub corrected_html: Option<String>,
}

/// Asks a configured OpenAI-compatible chat endpoint for a concrete fix for a
/// failing SEO check. The response is a suggestion only — nothing is written
/// back to the page. Requires the `ai_enabled`, `ai_api_key`, `ai_base_url`
/// and `ai_model` settings.
#[tauri::command]
pub async fn suggest_fix(
    state: State<'_, Arc<RwLock<AppState>>>,
    http: State<'_, reqwest::Client>,
    check_id: String,
    check_message: String,
    check_guidance: String,
    element_snippet: Option<String>,
    language: String,
) -> Result<FixSuggestion, AppError> {
    let settings = with_repo(&state, |repo| repo.get_all_settings()).await?;

    let enabled = settings
        .get("ai_enabled")
        .map(|v| v == "true")
        .unwrap_or(false);
    if !enabled {
        return Err(AppError::Crawl(
            "AI suggestions are not enabled. Turn them on in Settings → AI Assistant.".to_string(),
        ));
    }
    let api_key = with_repo(&state, |repo| crate::secrets::get(repo, "ai_api_key"))
        .await?
        .unwrap_or_default();
    if api_key.is_empty() {
        return Err(AppError::Crawl(
            "No AI API key configured in Settings → AI Assistant.".to_string(),
        ));
    }
    let base_url = settings
        .get("ai_base_url")
        .cloned()
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
    let model = settings
        .get("ai_model")
        .cloned()
        .unwrap_or_else(|| "gpt-4o-mini".to_string());

    let lang_hint = if language.starts_with("es") {
        "Answer in Spanish."
    } else {
        "Answer in English."
    };

    let prompt = format!(
        "You are an SEO expert helping fix a failing SEO check on a web page.\n\
         Check id: {check_id}\n\
         Message: {check_message}\n\
         Guidance: {check_guidance}\n\
         Offending element snippet:\n```\n{}\n```\n\n\
         Explain concisely why this fails and provide the exact corrected HTML \
         that fixes the problem (a before/after diff style answer). {lang_hint}",
        element_snippet.unwrap_or_else(|| "<no element context available>".to_string())
    );

    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let response = http
        .post(&endpoint)
        .timeout(std::time::Duration::from_secs(60))
        .bearer_auth(&api_key)
        .json(&serde_json::json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.2,
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Crawl(format!(
            "AI request failed ({status}): {}",
            body.chars().take(500).collect::<String>()
        )));
    }

    let json: serde_json::Value = response.json().await?;
    let text = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_string();
    if text.is_empty() {
        return Err(AppError::Crawl("AI returned an empty response".to_string()));
    }

    Ok(FixSuggestion {
        corrected_html: extract_code_block(&text),
        suggestion: text,
    })
}

/// Pulls the first fenced code block (```lang ... ```) out of a model answer,
/// so the UI can render the corrected markup separately from the explanation.
fn extract_code_block(text: &str) -> Option<String> {
    let start = text.find("```")? + 3;
    let content_start = match text[start..].find('\n') {
        Some(i) => start + i + 1,
        None => start,
    };
    let rest = &text[content_start..];
    let end = rest.find("```")?;
    let block = rest[..end].trim().to_string();
    if block.is_empty() {
        None
    } else {
        Some(block)
    }
}

/// Project-wide SEO overview aggregated from stored audits.
#[tauri::command]
pub async fn get_seo_overview(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<crate::models::SeoOverview, AppError> {
    with_repo(&state, move |repo| repo.get_seo_overview(&project_id)).await
}

#[derive(serde::Serialize, Clone)]
pub struct SeoAuditProgress {
    pub project_id: String,
    pub processed: u32,
    pub total: u32,
    pub errors: u32,
    pub percent: f32,
}

/// Returns the progress of a running project-wide SEO re-audit, if any.
/// Each project owns its audit independently so parallel project windows can
/// poll without cross-talk.
#[tauri::command]
pub async fn get_seo_audit_status(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<Option<SeoAuditProgress>, AppError> {
    let state = state.inner().clone();
    let state_read = state.read().await;
    let audits = state_read.seo_audits.read().await;
    Ok(audits.get(&project_id).map(|s| s.progress.clone()))
}

/// Cancels the running SEO re-audit for a project. The audit loop observes the
/// token and stops at the next page boundary.
#[tauri::command]
pub async fn stop_seo_audit(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<(), AppError> {
    let state = state.inner().clone();
    let state_write = state.write().await;
    let mut audits = state_write.seo_audits.write().await;
    if let Some(audit) = audits.remove(&project_id) {
        drop(audits);
        drop(state_write);
        audit.cancellation.cancel();
        info!(
            "SEO audit cancellation signal sent for project: {}",
            project_id
        );
        let _ = app.emit(
            "seo-audit-complete",
            serde_json::json!({ "project_id": &project_id, "cancelled": true }),
        );
    } else {
        warn!("No SEO audit running for project: {}", project_id);
    }
    Ok(())
}

/// Re-fetches every page in the project, runs the SEO audit on each fresh
/// response and persists the updated score + JSON. Emits `seo-audit-progress`
/// events so the UI can show progress while the loop runs. Registered in
/// `AppState.seo_audits` so each project can be polled/cancelled independently.
#[tauri::command]
pub async fn run_seo_audit_all(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    http: State<'_, reqwest::Client>,
    project_id: String,
) -> Result<SeoAuditProgress, AppError> {
    // Guard: only one audit per project at a time.
    {
        let state_read = state.read().await;
        let audits = state_read.seo_audits.read().await;
        if audits.contains_key(&project_id) {
            return Err(AppError::Crawl(format!(
                "SEO audit already running for project: {}",
                project_id
            )));
        }
    }

    let pid = project_id.clone();
    let pages = with_repo(&state, move |repo| repo.list_page_id_urls(&pid)).await?;
    let total = pages.len() as u32;

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

    let user_agent = crate::models::crawl_config::IMPLICIT_USER_AGENT;
    let fetcher = HttpFetcher::new(
        Some(http.inner().clone()),
        user_agent,
        30000,
        Vec::new(),
        cookies,
        site_auth,
        None,
    )?;
    let parser = SeoParser::new();

    let token = tokio_util::sync::CancellationToken::new();

    let initial = SeoAuditProgress {
        project_id: project_id.clone(),
        processed: 0,
        total,
        errors: 0,
        percent: if total == 0 { 100.0 } else { 0.0 },
    };
    {
        let state_write = state.write().await;
        let mut audits = state_write.seo_audits.write().await;
        audits.insert(
            project_id.clone(),
            crate::SeoAuditState {
                cancellation: token.clone(),
                progress: initial,
            },
        );
    }

    let mut processed: u32 = 0;
    let mut errors: u32 = 0;

    // Concurrently fetch + parse pages (the slow network part) with a bounded
    // buffer; DB writes stay serialized through `with_repo` in the consumer.
    const SEO_AUDIT_CONCURRENCY: usize = 4;
    let fetcher = Arc::new(fetcher);
    let site_client = http.inner().clone();
    let work = pages.into_iter().map(|(page_id, url)| {
        let fetcher = fetcher.clone();
        let parser = parser.clone();
        let site_client = site_client.clone();
        async move {
            let parsed = match url::Url::parse(&url) {
                Ok(u) => u,
                Err(_) => {
                    return Err(AppError::Crawl(format!("Invalid URL: {url}")));
                }
            };
            let response = fetcher.fetch(&parsed).await?;
            let (seo_data, _) = parser.parse(&response.html, &parsed);
            let audit = audit_page_with_site(
                &seo_data,
                &response.html,
                &AuditContext {
                    url: response.url.to_string(),
                    status_code: response.status,
                    size_bytes: response.size_bytes,
                    load_time_ms: response.load_time_ms,
                    pagespeed_score: None,
                    response_headers: response.headers.clone(),
                    site_resources: None,
                },
                &site_client,
            )
            .await;
            let json = serde_json::to_string(&audit).ok();
            let headers_json = if response.headers.is_empty() {
                None
            } else {
                serde_json::to_string(&response.headers).ok()
            };
            Ok((page_id, audit.score, json, headers_json))
        }
    });

    let mut stream = futures::stream::iter(work).buffer_unordered(SEO_AUDIT_CONCURRENCY);
    while let Some(outcome) = stream.next().await {
        if token.is_cancelled() {
            info!("SEO audit cancelled for project: {}", project_id);
            break;
        }

        match outcome {
            Ok((page_id, score, json, headers_json)) => {
                let pid = page_id;
                let pid_project = project_id.clone();
                if with_repo(&state, move |repo| {
                    repo.update_seo_audit(
                        &pid_project,
                        &pid,
                        score,
                        json.as_deref(),
                        headers_json.as_deref(),
                    )
                })
                .await
                .is_err()
                {
                    errors += 1;
                }
            }
            Err(_) => errors += 1,
        }
        processed += 1;
        emit_seo_progress(&app, &state, &project_id, processed, total, errors).await;
    }

    // Deregister the running audit (also when cancelled).
    {
        let state_write = state.write().await;
        let mut audits = state_write.seo_audits.write().await;
        audits.remove(&project_id);
    }

    let percent = if total == 0 {
        100.0
    } else {
        (processed as f32 / total as f32) * 100.0
    };
    let final_progress = SeoAuditProgress {
        project_id: project_id.clone(),
        processed,
        total,
        errors,
        percent,
    };

    let _ = app.emit(
        "seo-audit-complete",
        serde_json::json!({
            "project_id": &project_id,
            "cancelled": token.is_cancelled(),
        }),
    );

    Ok(final_progress)
}

async fn emit_seo_progress(
    app: &AppHandle,
    state: &State<'_, Arc<RwLock<AppState>>>,
    project_id: &str,
    processed: u32,
    total: u32,
    errors: u32,
) {
    let percent = if total == 0 {
        100.0
    } else {
        (processed as f32 / total as f32) * 100.0
    };
    let progress = SeoAuditProgress {
        project_id: project_id.to_string(),
        processed,
        total,
        errors,
        percent,
    };
    // Keep the registry snapshot fresh so `get_seo_audit_status` returns live data.
    let state = state.inner().clone();
    let pid = project_id.to_string();
    {
        let state_read = state.read().await;
        let mut audits = state_read.seo_audits.write().await;
        if let Some(audit) = audits.get_mut(&pid) {
            audit.progress = progress.clone();
        }
    }
    let _ = app.emit("seo-audit-progress", progress);
}
