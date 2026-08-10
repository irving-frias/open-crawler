-- Composite index for the site-graph node query. The top_urls CTE filters
-- crawled_pages by project then groups by url, and the final SELECT joins
-- crawled_pages back on (url, project_id) — a single (project_id, url) index
-- serves both, avoiding per-row url lookups over the whole project.
CREATE INDEX IF NOT EXISTS idx_pages_project_url
    ON crawled_pages(project_id, url);
