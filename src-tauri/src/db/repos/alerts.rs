use rusqlite::{params, Row};

use crate::error::AppError;
use crate::models::alerts::{severity_meets_threshold, ALERT_SAMPLE_LIMIT};
use crate::models::{AlertSettings, NewErrors, NewFailedCheck};

use super::CrawlRepo;

/// Rows scanned per chunk when diffing failed checks, so a large pair of
/// snapshots never materializes two full check maps in memory.
const DIFF_CHUNK_SIZE: i64 = 500;

/// Parses a snapshot `failed_checks_json` value (`{ check_id: severity }`).
/// NULL (snapshots taken before migration 019) or malformed JSON means "no
/// failed checks".
fn parse_failed_checks(json: &Option<String>) -> std::collections::HashMap<String, String> {
    json.as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

impl<'a> CrawlRepo<'a> {
    fn row_to_alert_settings(row: &Row<'_>) -> rusqlite::Result<AlertSettings> {
        let channels_json: String = row.get(2)?;
        let channels: Vec<String> = serde_json::from_str(&channels_json).unwrap_or_default();
        Ok(AlertSettings {
            project_id: row.get(0)?,
            enabled: row.get::<_, i64>(1)? != 0,
            channels,
            min_severity: row.get(3)?,
            quiet_start: row.get(4)?,
            quiet_end: row.get(5)?,
            last_alerted_snapshot_id: row.get(6)?,
        })
    }

    const ALERT_COLUMNS: &'static str = "project_id, enabled, channels_json, min_severity, quiet_start, quiet_end, last_alerted_snapshot_id";

    /// Alert settings for a project. A project without a stored row gets the
    /// defaults (alerts on, desktop channel, `warning` threshold).
    pub fn get_alert_settings(&self, project_id: &str) -> Result<AlertSettings, AppError> {
        let result = self.conn.query_row(
            &format!(
                "SELECT {} FROM project_alert_settings WHERE project_id = ?1",
                Self::ALERT_COLUMNS
            ),
            params![project_id],
            Self::row_to_alert_settings,
        );
        match result {
            Ok(settings) => Ok(settings.normalized()),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AlertSettings::default_for(project_id)),
            Err(e) => Err(e.into()),
        }
    }

    /// Inserts or replaces the project's alert settings (normalized first).
    /// Preserves `last_alerted_snapshot_id` when the caller leaves it empty,
    /// so saving settings from the UI never resets the dedup marker.
    pub fn save_alert_settings(&self, settings: &AlertSettings) -> Result<AlertSettings, AppError> {
        let settings = settings.clone().normalized();
        let channels_json = serde_json::to_string(&settings.channels)?;
        let previous = self.get_alert_settings(&settings.project_id)?;
        let last_alerted = settings
            .last_alerted_snapshot_id
            .clone()
            .or(previous.last_alerted_snapshot_id);
        self.conn.execute(
            "INSERT INTO project_alert_settings (project_id, enabled, channels_json, min_severity, quiet_start, quiet_end, last_alerted_snapshot_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(project_id) DO UPDATE SET
               enabled = excluded.enabled,
               channels_json = excluded.channels_json,
               min_severity = excluded.min_severity,
               quiet_start = excluded.quiet_start,
               quiet_end = excluded.quiet_end,
               last_alerted_snapshot_id = excluded.last_alerted_snapshot_id",
            params![
                settings.project_id,
                settings.enabled as i64,
                channels_json,
                settings.min_severity,
                settings.quiet_start,
                settings.quiet_end,
                last_alerted,
            ],
        )?;
        self.get_alert_settings(&settings.project_id)
    }

    /// Records the newest snapshot an alert was sent for, so the same
    /// errors are not notified again on the next scheduled run.
    pub fn set_last_alerted_snapshot(
        &self,
        project_id: &str,
        snapshot_id: &str,
    ) -> Result<(), AppError> {
        // Upsert a defaults row when the project has no settings yet, so the
        // marker survives even if the user never opened the alert settings.
        let mut settings = self.get_alert_settings(project_id)?;
        settings.last_alerted_snapshot_id = Some(snapshot_id.to_string());
        self.save_alert_settings(&settings)?;
        Ok(())
    }

    /// Errors that appeared between two snapshots of the same project
    /// (`snapshot_a` older, `snapshot_b` newer), read only from stored
    /// snapshot data — no page is re-downloaded and no audit is re-run.
    ///
    /// * New error URLs: pages whose status in B is >= 400 while in A it
    ///   was < 400 or the page did not exist yet.
    /// * New failed checks: checks present in B's `failed_checks_json` but
    ///   not in A's for the same URL, filtered by `min_severity`.
    /// * New broken links: distinct `page_links` targets that are error
    ///   pages in B but were not error pages in A, linked from a page that
    ///   exists in B.
    pub fn compute_new_errors(
        &self,
        project_id: &str,
        snapshot_a: &str,
        snapshot_b: &str,
        min_severity: &str,
    ) -> Result<NewErrors, AppError> {
        let mut out = NewErrors::default();

        // --- New error URLs -----------------------------------------------
        let new_error_where = "b.status_code >= 400 AND NOT EXISTS (
            SELECT 1 FROM crawl_snapshot_data a
            WHERE a.snapshot_id = ?1 AND a.url = b.url AND a.status_code >= 400
        )";
        let total: i64 = self.conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM crawl_snapshot_data b
                 WHERE b.snapshot_id = ?2 AND {new_error_where}"
            ),
            params![snapshot_a, snapshot_b],
            |row| row.get(0),
        )?;
        out.new_error_url_total = total as u32;
        {
            let mut stmt = self.conn.prepare(&format!(
                "SELECT b.url FROM crawl_snapshot_data b
                 WHERE b.snapshot_id = ?2 AND {new_error_where}
                 ORDER BY b.url LIMIT ?3"
            ))?;
            out.new_error_urls = stmt
                .query_map(
                    params![snapshot_a, snapshot_b, ALERT_SAMPLE_LIMIT as i64],
                    |row| row.get::<_, String>(0),
                )?
                .collect::<Result<Vec<_>, _>>()?;
        }

        // --- New failed checks (chunked over snapshot B) ------------------
        let mut offset: i64 = 0;
        loop {
            let mut stmt = self.conn.prepare(
                "SELECT url, failed_checks_json FROM crawl_snapshot_data
                 WHERE snapshot_id = ?1 ORDER BY url LIMIT ?2 OFFSET ?3",
            )?;
            let rows_b: Vec<(String, Option<String>)> = stmt
                .query_map(params![snapshot_b, DIFF_CHUNK_SIZE, offset], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            if rows_b.is_empty() {
                break;
            }
            offset += rows_b.len() as i64;

            let urls: Vec<&str> = rows_b.iter().map(|(u, _)| u.as_str()).collect();
            let placeholders = urls.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let mut a_params: Vec<Box<dyn rusqlite::types::ToSql>> =
                Vec::with_capacity(urls.len() + 1);
            a_params.push(Box::new(snapshot_a.to_string()));
            for url in &urls {
                a_params.push(Box::new(url.to_string()));
            }
            let mut stmt_a = self.conn.prepare(&format!(
                "SELECT url, failed_checks_json FROM crawl_snapshot_data
                 WHERE snapshot_id = ?1 AND url IN ({placeholders})"
            ))?;
            let rows_a: std::collections::HashMap<
                String,
                std::collections::HashMap<String, String>,
            > = stmt_a
                .query_map(
                    rusqlite::params_from_iter(a_params.iter().map(|p| p.as_ref())),
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            parse_failed_checks(&row.get::<_, Option<String>>(1)?),
                        ))
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .collect();

            for (url, json_b) in &rows_b {
                let map_b = parse_failed_checks(json_b);
                let empty = std::collections::HashMap::new();
                let map_a = rows_a.get(url).unwrap_or(&empty);
                for (check_id, severity) in &map_b {
                    if !map_a.contains_key(check_id)
                        && severity_meets_threshold(severity, min_severity)
                    {
                        out.new_failed_check_total += 1;
                        if out.new_failed_checks.len() < ALERT_SAMPLE_LIMIT as usize {
                            out.new_failed_checks.push(NewFailedCheck {
                                url: url.clone(),
                                check_id: check_id.clone(),
                                severity: severity.clone(),
                            });
                        }
                    }
                }
            }
            if (rows_b.len() as i64) < DIFF_CHUNK_SIZE {
                break;
            }
        }
        out.new_failed_checks
            .sort_by(|a, b| a.url.cmp(&b.url).then(a.check_id.cmp(&b.check_id)));

        // --- New broken links ----------------------------------------------
        let broken_where = "b.status_code >= 400
            AND l.project_id = ?3
            AND l.from_url IN (SELECT url FROM crawl_snapshot_data WHERE snapshot_id = ?2)
            AND NOT EXISTS (
                SELECT 1 FROM crawl_snapshot_data a
                WHERE a.snapshot_id = ?1 AND a.url = l.to_url AND a.status_code >= 400
            )";
        let total: i64 = self.conn.query_row(
            &format!(
                "SELECT COUNT(DISTINCT l.to_url) FROM page_links l
                 JOIN crawl_snapshot_data b ON b.snapshot_id = ?2 AND b.url = l.to_url
                 WHERE {broken_where}"
            ),
            params![snapshot_a, snapshot_b, project_id],
            |row| row.get(0),
        )?;
        out.new_broken_link_total = total as u32;
        {
            let mut stmt = self.conn.prepare(&format!(
                "SELECT DISTINCT l.to_url FROM page_links l
                 JOIN crawl_snapshot_data b ON b.snapshot_id = ?2 AND b.url = l.to_url
                 WHERE {broken_where}
                 ORDER BY l.to_url LIMIT ?4"
            ))?;
            out.new_broken_links = stmt
                .query_map(
                    params![
                        snapshot_a,
                        snapshot_b,
                        project_id,
                        ALERT_SAMPLE_LIMIT as i64
                    ],
                    |row| row.get::<_, String>(0),
                )?
                .collect::<Result<Vec<_>, _>>()?;
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;
    use rusqlite::Connection;

    fn repo() -> CrawlRepo<'static> {
        let conn = Box::leak(Box::new(Connection::open_in_memory().unwrap()));
        run_migrations(conn).unwrap();
        conn.execute_batch(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('p1', 'P1', datetime('now'), datetime('now'));",
        )
        .unwrap();
        CrawlRepo::new(conn, None)
    }

    #[test]
    fn test_alert_settings_defaults_for_unknown_project() {
        let repo = repo();
        let settings = repo.get_alert_settings("p1").unwrap();
        assert!(settings.enabled);
        assert_eq!(settings.channels, vec!["desktop".to_string()]);
        assert_eq!(settings.min_severity, "warning");
        assert_eq!(settings.quiet_start, None);
        assert_eq!(settings.last_alerted_snapshot_id, None);
    }

    #[test]
    fn test_alert_settings_roundtrip_and_dedup_marker() {
        let repo = repo();
        let settings = AlertSettings {
            project_id: "p1".to_string(),
            enabled: true,
            channels: vec!["desktop".to_string(), "telegram".to_string()],
            min_severity: "error".to_string(),
            quiet_start: Some("22:00".to_string()),
            quiet_end: Some("07:00".to_string()),
            last_alerted_snapshot_id: None,
        };
        let saved = repo.save_alert_settings(&settings).unwrap();
        assert_eq!(saved.channels.len(), 2);
        assert_eq!(saved.min_severity, "error");
        assert_eq!(saved.quiet_start.as_deref(), Some("22:00"));

        // Saving again without the marker preserves the stored one.
        repo.set_last_alerted_snapshot("p1", "snap-1").unwrap();
        let reloaded = repo.get_alert_settings("p1").unwrap();
        assert_eq!(reloaded.last_alerted_snapshot_id.as_deref(), Some("snap-1"));
        let saved_again = repo
            .save_alert_settings(&AlertSettings {
                last_alerted_snapshot_id: None,
                ..reloaded.clone()
            })
            .unwrap();
        assert_eq!(
            saved_again.last_alerted_snapshot_id.as_deref(),
            Some("snap-1")
        );
    }

    #[test]
    fn test_alert_settings_normalizes_invalid_values() {
        let repo = repo();
        let settings = AlertSettings {
            project_id: "p1".to_string(),
            enabled: true,
            channels: vec!["smoke-signal".to_string()],
            min_severity: "critical".to_string(),
            quiet_start: Some("25:99".to_string()),
            quiet_end: Some("07:00".to_string()),
            last_alerted_snapshot_id: None,
        };
        let saved = repo.save_alert_settings(&settings).unwrap();
        assert_eq!(saved.channels, vec!["desktop".to_string()]);
        assert_eq!(saved.min_severity, "warning");
        assert_eq!(saved.quiet_start, None);
        assert_eq!(saved.quiet_end.as_deref(), Some("07:00"));
    }

    #[test]
    fn test_compute_new_errors_between_snapshots() {
        let repo = repo();
        repo.conn
            .execute_batch(
                "INSERT INTO crawl_config (id, project_id, seed_urls, created_at) VALUES ('cfg', 'p1', '[]', datetime('now'));",
            )
            .unwrap();
        let insert_page = |id: &str, url: &str, status: i64| {
            repo.conn
                .execute(
                    "INSERT INTO crawled_pages (id, config_id, project_id, url, status_code, crawl_timestamp)
                     VALUES (?1, 'cfg', 'p1', ?2, ?3, '2026-01-01T00:00:00Z')",
                    params![id, url, status],
                )
                .unwrap();
        };
        let insert_check = |page_id: &str, check_id: &str, severity: &str| {
            repo.conn
                .execute(
                    "INSERT INTO seo_check_issues (page_id, project_id, category, severity, check_id, message, guidance)
                     VALUES (?1, 'p1', 'meta', ?2, ?3, 'm', 'g')",
                    params![page_id, severity, check_id],
                )
                .unwrap();
        };

        // Snapshot A: /a 200 (1 warning check), /b 200, /c 404.
        insert_page("pg-a", "https://x.com/a", 200);
        insert_page("pg-b", "https://x.com/b", 200);
        insert_page("pg-c", "https://x.com/c", 404);
        insert_check("pg-a", "missing_title", "warning");
        repo.conn
            .execute(
                "INSERT INTO page_links (from_url, to_url, config_id, project_id, link_type)
                 VALUES ('https://x.com/a', 'https://x.com/c', 'cfg', 'p1', 'internal')",
                [],
            )
            .unwrap();
        let snap_a = repo.create_crawl_snapshot("p1", "cfg").unwrap();

        // Snapshot B: /a now 404 with a new error check, /b 200 with a new
        // warning check, /c still 404, /d new 200 with a new info check,
        // /e new 404. /b links to /a and /e (both newly broken).
        repo.conn
            .execute_batch(
                "UPDATE crawled_pages SET status_code = 404 WHERE id = 'pg-a';
                 INSERT INTO crawled_pages (id, config_id, project_id, url, status_code, crawl_timestamp)
                 VALUES ('pg-d', 'cfg', 'p1', 'https://x.com/d', 200, '2026-01-02T00:00:00Z'),
                        ('pg-e', 'cfg', 'p1', 'https://x.com/e', 404, '2026-01-02T00:00:00Z');
                 INSERT INTO page_links (from_url, to_url, config_id, project_id, link_type)
                 VALUES ('https://x.com/b', 'https://x.com/a', 'cfg', 'p1', 'internal'),
                        ('https://x.com/b', 'https://x.com/e', 'cfg', 'p1', 'internal');",
            )
            .unwrap();
        insert_check("pg-a", "form_labels", "error");
        insert_check("pg-b", "missing_title", "warning");
        insert_check("pg-d", "img_no_alt", "info");
        let snap_b = repo.create_crawl_snapshot("p1", "cfg").unwrap();

        // The snapshot itself must carry the failed checks.
        let stored: String = repo
            .conn
            .query_row(
                "SELECT failed_checks_json FROM crawl_snapshot_data
                 WHERE snapshot_id = ?1 AND url = 'https://x.com/a'",
                params![snap_b.id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(stored.contains("form_labels"));

        // Threshold "warning": /a (200->404) and /e (new 404) are new error
        // URLs; /c was already 404. New checks: form_labels (error) and
        // missing_title on /b (warning); img_no_alt (info) is below threshold.
        let errors = repo
            .compute_new_errors("p1", &snap_a.id, &snap_b.id, "warning")
            .unwrap();
        assert_eq!(errors.new_error_url_total, 2);
        assert_eq!(
            errors.new_error_urls,
            vec!["https://x.com/a", "https://x.com/e"]
        );
        assert_eq!(errors.new_failed_check_total, 2);
        assert!(errors
            .new_failed_checks
            .iter()
            .any(|c| c.url == "https://x.com/a"
                && c.check_id == "form_labels"
                && c.severity == "error"));
        assert!(errors
            .new_failed_checks
            .iter()
            .any(|c| c.url == "https://x.com/b" && c.check_id == "missing_title"));
        // New broken links: /a and /e (linked from /b); /c was already broken.
        assert_eq!(errors.new_broken_link_total, 2);
        assert_eq!(
            errors.new_broken_links,
            vec!["https://x.com/a", "https://x.com/e"]
        );
        assert_eq!(errors.total_count(), 6);
        assert!(!errors.is_empty());

        // Threshold "info" also reports the info-level check on /d.
        let errors = repo
            .compute_new_errors("p1", &snap_a.id, &snap_b.id, "info")
            .unwrap();
        assert_eq!(errors.new_failed_check_total, 3);

        // Threshold "error" reports only the error-level check.
        let errors = repo
            .compute_new_errors("p1", &snap_a.id, &snap_b.id, "error")
            .unwrap();
        assert_eq!(errors.new_failed_check_total, 1);

        // Same snapshot twice: nothing is new.
        let none = repo
            .compute_new_errors("p1", &snap_b.id, &snap_b.id, "info")
            .unwrap();
        assert!(none.is_empty());
    }
}
