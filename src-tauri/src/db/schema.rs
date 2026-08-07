use rusqlite::Connection;
use tracing::info;

use crate::error::AppError;

pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    info!("Running database migrations");

    // Helper: check if a column exists in a table
    fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        columns.contains(&column.to_string())
    }

    // Check if projects table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='projects'",
            [],
            |row| row.get::<_, String>(0),
        )
        .is_ok();

    if !table_exists {
        info!("Running v4 migration: full schema with sessions and queue");

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS crawl_config (
                id TEXT PRIMARY KEY,
                project_id TEXT REFERENCES projects(id),
                seed_urls TEXT NOT NULL,
                max_pages INTEGER NOT NULL DEFAULT 1000,
                max_depth INTEGER NOT NULL DEFAULT 10,
                user_agent TEXT NOT NULL DEFAULT 'OpenCrawler/1.0',
                respect_robots INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS crawled_pages (
                id TEXT PRIMARY KEY,
                config_id TEXT NOT NULL REFERENCES crawl_config(id),
                project_id TEXT REFERENCES projects(id),
                url TEXT NOT NULL,
                status_code INTEGER,
                title TEXT,
                meta_description TEXT,
                h1 TEXT,
                canonical TEXT,
                size_bytes INTEGER,
                load_time_ms INTEGER,
                is_indexable INTEGER,
                depth INTEGER,
                parent_url TEXT,
                crawl_timestamp TEXT NOT NULL,
                html_lang TEXT,
                hreflang_json TEXT,
                semantic_issues_json TEXT,
                html_body TEXT,
                screenshot_png BLOB
            );

            CREATE TABLE IF NOT EXISTS page_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_url TEXT NOT NULL,
                to_url TEXT NOT NULL,
                config_id TEXT NOT NULL REFERENCES crawl_config(id),
                project_id TEXT REFERENCES projects(id),
                link_type TEXT NOT NULL,
                anchor_text TEXT,
                is_follow INTEGER
            );

            CREATE TABLE IF NOT EXISTS crawl_errors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                config_id TEXT NOT NULL REFERENCES crawl_config(id),
                project_id TEXT REFERENCES projects(id),
                error_type TEXT NOT NULL,
                error_message TEXT,
                timestamp TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS crawl_sessions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id),
                config_json TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'interrupted',
                pages_crawled INTEGER NOT NULL DEFAULT 0,
                errors INTEGER NOT NULL DEFAULT 0,
                elapsed_secs INTEGER NOT NULL DEFAULT 0,
                seed_urls TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS crawl_queue (
                session_id TEXT NOT NULL REFERENCES crawl_sessions(id),
                url TEXT NOT NULL,
                depth INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (session_id, url)
            );

            CREATE INDEX IF NOT EXISTS idx_config_project ON crawl_config(project_id);
            CREATE INDEX IF NOT EXISTS idx_pages_project ON crawled_pages(project_id);
            CREATE INDEX IF NOT EXISTS idx_pages_project_ts ON crawled_pages(project_id, crawl_timestamp);
            CREATE INDEX IF NOT EXISTS idx_links_project ON page_links(project_id);
            CREATE INDEX IF NOT EXISTS idx_links_from_url ON page_links(from_url);
            CREATE INDEX IF NOT EXISTS idx_errors_project ON crawl_errors(project_id);
            CREATE INDEX IF NOT EXISTS idx_errors_url ON crawl_errors(url);
            CREATE INDEX IF NOT EXISTS idx_pages_config ON crawled_pages(config_id);
            CREATE INDEX IF NOT EXISTS idx_pages_url ON crawled_pages(url);
            CREATE INDEX IF NOT EXISTS idx_sessions_project ON crawl_sessions(project_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_status ON crawl_sessions(status);
            CREATE INDEX IF NOT EXISTS idx_queue_session ON crawl_queue(session_id);
            CREATE INDEX IF NOT EXISTS idx_queue_url ON crawl_queue(url);",
        )?;

        conn.execute_batch(
            "INSERT OR IGNORE INTO projects (id, name, created_at, updated_at)
             VALUES ('default', 'Default', datetime('now'), datetime('now'));
            UPDATE crawl_config SET project_id = 'default' WHERE project_id IS NULL;
            UPDATE crawled_pages SET project_id = 'default' WHERE project_id IS NULL;
            UPDATE page_links SET project_id = 'default' WHERE project_id IS NULL;
            UPDATE crawl_errors SET project_id = 'default' WHERE project_id IS NULL;",
        )?;

        info!("v4 migration completed");
    } else {
        info!("Projects table already exists, checking columns...");

        // Ensure project_id column exists on crawl_config
        if !column_exists(conn, "crawl_config", "project_id") {
            info!("Adding project_id column to crawl_config");
            conn.execute_batch(
                "ALTER TABLE crawl_config ADD COLUMN project_id TEXT REFERENCES projects(id)",
            )?;
        }
        if !column_exists(conn, "crawled_pages", "project_id") {
            info!("Adding project_id column to crawled_pages");
            conn.execute_batch(
                "ALTER TABLE crawled_pages ADD COLUMN project_id TEXT REFERENCES projects(id)",
            )?;
        }
        if !column_exists(conn, "page_links", "project_id") {
            info!("Adding project_id column to page_links");
            conn.execute_batch(
                "ALTER TABLE page_links ADD COLUMN project_id TEXT REFERENCES projects(id)",
            )?;
        }
        if !column_exists(conn, "crawl_errors", "project_id") {
            info!("Adding project_id column to crawl_errors");
            conn.execute_batch(
                "ALTER TABLE crawl_errors ADD COLUMN project_id TEXT REFERENCES projects(id)",
            )?;
        }
        // hreflang columns
        if !column_exists(conn, "crawled_pages", "html_lang") {
            info!("Adding html_lang column to crawled_pages");
            conn.execute_batch("ALTER TABLE crawled_pages ADD COLUMN html_lang TEXT")?;
        }
        if !column_exists(conn, "crawled_pages", "hreflang_json") {
            info!("Adding hreflang_json column to crawled_pages");
            conn.execute_batch("ALTER TABLE crawled_pages ADD COLUMN hreflang_json TEXT")?;
        }
        // semantic issues column
        if !column_exists(conn, "crawled_pages", "semantic_issues_json") {
            info!("Adding semantic_issues_json column to crawled_pages");
            conn.execute_batch("ALTER TABLE crawled_pages ADD COLUMN semantic_issues_json TEXT")?;
        }
        // v5: html_body for DOM tree analysis
        if !column_exists(conn, "crawled_pages", "html_body") {
            info!("Adding html_body column to crawled_pages");
            conn.execute_batch("ALTER TABLE crawled_pages ADD COLUMN html_body TEXT")?;
        }
        // v6: screenshot PNG for visual preview
        if !column_exists(conn, "crawled_pages", "screenshot_png") {
            info!("Adding screenshot_png column to crawled_pages");
            conn.execute_batch("ALTER TABLE crawled_pages ADD COLUMN screenshot_png BLOB")?;
        }

        // v4: crawl_sessions and crawl_queue tables
        let sessions_exists: bool = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='crawl_sessions'",
                [],
                |row| row.get::<_, String>(0),
            )
            .is_ok();

        if !sessions_exists {
            info!("Creating crawl_sessions and crawl_queue tables");
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS crawl_sessions (
                    id TEXT PRIMARY KEY,
                    project_id TEXT NOT NULL REFERENCES projects(id),
                    config_json TEXT NOT NULL,
                    status TEXT NOT NULL DEFAULT 'interrupted',
                    pages_crawled INTEGER NOT NULL DEFAULT 0,
                    errors INTEGER NOT NULL DEFAULT 0,
                    elapsed_secs INTEGER NOT NULL DEFAULT 0,
                    seed_urls TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS crawl_queue (
                    session_id TEXT NOT NULL REFERENCES crawl_sessions(id),
                    url TEXT NOT NULL,
                    depth INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (session_id, url)
                );

                CREATE INDEX IF NOT EXISTS idx_sessions_project ON crawl_sessions(project_id);
                CREATE INDEX IF NOT EXISTS idx_sessions_status ON crawl_sessions(status);
                CREATE INDEX IF NOT EXISTS idx_queue_session ON crawl_queue(session_id);",
            )?;
        }
    }

    // Migration v7: settings table for user preferences
    let settings_exists: bool = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='settings'",
            [],
            |row| row.get::<_, String>(0),
        )
        .is_ok();

    if !settings_exists {
        info!("Creating settings table");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Default settings
            INSERT OR IGNORE INTO settings (key, value) VALUES
                ('language', 'en'),
                ('theme', 'system'),
                ('ui_style', 'classic'),
                ('page_size', '50'),
                ('max_depth', '10'),
                ('respect_robots', 'true'),
                ('check_sitemap', 'true'),
                ('check_semantics', 'true'),
                ('max_crawl_time', '3600');",
        )?;
    }

    // Migration v8: missing indexes for query performance
    let idx_pages_project_ts: bool = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_pages_project_ts'",
            [],
            |row| row.get::<_, String>(0),
        )
        .is_ok();

    if !idx_pages_project_ts {
        info!("Creating missing indexes for query performance");
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_pages_project_ts ON crawled_pages(project_id, crawl_timestamp);
             CREATE INDEX IF NOT EXISTS idx_links_from_url ON page_links(from_url);
             CREATE INDEX IF NOT EXISTS idx_errors_url ON crawl_errors(url);
             CREATE INDEX IF NOT EXISTS idx_queue_url ON crawl_queue(url);",
        )?;
    }

    // Migration v9: index on (project_id, title) for the duplicate-title filter
    let idx_pages_title: bool = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_pages_title'",
            [],
            |row| row.get::<_, String>(0),
        )
        .is_ok();

    if !idx_pages_title {
        info!("Creating title index for duplicate-title detection");
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_pages_title ON crawled_pages(project_id, title)",
        )?;
    }

    // Migration v10: indexes on FK config_id columns to speed up delete_project
    let idx_pages_config: bool = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_pages_config'",
            [],
            |row| row.get::<_, String>(0),
        )
        .is_ok();

    if !idx_pages_config {
        info!("Creating config_id indexes for fast project deletion");
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_pages_config ON crawled_pages(config_id);
             CREATE INDEX IF NOT EXISTS idx_links_config ON page_links(config_id);
             CREATE INDEX IF NOT EXISTS idx_errors_config ON crawl_errors(config_id);
             CREATE INDEX IF NOT EXISTS idx_sessions_project ON crawl_sessions(project_id);
             CREATE INDEX IF NOT EXISTS idx_queue_session ON crawl_queue(session_id);",
        )?;
    }

    // Migration v11: unique (project_id, url) on crawled_pages. Re-crawls used
    // to accumulate one row per URL per session (INSERT OR REPLACE keyed by id),
    // which produced duplicate URLs and broke keyed each blocks (site tree,
    // duplicates, ...). Deduplicate keeping the newest row, then enforce it.
    let idx_pages_project_url: bool = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_pages_project_url'",
            [],
            |row| row.get::<_, String>(0),
        )
        .is_ok();

    if !idx_pages_project_url {
        info!("Deduplicating crawled_pages rows and adding unique (project_id, url) index");
        conn.execute_batch(
            "DELETE FROM crawled_pages
             WHERE id NOT IN (
                 SELECT id FROM (
                     SELECT id,
                            ROW_NUMBER() OVER (PARTITION BY project_id, url ORDER BY crawl_timestamp DESC) AS rn
                     FROM crawled_pages
                 )
                 WHERE rn = 1
             );
             CREATE UNIQUE INDEX IF NOT EXISTS idx_pages_project_url ON crawled_pages(project_id, url);",
        )?;
    }

    // Migration v12: scan_type column on crawl_config for local vs web scans
    if !column_exists(conn, "crawl_config", "scan_type") {
        info!("Adding scan_type column to crawl_config");
        conn.execute_batch(
            "ALTER TABLE crawl_config ADD COLUMN scan_type TEXT DEFAULT 'web';",
        )?;
    }

    // Ensure default project exists
    conn.execute(
        "INSERT OR IGNORE INTO projects (id, name, created_at, updated_at)
         VALUES ('default', 'Default', datetime('now'), datetime('now'))",
        [],
    )?;

    crate::db::migrations::run(conn)?;

    info!("Database migrations completed");
    Ok(())
}
