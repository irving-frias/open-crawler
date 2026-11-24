use rusqlite::{params, Transaction};

use tracing::info;

use crate::crawler::parser::SemanticIssue;
use crate::db::CrawlSessionInfo;
use crate::error::AppError;
use crate::models::{CrawlConfig, CrawlResult, PageLink, RedirectRecord};

use super::seo::{delete_seo_normalized, save_seo_normalized, save_seo_normalized_batch};
use super::{
    compress_html_body, delete_where_id_in, execute_multi_row_insert, row_value_in_groups, sql_box,
    CrawlRepo, MAX_IN_PAIRS_PER_STATEMENT,
};

/// Serializes rel tokens to a compact JSON array string (NULL when empty).
fn serialize_rel_tokens(tokens: &[String]) -> Option<String> {
    if tokens.is_empty() {
        None
    } else {
        serde_json::to_string(tokens).ok()
    }
}

/// Parses rel tokens previously stored by [`serialize_rel_tokens`].
pub fn deserialize_rel_tokens(raw: Option<&str>) -> Vec<String> {
    raw.and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .unwrap_or_default()
}

/// Deletes the normalized `page_issues` rows belonging to the given page ids
/// in a single statement. Re-crawls replace page rows per URL, and the
/// replaced rows' ids differ from the new ones, so stale issue rows must be
/// removed explicitly.
fn delete_page_issues(tx: &Transaction<'_>, page_ids: &[String]) -> Result<(), AppError> {
    delete_where_id_in(tx, "page_issues", "page_id", page_ids)
}

/// Deletes the materialized `page_keywords` rows for the given page ids in a
/// single statement.
fn delete_page_keywords(tx: &Transaction<'_>, page_ids: &[String]) -> Result<(), AppError> {
    delete_where_id_in(tx, "page_keywords", "page_id", page_ids)
}

/// Collects the `page_issues` rows for one page from its
/// `semantic_issues_json`, preserving the array position for stable ordering.
/// Invalid JSON yields no rows, matching the old per-row writer.
fn page_issue_rows(
    project_id: &str,
    page_id: &str,
    semantic_issues_json: &Option<String>,
) -> Vec<Vec<Box<dyn rusqlite::types::ToSql>>> {
    let Some(json) = semantic_issues_json else {
        return Vec::new();
    };
    let issues: Vec<SemanticIssue> = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    issues
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            vec![
                sql_box(project_id.to_string()),
                sql_box(page_id.to_string()),
                sql_box(issue.issue_type.clone()),
                sql_box(issue.severity.clone()),
                sql_box(issue.message.clone()),
                sql_box(issue.element.clone()),
                sql_box(issue.css_selector.clone()),
                sql_box(issue.xpath.clone()),
                sql_box(i as i64),
            ]
        })
        .collect()
}

/// Collects the materialized `page_keywords` rows for one page from its
/// `keywords_json` payload. Kept in sync with what the parser stored per page
/// so the project-wide keyword report is a plain GROUP BY.
fn page_keyword_rows(
    project_id: &str,
    page_id: &str,
    keywords_json: Option<&str>,
) -> Vec<Vec<Box<dyn rusqlite::types::ToSql>>> {
    let items: Vec<serde_json::Value> = match keywords_json {
        Some(json) => serde_json::from_str(json).unwrap_or_default(),
        None => Vec::new(),
    };
    items
        .iter()
        .filter_map(|item| {
            let keyword = item.get("keyword")?.as_str()?;
            let count = item.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as i64;
            Some(vec![
                sql_box(project_id.to_string()),
                sql_box(page_id.to_string()),
                sql_box(keyword.to_string()),
                sql_box(count),
            ])
        })
        .collect()
}

