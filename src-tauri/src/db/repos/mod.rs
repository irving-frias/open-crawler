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
        assert_ne!(page1[0].id, page2[0].id, "pages must return distinct groups");

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
        assert_eq!(items[0].seo_score, Some(75.0), "scalar score must be present");
        assert_eq!(items[0].pagespeed_score, Some(80.0));
        assert_eq!(items[0].hreflang_json, None);
        assert_eq!(items[0].keywords_json, None);
        assert_eq!(items[0].og_json, None);
        assert_eq!(items[0].pagespeed_json, None);
        assert_eq!(items[0].seo_audit_json, None);
        assert_eq!(items[0].response_headers_json, None);

        // The detail path still returns the full blobs.
        let detail = repo.get_page_detail("pg1").unwrap();
        assert_eq!(detail.page.hreflang_json.as_deref(), Some(r#"[{"hreflang":"en","href":"https://x.com/a"}]"#));
        assert_eq!(detail.page.keywords_json.as_deref(), Some(r#"[{"keyword":"seo","count":2}]"#));
        assert_eq!(detail.page.og_json.as_deref(), Some(r#"{"title":"OG"}"#));
        assert_eq!(detail.page.pagespeed_json.as_deref(), Some(r#"{"score":80}"#));
        assert_eq!(detail.page.seo_audit_json.as_deref(), Some(r#"{"score":75,"categories":[]}"#));
        assert_eq!(detail.page.response_headers_json.as_deref(), Some(r#"{"content-type":"text/html"}"#));
        assert_eq!(detail.page.seo_score, Some(75.0));
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
        assert!(changed
            .changed_urls[0]
            .diffs
            .iter()
            .any(|d| d.field == "title" && d.before.as_deref() == Some("A") && d.after.as_deref() == Some("A2")));
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
    fn test_page_keywords_materialized_and_aggregated() {
        let repo = test_repo();
        let mut a = page("a", "https://x.com/a", Some("A"), 200, true);
        a.keywords_json = Some(
            r#"[{"keyword":"seo","count":3},{"keyword":"crawl","count":2}]"#.to_string(),
        );
        let mut b = page("b", "https://x.com/b", Some("B"), 200, true);
        b.keywords_json = Some(r#"[{"keyword":"seo","count":1}]"#.to_string());
        repo.save_results_batch(&[a, b]).unwrap();

        let rows: i64 = repo
            .conn
            .query_row("SELECT COUNT(*) FROM page_keywords", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 3, "keywords_json must be materialized row per keyword");

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
        assert_eq!(total, 5, "k0 shared by all 5 pages plus one unique per page");
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
}
