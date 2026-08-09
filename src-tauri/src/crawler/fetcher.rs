use async_trait::async_trait;
use futures::StreamExt;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use crate::crawler::{apply_basic_auth, client_with_proxy, cookie_header_value};
use crate::error::AppError;
use crate::models::{ProxyConfig, SiteAuth};

/// Hard cap on the number of body bytes we read. Pages larger than this are
/// truncated so a single huge download can't exhaust memory.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

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

pub struct HttpFetcher {
    client: reqwest::Client,
    custom_headers: Vec<(String, String)>,
    cookies: Vec<String>,
    site_auth: Option<SiteAuth>,
}

impl HttpFetcher {
    pub fn new(
        user_agent: &str,
        timeout_ms: u64,
        custom_headers: Vec<(String, String)>,
        cookies: Vec<String>,
        site_auth: Option<SiteAuth>,
        proxy: Option<&ProxyConfig>,
    ) -> Result<Self, AppError> {
        let client = client_with_proxy(proxy)?
            .user_agent(user_agent)
            .timeout(Duration::from_millis(timeout_ms))
            .redirect(reqwest::redirect::Policy::limited(10))
            .gzip(true)
            .brotli(true)
            .build()?;

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

        let mut request = self.client.get(url.as_str());
        for (key, value) in &self.custom_headers {
            request = request.header(key.as_str(), value.as_str());
        }
        if let Some(cookie) = cookie_header_value(&self.cookies) {
            request = request.header(reqwest::header::COOKIE, cookie);
        }
        request = apply_basic_auth(request, &self.site_auth);
        let response = request.send().await?;

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

        Ok(FetchResponse {
            url: final_url,
            status,
            html,
            headers,
            load_time_ms,
            size_bytes,
            blocked,
        })
    }
}
