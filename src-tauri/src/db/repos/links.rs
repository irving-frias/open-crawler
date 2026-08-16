use rusqlite::params;

use crate::error::AppError;
use crate::models::{AnchorAgg, AnchorQuality, DomainAgg, LinkAnalysis};

use super::CrawlRepo;

/// Anchor phrases that do not describe the linked resource and add no
/// topical signal (lowercased, trimmed).
const GENERIC_ANCHORS: &[&str] = &[
    "here",
    "click here",
    "read more",
    "learn more",
    "more",
    "this",
    "that",
    "link",
    "continue reading",
    "see more",
    "find out more",
    "get started",
    "sign up",
    "sign in",
    "login",
    "register",
    "download",
    "next",
    "previous",
    "back",
    "home",
    "go",
];

impl<'a> CrawlRepo<'a> {
    fn link_count(&self, project_id: &str, extra: &str) -> Result<usize, AppError> {
        let n: i64 = self.conn.query_row(
            &format!("SELECT COUNT(*) FROM page_links WHERE project_id = ?1 {extra}"),
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(n as usize)
    }

    fn page_count(&self, project_id: &str, extra: &str) -> Result<usize, AppError> {
        let n: i64 = self.conn.query_row(
            &format!("SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 {extra}"),
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(n as usize)
    }

    /// Pages that have no incoming internal link (excluding self links). These
    /// are hard to discover for both crawlers and users unless they are seeds.
    /// Returns `(urls, total)` for `page` (1-based) with `page_size` rows,
    /// ordered by URL so OFFSET paging is stable across reloads.
    pub fn orphan_pages_page(
        &self,
        project_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<String>, usize), AppError> {
        let where_ = "SELECT p.url FROM crawled_pages p
             WHERE p.project_id = ?1
               AND NOT EXISTS (
                   SELECT 1 FROM page_links l
                   WHERE l.project_id = ?1
                     AND l.is_internal = 1
                     AND l.from_url != l.to_url
                     AND l.to_url = p.url
               )
             ORDER BY p.url";
        let count: i64 = self.conn.query_row(
            &format!("SELECT COUNT(*) FROM ({where_})"),
            params![project_id],
            |row| row.get(0),
        )?;
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let limit = page_size as i64;
        let mut stmt = self.conn.prepare(&format!("{where_} LIMIT ?2 OFFSET ?3"))?;
        let urls = stmt
            .query_map(params![project_id, limit, offset], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((urls, count as usize))
    }

    /// Pages that point to no internal page (dead ends in the crawl graph).
    /// Returns `(urls, total)` for `page` (1-based) with `page_size` rows.
    pub fn dead_end_pages_page(
        &self,
        project_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<String>, usize), AppError> {
        let where_ = "SELECT p.url FROM crawled_pages p
             WHERE p.project_id = ?1
               AND NOT EXISTS (
                   SELECT 1 FROM page_links l
                   WHERE l.project_id = ?1
                     AND l.is_internal = 1
                     AND l.from_url = p.url
               )
             ORDER BY p.url";
        let count: i64 = self.conn.query_row(
            &format!("SELECT COUNT(*) FROM ({where_})"),
            params![project_id],
            |row| row.get(0),
        )?;
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let limit = page_size as i64;
        let mut stmt = self.conn.prepare(&format!("{where_} LIMIT ?2 OFFSET ?3"))?;
        let urls = stmt
            .query_map(params![project_id, limit, offset], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((urls, count as usize))
    }

    /// Most-used anchor phrases, aggregated in SQL. Returns `(anchors, total)`
    /// for `page` (1-based) with `page_size` rows.
    pub fn top_anchors_page(
        &self,
        project_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<AnchorAgg>, usize), AppError> {
        let total: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM (
                 SELECT anchor_text FROM page_links
                 WHERE project_id = ?1 AND trim(coalesce(anchor_text, '')) != ''
                 GROUP BY anchor_text
             )",
            params![project_id],
            |row| row.get(0),
        )?;
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let limit = page_size as i64;
        let mut stmt = self.conn.prepare(
            "SELECT anchor_text, COUNT(*) AS c FROM page_links
             WHERE project_id = ?1 AND trim(coalesce(anchor_text, '')) != ''
             GROUP BY anchor_text ORDER BY c DESC LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt
            .query_map(params![project_id, limit, offset], |row| {
                Ok(AnchorAgg {
                    anchor: row.get(0)?,
                    count: row.get::<_, i64>(1)? as usize,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((rows, total as usize))
    }

    /// External links aggregated by host (scheme-less authority, port stripped,
    /// lowercased) with a single GROUP BY, so the top domains come back
    /// directly instead of materializing every external row in Rust. The host
    /// extraction mirrors the backfill in migration 014 (`is_internal`),
    /// keeping both views consistent. Returns `(domains, total)` for `page`
    /// (1-based) with `page_size` rows.
    fn external_domain_rows_sql() -> &'static str {
        "SELECT
            lower(CASE WHEN instr(auth, ':') = 0 THEN auth
                       ELSE substr(auth, 1, instr(auth, ':') - 1) END) AS host,
            COUNT(*) AS count,
            SUM(CASE WHEN is_follow = 0 THEN 1 ELSE 0 END) AS nofollow,
            SUM(is_sponsored) AS sponsored,
            SUM(is_ugc) AS ugc
         FROM (
             SELECT
                 CASE WHEN instr(to_url, '://') = 0 THEN to_url
                      ELSE substr(to_url, instr(to_url, '://') + 3,
                          CASE WHEN instr(substr(to_url, instr(to_url, '://') + 3), '/') = 0
                               THEN length(substr(to_url, instr(to_url, '://') + 3))
                               ELSE instr(substr(to_url, instr(to_url, '://') + 3), '/') - 1 END)
                      END AS auth,
                 is_follow, is_sponsored, is_ugc
             FROM page_links
             WHERE project_id = ?1 AND is_internal = 0
         )
         GROUP BY host
         ORDER BY count DESC"
    }

    pub fn external_domains_page(
        &self,
        project_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<DomainAgg>, usize), AppError> {
        let sql = Self::external_domain_rows_sql();
        let total: i64 = self.conn.query_row(
            &format!("SELECT COUNT(*) FROM ({sql})"),
            params![project_id],
            |row| row.get(0),
        )?;
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let limit = page_size as i64;
        let mut stmt = self.conn.prepare(&format!("{sql} LIMIT ?2 OFFSET ?3"))?;
        let rows = stmt
            .query_map(params![project_id, limit, offset], |row| {
                Ok(DomainAgg {
                    domain: row.get(0)?,
                    count: row.get::<_, i64>(1)? as usize,
                    nofollow: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as usize,
                    sponsored: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as usize,
                    ugc: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as usize,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((rows, total as usize))
    }

    fn anchor_quality(&self, project_id: &str) -> Result<AnchorQuality, AppError> {
        let generic_sql = GENERIC_ANCHORS
            .iter()
            .map(|a| format!("'{}'", a.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ");
        let norm = "trim(coalesce(lower(anchor_text), ''))";
        let row = self.conn.query_row(
            &format!(
                "SELECT
                    SUM(CASE WHEN trim(coalesce(anchor_text, '')) = '' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN {norm} IN ({generic_sql}) THEN 1 ELSE 0 END),
                    SUM(CASE WHEN anchor_text = to_url THEN 1 ELSE 0 END),
                    SUM(CASE WHEN trim(coalesce(anchor_text, '')) != ''
                                  AND anchor_text != to_url
                                  AND {norm} NOT IN ({generic_sql})
                             THEN 1 ELSE 0 END)
                 FROM page_links WHERE project_id = ?1"
            ),
            params![project_id],
            |row| {
                Ok(AnchorQuality {
                    empty: row.get::<_, Option<i64>>(0)?.unwrap_or(0) as usize,
                    generic: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as usize,
                    url_anchors: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as usize,
                    descriptive: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as usize,
                })
            },
        )?;
        Ok(row)
    }

    /// Computes the aggregate link metrics for a project (see
    /// [`LinkAnalysis`]). All lookups are scoped to `project_id`.
    pub fn get_link_analysis(&self, project_id: &str) -> Result<LinkAnalysis, AppError> {
        let (orphan_pages, orphan_count) = self.orphan_pages_page(project_id, 1, 10)?;
        let (dead_end_pages, dead_end_count) = self.dead_end_pages_page(project_id, 1, 10)?;
        let unique_internal_targets: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT to_url) FROM page_links WHERE project_id = ?1 AND is_internal = 1",
            params![project_id],
            |row| row.get(0),
        )?;

        Ok(LinkAnalysis {
            total_links: self.link_count(project_id, "")?,
            internal_links: self.link_count(project_id, "AND is_internal = 1")?,
            external_links: self.link_count(project_id, "AND is_internal = 0")?,
            self_links: self.link_count(project_id, "AND from_url = to_url")?,
            followed_links: self.link_count(project_id, "AND is_follow = 1")?,
            nofollow_links: self.link_count(project_id, "AND is_follow = 0")?,
            sponsored_links: self.link_count(project_id, "AND is_sponsored = 1")?,
            ugc_links: self.link_count(project_id, "AND is_ugc = 1")?,
            unique_internal_targets: unique_internal_targets as usize,
            internal_pages: self.page_count(project_id, "")?,
            orphan_count,
            orphan_pages,
            dead_end_count,
            dead_end_pages,
            top_anchors: self.top_anchors_page(project_id, 1, 20)?.0,
            anchor_quality: self.anchor_quality(project_id)?,
            external_domains: self.external_domains_page(project_id, 1, 50)?.0,
        })
    }

    /// Whether the project has any links stored at all (used by the UI to
    /// decide between "no crawl yet" and "site has no outbound links").
    pub fn project_has_links(&self, project_id: &str) -> Result<bool, AppError> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM page_links WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(n > 0)
    }

    /// Fetches a single page's stored links for a UI badge count.
    pub fn get_page_link_count(&self, url: &str) -> Result<usize, AppError> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM page_links WHERE from_url = ?1",
            params![url],
            |row| row.get(0),
        )?;
        Ok(n as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;
    use crate::models::{CrawlResult, PageLink};
    use rusqlite::Connection;

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

    fn page(url: &str) -> CrawlResult {
        CrawlResult {
            id: format!("id-{}", url),
            config_id: "cfg".to_string(),
            project_id: "p1".to_string(),
            url: url.to_string(),
            status_code: Some(200),
            blocked: false,
            title: None,
            meta_description: None,
            h1: None,
            canonical: None,
            size_bytes: None,
            load_time_ms: None,
            is_indexable: Some(true),
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

    fn link(from: &str, to: &str, anchor: Option<&str>, rel: &[&str]) -> PageLink {
        let rel_tokens: Vec<String> = rel.iter().map(|r| r.to_string()).collect();
        PageLink {
            from_url: from.to_string(),
            to_url: to.to_string(),
            config_id: "cfg".to_string(),
            project_id: "p1".to_string(),
            link_type: "a".to_string(),
            anchor_text: anchor.map(|a| a.to_string()),
            is_follow: !rel_tokens.iter().any(|t| t == "nofollow"),
            rel_tokens,
            is_sponsored: rel.iter().any(|t| *t == "sponsored"),
            is_ugc: rel.iter().any(|t| *t == "ugc"),
            is_internal: false,
        }
    }

    #[test]
    fn test_get_link_analysis_aggregates() {
        let repo = test_repo();
        repo.save_results_batch(&[
            page("https://x.com/a"),
            page("https://x.com/b"),
            page("https://x.com/c"),
        ])
        .unwrap();

        let mut links = vec![
            link("https://x.com/a", "https://x.com/b", Some("About us"), &[]),
            link(
                "https://x.com/a",
                "https://x.com/c",
                Some("here"),
                &["nofollow"],
            ),
            link("https://x.com/b", "https://x.com/b", Some("self"), &[]),
            link(
                "https://x.com/b",
                "https://ads.y.com/land",
                Some("Buy now"),
                &["sponsored"],
            ),
            link(
                "https://x.com/c",
                "https://forum.z.com/t",
                Some("User post"),
                &["ugc"],
            ),
        ];
        for l in &mut links {
            let from_host = l
                .from_url
                .split("//")
                .nth(1)
                .and_then(|h| h.split('/').next());
            let to_host = l
                .to_url
                .split("//")
                .nth(1)
                .and_then(|h| h.split('/').next());
            l.is_internal = from_host == to_host;
        }
        repo.save_links_batch(&links).unwrap();

        let analysis = repo.get_link_analysis("p1").unwrap();
        assert_eq!(analysis.total_links, 5);
        assert_eq!(analysis.internal_links, 3);
        assert_eq!(analysis.external_links, 2);
        assert_eq!(analysis.self_links, 1);
        assert_eq!(analysis.followed_links, 4);
        assert_eq!(analysis.nofollow_links, 1);
        assert_eq!(analysis.sponsored_links, 1);
        assert_eq!(analysis.ugc_links, 1);
        assert_eq!(analysis.unique_internal_targets, 2);
        assert_eq!(analysis.internal_pages, 3);

        assert_eq!(
            analysis.orphan_count, 1,
            "only the seed page a has no incoming internal link"
        );
        assert_eq!(analysis.orphan_pages, vec!["https://x.com/a".to_string()]);

        assert_eq!(
            analysis.dead_end_count, 1,
            "only c has no internal outbound"
        );
        assert_eq!(analysis.dead_end_pages, vec!["https://x.com/c".to_string()]);

        let anchors: Vec<&str> = analysis
            .top_anchors
            .iter()
            .map(|a| a.anchor.as_str())
            .collect();
        assert_eq!(anchors.len(), 5, "all non-empty anchors are listed");
        assert!(anchors.contains(&"here"));
        assert!(anchors.contains(&"About us"));

        assert_eq!(analysis.anchor_quality.generic, 1, "\"here\" is generic");
        assert_eq!(analysis.anchor_quality.descriptive, 4);
        assert_eq!(analysis.anchor_quality.url_anchors, 0);
        assert_eq!(analysis.anchor_quality.empty, 0);

        assert_eq!(analysis.external_domains.len(), 2);
        let ads = analysis
            .external_domains
            .iter()
            .find(|d| d.domain == "ads.y.com")
            .unwrap();
        assert_eq!(ads.count, 1);
        assert!(ads.sponsored == 1);
        let forum = analysis
            .external_domains
            .iter()
            .find(|d| d.domain == "forum.z.com")
            .unwrap();
        assert!(forum.ugc == 1);
    }

    #[test]
    fn test_project_has_links_empty() {
        let repo = test_repo();
        assert!(!repo.project_has_links("p1").unwrap());
        repo.save_links_batch(&[link("https://x.com/a", "https://x.com/b", None, &[])])
            .unwrap();
        assert!(repo.project_has_links("p1").unwrap());
    }

    #[test]
    fn test_paginated_link_lists() {
        let repo = test_repo();
        let pages: Vec<CrawlResult> = (b'a'..=b'o').map(|c| page(&format!("https://x.com/{}", c as char))).collect();
        repo.save_results_batch(&pages).unwrap();

        let mut links = Vec::new();
        // root -> a..j (incoming internal), a..j -> root (outgoing internal).
        for c in b'a'..=b'j' {
            let p = format!("https://x.com/{}", c as char);
            links.push(link("https://x.com/root", &p, Some("hit"), &[]));
            links.push(link(&p, "https://x.com/root", Some("back"), &[]));
        }
        // One external link so the external-domains aggregation has a row.
        links.push(link("https://x.com/root", "https://ext.example.com", Some("ext"), &[]));
        // k..o (5 pages) get no links at all: orphans + dead ends.
        for l in &mut links {
            let from_host = l
                .from_url
                .split("//")
                .nth(1)
                .and_then(|h| h.split('/').next());
            let to_host = l
                .to_url
                .split("//")
                .nth(1)
                .and_then(|h| h.split('/').next());
            l.is_internal = from_host == to_host;
        }
        repo.save_links_batch(&links).unwrap();

        let (page1, total) = repo.orphan_pages_page("p1", 1, 3).unwrap();
        assert_eq!(total, 5, "k..o are the only orphans");
        assert_eq!(page1.len(), 3);
        let (page2, _) = repo.orphan_pages_page("p1", 2, 3).unwrap();
        assert_eq!(page2.len(), 2);
        let mut combined = page1;
        combined.extend(page2);
        let expected: Vec<String> = (b'k'..=b'o').map(|c| format!("https://x.com/{}", c as char)).collect();
        assert_eq!(combined, expected);

        let (dead, dtotal) = repo.dead_end_pages_page("p1", 1, 10).unwrap();
        assert_eq!(dtotal, 5, "k..o have no outgoing internal links");
        assert_eq!(dead, expected);

        let (anchors, atotal) = repo.top_anchors_page("p1", 1, 2).unwrap();
        assert_eq!(atotal, 3, "hit, back and ext are the distinct non-empty anchors");
        assert_eq!(anchors.len(), 2);
        let (anchors2, _) = repo.top_anchors_page("p1", 2, 2).unwrap();
        assert_eq!(anchors2.len(), 1);
        assert_eq!(anchors[0].count, 10, "hit is used by root for a..j");

        let (domains, dtotal2) = repo.external_domains_page("p1", 1, 10).unwrap();
        assert_eq!(dtotal2, 1);
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0].domain, "ext.example.com");
        assert_eq!(domains[0].count, 1);
    }
}
