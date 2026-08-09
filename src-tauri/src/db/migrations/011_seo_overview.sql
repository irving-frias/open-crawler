-- Normalized SEO data for fast project-level aggregation. Category scores and
-- failing checks are written once per page (at audit time) so `get_seo_overview`
-- can be answered with SQL GROUP BY queries instead of loading every
-- `seo_audit_json` blob and re-parsing it with serde on each call.
CREATE TABLE IF NOT EXISTS seo_category_scores (
    page_id   TEXT NOT NULL,
    project_id TEXT NOT NULL,
    category  TEXT NOT NULL,
    score     REAL NOT NULL,
    PRIMARY KEY (page_id, category)
);
CREATE INDEX IF NOT EXISTS idx_seo_category_scores_project
    ON seo_category_scores (project_id, category);

CREATE TABLE IF NOT EXISTS seo_check_issues (
    page_id     TEXT NOT NULL,
    project_id  TEXT NOT NULL,
    category    TEXT NOT NULL,
    severity    TEXT NOT NULL,
    check_id    TEXT NOT NULL,
    message     TEXT NOT NULL,
    guidance    TEXT NOT NULL,
    evidence    TEXT,
    examples_json TEXT,
    PRIMARY KEY (page_id, check_id)
);
CREATE INDEX IF NOT EXISTS idx_seo_check_issues_project
    ON seo_check_issues (project_id, category, severity);

ALTER TABLE crawled_pages ADD COLUMN seo_priority_fix_count INTEGER NOT NULL DEFAULT 0;

-- Backfill from audits already stored by previous engine versions.
INSERT INTO seo_category_scores (page_id, project_id, category, score)
SELECT p.id, p.project_id,
       json_extract(c.value, '$.category'),
       json_extract(c.value, '$.score')
FROM crawled_pages p, json_each(p.seo_audit_json, '$.categories') c
WHERE p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json);

INSERT INTO seo_check_issues
    (page_id, project_id, category, severity, check_id, message, guidance, evidence, examples_json)
SELECT p.id, p.project_id,
       json_extract(c.value, '$.category'),
       json_extract(c.value, '$.severity'),
       json_extract(c.value, '$.id'),
       json_extract(c.value, '$.message'),
       json_extract(c.value, '$.guidance'),
       json_extract(c.value, '$.evidence'),
       json_extract(c.value, '$.examples')
FROM crawled_pages p, json_each(p.seo_audit_json, '$.checks') c
WHERE p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json)
  AND json_extract(c.value, '$.passed') = 0;

UPDATE crawled_pages SET seo_priority_fix_count = (
    SELECT COUNT(*) FROM json_each(crawled_pages.seo_audit_json, '$.priority_fixes')
)
WHERE seo_audit_json IS NOT NULL AND json_valid(seo_audit_json);
