use rusqlite::Connection;
use tracing::info;

use crate::error::AppError;

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_add_content_analysis",
        include_str!("001_add_content_analysis.sql"),
    ),
    (
        "002_add_social_meta",
        include_str!("002_add_social_meta.sql"),
    ),
    ("003_add_pagespeed", include_str!("003_add_pagespeed.sql")),
    (
        "004_create_crawl_snapshots",
        include_str!("004_create_crawl_snapshots.sql"),
    ),
    (
        "005_add_blocked_flag",
        include_str!("005_add_blocked_flag.sql"),
    ),
    (
        "006_add_page_issues",
        include_str!("006_add_page_issues.sql"),
    ),
    ("007_scheduled_jobs", include_str!("007_scheduled_jobs.sql")),
    ("008_seo_audit", include_str!("008_seo_audit.sql")),
    ("009_seo_redirects", include_str!("009_seo_redirects.sql")),
    ("010_seo_clusters", include_str!("010_seo_clusters.sql")),
    ("011_seo_overview", include_str!("011_seo_overview.sql")),
    (
        "012_add_graph_indexes",
        include_str!("012_add_graph_indexes.sql"),
    ),
    (
        "013_add_pages_project_url_index",
        include_str!("013_add_pages_project_url_index.sql"),
    ),
    ("014_link_analysis", include_str!("014_link_analysis.sql")),
    (
        "015_response_headers",
        include_str!("015_response_headers.sql"),
    ),
    ("016_perf_indexes", include_str!("016_perf_indexes.sql")),
];

