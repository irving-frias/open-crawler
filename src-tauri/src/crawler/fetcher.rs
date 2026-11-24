use async_trait::async_trait;
use futures::StreamExt;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use crate::crawler::{apply_basic_auth, client_with_proxy, cookie_header_value};
use crate::error::AppError;
use crate::models::{ProxyConfig, RedirectHop, SiteAuth};

/// Hard cap on the number of body bytes we read. Pages larger than this are
/// truncated so a single huge download can't exhaust memory.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Maximum number of redirects the fetcher will follow manually before
/// returning the last 3xx response unchanged.
pub const MAX_REDIRECTS: usize = 10;

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub url: Url,
    pub status: u16,
    pub html: String,
    pub headers: HashMap<String, String>,
    pub load_time_ms: u64,
    pub size_bytes: usize,
    /// True when the response is a security/challenge page (e.g. Cloudflare)
    /// rather than an actual server error. Blocked pages are not counted as
    /// broken pages.
    pub blocked: bool,
    /// The originally-requested URL when the fetch ended somewhere else via
    /// redirects (None when no redirect happened).
    pub redirect_from_url: Option<String>,
    /// The full redirect chain followed (empty when the page did not redirect).
    pub redirect_chain: Vec<RedirectHop>,
}

#[async_trait]
pub trait HtmlFetcher: Send + Sync {
    async fn fetch(&self, url: &Url) -> Result<FetchResponse, AppError>;
}

const CLOUDFLARE_CHALLENGE_MARKERS: &[&str] = &[
    "just a moment",
    "cf-chl",
    "cf_chl",
    "attention required",
    "enable javascript and cookies to continue",
    "verify you are human",
    "checking your browser before accessing",
    "performance & security by cloudflare",
];

/// Detects whether a response is a Cloudflare security/challenge hook instead
/// of a genuine HTTP error page.
pub fn is_cloudflare_challenge(status: u16, headers: &HashMap<String, String>, html: &str) -> bool {
    if status < 400 {
        return false;
    }
    if headers.get("cf-mitigated").is_some() {
        return true;
    }
    let server = headers
        .get("server")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if !server.contains("cloudflare") {
        return false;
    }
    let body = html.to_lowercase();
    CLOUDFLARE_CHALLENGE_MARKERS
        .iter()
        .any(|m| body.contains(m))
}

/// Sends a GET following HTTP redirects manually (the shared client uses
/// `Policy::none`) so the hop-by-hop chain can be captured. Returns the final
/// response and every redirect hop. Headers, cookies and Basic Auth are
/// re-applied to each hop.
pub(crate) async fn send_with_redirects(
    client: &reqwest::Client,
    url: &Url,
    custom_headers: &[(String, String)],
    cookies: &[String],
    site_auth: &Option<SiteAuth>,
) -> Result<(reqwest::Response, Vec<RedirectHop>), AppError> {
    let mut current = url.clone();
    let mut chain: Vec<RedirectHop> = Vec::new();
    loop {
        let mut request = client.get(current.as_str());
        for (key, value) in custom_headers {
            request = request.header(key.as_str(), value.as_str());
        }
        if let Some(cookie) = cookie_header_value(cookies) {
            request = request.header(reqwest::header::COOKIE, cookie);
        }
        request = apply_basic_auth(request, site_auth);
        let response = request.send().await?;

        if chain.len() < MAX_REDIRECTS && response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok());
            if let Some(location) = location {
                if let Ok(next) = current.join(location) {
                    chain.push(RedirectHop {
                        from_url: current.to_string(),
                        to_url: next.to_string(),
                        status_code: response.status().as_u16(),
                    });
                    current = next;
                    continue;
                }
            }
        }
        return Ok((response, chain));
    }
}

pub struct HttpFetcher {
    client: reqwest::Client,
    custom_headers: Vec<(String, String)>,
    cookies: Vec<String>,
    site_auth: Option<SiteAuth>,
}

impl HttpFetcher {
    /// Builds a fetcher that reuses the shared `client` (connection pool + DNS
    /// reused across crawls instead of building a fresh client per crawl). When
    /// a proxy is configured a dedicated client is required (proxies are
    /// client-level in reqwest) so one is built per crawler instance.
    pub fn new(
        client: Option<reqwest::Client>,
        user_agent: &str,
        timeout_ms: u64,
        custom_headers: Vec<(String, String)>,
        cookies: Vec<String>,
        site_auth: Option<SiteAuth>,
        proxy: Option<&ProxyConfig>,
    ) -> Result<Self, AppError> {
        let client = match client {
            Some(shared) if proxy.is_none() => shared,
            _ => client_with_proxy(proxy)?
                .user_agent(user_agent)
                .timeout(Duration::from_millis(timeout_ms))
                .redirect(reqwest::redirect::Policy::none())
                .gzip(true)
                .brotli(true)
                .build()?,
        };

        Ok(Self {
            client,
            custom_headers,
            cookies,
            site_auth,
        })
    }
}

