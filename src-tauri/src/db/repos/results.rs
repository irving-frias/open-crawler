use rusqlite::params;
use rusqlite::OptionalExtension;

use crate::error::AppError;
use crate::models::{CrawlResult, PageDetail, PageLink};
use crate::ResultsCacheKey;

use super::{compress_png, decompress_html_body, decompress_png, CrawlRepo};

impl<'a> CrawlRepo<'a> {
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

        let mut and_clauses = vec!["project_id = ?1".to_string()];
        let mut filter_clauses: Vec<String> = Vec::new();
        let mut param_index = 2u32;

        // Text search narrows whatever facet union is active, so it stays ANDed.
        if let Some(s) = search {
            if !s.is_empty() {
                and_clauses.push(format!(
                    "(url LIKE ?{} OR title LIKE ?{} OR h1 LIKE ?{})",
                    param_index, param_index, param_index
                ));
                param_index += 1;
            }
        }

        if let Some(_issue_type) = semantic_issue_type {
            filter_clauses.push(format!(
                "EXISTS (SELECT 1 FROM page_issues pi WHERE pi.page_id = crawled_pages.id AND pi.project_id = crawled_pages.project_id AND pi.issue_type = ?{})",
                param_index
            ));
            param_index += 1;
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
                filter_clauses.push(format!("status_code IN ({})", placeholders.join(",")));
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
                filter_clauses.push(format!(
                    "EXISTS (SELECT 1 FROM page_issues pi WHERE pi.page_id = crawled_pages.id AND pi.project_id = crawled_pages.project_id AND pi.severity IN ({}))",
                    placeholders.join(",")
                ));
            }
        }

        if let Some(domain) = domain_filter {
            if !domain.is_empty() {
                filter_clauses.push(format!(
                    "(url LIKE ?{} OR url LIKE ?{})",
                    param_index,
                    param_index + 1
                ));
                param_index += 2;
            }
        }

        if let Some(_depth) = depth_filter {
            filter_clauses.push(format!("depth <= ?{}", param_index));
            param_index += 1;
        }

        if missing_title {
            filter_clauses.push("(title IS NULL OR trim(title) = '')".to_string());
        }

        if duplicate_title {
            filter_clauses.push(
                "title IS NOT NULL AND trim(title) <> '' AND (SELECT COUNT(*) FROM crawled_pages c2 WHERE c2.project_id = crawled_pages.project_id AND c2.title = crawled_pages.title) > 1".to_string(),
            );
        }

        if noindex_only {
            filter_clauses.push("is_indexable = 0".to_string());
        }

        if is_404 {
            filter_clauses.push("status_code = 404".to_string());
        }

        // Selected filters are unions (OR): a page matches if it satisfies ANY
        // active filter. Only the project scoping and the text search are ANDed.
        if !filter_clauses.is_empty() {
            and_clauses.push(format!("({})", filter_clauses.join(" OR ")));
        }

        let where_sql = and_clauses.join(" AND ");
        let count_sql = format!("SELECT COUNT(*) FROM crawled_pages WHERE {}", where_sql);
        // Note: html_body is intentionally excluded from the list projection. It
        // is only served by get_page_detail/get_page_html, so list queries stay
        // small (less IPC payload, no per-row gzip+base64 decompression) and the
        // results cache does not hold megabytes of raw HTML.
        let query_sql = format!(
            "SELECT id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, readability_score, content_hash, duplicate_group_id, keywords_json, og_json, pagespeed_score, pagespeed_json, seo_score, seo_audit_json, blocked
             FROM crawled_pages WHERE {}
             ORDER BY crawl_timestamp DESC
             LIMIT ?{} OFFSET ?{}",
            where_sql, param_index, param_index + 1
        );

        // Build params for count query
        let mut count_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        count_params.push(Box::new(project_id.to_string()));
        if let Some(s) = search {
            if !s.is_empty() {
                count_params.push(Box::new(format!("%{}%", s)));
            }
        }
        if let Some(issue_type) = semantic_issue_type {
            count_params.push(Box::new(issue_type.to_string()));
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
        if let Some(s) = search {
            if !s.is_empty() {
                query_params.push(Box::new(format!("%{}%", s)));
            }
        }
        if let Some(issue_type) = semantic_issue_type {
            query_params.push(Box::new(issue_type.to_string()));
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
                Self::row_to_result_light,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(ref cache_arc) = self.results_cache {
            let mut cache = cache_arc.lock().unwrap();
            cache.put(cache_key, (results.clone(), total));
        }

        Ok((results, total))
    }

    /// Row mapper for the list query (get_results). Same projection as
    /// `row_to_result` but without `html_body` — the column is not selected.
    fn row_to_result_light(row: &rusqlite::Row) -> Result<CrawlResult, rusqlite::Error> {
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
            html_body: None,
            readability_score: row.get(18)?,
            content_hash: row.get(19)?,
            duplicate_group_id: row.get(20)?,
            keywords_json: row.get(21)?,
            og_json: row.get(22)?,
            pagespeed_score: row.get(23)?,
            pagespeed_json: row.get(24)?,
            seo_score: row.get(25)?,
            seo_audit_json: row.get(26)?,
            blocked: row.get::<_, i32>(27)? != 0,
        })
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
            readability_score: row.get(19)?,
            content_hash: row.get(20)?,
            duplicate_group_id: row.get(21)?,
            keywords_json: row.get(22)?,
            og_json: row.get(23)?,
            pagespeed_score: row.get(24)?,
            pagespeed_json: row.get(25)?,
            seo_score: row.get(26)?,
            seo_audit_json: row.get(27)?,
            blocked: row.get::<_, i32>(28)? != 0,
        })
    }

    pub fn get_page_detail(&self, page_id: &str) -> Result<PageDetail, AppError> {
        let result = self.conn.query_row(
            "SELECT id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body, readability_score, content_hash, duplicate_group_id, keywords_json, og_json, pagespeed_score, pagespeed_json, seo_score, seo_audit_json, blocked
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

        Ok(PageDetail {
            page: result,
            links,
        })
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

    pub fn find_page_id(&self, project_id: &str, url: &str) -> Result<Option<String>, AppError> {
        let id: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM crawled_pages WHERE project_id = ?1 AND url = ?2
                 ORDER BY crawl_timestamp DESC LIMIT 1",
                params![project_id, url],
                |row| row.get(0),
            )
            .optional()?;
        Ok(id)
    }

    pub fn list_page_id_urls(&self, project_id: &str) -> Result<Vec<(String, String)>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, url FROM crawled_pages WHERE project_id = ?1 ORDER BY url")?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}
