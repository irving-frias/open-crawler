use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    pub id: String,
    pub config_id: String,
    pub project_id: String,
    pub url: String,
    pub status_code: Option<u16>,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub h1: Option<String>,
    pub canonical: Option<String>,
    pub size_bytes: Option<usize>,
    pub load_time_ms: Option<u64>,
    pub is_indexable: Option<bool>,
    pub depth: u32,
    pub parent_url: Option<String>,
    pub crawl_timestamp: String,
    pub links: Vec<PageLink>,
    pub html_lang: Option<String>,
    pub hreflang_json: Option<String>,
    pub semantic_issues_json: Option<String>,
    pub html_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLink {
    pub from_url: String,
    pub to_url: String,
    pub config_id: String,
    pub project_id: String,
    pub link_type: String,
    pub anchor_text: Option<String>,
    pub is_follow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlProgress {
    pub project_id: String,
    pub urls_crawled: u32,
    pub urls_queued: u32,
    pub current_url: String,
    pub errors: u32,
    pub elapsed_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResults {
    pub items: Vec<CrawlResult>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultsFilter {
    pub status_code: Option<u16>,
    pub has_title: Option<bool>,
    pub has_description: Option<bool>,
    pub has_canonical: Option<bool>,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCount {
    pub issue_type: String,
    pub severity: String,
    pub count: u32,
}
