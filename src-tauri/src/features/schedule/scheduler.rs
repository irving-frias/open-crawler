use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::db::CrawlRepo;
use crate::error::AppError;
use crate::features::crawl::commands::start_crawl_internal;
use crate::models::CrawlConfig;
use crate::AppState;

use crate::db::repos::schedule::{compute_next_run, now_rfc3339_secs};

const TICK_INTERVAL_SECS: u64 = 60;

/// Background task that polls for due `scheduled_jobs` once a minute and
/// launches their crawls. Runs for the lifetime of the app process.
pub async fn run_scheduler(app: AppHandle, state: Arc<RwLock<AppState>>) {
    info!("Scheduler started (tick: {}s)", TICK_INTERVAL_SECS);

    let mut tick = tokio::time::interval(Duration::from_secs(TICK_INTERVAL_SECS));
    // The first tick fires immediately; skip it so a freshly created job is not
    // evaluated before the app settles.
    tick.tick().await;

    loop {
        tick.tick().await;
        if let Err(e) = run_due_jobs(&app, &state).await {
            error!("Scheduler tick failed: {}", e);
        }
    }
}

async fn run_due_jobs(app: &AppHandle, state: &Arc<RwLock<AppState>>) -> Result<(), AppError> {
    let now = now_rfc3339_secs();

    let jobs = {
        let state = state.clone();
        let now = now.clone();
        tokio::task::spawn_blocking(move || {
            let state_read = state.blocking_read();
            let db = state_read
                .db
                .lock()
                .map_err(|e| AppError::Crawl(e.to_string()))?;
            let repo = CrawlRepo::new(&db, None);
            repo.get_due_scheduled_jobs(&now)
        })
        .await
        .map_err(|e| AppError::Crawl(format!("Scheduler DB worker panicked: {e}")))?
    }?;

    for job in jobs {
        // Never run two crawls of the same project at once.
        {
            let state_read = state.read().await;
            let crawls = state_read.crawls.read().await;
            if crawls.contains_key(&job.project_id) {
                warn!(
                    "Scheduled job {} skipped: crawl already running for project {}",
                    job.id, job.project_id
                );
                advance_next_run(state, &job).await;
                continue;
            }
        }

        info!(
            "Scheduled job {} firing for project {}",
            job.id, job.project_id
        );

        advance_next_run(state, &job).await;

        let config: CrawlConfig = match serde_json::from_str(&job.config_json) {
            Ok(c) => c,
            Err(e) => {
                error!("Scheduled job {} has invalid config_json: {}", job.id, e);
                continue;
            }
        };

        let _ = app.emit(
            "scheduled-job-ran",
            serde_json::json!({
                "job_id": job.id,
                "project_id": job.project_id,
            }),
        );

        if let Err(e) =
            start_crawl_internal(app.clone(), state.clone(), config, &job.project_id).await
        {
            error!("Failed to run scheduled job {}: {}", job.id, e);
        }
    }

    Ok(())
}

/// Records `last_run` and rolls `next_run` forward to the following cron
/// occurrence so the same due job does not fire again on the next tick.
async fn advance_next_run(state: &Arc<RwLock<AppState>>, job: &crate::models::ScheduledJob) {
    let now = now_rfc3339_secs();
    let job_id = job.id.clone();
    let cron_expression = job.cron_expression.clone();
    let state = state.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let state_read = state.blocking_read();
        let db = state_read
            .db
            .lock()
            .map_err(|e| AppError::Crawl(e.to_string()))?;
        let repo = CrawlRepo::new(&db, None);
        repo.set_job_last_run(&job_id, &now)?;
        if let Some(next) = compute_next_run(&cron_expression, &now)? {
            repo.set_job_next_run(&job_id, &next)?;
        }
        Ok::<(), AppError>(())
    })
    .await;
}
