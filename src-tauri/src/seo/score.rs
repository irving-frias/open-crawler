use super::audit::{CategoryResult, CheckResult};

pub const CATEGORY_ORDER: &[&str] = &[
    "meta",
    "technical",
    "links",
    "social",
    "accessibility",
    "semantic_html",
    "performance",
    "ai_readability",
    "sxo",
    "security",
    "compliance",
];

/// Category weights, proportional to the number of checks in each category
/// (94 per-page checks at the time of writing). Sums to exactly 1.0 — enforced
/// by `test_weights_sum_to_one`.
pub const CATEGORY_WEIGHTS: &[(&str, f64)] = &[
    ("meta", 0.16),
    ("technical", 0.13),
    ("sxo", 0.13),
    ("accessibility", 0.11),
    ("performance", 0.09),
    ("security", 0.09),
    ("social", 0.08),
    ("semantic_html", 0.08),
    ("ai_readability", 0.06),
    ("compliance", 0.05),
    ("links", 0.02),
];

fn category_weight(category: &str) -> f64 {
    CATEGORY_WEIGHTS
        .iter()
        .find(|(c, _)| *c == category)
        .map(|(_, w)| *w)
        .unwrap_or(0.05)
}

pub fn grade_for(score: f64) -> String {
    match score {
        s if s >= 90.0 => "A",
        s if s >= 80.0 => "B",
        s if s >= 70.0 => "C",
        s if s >= 60.0 => "D",
        _ => "F",
    }
    .to_string()
}

