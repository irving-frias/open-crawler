use rusqlite::{params, Transaction};

use tracing::info;

use crate::crawler::parser::SemanticIssue;
use crate::db::CrawlSessionInfo;
use crate::error::AppError;
use crate::models::{CrawlConfig, CrawlResult, PageLink};

use super::{compress_html_body, CrawlRepo};
use super::seo::{delete_seo_normalized, save_seo_normalized};

/// Deletes the normalized `page_issues` rows belonging to the given page ids.
/// Re-crawls replace page rows per URL, and the replaced rows' ids differ from
/// the new ones, so stale issue rows must be removed explicitly.
fn delete_page_issues(tx: &Transaction<'_>, page_ids: &[String]) -> Result<(), AppError> {
    if page_ids.is_empty() {
        return Ok(());
    }
    let mut stmt = tx.prepare("DELETE FROM page_issues WHERE page_id = ?1")?;
    for id in page_ids {
        stmt.execute(params![id])?;
    }
    Ok(())
}

/// Writes one `page_issues` row per issue occurrence parsed from the page's
/// `semantic_issues_json`, preserving the array position for stable ordering.
fn save_page_issues(
    tx: &Transaction<'_>,
    project_id: &str,
    page_id: &str,
    semantic_issues_json: &Option<String>,
) -> Result<(), AppError> {
    let Some(json) = semantic_issues_json else {
        return Ok(());
    };
    let issues: Vec<SemanticIssue> = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    if issues.is_empty() {
        return Ok(());
    }
    let mut stmt = tx.prepare(
        "INSERT INTO page_issues (project_id, page_id, issue_type, severity, message, element, css_selector, xpath, position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    for (i, issue) in issues.iter().enumerate() {
        stmt.execute(params![
            project_id,
            page_id,
            issue.issue_type,
            issue.severity,
            issue.message,
            issue.element,
            issue.css_selector,
            issue.xpath,
            i as i64,
        ])?;
    }
    Ok(())
}

impl<'a> CrawlRepo<'a> {
    pub fn save_config(&self, config: &CrawlConfig) -> Result<(), AppError> {
        let id = config.id.as_deref().unwrap_or("default");
        let project_id = config.project_id.as_deref().unwrap_or("default");
        let seed_urls = serde_json::to_string(&config.seed_urls)?;
        let scan_type = serde_json::to_string(&config.scan_type)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO crawl_config (id, project_id, seed_urls, max_pages, max_depth, user_agent, respect_robots, scan_type, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
            params![
                id,
                project_id,
                seed_urls,
                1000i32,
                config.max_depth as i32,
                config.user_agent(),
                config.respect_robots as i32,
                scan_type,
            ],
        )?;

        info!("Saved crawl config: {} for project: {}", id, project_id);
        Ok(())
    }

    pub fn save_result(&self, result: &CrawlResult) -> Result<(), AppError> {
        let project_id = &result.project_id;

        let tx = self.conn.unchecked_transaction()?;

        // Re-crawls replace the previous row for the same URL instead of
        // accumulating duplicates (unique index idx_pages_project_url).
        let old_ids: Vec<String> = {
            let mut stmt =
                tx.prepare("SELECT id FROM crawled_pages WHERE project_id = ?1 AND url = ?2")?;
            let rows = stmt.query_map(params![project_id, result.url], |row| row.get(0))?;
            let ids: Result<Vec<String>, _> = rows.collect();
            ids?
        };
        delete_page_issues(&tx, &old_ids)?;
        delete_page_issues(&tx, std::slice::from_ref(&result.id))?;
        tx.execute(
            "DELETE FROM crawled_pages WHERE project_id = ?1 AND url = ?2",
            params![project_id, result.url],
        )?;
        // Re-crawls re-insert the page's own outbound links; drop the stale ones
        // so the export / site tree don't accumulate duplicates across re-crawls.
        tx.execute(
            "DELETE FROM page_links WHERE project_id = ?1 AND from_url = ?2",
            params![project_id, result.url],
        )?;
        tx.execute(
            "INSERT INTO crawled_pages
             (id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body, readability_score, content_hash, duplicate_group_id, keywords_json, og_json, pagespeed_score, pagespeed_json, seo_score, seo_audit_json, blocked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)",
            params![
                result.id,
                result.config_id,
                project_id,
                result.url,
                result.status_code.map(|s| s as i32),
                result.title,
                result.meta_description,
                result.h1,
                result.canonical,
                result.size_bytes.map(|s| s as i64),
                result.load_time_ms,
                result.is_indexable.map(|i| i as i32),
                result.depth as i32,
                result.parent_url,
                result.crawl_timestamp,
                result.html_lang,
                result.hreflang_json,
                result.semantic_issues_json,
                compress_html_body(&result.html_body),
                result.readability_score,
                result.content_hash,
                result.duplicate_group_id,
                result.keywords_json,
                result.og_json,
                result.pagespeed_score,
                result.pagespeed_json,
                result.seo_score,
                result.seo_audit_json,
                result.blocked as i32,
            ],
        )?;

        for link in &result.links {
            tx.execute(
                "INSERT INTO page_links (from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    link.from_url,
                    link.to_url,
                    link.config_id,
                    project_id,
                    link.link_type,
                    link.anchor_text,
                    link.is_follow as i32,
                ],
            )?;
        }

