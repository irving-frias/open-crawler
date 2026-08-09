-- SEO clustering: near-duplicate / duplicate content groups detected after a
-- crawl. `seo_clusters` holds one row per cluster (canonical picked by the
-- engine), `seo_cluster_members` the pages that belong to each cluster.

CREATE TABLE IF NOT EXISTS seo_clusters (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  TEXT NOT NULL,
    cluster_key TEXT NOT NULL,
    canonical   TEXT NOT NULL,
    member_count INTEGER NOT NULL,
    similarity  REAL NOT NULL,
    issue       TEXT NOT NULL DEFAULT 'duplicate',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_seo_clusters_project ON seo_clusters (project_id);

CREATE TABLE IF NOT EXISTS seo_cluster_members (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id  INTEGER NOT NULL,
    page_id     TEXT NOT NULL,
    url         TEXT NOT NULL,
    FOREIGN KEY (cluster_id) REFERENCES seo_clusters (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_seo_cluster_members_cluster ON seo_cluster_members (cluster_id);
CREATE INDEX IF NOT EXISTS idx_seo_cluster_members_page ON seo_cluster_members (page_id);
