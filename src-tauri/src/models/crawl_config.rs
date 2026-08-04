use serde::{Deserialize, Serialize};

pub const IMPLICIT_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 OpenCrawler/1.0";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlConfig {
    pub id: Option<String>,
    pub project_id: Option<String>,
    pub seed_urls: Vec<String>,
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    #[serde(default = "default_true")]
    pub respect_robots: bool,
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u64,
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
    #[serde(default = "default_true")]
    pub same_origin_only: bool,
    #[serde(default = "default_true")]
    pub check_sitemap: bool,
    #[serde(default = "default_max_crawl_secs")]
    pub max_crawl_time_secs: u64,
    #[serde(default = "default_true")]
    pub check_semantics: bool,
    #[serde(default)]
    pub include_patterns: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub custom_headers: Vec<(String, String)>,
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
}

fn default_max_depth() -> u32 {
    10
}
fn default_true() -> bool {
    true
}
fn default_concurrency() -> u32 {
    10
}
fn default_delay_ms() -> u64 {
    100
}
fn default_max_body_size() -> usize {
    5 * 1024 * 1024 // 5MB
}
fn default_max_crawl_secs() -> u64 {
    3600 // 1 hour
}
fn default_request_timeout_ms() -> u64 {
    30000 // 30 seconds
}

impl CrawlConfig {
    pub fn user_agent(&self) -> &str {
        IMPLICIT_USER_AGENT
    }
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            id: None,
            project_id: None,
            seed_urls: Vec::new(),
            max_depth: default_max_depth(),
            respect_robots: default_true(),
            concurrency: default_concurrency(),
            delay_ms: default_delay_ms(),
            max_body_size: default_max_body_size(),
            same_origin_only: true,
            check_sitemap: true,
            max_crawl_time_secs: default_max_crawl_secs(),
            check_semantics: true,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            custom_headers: Vec::new(),
            request_timeout_ms: default_request_timeout_ms(),
            proxy: None,
        }
    }
}
