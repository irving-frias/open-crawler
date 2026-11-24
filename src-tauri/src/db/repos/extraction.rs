//! Per-project custom extraction rules (`custom_extractions`, migration 020)
//! and the batch write of the values computed for each page.

use rusqlite::params;

use crate::crawler::extraction::{validate, CustomExtraction, ExtractionMode, SelectorType};
use crate::error::AppError;

use super::CrawlRepo;

impl<'a> CrawlRepo<'a> {
    /// The project's rules in display order (`position`, then name).
    pub fn get_custom_extractions(
        &self,
        project_id: &str,
    ) -> Result<Vec<CustomExtraction>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, selector_type, selector, mode, attribute, position
             FROM custom_extractions WHERE project_id = ?1
             ORDER BY position ASC, name ASC",
        )?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                let selector_type: String = row.get(2)?;
                let mode: String = row.get(4)?;
                Ok(CustomExtraction {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    selector_type: SelectorType::from_name(&selector_type)
                        .unwrap_or(SelectorType::Css),
                    selector: row.get(3)?,
                    mode: ExtractionMode::from_name(&mode).unwrap_or(ExtractionMode::Text),
                    attribute: row.get(5)?,
                    position: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Replaces the project's rules with the given list, in one transaction.
    /// Every rule is validated first, so a bad selector never wipes the
    /// stored rules. Rules without an id get a fresh one; `position` follows
    /// the list order. Returns the stored rules.
    pub fn save_custom_extractions(
        &self,
        project_id: &str,
        rules: &[CustomExtraction],
    ) -> Result<Vec<CustomExtraction>, AppError> {
        for rule in rules {
            validate(rule).map_err(AppError::InvalidConfig)?;
        }
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM custom_extractions WHERE project_id = ?1",
            params![project_id],
        )?;
        let mut stored = Vec::with_capacity(rules.len());
        for (position, rule) in rules.iter().enumerate() {
            let id = if rule.id.trim().is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                rule.id.clone()
            };
            tx.execute(
                "INSERT INTO custom_extractions
                 (id, project_id, name, selector_type, selector, mode, attribute, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id,
                    project_id,
                    rule.name,
                    rule.selector_type.as_str(),
                    rule.selector,
                    rule.mode.as_str(),
                    rule.attribute,
                    position as i64,
                ],
            )?;
            stored.push(CustomExtraction {
                id,
                position: position as i64,
                ..rule.clone()
            });
        }
        tx.commit()?;
        Ok(stored)
    }

    /// Writes computed extraction values (`{name: value}` JSON, or `None`)
    /// for a batch of pages, in a single transaction. Used by the re-audit
    /// and the on-demand recompute; the crawl path persists the values with
    /// the page row itself.
    pub fn update_custom_extractions_batch(
        &self,
        rows: &[(String, Option<String>)],
    ) -> Result<(), AppError> {
        if rows.is_empty() {
            return Ok(());
        }
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt =
                tx.prepare("UPDATE crawled_pages SET custom_extractions_json = ?2 WHERE id = ?1")?;
            for (page_id, json) in rows {
                stmt.execute(params![page_id, json])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repos::CrawlRepo;
    use crate::db::schema::run_migrations;
    use rusqlite::Connection;

    fn test_repo() -> CrawlRepo<'static> {
        let conn = Box::leak(Box::new(Connection::open_in_memory().unwrap()));
        run_migrations(conn).unwrap();
        conn.execute_batch(
            "INSERT INTO projects (id, name, created_at, updated_at)
             VALUES ('p1', 'P1', datetime('now'), datetime('now'));
             INSERT INTO crawl_config (id, project_id, seed_urls, created_at)
             VALUES ('cfg', 'p1', '[]', datetime('now'));",
        )
        .unwrap();
        CrawlRepo::new(conn, None)
    }

    fn css_rule(name: &str, selector: &str) -> CustomExtraction {
        CustomExtraction {
            id: String::new(),
            name: name.into(),
            selector_type: SelectorType::Css,
            selector: selector.into(),
            mode: ExtractionMode::Text,
            attribute: None,
            position: 0,
        }
    }

    #[test]
    fn save_and_get_round_trip_in_order() {
        let repo = test_repo();
        let stored = repo
            .save_custom_extractions(
                "p1",
                &[css_rule("price", ".price"), css_rule("title", "h1")],
            )
            .unwrap();
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].name, "price");
        assert_eq!(stored[0].position, 0);
        assert_eq!(stored[1].name, "title");
        assert!(!stored[0].id.is_empty());

        let back = repo.get_custom_extractions("p1").unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].name, "price");
        assert_eq!(back[1].selector, "h1");

        // Saving again replaces the whole list.
        repo.save_custom_extractions("p1", &[css_rule("only", "h2")])
            .unwrap();
        let back = repo.get_custom_extractions("p1").unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].name, "only");
    }

    #[test]
    fn save_rejects_invalid_rule_without_wiping() {
        let repo = test_repo();
        repo.save_custom_extractions("p1", &[css_rule("ok", "h1")])
            .unwrap();
        let bad = css_rule("bad", "div >>");
        let err = repo.save_custom_extractions("p1", &[bad]).unwrap_err();
        assert!(matches!(err, AppError::InvalidConfig(_)));
        // The stored rules survive the failed save.
        assert_eq!(repo.get_custom_extractions("p1").unwrap().len(), 1);
    }

    #[test]
    fn batch_update_writes_page_values() {
        let repo = test_repo();
        repo.conn
            .execute_batch(
                "INSERT INTO crawled_pages (id, config_id, project_id, url, crawl_timestamp)
                 VALUES ('pg1', 'cfg', 'p1', 'https://x.com/a', datetime('now')),
                        ('pg2', 'cfg', 'p1', 'https://x.com/b', datetime('now'));",
            )
            .unwrap();
        repo.update_custom_extractions_batch(&[
            ("pg1".to_string(), Some(r#"{"price":"$9.99"}"#.to_string())),
            ("pg2".to_string(), None),
        ])
        .unwrap();
        let json: Option<String> = repo
            .conn
            .query_row(
                "SELECT custom_extractions_json FROM crawled_pages WHERE id = 'pg1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(json.as_deref(), Some(r#"{"price":"$9.99"}"#));
    }
}
