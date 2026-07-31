use rusqlite::{params, Connection};
use tracing::info;

use crate::error::AppError;
use crate::models::{CrawlConfig, CrawlResult, IssueCount, PageLink, Project};

pub struct CrawlRepo<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone)]
pub struct CrawlSessionInfo {
    pub id: String,
    pub project_id: String,
    pub config_json: String,
    pub status: String,
    pub pages_crawled: u32,
    pub errors: u32,
    pub elapsed_secs: u64,
    pub seed_urls: String,
    pub created_at: String,
    pub updated_at: String,
}

impl<'a> CrawlRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // ==================== PROJECT CRUD ====================

    pub fn create_project(&self, name: &str) -> Result<Project, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, now, now],
        )?;

        info!("Created project: {} ({})", name, id);

        Ok(Project {
            id,
            name: name.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, created_at, updated_at FROM projects ORDER BY created_at DESC")?;

        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>, AppError> {
        let result = self.conn.query_row(
            "SELECT id, name, created_at, updated_at FROM projects WHERE id = ?1",
            params![id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        );

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn rename_project(&self, id: &str, name: &str) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now, id],
        )?;
        info!("Renamed project {} to {}", id, name);
        Ok(())
    }

    pub fn delete_project(&self, id: &str) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;

        // 1. Delete crawl_queue (child of crawl_sessions)
        tx.execute(
            "DELETE FROM crawl_queue WHERE session_id IN (SELECT id FROM crawl_sessions WHERE project_id = ?1)",
            params![id],
        )?;

        // 2. Delete crawl_sessions (child of projects)
        tx.execute(
            "DELETE FROM crawl_sessions WHERE project_id = ?1",
            params![id],
        )?;

        // 3. Delete page_links (FK: config_id -> crawl_config)
        tx.execute(
            "DELETE FROM page_links WHERE config_id IN (SELECT id FROM crawl_config WHERE project_id = ?1)",
            params![id],
        )?;

        // 4. Delete crawl_errors (FK: config_id -> crawl_config)
        tx.execute(
            "DELETE FROM crawl_errors WHERE config_id IN (SELECT id FROM crawl_config WHERE project_id = ?1)",
            params![id],
        )?;

        // 5. Delete crawled_pages (FK: config_id -> crawl_config)
        tx.execute(
            "DELETE FROM crawled_pages WHERE config_id IN (SELECT id FROM crawl_config WHERE project_id = ?1)",
            params![id],
        )?;

        // 6. Delete crawl_config (FK: project_id -> projects)
        tx.execute(
            "DELETE FROM crawl_config WHERE project_id = ?1",
            params![id],
        )?;

        // 7. Delete project
        tx.execute("DELETE FROM projects WHERE id = ?1", params![id])?;

        tx.commit()?;
        info!("Deleted project {}", id);
        Ok(())
    }

    pub fn get_project_stats(&self, project_id: &str) -> Result<serde_json::Value, AppError> {
        let pages_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;

        let errors_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawl_errors WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;

        let links_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM page_links WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;

        Ok(serde_json::json!({
            "pages": pages_count,
            "errors": errors_count,
            "links": links_count,
        }))
    }

    // ==================== CRAWL CONFIG ====================

    pub fn save_config(&self, config: &CrawlConfig) -> Result<(), AppError> {
        let id = config.id.as_deref().unwrap_or("default");
        let project_id = config.project_id.as_deref().unwrap_or("default");
        let seed_urls = serde_json::to_string(&config.seed_urls)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO crawl_config (id, project_id, seed_urls, max_pages, max_depth, user_agent, respect_robots, render_js, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
            params![
                id,
                project_id,
                seed_urls,
                1000i32,
                config.max_depth as i32,
                config.user_agent(),
                config.respect_robots as i32,
                config.render_js as i32,
            ],
        )?;

        info!("Saved crawl config: {} for project: {}", id, project_id);
        Ok(())
    }

    // ==================== CRAWL RESULTS ====================

    pub fn save_result(&self, result: &CrawlResult) -> Result<(), AppError> {
        let project_id = &result.project_id;

        self.conn.execute(
            "INSERT OR REPLACE INTO crawled_pages
             (id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
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
                result.html_body,
            ],
        )?;

        for link in &result.links {
            self.conn.execute(
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

        Ok(())
    }

    pub fn save_results_batch(&self, results: &[CrawlResult]) -> Result<(), AppError> {
        if results.is_empty() {
            return Ok(());
        }

        let tx = self.conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO crawled_pages
                 (id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            )?;

            for result in results {
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
                    result.html_body,
                ])?;
            }
        }

        tx.commit()?;
        info!("Batch saved {} results", results.len());
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

    pub fn get_results(
        &self,
        project_id: &str,
        page: u32,
        page_size: u32,
        semantic_issue_type: Option<&str>,
        search: Option<&str>,
    ) -> Result<(Vec<CrawlResult>, u32), AppError> {
        let offset = (page - 1) * page_size;

        let has_search = search.map(|s| !s.is_empty()).unwrap_or(false);
        let has_issue_type = semantic_issue_type.is_some();

        let mut where_clauses = vec!["project_id = ?1".to_string()];
        let mut param_index = 2u32;

        if has_issue_type {
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM json_each(crawled_pages.semantic_issues_json) WHERE json_each.value->>'$.issue_type' = ?{})",
                param_index
            ));
            param_index += 1;
        }

        if has_search {
            where_clauses.push(format!(
                "(url LIKE ?{} OR title LIKE ?{} OR h1 LIKE ?{})",
                param_index, param_index, param_index
            ));
            param_index += 1;
        }

        let where_sql = where_clauses.join(" AND ");
        let count_sql = format!("SELECT COUNT(*) FROM crawled_pages WHERE {}", where_sql);
        let query_sql = format!(
            "SELECT id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body
             FROM crawled_pages WHERE {}
             ORDER BY crawl_timestamp DESC
             LIMIT ?{} OFFSET ?{}",
            where_sql, param_index, param_index + 1
        );

        let total: u32 = if has_issue_type && has_search {
            let search_pat = format!("%{}%", search.unwrap_or(""));
            let issuetype = semantic_issue_type.unwrap();
            self.conn.query_row(&count_sql, params![project_id, issuetype, search_pat], |row| row.get(0))?
        } else if has_issue_type {
            let issuetype = semantic_issue_type.unwrap();
            self.conn.query_row(&count_sql, params![project_id, issuetype], |row| row.get(0))?
        } else if has_search {
            let search_pat = format!("%{}%", search.unwrap_or(""));
            self.conn.query_row(&count_sql, params![project_id, search_pat], |row| row.get(0))?
        } else {
            self.conn.query_row(&count_sql, params![project_id], |row| row.get(0))?
        };

        let mut stmt = self.conn.prepare(&query_sql)?;

        let results = if has_issue_type && has_search {
            let search_pat = format!("%{}%", search.unwrap_or(""));
            let issuetype = semantic_issue_type.unwrap();
            stmt.query_map(params![project_id, issuetype, search_pat, page_size, offset], |row| {
                Self::row_to_result(row)
            })?
            .collect::<Result<Vec<_>, _>>()?
        } else if has_issue_type {
            let issuetype = semantic_issue_type.unwrap();
            stmt.query_map(params![project_id, issuetype, page_size, offset], |row| {
                Self::row_to_result(row)
            })?
            .collect::<Result<Vec<_>, _>>()?
        } else if has_search {
            let search_pat = format!("%{}%", search.unwrap_or(""));
            stmt.query_map(params![project_id, search_pat, page_size, offset], |row| {
                Self::row_to_result(row)
            })?
            .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(params![project_id, page_size, offset], |row| {
                Self::row_to_result(row)
            })?
            .collect::<Result<Vec<_>, _>>()?
        };

        Ok((results, total))
    }

    fn row_to_result(row: &rusqlite::Row) -> Result<CrawlResult, rusqlite::Error> {
        Ok(CrawlResult {
            id: row.get(0)?,
            config_id: row.get(1)?,
            project_id: row.get(2)?,
            url: row.get(3)?,
            status_code: row.get::<_, Option<i32>>(4)?.map(|s| s as u16),
            title: row.get(5)?,
            meta_description: row.get(6)?,
            h1: row.get(7)?,
            canonical: row.get(8)?,
            size_bytes: row.get::<_, Option<i64>>(9)?.map(|s| s as usize),
            load_time_ms: row.get::<_, Option<i64>>(10)?.map(|l| l as u64),
            is_indexable: row.get::<_, Option<i32>>(11)?.map(|i| i != 0),
            depth: row.get::<_, i32>(12)? as u32,
            parent_url: row.get(13)?,
            crawl_timestamp: row.get(14)?,
            links: Vec::new(),
            html_lang: row.get(15)?,
            hreflang_json: row.get(16)?,
            semantic_issues_json: row.get(17)?,
            html_body: row.get(18)?,
        })
    }

    pub fn get_page_detail(&self, page_id: &str) -> Result<(CrawlResult, Vec<PageLink>), AppError> {
        let result = self.conn.query_row(
            "SELECT id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body
             FROM crawled_pages WHERE id = ?1",
            params![page_id],
                            Self::row_to_result,
        )?;

        let mut stmt = self.conn.prepare(
            "SELECT from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow
             FROM page_links WHERE from_url = ?1",
        )?;

        let links = stmt
            .query_map(params![result.url], |row| {
                Ok(PageLink {
                    from_url: row.get(0)?,
                    to_url: row.get(1)?,
                    config_id: row.get(2)?,
                    project_id: row.get(3)?,
                    link_type: row.get(4)?,
                    anchor_text: row.get(5)?,
                    is_follow: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((result, links))
    }

    pub fn get_page_html(&self, page_id: &str) -> Result<Option<String>, AppError> {
        let html: Option<String> = self.conn.query_row(
            "SELECT html_body FROM crawled_pages WHERE id = ?1",
            params![page_id],
            |row| row.get(0),
        )?;
        Ok(html)
    }

    pub fn save_screenshot(&self, page_id: &str, png_data: &[u8]) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE crawled_pages SET screenshot_png = ?1 WHERE id = ?2",
            params![png_data, page_id],
        )?;
        Ok(())
    }

    pub fn get_screenshot(&self, page_id: &str) -> Result<Option<Vec<u8>>, AppError> {
        let data: Option<Vec<u8>> = self.conn.query_row(
            "SELECT screenshot_png FROM crawled_pages WHERE id = ?1",
            params![page_id],
            |row| row.get(0),
        )?;
        Ok(data)
    }

    pub fn get_semantic_issue_counts(&self, project_id: &str) -> Result<Vec<IssueCount>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT
                json_each.value->>'$.issue_type' as issue_type,
                json_each.value->>'$.severity' as severity,
                COUNT(*) as cnt
             FROM crawled_pages, json_each(crawled_pages.semantic_issues_json)
             WHERE crawled_pages.project_id = ?1
               AND crawled_pages.semantic_issues_json IS NOT NULL
             GROUP BY issue_type, severity
             ORDER BY cnt DESC",
        )?;

        let counts = stmt
            .query_map(params![project_id], |row| {
                Ok(IssueCount {
                    issue_type: row.get(0)?,
                    severity: row.get(1)?,
                    count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(counts)
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

        info!("Created crawl session: {} for project: {}", session_id, project_id);
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
        self.conn
            .execute("DELETE FROM crawl_queue WHERE session_id = ?1", params![session_id])?;
        info!("Completed crawl session: {}", session_id);
        Ok(())
    }

    pub fn interrupt_session(&self, session_id: &str, pages_crawled: u32, errors: u32, elapsed_secs: u64) -> Result<(), AppError> {
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

    // ==================== CRAWL QUEUE (RESUME) ====================

    pub fn save_queue_batch(&self, session_id: &str, urls: &[(String, u32)]) -> Result<(), AppError> {
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
        info!("Saved {} queue entries for session: {}", urls.len(), session_id);
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

        info!("Loaded {} queue entries for session: {}", entries.len(), session_id);
        Ok(entries)
    }

    pub fn clear_queue(&self, session_id: &str) -> Result<(), AppError> {
        self.conn
            .execute("DELETE FROM crawl_queue WHERE session_id = ?1", params![session_id])?;
        Ok(())
    }

    pub fn get_visited_urls_for_project(&self, project_id: &str) -> Result<Vec<String>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT url FROM crawled_pages WHERE project_id = ?1")?;

        let urls = stmt
            .query_map(params![project_id], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(urls)
    }

    // ── Settings ──────────────────────────────────────────────

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Crawl(e.to_string())),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<std::collections::HashMap<String, String>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM settings")?;
        let settings = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(settings)
    }

    pub fn get_all_results(&self, project_id: &str) -> Result<Vec<CrawlResult>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body
             FROM crawled_pages WHERE project_id = ?1
             ORDER BY crawl_timestamp DESC",
        )?;
        let results = stmt
            .query_map(params![project_id], Self::row_to_result)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }

    pub fn get_links_for_project(&self, project_id: &str) -> Result<Vec<PageLink>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow
             FROM page_links WHERE project_id = ?1",
        )?;
        let links = stmt
            .query_map(params![project_id], |row| {
                Ok(PageLink {
                    from_url: row.get(0)?,
                    to_url: row.get(1)?,
                    config_id: row.get(2)?,
                    project_id: row.get(3)?,
                    link_type: row.get(4)?,
                    anchor_text: row.get(5)?,
                    is_follow: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(links)
    }
}
