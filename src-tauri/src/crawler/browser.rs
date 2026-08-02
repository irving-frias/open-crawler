use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use tracing::{info, warn};
use url::Url;

use crate::crawler::screenshot::find_chrome;
use crate::error::AppError;

const BROWSER_COUNT: usize = 3;
const PAGES_PER_BROWSER: usize = 5;
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(30);
const RENDER_SETTLE_MS: u64 = 2000;

struct PageSlot {
    page: Page,
    lock: Arc<Semaphore>,
}

/// A small pool of headless Chromium browsers. Each browser exposes a fixed
/// number of tabs (pages) that are reused across fetches, preferring to reuse
/// the same tab for the same domain.
pub struct BrowserPool {
    /// Kept alive for the lifetime of the pool. Dropping a `Browser` kills its
    /// Chromium child process (see `Drop for Browser`).
    _browsers: Vec<Browser>,
    /// Unique profile directories per browser, kept alive so Chromium can
    /// launch multiple instances without clashing on the ProcessSingleton.
    _temp_dirs: Vec<tempfile::TempDir>,
    pages: Vec<Arc<PageSlot>>,
    domain_pages: RwLock<HashMap<String, usize>>,
    cursor: AtomicUsize,
    semaphore: Arc<Semaphore>,
}

/// A page checked out from the pool. The page stays reserved for the current
/// fetch until this value is dropped.
pub struct AcquiredPage {
    page: Page,
    _guard: OwnedSemaphorePermit,
    _permit: OwnedSemaphorePermit,
}

impl AcquiredPage {
    pub fn page(&self) -> &Page {
        &self.page
    }
}

impl BrowserPool {
    pub async fn launch() -> Result<Self, AppError> {
        let chrome_path = find_chrome().map_err(|_| {
            AppError::Crawl(
                "Chromium/Chrome not found. Install Chrome to enable JS rendering.".to_string(),
            )
        })?;

        let mut browsers = Vec::with_capacity(BROWSER_COUNT);
        let mut temp_dirs = Vec::with_capacity(BROWSER_COUNT);
        let mut pages: Vec<Arc<PageSlot>> = Vec::with_capacity(BROWSER_COUNT * PAGES_PER_BROWSER);

        for _ in 0..BROWSER_COUNT {
            // Chromium aborts a second instance sharing a profile directory
            // (ProcessSingleton), so each browser needs its own user data dir.
            let data_dir = tempfile::tempdir()
                .map_err(|e| AppError::Crawl(format!("Failed to create browser profile dir: {e}")))?;

            let config = BrowserConfig::builder()
                .chrome_executable(&chrome_path)
                .user_data_dir(data_dir.path())
                .no_sandbox()
                .window_size(1280, 900)
                .launch_timeout(LAUNCH_TIMEOUT)
                .build()
                .map_err(|e| AppError::Crawl(format!("Invalid browser config: {e}")))?;

            let (browser, mut handler) = Browser::launch(config)
                .await
                .map_err(|e| AppError::Crawl(format!("Failed to launch Chromium: {e}")))?;

            tokio::spawn(async move {
                while let Some(_event) = handler.next().await {}
            });

            for _ in 0..PAGES_PER_BROWSER {
                let page = browser
                    .new_page("about:blank")
                    .await
                    .map_err(|e| AppError::Crawl(format!("Failed to open browser tab: {e}")))?;
                pages.push(Arc::new(PageSlot {
                    page,
                    lock: Arc::new(Semaphore::new(1)),
                }));
            }

            browsers.push(browser);
            temp_dirs.push(data_dir);
        }

        let total = pages.len();
        info!("Launched {BROWSER_COUNT} Chromium browsers with {total} tabs");

        Ok(Self {
            _browsers: browsers,
            _temp_dirs: temp_dirs,
            pages,
            domain_pages: RwLock::new(HashMap::new()),
            cursor: AtomicUsize::new(0),
            semaphore: Arc::new(Semaphore::new(total)),
        })
    }