        save_page_issues(&tx, project_id, &result.id, &result.semantic_issues_json)?;
        delete_seo_normalized(&tx, &old_ids)?;
        delete_seo_normalized(&tx, std::slice::from_ref(&result.id))?;
        save_seo_normalized(&tx, project_id, &result.id, &result.seo_audit_json)?;

        tx.commit()?;
        self.invalidate_cache_for_project(project_id);

        Ok(())
    }

    pub fn save_results_batch(&self, results: &[CrawlResult]) -> Result<(), AppError> {
        if results.is_empty() {
            return Ok(());
        }

        let tx = self.conn.unchecked_transaction()?;

        // Deduplicate within the batch keeping the last row per (project_id, url),
        // then remove existing rows for those URLs so re-crawls replace instead
        // of accumulating duplicates (unique index idx_pages_project_url).
        let mut unique: std::collections::HashMap<(String, String), &CrawlResult> =
            std::collections::HashMap::new();
        for result in results {
            unique.insert((result.project_id.clone(), result.url.clone()), result);
        }

        // Capture the ids of the rows being replaced so their normalized
        // page_issues rows can be removed alongside them.
        let mut old_ids: Vec<String> = Vec::new();
        {
            let mut q =
                tx.prepare("SELECT id FROM crawled_pages WHERE project_id = ?1 AND url = ?2")?;
            for (project_id, url) in unique.keys() {
                let ids: Vec<String> = q
                    .query_map(params![project_id, url], |row| row.get(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                old_ids.extend(ids);
            }
        }
        delete_page_issues(&tx, &old_ids)?;
        delete_seo_normalized(&tx, &old_ids)?;

        {
            let mut del =
                tx.prepare("DELETE FROM crawled_pages WHERE project_id = ?1 AND url = ?2")?;
            for (project_id, url) in unique.keys() {
                del.execute(params![project_id, url])?;
            }
        }

        // Drop stale outbound links for replaced URLs; each result re-inserts its own.
        {
            let mut del =
                tx.prepare("DELETE FROM page_links WHERE project_id = ?1 AND from_url = ?2")?;
            for (project_id, url) in unique.keys() {
                del.execute(params![project_id, url])?;
            }
        }

        {
            let mut stmt = tx.prepare(
                "INSERT INTO crawled_pages
                 (id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body, readability_score, content_hash, duplicate_group_id, keywords_json, og_json, pagespeed_score, pagespeed_json, seo_score, seo_audit_json, blocked)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)",
            )?;

            for result in unique.values() {
                stmt.execute(params![
                    result.id,
                    result.config_id,
                    result.project_id,
                    result.url,
                    result.status_code.map(|s| s as i32),
                    result.title,
                    result.meta_description,
                    result.h1,
                    result.canonical,
                    result.size_bytes.map(|s| s as i64),
                    result.load_time_ms,
                    result.is_indexable.map(|i| i as i32),
                    result.depth as i32,
                    result.parent_url,
                    result.crawl_timestamp,
                    result.html_lang,
                    result.hreflang_json,
                    result.semantic_issues_json,
                    compress_html_body(&result.html_body),
                    result.readability_score,
                    result.content_hash,
                    result.duplicate_group_id,
                    result.keywords_json,
                    result.og_json,
                    result.pagespeed_score,
                    result.pagespeed_json,
                    result.seo_score,
                    result.seo_audit_json,
                    result.blocked as i32,
                ])?;
            }
        }

        {
            let mut stmt = tx.prepare(
                "INSERT INTO page_issues (project_id, page_id, issue_type, severity, message, element, css_selector, xpath, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )?;
            for result in unique.values() {
                let issues: Vec<SemanticIssue> = match result.semantic_issues_json.as_deref() {
                    Some(json) => serde_json::from_str(json).unwrap_or_default(),
                    None => Vec::new(),
                };
                for (i, issue) in issues.iter().enumerate() {
                    stmt.execute(params![
                        result.project_id,
                        result.id,
                        issue.issue_type,
                        issue.severity,
                        issue.message,
                        issue.element,
                        issue.css_selector,
                        issue.xpath,
                        i as i64,
                    ])?;
                }
            }
        }

        // Normalized SEO rows for the freshly written pages.
        for result in unique.values() {
            save_seo_normalized(
                &tx,
                &result.project_id,
                &result.id,
                &result.seo_audit_json,
            )?;
        }

        tx.commit()?;
        info!("Batch saved {} results", results.len());

        let project_ids: std::collections::HashSet<String> =
            results.iter().map(|r| r.project_id.clone()).collect();
        for project_id in project_ids {
            self.invalidate_cache_for_project(&project_id);
        }

        Ok(())
    }

    pub fn save_links_batch(&self, links: &[PageLink]) -> Result<(), AppError> {
        if links.is_empty() {
            return Ok(());
        }

        let tx = self.conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO page_links (from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;

            for link in links {
                stmt.execute(params![
                    link.from_url,
                    link.to_url,
                    link.config_id,
                    link.project_id,
                    link.link_type,
                    link.anchor_text,
                    link.is_follow as i32,
                ])?;
            }
        }

        tx.commit()?;
        info!("Batch saved {} links", links.len());

        let project_ids: std::collections::HashSet<String> =
            links.iter().map(|l| l.project_id.clone()).collect();
        for project_id in project_ids {
            self.invalidate_cache_for_project(&project_id);
        }

        Ok(())
    }

    pub fn url_exists(&self, url: &str) -> Result<bool, AppError> {
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM crawled_pages WHERE url = ?1)",
            params![url],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    pub fn save_error(
        &self,
        url: &str,
        config_id: &str,
        project_id: &str,
        error_type: &str,
        error_message: &str,
    ) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO crawl_errors (url, config_id, project_id, error_type, error_message, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
            params![url, config_id, project_id, error_type, error_message],
        )?;
        Ok(())
    }

    // ==================== CRAWL SESSION (RESUME) ====================

    pub fn create_session(
        &self,
        project_id: &str,
        config: &CrawlConfig,
    ) -> Result<String, AppError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let config_json = serde_json::to_string(config)?;
        let seed_urls = serde_json::to_string(&config.seed_urls)?;

        self.conn.execute(
            "INSERT INTO crawl_sessions (id, project_id, config_json, status, seed_urls, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'running', ?4, ?5, ?6)",
            params![session_id, project_id, config_json, seed_urls, now, now],
        )?;

        info!(
            "Created crawl session: {} for project: {}",
            session_id, project_id
        );
        Ok(session_id)
    }

    pub fn update_session_progress(
        &self,
        session_id: &str,
        pages_crawled: u32,
        errors: u32,
        elapsed_secs: u64,
    ) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE crawl_sessions SET pages_crawled = ?1, errors = ?2, elapsed_secs = ?3, updated_at = ?4 WHERE id = ?5",
            params![pages_crawled, errors, elapsed_secs, now, session_id],
        )?;
        Ok(())
    }

    pub fn complete_session(&self, session_id: &str) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE crawl_sessions SET status = 'completed', updated_at = ?1 WHERE id = ?2",
            params![now, session_id],
        )?;
        // Clear queue since crawl is done
        self.conn.execute(
            "DELETE FROM crawl_queue WHERE session_id = ?1",
            params![session_id],
        )?;
        info!("Completed crawl session: {}", session_id);
        Ok(())
    }

    pub fn interrupt_session(
        &self,
        session_id: &str,
        pages_crawled: u32,
        errors: u32,
        elapsed_secs: u64,
    ) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE crawl_sessions SET status = 'interrupted', pages_crawled = ?1, errors = ?2, elapsed_secs = ?3, updated_at = ?4 WHERE id = ?5",
            params![pages_crawled, errors, elapsed_secs, now, session_id],
        )?;
        info!("Interrupted crawl session: {}", session_id);
        Ok(())
    }

    pub fn get_interrupted_session(
        &self,
        project_id: &str,
    ) -> Result<Option<CrawlSessionInfo>, AppError> {
        let result = self.conn.query_row(
            "SELECT id, project_id, config_json, status, pages_crawled, errors, elapsed_secs, seed_urls, created_at, updated_at
             FROM crawl_sessions WHERE project_id = ?1 AND status = 'interrupted'
             ORDER BY updated_at DESC LIMIT 1",
            params![project_id],
            |row| {
                Ok(CrawlSessionInfo {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    config_json: row.get(2)?,
                    status: row.get(3)?,
                    pages_crawled: row.get(4)?,
                    errors: row.get(5)?,
                    elapsed_secs: row.get(6)?,
                    seed_urls: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        );

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_session_config(&self, session_id: &str) -> Result<Option<CrawlConfig>, AppError> {
        let result: Result<String, _> = self.conn.query_row(
            "SELECT config_json FROM crawl_sessions WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        );

        match result {
            Ok(json) => {
                let config: CrawlConfig = serde_json::from_str(&json)?;
                Ok(Some(config))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns the most recently created crawl session's full config for a
    /// project (the "last crawl" settings), so the UI can restore the seed URL
    /// and options after a reload or project switch.
    pub fn get_latest_session_config(
        &self,
        project_id: &str,
    ) -> Result<Option<CrawlConfig>, AppError> {
        let result: Result<String, _> = self.conn.query_row(
            "SELECT config_json FROM crawl_sessions
             WHERE project_id = ?1
             ORDER BY created_at DESC, rowid DESC
             LIMIT 1",
            params![project_id],
            |row| row.get(0),
        );

        match result {
            Ok(json) => {
                let config: CrawlConfig = serde_json::from_str(&json)?;
                Ok(Some(config))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // ==================== CRAWL QUEUE (RESUME) ====================

    pub fn save_queue_batch(
        &self,
        session_id: &str,
        urls: &[(String, u32)],
    ) -> Result<(), AppError> {
        if urls.is_empty() {
            return Ok(());
        }

        let tx = self.conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO crawl_queue (session_id, url, depth) VALUES (?1, ?2, ?3)",
            )?;

            for (url, depth) in urls {
                stmt.execute(params![session_id, url, depth])?;
            }
        }

        tx.commit()?;
        info!(
            "Saved {} queue entries for session: {}",
            urls.len(),
            session_id
        );
        Ok(())
    }

    pub fn load_queue(&self, session_id: &str) -> Result<Vec<(String, u32)>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT url, depth FROM crawl_queue WHERE session_id = ?1")?;

        let entries = stmt
            .query_map(params![session_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        info!(
            "Loaded {} queue entries for session: {}",
            entries.len(),
            session_id
        );
        Ok(entries)
    }

    pub fn clear_queue(&self, session_id: &str) -> Result<(), AppError> {
        self.conn.execute(
            "DELETE FROM crawl_queue WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    pub fn get_visited_urls_for_project(&self, project_id: &str) -> Result<Vec<String>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT url FROM crawled_pages WHERE project_id = ?1 LIMIT 100000")?;

        let urls = stmt
            .query_map(params![project_id], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(urls)
    }
}
