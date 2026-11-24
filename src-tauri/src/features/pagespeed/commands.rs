use std::sync::Arc;

use futures::StreamExt;
use serde::Serialize;
use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::{with_repo, with_repo_arc};
use crate::pagespeed::{PageSpeedData, PAGESPEED_CACHE_TTL_HOURS};
use crate::AppState;

/// Max PageSpeed API calls in flight in the bulk command. The PSI API is
/// quota-limited and slow (a Lighthouse run takes seconds), so the queue is
/// deliberately small: 4 parallel calls keep a big audit moving without
/// tripping the API's rate limit. Retries on 429 back off inside
/// `fetch_pagespeed` itself.
pub const PAGESPEED_BULK_CONCURRENCY: usize = 4;

/// One URL's outcome in a bulk PageSpeed audit. Failures are reported per
/// URL instead of failing the whole run, so one bad URL never discards the
/// other results.
#[derive(Debug, Clone, Serialize)]
pub struct PageSpeedBulkResult {
    pub url: String,
    pub data: Option<PageSpeedData>,
    pub error: Option<String>,
    pub cached: bool,
}

/// Reads the cached result for `page_id` when it is still within the TTL.
/// `None` means "no cache or stale": the caller must hit the API.
async fn cached_pagespeed(
    state: &Arc<RwLock<AppState>>,
    page_id: &str,
) -> Result<Option<PageSpeedData>, AppError> {
    let pid = page_id.to_string();
    with_repo_arc(state, move |repo| {
        repo.get_cached_pagespeed(&pid, PAGESPEED_CACHE_TTL_HOURS)
    })
    .await
}

/// Persists a fresh result (and stamps `pagespeed_fetched_at`, which is
/// what makes the next audit a cache hit). Best-effort: a write failure
/// must not discard data that was already fetched.
async fn save_pagespeed(state: &Arc<RwLock<AppState>>, page_id: &str, data: &PageSpeedData) {
    let pid = page_id.to_string();
    let score = data.score.map(|s| s as f64);
    let json = serde_json::to_string(data).ok();
    let _ = with_repo_arc(state, move |repo| {
        repo.update_pagespeed(&pid, score, json.as_deref())
    })
    .await;
}

/// Audits `urls` for `project_id` through `fetch`, reusing the per-URL cache
/// whenever it is fresh and running at most [`PAGESPEED_BULK_CONCURRENCY`]
/// fetches in parallel. Results come back in input order.
///
/// `fetch` is any async function `url -> PageSpeedData` (the Tauri command
/// passes the real API call; tests pass a counting fake). A fetch failure
/// lands in that URL's `error` field; the other URLs still complete.
pub(crate) async fn audit_pagespeed_urls<F, Fut>(
    state: &Arc<RwLock<AppState>>,
    fetch: F,
    project_id: &str,
    urls: &[String],
) -> Result<Vec<PageSpeedBulkResult>, AppError>
where
    F: Fn(String) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<PageSpeedData, AppError>> + Send,
{
    let fetch = Arc::new(fetch);
    // Owned URLs (like seo/commands.rs): the per-URL futures must not borrow
    // the input slice, or `buffer_unordered` cannot hold them.
    let work = urls.iter().cloned().enumerate().map(|(idx, url)| {
        let fetch = Arc::clone(&fetch);
        let state = Arc::clone(state);
        let project_id = project_id.to_string();
        async move {
            let pid_lookup = project_id.clone();
            let url_lookup = url.clone();
            let page_id = with_repo_arc(&state, move |repo| {
                repo.find_page_id(&pid_lookup, &url_lookup)
            })
            .await
            .ok()
            .flatten();

            if let Some(pid) = page_id.clone() {
                if let Ok(Some(cached)) = cached_pagespeed(&state, &pid).await {
                    return (
                        idx,
                        PageSpeedBulkResult {
                            url,
                            data: Some(cached),
                            error: None,
                            cached: true,
                        },
                    );
                }
            }

            let result = match fetch(url.clone()).await {
                Ok(data) => {
                    if let Some(pid) = page_id {
                        save_pagespeed(&state, &pid, &data).await;
                    }
                    PageSpeedBulkResult {
                        url,
                        data: Some(data),
                        error: None,
                        cached: false,
                    }
                }
                Err(err) => PageSpeedBulkResult {
                    url,
                    data: None,
                    error: Some(err.to_string()),
                    cached: false,
                },
            };
            (idx, result)
        }
    });

    let mut stream = futures::stream::iter(work).buffer_unordered(PAGESPEED_BULK_CONCURRENCY);
    let mut indexed = Vec::with_capacity(urls.len());
    while let Some(item) = stream.next().await {
        indexed.push(item);
    }
    indexed.sort_by_key(|(idx, _)| *idx);
    Ok(indexed.into_iter().map(|(_, result)| result).collect())
}

