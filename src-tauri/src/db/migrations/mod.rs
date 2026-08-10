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
    ("012_add_graph_indexes", include_str!("012_add_graph_indexes.sql")),
    (
        "013_add_pages_project_url_index",
        include_str!("013_add_pages_project_url_index.sql"),
    ),
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
