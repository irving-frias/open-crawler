pub mod analytics;
pub mod crawl;
pub mod export;
pub mod links;
pub mod pagespeed;
pub mod projects;
pub mod results;
pub mod schedule;
pub mod seo;
pub mod settings;
pub mod snapshots;

use rusqlite::Connection;

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
                 WHERE p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json);

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
}