#[tauri::command]
pub async fn get_pagespeed_score(
    state: State<'_, Arc<RwLock<AppState>>>,
    http: State<'_, reqwest::Client>,
    project_id: String,
    url: String,
) -> Result<PageSpeedData, AppError> {
    let url_clone = url.clone();
    let page_id = with_repo(&state, move |repo| {
        repo.find_page_id(&project_id, &url_clone)
    })
    .await?;

    // Same-day cache: a fresh stored result is returned without calling the
    // API at all. Stale or unparsable rows fall through to a re-fetch.
    if let Some(pid) = page_id.clone() {
        if let Some(cached) = cached_pagespeed(state.inner(), &pid).await? {
            return Ok(cached);
        }
    }

    let api_key = with_repo(&state, |repo| {
        crate::secrets::get(repo, "pagespeed_api_key")
    })
    .await?;

    let data = crate::pagespeed::fetch_pagespeed(&http, &url, api_key.as_deref()).await?;

    if let Some(pid) = page_id {
        save_pagespeed(state.inner(), &pid, &data).await;
    }

    Ok(data)
}

/// Audits several URLs with a bounded-concurrency queue, reusing the cache
/// for URLs audited within the last [`PAGESPEED_CACHE_TTL_HOURS`] hours.
/// Per-URL failures are reported in the matching result instead of failing
/// the whole batch. (Frontend wiring is intentionally out of scope for this
/// phase; the single-URL command above is the one the UI calls today.)
#[tauri::command]
pub async fn get_pagespeed_scores_bulk(
    state: State<'_, Arc<RwLock<AppState>>>,
    http: State<'_, reqwest::Client>,
    project_id: String,
    urls: Vec<String>,
) -> Result<Vec<PageSpeedBulkResult>, AppError> {
    let api_key = with_repo(&state, |repo| {
        crate::secrets::get(repo, "pagespeed_api_key")
    })
    .await?;
    let client = http.inner().clone();
    let fetch = move |url: String| {
        let client = client.clone();
        let api_key = api_key.clone();
        async move { crate::pagespeed::fetch_pagespeed(&client, &url, api_key.as_deref()).await }
    };
    audit_pagespeed_urls(state.inner(), fetch, &project_id, &urls).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;
    use rusqlite::Connection;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn test_state() -> Arc<RwLock<AppState>> {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn.execute_batch(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('p1', 'P1', datetime('now'), datetime('now'));
             INSERT INTO crawl_config (id, project_id, seed_urls, created_at) VALUES ('cfg', 'p1', '[]', datetime('now'));",
        )
        .unwrap();
        let cache = Arc::new(std::sync::Mutex::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(8).unwrap(),
        )));
        Arc::new(RwLock::new(AppState {
            db: std::sync::Mutex::new(conn),
            crawls: Arc::new(RwLock::new(std::collections::HashMap::new())),
            seo_audits: Arc::new(RwLock::new(std::collections::HashMap::new())),
            results_cache: cache,
            transfer_server: std::sync::Mutex::new(None),
        }))
    }

    async fn seed_page(state: &Arc<RwLock<AppState>>, id: &str, url: &str) {
        let guard = state.read().await;
        let db = guard.db.lock().unwrap();
        db.execute(
            "INSERT INTO crawled_pages (id, config_id, project_id, url, crawl_timestamp)
             VALUES (?1, 'cfg', 'p1', ?2, datetime('now'))",
            rusqlite::params![id, url],
        )
        .unwrap();
    }

    async fn set_fetched_at(state: &Arc<RwLock<AppState>>, id: &str, expr: &str) {
        let guard = state.read().await;
        let db = guard.db.lock().unwrap();
        db.execute(
            &format!("UPDATE crawled_pages SET pagespeed_fetched_at = {expr} WHERE id = ?1"),
            rusqlite::params![id],
        )
        .unwrap();
    }

    async fn set_cached_json(state: &Arc<RwLock<AppState>>, id: &str, json: &str) {
        let guard = state.read().await;
        let db = guard.db.lock().unwrap();
        db.execute(
            "UPDATE crawled_pages SET pagespeed_json = ?1 WHERE id = ?2",
            rusqlite::params![json, id],
        )
        .unwrap();
    }

    fn canned(score: u32) -> PageSpeedData {
        PageSpeedData {
            score: Some(score),
            fcp: Some("1.2 s".to_string()),
            ..Default::default()
        }
    }

    fn counting_fetch(
        calls: &Arc<AtomicUsize>,
        score: u32,
    ) -> impl Fn(
        String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PageSpeedData, AppError>> + Send>,
    > + Send
           + Sync {
        let calls = calls.clone();
        move |_url: String| {
            let calls = calls.clone();
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(canned(score))
            })
        }
    }

    #[tokio::test]
    async fn bulk_second_run_uses_cache_without_calling_api() {
        let state = test_state();
        seed_page(&state, "pg1", "https://x.com/a").await;
        seed_page(&state, "pg2", "https://x.com/b").await;
        let urls = vec!["https://x.com/a".to_string(), "https://x.com/b".to_string()];

        let calls = Arc::new(AtomicUsize::new(0));
        let first = audit_pagespeed_urls(&state, counting_fetch(&calls, 90), "p1", &urls)
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2, "first run fetches both");
        assert!(first.iter().all(|r| !r.cached && r.data.is_some()));
        assert_eq!(first[0].data.as_ref().unwrap().score, Some(90));

        // Second audit of the same URLs: zero API calls, all cached.
        let second = audit_pagespeed_urls(&state, counting_fetch(&calls, 90), "p1", &urls)
            .await
            .unwrap();
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "second run must not call the API"
        );
        assert!(second.iter().all(|r| r.cached && r.data.is_some()));
        assert_eq!(second[1].data.as_ref().unwrap().score, Some(90));
        assert_eq!(second[0].url, urls[0], "results keep input order");
    }

    #[tokio::test]
    async fn bulk_stale_cache_refetches() {
        let state = test_state();
        seed_page(&state, "pg1", "https://x.com/a").await;
        let urls = vec!["https://x.com/a".to_string()];

        let calls = Arc::new(AtomicUsize::new(0));
        audit_pagespeed_urls(&state, counting_fetch(&calls, 90), "p1", &urls)
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        // Age the cached row past the 24 h TTL: the next audit re-fetches.
        set_fetched_at(&state, "pg1", "datetime('now', '-25 hours')").await;
        let rerun = audit_pagespeed_urls(&state, counting_fetch(&calls, 90), "p1", &urls)
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2, "stale cache must re-fetch");
        assert!(!rerun[0].cached);
    }

    #[tokio::test]
    async fn bulk_unparsable_cache_refetches() {
        let state = test_state();
        seed_page(&state, "pg1", "https://x.com/a").await;
        set_cached_json(&state, "pg1", "not json at all").await;
        set_fetched_at(&state, "pg1", "datetime('now')").await;

        let calls = Arc::new(AtomicUsize::new(0));
        let results = audit_pagespeed_urls(
            &state,
            counting_fetch(&calls, 80),
            "p1",
            &["https://x.com/a".to_string()],
        )
        .await
        .unwrap();
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "bad JSON is not a cache hit"
        );
        assert_eq!(results[0].data.as_ref().unwrap().score, Some(80));
    }

    #[tokio::test]
    async fn bulk_partial_failure_keeps_other_results() {
        let state = test_state();
        seed_page(&state, "pg1", "https://x.com/a").await;
        seed_page(&state, "pg2", "https://x.com/bad").await;
        let urls = vec![
            "https://x.com/a".to_string(),
            "https://x.com/bad".to_string(),
        ];
        let fetch = |url: String| async move {
            if url.ends_with("/bad") {
                Err(AppError::Pagespeed("boom".to_string()))
            } else {
                Ok(canned(70))
            }
        };
        let results = audit_pagespeed_urls(&state, fetch, "p1", &urls)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].data.is_some() && results[0].error.is_none());
        assert!(results[1].data.is_none());
        assert!(
            results[1].error.as_deref().unwrap().contains("boom"),
            "error was: {:?}",
            results[1].error
        );
    }

    #[tokio::test]
    async fn bulk_url_without_page_still_fetches_but_does_not_cache() {
        let state = test_state();
        let calls = Arc::new(AtomicUsize::new(0));
        let urls = vec!["https://x.com/unknown".to_string()];
        let first = audit_pagespeed_urls(&state, counting_fetch(&calls, 60), "p1", &urls)
            .await
            .unwrap();
        assert_eq!(first[0].data.as_ref().unwrap().score, Some(60));
        // No crawled_pages row, so nothing to cache into: a second run
        // fetches again instead of crashing or returning a phantom hit.
        audit_pagespeed_urls(&state, counting_fetch(&calls, 60), "p1", &urls)
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
