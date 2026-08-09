use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppError;
use crate::models::{GradeCount, SeoCategoryAvg, SeoIssueCount, SeoOverview};
use crate::seo::SeoAuditResult;

use super::CrawlRepo;

/// Stored per-page SEO columns.
#[derive(Debug, Clone)]
pub struct StoredSeoAudit {
    pub score: Option<f64>,
    pub json: Option<String>,
}

/// Deletes the normalized `seo_category_scores` / `seo_check_issues` rows for
/// the given page ids. Re-crawls replace page rows per URL and the replaced
/// ids differ from the new ones, so stale normalized rows must be removed.
pub(crate) fn delete_seo_normalized(
    tx: &Connection,
    page_ids: &[String],
) -> Result<(), AppError> {
    if page_ids.is_empty() {
        return Ok(());
    }
    for table in ["seo_category_scores", "seo_check_issues"] {
        let mut stmt = tx.prepare(&format!("DELETE FROM {table} WHERE page_id = ?1"))?;
        for id in page_ids {
            stmt.execute(params![id])?;
        }
    }
    Ok(())
}

/// Writes the normalized SEO rows (per-category scores, failing checks and the
/// priority-fix count) for a single page from its stored audit JSON. Called
/// from within the page save transaction so the normalized data is always in
/// sync with `crawled_pages`.
pub(crate) fn save_seo_normalized(
    tx: &Connection,
    project_id: &str,
    page_id: &str,
    seo_audit_json: &Option<String>,
) -> Result<(), AppError> {
    let Some(json) = seo_audit_json else {
        return Ok(());
    };
    let Ok(audit) = serde_json::from_str::<SeoAuditResult>(json) else {
        return Ok(());
    };

    {
        let mut cat_stmt = tx.prepare(
            "INSERT INTO seo_category_scores (page_id, project_id, category, score)
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        for cat in &audit.categories {
            cat_stmt.execute(params![page_id, project_id, cat.category, cat.score])?;
        }
    }

    {
        let mut chk_stmt = tx.prepare(
            "INSERT INTO seo_check_issues
                (page_id, project_id, category, severity, check_id, message, guidance, evidence, examples_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;
        for check in audit.checks.iter().filter(|c| !c.passed) {
            let examples_json = if check.examples.is_empty() {
                None
            } else {
                serde_json::to_string(&check.examples).ok()
            };
            chk_stmt.execute(params![
                page_id,
                project_id,
                check.category,
                check.severity,
                check.id,
                check.message,
                check.guidance,
                check.evidence,
                examples_json,
            ])?;
        }
    }

    tx.execute(
        "UPDATE crawled_pages SET seo_priority_fix_count = ?1 WHERE id = ?2",
        params![audit.priority_fixes.len() as i64, page_id],
    )?;
    Ok(())
}

impl<'a> CrawlRepo<'a> {
    pub fn get_seo_audit_json(&self, page_id: &str) -> Result<Option<StoredSeoAudit>, AppError> {
        let row = self
            .conn
            .query_row(
                "SELECT seo_score, seo_audit_json FROM crawled_pages WHERE id = ?1",
                params![page_id],
                |row| {
                    Ok(StoredSeoAudit {
                        score: row.get::<_, Option<f64>>(0)?,
                        json: row.get::<_, Option<String>>(1)?,
                    })
                },
            )
            .optional()?;
        Ok(row)
    }

    pub fn update_seo_audit(
        &self,
        project_id: &str,
        page_id: &str,
        score: f64,
        json: Option<&str>,
    ) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE crawled_pages SET seo_score = ?1, seo_audit_json = ?2 WHERE id = ?3",
            params![score, json, page_id],
        )?;
        // Keep normalized rows in sync after a re-audit.
        delete_seo_normalized(&tx, std::slice::from_ref(&page_id.to_string()))?;
        save_seo_normalized(&tx, project_id, page_id, &json.map(|s| s.to_string()))?;
        tx.commit()?;
        Ok(())
    }

    /// Aggregates stored SEO audits into a project-level overview using SQL
    /// over the normalized `seo_category_scores` / `seo_check_issues` tables
    /// (instead of loading every `seo_audit_json` blob and re-parsing it).
    pub fn get_seo_overview(&self, project_id: &str) -> Result<SeoOverview, AppError> {
        let total_pages: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )?;

        let (audited_pages, avg_score, total_fixes): (u32, f64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(AVG(seo_score), 0),
                        COALESCE(SUM(seo_priority_fix_count), 0)
                 FROM crawled_pages
                 WHERE project_id = ?1 AND seo_score IS NOT NULL",
                params![project_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;

        if audited_pages == 0 {
            return Ok(SeoOverview {
                audited_pages: 0,
                total_pages,
                ..Default::default()
            });
        }

        // Grade distribution from the score column.
        let mut grade_stmt = self.conn.prepare(
            "SELECT CASE
                        WHEN seo_score >= 90 THEN 'A'
                        WHEN seo_score >= 80 THEN 'B'
                        WHEN seo_score >= 70 THEN 'C'
                        WHEN seo_score >= 60 THEN 'D'
                        ELSE 'F'
                    END AS grade, COUNT(*)
             FROM crawled_pages
             WHERE project_id = ?1 AND seo_score IS NOT NULL
             GROUP BY grade",
        )?;
        let grade_counts: Vec<GradeCount> = grade_stmt
            .query_map(params![project_id], |row| {
                Ok(GradeCount {
                    grade: row.get(0)?,
                    count: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Per-category averages across pages.
        let mut cat_stmt = self.conn.prepare(
            "SELECT category, AVG(score), COUNT(DISTINCT page_id)
             FROM seo_category_scores
             WHERE project_id = ?1
             GROUP BY category",
        )?;
        let mut category_averages: Vec<SeoCategoryAvg> = cat_stmt
            .query_map(params![project_id], |row| {
                Ok(SeoCategoryAvg {
                    category: row.get(0)?,
                    avg_score: row.get(1)?,
                    pages: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        category_averages.sort_by(|a, b| a.category.cmp(&b.category));

        // Most common failing checks, aggregated across pages.
        struct FailingCheck {
            category: String,
            severity: String,
            check_id: String,
            occurrences: i64,
            message: String,
            guidance: String,
            evidence: Option<String>,
        }

        let mut issue_stmt = self.conn.prepare(
            "SELECT category, severity, check_id, COUNT(*),
                    MAX(message), MAX(guidance), MAX(evidence)
             FROM seo_check_issues
             WHERE project_id = ?1
             GROUP BY category, severity, check_id
             ORDER BY COUNT(*) DESC
             LIMIT 15",
        )?;
        let top_checks: Vec<FailingCheck> = issue_stmt
            .query_map(params![project_id], |row| {
                Ok(FailingCheck {
                    category: row.get(0)?,
                    severity: row.get(1)?,
                    check_id: row.get(2)?,
                    occurrences: row.get(3)?,
                    message: row.get(4)?,
                    guidance: row.get(5)?,
                    evidence: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Sample up to 3 concrete offending elements per failing check.
        let mut top_issues: Vec<SeoIssueCount> = Vec::with_capacity(top_checks.len());
        for failing in top_checks {
            let mut examples = Vec::new();
            let mut ex_stmt = self.conn.prepare(
                "SELECT examples_json FROM seo_check_issues
                 WHERE project_id = ?1 AND check_id = ?2
                   AND examples_json IS NOT NULL
                 LIMIT 3",
            )?;
            let example_rows: Vec<String> = ex_stmt
                .query_map(params![project_id, failing.check_id], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;
            for json in example_rows {
                if let Ok(items) = serde_json::from_str::<Vec<crate::crawler::parser::SemanticIssue>>(
                    &json,
                ) {
                    examples.extend(items);
                }
            }
            examples.truncate(3);

            top_issues.push(SeoIssueCount {
                id: failing.check_id,
                category: failing.category,
                severity: failing.severity,
                occurrences: failing.occurrences as u32,
                message: failing.message,
                guidance: failing.guidance,
                evidence: failing.evidence,
                examples,
            });
        }

        Ok(SeoOverview {
            audited_pages,
            total_pages,
            avg_score: Some(avg_score),
            avg_grade: Some(crate::seo::score::grade_for(avg_score)),
            grade_distribution: grade_counts,
            category_averages,
            top_issues,
            total_fixes: total_fixes as u32,
        })
    }
}
