-- Persist the final response headers of every crawled page so the security /
-- compliance audit checks can be recomputed offline (from stored HTML) and so
-- re-audits keep the stored snapshot faithful to what was actually served.

ALTER TABLE crawled_pages ADD COLUMN response_headers_json TEXT;
