use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use crate::crawler::client_with_proxy;
use crate::error::AppError;
use crate::models::ProxyConfig;

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub url: Url,
    pub status: u16,
    pub html: String,
    pub headers: HashMap<String, String>,
    pub load_time_ms: u64,
    pub size_bytes: usize,
}

#[async_trait]
pub trait HtmlFetcher: Send + Sync {
    async fn fetch(&self, url: &Url) -> Result<FetchResponse, AppError>;
}

pub struct HttpFetcher {
    client: reqwest::Client,
    custom_headers: Vec<(String, String)>,
}

impl HttpFetcher {
    pub fn new(
        user_agent: &str,
        timeout_ms: u64,
        custom_headers: Vec<(String, String)>,
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
        let response = request.send().await?;

        let status = response.status().as_u16();
        let final_url = response.url().clone();

        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response.bytes().await?;
        let size_bytes = body.len();

        let html = String::from_utf8_lossy(&body).to_string();

        let load_time_ms = start.elapsed().as_millis() as u64;

        Ok(FetchResponse {
            url: final_url,
            status,
            html,
            headers,
            load_time_ms,
            size_bytes,
        })
    }
}
