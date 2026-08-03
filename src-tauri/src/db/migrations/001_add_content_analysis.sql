ALTER TABLE crawled_pages ADD COLUMN readability_score REAL;
ALTER TABLE crawled_pages ADD COLUMN content_hash TEXT;
ALTER TABLE crawled_pages ADD COLUMN duplicate_group_id INTEGER;
ALTER TABLE crawled_pages ADD COLUMN keywords_json TEXT;