pub fn run(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            name TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let mut applied: Vec<String> = conn
        .prepare("SELECT name FROM schema_migrations")?
        .query_map([], |row| row.get(0))?
        .filter_map(|row| row.ok())
        .collect();

    applied.sort();

    for (name, sql) in MIGRATIONS {
        if applied.iter().any(|n| n == name) {
            continue;
        }

        info!("Applying migration {name}");
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute("INSERT INTO schema_migrations (name) VALUES (?1)", [name])?;
        tx.commit()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;

    fn column_names(conn: &Connection, table: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .unwrap();
        stmt.query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    #[test]
    fn test_migrations_apply_columns_and_snapshot_tables() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let cols = column_names(&conn, "crawled_pages");
        for expected in [
            "readability_score",
            "content_hash",
            "duplicate_group_id",
            "keywords_json",
            "og_json",
            "pagespeed_score",
            "pagespeed_json",
            "seo_score",
            "seo_audit_json",
        ] {
            assert!(
                cols.contains(&expected.to_string()),
                "missing column {}",
                expected
            );
        }

        let snapshots: String = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='crawl_snapshots'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(snapshots, "crawl_snapshots");

        let page_issues: String = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='page_issues'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(page_issues, "page_issues");

        for table in ["seo_category_scores", "seo_check_issues"] {
            let name: String = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(name, table);
        }

        for table in ["page_redirects", "seo_clusters", "seo_cluster_members"] {
            let name: String = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(name, table);
        }

        let redirect_from: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('crawled_pages') WHERE name='redirect_from_url'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(redirect_from, 1);

        let seo_priority: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('crawled_pages') WHERE name='seo_priority_fix_count'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(seo_priority, 1);

        let link_cols = column_names(&conn, "page_links");
        for expected in ["rel_tokens", "is_sponsored", "is_ugc", "is_internal"] {
            assert!(
                link_cols.contains(&expected.to_string()),
                "missing column {}",
                expected
            );
        }

        let perf_indexes: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type='index' AND name IN (
                     'idx_pages_project_seo_score',
                     'idx_pages_project_duplicate_group',
                     'idx_links_project_internal_from'
                 )",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(perf_indexes, 3, "perf indexes from 016 must exist");
    }

    #[test]
    fn test_page_keywords_backfill_from_keywords_json() {
        let conn = Connection::open_in_memory().unwrap();
        // Base v4 schema (before the content-analysis columns) so the keyword
        // columns are introduced by the real migrations, exactly like an
        // upgraded database.
        conn.execute_batch(
            "CREATE TABLE projects (id TEXT PRIMARY KEY, name TEXT, created_at TEXT, updated_at TEXT);
             INSERT INTO projects VALUES ('p1', 'P1', datetime('now'), datetime('now'));
             CREATE TABLE crawl_config (id TEXT PRIMARY KEY, project_id TEXT, seed_urls TEXT, max_pages INTEGER, created_at TEXT);
             INSERT INTO crawl_config VALUES ('cfg', 'p1', '[]', 10, datetime('now'));
             CREATE TABLE crawled_pages (
                 id TEXT PRIMARY KEY, config_id TEXT, project_id TEXT, url TEXT,
                 status_code INTEGER, title TEXT, meta_description TEXT, h1 TEXT, canonical TEXT,
                 size_bytes INTEGER, load_time_ms INTEGER, is_indexable INTEGER, depth INTEGER,
                 parent_url TEXT, crawl_timestamp TEXT, html_lang TEXT, hreflang_json TEXT,
                 semantic_issues_json TEXT, html_body TEXT, screenshot_png BLOB
             );
             INSERT INTO crawled_pages (id, config_id, project_id, url, crawl_timestamp)
             VALUES ('pg1', 'cfg', 'p1', 'https://x.com/a', datetime('now')),
                    ('pg2', 'cfg', 'p1', 'https://x.com/b', datetime('now'));
             CREATE TABLE page_links (
                 id INTEGER PRIMARY KEY AUTOINCREMENT, from_url TEXT NOT NULL, to_url TEXT NOT NULL,
                 config_id TEXT NOT NULL, project_id TEXT, link_type TEXT NOT NULL,
                 anchor_text TEXT, is_follow INTEGER
             );
             CREATE TABLE crawl_errors (
                 id INTEGER PRIMARY KEY AUTOINCREMENT, url TEXT NOT NULL, config_id TEXT NOT NULL,
                 project_id TEXT, error_type TEXT NOT NULL, error_message TEXT, timestamp TEXT NOT NULL
             );
             CREATE TABLE crawl_sessions (
                 id TEXT PRIMARY KEY, project_id TEXT NOT NULL, config_json TEXT NOT NULL,
                 status TEXT NOT NULL DEFAULT 'interrupted', pages_crawled INTEGER NOT NULL DEFAULT 0,
                 errors INTEGER NOT NULL DEFAULT 0, elapsed_secs INTEGER NOT NULL DEFAULT 0,
                 seed_urls TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
             );
             CREATE TABLE crawl_queue (
                 session_id TEXT NOT NULL REFERENCES crawl_sessions(id), url TEXT NOT NULL,
                 depth INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (session_id, url)
             );",
        )
        .unwrap();

        run_migrations(&conn).unwrap();

        // Seed keywords_json only after the migration run, so the backfill from
        // 016 is exercised on rows that already carry the column.
        conn.execute_batch(
            "UPDATE crawled_pages SET keywords_json = '[{\"keyword\":\"seo\",\"count\":3},{\"keyword\":\"crawl\",\"count\":1}]'
             WHERE id = 'pg1';
             UPDATE crawled_pages SET keywords_json = '[{\"keyword\":\"seo\",\"count\":2}]'
             WHERE id = 'pg2';",
        )
        .unwrap();

        // Re-run just the 016 backfill by applying the migration body directly.
        conn.execute_batch(include_str!("016_perf_indexes.sql"))
            .unwrap();

        let (seo, crawl): (i64, i64) = conn
            .query_row(
                "SELECT
                    SUM(CASE WHEN keyword='seo' THEN count ELSE 0 END),
                    SUM(CASE WHEN keyword='crawl' THEN count ELSE 0 END)
                 FROM page_keywords",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(seo, 5, "seo count aggregated across both pages");
        assert_eq!(crawl, 1);
        let pages: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT page_id) FROM page_keywords",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(pages, 2);
    }

    #[test]
    fn test_link_analysis_backfill_is_internal() {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute_batch(
            "CREATE TABLE page_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_url TEXT NOT NULL,
                to_url TEXT NOT NULL,
                config_id TEXT NOT NULL,
                project_id TEXT,
                link_type TEXT NOT NULL,
                anchor_text TEXT,
                is_follow INTEGER
            );
            INSERT INTO page_links (from_url, to_url, config_id, project_id, link_type, is_follow)
            VALUES
                ('https://x.com/a', 'https://x.com/b', 'cfg', 'p1', 'a', 1),
                ('https://x.com/a', 'https://y.com/ext', 'cfg', 'p1', 'a', 1),
                ('http://x.com:8080/a', 'https://x.com/b', 'cfg', 'p1', 'a', 1),
                ('https://x.com:443/a', 'https://x.com/a', 'cfg', 'p1', 'a', 1);",
        )
        .unwrap();

        run_migrations(&conn).unwrap();

        let internal: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM page_links WHERE is_internal = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let external: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM page_links WHERE is_internal = 0",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            internal, 3,
            "same host (with scheme/port variance) is internal"
        );
        assert_eq!(external, 1, "different host is external");
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
    }
}
