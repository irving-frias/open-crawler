pub mod analytics;
pub mod app;
pub mod crawl;
pub mod export;
pub mod pagespeed;
pub mod projects;
pub mod results;
pub mod schedule;
pub mod seo;
pub mod settings;
pub mod snapshots;

use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::db::CrawlRepo;
use crate::error::AppError;
use crate::AppState;

/// Runs a closure against a `CrawlRepo` while holding the app DB lock.
///
/// Every command that touches the database funnels through here, replacing the
/// repeated lock-and-repo boilerplate. The results cache is always attached;
/// it is only populated/consulted by `get_results`.
///
/// The DB work runs on a blocking thread (`spawn_blocking`) so that large
/// queries / batched writes never stall the async runtime that also carries the
/// crawl engine and IPC events. Callers pass `move` closures capturing owned
/// values; the closure must be `Send + 'static`.
pub(crate) async fn with_repo<T, F>(
    state: &State<'_, Arc<RwLock<AppState>>>,
    f: F,
) -> Result<T, AppError>
where
    F: FnOnce(&CrawlRepo<'_>) -> Result<T, AppError> + Send + 'static,
    T: Send + 'static,
{
    with_repo_arc(state.inner(), f).await
}

/// Same as [`with_repo`] but takes the `Arc<RwLock<AppState>>` directly, so
/// writers can run against an in-memory `AppState` in unit tests.
pub(crate) async fn with_repo_arc<T, F>(state: &Arc<RwLock<AppState>>, f: F) -> Result<T, AppError>
where
    F: FnOnce(&CrawlRepo<'_>) -> Result<T, AppError> + Send + 'static,
    T: Send + 'static,
{
    let state = state.clone();
    tokio::task::spawn_blocking(move || {
        let state_read = state.blocking_read();
        let db = state_read
            .db
            .lock()
            .map_err(|e| AppError::Crawl(e.to_string()))?;
        let repo = CrawlRepo::new(&db, Some(&state_read.results_cache));
        f(&repo)
    })
    .await
    .map_err(|e| AppError::Crawl(format!("DB worker panicked: {e}")))?
}
