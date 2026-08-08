use rusqlite::params;
use rusqlite::OptionalExtension;

use crate::crawler::parser::SemanticIssue;
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
        page_id: &str,
        score: f64,
        json: Option<&str>,
    ) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE crawled_pages SET seo_score = ?1, seo_audit_json = ?2 WHERE id = ?3",
            params![score, json, page_id],
        )?;
        Ok(())
    }

    /// Aggregates stored SEO audits into a project-level overview: average
    /// score, grade distribution, per-category averages and the most common
    /// failing checks.
    pub fn get_seo_overview(&self, project_id: &str) -> Result<SeoOverview, AppError> {
        let total_pages: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )?;

        let mut stmt = self.conn.prepare(
            "SELECT seo_score, seo_audit_json FROM crawled_pages
             WHERE project_id = ?1 AND seo_score IS NOT NULL",
        )?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok((row.get::<_, f64>(0)?, row.get::<_, Option<String>>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if rows.is_empty() {
            return Ok(SeoOverview {
                audited_pages: 0,
                total_pages,
                ..Default::default()
            });
        }

        let audited_pages = rows.len() as u32;

        let mut score_sum = 0.0;
        let mut grade_counts: std::collections::BTreeMap<String, u32> =
            std::collections::BTreeMap::new();
        let mut category_scores: std::collections::BTreeMap<String, (f64, u32)> =
            std::collections::BTreeMap::new();
        type IssueCountMap = std::collections::BTreeMap<
            (String, String, String),
            (u32, String, String, Option<String>, Vec<SemanticIssue>),
        >;
        let mut issue_counts: IssueCountMap = IssueCountMap::new();
        let mut total_fixes: u32 = 0;

        for (score, json) in &rows {
            score_sum += score;
            let grade = crate::seo::score::grade_for(*score);
            *grade_counts.entry(grade).or_insert(0) += 1;

            let Some(json) = json else {
                continue;
            };
            let Ok(audit) = serde_json::from_str::<SeoAuditResult>(json) else {
                continue;
            };

            for cat in &audit.categories {
                let entry = category_scores
                    .entry(cat.category.clone())
                    .or_insert((0.0, 0));
                entry.0 += cat.score;
                entry.1 += 1;
            }
            for check in &audit.checks {
                if !check.passed {
                    let entry = issue_counts
                        .entry((
                            check.category.clone(),
                            check.severity.clone(),
                            check.id.clone(),
                        ))
                        .or_insert_with(|| {
                            (
                                0,
                                check.message.clone(),
                                check.guidance.clone(),
                                check.evidence.clone(),
                                Vec::new(),
                            )
                        });
                    entry.0 += 1;
                    for example in &check.examples {
                        if entry.4.len() < 3 {
                            entry.4.push(example.clone());
                        }
                    }
                }
            }
            total_fixes += audit.priority_fixes.len() as u32;
        }

        let avg_score = score_sum / audited_pages as f64;

        let grade_distribution = grade_counts
            .into_iter()
            .map(|(grade, count)| GradeCount { grade, count })
            .collect();

        let mut category_averages: Vec<SeoCategoryAvg> = category_scores
            .into_iter()
            .map(|(category, (sum, pages))| SeoCategoryAvg {
                category,
                avg_score: sum / pages as f64,
                pages,
            })
            .collect();
        category_averages.sort_by(|a, b| a.category.cmp(&b.category));

        let mut top_issues: Vec<SeoIssueCount> = issue_counts
            .into_iter()
            .map(
                |(
                    (category, severity, id),
                    (occurrences, message, guidance, evidence, examples),
                )| SeoIssueCount {
                    id,
                    category,
                    severity,
                    occurrences,
                    message,
                    guidance,
                    evidence,
                    examples,
                },
            )
            .collect();
        top_issues.sort_by_key(|b| std::cmp::Reverse(b.occurrences));
        top_issues.truncate(15);

        Ok(SeoOverview {
            audited_pages,
            total_pages,
            avg_score: Some(avg_score),
            avg_grade: Some(crate::seo::score::grade_for(avg_score)),
            grade_distribution,
            category_averages,
            top_issues,
            total_fixes,
        })
    }
}
