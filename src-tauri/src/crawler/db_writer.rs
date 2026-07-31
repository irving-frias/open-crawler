use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::Emitter;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

use crate::db::CrawlRepo;
use crate::models::{CrawlResult, PageLink};
use crate::AppState;

const DEFAULT_BATCH_SIZE: usize = 50;
const DEFAULT_FLUSH_INTERVAL_MS: u64 = 2000;

pub enum CrawlResultMsg {
    Page(Box<CrawlResult>),
    Links(Vec<PageLink>),
    Error {
        url: String,
        config_id: String,
        project_id: String,
        error_type: String,
        message: String,
    },
    Flush,
    Done,
}

pub struct DbWriter {
    rx: mpsc::Receiver<CrawlResultMsg>,
    batch_results: Vec<CrawlResult>,
    batch_links: Vec<PageLink>,
    batch_size: usize,
    flush_interval: Duration,
    last_flush: Instant,
    app_handle: Arc<tauri::AppHandle>,
    state: Arc<RwLock<AppState>>,
    project_id: String,
    total_crawled: u32,
    total_errors: u32,
}

impl DbWriter {
    pub fn new(
        rx: mpsc::Receiver<CrawlResultMsg>,
        app_handle: Arc<tauri::AppHandle>,
        state: Arc<RwLock<AppState>>,
        project_id: String,
    ) -> Self {
        Self {
            rx,
            batch_results: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            batch_links: Vec::with_capacity(DEFAULT_BATCH_SIZE * 5),
            batch_size: DEFAULT_BATCH_SIZE,
            flush_interval: Duration::from_millis(DEFAULT_FLUSH_INTERVAL_MS),
            last_flush: Instant::now(),
            app_handle,
            state,
            project_id,
            total_crawled: 0,
            total_errors: 0,
        }
    }

    pub async fn run(&mut self) {
        info!(
            "DbWriter started for project: {} (batch_size={}, flush_interval={}ms)",
            self.project_id,
            self.batch_size,
            self.flush_interval.as_millis()
        );

        loop {
            let should_flush = self.last_flush.elapsed() >= self.flush_interval
                || self.batch_results.len() >= self.batch_size;

            if should_flush && (!self.batch_results.is_empty() || !self.batch_links.is_empty()) {
                self.flush().await;
            }

            tokio::select! {
                msg = self.rx.recv() => {
                    match msg {
                        Some(CrawlResultMsg::Page(result)) => {
                            self.total_crawled += 1;
                            self.batch_results.push(*result);
                        }
                        Some(CrawlResultMsg::Links(links)) => {
                            self.batch_links.extend(links);
                        }
                        Some(CrawlResultMsg::Error { url, config_id, project_id, error_type, message }) => {
                            self.total_errors += 1;
                            self.save_error(&url, &config_id, &project_id, &error_type, &message).await;
                        }
                        Some(CrawlResultMsg::Flush) => {
                            if !self.batch_results.is_empty() || !self.batch_links.is_empty() {
                                self.flush().await;
                            }
                        }
                        Some(CrawlResultMsg::Done) => {
                            if !self.batch_results.is_empty() || !self.batch_links.is_empty() {
                                self.flush().await;
                            }
                            info!(
                                "DbWriter finished for project: {} ({} pages, {} errors)",
                                self.project_id, self.total_crawled, self.total_errors
                            );
                            break;
                        }
                        None => {
                            if !self.batch_results.is_empty() || !self.batch_links.is_empty() {
                                self.flush().await;
                            }
                            info!("DbWriter channel closed for project: {}", self.project_id);
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Check for flush interval
                }
            }
        }
    }

    async fn flush(&mut self) {
        let results = std::mem::take(&mut self.batch_results);
        let links = std::mem::take(&mut self.batch_links);
        let count = results.len();
        let link_count = links.len();

        if count == 0 && link_count == 0 {
            return;
        }

        let state_read = match self.state.try_read() {
            Ok(read) => read,
            Err(e) => {
                error!("Failed to acquire state read lock: {}", e);
                self.batch_results = results;
                self.batch_links = links;
                return;
            }
        };

        let db = match state_read.db.lock() {
            Ok(db) => db,
            Err(e) => {
                error!("Failed to acquire DB lock: {}", e);
                self.batch_results = results;
                self.batch_links = links;
                return;
            }
        };

        let repo = CrawlRepo::new(&db);

        // Batch insert pages
        if let Err(e) = repo.save_results_batch(&results) {
            error!("Failed to batch save {} results: {}", count, e);
        }

        // Batch insert links
        if !links.is_empty() {
            if let Err(e) = repo.save_links_batch(&links) {
                error!("Failed to batch save {} links: {}", link_count, e);
            }
        }

        self.last_flush = Instant::now();

        // Emit crawl-batch event for real-time frontend updates
        if count > 0 {
            let batch_items: Vec<serde_json::Value> = results
                .iter()
                .map(|r| {
                    serde_json::to_value(r).unwrap_or(serde_json::Value::Null)
                })
                .collect();

            if let Err(e) = self.app_handle.emit(
                "crawl-batch",
                serde_json::json!({
                    "project_id": &self.project_id,
                    "items": batch_items,
                    "total": self.total_crawled,
                    "errors": self.total_errors,
                }),
            ) {
                warn!("Failed to emit crawl-batch: {}", e);
            }
        }

        info!(
            "DbWriter flushed: {} pages, {} links for project: {}",
            count, link_count, self.project_id
        );
    }

    async fn save_error(
        &self,
        url: &str,
        config_id: &str,
        project_id: &str,
        error_type: &str,
        message: &str,
    ) {
        if let Ok(state_read) = self.state.try_read() {
            if let Ok(db) = state_read.db.lock() {
                let repo = CrawlRepo::new(&db);
                if let Err(e) = repo.save_error(url, config_id, project_id, error_type, message) {
                    error!("Failed to save error for {}: {}", url, e);
                }
            }
        }
    }
}

pub fn create_db_writer_channel(
    buffer_size: usize,
) -> (mpsc::Sender<CrawlResultMsg>, mpsc::Receiver<CrawlResultMsg>) {
    mpsc::channel(buffer_size)
}