/// Computes per-category scores, the overall 0-100 score and a letter grade.
/// Categories without any applicable checks are emitted with `score: None`
/// (skipped) so the UI can show "N/A"; they do not contribute to the weighted
/// average and thus do not penalize the page.
pub fn compute(checks: &[CheckResult]) -> (f64, String, Vec<CategoryResult>) {
    let mut categories: Vec<CategoryResult> = Vec::new();

    for category in CATEGORY_ORDER {
        let cat_checks: Vec<&CheckResult> =
            checks.iter().filter(|c| c.category == *category).collect();
        let weight = category_weight(category);
        if cat_checks.is_empty() {
            categories.push(CategoryResult {
                category: category.to_string(),
                score: None,
                weight,
                passed_weight: 0.0,
                total_weight: 0.0,
                passed_checks: 0,
                total_checks: 0,
            });
            continue;
        }
        let total_weight: f64 = cat_checks.iter().map(|c| c.weight).sum();
        let passed_weight: f64 = cat_checks
            .iter()
            .filter(|c| c.passed)
            .map(|c| c.weight)
            .sum();
        let score = if total_weight > 0.0 {
            passed_weight / total_weight * 100.0
        } else {
            100.0
        };
        categories.push(CategoryResult {
            category: category.to_string(),
            score: Some(score),
            weight,
            passed_weight,
            total_weight,
            passed_checks: cat_checks.iter().filter(|c| c.passed).count(),
            total_checks: cat_checks.len(),
        });
    }

    let present_weight: f64 = categories
        .iter()
        .filter(|c| c.score.is_some())
        .map(|c| c.weight)
        .sum();
    let weighted = categories
        .iter()
        .filter_map(|c| c.score.map(|s| s * c.weight))
        .sum::<f64>();
    let overall = if present_weight > 0.0 {
        weighted / present_weight
    } else {
        0.0
    };
    let overall = overall.clamp(0.0, 100.0);

    (overall, grade_for(overall), categories)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weights_sum_to_one_and_order_covered() {
        let total: f64 = CATEGORY_WEIGHTS.iter().map(|(_, w)| w).sum();
        assert!(
            (total - 1.0).abs() < 1e-9,
            "CATEGORY_WEIGHTS must sum to 1.0, got {total}"
        );
        // Every weight must be positive and no category may be duplicated.
        let mut seen = std::collections::HashSet::new();
        for (cat, w) in CATEGORY_WEIGHTS {
            assert!(*w > 0.0, "category {cat} has a non-positive weight {w}");
            assert!(seen.insert(*cat), "duplicate category in weights: {cat}");
        }
        // Every category in CATEGORY_ORDER must have a weight.
        for cat in CATEGORY_ORDER {
            assert!(
                CATEGORY_WEIGHTS.iter().any(|(c, _)| c == cat),
                "CATEGORY_ORDER entry {cat} has no weight"
            );
        }
    }

    #[test]
    fn test_grade_boundaries() {
        assert_eq!(grade_for(95.0), "A");
        assert_eq!(grade_for(89.9), "B");
        assert_eq!(grade_for(80.0), "B");
        assert_eq!(grade_for(70.0), "C");
        assert_eq!(grade_for(60.0), "D");
        assert_eq!(grade_for(59.9), "F");
    }

    fn ck(category: &str, severity: &str, passed: bool, weight: f64) -> CheckResult {
        CheckResult {
            id: "test".to_string(),
            category: category.to_string(),
            severity: severity.to_string(),
            passed,
            weight,
            message: "m".to_string(),
            guidance: "g".to_string(),
            evidence: None,
            examples: Vec::new(),
        }
    }

    #[test]
    fn test_compute_all_passed_is_100() {
        let checks = vec![
            ck("meta", "error", true, 3.0),
            ck("technical", "warning", true, 2.0),
            ck("accessibility", "error", true, 3.0),
        ];
        let (score, grade, categories) = compute(&checks);
        assert!((score - 100.0).abs() < 0.001, "score: {score}");
        assert_eq!(grade, "A");
        // All weighted categories are emitted; categories without checks are
        // skipped (score: None) and do not drag the overall score down.
        assert_eq!(categories.len(), CATEGORY_ORDER.len());
        assert!(categories
            .iter()
            .all(|c| c.score == Some(100.0) || c.score.is_none()));
        let skipped: Vec<_> = categories.iter().filter(|c| c.total_checks == 0).collect();
        assert!(
            !skipped.is_empty(),
            "expected at least one skipped category"
        );
        assert!(skipped.iter().all(|c| c.score.is_none()));
    }

    #[test]
    fn test_compute_all_failed_is_zero() {
        let checks = vec![
            ck("meta", "error", false, 3.0),
            ck("technical", "warning", false, 2.0),
        ];
        let (score, grade, categories) = compute(&checks);
        assert_eq!(score, 0.0);
        assert_eq!(grade, "F");
        let present: Vec<_> = categories.iter().filter(|c| c.score.is_some()).collect();
        assert_eq!(present.len(), 2, "meta and technical should be scored");
    }

    #[test]
    fn test_category_score_uses_weights() {
        let checks = vec![
            ck("meta", "error", true, 3.0),
            ck("meta", "warning", false, 2.0),
            ck("meta", "info", true, 1.0),
        ];
        let (_, _, categories) = compute(&checks);
        let meta = categories.iter().find(|c| c.category == "meta").unwrap();
        assert!((meta.score.unwrap() - 66.6666).abs() < 0.001); // 4 passed-weight / 6 total
        assert_eq!(meta.passed_checks, 2);
        assert_eq!(meta.total_checks, 3);
    }

    #[test]
    fn test_every_weighted_category_has_checks() {
        // Every category with a weight must actually be produced by run_all for
        // a typical page, otherwise the weight silently no-ops. "links" is the
        // exception: it is site-level (see run_site_link_checks), not per-page.
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="description" content="A page used to assert that every weighted SEO category maps to at least one real check.">
    <title>Category coverage test page</title>
    <link rel="canonical" href="https://example.com/page">
</head>
<body>
    <header>Header</header>
    <nav><a href="/">Home</a></nav>
    <main>
        <h1>Category coverage</h1>
        <h2>Section one</h2>
        <p>This paragraph provides enough words for the audit to measure readability and paragraph structure, with several complete sentences.</p>
        <p>A second paragraph expands the topic and keeps the page substantial enough for the content checks to run.</p>
        <img src="/img.png" alt="Coverage image" width="800" height="600">
    </main>
    <footer>Footer</footer>
</body>
</html>"#;
        let extras = crate::seo::checks::PageExtras::extract(html, "https://example.com/page");
        let url = url::Url::parse("https://example.com/page").unwrap();
        let parser = crate::crawler::parser::SeoParser::new();
        let (seo, _) = parser.parse(html, &url);
        let ctx = crate::seo::AuditContext {
            url: "https://example.com/page".to_string(),
            status_code: 200,
            size_bytes: 4096,
            load_time_ms: 120,
            pagespeed_score: None,
            response_headers: Default::default(),
        };
        let checks = crate::seo::checks::run_all(&seo, &extras, &ctx);
        let mut produced: std::collections::HashSet<&str> =
            checks.iter().map(|c| c.category.as_str()).collect();
        produced.insert("links"); // site-level category, covered separately.
        for (cat, _) in CATEGORY_WEIGHTS {
            assert!(
                produced.contains(cat),
                "category {cat} has a weight but no per-page check produced it"
            );
        }
    }
}
