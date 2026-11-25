use std::cmp::Ordering;

use rusqlite::{params, Connection, OptionalExtension};
use tracing::info;

use crate::error::AppError;
use crate::models::{GradeCount, SeoCategoryAvg, SeoIssueCount, SeoOverview};
use crate::seo::SeoAuditResult;

use super::{delete_where_id_in, execute_multi_row_insert, sql_box, CrawlRepo};

/// Stored per-page SEO columns.
#[derive(Debug, Clone)]
pub struct StoredSeoAudit {
    pub score: Option<f64>,
    pub json: Option<String>,
}

/// Deletes the normalized `seo_category_scores` / `seo_check_issues` rows for
/// the given page ids, one statement per table. Re-crawls replace page rows
/// per URL and the replaced ids differ from the new ones, so stale normalized
/// rows must be removed.
/// Deletes the normalized SEO rows of the given pages, keeping the
/// per-project counters (`project_seo_counts`, T14) in sync: the pages'
/// current counts are subtracted before the rows are removed. Callers must
/// call this while the `crawled_pages` rows still exist (their
/// `seo_priority_fix_count` is the old fix count); `save_result` orders its
/// deletes accordingly. If a page row is already gone, its project is read
/// from the normalized rows and only the category/check counts are
/// subtracted.
pub(crate) fn delete_seo_normalized(tx: &Connection, page_ids: &[String]) -> Result<(), AppError> {
    if page_ids.is_empty() {
        return Ok(());
    }
    for page_id in page_ids {
        let (project_id, fix_count): (String, i64) = match tx.query_row(
            "SELECT project_id, seo_priority_fix_count FROM crawled_pages WHERE id = ?1",
            params![page_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ) {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Page row already gone (should not happen in the current
                // call order): fall back to the normalized rows' project and
                // no fix count, so the other two counters stay exact.
                match tx.query_row(
                    "SELECT DISTINCT project_id FROM seo_category_scores WHERE page_id = ?1",
                    params![page_id],
                    |row| row.get::<_, String>(0),
                ) {
                    Ok(p) => (p, 0),
                    Err(_) => continue,
                }
            }
            Err(e) => return Err(e.into()),
        };
        let cat: i64 = tx.query_row(
            "SELECT COUNT(*) FROM seo_category_scores WHERE page_id = ?1",
            params![page_id],
            |row| row.get(0),
        )?;
        let chk: i64 = tx.query_row(
            "SELECT COUNT(*) FROM seo_check_issues WHERE page_id = ?1",
            params![page_id],
            |row| row.get(0),
        )?;
        tx.execute(
            "UPDATE project_seo_counts
             SET category_count = MAX(0, category_count - ?2),
                 check_issue_count = MAX(0, check_issue_count - ?3),
                 priority_fix_total = MAX(0, priority_fix_total - ?4)
             WHERE project_id = ?1",
            params![project_id, cat, chk, fix_count],
        )?;
    }
    for table in ["seo_category_scores", "seo_check_issues"] {
        delete_where_id_in(tx, table, "page_id", page_ids)?;
    }
    Ok(())
}

/// Writes the normalized SEO rows (per-category scores, failing checks and
/// the priority-fix count) for a single page from its stored audit JSON.
/// Called from within the page save transaction so the normalized data is
/// always in sync with `crawled_pages`.
pub(crate) fn save_seo_normalized(
    tx: &Connection,
    project_id: &str,
    page_id: &str,
    seo_audit_json: &Option<String>,
) -> Result<(), AppError> {
    save_seo_normalized_batch(tx, &[(project_id, page_id, seo_audit_json)])
}

/// Batch variant of [`save_seo_normalized`]: one multi-row INSERT per table
/// for the whole batch (chunked under the SQLite bind-parameter limit) plus
/// a single `UPDATE ... CASE` per chunk for the `seo_priority_fix_count`
/// column. Pages with no audit JSON or unparseable JSON contribute no rows,
/// exactly like the single-page writer. Callers must have inserted the
/// `crawled_pages` rows first, since the fix-count UPDATE targets them.
pub(crate) fn save_seo_normalized_batch(
    tx: &Connection,
    pages: &[(&str, &str, &Option<String>)],
) -> Result<(), AppError> {
    let mut cat_rows: Vec<Vec<Box<dyn rusqlite::types::ToSql>>> = Vec::new();
    let mut check_rows: Vec<Vec<Box<dyn rusqlite::types::ToSql>>> = Vec::new();
    let mut fix_counts: Vec<(String, i64)> = Vec::new();
    // Per-project deltas for the materialized counters (T14).
    let mut deltas: std::collections::HashMap<String, (i64, i64, i64)> =
        std::collections::HashMap::new();
    for (project_id, page_id, seo_audit_json) in pages {
        let Some(json) = seo_audit_json else {
            continue;
        };
        let Ok(audit) = serde_json::from_str::<SeoAuditResult>(json) else {
            continue;
        };
        let failed = audit.checks.iter().filter(|c| !c.passed).count() as i64;
        let entry = deltas.entry(project_id.to_string()).or_default();
        entry.0 += audit.categories.len() as i64;
        entry.1 += failed;
        entry.2 += audit.priority_fixes.len() as i64;
        // Every category is persisted, including skipped ones (score None ->
        // NULL): the XLSX "SEO Audit" sheet writes one row per category and
        // `count_seo_rows` counts them from this table, so omitting skipped
        // categories would undercount. `get_seo_overview` filters NULL
        // scores out of its averages, so aggregation is unaffected.
        for cat in &audit.categories {
            cat_rows.push(vec![
                sql_box(page_id.to_string()),
                sql_box(project_id.to_string()),
                sql_box(cat.category.clone()),
                sql_box(cat.score),
            ]);
        }
        for check in audit.checks.iter().filter(|c| !c.passed) {
            let examples_json = if check.examples.is_empty() {
                None
            } else {
                serde_json::to_string(&check.examples).ok()
            };
            check_rows.push(vec![
                sql_box(page_id.to_string()),
                sql_box(project_id.to_string()),
                sql_box(check.category.clone()),
                sql_box(check.severity.clone()),
                sql_box(check.id.clone()),
                sql_box(check.message.clone()),
                sql_box(check.guidance.clone()),
                sql_box(check.evidence.clone()),
                sql_box(examples_json),
            ]);
        }
        fix_counts.push((page_id.to_string(), audit.priority_fixes.len() as i64));
    }

    execute_multi_row_insert(
        tx,
        "INSERT INTO seo_category_scores (page_id, project_id, category, score) VALUES",
        &cat_rows,
    )?;
    execute_multi_row_insert(
        tx,
        "INSERT INTO seo_check_issues
            (page_id, project_id, category, severity, check_id, message, guidance, evidence, examples_json) VALUES",
        &check_rows,
    )?;

    // One UPDATE per chunk: CASE maps each page id to its fix count.
    for chunk in fix_counts.chunks(200) {
        let mut sql = String::from("UPDATE crawled_pages SET seo_priority_fix_count = CASE id");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        for (page_id, count) in chunk {
            sql.push_str(" WHEN ? THEN ?");
            params.push(sql_box(page_id.clone()));
            params.push(sql_box(*count));
        }
        sql.push_str(" END WHERE id IN (");
        sql.push_str(
            &std::iter::repeat_n("?", chunk.len())
                .collect::<Vec<_>>()
                .join(","),
        );
        sql.push(')');
        for (page_id, _) in chunk {
            params.push(sql_box(page_id.clone()));
        }
        tx.execute(
            &sql,
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        )?;
    }

    // Keep the per-project counters in sync (T14). The INSERTs above already
    // succeeded, so a failure here fails the whole transaction like any
    // other write in this function.
    for (project_id, (cats, checks, fixes)) in &deltas {
        tx.execute(
            "INSERT INTO project_seo_counts
                 (project_id, category_count, check_issue_count, priority_fix_total)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(project_id) DO UPDATE SET
                 category_count = category_count + excluded.category_count,
                 check_issue_count = check_issue_count + excluded.check_issue_count,
                 priority_fix_total = priority_fix_total + excluded.priority_fix_total",
            params![project_id, cats, checks, fixes],
        )?;
    }
    Ok(())
}

/// Single-page write inside an already-open transaction. Shared by
/// [`CrawlRepo::update_seo_audit`] (one transaction per call) and
/// [`CrawlRepo::update_seo_audits_batch`] (one transaction for the batch).
fn update_seo_audit_in_tx(
    tx: &Connection,
    project_id: &str,
    page_id: &str,
    score: f64,
    json: Option<&str>,
    response_headers_json: Option<&str>,
) -> Result<(), AppError> {
    tx.execute(
        "UPDATE crawled_pages SET seo_score = ?1, seo_audit_json = ?2, response_headers_json = ?3 WHERE id = ?4",
        params![score, json, response_headers_json, page_id],
    )?;
    // Keep normalized rows in sync after a re-audit.
    delete_seo_normalized(tx, std::slice::from_ref(&page_id.to_string()))?;
    save_seo_normalized(tx, project_id, page_id, &json.map(|s| s.to_string()))?;
    Ok(())
}

/// Max pages per keyword bucket in `compute_seo_clusters`. A keyword shared
/// by more pages is generic noise with no clustering signal; comparing all
/// its pairs is what made the function quadratic (T13: 247 s at 10k worst
/// case, ~7 h extrapolated at 100k).
const MAX_CLUSTER_BUCKET_PAGES: usize = 400;

/// Shared-keyword count of two *sorted, deduplicated* keyword lists
/// (two-pointer intersection, O(|a| + |b|)).
fn count_shared_sorted(a: &[String], b: &[String]) -> usize {
    let (mut i, mut j, mut shared) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            Ordering::Equal => {
                shared += 1;
                i += 1;
                j += 1;
            }
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
        }
    }
    shared
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
        response_headers_json: Option<&str>,
    ) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;
        update_seo_audit_in_tx(&tx, project_id, page_id, score, json, response_headers_json)?;
        tx.commit()?;
        Ok(())
    }

    /// Persists a whole re-audit in **one** transaction: one `UPDATE` per
    /// page plus its normalized rows, committed together. This is what the
    /// bulk re-audit uses — the old per-page `update_seo_audit` call inside
    /// the audit loop was one transaction per page (the N+1 write pattern).
    /// An empty batch is a no-op. On any failure nothing is committed.
    pub fn update_seo_audits_batch(
        &self,
        project_id: &str,
        audits: &[(String, f64, Option<String>, Option<String>)],
    ) -> Result<(), AppError> {
        if audits.is_empty() {
            return Ok(());
        }
        let tx = self.conn.unchecked_transaction()?;
        for (page_id, score, json, headers_json) in audits {
            update_seo_audit_in_tx(
                &tx,
                project_id,
                page_id,
                *score,
                json.as_deref(),
                headers_json.as_deref(),
            )?;
        }
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

        // Site-level link health: computed once per project from the link graph.
        let (link_checks, link_score, link_grade) = {
            let analysis = self.get_link_analysis(project_id)?;
            if analysis.internal_pages == 0 && analysis.total_links == 0 {
                (Vec::new(), None, None)
            } else {
                let checks = crate::seo::checks::run_site_link_checks(&analysis);
                let (score, grade, _) = crate::seo::score::compute(&checks);
                (checks, Some(score), Some(grade))
            }
        };

        let (audited_pages, avg_score, total_fixes): (u32, f64, i64) = self.conn.query_row(
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
                link_checks,
                link_score,
                link_grade,
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

        // Per-category averages across pages. Skipped categories are stored
        // with a NULL score (see `save_seo_normalized`) and must not count
        // towards the average or the page count for the category.
        let mut cat_stmt = self.conn.prepare(
            "SELECT category, AVG(score), COUNT(DISTINCT page_id)
             FROM seo_category_scores
             WHERE project_id = ?1 AND score IS NOT NULL
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
                if let Ok(items) =
                    serde_json::from_str::<Vec<crate::crawler::parser::SemanticIssue>>(&json)
                {
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
            link_checks,
            link_score,
            link_grade,
        })
    }

    /// Groups pages that target the same topic (overlapping top keywords) into
    /// content clusters. Recomputes from scratch for the project: previous
    /// cluster rows are cleared first. Returns the number of clusters found.
    ///
    /// Candidate pairs are only generated between pages that share at least one
    /// keyword (via keyword bucketing), so this avoids an O(n²) scan on large
    /// sites. Pages in the same cluster share at least two keywords with Jaccard
    /// similarity >= 0.6. The canonical URL of each cluster is the page with the
    /// highest SEO score (shortest URL on ties).
    pub fn compute_seo_clusters(&self, project_id: &str) -> Result<u32, AppError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM seo_cluster_members WHERE cluster_id IN (SELECT id FROM seo_clusters WHERE project_id = ?1)", params![project_id])?;
        tx.execute(
            "DELETE FROM seo_clusters WHERE project_id = ?1",
            params![project_id],
        )?;

        let mut stmt = tx.prepare(
            "SELECT id, url, keywords_json, seo_score FROM crawled_pages
             WHERE project_id = ?1 AND keywords_json IS NOT NULL",
        )?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<f64>>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        if rows.len() < 2 {
            tx.commit()?;
            return Ok(0);
        }

        let mut keyword_sets: Vec<Vec<String>> = Vec::with_capacity(rows.len());
        for (_, _, keywords_json, _) in &rows {
            let keywords: Vec<String> =
                serde_json::from_str::<Vec<crate::crawler::parser::Keyword>>(keywords_json)
                    .map(|items| items.into_iter().map(|k| k.keyword).collect())
                    .unwrap_or_default();
            keyword_sets.push(keywords);
        }
        // Sorted + deduplicated: the pair loop and the canonical similarity
        // below both intersect with a two-pointer walk (T13).
        for keywords in keyword_sets.iter_mut() {
            keywords.sort();
            keywords.dedup();
        }

        let n = rows.len();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(parent: &mut Vec<usize>, x: usize) -> usize {
            if parent[x] != x {
                parent[x] = find(parent, parent[x]);
            }
            parent[x]
        }
        fn union(parent: &mut Vec<usize>, a: usize, b: usize) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                parent[rb] = ra;
            }
        }

        // Keyword -> page indices. Pages sharing a keyword become candidates.
        let mut buckets: std::collections::HashMap<&str, Vec<usize>> =
            std::collections::HashMap::new();
        for (i, keywords) in keyword_sets.iter().enumerate() {
            for keyword in keywords {
                buckets.entry(keyword.as_str()).or_default().push(i);
            }
        }
        let mut compared: std::collections::HashSet<(usize, usize)> =
            std::collections::HashSet::new();
        for indices in buckets.values() {
            // A keyword shared by more pages is generic noise with no
            // clustering signal; comparing all its pairs is what made this
            // function quadratic (T13: 247 s at 10k worst case).
            if indices.len() > MAX_CLUSTER_BUCKET_PAGES {
                continue;
            }
            for a in 0..indices.len() {
                for b in (a + 1)..indices.len() {
                    let (ia, ib) = (indices[a], indices[b]);
                    let (lo, hi) = if ia < ib { (ia, ib) } else { (ib, ia) };
                    if !compared.insert((lo, hi)) {
                        continue;
                    }
                    let sa = &keyword_sets[ia];
                    let sb = &keyword_sets[ib];
                    // Upper bound on Jaccard: shared <= min(|a|, |b|) and
                    // union >= max(|a|, |b|). Skip pairs that cannot reach 0.6.
                    let (min_len, max_len) = if sa.len() < sb.len() {
                        (sa.len(), sb.len())
                    } else {
                        (sb.len(), sa.len())
                    };
                    if 5 * min_len < 3 * max_len {
                        continue;
                    }
                    // Minimum shared for Jaccard >= 0.6 (shared/(|a|+|b|-shared)
                    // >= 0.6  <=>  shared >= 3*(|a|+|b|)/8), keeping the
                    // shared >= 2 rule. Two-pointer intersection with early
                    // exit: once the remaining keywords cannot reach the
                    // minimum, the pair is settled.
                    let min_shared = (3 * (sa.len() + sb.len())).div_ceil(8).max(2);
                    let (mut i, mut j, mut shared) = (0, 0, 0);
                    let mut reaches = false;
                    while i < sa.len() && j < sb.len() {
                        match sa[i].cmp(&sb[j]) {
                            Ordering::Equal => {
                                shared += 1;
                                i += 1;
                                j += 1;
                                if shared >= min_shared {
                                    reaches = true;
                                    break;
                                }
                            }
                            Ordering::Less => i += 1,
                            Ordering::Greater => j += 1,
                        }
                        if shared + (sa.len() - i).min(sb.len() - j) < min_shared {
                            break;
                        }
                    }
                    if reaches {
                        union(&mut parent, ia, ib);
                    }
                }
            }
        }

        let mut roots: std::collections::HashMap<usize, Vec<usize>> =
            std::collections::HashMap::new();
        for i in 0..n {
            roots.entry(find(&mut parent, i)).or_default().push(i);
        }

        let mut cluster_count = 0u32;
        {
            let mut insert_cluster = tx.prepare(
                "INSERT INTO seo_clusters (project_id, cluster_key, canonical, member_count, similarity)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            let mut insert_member = tx.prepare(
                "INSERT INTO seo_cluster_members (cluster_id, page_id, url)
                 VALUES (?1, ?2, ?3)",
            )?;
            for members in roots.values() {
                if members.len() < 2 {
                    continue;
                }
                cluster_count += 1;
                // Canonical: highest seo_score, shortest URL on ties.
                let canonical_idx = *members
                    .iter()
                    .max_by(|&&a, &&b| {
                        let score_a = rows[a].3.unwrap_or(0.0);
                        let score_b = rows[b].3.unwrap_or(0.0);
                        score_a
                            .partial_cmp(&score_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then_with(|| rows[b].1.len().cmp(&rows[a].1.len()))
                    })
                    .expect("non-empty cluster");
                let cluster_key = format!("k{}", cluster_count);
                // Average keyword-overlap (Jaccard) of each member vs canonical.
                let canonical_keywords = &keyword_sets[canonical_idx];
                let avg_sim: f64 = {
                    let mut sum = 0.0;
                    for &i in members {
                        let shared = count_shared_sorted(&keyword_sets[i], canonical_keywords);
                        let union_size = keyword_sets[i].len() + canonical_keywords.len() - shared;
                        if union_size > 0 {
                            sum += shared as f64 / union_size as f64;
                        }
                    }
                    sum / members.len() as f64
                };
                insert_cluster.execute(params![
                    project_id,
                    cluster_key,
                    rows[canonical_idx].1,
                    members.len() as i64,
                    avg_sim,
                ])?;
                let cluster_id: i64 = tx.last_insert_rowid();
                for &i in members {
                    insert_member.execute(params![cluster_id, rows[i].0, rows[i].1])?;
                }
            }
        }

        tx.commit()?;
        info!(
            "Computed {} SEO clusters for project {}",
            cluster_count, project_id
        );
        Ok(cluster_count)
    }
}
