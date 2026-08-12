-- Link analysis metadata. Previously page_links only stored a single
-- `is_follow` boolean derived from `rel`. We now persist the raw rel tokens
-- plus per-token flags and the internal/external classification so the
-- link-analysis reports (anchor text, sponsored/ugc, orphan pages, external
-- domain distribution) can be answered with plain SQL.

ALTER TABLE page_links ADD COLUMN rel_tokens TEXT;
ALTER TABLE page_links ADD COLUMN is_sponsored INTEGER NOT NULL DEFAULT 0;
ALTER TABLE page_links ADD COLUMN is_ugc INTEGER NOT NULL DEFAULT 0;
ALTER TABLE page_links ADD COLUMN is_internal INTEGER NOT NULL DEFAULT 0;

-- Backfill internal/external by comparing the host portion (scheme-less
-- authority, port stripped, lowercased) of both endpoints.
UPDATE page_links AS pl
SET is_internal = CASE WHEN lower(f.host) = lower(t.host) THEN 1 ELSE 0 END
FROM (
    SELECT id,
           CASE WHEN instr(auth, ':') = 0 THEN auth
                ELSE substr(auth, 1, instr(auth, ':') - 1) END AS host
    FROM (
        SELECT id,
               CASE WHEN instr(from_url, '://') = 0 THEN from_url
                    ELSE substr(from_url, instr(from_url, '://') + 3,
                         CASE WHEN instr(substr(from_url, instr(from_url, '://') + 3), '/') = 0
                              THEN length(substr(from_url, instr(from_url, '://') + 3))
                              ELSE instr(substr(from_url, instr(from_url, '://') + 3), '/') - 1 END)
               END AS auth
        FROM page_links
    )
) AS f
JOIN (
    SELECT id,
           CASE WHEN instr(auth, ':') = 0 THEN auth
                ELSE substr(auth, 1, instr(auth, ':') - 1) END AS host
    FROM (
        SELECT id,
               CASE WHEN instr(to_url, '://') = 0 THEN to_url
                    ELSE substr(to_url, instr(to_url, '://') + 3,
                         CASE WHEN instr(substr(to_url, instr(to_url, '://') + 3), '/') = 0
                              THEN length(substr(to_url, instr(to_url, '://') + 3))
                              ELSE instr(substr(to_url, instr(to_url, '://') + 3), '/') - 1 END)
               END AS auth
        FROM page_links
    )
) AS t ON t.id = f.id
WHERE pl.id = f.id;

CREATE INDEX IF NOT EXISTS idx_links_project_internal
    ON page_links (project_id, is_internal, to_url);
