use std::collections::BTreeMap;

use rusqlite::params;

use crate::error::AppError;

use super::CrawlRepo;

/// One maintenance sweep (T15): delete rows whose parent is gone, across
/// every project-keyed table. Returns per-table deleted counts, so the UI
/// can report what was cleaned. Runs in a single transaction, children
/// before parents.
impl<'a> CrawlRepo<'a> {
    pub fn cleanup_orphans(&self) -> Result<BTreeMap<String, u64>, AppError> {
        // (report key, DELETE statement). Order matters: children first.
        const CLEANUPS: &[(&str, &str)] = &[
            (
                "crawl_snapshot_data",
                "DELETE FROM crawl_snapshot_data WHERE snapshot_id NOT IN (SELECT id FROM crawl_snapshots)",
            ),
            (
                "crawl_snapshots",
                "DELETE FROM crawl_snapshots WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "scheduled_jobs",
                "DELETE FROM scheduled_jobs WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "custom_extractions",
                "DELETE FROM custom_extractions WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "project_alert_settings",
                "DELETE FROM project_alert_settings WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "crawl_config",
                "DELETE FROM crawl_config WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "crawled_pages",
                "DELETE FROM crawled_pages WHERE config_id NOT IN (SELECT id FROM crawl_config)",
            ),
            (
                "page_links",
                "DELETE FROM page_links WHERE config_id NOT IN (SELECT id FROM crawl_config)",
            ),
            (
                "crawl_errors",
                "DELETE FROM crawl_errors WHERE config_id NOT IN (SELECT id FROM crawl_config)",
            ),
            (
                "page_issues",
                "DELETE FROM page_issues WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "page_keywords",
                "DELETE FROM page_keywords WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "seo_category_scores",
                "DELETE FROM seo_category_scores WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "seo_check_issues",
                "DELETE FROM seo_check_issues WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "project_seo_counts",
                "DELETE FROM project_seo_counts WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "crawl_sessions",
                "DELETE FROM crawl_sessions WHERE project_id NOT IN (SELECT id FROM projects)",
            ),
            (
                "crawl_queue",
                "DELETE FROM crawl_queue WHERE session_id NOT IN (SELECT id FROM crawl_sessions)",
            ),
        ];

        let tx = self.conn.unchecked_transaction()?;
        let mut report = BTreeMap::new();
        for (table, sql) in CLEANUPS {
            let deleted = tx.execute(sql, params![])?;
            report.insert(table.to_string(), deleted as u64);
        }
        tx.commit()?;
        Ok(report)
    }
}
