use serde::{Deserialize, Serialize};

/// A cron-scheduled crawl job bound to a project. `config_json` holds the full
/// `CrawlConfig` (JSON) used to run the crawl when the job fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub project_id: String,
    pub cron_expression: String,
    pub config_json: String,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledJobRequest {
    pub project_id: String,
    pub cron_expression: String,
    pub config_json: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateScheduledJobRequest {
    pub id: String,
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
}
