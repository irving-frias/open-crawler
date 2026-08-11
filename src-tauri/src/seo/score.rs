use super::audit::{CategoryResult, CheckResult};

pub const CATEGORY_ORDER: &[&str] = &[
    "meta",
    "technical",
    "social",
    "accessibility",
    "semantic_html",
    "performance",
    "ai_readability",
    "sxo",
];

pub const CATEGORY_WEIGHTS: &[(&str, f64)] = &[
    ("meta", 0.25),
    ("technical", 0.18),
    ("accessibility", 0.12),
    ("semantic_html", 0.12),
    ("social", 0.07),
    ("performance", 0.10),
    ("ai_readability", 0.08),
    ("sxo", 0.08),
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
/// Categories without any applicable checks are excluded from the weighted
/// average so they do not penalize the page.
pub fn compute(checks: &[CheckResult]) -> (f64, String, Vec<CategoryResult>) {
    let mut categories: Vec<CategoryResult> = Vec::new();

    for category in CATEGORY_ORDER {
        let cat_checks: Vec<&CheckResult> =
            checks.iter().filter(|c| c.category == *category).collect();
        if cat_checks.is_empty() {
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
            score,
            weight: category_weight(category),
            passed_weight,
            total_weight,
            passed_checks: cat_checks.iter().filter(|c| c.passed).count(),
            total_checks: cat_checks.len(),
        });
    }

    let present_weight: f64 = categories.iter().map(|c| c.weight).sum();
    let weighted = categories.iter().map(|c| c.score * c.weight).sum::<f64>();
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
        assert_eq!(categories.len(), 3);
        assert!(categories.iter().all(|c| c.score == 100.0));
    }

    #[test]
    fn test_compute_all_failed_is_zero() {
        let checks = vec![
            ck("meta", "error", false, 3.0),
            ck("technical", "warning", false, 2.0),
        ];
        let (score, grade, _) = compute(&checks);
        assert_eq!(score, 0.0);
        assert_eq!(grade, "F");
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
        assert!((meta.score - 66.6666).abs() < 0.001); // 4 passed-weight / 6 total
        assert_eq!(meta.passed_checks, 2);
        assert_eq!(meta.total_checks, 3);
    }
}
