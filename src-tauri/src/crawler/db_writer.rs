use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::Emitter;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

use crate::db::CrawlRepo;
use crate::models::{CrawlResult, PageLink, RedirectRecord};
use crate::AppState;

const DEFAULT_BATCH_SIZE: usize = 50;
const MAX_BATCH_SIZE: usize = 500;
const DEFAULT_FLUSH_INTERVAL_MS: u64 = 2000;

pub enum CrawlResultMsg {
    Page(Box<CrawlResult>),
    Links(Vec<PageLink>),
    Redirects(RedirectRecord),
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
    batch_redirects: Vec<RedirectRecord>,
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
            batch_redirects: Vec::with_capacity(DEFAULT_BATCH_SIZE),
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
                || self.batch_results.len() >= self.batch_size
                || self.batch_links.len() >= self.batch_size * 5;

            if should_flush
                && (!self.batch_results.is_empty()
                    || !self.batch_links.is_empty()
                    || !self.batch_redirects.is_empty())
            {
                self.flush().await;
            }

            tokio::select! {
                msg = self.rx.recv() => {
                    match msg {
                        Some(CrawlResultMsg::Page(result)) => {
                            self.total_crawled += 1;
                            self.batch_results.push(*result);
                            if self.batch_results.len() >= self.batch_size {
                                self.flush().await;
                            }
                        }
                        Some(CrawlResultMsg::Links(links)) => {
                            self.batch_links.extend(links);
                            if self.batch_links.len() >= self.batch_size * 5 {
                                self.flush().await;
                            }
                        }
                        Some(CrawlResultMsg::Redirects(record)) => {
                            self.batch_redirects.push(record);
                        }
                        Some(CrawlResultMsg::Error { url, config_id, project_id, error_type, message }) => {
                            self.total_errors += 1;
                            self.save_error(&url, &config_id, &project_id, &error_type, &message).await;
                        }
                        Some(CrawlResultMsg::Flush) => {
                            if !self.batch_results.is_empty() || !self.batch_links.is_empty() || !self.batch_redirects.is_empty() {
                                self.flush().await;
                            }
                        }
                        Some(CrawlResultMsg::Done) => {
                            if !self.batch_results.is_empty() || !self.batch_links.is_empty() || !self.batch_redirects.is_empty() {
                                self.flush().await;
                            }
                            info!(
                                "DbWriter finished for project: {} ({} pages, {} errors)",
                                self.project_id, self.total_crawled, self.total_errors
                            );
                            break;
                        }
                        None => {
                            if !self.batch_results.is_empty() || !self.batch_links.is_empty() || !self.batch_redirects.is_empty() {
                                self.flush().await;
                            }
                            info!("DbWriter channel closed for project: {}", self.project_id);
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    if self.last_flush.elapsed() >= self.flush_interval
                        && (!self.batch_results.is_empty() || !self.batch_links.is_empty() || !self.batch_redirects.is_empty()) {
                        self.flush().await;
                    }
                }
            }
        }
    }

    async fn flush(&mut self) {
        let results = std::mem::take(&mut self.batch_results);
        let links = std::mem::take(&mut self.batch_links);
        let redirects = std::mem::take(&mut self.batch_redirects);
        let count = results.len();
        let link_count = links.len();
        let redirect_count = redirects.len();

        if count == 0 && link_count == 0 && redirect_count == 0 {
            return;
        }

        let state_read = match self.state.try_read() {
            Ok(read) => read,
            Err(e) => {
                error!("Failed to acquire state read lock: {}", e);
                self.batch_results = results;
                self.batch_links = links;
                self.batch_redirects = redirects;
                return;
            }
        };

        let db = match state_read.db.lock() {
            Ok(db) => db,
            Err(e) => {
                error!("Failed to acquire DB lock: {}", e);
                self.batch_results = results;
                self.batch_links = links;
                self.batch_redirects = redirects;
                return;
            }
        };

        let repo = CrawlRepo::new(&db, Some(&state_read.results_cache));

        // Batch insert pages (this also invalidates the results cache for the project)
        if let Err(e) = repo.save_results_batch(&results) {
            error!(
                "Batch save of {} results failed ({}), retrying per page",
                count, e
            );
            // A single offending row must not drop the whole batch: fall back to
            // saving each page in its own transaction, skipping only the bad ones.
            for result in &results {
                if let Err(e) = repo.save_result(result) {
                    error!(
                        "Skipping page {} ({}) after batch fallback: {}",
                        result.url, result.id, e
                    );
                    self.total_errors += 1;
                }
            }
        }

        // Batch insert links
        if !links.is_empty() {
            if let Err(e) = repo.save_links_batch(&links) {
                error!(
                    "Batch save of {} links failed ({}), retrying per link",
                    link_count, e
                );
                for link in &links {
                    if let Err(e) = repo.save_links_batch(std::slice::from_ref(link)) {
                        error!("Skipping link {} -> {}: {}", link.from_url, link.to_url, e);
                    }
                }
            }
        }

        // Batch insert redirects
        if !redirects.is_empty() {
            if let Err(e) = repo.save_redirect_batch(&redirects) {
                error!(
                    "Batch save of {} redirects failed ({}), retrying per record",
                    redirect_count, e
                );
                for record in &redirects {
                    if let Err(e) = repo.save_redirect_batch(std::slice::from_ref(record)) {
                        error!(
                            "Skipping redirect for page {} ({} -> {:?}): {}",
                            record.page_id,
                            record.redirect_from_url.as_deref().unwrap_or("<none>"),
                            record.chain.first().map(|h| &h.to_url),
                            e
                        );
                    }
                }
            }
        }

        self.last_flush = Instant::now();

        // Adaptive batch sizing: increase batch size when throughput is high
        if count >= self.batch_size && self.batch_size < MAX_BATCH_SIZE {
            self.batch_size = (self.batch_size * 2).min(MAX_BATCH_SIZE);
        } else if count < self.batch_size / 2 && self.batch_size > DEFAULT_BATCH_SIZE {
            self.batch_size = (self.batch_size / 2).max(DEFAULT_BATCH_SIZE);
        }

        // Emit crawl-batch event on every flush so results appear on-demand
        if count > 0 {
            if let Err(e) = self.app_handle.emit(
                "crawl-batch",
                serde_json::json!({
                    "project_id": &self.project_id,
                    "count": count,
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
                let repo = CrawlRepo::new(&db, None);
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
