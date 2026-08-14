use serde::{Deserialize, Serialize};

/// Aggregate SEO stats across a project's crawled pages.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeoOverview {
    /// Number of pages that have been audited (seo_score present).
    pub audited_pages: u32,
    /// Total pages in the project (audited or not).
    pub total_pages: u32,
    /// Average SEO score across audited pages (0..100).
    pub avg_score: Option<f64>,
    /// Overall letter grade for the average score (A..F).
    pub avg_grade: Option<String>,
    /// Number of audited pages per letter grade.
    pub grade_distribution: Vec<GradeCount>,
    /// Average score per category across audited pages.
    pub category_averages: Vec<SeoCategoryAvg>,
    /// Most common failing checks across all audited pages.
    pub top_issues: Vec<SeoIssueCount>,
    /// Total priority fixes across all audited pages.
    pub total_fixes: u32,
    /// Site-level link health checks (orphans, dead ends, anchor quality,
    /// link-nature balance) computed from the crawl's `page_links` graph.
    #[serde(default)]
    pub link_checks: Vec<crate::seo::audit::CheckResult>,
    /// 0..100 score derived from `link_checks` (None when there is no link
    /// data yet, e.g. before the first crawl completes).
    #[serde(default)]
    pub link_score: Option<f64>,
    /// Letter grade derived from `link_score`.
    #[serde(default)]
    pub link_grade: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeCount {
    pub grade: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoCategoryAvg {
    pub category: String,
    pub avg_score: f64,
    pub pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoIssueCount {
    pub id: String,
    pub category: String,
    pub severity: String,
    pub occurrences: u32,
    pub message: String,
    pub guidance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    /// Concrete offending elements (up to 3) sampled from failing pages, so the
    /// overview can point at real markup instead of a bare check name.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<crate::crawler::parser::SemanticIssue>,
}