    /// Acquire a free tab. Prefers the tab that previously rendered the same
    /// domain when available, otherwise falls back to round-robin. Concurrency
    /// is bounded by the total number of tabs.
    pub async fn acquire(&self, domain: &str) -> Result<AcquiredPage, AppError> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| AppError::Crawl(e.to_string()))?;

        let n = self.pages.len();
        if n == 0 {
            return Err(AppError::Crawl("Browser pool is empty".to_string()));
        }

        if !domain.is_empty() {
            if let Some(idx) = self.domain_pages.read().await.get(domain).copied() {
                if let Ok(guard) = self.pages[idx].lock.clone().try_acquire_owned() {
                    return Ok(AcquiredPage {
                        page: self.pages[idx].page.clone(),
                        _guard: guard,
                        _permit: permit,
                    });
                }
            }
        }

        let start = self.cursor.fetch_add(1, Ordering::Relaxed) % n;
        for i in 0..n {
            let idx = (start + i) % n;
            if let Ok(guard) = self.pages[idx].lock.clone().try_acquire_owned() {
                if !domain.is_empty() {
                    self.domain_pages.write().await.insert(domain.to_string(), idx);
                }
                return Ok(AcquiredPage {
                    page: self.pages[idx].page.clone(),
                    _guard: guard,
                    _permit: permit,
                });
            }
        }

        // Safety net: with permits == tab count a free tab must exist, but
        // block if a race left all tabs briefly held.
        let idx = start;
        let guard = self.pages[idx]
            .lock
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| AppError::Crawl(format!("Failed to acquire browser tab: {e}")))?;
        if !domain.is_empty() {
            self.domain_pages.write().await.insert(domain.to_string(), idx);
        }
        Ok(AcquiredPage {
            page: self.pages[idx].page.clone(),
            _guard: guard,
            _permit: permit,
        })
    }

    /// Navigate to the given URL and return the fully rendered HTML and the
    /// final URL after redirects.
    pub async fn render(&self, url: &str) -> Result<(String, String), AppError> {
        let domain = Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_default();

        let acquired = self.acquire(&domain).await?;

        let page = acquired.page();
        page.goto(url)
            .await
            .map_err(|e| AppError::Crawl(format!("Navigation failed: {e}")))?;
        page.wait_for_navigation()
            .await
            .map_err(|e| AppError::Crawl(format!("Navigation wait failed: {e}")))?;

        if RENDER_SETTLE_MS > 0 {
            tokio::time::sleep(Duration::from_millis(RENDER_SETTLE_MS)).await;
        }

        let html = page
            .content()
            .await
            .map_err(|e| AppError::Crawl(format!("Failed to read rendered content: {e}")))?;
        let final_url = page
            .url()
            .await
            .ok()
            .flatten()
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| url.to_string());

        Ok((html, final_url))
    }
}

/// A fetch that first performs a plain HTTP request (for status, headers and
/// redirects) and then re-renders the page with JavaScript through the browser
/// pool, replacing the HTML body with the fully rendered DOM.
pub struct JsFetcher {
    http: super::fetcher::HttpFetcher,
    pool: Arc<BrowserPool>,
}

impl JsFetcher {
    pub async fn new(
        user_agent: &str,
        timeout_ms: u64,
        custom_headers: Vec<(String, String)>,
        proxy: Option<&crate::models::ProxyConfig>,
    ) -> Result<Self, AppError> {
        let http = super::fetcher::HttpFetcher::new(user_agent, timeout_ms, custom_headers, proxy)?;
        let pool = Arc::new(BrowserPool::launch().await?);
        Ok(Self { http, pool })
    }
}

#[async_trait::async_trait]
impl super::fetcher::HtmlFetcher for JsFetcher {
    async fn fetch(&self, url: &Url) -> Result<super::fetcher::FetchResponse, AppError> {
        let mut response = self.http.fetch(url).await?;

        let render_start = std::time::Instant::now();
        match self.pool.render(response.url.as_str()).await {
            Ok((html, final_url)) => {
                response.size_bytes = html.len();
                response.html = html;
                response.load_time_ms += render_start.elapsed().as_millis() as u64;
                if let Ok(parsed) = Url::parse(&final_url) {
                    response.url = parsed;
                }
                info!(
                    "Rendered JS for {}: {} bytes, {}ms total",
                    response.url, response.size_bytes, response.load_time_ms
                );
            }
            Err(e) => {
                warn!(
                    "JS render failed for {}: {}, using HTTP body",
                    response.url, e
                );
            }
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_sizing() {
        assert_eq!(BROWSER_COUNT * PAGES_PER_BROWSER, 15);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore]
    async fn browser_pool_renders_js() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let serve = tokio::spawn(async move {
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => break,
                };
                let mut buf = [0u8; 4096];
                let _ = socket.read(&mut buf).await;
                if String::from_utf8_lossy(&buf).starts_with("GET / ") {
                    let body = "<html><head><title>spa</title></head><body><div id='root'></div><script>document.getElementById('root').textContent = 'rendered-by-js';</script></body></html>";
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                }
            }
        });

