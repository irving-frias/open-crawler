use rusqlite::params;

use crate::error::AppError;
use crate::models::{ChangedUrl, CompareResult, CrawlSnapshot, SnapshotStats, UrlFieldDiff};

use super::CrawlRepo;

struct SnapshotRow {
    url: String,
    status_code: Option<i32>,
    title: Option<String>,
    meta_description: Option<String>,
    size_bytes: Option<i64>,
    load_time_ms: Option<i64>,
    is_indexable: Option<i32>,
    readability_score: Option<f64>,
    seo_score: Option<f64>,
}

impl SnapshotRow {
    fn diff(&self, other: &SnapshotRow) -> Vec<UrlFieldDiff> {
        let mut diffs = Vec::new();
        push_diff(&mut diffs, "status_code", self.status_code.map(|v| v.to_string()), other.status_code.map(|v| v.to_string()));
        push_diff(&mut diffs, "title", self.title.clone(), other.title.clone());
        push_diff(&mut diffs, "meta_description", self.meta_description.clone(), other.meta_description.clone());
        push_diff(&mut diffs, "size_bytes", self.size_bytes.map(|v| v.to_string()), other.size_bytes.map(|v| v.to_string()));
        push_diff(&mut diffs, "load_time_ms", self.load_time_ms.map(|v| v.to_string()), other.load_time_ms.map(|v| v.to_string()));
        push_diff(&mut diffs, "is_indexable", self.is_indexable.map(|v| v.to_string()), other.is_indexable.map(|v| v.to_string()));
        push_diff(&mut diffs, "readability_score", self.readability_score.map(|v| format!("{:.1}", v)), other.readability_score.map(|v| format!("{:.1}", v)));
        push_diff(&mut diffs, "seo_score", self.seo_score.map(|v| format!("{:.1}", v)), other.seo_score.map(|v| format!("{:.1}", v)));
        diffs
    }
}

fn push_diff(
    diffs: &mut Vec<UrlFieldDiff>,
    field: &str,
    before: Option<String>,
    after: Option<String>,
) {
    if before != after {
        diffs.push(UrlFieldDiff {
            field: field.to_string(),
            before,
            after,
        });
    }
}

impl<'a> CrawlRepo<'a> {
    /// Snapshot the latest crawl for a project so it can be compared later.
    pub fn create_crawl_snapshot(
        &self,
        project_id: &str,
        config_id: &str,
    ) -> Result<CrawlSnapshot, AppError> {
        let snapshot_id = uuid::Uuid::new_v4().to_string();
        let snapshot_time = chrono::Utc::now().to_rfc3339();

        let total_pages: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND config_id = ?2",
            params![project_id, config_id],
            |r| r.get(0),
        )?;

        if total_pages == 0 {
            return Ok(CrawlSnapshot {
                id: snapshot_id,
                project_id: project_id.to_string(),
                snapshot_time,
                total_pages: 0,
                indexed_pages: 0,
                broken_pages: 0,
                avg_load_ms: 0.0,
                avg_size_bytes: 0.0,
                avg_readability: None,
                avg_seo_score: None,
            });
        }