/// Writes one `page_issues` row per issue occurrence parsed from the page's
/// `semantic_issues_json`, as a single multi-row INSERT.
fn save_page_issues(
    tx: &Transaction<'_>,
    project_id: &str,
    page_id: &str,
    semantic_issues_json: &Option<String>,
) -> Result<(), AppError> {
    let rows = page_issue_rows(project_id, page_id, semantic_issues_json);
    execute_multi_row_insert(
        tx,
        "INSERT INTO page_issues (project_id, page_id, issue_type, severity, message, element, css_selector, xpath, position) VALUES",
        &rows,
    )
}

/// Writes the `page_issues` rows for every page in the batch as chunked
/// multi-row INSERTs instead of one statement per issue.
fn save_page_issues_batch(tx: &Transaction<'_>, results: &[&CrawlResult]) -> Result<(), AppError> {
    let mut rows = Vec::new();
    for result in results {
        rows.extend(page_issue_rows(
            &result.project_id,
            &result.id,
            &result.semantic_issues_json,
        ));
    }
    execute_multi_row_insert(
        tx,
        "INSERT INTO page_issues (project_id, page_id, issue_type, severity, message, element, css_selector, xpath, position) VALUES",
        &rows,
    )
}

/// Re-inserts the materialized keyword rows for the given page from its
/// `keywords_json` payload, as a single multi-row INSERT.
fn save_page_keywords(
    tx: &Transaction<'_>,
    project_id: &str,
    page_id: &str,
    keywords_json: Option<&str>,
) -> Result<(), AppError> {
    let rows = page_keyword_rows(project_id, page_id, keywords_json);
    execute_multi_row_insert(
        tx,
        "INSERT OR REPLACE INTO page_keywords (project_id, page_id, keyword, count) VALUES",
        &rows,
    )
}

/// Writes the materialized keyword rows for every page in the batch as
/// chunked multi-row INSERTs instead of one statement per keyword.
fn save_page_keywords_batch(
    tx: &Transaction<'_>,
    results: &[&CrawlResult],
) -> Result<(), AppError> {
    let mut rows = Vec::new();
    for result in results {
        rows.extend(page_keyword_rows(
            &result.project_id,
            &result.id,
            result.keywords_json.as_deref(),
        ));
    }
    execute_multi_row_insert(
        tx,
        "INSERT OR REPLACE INTO page_keywords (project_id, page_id, keyword, count) VALUES",
        &rows,
    )
}

/// Deletes rows for the given `(project_id, url)` pairs from `table` with one
/// statement per chunk, matching on `url_column` (e.g. `url` on
/// `crawled_pages`, `from_url` on `page_links`).
fn delete_by_project_url_pairs(
    tx: &Transaction<'_>,
    table: &str,
    url_column: &str,
    pairs: &[(&str, &str)],
) -> Result<(), AppError> {
    for chunk in pairs.chunks(MAX_IN_PAIRS_PER_STATEMENT) {
        let sql = format!(
            "DELETE FROM {table} WHERE (project_id, {url_column}) IN ({})",
            row_value_in_groups(chunk.len())
        );
        let params: Vec<Box<dyn rusqlite::types::ToSql>> = chunk
            .iter()
            .flat_map(|(project_id, url)| {
                [sql_box(project_id.to_string()), sql_box(url.to_string())]
            })
            .collect();
        tx.execute(
            &sql,
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        )?;
    }
    Ok(())
}

