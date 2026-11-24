-- Alert settings for scheduled crawls (Fase 3). One row per project; the
-- defaults (alerts on, desktop channel, `warning` threshold, no quiet hours)
-- apply when no row exists, so existing projects need no backfill.
CREATE TABLE IF NOT EXISTS project_alert_settings (
    project_id TEXT PRIMARY KEY REFERENCES projects(id),
    enabled INTEGER NOT NULL DEFAULT 1,
    channels_json TEXT NOT NULL DEFAULT '["desktop"]',
    min_severity TEXT NOT NULL DEFAULT 'warning',
    quiet_start TEXT,
    quiet_end TEXT,
    last_alerted_snapshot_id TEXT
);

-- Failed SEO checks per snapshotted page, as a JSON object
-- `{ check_id: severity }` taken from the normalized `seo_check_issues`
-- table at snapshot time. Lets the alert diff find newly failing checks
-- without re-reading (or re-downloading) any page. NULL for snapshots taken
-- before this migration, treated as "no failed checks".
ALTER TABLE crawl_snapshot_data ADD COLUMN failed_checks_json TEXT;
