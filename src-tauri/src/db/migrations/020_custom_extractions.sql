-- Custom extraction rules defined by the user, one row per rule per project.
-- A rule names a value to pull out of every crawled page (CSS selector, XPath
-- subset, or regex) and how to read it (text, attribute, or match count).
-- Values are stored per page as a JSON object {rule name: string|number|null}
-- in crawled_pages.custom_extractions_json, computed at crawl time and
-- recomputed on re-audit / on demand, so the definitions can change without
-- re-crawling.

CREATE TABLE IF NOT EXISTS custom_extractions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    selector_type TEXT NOT NULL,
    selector TEXT NOT NULL,
    mode TEXT NOT NULL,
    attribute TEXT,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (project_id, name)
);

CREATE INDEX IF NOT EXISTS idx_custom_extractions_project
    ON custom_extractions(project_id);

ALTER TABLE crawled_pages ADD COLUMN custom_extractions_json TEXT;
