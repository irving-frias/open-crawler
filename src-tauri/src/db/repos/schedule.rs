use rusqlite::{params, Row};

use crate::error::AppError;
use crate::models::{CreateScheduledJobRequest, ScheduledJob, UpdateScheduledJobRequest};

use super::CrawlRepo;

/// RFC3339 "now" truncated to whole seconds. `next_run` is stored in this exact
/// fixed-width format so the `next_run <= ?` SQL comparison is lexicographically
/// correct (no variable-length fractional seconds).
pub(crate) fn now_rfc3339_secs() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Computes the next cron occurrence strictly after `from` (RFC3339).
pub(crate) fn compute_next_run(
    cron_expression: &str,
    from: &str,
) -> Result<Option<String>, AppError> {
    let schedule: cron::Schedule = cron_expression
        .parse()
        .map_err(|e| AppError::Crawl(format!("Invalid cron expression '{cron_expression}': {e}")))?;
    let from_dt = chrono::DateTime::parse_from_rfc3339(from)
        .map_err(|e| AppError::Crawl(e.to_string()))?
        .with_timezone(&chrono::Utc);
    Ok(schedule
        .after(&from_dt)
        .next()
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)))
}

impl<'a> CrawlRepo<'a> {
    fn row_to_job(row: &Row<'_>) -> rusqlite::Result<ScheduledJob> {
        Ok(ScheduledJob {
            id: row.get(0)?,
            project_id: row.get(1)?,
            cron_expression: row.get(2)?,
            config_json: row.get(3)?,
            enabled: row.get::<_, i64>(4)? != 0,
            last_run: row.get(5)?,
            next_run: row.get(6)?,
            created_at: row.get(7)?,
        })
    }

    const JOB_COLUMNS: &'static str = "id, project_id, cron_expression, config_json, enabled, last_run, next_run, created_at";

    pub fn list_scheduled_jobs(&self) -> Result<Vec<ScheduledJob>, AppError> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM scheduled_jobs ORDER BY created_at ASC",
            Self::JOB_COLUMNS
        ))?;
        let jobs = stmt
            .query_map([], Self::row_to_job)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(jobs)
    }

    pub fn get_scheduled_job(&self, id: &str) -> Result<Option<ScheduledJob>, AppError> {
        let result = self.conn.query_row(
            &format!("SELECT {} FROM scheduled_jobs WHERE id = ?1", Self::JOB_COLUMNS),
            params![id],
            Self::row_to_job,
        );
        match result {
            Ok(job) => Ok(Some(job)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_scheduled_job(
        &self,
        req: &CreateScheduledJobRequest,
    ) -> Result<ScheduledJob, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_rfc3339_secs();
        let next_run = compute_next_run(&req.cron_expression, &now)?;
        self.conn.execute(
            "INSERT INTO scheduled_jobs (id, project_id, cron_expression, config_json, enabled, last_run, next_run, created_at)
             VALUES (?1, ?2, ?3, ?4, 1, NULL, ?5, ?6)",
            params![id, req.project_id, req.cron_expression, req.config_json, next_run, now],
        )?;
        self.get_scheduled_job(&id)?
            .ok_or_else(|| AppError::Crawl("Failed to create scheduled job".to_string()))
    }

    pub fn update_scheduled_job(
        &self,
        req: &UpdateScheduledJobRequest,
    ) -> Result<Option<ScheduledJob>, AppError> {
        let Some(job) = self.get_scheduled_job(&req.id)? else {
            return Ok(None);
        };
        let cron_expression = req
            .cron_expression
            .clone()
            .unwrap_or_else(|| job.cron_expression.clone());
        let enabled = req.enabled.unwrap_or(job.enabled);
        let next_run = if req.cron_expression.is_some() {
            compute_next_run(&cron_expression, &now_rfc3339_secs())?
        } else {
            job.next_run
        };
        self.conn.execute(
            "UPDATE scheduled_jobs SET cron_expression = ?1, enabled = ?2, next_run = ?3 WHERE id = ?4",
            params![cron_expression, enabled as i64, next_run, req.id],
        )?;
        self.get_scheduled_job(&req.id)
    }

    pub fn delete_scheduled_job(&self, id: &str) -> Result<(), AppError> {
        self.conn
            .execute("DELETE FROM scheduled_jobs WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Jobs that are enabled and whose `next_run` is null (first run, defensive)
    /// or not yet in the future.
    pub fn get_due_scheduled_jobs(&self, now: &str) -> Result<Vec<ScheduledJob>, AppError> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM scheduled_jobs WHERE enabled = 1 AND (next_run IS NULL OR next_run <= ?1)",
            Self::JOB_COLUMNS
        ))?;
        let jobs = stmt
            .query_map(params![now], Self::row_to_job)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(jobs)
    }

    pub fn set_job_last_run(&self, id: &str, ts: &str) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE scheduled_jobs SET last_run = ?1 WHERE id = ?2",
            params![ts, id],
        )?;
        Ok(())
    }

    pub fn set_job_next_run(&self, id: &str, next: &str) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE scheduled_jobs SET next_run = ?1 WHERE id = ?2",
            params![next, id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;
    use rusqlite::Connection;

    fn repo() -> CrawlRepo<'static> {
        let conn = Box::leak(Box::new(Connection::open_in_memory().unwrap()));
        run_migrations(conn).unwrap();
        conn.execute_batch(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('p1', 'P1', datetime('now'), datetime('now'));",
        )
        .unwrap();
        CrawlRepo::new(conn, None)
    }

    #[test]
    fn test_compute_next_run_valid() {
        let next = compute_next_run("0 0 * * * *", "2026-08-06T10:00:00Z")
            .unwrap()
            .unwrap();
        assert_eq!(next, "2026-08-06T11:00:00Z");
    }

    #[test]
    fn test_compute_next_run_invalid() {
        assert!(compute_next_run("not a cron", "2026-08-06T10:00:00Z").is_err());
    }

    #[test]
    fn test_scheduled_jobs_crud_and_due() {
        let repo = repo();
        let req = CreateScheduledJobRequest {
            project_id: "p1".to_string(),
            cron_expression: "*/5 * * * * *".to_string(),
            config_json: "{}".to_string(),
        };
        let job = repo.create_scheduled_job(&req).unwrap();
        assert!(job.next_run.is_some());
        assert_eq!(repo.list_scheduled_jobs().unwrap().len(), 1);

        // A future next_run must not be due.
        let due = repo.get_due_scheduled_jobs("2020-01-01T00:00:00Z").unwrap();
        assert!(due.is_empty());

        // Force next_run into the past -> due.
        repo.set_job_next_run(&job.id, "2020-01-01T00:00:00Z")
            .unwrap();
        let due = repo.get_due_scheduled_jobs("2026-08-06T00:00:00Z").unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, job.id);

        // Disabling clears it from the due set.
        repo.update_scheduled_job(&UpdateScheduledJobRequest {
            id: job.id.clone(),
            cron_expression: None,
            enabled: Some(false),
        })
        .unwrap();
        let due = repo.get_due_scheduled_jobs("2026-08-06T00:00:00Z").unwrap();
        assert!(due.is_empty());

        repo.delete_scheduled_job(&job.id).unwrap();
        assert!(repo.list_scheduled_jobs().unwrap().is_empty());
    }
}