        let indexed_pages: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND config_id = ?2 AND is_indexable = 1",
            params![project_id, config_id],
            |r| r.get(0),
        )?;
        let broken_pages: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1 AND config_id = ?2 AND status_code >= 400 AND blocked = 0",
            params![project_id, config_id],
            |r| r.get(0),
        )?;
        let avg_load_ms: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(load_time_ms), 0) FROM crawled_pages WHERE project_id = ?1 AND config_id = ?2 AND load_time_ms IS NOT NULL",
            params![project_id, config_id],
            |r| r.get(0),
        )?;
        let avg_size_bytes: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(size_bytes), 0) FROM crawled_pages WHERE project_id = ?1 AND config_id = ?2 AND size_bytes IS NOT NULL",
            params![project_id, config_id],
            |r| r.get(0),
        )?;
        let avg_readability: Option<f64> = self.conn.query_row(
            "SELECT AVG(readability_score) FROM crawled_pages WHERE project_id = ?1 AND config_id = ?2 AND readability_score IS NOT NULL",
            params![project_id, config_id],
            |r| r.get(0),
        )?;
        let avg_seo_score: Option<f64> = self.conn.query_row(
            "SELECT AVG(seo_score) FROM crawled_pages WHERE project_id = ?1 AND config_id = ?2 AND seo_score IS NOT NULL",
            params![project_id, config_id],
            |r| r.get(0),
        )?;

        let mut status_stmt = self.conn.prepare(
            "SELECT status_code, COUNT(*) FROM crawled_pages
             WHERE project_id = ?1 AND config_id = ?2 AND status_code IS NOT NULL AND blocked = 0
             GROUP BY status_code",
        )?;
        let status_rows = status_stmt
            .query_map(params![project_id, config_id], |row| {
                Ok((row.get::<_, i32>(0)?, row.get::<_, u32>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let status_counts_json = serde_json::to_string(
            &status_rows
                .iter()
                .map(|(code, count)| (code.to_string(), count))
                .collect::<std::collections::HashMap<_, _>>(),
        )?;

        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO crawl_snapshots
             (id, project_id, config_id, snapshot_time, total_pages, indexed_pages, broken_pages, avg_load_ms, avg_size_bytes, avg_readability, avg_seo_score, status_counts_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                snapshot_id,
                project_id,
                config_id,
                snapshot_time,
                total_pages,
                indexed_pages,
                broken_pages,
                avg_load_ms,
                avg_size_bytes,
                avg_readability,
                avg_seo_score,
                status_counts_json,
            ],
        )?;

        let rows = {
            let mut stmt = tx.prepare(
                "SELECT id, url, status_code, title, meta_description, size_bytes, load_time_ms, is_indexable, readability_score, seo_score
                 FROM crawled_pages WHERE project_id = ?1 AND config_id = ?2",
            )?;
            let rows = stmt
                .query_map(params![project_id, config_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<i32>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                        row.get::<_, Option<i64>>(6)?,
                        row.get::<_, Option<i32>>(7)?,
                        row.get::<_, Option<f64>>(8)?,
                        row.get::<_, Option<f64>>(9)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            drop(stmt);
            rows
        };

        {
            let mut ins = tx.prepare(
                "INSERT OR REPLACE INTO crawl_snapshot_data
                 (snapshot_id, page_id, url, status_code, title, meta_description, size_bytes, load_time_ms, is_indexable, readability_score, seo_score)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )?;
            for (page_id, url, status, title, desc, size, load, indexable, readability, seo) in rows {
                ins.execute(params![
                    snapshot_id,
                    page_id,
                    url,
                    status,
                    title,
                    desc,
                    size,
                    load,
                    indexable,
                    readability,
                    seo,
                ])?;
            }
        }

        tx.commit()?;

        Ok(CrawlSnapshot {
            id: snapshot_id,
            project_id: project_id.to_string(),
            snapshot_time,
            total_pages,
            indexed_pages,
            broken_pages,
            avg_load_ms,
            avg_size_bytes,
            avg_readability,
            avg_seo_score,
        })
    }

    pub fn list_crawl_snapshots(&self, project_id: &str) -> Result<Vec<CrawlSnapshot>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, snapshot_time, total_pages, indexed_pages, broken_pages, avg_load_ms, avg_size_bytes, avg_readability, avg_seo_score
             FROM crawl_snapshots WHERE project_id = ?1
             ORDER BY snapshot_time DESC LIMIT 50",
        )?;
        let snapshots = stmt
            .query_map(params![project_id], |row| {
                Ok(CrawlSnapshot {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    snapshot_time: row.get(2)?,
                    total_pages: row.get(3)?,
                    indexed_pages: row.get(4)?,
                    broken_pages: row.get(5)?,
                    avg_load_ms: row.get(6)?,
                    avg_size_bytes: row.get(7)?,
                    avg_readability: row.get(8)?,
                    avg_seo_score: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(snapshots)
    }

    fn snapshot_stats(&self, snapshot_id: &str) -> Result<SnapshotStats, AppError> {
        let (total_pages, indexed_pages, broken_pages, avg_load_ms, avg_size_bytes, avg_readability, avg_seo_score) =
            self.conn.query_row(
                "SELECT total_pages, indexed_pages, broken_pages, avg_load_ms, avg_size_bytes, avg_readability, avg_seo_score
                 FROM crawl_snapshots WHERE id = ?1",
                params![snapshot_id],
                |row| {
                    Ok((
                        row.get::<_, u32>(0)?,
                        row.get::<_, u32>(1)?,
                        row.get::<_, u32>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, Option<f64>>(5)?,
                        row.get::<_, Option<f64>>(6)?,
                    ))
                },
            )?;
        Ok(SnapshotStats {
            total_pages,
            indexed_pages,
            broken_pages,
            avg_load_ms,
            avg_size_bytes,
            avg_readability,
            avg_seo_score,
        })
    }

    fn snapshot_rows(
        &self,
        snapshot_id: &str,
    ) -> Result<std::collections::HashMap<String, SnapshotRow>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT url, status_code, title, meta_description, size_bytes, load_time_ms, is_indexable, readability_score, seo_score
             FROM crawl_snapshot_data WHERE snapshot_id = ?1",
        )?;
        let rows = stmt
            .query_map(params![snapshot_id], |row| {
                Ok(SnapshotRow {
                    url: row.get(0)?,
                    status_code: row.get(1)?,
                    title: row.get(2)?,
                    meta_description: row.get(3)?,
                    size_bytes: row.get(4)?,
                    load_time_ms: row.get(5)?,
                    is_indexable: row.get(6)?,
                    readability_score: row.get(7)?,
                    seo_score: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows.into_iter().map(|r| (r.url.clone(), r)).collect())
    }

    /// Diff two snapshots by URL. `snapshot_a` is the older, `snapshot_b` the newer.
    pub fn compare_crawl_snapshots(
        &self,
        snapshot_a: &str,
        snapshot_b: &str,
    ) -> Result<CompareResult, AppError> {
        let map_a = self.snapshot_rows(snapshot_a)?;
        let map_b = self.snapshot_rows(snapshot_b)?;

        let mut new_urls: Vec<String> = map_b
            .keys()
            .filter(|url| !map_a.contains_key(*url))
            .cloned()
            .collect();
        new_urls.sort();

        let mut removed_urls: Vec<String> = map_a
            .keys()
            .filter(|url| !map_b.contains_key(*url))
            .cloned()
            .collect();
        removed_urls.sort();

        let mut changed_urls: Vec<ChangedUrl> = Vec::new();
        let mut unchanged_count: u32 = 0;
        for (url, row_b) in &map_b {
            let Some(row_a) = map_a.get(url) else {
                continue;
            };
            let diffs = row_a.diff(row_b);
            if diffs.is_empty() {
                unchanged_count += 1;
            } else {
                changed_urls.push(ChangedUrl {
                    url: url.clone(),
                    diffs,
                });
            }
        }
        changed_urls.sort_by(|a, b| a.url.cmp(&b.url));

        Ok(CompareResult {
            new_urls,
            removed_urls,
            changed_urls,
            unchanged_count,
            before: self.snapshot_stats(snapshot_a)?,
            after: self.snapshot_stats(snapshot_b)?,
        })
    }
}
