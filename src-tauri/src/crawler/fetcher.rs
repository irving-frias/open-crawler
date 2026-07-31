use async_trait::async_trait;
use std::collections::HashMap;
use url::Url;

use crate::error::AppError;

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
}

impl HttpFetcher {
    pub fn new(user_agent: &str) -> Result<Self, AppError> {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .gzip(true)
            .brotli(true)
            .build()?;

        Ok(Self { client })
    }
}

#[async_trait]
impl HtmlFetcher for HttpFetcher {
    async fn fetch(&self, url: &Url) -> Result<FetchResponse, AppError> {
        let start = std::time::Instant::now();

        let response = self.client.get(url.as_str()).send().await?;

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
