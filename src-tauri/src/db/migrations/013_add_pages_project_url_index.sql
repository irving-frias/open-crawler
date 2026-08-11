-- Composite index for queries that filter crawled_pages by project then
-- group by url (site tree, results, duplicate detection). A single
-- (project_id, url) index serves all of them, avoiding per-row url lookups
-- over the whole project.
CREATE INDEX IF NOT EXISTS idx_pages_project_url
    ON crawled_pages(project_id, url);
