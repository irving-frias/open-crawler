use rusqlite::{params, Row};

use crate::error::AppError;
use crate::models::AlertSettings;

use super::CrawlRepo;

impl<'a> CrawlRepo<'a> {
    fn row_to_alert_settings(row: &Row<'_>) -> rusqlite::Result<AlertSettings> {
        let channels_json: String = row.get(2)?;
        let channels: Vec<String> = serde_json::from_str(&channels_json).unwrap_or_default();
        Ok(AlertSettings {
            project_id: row.get(0)?,
            enabled: row.get::<_, i64>(1)? != 0,
            channels,
            min_severity: row.get(3)?,
            quiet_start: row.get(4)?,
            quiet_end: row.get(5)?,
            last_alerted_snapshot_id: row.get(6)?,
        })
    }

    const ALERT_COLUMNS: &'static str = "project_id, enabled, channels_json, min_severity, quiet_start, quiet_end, last_alerted_snapshot_id";

    /// Alert settings for a project. A project without a stored row gets the
    /// defaults (alerts on, desktop channel, `warning` threshold).
    pub fn get_alert_settings(&self, project_id: &str) -> Result<AlertSettings, AppError> {
        let result = self.conn.query_row(
            &format!(
                "SELECT {} FROM project_alert_settings WHERE project_id = ?1",
                Self::ALERT_COLUMNS
            ),
            params![project_id],
            Self::row_to_alert_settings,
        );
        match result {
            Ok(settings) => Ok(settings.normalized()),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AlertSettings::default_for(project_id)),
            Err(e) => Err(e.into()),
        }
    }

    /// Inserts or replaces the project's alert settings (normalized first).
    /// Preserves `last_alerted_snapshot_id` when the caller leaves it empty,
    /// so saving settings from the UI never resets the dedup marker.
    pub fn save_alert_settings(&self, settings: &AlertSettings) -> Result<AlertSettings, AppError> {
        let settings = settings.clone().normalized();
        let channels_json = serde_json::to_string(&settings.channels)?;
        let previous = self.get_alert_settings(&settings.project_id)?;
        let last_alerted = settings
            .last_alerted_snapshot_id
            .clone()
            .or(previous.last_alerted_snapshot_id);
        self.conn.execute(
            "INSERT INTO project_alert_settings (project_id, enabled, channels_json, min_severity, quiet_start, quiet_end, last_alerted_snapshot_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(project_id) DO UPDATE SET
               enabled = excluded.enabled,
               channels_json = excluded.channels_json,
               min_severity = excluded.min_severity,
               quiet_start = excluded.quiet_start,
               quiet_end = excluded.quiet_end,
               last_alerted_snapshot_id = excluded.last_alerted_snapshot_id",
            params![
                settings.project_id,
                settings.enabled as i64,
                channels_json,
                settings.min_severity,
                settings.quiet_start,
                settings.quiet_end,
                last_alerted,
            ],
        )?;
        self.get_alert_settings(&settings.project_id)
    }

    /// Records the newest snapshot an alert was sent for, so the same
    /// errors are not notified again on the next scheduled run.
    pub fn set_last_alerted_snapshot(
        &self,
        project_id: &str,
        snapshot_id: &str,
    ) -> Result<(), AppError> {
        // Upsert a defaults row when the project has no settings yet, so the
        // marker survives even if the user never opened the alert settings.
        let mut settings = self.get_alert_settings(project_id)?;
        settings.last_alerted_snapshot_id = Some(snapshot_id.to_string());
        self.save_alert_settings(&settings)?;
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
    fn test_alert_settings_defaults_for_unknown_project() {
        let repo = repo();
        let settings = repo.get_alert_settings("p1").unwrap();
        assert!(settings.enabled);
        assert_eq!(settings.channels, vec!["desktop".to_string()]);
        assert_eq!(settings.min_severity, "warning");
        assert_eq!(settings.quiet_start, None);
        assert_eq!(settings.last_alerted_snapshot_id, None);
    }

    #[test]
    fn test_alert_settings_roundtrip_and_dedup_marker() {
        let repo = repo();
        let settings = AlertSettings {
            project_id: "p1".to_string(),
            enabled: true,
            channels: vec!["desktop".to_string(), "telegram".to_string()],
            min_severity: "error".to_string(),
            quiet_start: Some("22:00".to_string()),
            quiet_end: Some("07:00".to_string()),
            last_alerted_snapshot_id: None,
        };
        let saved = repo.save_alert_settings(&settings).unwrap();
        assert_eq!(saved.channels.len(), 2);
        assert_eq!(saved.min_severity, "error");
        assert_eq!(saved.quiet_start.as_deref(), Some("22:00"));

        // Saving again without the marker preserves the stored one.
        repo.set_last_alerted_snapshot("p1", "snap-1").unwrap();
        let reloaded = repo.get_alert_settings("p1").unwrap();
        assert_eq!(reloaded.last_alerted_snapshot_id.as_deref(), Some("snap-1"));
        let saved_again = repo
            .save_alert_settings(&AlertSettings {
                last_alerted_snapshot_id: None,
                ..reloaded.clone()
            })
            .unwrap();
        assert_eq!(
            saved_again.last_alerted_snapshot_id.as_deref(),
            Some("snap-1")
        );
    }

    #[test]
    fn test_alert_settings_normalizes_invalid_values() {
        let repo = repo();
        let settings = AlertSettings {
            project_id: "p1".to_string(),
            enabled: true,
            channels: vec!["smoke-signal".to_string()],
            min_severity: "critical".to_string(),
            quiet_start: Some("25:99".to_string()),
            quiet_end: Some("07:00".to_string()),
            last_alerted_snapshot_id: None,
        };
        let saved = repo.save_alert_settings(&settings).unwrap();
        assert_eq!(saved.channels, vec!["desktop".to_string()]);
        assert_eq!(saved.min_severity, "warning");
        assert_eq!(saved.quiet_start, None);
        assert_eq!(saved.quiet_end.as_deref(), Some("07:00"));
    }
}
