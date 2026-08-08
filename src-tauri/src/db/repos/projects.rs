use rusqlite::params;
use tracing::info;

use crate::error::AppError;
use crate::models::Project;

use super::CrawlRepo;

impl<'a> CrawlRepo<'a> {
    pub fn create_project(&self, name: &str) -> Result<Project, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, now, now],
        )?;

        info!("Created project: {} ({})", name, id);

        Ok(Project {
            id,
            name: name.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at, updated_at FROM projects ORDER BY created_at DESC",
        )?;

        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>, AppError> {
        let result = self.conn.query_row(
            "SELECT id, name, created_at, updated_at FROM projects WHERE id = ?1",
            params![id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        );

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn rename_project(&self, id: &str, name: &str) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now, id],
        )?;
        info!("Renamed project {} to {}", id, name);
        Ok(())
    }

    pub fn delete_project(&self, id: &str) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;

        // 1. Delete crawl_queue (child of crawl_sessions)
        tx.execute(
            "DELETE FROM crawl_queue WHERE session_id IN (SELECT id FROM crawl_sessions WHERE project_id = ?1)",
            params![id],
        )?;

        // 2. Delete crawl_sessions (child of projects)
        tx.execute(
            "DELETE FROM crawl_sessions WHERE project_id = ?1",
            params![id],
        )?;

        // 3. Delete page_links (FK: config_id -> crawl_config)
        tx.execute(
            "DELETE FROM page_links WHERE config_id IN (SELECT id FROM crawl_config WHERE project_id = ?1)",
            params![id],
        )?;

        // 4. Delete crawl_errors (FK: config_id -> crawl_config)
        tx.execute(
            "DELETE FROM crawl_errors WHERE config_id IN (SELECT id FROM crawl_config WHERE project_id = ?1)",
            params![id],
        )?;

        // 5. Delete crawled_pages (FK: config_id -> crawl_config)
        tx.execute(
            "DELETE FROM crawled_pages WHERE config_id IN (SELECT id FROM crawl_config WHERE project_id = ?1)",
            params![id],
        )?;

        // 5b. Delete page_issues (no FK; keyed by project_id)
        tx.execute("DELETE FROM page_issues WHERE project_id = ?1", params![id])?;

        // 6. Delete crawl_snapshot_data (child of crawl_snapshots)
        tx.execute(
            "DELETE FROM crawl_snapshot_data WHERE snapshot_id IN (SELECT id FROM crawl_snapshots WHERE project_id = ?1)",
            params![id],
        )?;

        // 7. Delete crawl_snapshots (FK: project_id -> projects)
        tx.execute(
            "DELETE FROM crawl_snapshots WHERE project_id = ?1",
            params![id],
        )?;

        // 8. Delete crawl_config (FK: project_id -> projects)
        tx.execute(
            "DELETE FROM crawl_config WHERE project_id = ?1",
            params![id],
        )?;

        // 9. Delete project
        tx.execute("DELETE FROM projects WHERE id = ?1", params![id])?;

        tx.commit()?;
        info!("Deleted project {}", id);

        self.invalidate_cache_for_project(id);

        Ok(())
    }

    pub fn get_project_stats(&self, project_id: &str) -> Result<serde_json::Value, AppError> {
        let pages_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawled_pages WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;

        let errors_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM crawl_errors WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;

        let links_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM page_links WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;

        Ok(serde_json::json!({
            "pages": pages_count,
            "errors": errors_count,
            "links": links_count,
        }))
    }
}
