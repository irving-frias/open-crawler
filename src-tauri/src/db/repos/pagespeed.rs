use rusqlite::OptionalExtension;
use rusqlite::params;

use crate::error::AppError;

use super::CrawlRepo;

impl<'a> CrawlRepo<'a> {
    pub fn get_pagespeed(&self, page_id: &str) -> Result<(Option<f64>, Option<String>), AppError> {
        let row = self
            .conn
            .query_row(
                "SELECT pagespeed_score, pagespeed_json FROM crawled_pages WHERE id = ?1",
                params![page_id],
                |row| Ok((row.get::<_, Option<f64>>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()?;
        Ok(row.unwrap_or((None, None)))
    }

    pub fn update_pagespeed(
        &self,
        page_id: &str,
        score: Option<f64>,
        json: Option<&str>,
    ) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE crawled_pages SET pagespeed_score = ?1, pagespeed_json = ?2 WHERE id = ?3",
            params![score, json, page_id],
        )?;
        Ok(())
    }
}
