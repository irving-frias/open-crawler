pub mod alerts;
pub mod analytics;
pub mod crawl;
pub mod export;
pub mod extraction;
pub mod links;
pub mod maintenance;
pub mod pagespeed;
pub mod projects;
pub mod results;
pub mod schedule;
pub mod seo;
pub mod settings;
pub mod snapshots;

use rusqlite::types::ToSql;
use rusqlite::Connection;

use crate::error::AppError;
use crate::{ResultsCacheArc, ResultsCacheKey};

pub fn compress_gzip(data: &[u8]) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

pub(crate) fn decompress_gzip(data: &[u8]) -> Vec<u8> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).unwrap();
    out
}

pub(crate) fn compress_html_body(html: &Option<String>) -> Option<String> {
    html.as_ref().map(|s| {
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            compress_gzip(s.as_bytes()),
        )
    })
}

pub(crate) fn decompress_html_body(encoded: &Option<String>) -> Option<String> {
    encoded
        .as_ref()
        .and_then(|e| base64::Engine::decode(&base64::engine::general_purpose::STANDARD, e).ok())
        .and_then(|bytes| String::from_utf8(decompress_gzip(&bytes)).ok())
}

pub(crate) fn compress_png(png: &Option<Vec<u8>>) -> Option<Vec<u8>> {
    png.as_ref().map(|data| compress_gzip(data))
}

pub(crate) fn decompress_png(data: &Option<Vec<u8>>) -> Option<Vec<u8>> {
    data.as_ref().map(|bytes| decompress_gzip(bytes))
}

/// Boxes one bind value for a multi-row insert row.
pub(crate) fn sql_box<T: ToSql + 'static>(value: T) -> Box<dyn ToSql> {
    Box::new(value)
}

/// Max bind parameters per statement. SQLite's default
/// `SQLITE_MAX_VARIABLE_NUMBER` is 999 on some builds (32766 on newer ones),
/// so chunked multi-row writes stay under 900 to be safe on either.
const MAX_SQL_PARAMS_PER_STATEMENT: usize = 900;

/// One `?` placeholder group per row, e.g. `(?,?)` for a 2-column row.
fn row_placeholders(columns: usize) -> String {
    let marks = std::iter::repeat_n("?", columns)
        .collect::<Vec<_>>()
        .join(",");
    format!("({marks})")
}

/// Executes `INSERT <prefix> VALUES (...),(...),...` for all `rows`, split
/// into chunks that stay under [`MAX_SQL_PARAMS_PER_STATEMENT`] bind
/// parameters. Each inner vec holds one row's values in column order; a row
/// with a different length than the first is a programming error and fails
/// the whole call instead of writing a partial batch.
pub(crate) fn execute_multi_row_insert(
    conn: &Connection,
    prefix: &str,
    rows: &[Vec<Box<dyn ToSql>>],
) -> Result<(), AppError> {
    if rows.is_empty() {
        return Ok(());
    }
    let columns = rows[0].len();
    if columns == 0 || rows.iter().any(|r| r.len() != columns) {
        return Err(AppError::Crawl(
            "multi-row insert: all rows must have the same non-zero column count".to_string(),
        ));
    }
    let rows_per_chunk = (MAX_SQL_PARAMS_PER_STATEMENT / columns).max(1);
    let group = row_placeholders(columns);
    for chunk in rows.chunks(rows_per_chunk) {
        let sql = format!(
            "{prefix} {}",
            std::iter::repeat_n(group.as_str(), chunk.len())
                .collect::<Vec<_>>()
                .join(",")
        );
        let params: Vec<&dyn ToSql> = chunk
            .iter()
            .flat_map(|r| r.iter().map(|v| v.as_ref()))
            .collect();
        conn.execute(&sql, rusqlite::params_from_iter(params))?;
    }
    Ok(())
}

