ALTER TABLE crawled_pages ADD COLUMN seo_score REAL;
ALTER TABLE crawled_pages ADD COLUMN seo_audit_json TEXT;
ALTER TABLE crawl_snapshots ADD COLUMN avg_seo_score REAL;
ALTER TABLE crawl_snapshot_data ADD COLUMN seo_score REAL;