        let url = format!("http://{addr}/");
        let pool = BrowserPool::launch().await.expect("pool launch");
        eprintln!("pool launched, rendering {url}");
        let (html, final_url) = pool.render(&url).await.expect("render");
        serve.abort();

        assert!(html.contains("rendered-by-js"), "html: {html}");
        assert!(!final_url.is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore]
    async fn direct_browser_goto_works() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let serve = tokio::spawn(async move {
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => break,
                };
                let mut buf = [0u8; 4096];
                let _ = socket.read(&mut buf).await;
                if String::from_utf8_lossy(&buf).starts_with("GET / ") {
                    let body = "<html><body>hello world</body></html>";
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                }
            }
        });

        let chrome_path = find_chrome().unwrap();
        let config = BrowserConfig::builder()
            .chrome_executable(&chrome_path)
            .no_sandbox()
            .build()
            .unwrap();
        let (browser, mut handler) = Browser::launch(config).await.unwrap();
        let handle = tokio::spawn(async move {
            while let Some(_e) = handler.next().await {}
        });

        let url = format!("http://{addr}/");
        eprintln!("direct goto {url}");
        let page = browser.new_page(url.as_str()).await.unwrap();
        page.wait_for_navigation().await.unwrap();
        let html = page.content().await.unwrap();
        serve.abort();
        handle.abort();

        assert!(html.contains("hello world"), "html: {html}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore]
    async fn repro_preview_renders_html() {
        let chrome_path = find_chrome().unwrap();
        let config = BrowserConfig::builder()
            .chrome_executable(&chrome_path)
            .no_sandbox()
            .build()
            .unwrap();
        let (browser, mut handler) = Browser::launch(config).await.unwrap();
        let handle = tokio::spawn(async move {
            while let Some(_e) = handler.next().await {}
        });

        let page = browser
            .new_page("http://[::1]:5174/repro")
            .await
            .unwrap();
        page.wait_for_navigation().await.unwrap();
        tokio::time::sleep(Duration::from_secs(6)).await;

        let tabs_js = r#"
          (() => {
            const all = [...document.querySelectorAll('[data-slot="tabs-trigger"]')].map(t => t.textContent?.trim() ?? '');
            return { tabs: all, body: document.body.innerText.slice(0, 120) };
          })()
        "#;
        let tabs: serde_json::Value = page.evaluate(tabs_js).await.unwrap().into_value().unwrap();
        eprintln!("TABS: {tabs}");

        let js = r#"
          (() => {
            const tab = [...document.querySelectorAll('[data-slot="tabs-trigger"]')].find(t => t.textContent.includes('Vista previa'));
            tab?.click();
            return !!tab;
          })()
        "#;
        let clicked: serde_json::Value = page.evaluate(js).await.unwrap().into_value().unwrap();
        eprintln!("CLICKED_TAB: {clicked}");
        tokio::time::sleep(Duration::from_secs(2)).await;

        let inspect_js = r#"
          (() => {
            const iframe = document.querySelector('iframe.preview-iframe');
            const markers = [...document.querySelectorAll('.overlay-marker')].map(m => ({
              x: m.style.left, y: m.style.top, cls: m.className
            }));
            const doc = iframe?.contentDocument;
            const body = doc ? doc.body.innerText.slice(0, 120) : null;
            return {
              hasIframe: !!iframe,
              iframeHeight: iframe?.style.height,
              markerCount: markers.length,
              markers,
              legendExists: !!document.querySelector('.overlay-legend'),
              hasScriptTag: doc ? (doc.querySelectorAll('script').length > 0) : null,
              iframeBodyText: body,
            };
          })()
        "#;
        let v: serde_json::Value = page.evaluate(inspect_js).await.unwrap().into_value().unwrap();
        eprintln!("PREVIEW_STATE: {}", serde_json::to_string_pretty(&v).unwrap());

        handle.abort();
    }
}
