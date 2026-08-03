use rusqlite::params;

use crate::error::AppError;
use crate::models::{
    DashboardStats, DuplicateGroup, DuplicateGroupUrl, IssueCount, KeywordAggregate, SiteTreeNode,
    StatusBucket,
};

use super::CrawlRepo;

impl<'a> CrawlRepo<'a> {
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
                let sql = "SELECT pl.to_url, cp.title, cp.status_code, cp.depth,
                            EXISTS (
                                SELECT 1 FROM page_links pl2
                                WHERE pl2.from_url = pl.to_url
                                  AND pl2.to_url <> pl2.from_url
                                  AND EXISTS (
                                    SELECT 1 FROM crawled_pages cp2
                                    WHERE cp2.project_id = ?3 AND cp2.url = pl2.to_url
                                  )
                            ),
                            COALESCE(json_array_length(cp.semantic_issues_json), 0)
                     FROM page_links pl
                     JOIN crawled_pages cp ON cp.url = pl.to_url AND cp.project_id = ?3
                     WHERE pl.project_id = ?3 AND pl.from_url = ?4 AND pl.to_url <> ?4
                     GROUP BY pl.to_url
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
                            ),
                            COALESCE(json_array_length(semantic_issues_json), 0)
                     FROM crawled_pages
                     WHERE project_id = ?1 AND depth = 0
                     GROUP BY url
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
        let rows = stmt
            .query_map(
                rusqlite::params_from_iter(row_params.iter().map(|p| p.as_ref())),
                Self::row_to_site_tree_node,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        // Defensive dedupe by URL: crawled_pages can hold repeated rows for the
        // same URL (e.g. from re-crawls), which would break keyed each blocks.
        let mut seen = std::collections::HashSet::new();
        let mut children: Vec<SiteTreeNode> = Vec::with_capacity(rows.len());
        for node in rows {
            if seen.insert(node.url.clone()) {
                children.push(node);
            }
        }
        children.truncate(limit as usize);

        Ok(children)
    }

    fn row_to_site_tree_node(row: &rusqlite::Row) -> Result<SiteTreeNode, rusqlite::Error> {
        Ok(SiteTreeNode {
            url: row.get(0)?,
            title: row.get(1)?,
            status_code: row.get::<_, Option<i32>>(2)?.map(|s| s as u16),
            depth: row.get::<_, i32>(3)? as u32,
            has_children: row.get(4)?,
            issue_count: row.get::<_, i64>(5)? as u32,
        })
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

    /// Aggregated metrics for the Dashboard tab.
    pub fn get_dashboard_stats(&self, project_id: &str) -> Result<DashboardStats, AppError> {
        let count = |sql: &str| -> Result<u32, AppError> {
            Ok(self
                .conn
                .query_row(sql, params![project_id], |r| r.get(0))?)
        };

        let total_pages = count(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1",
        )?;
        let indexed_pages = count(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND is_indexable = 1",
        )?;
        let broken_pages = count(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND status_code >= 400",
        )?;
        let duplicate_count = count(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND duplicate_group_id IS NOT NULL",
        )?;
        let missing_title_count = count(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND (title IS NULL OR trim(title) = '')",
        )?;
        let missing_description_count = count(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND (meta_description IS NULL OR trim(meta_description) = '')",
        )?;
        let missing_h1_count = count(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND (h1 IS NULL OR trim(h1) = '')",
        )?;

        let avg_load_ms: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(load_time_ms), 0) FROM crawled_pages WHERE project_id = ?1 AND load_time_ms IS NOT NULL",
            params![project_id],
            |r| r.get(0),
        )?;

        let avg_size_bytes: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(size_bytes), 0) FROM crawled_pages WHERE project_id = ?1 AND size_bytes IS NOT NULL",
            params![project_id],
            |r| r.get(0),
        )?;

        let avg_readability: Option<f64> = self.conn.query_row(
            "SELECT AVG(readability_score) FROM crawled_pages WHERE project_id = ?1 AND readability_score IS NOT NULL",
            params![project_id],
            |r| r.get(0),
        )?;

        let mut status_stmt = self.conn.prepare(
            "SELECT status_code, COUNT(*) FROM crawled_pages
             WHERE project_id = ?1 AND status_code IS NOT NULL
             GROUP BY status_code ORDER BY COUNT(*) DESC LIMIT 8",
        )?;
        let status_distribution = status_stmt
            .query_map(params![project_id], |row| {
                Ok(StatusBucket {
                    status: row.get::<_, i32>(0)? as u16,
                    count: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let top_issues = self.get_semantic_issue_counts(project_id)?;

        Ok(DashboardStats {
            total_pages,
            indexed_pages,
            broken_pages,
            avg_load_ms,
            avg_size_bytes,
            avg_readability,
            duplicate_count,
            missing_title_count,
            missing_description_count,
            missing_h1_count,
            status_distribution,
            top_issues,
        })
    }

    /// Recomputes duplicate groups for a project using simhash hamming distance
    /// (distance <= 10) on each page's content_hash. Clears previous groups.
    pub fn compute_duplicate_groups(&self, project_id: &str) -> Result<u32, AppError> {
        self.conn.execute(
            "UPDATE crawled_pages SET duplicate_group_id = NULL WHERE project_id = ?1",
            params![project_id],
        )?;

        let mut stmt = self
            .conn
            .prepare(
                "SELECT url, content_hash FROM crawled_pages
                 WHERE project_id = ?1 AND content_hash IS NOT NULL",
            )?;
        let rows: Vec<(String, Option<String>)> = stmt
            .query_map(params![project_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let hashes: Vec<(usize, u64)> = rows
            .iter()
            .enumerate()
            .filter_map(|(i, (_, h))| {
                h.as_deref()
                    .and_then(|hex| u64::from_str_radix(hex, 16).ok())
                    .map(|v| (i, v))
            })
            .collect();

        if hashes.is_empty() {
            return Ok(0);
        }

        let n = hashes.len();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(parent: &mut Vec<usize>, x: usize) -> usize {
            if parent[x] != x {
                parent[x] = find(parent, parent[x]);
            }
            parent[x]
        }
        fn union(parent: &mut Vec<usize>, a: usize, b: usize) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                parent[rb] = ra;
            }
        }

        for i in 0..n {
            for j in (i + 1)..n {
                if simhash::hamming_distance(hashes[i].1, hashes[j].1) <= 10 {
                    union(&mut parent, i, j);
                }
            }
        }

        let mut group_map: std::collections::HashMap<usize, i64> = std::collections::HashMap::new();
        let mut member_counts: std::collections::HashMap<usize, u32> = std::collections::HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            *member_counts.entry(root).or_insert(0) += 1;
        }
        let mut group_count = 0u32;
        for (root, count) in &member_counts {
            if *count < 2 {
                continue;
            }
            group_count += 1;
            group_map.insert(*root, group_count as i64);
        }
        for i in 0..n {
            let root = find(&mut parent, i);
            let Some(gid) = group_map.get(&root) else {
                continue;
            };
            let url = &rows[hashes[i].0].0;
            self.conn.execute(
                "UPDATE crawled_pages SET duplicate_group_id = ?1
                 WHERE project_id = ?2 AND url = ?3",
                params![gid, project_id, url],
            )?;
        }

        Ok(group_count)
    }

    pub fn get_duplicate_groups(&self, project_id: &str) -> Result<Vec<DuplicateGroup>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT url, title, status_code, duplicate_group_id FROM crawled_pages
             WHERE project_id = ?1 AND duplicate_group_id IS NOT NULL
             ORDER BY duplicate_group_id ASC",
        )?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<i32>>(2)?.map(|s| s as u16),
                    row.get::<_, i64>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut groups: std::collections::HashMap<i64, Vec<DuplicateGroupUrl>> =
            std::collections::HashMap::new();
        for (url, title, status_code, gid) in rows {
            let entry = groups.entry(gid).or_default();
            if entry.iter().any(|u| u.url == url) {
                continue;
            }
            entry.push(DuplicateGroupUrl {
                url,
                title,
                status_code,
            });
        }

        let mut result: Vec<DuplicateGroup> = groups
            .into_iter()
            .filter(|(_, urls)| urls.len() > 1)
            .map(|(id, urls)| {
                let size = urls.len() as u32;
                DuplicateGroup { id, size, urls }
            })
            .collect();
        result.sort_by(|a, b| b.size.cmp(&a.size).then(a.id.cmp(&b.id)));
        Ok(result)
    }

    /// Aggregates per-page keyword frequency lists into project-wide totals.
    pub fn get_keywords(&self, project_id: &str, limit: u32) -> Result<Vec<KeywordAggregate>, AppError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT keywords_json FROM crawled_pages
                 WHERE project_id = ?1 AND keywords_json IS NOT NULL",
            )?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                row.get::<_, String>(0)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut totals: std::collections::HashMap<String, (u64, u32)> =
            std::collections::HashMap::new();
        for json in rows {
            let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&json) else {
                continue;
            };
            for item in items {
                let Some(keyword) = item.get("keyword").and_then(|v| v.as_str()) else {
                    continue;
                };
                let count = item
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let entry = totals.entry(keyword.to_string()).or_insert((0, 0));
                entry.0 += count;
                entry.1 += 1;
            }
        }

        let mut result: Vec<KeywordAggregate> = totals
            .into_iter()
            .map(|(keyword, (count, pages))| KeywordAggregate {
                keyword,
                count,
                pages,
            })
            .collect();
        result.sort_by(|a, b| b.count.cmp(&a.count).then(a.keyword.cmp(&b.keyword)));
        result.truncate(limit as usize);
        Ok(result)
    }
}
