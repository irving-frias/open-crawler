CREATE TABLE IF NOT EXISTS crawl_config (
    id TEXT PRIMARY KEY,
    seed_urls TEXT NOT NULL,
    max_pages INTEGER NOT NULL DEFAULT 1000,
    max_depth INTEGER NOT NULL DEFAULT 10,
    user_agent TEXT NOT NULL DEFAULT 'OpenCrawler/1.0',
    respect_robots INTEGER NOT NULL DEFAULT 1,
    render_js INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS crawled_pages (
    id TEXT PRIMARY KEY,
    config_id TEXT NOT NULL REFERENCES crawl_config(id),
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
    crawl_timestamp TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS page_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_url TEXT NOT NULL,
    to_url TEXT NOT NULL,
    config_id TEXT NOT NULL REFERENCES crawl_config(id),
    link_type TEXT NOT NULL,
    anchor_text TEXT,
    is_follow INTEGER
);

CREATE TABLE IF NOT EXISTS crawl_errors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    config_id TEXT NOT NULL REFERENCES crawl_config(id),
    error_type TEXT NOT NULL,
    error_message TEXT,
    timestamp TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pages_config ON crawled_pages(config_id);
CREATE INDEX IF NOT EXISTS idx_pages_url ON crawled_pages(url);
CREATE INDEX IF NOT EXISTS idx_links_config ON page_links(config_id);
CREATE INDEX IF NOT EXISTS idx_errors_config ON crawl_errors(config_id);