#[async_trait]
impl HtmlFetcher for HttpFetcher {
    async fn fetch(&self, url: &Url) -> Result<FetchResponse, AppError> {
        let start = std::time::Instant::now();

        let (response, redirect_chain) = send_with_redirects(
            &self.client,
            url,
            &self.custom_headers,
            &self.cookies,
            &self.site_auth,
        )
        .await?;

        let status = response.status().as_u16();
        let final_url = response.url().clone();

        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let mut body_bytes: Vec<u8> = Vec::with_capacity(8192);
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AppError::Crawl(e.to_string()))?;
            if body_bytes.len() + chunk.len() > MAX_BODY_BYTES {
                let remaining = MAX_BODY_BYTES - body_bytes.len();
                body_bytes.extend_from_slice(&chunk[..remaining]);
                break;
            }
            body_bytes.extend_from_slice(&chunk);
        }
        let size_bytes = body_bytes.len();

        let html = String::from_utf8_lossy(&body_bytes).to_string();

        let load_time_ms = start.elapsed().as_millis() as u64;

        let blocked = is_cloudflare_challenge(status, &headers, &html);

        let redirect_from_url = if redirect_chain.is_empty() {
            None
        } else {
            Some(url.to_string())
        };

        Ok(FetchResponse {
            url: final_url,
            status,
            html,
            headers,
            load_time_ms,
            size_bytes,
            blocked,
            redirect_from_url,
            redirect_chain,
        })
    }
}

/// Result of probing for a system Chrome/Chromium install, without launching
/// a browser. The UI uses it to enable/disable the JS-rendering toggle.
#[derive(Debug, Clone, Serialize)]
pub struct JsAvailability {
    pub available: bool,
    pub chrome_path: Option<String>,
    pub message: String,
}

/// Probes for a system Chrome/Chromium without launching a browser.
pub fn check_js_availability() -> JsAvailability {
    match crate::crawler::screenshot::find_chrome() {
        Ok(path) => JsAvailability {
            available: true,
            chrome_path: Some(path),
            message: String::new(),
        },
        Err(_) => JsAvailability {
            available: false,
            chrome_path: None,
            message:
                "Chrome/Chromium not found. Install Google Chrome to enable JavaScript rendering."
                    .to_string(),
        },
    }
}

/// Fetches pages through a headless system Chrome (`--dump-dom`) so
/// JavaScript-rendered content becomes visible to the parser and the SEO
/// checks (SPAs, JS-injected links and content). The HTTP status, headers
/// and redirect chain still come from a plain `HttpFetcher` request —
/// `--dump-dom` cannot report them — and only the HTML body is replaced
/// with the rendered DOM. No new dependencies, no bundled browser: it reuses
/// the same `find_chrome()` as the screenshot feature.
///
/// One Chrome process per page, so crawls run with a lower concurrency when
/// this fetcher is active. Construction fails fast with a clear message when
/// no browser is installed, instead of failing on page one.
pub struct ChromiumFetcher {
    http: HttpFetcher,
    chrome_path: String,
    virtual_time_budget_ms: u64,
    timeout_ms: u64,
}

impl ChromiumFetcher {
    pub fn new(
        http: HttpFetcher,
        virtual_time_budget_ms: u64,
        timeout_ms: u64,
    ) -> Result<Self, AppError> {
        let chrome_path = crate::crawler::screenshot::find_chrome().map_err(|_| {
            AppError::Crawl(
                "Chrome/Chromium not found. Install Google Chrome to enable JavaScript rendering."
                    .to_string(),
            )
        })?;
        Ok(Self {
            http,
            chrome_path,
            virtual_time_budget_ms,
            timeout_ms,
        })
    }

    /// Renders `url` in headless Chrome and returns the serialized DOM.
    /// `--virtual-time-budget` fast-forwards virtual time so scripts and
    /// delayed rendering settle without waiting wall-clock time.
    async fn render_dom(&self, url: &Url) -> Result<String, AppError> {
        let output = tokio::time::timeout(
            Duration::from_millis(self.timeout_ms),
            tokio::process::Command::new(&self.chrome_path)
                .args([
                    "--headless",
                    "--disable-gpu",
                    "--no-sandbox",
                    "--disable-dev-shm-usage",
                    "--disable-software-rasterizer",
                    "--hide-scrollbars",
                    "--dump-dom",
                    &format!("--virtual-time-budget={}", self.virtual_time_budget_ms),
                    url.as_str(),
                ])
                .output(),
        )
        .await
        .map_err(|_| {
            AppError::Crawl(format!(
                "Timed out rendering {} ({}ms)",
                url, self.timeout_ms
            ))
        })?
        .map_err(|e| AppError::Crawl(format!("Failed to run Chrome: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Crawl(format!(
                "Chrome failed to render {}: {}",
                url,
                stderr.chars().take(300).collect::<String>()
            )));
        }
        let html = String::from_utf8_lossy(&output.stdout).to_string();
        if html.trim().is_empty() {
            return Err(AppError::Crawl(format!(
                "Chrome returned an empty DOM for {}",
                url
            )));
        }
        Ok(html)
    }
}

