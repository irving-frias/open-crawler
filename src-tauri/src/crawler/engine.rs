use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use lru::LruCache;
use tauri::Emitter;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use url::Url;
use uuid::Uuid;

use crate::crawler::db_writer::{create_db_writer_channel, CrawlResultMsg, DbWriter};
use crate::crawler::fetcher::{HtmlFetcher, HttpFetcher};
use crate::crawler::frontier::Frontier;
use crate::crawler::parser::SeoParser;
use crate::crawler::robots::RobotsChecker;
use crate::crawler::sitemap::SitemapParser;
use crate::db::CrawlRepo;
use crate::error::AppError;
use crate::models::{CrawlConfig, CrawlProgress, CrawlResult, PageLink};
use crate::AppState;

const LRU_CAPACITY: usize = 500_000;
const QUEUE_FLUSH_URLS: u32 = 100;

pub struct CrawlEngine {
    config: Option<CrawlConfig>,
    visited: LruCache<String, ()>,
    fetcher: Option<Arc<Box<dyn HtmlFetcher>>>,
    parser: SeoParser,
    allowed_origins: Vec<String>,
    frontier: Option<Frontier>,
}

impl Default for CrawlEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CrawlEngine {
    pub fn new() -> Self {
        Self {
            config: None,
            visited: LruCache::new(NonZeroUsize::new(LRU_CAPACITY).unwrap()),
            fetcher: None,
            parser: SeoParser::new(),
            allowed_origins: Vec::new(),
            frontier: None,
        }
    }

    pub fn set_config(&mut self, config: CrawlConfig) {
        self.config = Some(config);
    }

