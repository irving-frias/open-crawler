use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    pub id: String,
    pub config_id: String,
    pub project_id: String,
    pub url: String,
    pub status_code: Option<u16>,
    /// True when the page responded with a security/challenge page (e.g.
    /// Cloudflare) rather than an actual error; excluded from broken counts.
    pub blocked: bool,
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
    pub readability_score: Option<f64>,
    pub content_hash: Option<String>,
    pub duplicate_group_id: Option<i64>,
    pub keywords_json: Option<String>,
    pub og_json: Option<String>,
    pub pagespeed_score: Option<f64>,
    pub pagespeed_json: Option<String>,
    /// 0-100 SEO audit score computed during the crawl.
    pub seo_score: Option<f64>,
    /// Serialized `SeoAuditResult` with per-check details and priority fixes.
    pub seo_audit_json: Option<String>,
    /// Original URL before the redirect chain that reached this page, when any.
    #[serde(default)]
    pub redirect_from_url: Option<String>,
    /// Final response headers (lowercased names) serialized as JSON, kept so
    /// security/compliance checks can be recomputed offline and re-audits stay
    /// faithful to what was actually served.
    #[serde(default)]
    pub response_headers_json: Option<String>,
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
    /// Raw `rel` attribute tokens (lowercased), e.g. ["nofollow", "ugc"].
    #[serde(default)]
    pub rel_tokens: Vec<String>,
    #[serde(default)]
    pub is_sponsored: bool,
    #[serde(default)]
    pub is_ugc: bool,
    /// Whether both endpoints share the same host (derived at crawl time).
    #[serde(default)]
    pub is_internal: bool,
}

/// One hop of an HTTP redirect chain captured while fetching a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectHop {
    pub from_url: String,
    pub to_url: String,
    pub status_code: u16,
}

/// Redirect metadata for a single crawled page. The page id references the
/// `crawled_pages` row that was reached after following `chain`.
#[derive(Debug, Clone)]
pub struct RedirectRecord {
    pub page_id: String,
    pub project_id: String,
    pub redirect_from_url: Option<String>,
    pub chain: Vec<RedirectHop>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteTreeNode {
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<u16>,
    pub depth: u32,
    pub has_children: bool,
    pub issue_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteTreeFullNode {
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<u16>,
    pub depth: u32,
    pub issue_count: u32,
    pub has_children: bool,
    pub children: Vec<SiteTreeFullNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBucket {
    pub status: u16,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_pages: u32,
    pub indexed_pages: u32,
    pub broken_pages: u32,
    pub blocked_pages: u32,
    pub avg_load_ms: f64,
    pub avg_size_bytes: f64,
    pub avg_readability: Option<f64>,
    pub avg_seo_score: Option<f64>,
    pub duplicate_count: u32,
    pub missing_title_count: u32,
    pub missing_description_count: u32,
    pub missing_h1_count: u32,
    pub status_distribution: Vec<StatusBucket>,
    pub top_issues: Vec<IssueCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlSnapshot {
    pub id: String,
    pub project_id: String,
    pub snapshot_time: String,
    pub total_pages: u32,
    pub indexed_pages: u32,
    pub broken_pages: u32,
    pub avg_load_ms: f64,
    pub avg_size_bytes: f64,
    pub avg_readability: Option<f64>,
    pub avg_seo_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotStats {
    pub total_pages: u32,
    pub indexed_pages: u32,
    pub broken_pages: u32,
    pub avg_load_ms: f64,
    pub avg_size_bytes: f64,
    pub avg_readability: Option<f64>,
    pub avg_seo_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlFieldDiff {
    pub field: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedUrl {
    pub url: String,
    pub diffs: Vec<UrlFieldDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    pub new_urls: Vec<String>,
    pub removed_urls: Vec<String>,
    pub changed_urls: Vec<ChangedUrl>,
    pub unchanged_count: u32,
    pub before: SnapshotStats,
    pub after: SnapshotStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroupUrl {
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub id: i64,
    pub size: u32,
    pub urls: Vec<DuplicateGroupUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDetail {
    pub page: CrawlResult,
    pub links: Vec<PageLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordAggregate {
    pub keyword: String,
    pub count: u64,
    pub pages: u32,
}
