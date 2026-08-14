use rusqlite::params;

use crate::error::AppError;
use crate::models::{CrawlResult, PageLink};

use super::crawl::deserialize_rel_tokens;
use super::CrawlRepo;

impl<'a> CrawlRepo<'a> {
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
            "SELECT COUNT(*) FROM page_issues WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(n as u32)
    }

    /// Rows produced by the SEO audit export sheets: one per page×category,
    /// plus one per failing check and one per priority fix. Counted over the
    /// `seo_audit_json` blob so it stays correct even if the normalized
    /// `seo_category_scores` / `seo_check_issues` tables are out of sync.
    pub fn count_seo_rows(&self, project_id: &str) -> Result<u32, AppError> {
        let n: i64 = self.conn.query_row(
            "SELECT
                (SELECT COUNT(*) FROM crawled_pages p, json_each(p.seo_audit_json, '$.categories') c
                 WHERE p.project_id = ?1 AND p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json))
              + (SELECT COUNT(*) FROM crawled_pages p, json_each(p.seo_audit_json, '$.checks') c
                 WHERE p.project_id = ?1 AND p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json)
                   AND json_extract(c.value, '$.passed') = 0)
              + (SELECT COUNT(*) FROM crawled_pages p, json_each(p.seo_audit_json, '$.priority_fixes') c
                 WHERE p.project_id = ?1 AND p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json))",
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
        let cols = "id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, semantic_issues_json, blocked, seo_score, seo_audit_json, readability_score, keywords_json, og_json, pagespeed_score, pagespeed_json, hreflang_json, redirect_from_url, duplicate_group_id, response_headers_json";
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
                .query_map(
                    params![project_id, ts, id, limit],
                    Self::row_to_result_export,
                )?
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
            "SELECT rowid, from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow, rel_tokens, is_sponsored, is_ugc, is_internal
             FROM page_links WHERE project_id = ?1 AND rowid < ?2
             ORDER BY rowid DESC LIMIT ?3"
                .to_string()
        } else {
            "SELECT rowid, from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow, rel_tokens, is_sponsored, is_ugc, is_internal
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
            hreflang_json: row.get(25)?,
            semantic_issues_json: row.get(16)?,
            html_body: None,
            readability_score: row.get(20)?,
            content_hash: None,
            duplicate_group_id: row.get(27)?,
            keywords_json: row.get(21)?,
            og_json: row.get(22)?,
            pagespeed_score: row.get(23)?,
            pagespeed_json: row.get(24)?,
            seo_score: row.get(18)?,
            seo_audit_json: row.get(19)?,
            blocked: row.get::<_, i32>(17)? != 0,
            redirect_from_url: row.get(26)?,
            response_headers_json: row.get(28)?,
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
                rel_tokens: deserialize_rel_tokens(row.get::<_, Option<String>>(8)?.as_deref()),
                is_sponsored: row.get::<_, i32>(9)? != 0,
                is_ugc: row.get::<_, i32>(10)? != 0,
                is_internal: row.get::<_, i32>(11)? != 0,
            },
        ))
    }
}
