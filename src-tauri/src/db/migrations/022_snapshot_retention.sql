-- Per-project snapshot retention (T15): how many snapshots to keep per
-- project. The prune in create_crawl_snapshot reads it; the default (12)
-- matches the previous hardcoded limit.
ALTER TABLE projects ADD COLUMN snapshot_retention INTEGER NOT NULL DEFAULT 12;