impl<'a> CrawlRepo<'a> {
    pub fn save_config(&self, config: &CrawlConfig) -> Result<(), AppError> {
        let id = config.id.as_deref().unwrap_or("default");
        let project_id = config.project_id.as_deref().unwrap_or("default");
        let seed_urls = serde_json::to_string(&config.seed_urls)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO crawl_config (id, project_id, seed_urls, max_pages, max_depth, user_agent, respect_robots, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
            params![
                id,
                project_id,
                seed_urls,
                1000i32,
                config.max_depth as i32,
                config.user_agent(),
                config.respect_robots as i32,
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
             (id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body, readability_score, content_hash, duplicate_group_id, keywords_json, og_json, pagespeed_score, pagespeed_json, seo_score, seo_audit_json, blocked, response_headers_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30)",
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
                result.response_headers_json,
            ],
        )?;

        if !result.links.is_empty() {
            let rows: Vec<Vec<Box<dyn rusqlite::types::ToSql>>> = result
                .links
                .iter()
                .map(|link| {
                    vec![
                        sql_box(link.from_url.clone()),
                        sql_box(link.to_url.clone()),
                        sql_box(link.config_id.clone()),
                        sql_box(project_id.clone()),
                        sql_box(link.link_type.clone()),
                        sql_box(link.anchor_text.clone()),
                        sql_box(link.is_follow as i32),
                    ]
                })
                .collect();
            execute_multi_row_insert(
                &tx,
                "INSERT INTO page_links (from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow) VALUES",
                &rows,
            )?;
        }

        save_page_issues(&tx, project_id, &result.id, &result.semantic_issues_json)?;
        delete_seo_normalized(&tx, &old_ids)?;
        delete_seo_normalized(&tx, std::slice::from_ref(&result.id))?;
        save_seo_normalized(&tx, project_id, &result.id, &result.seo_audit_json)?;
        delete_page_keywords(&tx, &old_ids)?;
        delete_page_keywords(&tx, std::slice::from_ref(&result.id))?;
        save_page_keywords(&tx, project_id, &result.id, result.keywords_json.as_deref())?;

        tx.commit()?;
        self.invalidate_cache_for_project(project_id);

        Ok(())
    }

