use rusqlite::params;
use std::sync::Arc;

use crate::error::AppError;
use crate::models::{
    DashboardStats, DuplicateGroup, DuplicateGroupUrl, IssueCount, KeywordAggregate, SiteGraph,
    SiteGraphEdge, SiteGraphEdgePage, SiteGraphNode, SiteTreeFullNode, SiteTreeNode, StatusBucket,
};
use crate::{GraphEdgesCacheValue, MAX_GRAPH_EDGES, MAX_GRAPH_NODES};

use super::CrawlRepo;

/// CTE that ranks every crawled page by graph importance (in+out degree, then
/// depth) and keeps only the top [`MAX_GRAPH_NODES`] URLs. Both the node query
/// and the edge query reuse this exact same CTE, so every returned edge's
/// endpoints always exist in the rendered node set.
const GRAPH_TOP_URLS_CTE: &str = r#"
in_deg AS (
    SELECT to_url AS url, COUNT(*) AS c
    FROM page_links WHERE project_id = ?1 GROUP BY to_url
),
out_deg AS (
    SELECT from_url AS url, COUNT(*) AS c
    FROM page_links WHERE project_id = ?1 GROUP BY from_url
),
top_urls AS (
    SELECT cp.url AS url,
           COALESCE(id.c, 0) AS in_degree,
           COALESCE(od.c, 0) AS out_degree,
           MIN(cp.depth) AS depth
    FROM crawled_pages cp
    LEFT JOIN in_deg id ON id.url = cp.url
    LEFT JOIN out_deg od ON od.url = cp.url
    WHERE cp.project_id = ?1
    GROUP BY cp.url
    ORDER BY (in_degree + out_degree) DESC, depth ASC, cp.url ASC
    LIMIT ?2
)"#;

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
        let (sql, row_params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(
            from_url,
        ) = url
        {
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
                            (SELECT COUNT(*) FROM page_issues pi WHERE pi.page_id = cp.id)
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
                            (SELECT COUNT(*) FROM page_issues pi WHERE pi.page_id = crawled_pages.id)
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

    /// Returns the entire site tree for a project in a single round-trip:
    /// one query for the pages, one for the internal links, and the forest is
    /// assembled in memory (no per-node query like `get_site_tree`).
    pub fn get_site_tree_full(&self, project_id: &str) -> Result<Vec<SiteTreeFullNode>, AppError> {
        // Aggregate per URL (re-crawls can produce repeated rows for the same
        // URL, which the legacy tree dedupes by URL).
        let mut page_stmt = self.conn.prepare(
            "SELECT cp.url, cp.title, cp.status_code, cp.depth, COALESCE(pi.cnt, 0)
             FROM crawled_pages cp
             LEFT JOIN (
                 SELECT page_id, COUNT(*) AS cnt
                 FROM page_issues
                 WHERE project_id = ?1
                 GROUP BY page_id
             ) pi ON pi.page_id = cp.id
             WHERE cp.project_id = ?1
             GROUP BY cp.url",
        )?;
        type PageRow = (String, Option<String>, Option<i32>, i32, i64);
        let pages: Vec<PageRow> = page_stmt
            .query_map(params![project_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<i32>>(2)?,
                    row.get::<_, i32>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut by_url: std::collections::HashMap<String, SiteTreeFullNode> =
            std::collections::HashMap::new();
        for (url, title, status_code, depth, issue_count) in pages {
            let node_url = url.clone();
            by_url.insert(
                url,
                SiteTreeFullNode {
                    url: node_url,
                    title,
                    status_code: status_code.map(|s| s as u16),
                    depth: depth as u32,
                    issue_count: issue_count as u32,
                    has_children: false,
                    children: Vec::new(),
                },
            );
        }

        // Internal edges only: both endpoints are crawled pages of this project.
        let mut link_stmt = self.conn.prepare(
            "SELECT DISTINCT from_url, to_url
             FROM page_links
             WHERE project_id = ?1
               AND from_url <> to_url
               AND from_url IN (SELECT url FROM crawled_pages WHERE project_id = ?1)
               AND to_url IN (SELECT url FROM crawled_pages WHERE project_id = ?1)",
        )?;
        let links: Vec<(String, String)> = link_stmt
            .query_map(params![project_id], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        let mut children_map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (from, to) in links {
            children_map.entry(from).or_default().push(to);
        }

        // DFS from the depth-0 roots. A page is placed under its first-seen
        // parent so the result is a proper forest (no duplicated subtrees).
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut roots: Vec<SiteTreeFullNode> = Vec::new();

        fn build(
            url: &str,
            by_url: &std::collections::HashMap<String, SiteTreeFullNode>,
            children_map: &std::collections::HashMap<String, Vec<String>>,
            visited: &mut std::collections::HashSet<String>,
        ) -> Option<SiteTreeFullNode> {
            let mut node = by_url.get(url)?.clone();
            let mut kids: Vec<SiteTreeFullNode> = Vec::new();
            if let Some(children) = children_map.get(url) {
                for child in children {
                    if !visited.contains(child) {
                        visited.insert(child.clone());
                        if let Some(sub) = build(child, by_url, children_map, visited) {
                            kids.push(sub);
                        }
                    }
                }
            }
            kids.sort_by(|a, b| a.depth.cmp(&b.depth).then(a.url.cmp(&b.url)));
            node.has_children = !kids.is_empty();
            node.children = kids;
            Some(node)
        }

        let mut urls: Vec<String> = by_url.keys().cloned().collect();
        urls.sort_by(|a, b| {
            let (da, db) = (by_url[a].depth, by_url[b].depth);
            da.cmp(&db).then(a.cmp(b))
        });
        for url in urls {
            if by_url[&url].depth == 0 && !visited.contains(&url) {
                visited.insert(url.clone());
                if let Some(root) = build(&url, &by_url, &children_map, &mut visited) {
                    roots.push(root);
                }
            }
        }

        // Safety net: any page never reached from a depth-0 root (e.g. orphaned
        // after a partial crawl) is appended as an additional root.
        let mut orphan_urls: Vec<String> = by_url
            .keys()
            .filter(|u| !visited.contains(*u))
            .cloned()
            .collect();
        orphan_urls.sort();
        for url in orphan_urls {
            visited.insert(url.clone());
            if let Some(root) = build(&url, &by_url, &children_map, &mut visited) {
                roots.push(root);
            }
        }

        Ok(roots)
    }

    /// Computes (or loads from the per-project cache) the internal edge set
    /// restricted to the rendered node set (links whose source AND target are
    /// both among the top [`MAX_GRAPH_NODES`] pages). The set is capped at
    /// [`MAX_GRAPH_EDGES`] for sanity; the `truncated` flag reports whether
    /// anything was dropped.
    fn load_graph_edges(&self, project_id: &str) -> Result<Arc<Vec<SiteGraphEdge>>, AppError> {
        let cached = {
            let cache = self.graph_edges_cache.expect("graph cache attached");
            cache
                .lock()
                .unwrap()
                .get(project_id)
                .map(|v| v.edges.clone())
        };
        if let Some(edges) = cached {
            return Ok(edges);
        }

        let mut stmt = self.conn.prepare(&format!(
            "WITH {}
             SELECT DISTINCT pl.from_url, pl.to_url, pl.link_type, pl.is_follow
             FROM page_links pl
             JOIN top_urls tu_s ON tu_s.url = pl.from_url
             JOIN top_urls tu_t ON tu_t.url = pl.to_url
             WHERE pl.project_id = ?1 AND pl.from_url <> pl.to_url
             ORDER BY pl.from_url ASC, pl.to_url ASC
             LIMIT ?3",
            GRAPH_TOP_URLS_CTE
        ))?;
        let rows = stmt
            .query_map(
                params![project_id, MAX_GRAPH_NODES as i64, MAX_GRAPH_EDGES as i64],
                |row| {
                    Ok(SiteGraphEdge {
                        source: row.get(0)?,
                        target: row.get(1)?,
                        link_type: row.get(2)?,
                        is_follow: row.get::<_, i32>(3)? != 0,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;

        let truncated = rows.len() >= MAX_GRAPH_EDGES;
        let total = if truncated {
            // Dense graph: count the real total only when the cap was hit so
            // the UI can report it. This is a rare path (50k+ internal links
            // between the rendered nodes).
            self.conn.query_row(
                &format!(
                    "WITH {}
                     SELECT COUNT(*) FROM (
                         SELECT DISTINCT pl.from_url, pl.to_url
                         FROM page_links pl
                         JOIN top_urls tu_s ON tu_s.url = pl.from_url
                         JOIN top_urls tu_t ON tu_t.url = pl.to_url
                         WHERE pl.project_id = ?1 AND pl.from_url <> pl.to_url
                     )",
                    GRAPH_TOP_URLS_CTE
                ),
                params![project_id, MAX_GRAPH_NODES as i64],
                |r| r.get(0),
            )?
        } else {
            rows.len() as i64
        };

        let edges = Arc::new(rows);
        let cache = self.graph_edges_cache.expect("graph cache attached");
        cache.lock().unwrap().put(
            project_id.to_string(),
            GraphEdgesCacheValue {
                edges: edges.clone(),
                total: total as u32,
                truncated,
            },
        );
        Ok(edges)
    }

    /// Returns the number of internal edges between the rendered nodes,
    /// reusing the cached edge set when available (avoids a second DISTINCT
    /// scan over page_links).
    fn graph_edge_count(&self, project_id: &str) -> Result<u32, AppError> {
        if let Some(cache) = self.graph_edges_cache {
            {
                let mut cache = cache.lock().unwrap();
                if let Some(v) = cache.get(project_id) {
                    return Ok(v.total);
                }
            }
            self.load_graph_edges(project_id)?;
            let mut cache = cache.lock().unwrap();
            let v = cache.get(project_id).expect("graph edges just loaded");
            return Ok(v.total);
        }
        Ok(self.load_graph_edges(project_id)?.len() as u32)
    }

    /// Loads the interactive site graph: the top crawled pages by importance
    /// (deduplicated by URL) plus the total internal edge count among those
    /// pages so the frontend can render nodes immediately. The edges
    /// themselves are streamed via [`Self::get_site_graph_edges`].
    ///
    /// The node set is capped at [`MAX_GRAPH_NODES`] for huge sites: only the
    /// most-linked pages are rendered, which keeps Cytoscape responsive. Edges
    /// are likewise restricted to pairs of rendered nodes.
    ///
    /// Degrees are computed in CTEs before joining to the pages. Joining
    /// `page_links` twice directly would cross-multiply hub pages (a page with
    /// N in-links and M out-links materializes N×M intermediate rows), which
    /// stalls the shared DB lock for large sites.
    pub fn get_site_graph(&self, project_id: &str) -> Result<SiteGraph, AppError> {
        let total_nodes: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT url) FROM crawled_pages WHERE project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )?;

        let mut page_stmt = self.conn.prepare(&format!(
            "WITH {}
             SELECT tu.url,
                    MIN(cp.title),
                    MAX(cp.status_code),
                    tu.depth,
                    COALESCE(SUM(
                        CASE WHEN pi.id IS NULL THEN 0 ELSE 1 END
                    ), 0) AS issue_count,
                    MAX(cp.seo_score),
                    MAX(CASE WHEN cp.is_indexable = 0 THEN 0 ELSE 1 END) AS indexable,
                    MAX(cp.blocked),
                    MAX(cp.size_bytes),
                    MAX(cp.load_time_ms),
                    tu.in_degree,
                    tu.out_degree
             FROM top_urls tu
             JOIN crawled_pages cp ON cp.url = tu.url AND cp.project_id = ?1
             LEFT JOIN page_issues pi ON pi.page_id = cp.id AND pi.project_id = ?1
             GROUP BY tu.url, tu.in_degree, tu.out_degree, tu.depth
             ORDER BY (tu.in_degree + tu.out_degree) DESC, tu.depth ASC, tu.url ASC",
            GRAPH_TOP_URLS_CTE
        ))?;
        let nodes: Vec<SiteGraphNode> = page_stmt
            .query_map(params![project_id, MAX_GRAPH_NODES as i64], |row| {
                Ok(SiteGraphNode {
                    url: row.get(0)?,
                    title: row.get(1)?,
                    status_code: row.get::<_, Option<i32>>(2)?.map(|s| s as u16),
                    depth: row.get::<_, i32>(3)? as u32,
                    issue_count: row.get::<_, i64>(4)? as u32,
                    seo_score: row.get(5)?,
                    is_indexable: row.get::<_, Option<i32>>(6)?.map(|v| v == 1),
                    blocked: row.get(7)?,
                    size_bytes: row.get::<_, Option<i64>>(8)?.map(|v| v as u64),
                    load_time_ms: row.get::<_, Option<i64>>(9)?.map(|v| v as u64),
                    in_degree: row.get::<_, i64>(10)? as u32,
                    out_degree: row.get::<_, i64>(11)? as u32,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(page_stmt);

        let edge_count = self.graph_edge_count(project_id)?;
        let edges_truncated = if let Some(cache) = self.graph_edges_cache {
            cache
                .lock()
                .unwrap()
                .get(project_id)
                .map(|v| v.truncated)
                .unwrap_or(false)
        } else {
            false
        };

        Ok(SiteGraph {
            nodes,
            edge_count,
            edges_truncated,
            total_nodes: total_nodes as u32,
            nodes_truncated: total_nodes as usize > MAX_GRAPH_NODES,
        })
    }

    /// Returns one page of the internal edge set. Edges are cached in memory
    /// after the first request, so subsequent pages are served without hitting
    /// SQLite again.
    pub fn get_site_graph_edges(
        &self,
        project_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<SiteGraphEdgePage, AppError> {
        let edges = self.load_graph_edges(project_id)?;
        let total = edges.len() as u32;
        let start = (offset as usize).min(edges.len());
        let end = (start + limit as usize).min(edges.len());
        let page_edges = edges[start..end].to_vec();
        let truncated = if let Some(cache) = self.graph_edges_cache {
            cache
                .lock()
                .unwrap()
                .get(project_id)
                .map(|v| v.truncated)
                .unwrap_or(false)
        } else {
            false
        };
        Ok(SiteGraphEdgePage {
            edges: page_edges,
            offset,
            total,
            done: end >= edges.len(),
            truncated,
        })
    }

    pub fn get_semantic_issue_counts(&self, project_id: &str) -> Result<Vec<IssueCount>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT
                issue_type,
                severity,
                COUNT(*) as cnt
             FROM page_issues
             WHERE project_id = ?1
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

        let total_pages = count("SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1")?;
        let indexed_pages =
            count("SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND is_indexable = 1")?;
        let broken_pages = count(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND status_code >= 400 AND blocked = 0",
        )?;
        let blocked_pages =
            count("SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND blocked = 1")?;
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

        let avg_seo_score: Option<f64> = self.conn.query_row(
            "SELECT AVG(seo_score) FROM crawled_pages WHERE project_id = ?1 AND seo_score IS NOT NULL",
            params![project_id],
            |r| r.get(0),
        )?;

        let mut status_stmt = self.conn.prepare(
            "SELECT status_code, COUNT(*) FROM crawled_pages
             WHERE project_id = ?1 AND status_code IS NOT NULL AND blocked = 0
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
            blocked_pages,
            avg_load_ms,
            avg_size_bytes,
            avg_readability,
            avg_seo_score,
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
    ///
    /// Uses LSH bucketing instead of an O(n²) pairwise scan: the 64-bit hash is
    /// split into 16 bands of 4 bits, and candidates are compared only within
    /// shared buckets. Two hashes within hamming distance 10 share at least 6
    /// identical bands, so every near-duplicate pair is still compared exactly.
    pub fn compute_duplicate_groups(&self, project_id: &str) -> Result<u32, AppError> {
        self.conn.execute(
            "UPDATE crawled_pages SET duplicate_group_id = NULL WHERE project_id = ?1",
            params![project_id],
        )?;

        let mut stmt = self.conn.prepare(
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

        // Exact duplicates share the same hash — group them immediately so a
        // large run of identical pages never forms an O(n²) candidate bucket.
        let mut exact: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
        for (i, (_, h)) in hashes.iter().enumerate() {
            if let Some(&prev) = exact.get(h) {
                union(&mut parent, i, prev);
            } else {
                exact.insert(*h, i);
            }
        }

        // LSH bucketing over 16 bands of 4 bits. Two hashes at distance <= 10
        // differ in at most 10 bands, so at least 6 are identical and they land
        // in at least 6 shared buckets — guaranteed to be compared.
        let mut buckets: std::collections::HashMap<(usize, u8), Vec<usize>> =
            std::collections::HashMap::new();
        for (i, (_, h)) in hashes.iter().enumerate() {
            for band in 0..16u32 {
                let bits = ((h >> (band * 4)) & 0xF) as u8;
                buckets.entry((band as usize, bits)).or_default().push(i);
            }
        }
        for indices in buckets.values() {
            for a in 0..indices.len() {
                for b in (a + 1)..indices.len() {
                    let (ia, ib) = (indices[a], indices[b]);
                    if find(&mut parent, ia) == find(&mut parent, ib) {
                        continue;
                    }
                    if simhash::hamming_distance(hashes[ia].1, hashes[ib].1) <= 10 {
                        union(&mut parent, ia, ib);
                    }
                }
            }
        }

        let mut group_map: std::collections::HashMap<usize, i64> = std::collections::HashMap::new();
        let mut member_counts: std::collections::HashMap<usize, u32> =
            std::collections::HashMap::new();
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

        // Persist groups in a single transaction.
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut update = tx.prepare(
                "UPDATE crawled_pages SET duplicate_group_id = ?1
                 WHERE project_id = ?2 AND url = ?3",
            )?;
            for i in 0..n {
                let root = find(&mut parent, i);
                let Some(gid) = group_map.get(&root) else {
                    continue;
                };
                let url = &rows[hashes[i].0].0;
                update.execute(params![gid, project_id, url])?;
            }
        }
        tx.commit()?;

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
    pub fn get_keywords(
        &self,
        project_id: &str,
        limit: u32,
    ) -> Result<Vec<KeywordAggregate>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT keywords_json FROM crawled_pages
                 WHERE project_id = ?1 AND keywords_json IS NOT NULL",
        )?;
        let rows = stmt
            .query_map(params![project_id], |row| row.get::<_, String>(0))?
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
                let count = item.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
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
