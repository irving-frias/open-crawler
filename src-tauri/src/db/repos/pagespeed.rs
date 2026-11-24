use rusqlite::params;
use rusqlite::OptionalExtension;

use crate::error::AppError;
use crate::pagespeed::PageSpeedData;

use super::CrawlRepo;

impl<'a> CrawlRepo<'a> {
    pub fn get_pagespeed(&self, page_id: &str) -> Result<(Option<f64>, Option<String>), AppError> {
        let row = self
            .conn
            .query_row(
                "SELECT pagespeed_score, pagespeed_json FROM crawled_pages WHERE id = ?1",
                params![page_id],
                |row| {
                    Ok((
                        row.get::<_, Option<f64>>(0)?,
                        row.get::<_, Option<String>>(1)?,
                    ))
                },
            )
            .optional()?;
        Ok(row.unwrap_or((None, None)))
    }

    /// Returns the cached PageSpeed result for `page_id` only if it was
    /// fetched within the last `ttl_hours` hours. Returns `None` when there
    /// is no stored result, it is older than the TTL, or the stored JSON no
    /// longer parses — in all three cases the caller must re-fetch.
    ///
    /// Freshness is evaluated in SQLite (`datetime('now', ...)`), so the
    /// comparison uses the same clock that stamped `pagespeed_fetched_at`.
    pub fn get_cached_pagespeed(
        &self,
        page_id: &str,
        ttl_hours: i64,
    ) -> Result<Option<PageSpeedData>, AppError> {
        let max_age = format!("-{ttl_hours} hours");
        let json: Option<String> = self
            .conn
            .query_row(
                "SELECT pagespeed_json FROM crawled_pages
                 WHERE id = ?1 AND pagespeed_fetched_at >= datetime('now', ?2)",
                params![page_id, max_age],
                |row| row.get(0),
            )
            .optional()?;
        Ok(json.and_then(|json| serde_json::from_str::<PageSpeedData>(&json).ok()))
    }

    pub fn update_pagespeed(
        &self,
        page_id: &str,
        score: Option<f64>,
        json: Option<&str>,
    ) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE crawled_pages
             SET pagespeed_score = ?1, pagespeed_json = ?2,
                 pagespeed_fetched_at = datetime('now')
             WHERE id = ?3",
            params![score, json, page_id],
        )?;
        Ok(())
    }
}