    /// Persists a whole flush in **one** transaction with batched SQL: a
    /// single `SELECT ... WHERE (project_id, url) IN (...)` for the ids being
    /// replaced, single `DELETE ... IN (...)` statements for the replaced
    /// pages and their stale outbound links, and chunked multi-row INSERTs
    /// for the pages themselves plus their normalized rows (issues, SEO,
    /// keywords). Final table contents are identical to saving each page
    /// with [`CrawlRepo::save_result`].
    ///
    /// Note: `result.links` is deliberately *not* persisted here — the
    /// crawler sends links separately (`CrawlResultMsg::Links`) and they go
    /// through [`CrawlRepo::save_links_batch`]; inserting them here too
    /// would duplicate every link.
    pub fn save_results_batch(&self, results: &[CrawlResult]) -> Result<(), AppError> {
        if results.is_empty() {
            return Ok(());
        }

        let tx = self.conn.unchecked_transaction()?;

        // Deduplicate within the batch keeping the last row per (project_id,
        // url), then work in deterministic (project_id, url) order. The old
        // code iterated a HashMap, so insertion order was arbitrary; final
        // table contents don't depend on it.
        let mut by_key: std::collections::HashMap<(String, String), &CrawlResult> =
            std::collections::HashMap::new();
        for result in results {
            by_key.insert((result.project_id.clone(), result.url.clone()), result);
        }
        let mut unique: Vec<&CrawlResult> = by_key.into_values().collect();
        unique.sort_by(|a, b| {
            (a.project_id.as_str(), a.url.as_str()).cmp(&(b.project_id.as_str(), b.url.as_str()))
        });
        let pairs: Vec<(&str, &str)> = unique
            .iter()
            .map(|r| (r.project_id.as_str(), r.url.as_str()))
            .collect();

        // Capture the ids of the rows being replaced so their normalized
        // rows can be removed alongside them, in one SELECT per chunk.
        let mut old_ids: Vec<String> = Vec::new();
        for chunk in pairs.chunks(MAX_IN_PAIRS_PER_STATEMENT) {
            let sql = format!(
                "SELECT id FROM crawled_pages WHERE (project_id, url) IN ({})",
                row_value_in_groups(chunk.len())
            );
            let params: Vec<Box<dyn rusqlite::types::ToSql>> = chunk
                .iter()
                .flat_map(|(project_id, url)| {
                    [sql_box(project_id.to_string()), sql_box(url.to_string())]
                })
                .collect();
            let mut stmt = tx.prepare(&sql)?;
            let ids: Vec<String> = stmt
                .query_map(
                    rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
                    |row| row.get(0),
                )?
                .collect::<Result<Vec<_>, _>>()?;
            old_ids.extend(ids);
        }

        // Drop the normalized rows of the replaced pages — and of the new
        // ids too, mirroring `save_result`, which deletes for both: a
        // re-crawl that reuses an id must not leave rows from the old page
        // behind. All of this happens before the new rows are inserted.
        let mut stale_ids: Vec<String> = old_ids;
        stale_ids.extend(unique.iter().map(|r| r.id.clone()));
        stale_ids.sort();
        stale_ids.dedup();
        delete_page_issues(&tx, &stale_ids)?;
        delete_seo_normalized(&tx, &stale_ids)?;
        delete_page_keywords(&tx, &stale_ids)?;

        // Remove the replaced page rows and their stale outbound links so
        // re-crawls replace instead of accumulating duplicates (unique index
        // idx_pages_project_url). One DELETE per chunk each.
        delete_by_project_url_pairs(&tx, "crawled_pages", "url", &pairs)?;
        delete_by_project_url_pairs(&tx, "page_links", "from_url", &pairs)?;

        // Multi-row INSERT of the pages themselves (30 columns, chunked so
        // each statement stays under the SQLite bind-parameter limit).
        let rows: Vec<Vec<Box<dyn rusqlite::types::ToSql>>> = unique
            .iter()
            .map(|result| {
                vec![
                    sql_box(result.id.clone()),
                    sql_box(result.config_id.clone()),
                    sql_box(result.project_id.clone()),
                    sql_box(result.url.clone()),
                    sql_box(result.status_code.map(|s| s as i32)),
                    sql_box(result.title.clone()),
                    sql_box(result.meta_description.clone()),
                    sql_box(result.h1.clone()),
                    sql_box(result.canonical.clone()),
                    sql_box(result.size_bytes.map(|s| s as i64)),
                    sql_box(result.load_time_ms),
                    sql_box(result.is_indexable.map(|i| i as i32)),
                    sql_box(result.depth as i32),
                    sql_box(result.parent_url.clone()),
                    sql_box(result.crawl_timestamp.clone()),
                    sql_box(result.html_lang.clone()),
                    sql_box(result.hreflang_json.clone()),
                    sql_box(result.semantic_issues_json.clone()),
                    sql_box(compress_html_body(&result.html_body)),
                    sql_box(result.readability_score),
                    sql_box(result.content_hash.clone()),
                    sql_box(result.duplicate_group_id),
                    sql_box(result.keywords_json.clone()),
                    sql_box(result.og_json.clone()),
                    sql_box(result.pagespeed_score),
                    sql_box(result.pagespeed_json.clone()),
                    sql_box(result.seo_score),
                    sql_box(result.seo_audit_json.clone()),
                    sql_box(result.blocked as i32),
                    sql_box(result.response_headers_json.clone()),
                ]
            })
            .collect();
        execute_multi_row_insert(
            &tx,
            "INSERT INTO crawled_pages
             (id, config_id, project_id, url, status_code, title, meta_description, h1, canonical, size_bytes, load_time_ms, is_indexable, depth, parent_url, crawl_timestamp, html_lang, hreflang_json, semantic_issues_json, html_body, readability_score, content_hash, duplicate_group_id, keywords_json, og_json, pagespeed_score, pagespeed_json, seo_score, seo_audit_json, blocked, response_headers_json)
             VALUES",
            &rows,
        )?;

        // Normalized rows for the freshly written pages, all multi-row.
        save_page_issues_batch(&tx, &unique)?;
        let seo_pages: Vec<(&str, &str, &Option<String>)> = unique
            .iter()
            .map(|r| (r.project_id.as_str(), r.id.as_str(), &r.seo_audit_json))
            .collect();
        save_seo_normalized_batch(&tx, &seo_pages)?;
        save_page_keywords_batch(&tx, &unique)?;

        tx.commit()?;
        info!("Batch saved {} results", results.len());

        let project_ids: std::collections::HashSet<String> =
            results.iter().map(|r| r.project_id.clone()).collect();
        for project_id in project_ids {
            self.invalidate_cache_for_project(&project_id);
        }

        Ok(())
    }

