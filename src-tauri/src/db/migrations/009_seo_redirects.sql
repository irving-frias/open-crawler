-- HTTP redirects captured during crawl. The crawler follows redirects
-- manually so the hop-by-hop chain can be persisted here and reported by the
-- SEO Insights tab (redirects, chains and broken redirect targets).

ALTER TABLE crawled_pages ADD COLUMN redirect_from_url TEXT;

CREATE TABLE IF NOT EXISTS page_redirects (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  TEXT NOT NULL,
    page_id     TEXT NOT NULL,
    hop_index   INTEGER NOT NULL,
    from_url    TEXT NOT NULL,
    to_url      TEXT NOT NULL,
    status_code INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_page_redirects_project ON page_redirects (project_id);
CREATE INDEX IF NOT EXISTS idx_page_redirects_page ON page_redirects (page_id);
