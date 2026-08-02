use rusqlite::{params, Connection};
use tracing::info;

use crate::error::AppError;
use crate::models::{CrawlConfig, CrawlResult, IssueCount, PageLink, Project, SiteTreeNode};
use crate::{ResultsCacheArc, ResultsCacheKey};

fn compress_gzip(data: &[u8]) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

fn decompress_gzip(data: &[u8]) -> Vec<u8> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).unwrap();
    out
}

fn compress_html_body(html: &Option<String>) -> Option<String> {
    html.as_ref().map(|s| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, compress_gzip(s.as_bytes())))
}

fn decompress_html_body(encoded: &Option<String>) -> Option<String> {
    encoded.as_ref().and_then(|e| base64::Engine::decode(&base64::engine::general_purpose::STANDARD, e).ok()).and_then(|bytes| String::from_utf8(decompress_gzip(&bytes)).ok())
}

fn compress_png(png: &Option<Vec<u8>>) -> Option<Vec<u8>> {
    png.as_ref().map(|data| compress_gzip(data))
}

fn decompress_png(data: &Option<Vec<u8>>) -> Option<Vec<u8>> {
    data.as_ref().map(|bytes| decompress_gzip(bytes))
}

pub struct CrawlRepo<'a> {
    conn: &'a Connection,
    results_cache: Option<ResultsCacheArc>,
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
    pub fn new(conn: &'a Connection, results_cache: Option<ResultsCacheArc>) -> Self {
        Self { conn, results_cache }
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
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at, updated_at FROM projects ORDER BY created_at DESC",
        )?;

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

        self.invalidate_cache_for_project(id);

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
                compress_html_body(&result.html_body),
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
                    compress_html_body(&result.html_body),
                ])?;
            }
        }

        tx.commit()?;
        info!("Batch saved {} results", results.len());

        let project_ids: std::collections::HashSet<String> = results.iter().map(|r| r.project_id.clone()).collect();
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

        let project_ids: std::collections::HashSet<String> = links.iter().map(|l| l.project_id.clone()).collect();
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

    #[allow(clippy::too_many_arguments)]
    pub fn get_results(
        &self,
        project_id: &str,
        page: u32,
        page_size: u32,
        semantic_issue_type: Option<&str>,
        search: Option<&str>,
        status_filter: Option<&[u32]>,
        severity_filter: Option<&[String]>,
        domain_filter: Option<&str>,
        depth_filter: Option<u32>,
        missing_title: bool,
        duplicate_title: bool,
        noindex_only: bool,
        is_404: bool,
    ) -> Result<(Vec<CrawlResult>, u32), AppError> {
        let cache_key = ResultsCacheKey {
            project_id: project_id.to_string(),
            page,
            page_size,
            semantic_issue_type: semantic_issue_type.map(|s| s.to_string()),
            search: search.map(|s| s.to_string()),
            status_filter: status_filter.unwrap_or_default().to_vec(),
            severity_filter: severity_filter.unwrap_or_default().to_vec(),
            domain_filter: domain_filter.map(|s| s.to_string()),
            depth_filter,
            missing_title,
            duplicate_title,
            noindex_only,
            is_404,
        };

        if let Some(ref cache_arc) = self.results_cache {
            let mut cache = cache_arc.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let offset = (page - 1) * page_size;

        let mut where_clauses = vec!["project_id = ?1".to_string()];
        let mut param_index = 2u32;

        if let Some(_issue_type) = semantic_issue_type {
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM json_each(crawled_pages.semantic_issues_json) WHERE json_each.value->>'$.issue_type' = ?{})",
                param_index
            ));
            param_index += 1;
        }

        if let Some(s) = search {
            if !s.is_empty() {
                where_clauses.push(format!(
                    "(url LIKE ?{} OR title LIKE ?{} OR h1 LIKE ?{})",
                    param_index, param_index, param_index
                ));
                param_index += 1;
            }
        }

        if let Some(statuses) = status_filter {
            if !statuses.is_empty() {
                let placeholders: Vec<String> = statuses
                    .iter()
                    .map(|_| {
                        let idx = param_index;
                        param_index += 1;
                        format!("?{}", idx)
                    })
                    .collect();
                where_clauses.push(format!("status_code IN ({})", placeholders.join(",")));
            }
        }

        if let Some(severities) = severity_filter {
            if !severities.is_empty() {
                let placeholders: Vec<String> = severities
                    .iter()
                    .map(|_| {
                        let idx = param_index;
                        param_index += 1;
                        format!("?{}", idx)
                    })
                    .collect();
                where_clauses.push(format!(
                    "EXISTS (SELECT 1 FROM json_each(crawled_pages.semantic_issues_json) WHERE json_each.value->>'$.severity' IN ({}))",
                    placeholders.join(",")
                ));
            }
        }

        if let Some(domain) = domain_filter {
            if !domain.is_empty() {
                where_clauses.push(format!(
                    "(url LIKE ?{} OR url LIKE ?{})",
                    param_index,
                    param_index + 1
                ));
                param_index += 2;
            }
        }

        if let Some(_depth) = depth_filter {
            where_clauses.push(format!("depth <= ?{}", param_index));
            param_index += 1;
        }

        if missing_title {
            where_clauses.push("(title IS NULL OR trim(title) = '')".to_string());
        }

        if duplicate_title {
            where_clauses.push(
                "title IS NOT NULL AND trim(title) <> '' AND (SELECT COUNT(*) FROM crawled_pages c2 WHERE c2.project_id = crawled_pages.project_id AND c2.title = crawled_pages.title) > 1".to_string(),
            );
        }

        if noindex_only {
            where_clauses.push("is_indexable = 0".to_string());
        }

        if is_404 {
            where_clauses.push("status_code = 404".to_string());
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

        // Build params for count query
        let mut count_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        count_params.push(Box::new(project_id.to_string()));
        if let Some(issue_type) = semantic_issue_type {
            count_params.push(Box::new(issue_type.to_string()));
        }
        if let Some(s) = search {
            if !s.is_empty() {
                count_params.push(Box::new(format!("%{}%", s)));
            }
        }
        if let Some(statuses) = status_filter {
            for s in statuses {
                count_params.push(Box::new(*s as i32));
            }
        }
        if let Some(severities) = severity_filter {
            for sev in severities {
                count_params.push(Box::new(sev.clone()));
            }
        }
        if let Some(domain) = domain_filter {
            if !domain.is_empty() {
                count_params.push(Box::new(format!("https://{}/%", domain)));
                count_params.push(Box::new(format!("http://{}/%", domain)));
            }
        }
        if let Some(depth) = depth_filter {
            count_params.push(Box::new(depth as i32));
        }

        let total: u32 = self.conn.query_row(
            &count_sql,
            rusqlite::params_from_iter(count_params.iter().map(|p| p.as_ref())),
            |row| row.get(0),
        )?;

        // Build params for query (same base + limit/offset)
        let mut query_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        query_params.push(Box::new(project_id.to_string()));
        if let Some(issue_type) = semantic_issue_type {
            query_params.push(Box::new(issue_type.to_string()));
        }
        if let Some(s) = search {
            if !s.is_empty() {
                query_params.push(Box::new(format!("%{}%", s)));
            }
        }
        if let Some(statuses) = status_filter {
            for s in statuses {
                query_params.push(Box::new(*s as i32));
            }
        }
        if let Some(severities) = severity_filter {
            for sev in severities {
                query_params.push(Box::new(sev.clone()));
            }
        }
        if let Some(domain) = domain_filter {
            if !domain.is_empty() {
                query_params.push(Box::new(format!("https://{}/%", domain)));
                query_params.push(Box::new(format!("http://{}/%", domain)));
            }
        }
        if let Some(depth) = depth_filter {
            query_params.push(Box::new(depth as i32));
        }
        query_params.push(Box::new(page_size as i32));
        query_params.push(Box::new(offset as i32));

        let mut stmt = self.conn.prepare(&query_sql)?;
        let results = stmt
            .query_map(
                rusqlite::params_from_iter(query_params.iter().map(|p| p.as_ref())),
                Self::row_to_result,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(ref cache_arc) = self.results_cache {
            let mut cache = cache_arc.lock().unwrap();
            cache.put(cache_key, (results.clone(), total));
        }

        Ok((results, total))
    }

    fn invalidate_cache_for_project(&self, project_id: &str) {
        if let Some(ref cache_arc) = self.results_cache {
            let mut cache = cache_arc.lock().unwrap();
            let keys_to_remove: Vec<ResultsCacheKey> = cache.iter().filter(|(k, _)| k.project_id == project_id).map(|(k, _)| k.clone()).collect();
            for key in keys_to_remove {
                cache.pop(&key);
            }
        }
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
            html_body: decompress_html_body(&row.get(18)?),
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
        Ok(decompress_html_body(&html).or(html))
    }

    pub fn save_screenshot(&self, page_id: &str, png_data: &[u8]) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE crawled_pages SET screenshot_png = ?1 WHERE id = ?2",
            params![compress_png(&Some(png_data.to_vec())).as_deref(), page_id],
        )?;
        Ok(())
    }

    pub fn get_screenshot(&self, page_id: &str) -> Result<Option<Vec<u8>>, AppError> {
        let data: Option<Vec<u8>> = self.conn.query_row(
            "SELECT screenshot_png FROM crawled_pages WHERE id = ?1",
            params![page_id],
            |row| row.get(0),
        )?;
        Ok(decompress_png(&data))
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

    /// Returns the children of a node in the site tree. Internal pages only
    /// (pages that were actually crawled for this project). When `url` is
    /// `None`, returns the seed pages (depth 0) as roots.
    pub fn get_site_tree(
        &self,
        project_id: &str,
        url: Option<&str>,
        limit: u32,
    ) -> Result<Vec<SiteTreeNode>, AppError> {
        let (sql, row_params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
            if let Some(from_url) = url {
                let sql = "SELECT DISTINCT pl.to_url, cp.title, cp.status_code, cp.depth,
                            EXISTS (
                                SELECT 1 FROM page_links pl2
                                WHERE pl2.from_url = pl.to_url
                                  AND pl2.to_url <> pl2.from_url
                                  AND EXISTS (
                                    SELECT 1 FROM crawled_pages cp2
                                    WHERE cp2.project_id = ?3 AND cp2.url = pl2.to_url
                                  )
                            )
                     FROM page_links pl
                     JOIN crawled_pages cp ON cp.url = pl.to_url AND cp.project_id = ?3
                     WHERE pl.project_id = ?3 AND pl.from_url = ?4 AND pl.to_url <> ?4
                     ORDER BY cp.crawl_timestamp DESC
                     LIMIT ?5"
                    .to_string();
                let params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
                    Box::new(project_id.to_string()),
                    Box::new(project_id.to_string()),
                    Box::new(project_id.to_string()),
                    Box::new(from_url.to_string()),
                    Box::new(limit as i32),
                ];
                (sql, params)
            } else {
                let sql = "SELECT url, title, status_code, depth,
                            EXISTS (
                                SELECT 1 FROM page_links pl2
                                WHERE pl2.from_url = crawled_pages.url
                                  AND pl2.to_url <> pl2.from_url
                                  AND EXISTS (
                                    SELECT 1 FROM crawled_pages cp2
                                    WHERE cp2.project_id = ?2 AND cp2.url = pl2.to_url
                                  )
                            )
                     FROM crawled_pages
                     WHERE project_id = ?1 AND depth = 0
                     ORDER BY crawl_timestamp DESC
                     LIMIT ?3"
                    .to_string();
                let params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
                    Box::new(project_id.to_string()),
                    Box::new(project_id.to_string()),
                    Box::new(limit as i32),
                ];
                (sql, params)
            };

        let mut stmt = self.conn.prepare(&sql)?;
        let children = stmt
            .query_map(
                rusqlite::params_from_iter(row_params.iter().map(|p| p.as_ref())),
                Self::row_to_site_tree_node,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(children)
    }

    fn row_to_site_tree_node(row: &rusqlite::Row) -> Result<SiteTreeNode, rusqlite::Error> {
        Ok(SiteTreeNode {
            url: row.get(0)?,
            title: row.get(1)?,
            status_code: row.get::<_, Option<i32>>(2)?.map(|s| s as u16),
            depth: row.get::<_, i32>(3)? as u32,
            has_children: row.get(4)?,
        })
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
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let settings = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(settings)
    }

    pub fn count_pages(&self, project_id: &str) -> Result<u32, AppError> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(n as u32)
    }

    pub fn count_links(&self, project_id: &str) -> Result<u32, AppError> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM page_links WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(n as u32)
    }

    pub fn count_issues(&self, project_id: &str) -> Result<u32, AppError> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawled_pages, json_each(crawled_pages.semantic_issues_json)
             WHERE project_id = ?1 AND json_type(crawled_pages.semantic_issues_json) = 'array'",
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(n as u32)
    }

    /// Streams crawled pages for export, in `(crawl_timestamp DESC, id DESC)` order
    /// (same order as `get_all_results`), without the heavy `html_body`/`hreflang_json`
    /// columns. Pass the last row's `crawl_timestamp` + `id` from the previous batch.
    pub fn get_result_batch(
        &self,
        project_id: &str,
        last_timestamp: Option<&str>,
        last_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<CrawlResult>, AppError> {
        let cols = "id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, semantic_issues_json";
        let query = if last_timestamp.is_some() {
            format!(
                "SELECT {} FROM crawled_pages
                 WHERE project_id = ?1 AND (crawl_timestamp < ?2 OR (crawl_timestamp = ?2 AND id < ?3))
                 ORDER BY crawl_timestamp DESC, id DESC
                 LIMIT ?4",
                cols
            )
        } else {
            format!(
                "SELECT {} FROM crawled_pages
                 WHERE project_id = ?1
                 ORDER BY crawl_timestamp DESC, id DESC
                 LIMIT ?2",
                cols
            )
        };
        let mut stmt = self.conn.prepare(&query)?;
        let results = match (last_timestamp, last_id) {
            (Some(ts), Some(id)) => stmt
                .query_map(params![project_id, ts, id, limit], Self::row_to_result_export)?
                .collect::<Result<Vec<_>, _>>()?,
            _ => stmt
                .query_map(params![project_id, limit], Self::row_to_result_export)?
                .collect::<Result<Vec<_>, _>>()?,
        };
        Ok(results)
    }

    /// Streams links for export via `rowid` keyset pagination. Returns `(rowid, link)`
    /// pairs so the caller can continue paging with the last rowid.
    pub fn get_links_batch(
        &self,
        project_id: &str,
        last_rowid: Option<i64>,
        limit: u32,
    ) -> Result<Vec<(i64, PageLink)>, AppError> {
        let query = if last_rowid.is_some() {
            "SELECT rowid, from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow
             FROM page_links WHERE project_id = ?1 AND rowid < ?2
             ORDER BY rowid DESC LIMIT ?3"
                .to_string()
        } else {
            "SELECT rowid, from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow
             FROM page_links WHERE project_id = ?1
             ORDER BY rowid DESC LIMIT ?2"
                .to_string()
        };
        let mut stmt = self.conn.prepare(&query)?;
        let links = match last_rowid {
            Some(rid) => stmt
                .query_map(params![project_id, rid, limit], Self::row_to_link_export)?
                .collect::<Result<Vec<_>, _>>()?,
            None => stmt
                .query_map(params![project_id, limit], Self::row_to_link_export)?
                .collect::<Result<Vec<_>, _>>()?,
        };
        Ok(links)
    }

    fn row_to_result_export(row: &rusqlite::Row) -> Result<CrawlResult, rusqlite::Error> {
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
            hreflang_json: None,
            semantic_issues_json: row.get(16)?,
            html_body: None,
        })
    }

    fn row_to_link_export(row: &rusqlite::Row) -> Result<(i64, PageLink), rusqlite::Error> {
        Ok((
            row.get(0)?,
            PageLink {
                from_url: row.get(1)?,
                to_url: row.get(2)?,
                config_id: row.get(3)?,
                project_id: row.get(4)?,
                link_type: row.get(5)?,
                anchor_text: row.get(6)?,
                is_follow: row.get::<_, i32>(7)? != 0,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;

    fn test_repo() -> CrawlRepo<'static> {
        let conn = Box::leak(Box::new(Connection::open_in_memory().unwrap()));
        run_migrations(conn).unwrap();
        conn.execute_batch(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('p1', 'P1', datetime('now'), datetime('now'));
             INSERT INTO crawl_config (id, project_id, seed_urls, created_at) VALUES ('cfg', 'p1', '[]', datetime('now'));",
        )
        .unwrap();
        CrawlRepo::new(conn, None)
    }

    fn page(id: &str, url: &str, title: Option<&str>, status: u16, indexable: bool) -> CrawlResult {
        CrawlResult {
            id: id.to_string(),
            config_id: "cfg".to_string(),
            project_id: "p1".to_string(),
            url: url.to_string(),
            status_code: Some(status),
            title: title.map(|s| s.to_string()),
            meta_description: None,
            h1: None,
            canonical: None,
            size_bytes: None,
            load_time_ms: None,
            is_indexable: Some(indexable),
            depth: 0,
            parent_url: None,
            crawl_timestamp: "2026-01-01T00:00:00Z".to_string(),
            links: Vec::new(),
            html_lang: None,
            hreflang_json: None,
            semantic_issues_json: None,
            html_body: None,
        }
    }

    fn filter(repo: &CrawlRepo, missing: bool, dup: bool, noindex: bool, is404: bool) -> Vec<String> {
        let (items, _) = repo
            .get_results("p1", 1, 100, None, None, None, None, None, None, missing, dup, noindex, is404)
            .unwrap();
        items.into_iter().map(|r| r.url).collect()
    }

    #[test]
    fn test_filter_missing_title() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("a", "https://x.com/a", Some("Title A"), 200, true),
            page("b", "https://x.com/b", None, 200, true),
            page("c", "https://x.com/c", Some("  "), 200, true),
        ])
        .unwrap();

        let urls = filter(&repo, true, false, false, false);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://x.com/b".to_string()));
        assert!(urls.contains(&"https://x.com/c".to_string()));
    }

    #[test]
    fn test_filter_duplicate_title() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("a", "https://x.com/a", Some("Dup"), 200, true),
            page("b", "https://x.com/b", Some("Dup"), 200, true),
            page("c", "https://x.com/c", Some("Unique"), 200, true),
            page("d", "https://x.com/d", None, 200, true),
        ])
        .unwrap();

        let urls = filter(&repo, false, true, false, false);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://x.com/a".to_string()));
        assert!(urls.contains(&"https://x.com/b".to_string()));
    }

    #[test]
    fn test_filter_noindex_only() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("a", "https://x.com/a", Some("A"), 200, true),
            page("b", "https://x.com/b", Some("B"), 200, false),
        ])
        .unwrap();

        let urls = filter(&repo, false, false, true, false);
        assert_eq!(urls, vec!["https://x.com/b"]);
    }

    #[test]
    fn test_filter_is_404() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("a", "https://x.com/a", Some("A"), 200, true),
            page("b", "https://x.com/b", Some("B"), 404, true),
            page("c", "https://x.com/c", Some("C"), 301, true),
        ])
        .unwrap();

        let urls = filter(&repo, false, false, false, true);
        assert_eq!(urls, vec!["https://x.com/b"]);
    }

    #[test]
    fn test_filter_combined() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("a", "https://x.com/a", None, 404, false),
            page("b", "https://x.com/b", Some("B"), 404, true),
            page("c", "https://x.com/c", None, 200, true),
        ])
        .unwrap();

        let urls = filter(&repo, true, false, true, true);
        assert_eq!(urls, vec!["https://x.com/a"]);
    }

    #[test]
    fn test_filter_none_returns_all() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("a", "https://x.com/a", None, 404, false),
            page("b", "https://x.com/b", Some("B"), 200, true),
        ])
        .unwrap();

        let urls = filter(&repo, false, false, false, false);
        assert_eq!(urls.len(), 2);
    }

    fn link(from: &str, to: &str) -> PageLink {
        PageLink {
            from_url: from.to_string(),
            to_url: to.to_string(),
            config_id: "cfg".to_string(),
            project_id: "p1".to_string(),
            link_type: "href".to_string(),
            anchor_text: None,
            is_follow: true,
        }
    }

    #[test]
    fn test_site_tree_roots() {
        let repo = test_repo();
        let mut p_a = page("a", "https://x.com/a", Some("A"), 200, true);
        p_a.depth = 0;
        let mut p_b = page("b", "https://x.com/b", Some("B"), 200, true);
        p_b.depth = 1;
        repo.save_results_batch(&[p_a, p_b]).unwrap();

        let roots = repo.get_site_tree("p1", None, 100).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].url, "https://x.com/a");
        assert!(!roots[0].has_children);
    }

    #[test]
    fn test_site_tree_children_internal_only() {
        let repo = test_repo();
        let mut p_a = page("a", "https://x.com/a", Some("A"), 200, true);
        p_a.depth = 0;
        let mut p_b = page("b", "https://x.com/b", Some("B"), 200, true);
        p_b.depth = 1;
        repo.save_results_batch(&[p_a, p_b]).unwrap();
        repo.save_links_batch(&[
            link("https://x.com/a", "https://x.com/b"),
            link("https://x.com/a", "https://external.com/x"),
        ])
        .unwrap();

        let children = repo.get_site_tree("p1", Some("https://x.com/a"), 100).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].url, "https://x.com/b");
        assert!(!children[0].has_children);

        let roots = repo.get_site_tree("p1", None, 100).unwrap();
        assert_eq!(roots.len(), 1);
        assert!(roots[0].has_children);
    }
}