    /// Persists links as chunked multi-row INSERTs instead of one statement
    /// per link. Same rows, same column values as the old per-link loop.
    pub fn save_links_batch(&self, links: &[PageLink]) -> Result<(), AppError> {
        if links.is_empty() {
            return Ok(());
        }

        let tx = self.conn.unchecked_transaction()?;

        let rows: Vec<Vec<Box<dyn rusqlite::types::ToSql>>> = links
            .iter()
            .map(|link| {
                vec![
                    sql_box(link.from_url.clone()),
                    sql_box(link.to_url.clone()),
                    sql_box(link.config_id.clone()),
                    sql_box(link.project_id.clone()),
                    sql_box(link.link_type.clone()),
                    sql_box(link.anchor_text.clone()),
                    sql_box(link.is_follow as i32),
                    sql_box(serialize_rel_tokens(&link.rel_tokens)),
                    sql_box(link.is_sponsored as i32),
                    sql_box(link.is_ugc as i32),
                    sql_box(link.is_internal as i32),
                ]
            })
            .collect();
        execute_multi_row_insert(
            &tx,
            "INSERT INTO page_links (from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow, rel_tokens, is_sponsored, is_ugc, is_internal) VALUES",
            &rows,
        )?;

        tx.commit()?;
        info!("Batch saved {} links", links.len());

        let project_ids: std::collections::HashSet<String> =
            links.iter().map(|l| l.project_id.clone()).collect();
        for project_id in project_ids {
            self.invalidate_cache_for_project(&project_id);
        }

        Ok(())
    }

    /// Persists redirect chains captured by the fetcher. Also back-fills the
    /// `redirect_from_url` column on the reached page so the redirect origin is
    /// queryable without joining `page_redirects`.
    pub fn save_redirect_batch(&self, redirects: &[RedirectRecord]) -> Result<(), AppError> {
        if redirects.is_empty() {
            return Ok(());
        }

        let tx = self.conn.unchecked_transaction()?;
        {
            let mut update_page =
                tx.prepare("UPDATE crawled_pages SET redirect_from_url = ?1 WHERE id = ?2")?;
            let mut insert_hop = tx.prepare(
                "INSERT INTO page_redirects (project_id, page_id, hop_index, from_url, to_url, status_code)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for record in redirects {
                if let Some(from) = &record.redirect_from_url {
                    update_page.execute(params![from, record.page_id])?;
                }
                for (index, hop) in record.chain.iter().enumerate() {
                    insert_hop.execute(params![
                        record.project_id,
                        record.page_id,
                        index as i64,
                        hop.from_url,
                        hop.to_url,
                        hop.status_code as i64,
                    ])?;
                }
            }
        }
        tx.commit()?;
        info!("Batch saved {} redirect records", redirects.len());

        let project_ids: std::collections::HashSet<String> =
            redirects.iter().map(|r| r.project_id.clone()).collect();
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

    /// Discards an interrupted session for a fresh start: it no longer
    /// counts as resumable and its saved queue is dropped, so the next
    /// crawl starts from the seed URLs instead of silently resuming.
    pub fn discard_session(&self, session_id: &str) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE crawl_sessions SET status = 'discarded', updated_at = ?1 WHERE id = ?2",
            params![now, session_id],
        )?;
        self.clear_queue(session_id)?;
        info!("Discarded crawl session: {}", session_id);
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
