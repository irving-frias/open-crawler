CREATE TABLE IF NOT EXISTS crawl_snapshots (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    config_id TEXT NOT NULL,
    snapshot_time TEXT NOT NULL,
    total_pages INTEGER NOT NULL DEFAULT 0,
    indexed_pages INTEGER NOT NULL DEFAULT 0,
    broken_pages INTEGER NOT NULL DEFAULT 0,
    avg_load_ms REAL,
    avg_size_bytes REAL,
    avg_readability REAL,
    status_counts_json TEXT
);

CREATE TABLE IF NOT EXISTS crawl_snapshot_data (
    snapshot_id TEXT NOT NULL REFERENCES crawl_snapshots(id),
    page_id TEXT NOT NULL,
    url TEXT NOT NULL,
    status_code INTEGER,
    title TEXT,
    meta_description TEXT,
    size_bytes INTEGER,
    load_time_ms INTEGER,
    is_indexable INTEGER,
    readability_score REAL,
    PRIMARY KEY (snapshot_id, page_id)
);

CREATE INDEX IF NOT EXISTS idx_snapshots_project ON crawl_snapshots(project_id, snapshot_time);
CREATE INDEX IF NOT EXISTS idx_snapshot_data_url ON crawl_snapshot_data(snapshot_id, url);
