-- Persist skipped SEO categories (score NULL) in the normalized table.
--
-- `save_seo_normalized` used to skip categories with a NULL score (the
-- column was NOT NULL), but the XLSX "SEO Audit" sheet writes one row per
-- category *including* skipped ones, and the old `count_seo_rows` counted
-- them via `json_each` over `seo_audit_json`. Counting from the normalized
-- tables therefore undercounted by one per skipped category per page
-- (every real page audit skips at least the site-level "links" category).
-- Allowing NULL scores makes the normalized table a faithful copy of the
-- audit's category list, so counts derived from it match the JSON counts
-- exactly. `AVG(score)` ignores NULLs and `get_seo_overview` filters them
-- out of the per-category page counts, so site aggregation is unchanged.
CREATE TABLE seo_category_scores_new (
    page_id    TEXT NOT NULL,
    project_id TEXT NOT NULL,
    category   TEXT NOT NULL,
    score      REAL,
    PRIMARY KEY (page_id, category)
);
INSERT INTO seo_category_scores_new (page_id, project_id, category, score)
    SELECT page_id, project_id, category, score FROM seo_category_scores;
DROP TABLE seo_category_scores;
ALTER TABLE seo_category_scores_new RENAME TO seo_category_scores;
CREATE INDEX IF NOT EXISTS idx_seo_category_scores_project
    ON seo_category_scores (project_id, category);

-- Backfill skipped categories from audits stored before this migration.
-- Scored categories already exist (from 011), so only the NULL-score ones
-- are missing; OR IGNORE also guards against a duplicated category entry
-- in a single audit JSON.
INSERT OR IGNORE INTO seo_category_scores (page_id, project_id, category, score)
SELECT p.id, p.project_id,
       json_extract(c.value, '$.category'),
       NULL
FROM crawled_pages p, json_each(p.seo_audit_json, '$.categories') c
WHERE p.seo_audit_json IS NOT NULL AND json_valid(p.seo_audit_json)
  AND json_extract(c.value, '$.score') IS NULL
  AND json_extract(c.value, '$.category') IS NOT NULL;
