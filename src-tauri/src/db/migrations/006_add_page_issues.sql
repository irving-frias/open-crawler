-- Normalized table for per-page semantic issues.
--
-- Previously every filter / aggregation over issues was done with
-- `json_each(crawled_pages.semantic_issues_json)`, which requires a full table
-- scan and re-parsing of the JSON payload per row. Storing each issue as a row
-- lets `get_results`, the dashboard counts and the export use indexed JOINs /
-- simple WHERE clauses instead, and keeps warning/info issues queryable.

CREATE TABLE IF NOT EXISTS page_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    page_id TEXT NOT NULL,
    issue_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT,
    element TEXT,
    css_selector TEXT,
    xpath TEXT,
    position INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_page_issues_project_severity ON page_issues(project_id, severity);
CREATE INDEX IF NOT EXISTS idx_page_issues_project_type ON page_issues(project_id, issue_type);
CREATE INDEX IF NOT EXISTS idx_page_issues_page ON page_issues(page_id);

-- Backfill existing crawled_pages so legacy databases get the normalized rows
-- without requiring a re-crawl. Migrations run exactly once (tracked in
-- schema_migrations), so this cannot double-insert.
INSERT INTO page_issues (project_id, page_id, issue_type, severity, message, element, css_selector, xpath, position)
SELECT
    cp.project_id,
    cp.id,
    je.value ->> '$.issue_type',
    je.value ->> '$.severity',
    je.value ->> '$.message',
    je.value ->> '$.element',
    je.value ->> '$.css_selector',
    je.value ->> '$.xpath',
    CAST(je.key AS INTEGER)
FROM crawled_pages cp, json_each(cp.semantic_issues_json) je
WHERE cp.semantic_issues_json IS NOT NULL
  AND je.value ->> '$.issue_type' IS NOT NULL
  AND je.value ->> '$.severity' IS NOT NULL;