/// `DELETE FROM <table> WHERE <column> IN (?, ?, ...)` as a single
/// statement, chunked like [`execute_multi_row_insert`]. No-op on an empty
/// id list. `table`/`column` are internal identifiers, never user input.
pub(crate) fn delete_where_id_in(
    conn: &Connection,
    table: &str,
    column: &str,
    ids: &[String],
) -> Result<(), AppError> {
    if ids.is_empty() {
        return Ok(());
    }
    for chunk in ids.chunks(MAX_SQL_PARAMS_PER_STATEMENT) {
        let marks = std::iter::repeat_n("?", chunk.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("DELETE FROM {table} WHERE {column} IN ({marks})");
        let params: Vec<&dyn ToSql> = chunk.iter().map(|id| id as &dyn ToSql).collect();
        conn.execute(&sql, rusqlite::params_from_iter(params))?;
    }
    Ok(())
}

/// `(?,?),(?,?),...` row-value groups for `WHERE (a, b) IN (...)`, one
/// group per `(project_id, url)` pair. Callers chunk the pairs so the
/// statement stays under [`MAX_SQL_PARAMS_PER_STATEMENT`] parameters.
pub(crate) fn row_value_in_groups(pairs: usize) -> String {
    std::iter::repeat_n("(?,?)", pairs)
        .collect::<Vec<_>>()
        .join(",")
}

/// Max `(project_id, url)` pairs per batched `IN (...)` statement (2
/// parameters per pair, under [`MAX_SQL_PARAMS_PER_STATEMENT`]).
pub(crate) const MAX_IN_PAIRS_PER_STATEMENT: usize = 400;

pub struct CrawlRepo<'a> {
    conn: &'a Connection,
    results_cache: Option<&'a ResultsCacheArc>,
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
    pub fn new(conn: &'a Connection, results_cache: Option<&'a ResultsCacheArc>) -> Self {
        Self {
            conn,
            results_cache,
        }
    }

    /// Exposes the raw SQLite connection for low-level operations (e.g. the
    /// transfer/package module's `VACUUM INTO` and `ATTACH`).
    pub fn connection(&self) -> &'a Connection {
        self.conn
    }

    pub(crate) fn invalidate_cache_for_project(&self, project_id: &str) {
        if let Some(cache_arc) = self.results_cache {
            let mut cache = cache_arc.lock().unwrap();
            let keys_to_remove: Vec<ResultsCacheKey> = cache
                .iter()
                .filter(|(k, _)| k.project_id == project_id)
                .map(|(k, _)| k.clone())
                .collect();
            for key in keys_to_remove {
                cache.pop(&key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;
    use crate::models::{CrawlResult, PageLink};

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
            blocked: false,
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
            readability_score: None,
            content_hash: None,
            duplicate_group_id: None,
            keywords_json: None,
            og_json: None,
            pagespeed_score: None,
            pagespeed_json: None,
            seo_score: None,
            seo_audit_json: None,
            redirect_from_url: None,
            response_headers_json: None,
            custom_extractions_json: None,
        }
    }

    fn filter(
        repo: &CrawlRepo,
        missing: bool,
        dup: bool,
        noindex: bool,
        is404: bool,
    ) -> Vec<String> {
        let (items, _) = repo
            .get_results(
                "p1", 1, 100, None, None, None, None, None, None, missing, dup, noindex, is404,
            )
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

        // Filters are unions (OR): a page matches ANY active filter.
        let urls = filter(&repo, true, false, true, true);
        assert_eq!(urls.len(), 3);
        assert!(urls.contains(&"https://x.com/a".to_string()));
        assert!(urls.contains(&"https://x.com/b".to_string()));
        assert!(urls.contains(&"https://x.com/c".to_string()));
    }

    #[test]
    fn test_filter_or_across_categories() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("a", "https://x.com/a", Some("A"), 200, true),
            page("b", "https://x.com/b", None, 404, true),
            page("c", "https://x.com/c", Some("C"), 301, true),
        ])
        .unwrap();

        // status IN (200, 301) OR missing title -> all three pages match.
        let (items, total) = repo
            .get_results(
                "p1",
                1,
                100,
                None,
                None,
                Some(&[200, 301]),
                None,
                None,
                None,
                true,
                false,
                false,
                false,
            )
            .unwrap();
        assert_eq!(total, 3);
        let urls: Vec<String> = items.into_iter().map(|r| r.url).collect();
        assert!(urls.contains(&"https://x.com/a".to_string()));
        assert!(urls.contains(&"https://x.com/b".to_string()));
        assert!(urls.contains(&"https://x.com/c".to_string()));
    }

    #[test]
    fn test_filter_or_still_scoped_to_project() {
        let repo = test_repo();
        repo.conn
            .execute_batch(
                "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('p2', 'P2', datetime('now'), datetime('now'));",
            )
            .unwrap();
        let mut p_a = page("a", "https://x.com/a", Some("A"), 200, true);
        p_a.project_id = "p2".to_string();
        repo.save_results_batch(&[p_a]).unwrap();

        // A page outside the project must not leak in via the OR union.
        let (_, total) = repo
            .get_results(
                "p1",
                1,
                100,
                None,
                None,
                Some(&[200]),
                None,
                None,
                None,
                false,
                false,
                false,
                false,
            )
            .unwrap();
        assert_eq!(total, 0);
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
            rel_tokens: Vec::new(),
            is_sponsored: false,
            is_ugc: false,
            is_internal: false,
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

        let children = repo
            .get_site_tree("p1", Some("https://x.com/a"), 100)
            .unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].url, "https://x.com/b");
        assert!(!children[0].has_children);

        let roots = repo.get_site_tree("p1", None, 100).unwrap();
        assert_eq!(roots.len(), 1);
        assert!(roots[0].has_children);
    }

    #[test]
    fn test_site_tree_pages_stream_batches() {
        let repo = test_repo();
        let mut pages = Vec::new();
        for i in 0..5 {
            let mut p = page(
                &format!("p{i}"),
                &format!("https://x.com/page-{i}"),
                Some(&format!("Page {i}")),
                200,
                true,
            );
            p.depth = i as u32;
            pages.push(p);
        }
        repo.save_results_batch(&pages).unwrap();

        // First batch (keyset from the start, ordered by URL).
        let (first, total) = repo.get_site_tree_pages("p1", None, 2).unwrap();
        assert_eq!(total, 5);
        assert_eq!(first.len(), 2);
        assert!(first[0].url < first[1].url);

        // Second batch continues right after the last URL of the previous one.
        let (second, _) = repo
            .get_site_tree_pages("p1", Some(&first.last().unwrap().url), 2)
            .unwrap();
        assert_eq!(second.len(), 2);
        assert!(second[0].url > first.last().unwrap().url);
        assert!(second[0].url < second[1].url);

        // Third batch drains the remainder.
        let (third, _) = repo
            .get_site_tree_pages("p1", Some(&second.last().unwrap().url), 2)
            .unwrap();
        assert_eq!(third.len(), 1);

        // Collecting everything reconstructs the full sorted set.
        let all: Vec<String> = first
            .iter()
            .chain(second.iter())
            .chain(third.iter())
            .map(|n| n.url.clone())
            .collect();
        let mut sorted = all.clone();
        sorted.sort();
        assert_eq!(all, sorted);
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_duplicate_groups() {
        let repo = test_repo();
        let mut p_a = page("a", "https://x.com/a", Some("A"), 200, true);
        p_a.content_hash = Some(format!("{:016x}", 0x1234_u64));
        let mut p_b = page("b", "https://x.com/b", Some("B"), 200, true);
        p_b.content_hash = Some(format!("{:016x}", 0x1234_u64));
        let mut p_c = page("c", "https://x.com/c", Some("C"), 200, true);
        p_c.content_hash = Some(format!("{:016x}", 0xFFFF_u64));
        repo.save_results_batch(&[p_a, p_b, p_c]).unwrap();

        let groups = repo.compute_duplicate_groups("p1").unwrap();
        assert_eq!(groups, 1);

        let groups = repo.get_duplicate_groups("p1").unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].size, 2);
        let urls: Vec<&str> = groups[0].urls.iter().map(|u| u.url.as_str()).collect();
        assert!(urls.contains(&"https://x.com/a"));
        assert!(urls.contains(&"https://x.com/b"));
    }

    #[test]
    fn test_duplicate_groups_pagination() {
        let repo = test_repo();
        // Two groups: a/b (hash 0x0000) and c/d (hash 0xFFFF). The hashes are
        // 16 bits apart so the LSH similarity pass never merges the groups.
        for (id, url, hash) in [
            ("a", "https://x.com/a", 0x0000_u64),
            ("b", "https://x.com/b", 0x0000_u64),
            ("c", "https://x.com/c", 0xFFFF_u64),
            ("d", "https://x.com/d", 0xFFFF_u64),
        ] {
            let mut p = page(id, url, Some(id), 200, true);
            p.content_hash = Some(format!("{:016x}", hash));
            repo.save_results_batch(&[p]).unwrap();
        }
        repo.compute_duplicate_groups("p1").unwrap();

        // Page size 1 returns one group per page plus the correct total.
        let (page1, total) = repo.get_duplicate_groups_page("p1", 1, 1).unwrap();
        assert_eq!(total, 2);
        assert_eq!(page1.len(), 1);
        assert_eq!(page1[0].size, 2);

        let (page2, total) = repo.get_duplicate_groups_page("p1", 2, 1).unwrap();
        assert_eq!(total, 2);
        assert_eq!(page2.len(), 1);
        assert_ne!(
            page1[0].id, page2[0].id,
            "pages must return distinct groups"
        );

        let (all, total) = repo.get_duplicate_groups_page("p1", 1, 10).unwrap();
        assert_eq!(total, 2);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_duplicate_groups_dedupe_same_url() {
        let repo = test_repo();
        // Two rows with the same URL and matching content hash (e.g. from re-crawls)
        for id in ["a1", "a2"] {
            let mut p = page(id, "https://x.com/dup", Some("Dup"), 200, true);
            p.content_hash = Some(format!("{:016x}", 0xAAAA_u64));
            repo.save_results_batch(&[p]).unwrap();
        }
        repo.compute_duplicate_groups("p1").unwrap();

        let groups = repo.get_duplicate_groups("p1").unwrap();
        assert!(groups.is_empty(), "same-URL rows must not form a group");

        // Now a real second URL joins the group
        let mut p_b = page("b", "https://x.com/other", Some("Other"), 200, true);
        p_b.content_hash = Some(format!("{:016x}", 0xAAAA_u64));
        repo.save_results_batch(&[p_b]).unwrap();
        repo.compute_duplicate_groups("p1").unwrap();

        let groups = repo.get_duplicate_groups("p1").unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].urls.len(), 2);
        let urls: Vec<&str> = groups[0].urls.iter().map(|u| u.url.as_str()).collect();
        assert_eq!(urls, vec!["https://x.com/dup", "https://x.com/other"]);
    }

    fn page_with_issues(id: &str, url: &str, json: Option<&str>) -> CrawlResult {
        let mut p = page(id, url, Some("Title"), 200, true);
        p.semantic_issues_json = json.map(|s| s.to_string());
        p
    }

    #[test]
    fn test_page_issues_normalized_and_filterable() {
        let repo = test_repo();
        let issues = r#"[
            {"issue_type":"missing_title","severity":"error","element":"head","message":"Missing title"},
            {"issue_type":"img_no_alt","severity":"warning","element":"img","message":"Image without alt"}
        ]"#;
        repo.save_results_batch(&[page_with_issues("a", "https://x.com/a", Some(issues))])
            .unwrap();

        let count: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM page_issues WHERE page_id = 'a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2, "each issue occurrence becomes a page_issues row");

        let (items, _) = repo
            .get_results(
                "p1",
                1,
                100,
                None,
                None,
                None,
                Some(&["warning".to_string()]),
                None,
                None,
                false,
                false,
                false,
                false,
            )
            .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].url, "https://x.com/a");

        let (items, _) = repo
            .get_results(
                "p1",
                1,
                100,
                Some("img_no_alt"),
                None,
                None,
                None,
                None,
                None,
                false,
                false,
                false,
                false,
            )
            .unwrap();
        assert_eq!(items.len(), 1);

        // A page without issues must not match the issue-type filter.
        repo.save_results_batch(&[page("b", "https://x.com/b", Some("B"), 200, true)])
            .unwrap();
        let (items, _) = repo
            .get_results(
                "p1",
                1,
                100,
                Some("img_no_alt"),
                None,
                None,
                None,
                None,
                None,
                false,
                false,
                false,
                false,
            )
            .unwrap();
        assert_eq!(items.len(), 1);

        // Severity aggregation exposes warning + info, not just errors.
        let counts = repo.get_semantic_issue_counts("p1").unwrap();
        let pairs: Vec<(String, String, u32)> = counts
            .iter()
            .map(|c| (c.issue_type.clone(), c.severity.clone(), c.count))
            .collect();
        assert!(pairs.contains(&("missing_title".to_string(), "error".to_string(), 1)));
        assert!(pairs.contains(&("img_no_alt".to_string(), "warning".to_string(), 1)));
    }

    #[test]
    fn test_recrawl_replaces_page_issues() {
        let repo = test_repo();
        let issues = r#"[{"issue_type":"missing_title","severity":"error","element":"head","message":"Missing title"}]"#;
        repo.save_results_batch(&[page_with_issues("oldid", "https://x.com/a", Some(issues))])
            .unwrap();

        // Re-crawl of the same URL with a new id: old issue rows must be gone.
        repo.save_results_batch(&[page_with_issues("newid", "https://x.com/a", None)])
            .unwrap();

        let count: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM page_issues WHERE page_id = 'oldid'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            count, 0,
            "stale page_issues for the replaced row must be removed"
        );

        let count: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM page_issues WHERE page_id = 'newid'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    /// Dumps a table as sorted `value | value | ...` lines (one per row) so
    /// two databases can be compared cell by cell. `sql` must ORDER BY a
    /// deterministic key; autoincrement ids are excluded by the caller where
    /// insertion order may legitimately differ between write paths.
    fn snapshot(conn: &Connection, sql: &str) -> Vec<String> {
        let mut stmt = conn.prepare(sql).unwrap();
        let width = stmt.column_count();
        stmt.query_map([], |row| {
            let mut parts = Vec::with_capacity(width);
            for i in 0..width {
                parts.push(format!(
                    "{:?}",
                    row.get::<_, rusqlite::types::Value>(i).unwrap()
                ));
            }
            Ok(parts.join(" | "))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
    }

    fn snapshot_all(repo: &CrawlRepo) -> Vec<(String, Vec<String>)> {
        vec![
            (
                "crawled_pages".to_string(),
                snapshot(
                    repo.conn,
                    "SELECT * FROM crawled_pages ORDER BY project_id, url",
                ),
            ),
            (
                "page_issues".to_string(),
                snapshot(
                    repo.conn,
                    "SELECT project_id, page_id, issue_type, severity, message, element, css_selector, xpath, position \
                     FROM page_issues ORDER BY project_id, page_id, position, issue_type",
                ),
            ),
            (
                "page_keywords".to_string(),
                snapshot(
                    repo.conn,
                    "SELECT * FROM page_keywords ORDER BY project_id, page_id, keyword",
                ),
            ),
            (
                "seo_category_scores".to_string(),
                snapshot(
                    repo.conn,
                    "SELECT * FROM seo_category_scores ORDER BY page_id, category",
                ),
            ),
            (
                "seo_check_issues".to_string(),
                snapshot(
                    repo.conn,
                    "SELECT * FROM seo_check_issues ORDER BY page_id, check_id",
                ),
            ),
            (
                // Only the 7 columns `save_result` writes: `save_links_batch`
                // additionally fills rel_tokens/is_sponsored/is_ugc/is_internal,
                // which the dedicated links test below covers.
                "page_links".to_string(),
                snapshot(
                    repo.conn,
                    "SELECT from_url, to_url, config_id, project_id, link_type, anchor_text, is_follow \
                     FROM page_links ORDER BY from_url, to_url, link_type, anchor_text",
                ),
            ),
        ]
    }

    fn audit_json_v1() -> String {
        serde_json::json!({
            "score": 72.0,
            "grade": "C",
            "categories": [
                {"category": "meta", "score": 50.0, "weight": 0.25, "passed_weight": 10.0, "total_weight": 20.0, "passed_checks": 1, "total_checks": 2},
                {"category": "technical", "score": 90.0, "weight": 0.20, "passed_weight": 18.0, "total_weight": 20.0, "passed_checks": 2, "total_checks": 2},
                {"category": "social", "score": null, "weight": 0.07, "passed_weight": 0.0, "total_weight": 0.0, "passed_checks": 0, "total_checks": 0}
            ],
            "checks": [
                {"id": "title_present", "category": "meta", "severity": "error", "passed": false,
                 "weight": 1.0, "message": "Missing title", "guidance": "Add a title",
                 "examples": [{"issue_type": "missing_title", "severity": "error", "element": "head", "message": "x"}]},
                {"id": "meta_desc_present", "category": "meta", "severity": "warning", "passed": false,
                 "weight": 1.0, "message": "Short desc", "guidance": "Lengthen it"},
                {"id": "canonical_ok", "category": "technical", "severity": "info", "passed": true,
                 "weight": 1.0, "message": "ok", "guidance": "ok"}
            ],
            "priority_fixes": [
                {"id": "title_present", "priority": "critical", "message": "m", "guidance": "g", "category": "meta"},
                {"id": "meta_desc_present", "priority": "important", "message": "m2", "guidance": "g2", "category": "meta"}
            ]
        })
        .to_string()
    }

    fn audit_json_v2() -> String {
        serde_json::json!({
            "score": 88.0,
            "grade": "B",
            "categories": [
                {"category": "meta", "score": 88.0, "weight": 0.25, "passed_weight": 17.0, "total_weight": 20.0, "passed_checks": 2, "total_checks": 2}
            ],
            "checks": [
                {"id": "title_present", "category": "meta", "severity": "error", "passed": true,
                 "weight": 1.0, "message": "ok", "guidance": "ok"}
            ],
            "priority_fixes": []
        })
        .to_string()
    }

    fn rich_page(
        id: &str,
        url: &str,
        title: &str,
        issues_json: Option<&str>,
        keywords_json: Option<&str>,
        audit_json: Option<String>,
        seo_score: Option<f64>,
    ) -> CrawlResult {
        let mut p = page(id, url, Some(title), 200, true);
        p.semantic_issues_json = issues_json.map(|s| s.to_string());
        p.keywords_json = keywords_json.map(|s| s.to_string());
        p.seo_audit_json = audit_json;
        p.seo_score = seo_score;
        p.html_body = Some("<html><body>hello</body></html>".to_string());
        p
    }

    #[test]
    fn test_save_results_batch_matches_single_save_content() {
        // Same writes through two paths must leave identical contents in
        // every table: repo A saves page-by-page with `save_result` (the
        // pre-1.4 reference behavior), repo B with `save_results_batch` +
        // `save_links_batch` (the production flush path). Then both are
        // re-crawled with changed data, which must replace — not duplicate —
        // rows in both.
        let issues_v1 = r#"[
            {"issue_type":"missing_title","severity":"error","element":"head","message":"Missing title","css_selector":"head > title","xpath":"/html/head/title"},
            {"issue_type":"img_no_alt","severity":"warning","element":"img","message":"Image without alt"}
        ]"#;
        let issues_v2 = r#"[{"issue_type":"img_no_alt","severity":"warning","element":"img","message":"Image without alt"}]"#;
        let keywords_v1 = r#"[{"keyword":"seo","count":3},{"keyword":"crawler","count":1}]"#;
        let keywords_v2 = r#"[{"keyword":"seo","count":5}]"#;

        let batch1 = vec![
            rich_page(
                "a1",
                "https://x.com/a",
                "A v1",
                Some(issues_v1),
                Some(keywords_v1),
                Some(audit_json_v1()),
                Some(72.0),
            ),
            rich_page(
                "b1",
                "https://x.com/b",
                "B v1",
                Some(issues_v1),
                Some(keywords_v1),
                Some(audit_json_v1()),
                Some(72.0),
            ),
            // Unparseable payloads: both paths must store the page and
            // contribute no normalized rows.
            rich_page(
                "c1",
                "https://x.com/c",
                "C v1",
                Some("[broken"),
                Some("{not json"),
                Some("{not json".to_string()),
                None,
            ),
            // Duplicate URL inside the batch: the last row wins in both
            // paths. No links here: the single-save path deletes the URL's
            // links on the second save while the batch path deletes them
            // once, so link comparison for this URL would be path noise.
            rich_page(
                "a2",
                "https://x.com/a",
                "A v1 dup",
                Some(issues_v2),
                Some(keywords_v2),
                Some(audit_json_v2()),
                Some(88.0),
            ),
            // Same URL in another project is a different row.
            {
                let mut p = rich_page("a-p2", "https://x.com/a", "A p2", None, None, None, None);
                p.project_id = "p2".to_string();
                p
            },
        ];
        // Links for the non-duplicated URLs only (see above).
        let mut links1: Vec<PageLink> = vec![
            link("https://x.com/b", "https://x.com/a"),
            link("https://x.com/b", "https://x.com/c"),
        ];
        links1[0].anchor_text = Some("to A".to_string());
        // Attach the links to the /b page so `save_result` persists them.
        let mut batch1 = batch1;
        for p in batch1.iter_mut() {
            if p.id == "b1" {
                p.links = links1.clone();
            }
        }

        let batch2 = vec![
            rich_page(
                "a3",
                "https://x.com/a",
                "A v2",
                Some(issues_v2),
                Some(keywords_v2),
                Some(audit_json_v2()),
                Some(88.0),
            ),
            // Re-crawl with no issues/keywords/audit: stale rows must vanish.
            rich_page("b2", "https://x.com/b", "B v2", None, None, None, None),
            rich_page(
                "d1",
                "https://x.com/d",
                "D new",
                Some(issues_v1),
                Some(keywords_v1),
                Some(audit_json_v1()),
                Some(72.0),
            ),
        ];
        let links2 = vec![link("https://x.com/b", "https://x.com/d")];
        let mut batch2 = batch2;
        for p in batch2.iter_mut() {
            if p.id == "b2" {
                p.links = links2.clone();
            }
        }

        // Repo A: one `save_result` per page, in input order.
        let repo_a = test_repo();
        // Repo B: batched writes, links via `save_links_batch` as the
        // production flush does (results first, then links).
        let repo_b = test_repo();
        // The p2 page references the p2 project (FK enforced on these
        // connections), so seed it in both databases.
        for repo in [&repo_a, &repo_b] {
            repo.conn
                .execute_batch(
                    "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('p2', 'P2', datetime('now'), datetime('now'));",
                )
                .unwrap();
        }
        for p in batch1.iter().chain(batch2.iter()) {
            repo_a.save_result(p).unwrap();
        }

        repo_b.save_results_batch(&batch1).unwrap();
        repo_b.save_links_batch(&links1).unwrap();
        repo_b.save_results_batch(&batch2).unwrap();
        repo_b.save_links_batch(&links2).unwrap();

        let snap_a = snapshot_all(&repo_a);
        let snap_b = snapshot_all(&repo_b);
        for ((table_a, rows_a), (table_b, rows_b)) in snap_a.iter().zip(snap_b.iter()) {
            assert_eq!(table_a, table_b);
            assert_eq!(
                rows_a, rows_b,
                "table {table_a} differs between save_result and save_results_batch"
            );
        }

        // The re-crawl really replaced instead of duplicating.
        let count: i64 = repo_b
            .conn
            .query_row(
                "SELECT COUNT(*) FROM crawled_pages WHERE project_id = 'p1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 4, "p1 must hold /a, /b, /c, /d exactly once each");
    }

    #[test]
    fn test_save_results_batch_chunks_large_batches() {
        // 65 pages forces several multi-row chunks (30 pages per statement
        // for crawled_pages, 100 issue rows per statement, 200 keyword /
        // category rows per statement) — every row must still land.
        let repo = test_repo();
        let issues = r#"[
            {"issue_type":"missing_title","severity":"error","element":"head","message":"Missing title"},
            {"issue_type":"img_no_alt","severity":"warning","element":"img","message":"Image without alt"},
            {"issue_type":"empty_link_text","severity":"warning","element":"a","message":"Empty link"}
        ]"#;
        let keywords = r#"[{"keyword":"seo","count":2},{"keyword":"rust","count":1}]"#;
        let pages: Vec<CrawlResult> = (0..65)
            .map(|i| {
                rich_page(
                    &format!("pg{i:03}"),
                    &format!("https://x.com/p{i:03}"),
                    &format!("Page {i}"),
                    Some(issues),
                    Some(keywords),
                    Some(audit_json_v1()),
                    Some(72.0),
                )
            })
            .collect();
        repo.save_results_batch(&pages).unwrap();

        let counts: Vec<(String, i64)> = [
            ("crawled_pages", "SELECT COUNT(*) FROM crawled_pages"),
            ("page_issues", "SELECT COUNT(*) FROM page_issues"),
            ("page_keywords", "SELECT COUNT(*) FROM page_keywords"),
            (
                "seo_category_scores",
                "SELECT COUNT(*) FROM seo_category_scores",
            ),
            ("seo_check_issues", "SELECT COUNT(*) FROM seo_check_issues"),
        ]
        .iter()
        .map(|(name, sql)| {
            (
                name.to_string(),
                repo.conn
                    .query_row(sql, [], |row| row.get::<_, i64>(0))
                    .unwrap(),
            )
        })
        .collect();
        assert_eq!(
            counts,
            vec![
                ("crawled_pages".to_string(), 65),
                ("page_issues".to_string(), 65 * 3),
                ("page_keywords".to_string(), 65 * 2),
                ("seo_category_scores".to_string(), 65 * 3),
                ("seo_check_issues".to_string(), 65 * 2),
            ]
        );
        // Fix counts land via the CASE update (2 priority fixes per audit).
        let fix_sum: i64 = repo
            .conn
            .query_row(
                "SELECT COALESCE(SUM(seo_priority_fix_count), 0) FROM crawled_pages",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(fix_sum, 65 * 2);
    }

    #[test]
    fn test_save_links_batch_chunks_and_preserves_rel_tokens() {
        // 250 links forces several multi-row chunks (80 rows per statement
        // at 11 columns); rel tokens and flags must round-trip exactly.
        let repo = test_repo();
        let links: Vec<PageLink> = (0..250)
            .map(|i| PageLink {
                from_url: "https://x.com/a".to_string(),
                to_url: format!("https://x.com/t{i:03}"),
                config_id: "cfg".to_string(),
                project_id: "p1".to_string(),
                link_type: "href".to_string(),
                anchor_text: if i % 2 == 0 {
                    Some(format!("anchor {i}"))
                } else {
                    None
                },
                is_follow: i % 3 != 0,
                rel_tokens: match i % 3 {
                    0 => vec!["nofollow".to_string()],
                    1 => vec!["sponsored".to_string(), "ugc".to_string()],
                    _ => Vec::new(),
                },
                is_sponsored: i % 3 == 1,
                is_ugc: i % 3 == 1,
                is_internal: i % 2 == 0,
            })
            .collect();
        repo.save_links_batch(&links).unwrap();

        let count: i64 = repo
            .conn
            .query_row("SELECT COUNT(*) FROM page_links", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 250);

        let (rel, sponsored, ugc, internal): (
            Option<String>,
            i64,
            i64,
            i64,
        ) = repo
            .conn
            .query_row(
                "SELECT rel_tokens, is_sponsored, is_ugc, is_internal FROM page_links WHERE to_url = 'https://x.com/t001'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            crate::db::repos::crawl::deserialize_rel_tokens(rel.as_deref()),
            vec!["sponsored".to_string(), "ugc".to_string()]
        );
        assert_eq!((sponsored, ugc, internal), (1, 1, 0));
    }

    #[test]
    fn test_seo_overview_aggregates_normalized_rows() {
        let repo = test_repo();
        let audit = serde_json::json!({
            "score": 72.0,
            "grade": "C",
            "categories": [
                {"category": "meta", "score": 50.0, "weight": 0.25, "passed_weight": 10.0, "total_weight": 20.0, "passed_checks": 1, "total_checks": 2},
                {"category": "technical", "score": 90.0, "weight": 0.20, "passed_weight": 18.0, "total_weight": 20.0, "passed_checks": 2, "total_checks": 2}
            ],
            "checks": [
                {"id": "title_present", "category": "meta", "severity": "error", "passed": false,
                 "weight": 1.0, "message": "Missing title", "guidance": "Add a title",
                 "examples": [{"issue_type": "missing_title", "severity": "error", "element": "head", "message": "x"}]},
                {"id": "meta_desc_present", "category": "meta", "severity": "warning", "passed": true,
                 "weight": 1.0, "message": "ok", "guidance": "ok"}
            ],
            "priority_fixes": [{"id": "title_present", "priority": "critical", "message": "m", "guidance": "g", "category": "meta"}]
        });
        let mut p = page("pg1", "https://x.com/a", Some("A"), 200, true);
        p.seo_score = Some(72.0);
        p.seo_audit_json = Some(audit.to_string());
        repo.save_results_batch(&[p]).unwrap();

        let overview = repo.get_seo_overview("p1").unwrap();
        assert_eq!(overview.audited_pages, 1);
        assert_eq!(overview.total_pages, 1);
        assert!((overview.avg_score.unwrap() - 72.0).abs() < 1e-9);
        assert_eq!(overview.avg_grade.as_deref(), Some("C"));
        assert_eq!(overview.total_fixes, 1);
        assert_eq!(overview.top_issues.len(), 1);
        assert_eq!(overview.top_issues[0].id, "title_present");
        assert_eq!(overview.top_issues[0].occurrences, 1);
        assert_eq!(overview.category_averages.len(), 2);
    }

    #[test]
    fn test_seo_overview_recrawl_clears_stale_normalized() {
        let repo = test_repo();
        let audit = serde_json::json!({
            "score": 100.0,
            "grade": "A",
            "categories": [{"category": "meta", "score": 100.0, "weight": 0.25, "passed_weight": 20.0, "total_weight": 20.0, "passed_checks": 2, "total_checks": 2}],
            "checks": [{"id": "title_present", "category": "meta", "severity": "error", "passed": false,
                        "weight": 1.0, "message": "Missing title", "guidance": "Add a title"}],
            "priority_fixes": []
        });
        let mut p = page("oldid", "https://x.com/a", Some("A"), 200, true);
        p.seo_score = Some(100.0);
        p.seo_audit_json = Some(audit.to_string());
        repo.save_results_batch(&[p]).unwrap();

        // Re-crawl same URL with no audit: stale normalized rows must be gone.
        repo.save_results_batch(&[page("newid", "https://x.com/a", Some("A"), 200, true)])
            .unwrap();

        let category_rows: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM seo_category_scores WHERE project_id = 'p1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            category_rows, 0,
            "stale category rows must be removed on re-crawl"
        );

        let check_rows: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM seo_check_issues WHERE project_id = 'p1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            check_rows, 0,
            "stale check rows must be removed on re-crawl"
        );

        let overview = repo.get_seo_overview("p1").unwrap();
        assert_eq!(overview.audited_pages, 0);
        assert_eq!(overview.total_pages, 1);
    }

    #[test]
    fn test_seo_overview_backfills_legacy_json() {
        let repo = test_repo();
        let audit = serde_json::json!({
            "score": 85.0,
            "grade": "B",
            "categories": [{"category": "meta", "score": 80.0, "weight": 0.25, "passed_weight": 16.0, "total_weight": 20.0, "passed_checks": 2, "total_checks": 2}],
            "checks": [{"id": "h1_present", "category": "meta", "severity": "error", "passed": false,
                        "weight": 1.0, "message": "Missing h1", "guidance": "Add h1"}],
            "priority_fixes": [{"id": "h1_present", "priority": "critical", "message": "m", "guidance": "g", "category": "meta"}]
        });
        // Simulate a page written by an older engine that ran before migration
        // 011 existed: the json is present but no normalized rows exist yet.
        repo.conn
            .execute(
                "INSERT INTO crawled_pages (id, config_id, project_id, url, status_code, title, is_indexable, depth, crawl_timestamp, seo_score, seo_audit_json, blocked)
                 VALUES ('pg1', 'cfg', 'p1', 'https://x.com/a', 200, 'A', 1, 0, datetime('now'), 85.0, ?1, 0)",
                rusqlite::params![audit.to_string()],
            )
            .unwrap();

        // The overview must show no issues/categories yet because the
        // normalized rows don't exist (they are only created by migration
        // 011's backfill, which is applied to on-disk DBs on startup).
        let overview = repo.get_seo_overview("p1").unwrap();
        assert_eq!(overview.audited_pages, 1);
        assert!(overview.top_issues.is_empty());
        assert!(overview.category_averages.is_empty());
        assert_eq!(overview.total_fixes, 0);

        // Emulate migration 011's backfill for this page.
        repo.conn
            .execute_batch(
                "INSERT INTO seo_category_scores (page_id, project_id, category, score)
                 SELECT p.id, p.project_id, json_extract(c.value, '$.category'), json_extract(c.value, '$.score')
                 FROM crawled_pages p, json_each(p.seo_audit_json, '$.categories') c
                 WHERE p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json)
                   AND json_extract(c.value, '$.score') IS NOT NULL;

                 INSERT INTO seo_check_issues
                     (page_id, project_id, category, severity, check_id, message, guidance, evidence, examples_json)
                 SELECT p.id, p.project_id,
                        json_extract(c.value, '$.category'), json_extract(c.value, '$.severity'),
                        json_extract(c.value, '$.id'), json_extract(c.value, '$.message'),
                        json_extract(c.value, '$.guidance'), json_extract(c.value, '$.evidence'),
                        json_extract(c.value, '$.examples')
                 FROM crawled_pages p, json_each(p.seo_audit_json, '$.checks') c
                 WHERE p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json)
                   AND json_extract(c.value, '$.passed') = 0;

                 UPDATE crawled_pages SET seo_priority_fix_count = (
                     SELECT COUNT(*) FROM json_each(crawled_pages.seo_audit_json, '$.priority_fixes')
                 )
                 WHERE seo_audit_json IS NOT NULL AND json_valid(seo_audit_json);",
            )
            .unwrap();

        let overview = repo.get_seo_overview("p1").unwrap();
        assert_eq!(overview.audited_pages, 1);
        assert!((overview.avg_score.unwrap() - 85.0).abs() < 1e-9);
        assert_eq!(overview.total_fixes, 1);
        assert_eq!(overview.top_issues.len(), 1);
        assert_eq!(overview.top_issues[0].id, "h1_present");
    }

    /// Fase 1.3: `count_seo_rows` (normalized tables) must match the old
    /// `json_each` count exactly, including the tricky cases: a skipped
    /// category (score null, e.g. site-level "links" on a page audit) still
    /// produces an "SEO Audit" sheet row, an empty audit and a page without
    /// an audit contribute nothing, and invalid JSON contributes nothing.
    #[test]
    fn test_count_seo_rows_matches_legacy_json_count() {
        let repo = test_repo();
        // Page 1: 2 scored categories + 1 skipped, 1 passed + 2 failing
        // checks, 2 priority fixes -> 3 + 2 + 2 = 7 rows.
        let audit = serde_json::json!({
            "score": 72.0,
            "grade": "C",
            "categories": [
                {"category": "meta", "score": 80.0, "weight": 0.25, "passed_weight": 0.2, "total_weight": 0.25, "passed_checks": 3, "total_checks": 4},
                {"category": "links", "score": null, "weight": 0.02, "passed_weight": 0.0, "total_weight": 0.0, "passed_checks": 0, "total_checks": 0},
                {"category": "sxo", "score": 50.0, "weight": 0.08, "passed_weight": 0.04, "total_weight": 0.08, "passed_checks": 1, "total_checks": 3}
            ],
            "checks": [
                {"id": "title_length", "category": "meta", "severity": "warning", "passed": false, "weight": 2.0, "message": "Title too short", "guidance": "Extend"},
                {"id": "https_used", "category": "technical", "severity": "error", "passed": true, "weight": 3.0, "message": "HTTPS ok", "guidance": "-"},
                {"id": "h1_present", "category": "meta", "severity": "error", "passed": false, "weight": 1.0, "message": "Missing h1", "guidance": "Add h1"}
            ],
            "priority_fixes": [
                {"id": "title_length", "priority": "important", "message": "Title too short", "guidance": "Extend", "category": "meta"},
                {"id": "h1_present", "priority": "critical", "message": "Missing h1", "guidance": "Add h1", "category": "meta"}
            ]
        });
        let mut p1 = page("pg1", "https://x.com/a", Some("A"), 200, true);
        p1.seo_score = Some(72.0);
        p1.seo_audit_json = Some(audit.to_string());
        // Page 2: empty audit -> 0 rows.
        let mut p2 = page("pg2", "https://x.com/b", Some("B"), 200, true);
        p2.seo_score = Some(0.0);
        p2.seo_audit_json = Some(
            r#"{"score": 0.0, "grade": "F", "categories": [], "checks": [], "priority_fixes": []}"#
                .to_string(),
        );
        // Page 3: no audit at all -> 0 rows.
        let p3 = page("pg3", "https://x.com/c", Some("C"), 200, true);
        // Page 4: invalid JSON -> 0 rows.
        let mut p4 = page("pg4", "https://x.com/d", Some("D"), 200, true);
        p4.seo_audit_json = Some("{not json".to_string());
        repo.save_results_batch(&[p1, p2, p3, p4]).unwrap();

        // The pre-1.3 query, run verbatim against the same data.
        let legacy: i64 = repo
            .conn
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM crawled_pages p, json_each(p.seo_audit_json, '$.categories') c
                     WHERE p.project_id = ?1 AND p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json))
                  + (SELECT COUNT(*) FROM crawled_pages p, json_each(p.seo_audit_json, '$.checks') c
                     WHERE p.project_id = ?1 AND p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json)
                       AND json_extract(c.value, '$.passed') = 0)
                  + (SELECT COUNT(*) FROM crawled_pages p, json_each(p.seo_audit_json, '$.priority_fixes') c
                     WHERE p.project_id = ?1 AND p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json))",
                rusqlite::params!["p1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            legacy, 7,
            "legacy count: 3 categories + 2 failing checks + 2 fixes"
        );

        let new = repo.count_seo_rows("p1").unwrap();
        assert_eq!(
            new, legacy as u32,
            "normalized count must match the legacy json_each count"
        );

        // The skipped category must be persisted (NULL score) so the count
        // above can see it, but the overview must still ignore it.
        let skipped: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM seo_category_scores WHERE project_id = 'p1' AND score IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            skipped, 1,
            "skipped 'links' category must be stored with NULL score"
        );

        let overview = repo.get_seo_overview("p1").unwrap();
        assert_eq!(overview.total_fixes, 2);
        assert!(
            !overview
                .category_averages
                .iter()
                .any(|c| c.category == "links"),
            "skipped categories must not appear in the overview averages"
        );
    }

    #[test]
    fn test_recrawl_replaces_row_per_url() {
        let repo = test_repo();
        let p1 = page("id1", "https://x.com/a", Some("A v1"), 200, true);
        repo.save_results_batch(&[p1]).unwrap();

        // Re-crawl of the same URL with a new id must replace, not duplicate
        let mut p2 = page("id2", "https://x.com/a", Some("A v2"), 200, true);
        p2.id = "id2".to_string();
        repo.save_results_batch(&[p2]).unwrap();

        let count: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM crawled_pages WHERE url = 'https://x.com/a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "re-crawl must not accumulate duplicate URL rows");

        let (results, _) = repo
            .get_results(
                "p1", 1, 100, None, None, None, None, None, None, false, false, false, false,
            )
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id2");
        assert_eq!(results[0].title.as_deref(), Some("A v2"));
    }

    #[test]
    fn test_delete_project_with_snapshots() {
        let repo = test_repo();
        repo.conn.execute_batch(
            "INSERT INTO crawl_snapshots (id, project_id, config_id, snapshot_time) VALUES ('s1', 'p1', 'cfg', datetime('now'));
             INSERT INTO crawl_snapshot_data (snapshot_id, page_id, url, title) VALUES ('s1', 'pg1', 'https://x.com/a', 'A');",
        )
        .unwrap();

        repo.delete_project("p1").unwrap();

        let remaining: i64 = repo
            .conn
            .query_row("SELECT COUNT(*) FROM crawl_snapshots", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
        let remaining: i64 = repo
            .conn
            .query_row("SELECT COUNT(*) FROM projects WHERE id = 'p1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_create_crawl_snapshot_prunes_older_ones() {
        let repo = test_repo();
        repo.save_results_batch(&[page("pg1", "https://x.com/a", Some("A"), 200, true)])
            .unwrap();

        // Generate 20 snapshots; only the newest 12 must survive.
        for _ in 0..20 {
            repo.create_crawl_snapshot("p1", "cfg").unwrap();
        }

        let remaining: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM crawl_snapshots WHERE project_id = 'p1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 12, "snapshots must be pruned to 12 per project");

        let orphaned_data: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM crawl_snapshot_data
                 WHERE snapshot_id NOT IN (SELECT id FROM crawl_snapshots)",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(orphaned_data, 0, "pruned snapshots must not leak data rows");
    }

    #[test]
    fn test_list_projection_drops_heavy_json_blobs_but_detail_keeps_them() {
        let repo = test_repo();
        let mut p = page("pg1", "https://x.com/a", Some("A"), 200, true);
        p.hreflang_json = Some(r#"[{"hreflang":"en","href":"https://x.com/a"}]"#.to_string());
        p.keywords_json = Some(r#"[{"keyword":"seo","count":2}]"#.to_string());
        p.og_json = Some(r#"{"title":"OG"}"#.to_string());
        p.pagespeed_json = Some(r#"{"score":80}"#.to_string());
        p.seo_audit_json = Some(r#"{"score":75,"categories":[]}"#.to_string());
        p.response_headers_json = Some(r#"{"content-type":"text/html"}"#.to_string());
        p.seo_score = Some(75.0);
        p.pagespeed_score = Some(80.0);
        repo.save_results_batch(&[p]).unwrap();

        // The list view only needs scalars + semantic issues: the heavy per-page
        // JSON blobs must not cross the IPC boundary (they are served by
        // get_page_detail / the SEO audit command on demand).
        let (items, _) = repo
            .get_results(
                "p1", 1, 100, None, None, None, None, None, None, false, false, false, false,
            )
            .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].seo_score,
            Some(75.0),
            "scalar score must be present"
        );
        assert_eq!(items[0].pagespeed_score, Some(80.0));
        assert_eq!(items[0].hreflang_json, None);
        assert_eq!(items[0].keywords_json, None);
        assert_eq!(items[0].og_json, None);
        assert_eq!(items[0].pagespeed_json, None);
        assert_eq!(items[0].seo_audit_json, None);
        assert_eq!(items[0].response_headers_json, None);

        // The detail path still returns the full blobs.
        let detail = repo.get_page_detail("pg1").unwrap();
        assert_eq!(
            detail.page.hreflang_json.as_deref(),
            Some(r#"[{"hreflang":"en","href":"https://x.com/a"}]"#)
        );
        assert_eq!(
            detail.page.keywords_json.as_deref(),
            Some(r#"[{"keyword":"seo","count":2}]"#)
        );
        assert_eq!(detail.page.og_json.as_deref(), Some(r#"{"title":"OG"}"#));
        assert_eq!(
            detail.page.pagespeed_json.as_deref(),
            Some(r#"{"score":80}"#)
        );
        assert_eq!(
            detail.page.seo_audit_json.as_deref(),
            Some(r#"{"score":75,"categories":[]}"#)
        );
        assert_eq!(
            detail.page.response_headers_json.as_deref(),
            Some(r#"{"content-type":"text/html"}"#)
        );
        assert_eq!(detail.page.seo_score, Some(75.0));
    }

    #[test]
    fn test_get_page_id_by_url_returns_most_recent() {
        let repo = test_repo();
        let mut old = page("pg-old", "https://x.com/a", Some("A"), 200, true);
        old.crawl_timestamp = "2026-01-01T00:00:00Z".to_string();
        let mut new = page("pg-new", "https://x.com/a", Some("A"), 200, true);
        new.crawl_timestamp = "2026-01-02T00:00:00Z".to_string();
        repo.save_results_batch(&[old, new]).unwrap();

        assert_eq!(
            repo.get_page_id_by_url("p1", "https://x.com/a").unwrap(),
            Some("pg-new".to_string()),
            "the most recent crawl of the URL wins"
        );
        assert_eq!(
            repo.get_page_id_by_url("p1", "https://x.com/missing")
                .unwrap(),
            None
        );
        assert_eq!(
            repo.get_page_id_by_url("other-project", "https://x.com/a")
                .unwrap(),
            None
        );
    }

    fn kw_json(keywords: &[&str]) -> String {
        let items: Vec<serde_json::Value> = keywords
            .iter()
            .map(|k| serde_json::json!({"keyword": k, "count": 3}))
            .collect();
        serde_json::to_string(&items).unwrap()
    }

    #[test]
    fn test_compute_seo_clusters_groups_similar_pages() {
        let repo = test_repo();
        let mut a = page("pg1", "https://x.com/a", Some("A"), 200, true);
        a.keywords_json = Some(kw_json(&["seo", "auditoria", "herramienta"]));
        let mut b = page("pg2", "https://x.com/b", Some("B"), 200, true);
        b.keywords_json = Some(kw_json(&["seo", "auditoria", "herramienta"]));
        // e shares 3 of 4 keywords with a (Jaccard 0.75) -> same cluster.
        let mut e = page("pg5", "https://x.com/e", Some("E"), 200, true);
        e.keywords_json = Some(kw_json(&["seo", "auditoria", "herramienta", "extra"]));
        // c shares 2 of 4 with a (Jaccard 0.5) -> stays alone.
        let mut c = page("pg3", "https://x.com/c", Some("C"), 200, true);
        c.keywords_json = Some(kw_json(&["seo", "auditoria", "otra"]));
        // d shares nothing -> stays alone.
        let mut d = page("pg4", "https://x.com/d", Some("D"), 200, true);
        d.keywords_json = Some(kw_json(&["cocina", "recetas"]));
        repo.save_results_batch(&[a, b, c, d, e]).unwrap();

        let count = repo.compute_seo_clusters("p1").unwrap();
        assert_eq!(count, 1);
        let members: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM seo_cluster_members WHERE cluster_id IN (SELECT id FROM seo_clusters WHERE project_id = 'p1')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(members, 3);
    }

    #[test]
    fn test_compute_seo_clusters_skips_generic_buckets() {
        // 500 pages sharing only the generic keyword "generico": the bucket
        // exceeds MAX_CLUSTER_BUCKET_PAGES (400) and is skipped (T13), so
        // no cluster forms and the call stays fast.
        let repo = test_repo();
        let mut pages = Vec::with_capacity(500);
        for i in 0..500 {
            let mut p = page(
                &format!("pg{i}"),
                &format!("https://x.com/p{i}"),
                Some("P"),
                200,
                true,
            );
            p.keywords_json = Some(kw_json(&["generico", &format!("unico{i}")]));
            pages.push(p);
        }
        repo.save_results_batch(&pages).unwrap();
        assert_eq!(repo.compute_seo_clusters("p1").unwrap(), 0);
    }

    fn audit_json(cats: usize, failed: usize, fixes: usize) -> String {
        let categories: Vec<serde_json::Value> = (0..cats)
            .map(|i| {
                serde_json::json!({
                    "category": format!("cat{i}"),
                    "score": 80.0,
                    "weight": 0.1,
                    "passed_weight": 0.08,
                    "total_weight": 0.1,
                    "passed_checks": 4,
                    "total_checks": 5,
                })
            })
            .collect();
        let mut checks: Vec<serde_json::Value> = vec![serde_json::json!({
            "id": "passed_check", "category": "cat0", "severity": "info",
            "passed": true, "weight": 1.0, "message": "ok", "guidance": "g",
        })];
        for i in 0..failed {
            checks.push(serde_json::json!({
                "id": format!("failed_check{i}"), "category": "cat0",
                "severity": "warning", "passed": false,
                "weight": 1.0, "message": "m", "guidance": "g",
            }));
        }
        let fixes: Vec<serde_json::Value> = (0..fixes)
            .map(|i| {
                serde_json::json!({
                    "id": format!("fix{i}"), "priority": "critical",
                    "message": "m", "guidance": "g", "category": "cat0",
                })
            })
            .collect();
        serde_json::json!({
            "score": 80.0, "grade": "B",
            "categories": categories, "checks": checks, "priority_fixes": fixes,
        })
        .to_string()
    }

    fn seo_counts(repo: &CrawlRepo, project: &str) -> (i64, i64, i64) {
        repo.conn
            .query_row(
                "SELECT category_count, check_issue_count, priority_fix_total
                 FROM project_seo_counts WHERE project_id = ?1",
                [project],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap()
    }

    #[test]
    fn test_project_seo_counts_maintained() {
        let repo = test_repo();
        let mut a = page("pg1", "https://x.com/a", Some("A"), 200, true);
        a.seo_audit_json = Some(audit_json(2, 2, 1));
        let mut b = page("pg2", "https://x.com/b", Some("B"), 200, true);
        b.seo_audit_json = Some(audit_json(2, 2, 1));
        repo.save_results_batch(&[a, b]).unwrap();
        // 2 pages x (2 cats + 2 failed + 1 fix) = 10.
        assert_eq!(seo_counts(&repo, "p1"), (4, 4, 2));
        assert_eq!(repo.count_seo_rows("p1").unwrap(), 10);

        // Re-crawl of /a with a smaller audit: old rows subtracted, new added.
        let mut a2 = page("pg1b", "https://x.com/a", Some("A2"), 200, true);
        a2.seo_audit_json = Some(audit_json(1, 0, 0));
        repo.save_results_batch(&[a2]).unwrap();
        assert_eq!(seo_counts(&repo, "p1"), (3, 2, 1));
        assert_eq!(repo.count_seo_rows("p1").unwrap(), 6);

        // Direct delete of the remaining page's normalized rows.
        crate::db::repos::seo::delete_seo_normalized(repo.conn, &["pg2".to_string()]).unwrap();
        assert_eq!(seo_counts(&repo, "p1"), (1, 0, 0));
        assert_eq!(repo.count_seo_rows("p1").unwrap(), 1);
    }

    #[test]
    fn test_count_seo_rows_falls_back_without_counter_row() {
        let repo = test_repo();
        let mut a = page("pg1", "https://x.com/a", Some("A"), 200, true);
        a.seo_audit_json = Some(audit_json(2, 2, 1));
        repo.save_results_batch(&[a]).unwrap();
        assert_eq!(repo.count_seo_rows("p1").unwrap(), 5);
        repo.conn
            .execute("DELETE FROM project_seo_counts WHERE project_id = 'p1'", [])
            .unwrap();
        // Same total from the three aggregate queries.
        assert_eq!(repo.count_seo_rows("p1").unwrap(), 5);
    }

    #[test]
    fn test_delete_project_removes_seo_counters_and_normalized_rows() {
        let repo = test_repo();
        let mut a = page("pg1", "https://x.com/a", Some("A"), 200, true);
        a.seo_audit_json = Some(audit_json(2, 2, 1));
        repo.save_results_batch(&[a]).unwrap();
        assert_eq!(repo.count_seo_rows("p1").unwrap(), 5);
        repo.delete_project("p1").unwrap();
        let n: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM project_seo_counts WHERE project_id = 'p1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(n, 0);
        let n: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM seo_category_scores WHERE project_id = 'p1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(n, 0);
        let n: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM seo_check_issues WHERE project_id = 'p1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_compare_crawl_snapshots_page_sections() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("pg1", "https://x.com/a", Some("A"), 200, true),
            page("pg2", "https://x.com/b", Some("B"), 200, true),
            page("pg3", "https://x.com/c", Some("C"), 404, false),
        ])
        .unwrap();
        let snap_a = repo.create_crawl_snapshot("p1", "cfg").unwrap();

        // Second crawl: /a title changes, /b disappears, /d appears.
        repo.save_results_batch(&[
            page("pg1", "https://x.com/a", Some("A2"), 200, true),
            page("pg3", "https://x.com/c", Some("C"), 404, false),
            page("pg4", "https://x.com/d", Some("D"), 200, true),
        ])
        .unwrap();
        repo.conn
            .execute(
                "DELETE FROM crawled_pages WHERE project_id = 'p1' AND url = 'https://x.com/b'",
                [],
            )
            .unwrap();
        let snap_b = repo.create_crawl_snapshot("p1", "cfg").unwrap();

        // new: only /d
        let new = repo
            .compare_crawl_snapshots_page(&snap_a.id, &snap_b.id, "new", 1, 10)
            .unwrap();
        assert_eq!(new.total, 1);
        assert_eq!(new.new_urls, vec!["https://x.com/d"]);
        assert!(new.removed_urls.is_empty());
        assert!(new.changed_urls.is_empty());

        // removed: only /b
        let removed = repo
            .compare_crawl_snapshots_page(&snap_a.id, &snap_b.id, "removed", 1, 10)
            .unwrap();
        assert_eq!(removed.total, 1);
        assert_eq!(removed.removed_urls, vec!["https://x.com/b"]);
        assert!(removed.new_urls.is_empty());
        assert!(removed.changed_urls.is_empty());

        // changed: only /a (title changed), unchanged: /c
        let changed = repo
            .compare_crawl_snapshots_page(&snap_a.id, &snap_b.id, "changed", 1, 10)
            .unwrap();
        assert_eq!(changed.total, 1);
        assert_eq!(changed.changed_urls.len(), 1);
        assert_eq!(changed.changed_urls[0].url, "https://x.com/a");
        assert!(changed.changed_urls[0]
            .diffs
            .iter()
            .any(|d| d.field == "title"
                && d.before.as_deref() == Some("A")
                && d.after.as_deref() == Some("A2")));
        assert_eq!(changed.unchanged_count, 1);

        // pagination: page size 1 must page over the section
        let p1 = repo
            .compare_crawl_snapshots_page(&snap_a.id, &snap_b.id, "new", 1, 1)
            .unwrap();
        let p2 = repo
            .compare_crawl_snapshots_page(&snap_a.id, &snap_b.id, "new", 2, 1)
            .unwrap();
        assert_eq!(p1.total, 1);
        assert_eq!(p1.new_urls.len(), 1);
        assert_eq!(p2.new_urls.len(), 0);
    }

    #[test]
    fn test_snapshot_retention_prunes_on_create() {
        let repo = test_repo();
        repo.save_results_batch(&[page("pg1", "https://x.com/a", Some("A"), 200, true)])
            .unwrap();
        assert_eq!(repo.set_snapshot_retention("p1", 2).unwrap(), 2);
        assert_eq!(repo.get_snapshot_retention("p1").unwrap(), 2);

        let s1 = repo.create_crawl_snapshot("p1", "cfg").unwrap();
        let s2 = repo.create_crawl_snapshot("p1", "cfg").unwrap();
        let _s3 = repo.create_crawl_snapshot("p1", "cfg").unwrap();

        let kept = repo.list_crawl_snapshots("p1").unwrap();
        assert_eq!(kept.len(), 2);
        let ids: Vec<&str> = kept.iter().map(|s| s.id.as_str()).collect();
        assert!(!ids.contains(&s1.id.as_str()), "oldest snapshot pruned");
        assert!(ids.contains(&s2.id.as_str()));
    }

    #[test]
    fn test_set_snapshot_retention_prunes_immediately() {
        let repo = test_repo();
        repo.save_results_batch(&[page("pg1", "https://x.com/a", Some("A"), 200, true)])
            .unwrap();
        repo.create_crawl_snapshot("p1", "cfg").unwrap();
        repo.create_crawl_snapshot("p1", "cfg").unwrap();
        repo.create_crawl_snapshot("p1", "cfg").unwrap();
        assert_eq!(repo.list_crawl_snapshots("p1").unwrap().len(), 3);

        assert_eq!(repo.set_snapshot_retention("p1", 1).unwrap(), 1);
        assert_eq!(repo.list_crawl_snapshots("p1").unwrap().len(), 1);
        // Clamped to 1..=100.
        assert_eq!(repo.set_snapshot_retention("p1", 0).unwrap(), 1);
        assert_eq!(repo.set_snapshot_retention("p1", 500).unwrap(), 100);
        // Unknown project keeps the default.
        assert_eq!(repo.get_snapshot_retention("nope").unwrap(), 12);
    }

    #[test]
    fn test_delete_project_removes_jobs_extractions_and_alerts() {
        let repo = test_repo();
        repo.conn
            .execute_batch(
                "INSERT INTO scheduled_jobs (id, project_id, cron_expression, config_json, created_at) VALUES ('job1', 'p1', '0 0 * * *', '{}', datetime('now'));
                 INSERT INTO custom_extractions (id, project_id, name, selector_type, selector, mode) VALUES ('ex1', 'p1', 'price', 'css', '.price', 'text');
                 INSERT INTO project_alert_settings (project_id) VALUES ('p1');",
            )
            .unwrap();
        repo.delete_project("p1").unwrap();

        let count = |table: &str| -> i64 {
            repo.conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM {table} WHERE project_id = 'p1'"),
                    [],
                    |r| r.get(0),
                )
                .unwrap()
        };
        assert_eq!(count("scheduled_jobs"), 0);
        assert_eq!(count("custom_extractions"), 0);
        assert_eq!(count("project_alert_settings"), 0);
    }

    #[test]
    fn test_cleanup_orphans() {
        let repo = test_repo();
        // Fabricate orphans with FK enforcement off, the way legacy
        // writes, transfers and manual DB edits create them (the test
        // connection enforces FKs, so a plain INSERT would fail).
        repo.conn
            .execute_batch("PRAGMA foreign_keys = OFF;")
            .unwrap();
        repo.conn
            .execute_batch(
                "INSERT INTO crawl_snapshot_data (snapshot_id, page_id, url) VALUES ('ghost-snap', 'pg1', 'https://x.com/a');
                 INSERT INTO scheduled_jobs (id, project_id, cron_expression, config_json, created_at) VALUES ('job-ghost', 'ghost-project', '0 0 * * *', '{}', datetime('now'));",
            )
            .unwrap();
        repo.conn
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();

        let report = repo.cleanup_orphans().unwrap();
        assert_eq!(report.get("crawl_snapshot_data"), Some(&1));
        assert_eq!(report.get("scheduled_jobs"), Some(&1));

        let remaining: i64 = repo
            .conn
            .query_row(
                "SELECT COUNT(*) FROM crawl_snapshot_data WHERE snapshot_id = 'ghost-snap'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 0);

        // Second sweep: nothing left to clean.
        let report2 = repo.cleanup_orphans().unwrap();
        assert!(report2.values().all(|&v| v == 0));
    }

    #[test]
    fn test_page_keywords_materialized_and_aggregated() {
        let repo = test_repo();
        let mut a = page("a", "https://x.com/a", Some("A"), 200, true);
        a.keywords_json =
            Some(r#"[{"keyword":"seo","count":3},{"keyword":"crawl","count":2}]"#.to_string());
        let mut b = page("b", "https://x.com/b", Some("B"), 200, true);
        b.keywords_json = Some(r#"[{"keyword":"seo","count":1}]"#.to_string());
        repo.save_results_batch(&[a, b]).unwrap();

        let rows: i64 = repo
            .conn
            .query_row("SELECT COUNT(*) FROM page_keywords", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            rows, 3,
            "keywords_json must be materialized row per keyword"
        );

        let keywords = repo.get_keywords("p1", 10).unwrap();
        assert_eq!(keywords.len(), 2);
        assert_eq!(keywords[0].keyword, "seo");
        assert_eq!(keywords[0].count, 4, "counts aggregate across pages");
        assert_eq!(keywords[0].pages, 2, "distinct pages per keyword");
        assert_eq!(keywords[1].keyword, "crawl");

        // Re-crawl replacing the page must not duplicate materialized rows.
        let mut a2 = page("a", "https://x.com/a", Some("A2"), 200, true);
        a2.keywords_json = Some(r#"[{"keyword":"seo","count":5}]"#.to_string());
        repo.save_results_batch(&[a2]).unwrap();
        let rows: i64 = repo
            .conn
            .query_row("SELECT COUNT(*) FROM page_keywords", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 2, "re-crawl replaces page keywords, no duplicates");
    }

    #[test]
    fn test_keywords_page_pagination() {
        let repo = test_repo();
        let mut pages = Vec::new();
        for i in 0..5 {
            let mut p = page(
                &format!("k{i}"),
                &format!("https://x.com/k{i}"),
                Some(&format!("K{i}")),
                200,
                true,
            );
            p.keywords_json = Some(format!(
                r#"[{{"keyword":"k0","count":1}},{{"keyword":"k{i}","count":1}}]"#
            ));
            pages.push(p);
        }
        repo.save_results_batch(&pages).unwrap();

        let (page1, total) = repo.get_keywords_page("p1", 1, 2).unwrap();
        assert_eq!(
            total, 5,
            "k0 shared by all 5 pages plus one unique per page"
        );
        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].keyword, "k0");
        assert_eq!(page1[0].count, 5);
        assert_eq!(page1[0].pages, 5);

        let (page2, _) = repo.get_keywords_page("p1", 2, 2).unwrap();
        assert_eq!(page2.len(), 2);
        assert_ne!(page1[0].keyword, page2[0].keyword);

        let (page3, _) = repo.get_keywords_page("p1", 3, 2).unwrap();
        assert_eq!(page3.len(), 1);

        // The full report (get_keywords) matches the concatenation of pages.
        let all = repo.get_keywords("p1", 100).unwrap();
        assert_eq!(all.len(), 5);
        let mut collected: Vec<String> = page1
            .iter()
            .chain(page2.iter())
            .chain(page3.iter())
            .map(|k| k.keyword.clone())
            .collect();
        let mut all_keys: Vec<String> = all.iter().map(|k| k.keyword.clone()).collect();
        collected.sort();
        all_keys.sort();
        assert_eq!(collected, all_keys);
    }

    #[test]
    fn test_duplicate_title_filter_uses_existence() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("a", "https://x.com/a", Some("Shared"), 200, true),
            page("b", "https://x.com/b", Some("Shared"), 200, true),
            page("c", "https://x.com/c", Some("Unique"), 200, true),
        ])
        .unwrap();

        let (items, total) = repo
            .get_results(
                "p1", 1, 100, None, None, None, None, None, None, false, true, false, false,
            )
            .unwrap();
        assert_eq!(total, 2, "both pages sharing a title are flagged");
        let urls: Vec<&str> = items.iter().map(|p| p.url.as_str()).collect();
        assert!(urls.contains(&"https://x.com/a"));
        assert!(urls.contains(&"https://x.com/b"));
    }

    #[test]
    fn test_list_pages_for_seo_audit_returns_stored_html() {
        let repo = test_repo();
        let mut with_html = page("a", "https://x.com/a", Some("A"), 200, true);
        with_html.html_body =
            Some("<html><head><title>A</title></head><body>x</body></html>".to_string());
        with_html.size_bytes = Some(123);
        with_html.load_time_ms = Some(42);
        with_html.response_headers_json = Some("{\"x-frame-options\":\"DENY\"}".to_string());
        let without_html = page("b", "https://x.com/b", Some("B"), 404, true);
        repo.save_results_batch(&[with_html, without_html]).unwrap();

        let pages = repo.list_pages_for_seo_audit("p1").unwrap();
        assert_eq!(pages.len(), 2);
        let first = &pages[0];
        assert_eq!(first.url, "https://x.com/a");
        assert_eq!(first.status_code, 200);
        assert_eq!(first.size_bytes, 123);
        assert_eq!(first.load_time_ms, 42);
        assert!(first
            .html_body
            .as_deref()
            .unwrap()
            .contains("<title>A</title>"));
        assert_eq!(
            first.response_headers_json.as_deref(),
            Some("{\"x-frame-options\":\"DENY\"}")
        );
        // No stored HTML: the caller must re-fetch this page over HTTP.
        assert_eq!(pages[1].url, "https://x.com/b");
        assert_eq!(pages[1].status_code, 404);
        assert!(pages[1].html_body.is_none());
    }

    #[test]
    fn test_update_seo_audits_batch_writes_all_pages_in_one_call() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("a", "https://x.com/a", Some("A"), 200, true),
            page("b", "https://x.com/b", Some("B"), 200, true),
        ])
        .unwrap();

        // A real audit JSON, built with the same audit engine the re-audit
        // uses, so the normalized rows are exercised too.
        let parser = crate::crawler::parser::SeoParser::new();
        let url = url::Url::parse("https://x.com/a").unwrap();
        let html = "<html lang=\"en\"><head><title>A</title></head><body><main><h1>A</h1><p>Text.</p></main></body></html>";
        let (seo_data, _) = parser.parse(html, &url);
        let audit = crate::seo::audit::audit_page(
            &seo_data,
            html,
            &crate::seo::AuditContext {
                url: url.to_string(),
                status_code: 200,
                size_bytes: html.len(),
                load_time_ms: 10,
                pagespeed_score: None,
                response_headers: std::collections::HashMap::new(),
                site_resources: None,
            },
        );
        let json = serde_json::to_string(&audit).unwrap();

        repo.update_seo_audits_batch(
            "p1",
            &[
                ("a".to_string(), audit.score, Some(json.clone()), None),
                ("b".to_string(), 50.0, Some(json.clone()), None),
            ],
        )
        .unwrap();

        let stored_a = repo.get_seo_audit_json("a").unwrap().unwrap();
        assert_eq!(stored_a.score, Some(audit.score));
        assert_eq!(stored_a.json.as_deref(), Some(json.as_str()));
        let stored_b = repo.get_seo_audit_json("b").unwrap().unwrap();
        assert_eq!(stored_b.score, Some(50.0));
        // Normalized rows stay in sync with the batch write.
        let overview = repo.get_seo_overview("p1").unwrap();
        assert_eq!(overview.audited_pages, 2);

        // Empty batch is a no-op, not an error.
        repo.update_seo_audits_batch("p1", &[]).unwrap();
    }

    #[test]
    fn test_discard_session_drops_queue_and_resumable() {
        let repo = test_repo();
        let config = crate::models::CrawlConfig {
            seed_urls: vec!["https://example.com/".to_string()],
            ..Default::default()
        };
        let session_id = repo.create_session("p1", &config).unwrap();
        repo.save_queue_batch(
            &session_id,
            &[
                ("https://example.com/a".to_string(), 1),
                ("https://example.com/b".to_string(), 1),
            ],
        )
        .unwrap();
        repo.interrupt_session(&session_id, 3, 0, 42).unwrap();
        assert!(repo.get_interrupted_session("p1").unwrap().is_some());

        repo.discard_session(&session_id).unwrap();

        assert!(repo.get_interrupted_session("p1").unwrap().is_none());
        assert!(repo.load_queue(&session_id).unwrap().is_empty());
    }

    #[test]
    fn test_get_filter_options_covers_whole_project() {
        let repo = test_repo();
        let mut a = page("pg1", "https://x.com/a", Some("A"), 200, true);
        a.semantic_issues_json = Some(
            r#"[{"issue_type":"img_no_alt","severity":"error","element":"img","message":"m"}]"#
                .to_string(),
        );
        let mut b = page("pg2", "https://x.com/b", Some("B"), 404, true);
        b.semantic_issues_json = Some(
            r#"[{"issue_type":"missing_title","severity":"warning","element":"title","message":"m"}]"#
                .to_string(),
        );
        repo.save_results_batch(&[a, b]).unwrap();

        let opts = repo.get_filter_options("p1").unwrap();
        assert_eq!(opts.status_codes, vec![200, 404]);
        assert_eq!(
            opts.severities,
            vec!["error".to_string(), "warning".to_string()]
        );
        assert_eq!(
            opts.issue_types,
            vec!["img_no_alt".to_string(), "missing_title".to_string()]
        );
    }

    #[test]
    fn test_get_sitemap_pages_filters_and_dedupes() {
        let repo = test_repo();
        let mut a = page("pg1", "https://x.com/a", Some("A"), 200, true);
        a.seo_score = Some(90.0);
        // Same URL crawled again later: only the newest row counts.
        let mut a2 = page("pg2", "https://x.com/a", Some("A2"), 200, true);
        a2.crawl_timestamp = "2026-02-01T00:00:00Z".to_string();
        a2.seo_score = Some(85.0);
        let b = page("pg3", "https://x.com/b", Some("B"), 404, true);
        let c = page("pg4", "https://x.com/c", Some("C"), 200, false);
        let mut d = page("pg5", "https://x.com/d", Some("D"), 200, true);
        d.depth = 2;
        d.seo_score = Some(30.0);
        repo.save_results_batch(&[a, a2, b, c, d]).unwrap();

        let pages = repo.get_sitemap_pages("p1").unwrap();
        assert_eq!(pages.len(), 2, "404 and noindex pages must be excluded");
        assert_eq!(pages[0].url, "https://x.com/a");
        assert_eq!(pages[0].lastmod.as_deref(), Some("2026-02-01"));
        assert_eq!(pages[0].depth, 0);
        assert_eq!(pages[0].seo_score, Some(85.0));
        assert_eq!(pages[1].url, "https://x.com/d");
        assert_eq!(pages[1].depth, 2);
        assert_eq!(pages[1].seo_score, Some(30.0));
    }
}
