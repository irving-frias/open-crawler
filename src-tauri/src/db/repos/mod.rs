pub mod analytics;
pub mod crawl;
pub mod export;
pub mod pagespeed;
pub mod projects;
pub mod results;
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
    html.as_ref().map(|s| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, compress_gzip(s.as_bytes())))
}

pub(crate) fn decompress_html_body(encoded: &Option<String>) -> Option<String> {
    encoded.as_ref().and_then(|e| base64::Engine::decode(&base64::engine::general_purpose::STANDARD, e).ok()).and_then(|bytes| String::from_utf8(decompress_gzip(&bytes)).ok())
}

pub(crate) fn compress_png(png: &Option<Vec<u8>>) -> Option<Vec<u8>> {
    png.as_ref().map(|data| compress_gzip(data))
}

pub(crate) fn decompress_png(data: &Option<Vec<u8>>) -> Option<Vec<u8>> {
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

    pub(crate) fn invalidate_cache_for_project(&self, project_id: &str) {
        if let Some(ref cache_arc) = self.results_cache {
            let mut cache = cache_arc.lock().unwrap();
            let keys_to_remove: Vec<ResultsCacheKey> = cache.iter().filter(|(k, _)| k.project_id == project_id).map(|(k, _)| k.clone()).collect();
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
            .get_results("p1", 1, 100, None, None, None, None, None, None, false, false, false, false)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id2");
        assert_eq!(results[0].title.as_deref(), Some("A v2"));
    }

    #[test]
    fn test_delete_project_with_snapshots() {        let repo = test_repo();
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
            .query_row("SELECT COUNT(*) FROM projects WHERE id = 'p1'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
    }
}
