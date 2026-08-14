use std::collections::HashMap;

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
    /// All response headers of the final document (lowercased names), used by
    /// the security and compliance check groups.
    pub response_headers: HashMap<String, String>,
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
    /// Empty for checks without per-element detail. `default` keeps older
    /// stored audits (which omit the key for empty examples) deserializable.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<crate::crawler::parser::SemanticIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    pub category: String,
    /// 0..100 score, or `None` when the category is skipped for this context
    /// (no applicable checks, e.g. site-level "links" on a page audit). The UI
    /// renders skipped categories as "N/A" instead of a misleading zero.
    pub score: Option<f64>,
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
    /// Supporting measured detail, forwarded from the failing check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    /// Concrete offending elements (up to 5), forwarded from the failing check.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<crate::crawler::parser::SemanticIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoAuditResult {
    /// Overall score in the 0..100 range.
    pub score: f64,
    /// Letter grade A..F derived from `score`.
    pub grade: String,
    /// `default` tolerates audits stored by older engine versions.
    #[serde(default)]
    pub categories: Vec<CategoryResult>,
    #[serde(default)]
    pub checks: Vec<CheckResult>,
    #[serde(default)]
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
            response_headers: Default::default(),
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

        // All weighted categories should be reported (security + compliance
        // added). "links" is included but skipped (score: None) on page audits.
        assert_eq!(result.categories.len(), 11);
        let links = result
            .categories
            .iter()
            .find(|c| c.category == "links")
            .unwrap();
        assert!(links.score.is_none());
        assert_eq!(links.total_checks, 0);
        assert!(
            result
                .categories
                .iter()
                .filter(|c| c.score.is_some())
                .count()
                >= 10
        );
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
            response_headers: Default::default(),
        };
        let result = audit_page(&seo, html, &ctx);
        assert!(result.score < 50.0, "score should be low: {}", result.score);

        let fixes = result.priority_fixes;
        assert!(!fixes.is_empty());
        assert!(fixes.iter().any(|f| f.id == "title_present"));
    }

    #[test]
    fn test_audit_json_without_examples_deserializes() {
        // `examples` is omitted when empty (skip_serializing_if). Stored audits
        // from the current and previous engines must still parse so the site
        // overview can aggregate issues (regression: "score but no problems").
        let json = r#"{
            "score": 64.0,
            "grade": "D",
            "categories": [],
            "checks": [
                {"id":"title_length","category":"meta","severity":"warning","passed":false,"weight":2.0,"message":"Title length: 80 chars","guidance":"Keep titles 30-65 chars","evidence":"80"},
                {"id":"form_labels","category":"accessibility","severity":"error","passed":false,"weight":3.0,"message":"Inputs lack labels","guidance":"Add labels","examples":[{"issue_type":"input_no_label","severity":"error","element":"<input>","message":"x","xpath":"/html/body/input"}]}
            ],
            "priority_fixes": []
        }"#;
        let audit: SeoAuditResult = serde_json::from_str(json).unwrap();
        assert_eq!(audit.checks.len(), 2);
        assert!(audit.checks[0].examples.is_empty());
        assert_eq!(audit.checks[1].examples.len(), 1);
    }

    #[test]
    fn test_img_dimensions_checks_carry_offending_images() {
        // Images without width/height must be reported as concrete examples on
        // the img_dimensions / image_optimization checks (not just a count).
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="description" content="A page used to verify that images missing explicit dimensions are reported individually.">
    <title>Image dimensions test page with a reasonably long title</title>
</head>
<body>
    <main>
        <h1>Image dimensions test</h1>
        <p>This paragraph provides enough content for the page to be considered substantive by the audit engine, with several sentences of meaningful text.</p>
        <p>A second paragraph continues to describe the topic, expanding on the details and giving the page a comfortable amount of readable text.</p>
        <img src="/ok.png" alt="With dimensions" width="800" height="600">
        <img src="/bad1.png" alt="Missing dimensions one">
        <img src="/bad2.png" alt="Missing dimensions two">
    </main>
</body>
</html>"#;
        let url = Url::parse("https://example.com/page").unwrap();
        let parser = SeoParser::new();
        let (seo, _) = parser.parse(html, &url);
        let ctx = AuditContext {
            url: "https://example.com/page".to_string(),
            status_code: 200,
            size_bytes: 4096,
            load_time_ms: 120,
            pagespeed_score: None,
            response_headers: Default::default(),
        };
        let result = audit_page(&seo, html, &ctx);

        for id in ["img_dimensions", "image_optimization"] {
            let check = result.checks.iter().find(|c| c.id == id).unwrap();
            assert!(!check.passed, "{id} should fail");
            assert_eq!(
                check.examples.len(),
                2,
                "{id} should list the 2 offending images"
            );
            for ex in &check.examples {
                assert_eq!(ex.issue_type, "img_no_dimensions");
                assert!(ex.xpath.is_some(), "{id} example should carry an xpath");
                assert!(ex.snippet.is_some(), "{id} example should carry a snippet");
            }
        }
    }
}
