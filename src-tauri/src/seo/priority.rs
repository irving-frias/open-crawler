use super::audit::{CheckResult, PriorityFix};

/// Rank failed checks into an actionable, ordered fix list.
/// critical ← severity error, important ← warning, minor ← info.
pub fn build(checks: &[CheckResult]) -> Vec<PriorityFix> {
    fn rank(priority: &str) -> u8 {
        match priority {
            "critical" => 0,
            "important" => 1,
            "minor" => 2,
            _ => 3,
        }
    }

    let mut fixes: Vec<PriorityFix> = checks
        .iter()
        .filter(|c| !c.passed)
        .map(|c| {
            let priority = match c.severity.as_str() {
                "error" => "critical",
                "warning" => "important",
                _ => "minor",
            };
            PriorityFix {
                id: c.id.clone(),
                priority: priority.to_string(),
                message: c.message.clone(),
                guidance: c.guidance.clone(),
                category: c.category.clone(),
            }
        })
        .collect();

    fixes.sort_by(|a, b| {
        rank(&a.priority)
            .cmp(&rank(&b.priority))
            .then(a.category.cmp(&b.category))
    });

    fixes.truncate(12);
    fixes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ck(id: &str, category: &str, severity: &str, passed: bool) -> CheckResult {
        CheckResult {
            id: id.to_string(),
            category: category.to_string(),
            severity: severity.to_string(),
            passed,
            weight: 1.0,
            message: "m".to_string(),
            guidance: "g".to_string(),
            evidence: None,
            examples: Vec::new(),
        }
    }

    #[test]
    fn test_ranking_orders_by_severity() {
        let checks = vec![
            ck("a", "meta", "info", false),
            ck("b", "meta", "warning", false),
            ck("c", "meta", "error", false),
        ];
        let fixes = build(&checks);
        assert_eq!(fixes[0].id, "c");
        assert_eq!(fixes[0].priority, "critical");
        assert_eq!(fixes[1].id, "b");
        assert_eq!(fixes[1].priority, "important");
        assert_eq!(fixes[2].id, "a");
        assert_eq!(fixes[2].priority, "minor");
    }

    #[test]
    fn test_passed_checks_excluded() {
        let checks = vec![
            ck("ok", "meta", "error", true),
            ck("bad", "meta", "warning", false),
        ];
        let fixes = build(&checks);
        assert_eq!(fixes.len(), 1);
        assert_eq!(fixes[0].id, "bad");
    }
}
