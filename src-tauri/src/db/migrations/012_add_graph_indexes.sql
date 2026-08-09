-- Speed up graph/tree queries that group page_links by endpoint:
-- the degree CTEs (GROUP BY to_url / from_url) and the internal-edge
-- extraction (from_url IN crawled AND to_url IN crawled) all scan this table.
CREATE INDEX IF NOT EXISTS idx_links_project_from
    ON page_links(project_id, from_url);
CREATE INDEX IF NOT EXISTS idx_links_project_to
    ON page_links(project_id, to_url);
