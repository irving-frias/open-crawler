use serde::{Deserialize, Serialize};

/// Channels an alert can be delivered through.
pub const ALERT_CHANNEL_DESKTOP: &str = "desktop";
pub const ALERT_CHANNEL_TELEGRAM: &str = "telegram";

/// Severity levels for failed SEO checks, most severe first.
pub const ALERT_SEVERITIES: [&str; 3] = ["error", "warning", "info"];

/// Per-project alert configuration for scheduled crawls. Stored in
/// `project_alert_settings`; a project without a row uses
/// [`AlertSettings::default_for`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSettings {
    pub project_id: String,
    pub enabled: bool,
    /// Delivery channels, any combination of `"desktop"` and `"telegram"`.
    pub channels: Vec<String>,
    /// Minimum severity of a newly failing SEO check that triggers an
    /// alert (`"error"` | `"warning"` | `"info"`). New 404s and new broken
    /// links always alert: they are errors by nature.
    pub min_severity: String,
    /// Quiet hours as local `"HH:MM"`; alerts are suppressed while the
    /// local time is inside the window. `None` on either end disables it.
    /// A window where start > end crosses midnight (e.g. 22:00-07:00).
    pub quiet_start: Option<String>,
    pub quiet_end: Option<String>,
    /// Id of the newest snapshot an alert was already sent for, so the
    /// same errors are never notified twice.
    pub last_alerted_snapshot_id: Option<String>,
}

impl AlertSettings {
    pub fn default_for(project_id: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
            enabled: true,
            channels: vec![ALERT_CHANNEL_DESKTOP.to_string()],
            min_severity: "warning".to_string(),
            quiet_start: None,
            quiet_end: None,
            last_alerted_snapshot_id: None,
        }
    }

    /// Drops unknown channels / severities and malformed quiet hours,
    /// falling back to the defaults for the invalid parts.
    pub fn normalized(mut self) -> Self {
        self.channels = self
            .channels
            .into_iter()
            .filter(|c| c == ALERT_CHANNEL_DESKTOP || c == ALERT_CHANNEL_TELEGRAM)
            .collect::<Vec<_>>();
        self.channels.dedup();
        if self.channels.is_empty() {
            self.channels = vec![ALERT_CHANNEL_DESKTOP.to_string()];
        }
        if !ALERT_SEVERITIES.contains(&self.min_severity.as_str()) {
            self.min_severity = "warning".to_string();
        }
        if self
            .quiet_start
            .as_deref()
            .is_some_and(|s| parse_hhmm(s).is_none())
        {
            self.quiet_start = None;
        }
        if self
            .quiet_end
            .as_deref()
            .is_some_and(|s| parse_hhmm(s).is_none())
        {
            self.quiet_end = None;
        }
        self
    }
}

/// Parses a `"HH:MM"` 24-hour time. Returns `None` for anything else.
pub fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let (h, m) = s.split_once(':')?;
    if h.len() != 2 || m.len() != 2 {
        return None;
    }
    let h: u32 = h.parse().ok()?;
    let m: u32 = m.parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some((h, m))
}

/// Severity rank, most severe first: error 0, warning 1, anything else 2.
pub fn severity_rank(severity: &str) -> u8 {
    match severity {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    }
}

/// True when a failed check of `severity` meets the `min_severity` threshold.
pub fn severity_meets_threshold(severity: &str, min_severity: &str) -> bool {
    severity_rank(severity) <= severity_rank(min_severity)
}

/// One newly failing SEO check on one page, as found by the snapshot diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewFailedCheck {
    pub url: String,
    pub check_id: String,
    pub severity: String,
}

/// Errors that appeared between two snapshots of the same project: pages
/// that newly return an error status, SEO checks that newly fail, and
/// internal links that newly point at an error page. `*_total` counts every
/// match; the `Vec`s hold only a display sample (see `ALERT_SAMPLE_LIMIT`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewErrors {
    pub new_error_urls: Vec<String>,
    pub new_error_url_total: u32,
    pub new_failed_checks: Vec<NewFailedCheck>,
    pub new_failed_check_total: u32,
    pub new_broken_links: Vec<String>,
    pub new_broken_link_total: u32,
}

impl NewErrors {
    /// Total number of new errors across all three categories.
    pub fn total_count(&self) -> u32 {
        self.new_error_url_total + self.new_failed_check_total + self.new_broken_link_total
    }

    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }
}

/// How many sample items per category the diff returns (totals are exact).
pub const ALERT_SAMPLE_LIMIT: u32 = 20;
