-- Performance indexes for the results list / dashboard sort paths and the
-- duplicate-title filter. Ordering by seo_score, load_time_ms and size_bytes
-- are the three sortable columns of the results table; duplicate_group_id
-- backs the "duplicates" facet (GROUP BY duplicate_group_id HAVING COUNT(*) > 1).
-- A (project_id, title) index lets the duplicate_title filter turn the
-- per-row correlated COUNT into an indexed range scan.

CREATE INDEX IF NOT EXISTS idx_pages_project_seo_score
    ON crawled_pages(project_id, seo_score);
CREATE INDEX IF NOT EXISTS idx_pages_project_load_ms
    ON crawled_pages(project_id, load_time_ms);
CREATE INDEX IF NOT EXISTS idx_pages_project_size_bytes
    ON crawled_pages(project_id, size_bytes);
CREATE INDEX IF NOT EXISTS idx_pages_project_duplicate_group
    ON crawled_pages(project_id, duplicate_group_id);
CREATE INDEX IF NOT EXISTS idx_pages_project_title
    ON crawled_pages(project_id, title);

-- The orphan/dead-end reports look up page_links by (project_id, is_internal,
-- to_url) and (project_id, is_internal, from_url). A composite index covers
-- both scans without touching the full table.
CREATE INDEX IF NOT EXISTS idx_links_project_internal_from
    ON page_links(project_id, is_internal, from_url);

-- Materialized per-page keyword counts so the project-wide keyword report can
-- be answered with a single GROUP BY instead of loading and re-parsing every
-- page's keywords_json over IPC. Populated alongside save_results_batch.
CREATE TABLE IF NOT EXISTS page_keywords (
    project_id TEXT NOT NULL REFERENCES projects(id),
    page_id TEXT NOT NULL,
    keyword TEXT NOT NULL,
    count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (project_id, page_id, keyword)
);
CREATE INDEX IF NOT EXISTS idx_page_keywords_project
    ON page_keywords(project_id, keyword);

-- Backfill the materialized table from the keywords_json already stored on
-- every page (a one-time cost paid on upgrade, mirroring the 014 backfill).
INSERT OR REPLACE INTO page_keywords (project_id, page_id, keyword, count)
SELECT cp.project_id, cp.id,
       json_extract(value, '$.keyword'),
       json_extract(value, '$.count')
FROM crawled_pages cp,
     json_each(CASE WHEN cp.keywords_json IS NULL THEN '[]' ELSE cp.keywords_json END)
WHERE json_extract(value, '$.keyword') IS NOT NULL;
