use serde::{Deserialize, Serialize};

use crate::crawler::parser::SeoData;

use super::checks::{self, PageExtras};
use super::priority;
use super::score;

/// Contextual, crawl-time metrics available to the audit engine.
#[derive(Debug, Clone)]
pub struct AuditContext {
    pub url: String,
    pub status_code: u16,
    pub size_bytes: usize,
    pub load_time_ms: u64,
    /// Optional Google PageSpeed (PSI) performance score in the 0..1 range.
    /// When present it is merged into the performance category; when absent
    /// only local heuristics contribute.
    pub pagespeed_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Stable identifier, e.g. "title_present". Used by the frontend for i18n.
    pub id: String,
    /// Category slug: meta, technical, social, accessibility, performance,
    /// ai_readability, sxo.
    pub category: String,
    /// Base importance of the check: error | warning | info.
    pub severity: String,
    pub passed: bool,
    /// Contribution weight of this check within its category.
    pub weight: f64,
    /// Human readable status (English); the frontend localizes known ids.
    pub message: String,
    /// Actionable fix recommendation (English).
    pub guidance: String,
    /// Optional supporting detail (snippet, measured value, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    /// Concrete offending elements (up to 5) that explain why the check failed.
    /// Empty for checks without per-element detail.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<crate::crawler::parser::SemanticIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    pub category: String,
    pub score: f64,
    pub weight: f64,
    pub passed_weight: f64,
    pub total_weight: f64,
    pub passed_checks: usize,
    pub total_checks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityFix {
    pub id: String,
    /// critical | important | minor
    pub priority: String,
    pub message: String,
    pub guidance: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoAuditResult {
    /// Overall score in the 0..100 range.
    pub score: f64,
    /// Letter grade A..F derived from `score`.
    pub grade: String,
    pub categories: Vec<CategoryResult>,
    pub checks: Vec<CheckResult>,
    pub priority_fixes: Vec<PriorityFix>,
}

/// Runs the full SEO audit for a single page from its parsed SEO data, raw
/// HTML and crawl-time context. Cheap enough to run synchronously during the
/// crawl for every page.
pub fn audit_page(seo: &SeoData, html: &str, ctx: &AuditContext) -> SeoAuditResult {
    let extras = PageExtras::extract(html, &ctx.url);
    let checks = checks::run_all(seo, &extras, ctx);
    let (score, grade, categories) = score::compute(&checks);
    let priority_fixes = priority::build(&checks);
    SeoAuditResult {
        score,
        grade,
        categories,
        checks,
        priority_fixes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::parser::SeoParser;
    use url::Url;

    const HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="description" content="A sample page description used for testing the SEO audit engine with enough text to matter.">
    <title>Sample Page For SEO Testing</title>
    <link rel="canonical" href="https://example.com/page">
    <link rel="icon" href="/favicon.ico">
    <script type="application/ld+json">
    {"@context":"https://schema.org","@type":"FAQPage","mainEntity":[
        {"@type":"Question","name":"How does this work?","acceptedAnswer":{"@type":"Answer","text":"It works by analyzing pages."}}
    ]}
    </script>
</head>
<body>
    <header>Site header</header>
    <nav><a href="/">Home</a></nav>
    <main>
        <h1>Sample Page For SEO Testing</h1>
        <h2>How does this feature work?</h2>
        <p>This is the first paragraph of the page. It introduces the topic with a clear and direct answer that helps readers and search engines understand the content quickly.</p>
        <p>Here is a second paragraph that adds more detail about the subject, expands the context, and gives enough words to make the page substantial and readable.</p>
        <p>A third paragraph rounds out the page, summarizing the main points and offering a natural conclusion for the reader.</p>
        <img src="/image.jpg" alt="Sample image" width="1200" height="630">
        <a href="/other">Internal link</a>
        <a href="https://external.example.com/ref">Reference</a>
    </main>
    <footer>Site footer</footer>
</body>
</html>"#;

    #[test]
    fn test_audit_page_detects_signals() {
        let url = Url::parse("https://example.com/page").unwrap();
        let parser = SeoParser::new();
        let (seo, _) = parser.parse(HTML, &url);
        let ctx = AuditContext {
            url: "https://example.com/page".to_string(),
            status_code: 200,
            size_bytes: 4096,
            load_time_ms: 120,
            pagespeed_score: None,
        };
        let result = audit_page(&seo, HTML, &ctx);

        assert!(result.score > 50.0, "score too low: {}", result.score);
        assert!(matches!(result.grade.as_str(), "A" | "B" | "C" | "D" | "F"));

        let by_id = |id: &str| result.checks.iter().find(|c| c.id == id);
        assert!(by_id("title_present").unwrap().passed);
        assert!(by_id("viewport").unwrap().passed);
        assert!(by_id("faq_schema").unwrap().passed);
        assert!(by_id("question_headings").unwrap().passed);
        assert!(by_id("https_used").unwrap().passed);
        assert!(by_id("img_alt").unwrap().passed);

        // 7 categories should be reported.
        assert_eq!(result.categories.len(), 7);
    }

    #[test]
    fn test_audit_page_missing_everything_scores_low() {
        let html = r#"<!DOCTYPE html><html><head></head><body><div><p>x</p></div></body></html>"#;
        let url = Url::parse("http://example.com").unwrap();
        let parser = SeoParser::new();
        let (seo, _) = parser.parse(html, &url);
        let ctx = AuditContext {
            url: "http://example.com".to_string(),
            status_code: 200,
            size_bytes: 512,
            load_time_ms: 10,
            pagespeed_score: None,
        };
        let result = audit_page(&seo, html, &ctx);
        assert!(result.score < 50.0, "score should be low: {}", result.score);

        let fixes = result.priority_fixes;
        assert!(!fixes.is_empty());
        assert!(fixes.iter().any(|f| f.id == "title_present"));
    }
}