    #[allow(dead_code)]
    fn is_same_origin(&self, url: &Url) -> bool {
        if !self.config.as_ref().map(|c| c.same_origin_only).unwrap_or(true) {
            return true;
        }
        let origin = format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""));
        self.allowed_origins.contains(&origin)
    }

    fn url_visited(&self, url: &str) -> bool {
        self.visited.contains(url)
    }

    fn mark_visited(&mut self, url: String) {
        self.visited.put(url, ());
    }

    pub async fn start(
        &mut self,
        app: Arc<tauri::AppHandle>,
        state: Arc<RwLock<AppState>>,
        cancellation: CancellationToken,
        project_id: &str,
    ) -> Result<(), AppError> {
        let config = self
            .config
            .clone()
            .ok_or_else(|| AppError::InvalidConfig("No configuration set".to_string()))?;

        let concurrency = config.concurrency.max(1);
        let project_id = project_id.to_string();
        info!("=== CRAWL STARTING for project: {} ===", project_id);
        info!(
            "Seed URLs: {:?}, Concurrency: {}, Delay: {}ms, Time limit: {}s",
            config.seed_urls, concurrency, config.delay_ms, config.max_crawl_time_secs
        );

        self.visited.clear();
        self.allowed_origins.clear();

        // Extract allowed origins from seed URLs
        for seed in &config.seed_urls {
            let normalized = if !seed.starts_with("http://") && !seed.starts_with("https://") {
                format!("https://{}", seed)
            } else {
                seed.clone()
            };
            if let Ok(u) = Url::parse(&normalized) {
                let origin = format!("{}://{}", u.scheme(), u.host_str().unwrap_or(""));
                self.allowed_origins.push(origin);
            }
        }
        info!("Allowed origins: {:?}", self.allowed_origins);

        // Create fetcher with implicit user agent
        let fetcher = HttpFetcher::new(config.user_agent())?;
        self.fetcher = Some(Arc::new(Box::new(fetcher)));

        // Initialize frontier
        self.frontier = Some(Frontier::new(config.max_depth, 100_000));

        // Check for interrupted session (resume)
        let session_id;
        let is_resume;
        {
            let state_read = state.read().await;
            let db = state_read
                .db
                .lock()
                .map_err(|e| AppError::Crawl(e.to_string()))?;
            let repo = CrawlRepo::new(&db);

            if let Some(interrupted) = repo.get_interrupted_session(&project_id)? {
                info!(
                    "Found interrupted session: {} (pages: {}, errors: {}, elapsed: {}s)",
                    interrupted.id, interrupted.pages_crawled, interrupted.errors, interrupted.elapsed_secs
                );
                session_id = interrupted.id.clone();

                // Load queue from DB
                let queue_entries = repo.load_queue(&session_id)?;
                let frontier = self.frontier.as_mut().unwrap();
                frontier.restore(queue_entries);

                // Load visited URLs from DB
                let visited_urls = repo.get_visited_urls_for_project(&project_id)?;
                for url in &visited_urls {
                    self.visited.put(url.clone(), ());
                }

                is_resume = true;
                info!(
                    "Resuming: {} queued URLs, {} visited URLs loaded",
                    frontier.len(),
                    self.visited.len()
                );
            } else {
                // Create new session
                session_id = repo.create_session(&project_id, &config)?;
                is_resume = false;
            }
        }

        // Initialize queue with seed URLs (only if not resuming)
        if !is_resume {
            for seed in &config.seed_urls {
                let normalized = if !seed.starts_with("http://") && !seed.starts_with("https://") {
                    format!("https://{}", seed)
                } else {
                    seed.clone()
                };
                info!("Adding seed URL: {}", normalized);
                self.visited.put(normalized.clone(), ());
                if let Some(ref mut frontier) = self.frontier {
                    frontier.push(normalized, 0);
                }
            }
        }

        // Sitemap discovery
        if config.check_sitemap {
            let sitemap_parser = SitemapParser::new(config.user_agent())?;
            let origins_clone = self.allowed_origins.clone();
            for origin in &origins_clone {
                info!("Discovering sitemaps for: {}", origin);
                let result = sitemap_parser.discover(origin).await;
                let sitemap_urls = SitemapParser::urls_as_strings(&result.urls);
                info!(
                    "Sitemap: {} URLs from {} sitemaps checked",
                    sitemap_urls.len(),
                    result.sitemaps_checked.len()
                );
                for url in sitemap_urls {
                    if !self.url_visited(&url) {
                        self.mark_visited(url.clone());
                        if let Some(ref mut frontier) = self.frontier {
                            frontier.push(url, 0);
                        }
                    }
                }
                let _ = app.emit(
                    "sitemap-discovered",
                    serde_json::json!({
                        "origin": origin,
                        "urls_found": result.urls.len(),
                        "sitemaps_checked": result.sitemaps_checked.len(),
                    }),
                );
            }
        }

        let fetcher = self.fetcher.as_ref().unwrap().clone();
        let semaphore = Arc::new(Semaphore::new(concurrency as usize));
        let mut pages_crawled: u32 = 0;
        let mut errors: u32 = 0;
        let start_time = Instant::now();

        // Get config ID
        let config_id = {
            let state_read = state.read().await;
            let db = state_read
                .db
                .lock()
                .map_err(|e| AppError::Crawl(e.to_string()))?;
            let repo = CrawlRepo::new(&db);
            repo.save_config(&config)?;
            config.id.clone().unwrap_or_else(|| "default".to_string())
        };

        // Create DbWriter channel
        let (db_tx, db_rx) = create_db_writer_channel(1000);
        let mut db_writer = DbWriter::new(
            db_rx,
            app.clone(),
            state.clone(),
            project_id.clone(),
        );

        // Spawn DbWriter task
        let db_writer_handle = tokio::spawn(async move {
            db_writer.run().await;
        });

        // Create RobotsChecker
        let mut robots_checker = RobotsChecker::new(config.user_agent())?;

        info!("Frontier has {} URLs to process", {
            self.frontier.as_ref().map(|f| f.len()).unwrap_or(0)
        });

        // Channel for new URLs from fetch tasks
        let (url_tx, mut url_rx) = tokio::sync::mpsc::unbounded_channel::<(String, u32)>();

        // In-flight counter
        let in_flight = Arc::new(tokio::sync::RwLock::new(0u32));

        // Spawn URL receiver
        let frontier_for_recv = Arc::new(tokio::sync::RwLock::new(Frontier::new(config.max_depth, 100_000)));
        let frontier_recv = frontier_for_recv.clone();
        let visited_for_recv = Arc::new(tokio::sync::RwLock::new(LruCache::new(NonZeroUsize::new(LRU_CAPACITY).unwrap())));
        let visited_recv_clone = visited_for_recv.clone();
        let recv_handle = tokio::spawn(async move {
            while let Some((new_url, depth)) = url_rx.recv().await {
                let mut visited = visited_recv_clone.write().await;
                if !visited.contains(&new_url) {
                    visited.put(new_url.clone(), ());
                    frontier_recv.write().await.push(new_url, depth);
                }
            }
        });

        // Move frontier into shared state
        if let Some(frontier) = self.frontier.take() {
            {
                let mut f = frontier_for_recv.write().await;
                *f = frontier;
            }
        }

        // Periodic queue flush counter
        let mut urls_since_flush: u32 = 0;

        // Main crawl loop
        loop {
            if cancellation.is_cancelled() {
                info!("Crawl cancelled by user for project: {}", project_id);
                break;
            }

            // Time-based limit
            if config.max_crawl_time_secs > 0
                && start_time.elapsed().as_secs() >= config.max_crawl_time_secs
            {
                info!(
                    "Reached time limit: {}s",
                    config.max_crawl_time_secs
                );
                break;
            }

            // Try to get next URL from frontier
            let url_entry = {
                let mut f = frontier_for_recv.write().await;
                f.pop()
            };

            match url_entry {
                Some(entry) => {
                    let url_str = entry.url;
                    let url = match Url::parse(&url_str) {
                        Ok(u) => u,
                        Err(e) => {
                            warn!("Invalid URL {}: {}", url_str, e);
                            errors += 1;
                            continue;
                        }
                    };

                    // Robots.txt check
                    if config.respect_robots
                        && !robots_checker.can_fetch(&url).await
                    {
                        info!("Blocked by robots.txt: {}", url_str);
                        continue;
                    }

                    let q_len = { frontier_for_recv.read().await.len() };
                    let in_flight_count = { *in_flight.read().await };

                    info!(
                        "[{}/~] Fetching: {} (queued: {}, in-flight: {}, elapsed: {:.0}s)",
                        pages_crawled + 1,
                        url_str,
                        q_len,
                        in_flight_count,
                        start_time.elapsed().as_secs_f64(),
                    );

                    // Update crawl state progress
                    {
                        let state_write = state.write().await;
                        let mut crawls = state_write.crawls.write().await;
                        if let Some(crawl_state) = crawls.get_mut(&project_id) {
                            crawl_state.progress = CrawlProgress {
                                project_id: project_id.clone(),
                                urls_crawled: pages_crawled,
                                urls_queued: q_len as u32 + in_flight_count,
                                current_url: url_str.clone(),
                                errors,
                                elapsed_secs: start_time.elapsed().as_secs(),
                            };
                        }
                    }

                    let progress = CrawlProgress {
                        project_id: project_id.clone(),
                        urls_crawled: pages_crawled,
                        urls_queued: q_len as u32 + in_flight_count,
                        current_url: url_str.clone(),
                        errors,
                        elapsed_secs: start_time.elapsed().as_secs(),
                    };
                    if let Err(e) = app.emit("crawl-progress", &progress) {
                        warn!("Failed to emit progress: {}", e);
                    }

                    // Acquire semaphore permit
                    let permit = semaphore.clone().acquire_owned().await.unwrap();

                    // Increment in-flight
                    {
                        *in_flight.write().await += 1;
                    }

                    // Clone for the task
                    let fetcher_clone = fetcher.clone();
                    let config_id_clone = config_id.clone();
                    let project_id_clone = project_id.clone();
                    let url_clone = url_str.clone();
                    let visited_clone = visited_for_recv.clone();
                    let parser = self.parser.clone();
                    let delay_ms = config.delay_ms;
                    let url_tx_clone = url_tx.clone();
                    let in_flight_clone = in_flight.clone();
                    let allowed_origins_clone = self.allowed_origins.clone();
                    let same_origin_only = config.same_origin_only;
                    let depth = entry.depth;
                    let db_tx_clone = db_tx.clone();
                    let db_tx_error = db_tx.clone();

                    tokio::spawn(async move {
                        let result = Self::fetch_and_parse(
                            &**fetcher_clone,
                            &url,
                            &parser,
                            &config_id_clone,
                            &project_id_clone,
                            &visited_clone,
                            url_tx_clone,
                            &allowed_origins_clone,
                            same_origin_only,
                            depth,
                            db_tx_clone,
                        )
                        .await;

                        match result {
                            Ok((_pages, new_urls)) => {
                                info!("  -> Completed {}: {} new links", url_clone, new_urls);
                            }
                            Err(e) => {
                                warn!("  -> FAILED to fetch {}: {}", url_clone, e);

                                // Send error to DbWriter
                                let _ = db_tx_error.send(CrawlResultMsg::Error {
                                    url: url_clone.clone(),
                                    config_id: config_id_clone.clone(),
                                    project_id: project_id_clone.clone(),
                                    error_type: "fetch_error".to_string(),
                                    message: e.to_string(),
                                }).await;
                            }
                        }

                        if delay_ms > 0 {
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        }

                        // Decrement in-flight
                        {
                            *in_flight_clone.write().await -= 1;
                        }

                        drop(permit);
                    });

                    pages_crawled += 1;

                    // Periodic session progress update
                    urls_since_flush += 1;
                    if urls_since_flush >= QUEUE_FLUSH_URLS {
                        urls_since_flush = 0;

                        // Read frontier snapshot first (before acquiring DB lock)
                        let queue_entries: Vec<(String, u32)> = {
                            let frontier_snap = frontier_for_recv.read().await;
                            frontier_snap.drain_all()
                        };

                        // Now do DB operations (no await needed)
                        if let Ok(state_read) = state.try_read() {
                            if let Ok(db) = state_read.db.lock() {
                                let repo = CrawlRepo::new(&db);
                                let _ = repo.update_session_progress(
                                    &session_id,
                                    pages_crawled,
                                    errors,
                                    start_time.elapsed().as_secs(),
                                );

                                if !queue_entries.is_empty() {
                                    let _ = repo.save_queue_batch(&session_id, &queue_entries);
                                }
                            }
                        }
                    }
                }
                None => {
                    // Queue is empty - check if we should wait for more URLs
                    let in_flight_count = { *in_flight.read().await };

                    if in_flight_count == 0 {
                        info!("No more URLs to process and no in-flight tasks");
                        break;
                    }

                    // Wait for new URLs to arrive or timeout
                    info!(
                        "Queue empty, waiting for {} in-flight tasks to discover new URLs...",
                        in_flight_count
                    );
                    tokio::select! {
                        _ = cancellation.cancelled() => {
                            info!("Crawl cancelled while waiting for project: {}", project_id);
                            break;
                        }
                        _ = tokio::time::sleep(Duration::from_millis(500)) => {
                            let q_len = { frontier_for_recv.read().await.len() };
                            if q_len > 0 {
                                info!("Received {} new URLs, continuing...", q_len);
                            }
                            continue;
                        }
                    }
                }
            }
        }

        // Wait for in-flight tasks
        info!("Waiting for in-flight tasks to complete...");
        loop {
            let in_flight_count = { *in_flight.read().await };
            if in_flight_count == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Close URL channel
        drop(url_tx);
        let _ = recv_handle.await;

        // Signal DbWriter to finish
        let _ = db_tx.send(CrawlResultMsg::Done).await;

        // Update session status
        let state_read = state.read().await;
        if let Ok(db) = state_read.db.lock() {
            let repo = CrawlRepo::new(&db);
            let _ = repo.complete_session(&session_id);
        }
        drop(state_read);

        // Wait for DbWriter to finish
        let _ = db_writer_handle.await;

        // Emit final progress
        let progress = CrawlProgress {
            project_id: project_id.clone(),
            urls_crawled: pages_crawled,
            urls_queued: 0,
            current_url: String::new(),
            errors,
            elapsed_secs: start_time.elapsed().as_secs(),
        };
        let _ = app.emit("crawl-progress", &progress);

        // Emit completion
        info!("Emitting crawl-complete event for project: {}", project_id);
        let _ = app.emit(
            "crawl-complete",
            serde_json::json!({
                "project_id": &project_id,
                "pages_crawled": pages_crawled,
                "errors": errors,
                "elapsed_secs": start_time.elapsed().as_secs(),
            }),
        );

        info!(
            "=== CRAWL COMPLETE for project {}: {} pages, {} errors, {:.1}s ===",
            project_id,
            pages_crawled,
            errors,
            start_time.elapsed().as_secs_f64()
        );

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_and_parse(
        fetcher: &(dyn HtmlFetcher + 'static),
        url: &Url,
        parser: &SeoParser,
        config_id: &str,
        project_id: &str,
        visited: &Arc<tokio::sync::RwLock<LruCache<String, ()>>>,
        url_tx: tokio::sync::mpsc::UnboundedSender<(String, u32)>,
        allowed_origins: &[String],
        same_origin_only: bool,
        depth: u32,
        db_tx: mpsc::Sender<CrawlResultMsg>,
    ) -> Result<(u32, u32), AppError> {
        let response = fetcher.fetch(url).await?;

        info!(
            "  -> Status: {}, Size: {} bytes, Time: {}ms",
            response.status, response.size_bytes, response.load_time_ms
        );

        let (seo_data, outgoing_urls) = parser.parse(&response.html, url);
        info!("  -> Title: {:?}", seo_data.title);

        // Serialize hreflang
        let hreflang_json = if seo_data.hreflang_links.is_empty() {
            None
        } else {
            serde_json::to_string(&seo_data.hreflang_links).ok()
        };

        // Serialize semantic issues
        let semantic_issues_json = if seo_data.semantic_issues.is_empty() {
            None
        } else {
            serde_json::to_string(&seo_data.semantic_issues).ok()
        };

        // Store first 100KB of HTML for DOM tree analysis
        let html_body = {
            let html = &response.html;
            let max_bytes = 100 * 1024;
            if html.len() > max_bytes {
                Some(html[..max_bytes].to_string())
            } else {
                Some(html.clone())
            }
        };

        let links: Vec<PageLink> = seo_data
            .outgoing_links
            .iter()
            .map(|l| PageLink {
                from_url: response.url.to_string(),
                to_url: l.url.clone(),
                config_id: config_id.to_string(),
                project_id: project_id.to_string(),
                link_type: "a".to_string(),
                anchor_text: Some(l.anchor_text.clone()),
                is_follow: l.is_follow,
            })
            .collect();

        let result = CrawlResult {
            id: Uuid::new_v4().to_string(),
            config_id: config_id.to_string(),
            project_id: project_id.to_string(),
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
            parent_url: None,
            crawl_timestamp: chrono::Utc::now().to_rfc3339(),
            links: links.clone(),
            html_lang: seo_data.html_lang,
            hreflang_json,
            semantic_issues_json,
            html_body,
        };

        // Send result to DbWriter (async, non-blocking)
        let _ = db_tx.send(CrawlResultMsg::Page(Box::new(result))).await;
        if !links.is_empty() {
            let _ = db_tx.send(CrawlResultMsg::Links(links)).await;
        }

        // Send new URLs to receiver (with same-origin filter)
        let mut new_urls = 0u32;
        let visited_read = visited.read().await;
        for outgoing_url in &outgoing_urls {
            if visited_read.contains(outgoing_url) {
                continue;
            }
            if !outgoing_url.starts_with("http://") && !outgoing_url.starts_with("https://") {
                continue;
            }
            // Skip non-page files (assets, documents, media)
            if is_static_asset(outgoing_url) {
                continue;
            }
            // Same-origin filter
            if same_origin_only {
                if let Ok(parsed) = Url::parse(outgoing_url) {
                    let origin =
                        format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));
                    if !allowed_origins.contains(&origin) {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            let _ = url_tx.send((outgoing_url.clone(), depth + 1));
            new_urls += 1;
        }

        Ok((1, new_urls))
    }
}

/// Returns true if the URL points to a static asset (not an HTML page)
fn is_static_asset(url: &str) -> bool {
    // Strip query string and fragment for extension check
    let path = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);

    // If URL ends with a known non-HTML extension, skip it
    let path_lower = path.to_lowercase();
    let skip_extensions = [
        ".css", ".js", ".mjs", ".cjs",
        ".png", ".jpg", ".jpeg", ".gif", ".webp", ".avif", ".svg", ".ico", ".bmp", ".tiff",
        ".woff", ".woff2", ".ttf", ".eot", ".otf",
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx",
        ".zip", ".gz", ".tar", ".rar", ".7z",
        ".mp3", ".mp4", ".webm", ".ogg", ".wav", ".flac",
        ".avi", ".mov", ".mkv", ".m4v",
        ".xml", ".json", ".txt", ".csv",
        ".apk", ".exe", ".dmg", ".deb", ".rpm",
        ".mp3", ".aac", ".wma",
    ];

    for ext in &skip_extensions {
        if path_lower.ends_with(ext) {
            return true;
        }
    }

    // Also skip URLs with no extension that look like file downloads
    // (e.g., /path/to/file.zip?foo=bar already caught above, but
    //  /download/12345 is fine — it might be a page)

    false
}
