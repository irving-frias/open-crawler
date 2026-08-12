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
    fn orphan_pages(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<(usize, Vec<String>), AppError> {
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
        let mut stmt = self.conn.prepare(&format!("{where_} LIMIT ?2"))?;
        let urls = stmt
            .query_map(params![project_id, limit], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((count as usize, urls))
    }

    /// Pages that point to no internal page (dead ends in the crawl graph).
    fn dead_end_pages(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<(usize, Vec<String>), AppError> {
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
        let mut stmt = self.conn.prepare(&format!("{where_} LIMIT ?2"))?;
        let urls = stmt
            .query_map(params![project_id, limit], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((count as usize, urls))
    }

    fn top_anchors(&self, project_id: &str, limit: usize) -> Result<Vec<AnchorAgg>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT anchor_text, COUNT(*) AS c FROM page_links
             WHERE project_id = ?1 AND trim(coalesce(anchor_text, '')) != ''
             GROUP BY anchor_text ORDER BY c DESC LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![project_id, limit], |row| {
                Ok(AnchorAgg {
                    anchor: row.get(0)?,
                    count: row.get::<_, i64>(1)? as usize,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// External domains aggregated in Rust: external link counts are small
    /// compared to internal ones, and the `url` crate parses hosts more
    /// correctly than string surgery in SQL.
    fn external_domains(&self, project_id: &str, limit: usize) -> Result<Vec<DomainAgg>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT to_url, is_follow, is_sponsored, is_ugc
             FROM page_links WHERE project_id = ?1 AND is_internal = 0",
        )?;
        let mut acc: Vec<(String, usize, usize, usize, usize)> = Vec::new();
        let rows = stmt.query_map(params![project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)? != 0,
                row.get::<_, i32>(2)? != 0,
                row.get::<_, i32>(3)? != 0,
            ))
        })?;
        for row in rows {
            let (url, follow, sponsored, ugc) = row?;
            let Some(host) = url::Url::parse(&url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
            else {
                continue;
            };
            let host = host.to_ascii_lowercase();
            match acc.iter_mut().find(|(h, ..)| h == &host) {
                Some((_, count, nf, sp, u)) => {
                    *count += 1;
                    if !follow {
                        *nf += 1;
                    }
                    if sponsored {
                        *sp += 1;
                    }
                    if ugc {
                        *u += 1;
                    }
                }
                None => acc.push((
                    host,
                    1,
                    usize::from(!follow),
                    usize::from(sponsored),
                    usize::from(ugc),
                )),
            }
        }
        acc.sort_by_key(|a| std::cmp::Reverse(a.1));
        Ok(acc
            .into_iter()
            .take(limit)
            .map(|(domain, count, nofollow, sponsored, ugc)| DomainAgg {
                domain,
                count,
                nofollow,
                sponsored,
                ugc,
            })
            .collect())
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
        let (orphan_count, orphan_pages) = self.orphan_pages(project_id, 100)?;
        let (dead_end_count, dead_end_pages) = self.dead_end_pages(project_id, 100)?;
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
            top_anchors: self.top_anchors(project_id, 20)?,
            anchor_quality: self.anchor_quality(project_id)?,
            external_domains: self.external_domains(project_id, 50)?,
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
}