#[async_trait]
impl HtmlFetcher for ChromiumFetcher {
    async fn fetch(&self, url: &Url) -> Result<FetchResponse, AppError> {
        let mut response = self.http.fetch(url).await?;
        // Blocked (challenge) pages gain nothing from rendering; keep the
        // plain response and skip the Chrome process.
        if response.blocked {
            return Ok(response);
        }
        let rendered = self.render_dom(&response.url).await?;
        let mut bytes = rendered.into_bytes();
        if bytes.len() > MAX_BODY_BYTES {
            bytes.truncate(MAX_BODY_BYTES);
        }
        let html = String::from_utf8_lossy(&bytes).to_string();
        response.size_bytes = html.len();
        response.blocked = is_cloudflare_challenge(response.status, &response.headers, &html);
        response.html = html;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Serves one page whose only link is injected by JavaScript, plus the
    /// injected target. Mirrors the `spawn_no_sitemap_site` helper in
    /// `engine.rs` tests.
    async fn spawn_js_site() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let origin = format!("http://{}", addr);

        tokio::spawn(async move {
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => break,
                };
                let mut buf = [0u8; 4096];
                let _ = socket.read(&mut buf).await;
                let req = String::from_utf8_lossy(&buf);
                let path = req.split_whitespace().nth(1).unwrap_or("/");

                let (status, body) = match path {
                    "/" => (
                        "200 OK",
                        r#"<!DOCTYPE html><html lang="en"><head><title>JS Home</title></head><body><h1>JS Home</h1><div id="app"></div><script>document.getElementById('app').innerHTML='<a href="/injected">Injected</a>';</script></body></html>"#,
                    ),
                    "/injected" => (
                        "200 OK",
                        r#"<!DOCTYPE html><html lang="en"><head><title>Injected</title></head><body><h1>Injected</h1></body></html>"#,
                    ),
                    _ => ("404 Not Found", "<html><body>404</body></html>"),
                };

                let resp = format!(
                    "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n{}",
                    status,
                    body.len(),
                    body
                );
                let _ = socket.write_all(resp.as_bytes()).await;
                let _ = socket.shutdown().await;
            }
        });

        origin
    }

    #[test]
    fn js_availability_is_consistent() {
        let availability = check_js_availability();
        if availability.available {
            let path = availability.chrome_path.expect("path when available");
            assert!(
                std::path::Path::new(&path).exists(),
                "reported Chrome path does not exist: {}",
                path
            );
            assert!(availability.message.is_empty());
        } else {
            assert!(availability.chrome_path.is_none());
            assert!(!availability.message.is_empty());
        }
    }

    #[tokio::test]
    async fn chromium_fetcher_renders_javascript() {
        if !check_js_availability().available {
            eprintln!("skipping: no Chrome/Chromium installed");
            return;
        }
        let origin = spawn_js_site().await;
        let http =
            HttpFetcher::new(None, "OpenCrawler/test", 10_000, vec![], vec![], None, None).unwrap();
        let fetcher = ChromiumFetcher::new(http, 5000, 30_000).unwrap();
        let resp = fetcher
            .fetch(&Url::parse(&format!("{}/", origin)).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status, 200);
        assert!(
            resp.html.contains("/injected"),
            "rendered DOM should contain the JS-injected link, got: {}",
            &resp.html[..resp.html.len().min(500)]
        );
        assert!(resp.html.contains("Injected"));
    }

    #[tokio::test]
    async fn chromium_fetcher_keeps_http_status_on_404() {
        if !check_js_availability().available {
            eprintln!("skipping: no Chrome/Chromium installed");
            return;
        }
        let origin = spawn_js_site().await;
        let http =
            HttpFetcher::new(None, "OpenCrawler/test", 10_000, vec![], vec![], None, None).unwrap();
        let fetcher = ChromiumFetcher::new(http, 5000, 30_000).unwrap();
        let resp = fetcher
            .fetch(&Url::parse(&format!("{}/missing", origin)).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status, 404);
    }
}
