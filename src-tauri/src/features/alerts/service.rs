use std::sync::Arc;

use chrono::Timelike;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::AppError;
use crate::features::with_repo_arc;
use crate::models::alerts::parse_hhmm;
use crate::models::{
    AlertSettings, CrawlSnapshot, NewErrors, ALERT_CHANNEL_DESKTOP, ALERT_CHANNEL_TELEGRAM,
};
use crate::secrets;
use crate::AppState;

use super::notifier;

/// What happened when a scheduled crawl finished and alerts were evaluated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertOutcome {
    /// Alerts are disabled for this project.
    Disabled,
    /// Only one snapshot exists: this run is the baseline, nothing to diff.
    NoPreviousSnapshot,
    /// An alert was already sent for the newest snapshot.
    AlreadyAlerted,
    /// Inside the configured quiet hours; the alert is suppressed for this
    /// run (it is not queued: the next run diffs a newer snapshot pair).
    QuietHours,
    /// The diff found no new errors.
    NoNewErrors,
    /// At least one channel delivered the alert.
    Alerted { desktop: bool, telegram: bool },
    /// New errors exist but no channel could deliver (e.g. Telegram is the
    /// only channel and the bot is not configured). Nothing was recorded,
    /// so the failure is visible in the logs and the test button.
    NoChannelAvailable,
}

/// Result of the "send test notification" button in Settings.
#[derive(Debug, Clone, Serialize)]
pub struct TestAlertResult {
    pub desktop_sent: bool,
    pub telegram_sent: bool,
    pub telegram_configured: bool,
}

struct AlertContext {
    settings: AlertSettings,
    project_name: String,
    snapshots: Vec<CrawlSnapshot>,
    notifications_enabled: bool,
    spanish: bool,
    telegram_token: Option<String>,
    telegram_chat_id: Option<String>,
}

async fn load_context(
    state: &Arc<RwLock<AppState>>,
    project_id: &str,
) -> Result<AlertContext, AppError> {
    let project_id = project_id.to_string();
    with_repo_arc(state, move |repo| {
        let settings = repo.get_alert_settings(&project_id)?;
        let project_name = repo
            .get_project(&project_id)?
            .map(|p| p.name)
            .unwrap_or_else(|| project_id.clone());
        let snapshots = repo.list_crawl_snapshots(&project_id)?;
        let notifications_enabled = repo
            .get_setting("notifications_enabled")?
            .map(|v| v != "false")
            .unwrap_or(true);
        let spanish = repo
            .get_setting("language")?
            .map(|v| v == "es")
            .unwrap_or(false);
        let telegram_token =
            secrets::get(repo, "telegram_bot_token")?.filter(|v| !v.trim().is_empty());
        let telegram_chat_id =
            secrets::get(repo, "telegram_chat_id")?.filter(|v| !v.trim().is_empty());
        Ok(AlertContext {
            settings,
            project_name,
            snapshots,
            notifications_enabled,
            spanish,
            telegram_token,
            telegram_chat_id,
        })
    })
    .await
}

/// True when local time `now` (hour, minute) falls inside the quiet window.
/// A window where start > end crosses midnight (e.g. 22:00-07:00). Missing
/// or malformed bounds disable quiet hours.
pub fn is_quiet_now(quiet_start: Option<&str>, quiet_end: Option<&str>, now: (u32, u32)) -> bool {
    let (Some(start), Some(end)) = (
        quiet_start.and_then(parse_hhmm),
        quiet_end.and_then(parse_hhmm),
    ) else {
        return false;
    };
    let now_mins = now.0 * 60 + now.1;
    let start_mins = start.0 * 60 + start.1;
    let end_mins = end.0 * 60 + end.1;
    if start_mins <= end_mins {
        now_mins >= start_mins && now_mins < end_mins
    } else {
        now_mins >= start_mins || now_mins < end_mins
    }
}

fn is_quiet_now_local(quiet_start: Option<&str>, quiet_end: Option<&str>) -> bool {
    let now = chrono::Local::now();
    is_quiet_now(quiet_start, quiet_end, (now.hour(), now.minute()))
}

/// Diffs the two newest snapshots after a scheduled crawl and notifies
/// only when new errors appeared. Never fails the crawl: every failure
/// mode is logged and reported as an [`AlertOutcome`].
pub async fn check_and_notify(
    app: &AppHandle,
    state: &Arc<RwLock<AppState>>,
    project_id: &str,
    snapshot: &CrawlSnapshot,
) -> Result<AlertOutcome, AppError> {
    let ctx = load_context(state, project_id).await?;
    if !ctx.settings.enabled {
        return Ok(AlertOutcome::Disabled);
    }
    if ctx.snapshots.len() < 2 {
        return Ok(AlertOutcome::NoPreviousSnapshot);
    }
    let newest = &ctx.snapshots[0];
    let previous = &ctx.snapshots[1];
    if newest.id != snapshot.id {
        info!(
            "Alert check for project {}: a newer snapshot {} exists, diffing that one",
            project_id, newest.id
        );
    }
    if ctx
        .settings
        .last_alerted_snapshot_id
        .as_deref()
        .is_some_and(|id| id == newest.id)
    {
        return Ok(AlertOutcome::AlreadyAlerted);
    }
    if is_quiet_now_local(
        ctx.settings.quiet_start.as_deref(),
        ctx.settings.quiet_end.as_deref(),
    ) {
        return Ok(AlertOutcome::QuietHours);
    }

    let newest_id = newest.id.clone();
    let previous_id = previous.id.clone();
    let min_severity = ctx.settings.min_severity.clone();
    let project_id_owned = project_id.to_string();
    let errors: NewErrors = with_repo_arc(state, move |repo| {
        repo.compute_new_errors(&project_id_owned, &previous_id, &newest_id, &min_severity)
    })
    .await?;
    if errors.is_empty() {
        return Ok(AlertOutcome::NoNewErrors);
    }

    let message = notifier::format_alert_message(&ctx.project_name, &errors, ctx.spanish);
    let mut lines = message.split('\n');
    let title = lines.next().unwrap_or("OpenCrawler").to_string();
    let body = lines.collect::<Vec<_>>().join("\n");

    let mut desktop = false;
    let mut telegram = false;
    if ctx
        .settings
        .channels
        .iter()
        .any(|c| c == ALERT_CHANNEL_DESKTOP)
        && ctx.notifications_enabled
    {
        match notifier::send_desktop(app, &title, &body) {
            Ok(()) => desktop = true,
            Err(e) => warn!("Desktop alert failed for project {}: {}", project_id, e),
        }
    }
    if ctx
        .settings
        .channels
        .iter()
        .any(|c| c == ALERT_CHANNEL_TELEGRAM)
    {
        match (&ctx.telegram_token, &ctx.telegram_chat_id) {
            (Some(token), Some(chat_id)) => {
                match notifier::send_telegram(token, chat_id, &message).await {
                    Ok(()) => telegram = true,
                    Err(e) => warn!("Telegram alert failed for project {}: {}", project_id, e),
                }
            }
            _ => warn!(
                "Telegram alert enabled for project {} but bot token or chat id is not configured",
                project_id
            ),
        }
    }

    if desktop || telegram {
        let newest_id = newest.id.clone();
        let project_id_owned = project_id.to_string();
        with_repo_arc(state, move |repo| {
            repo.set_last_alerted_snapshot(&project_id_owned, &newest_id)
        })
        .await?;
        let _ = app.emit(
            "scheduled-alert",
            serde_json::json!({
                "project_id": project_id,
                "snapshot_id": newest.id,
                "new_errors": errors.total_count(),
                "new_error_urls": errors.new_error_url_total,
                "new_failed_checks": errors.new_failed_check_total,
                "new_broken_links": errors.new_broken_link_total,
            }),
        );
        info!(
            "Alert sent for project {} ({} new errors, desktop={}, telegram={})",
            project_id,
            errors.total_count(),
            desktop,
            telegram
        );
        Ok(AlertOutcome::Alerted { desktop, telegram })
    } else {
        warn!(
            "Alert for project {} has {} new errors but no channel delivered it",
            project_id,
            errors.total_count()
        );
        Ok(AlertOutcome::NoChannelAvailable)
    }
}

/// Sends a test notification on every configured channel. When
/// `project_id` is given, only that project's channels are tried;
/// otherwise desktop + Telegram (if configured) are both tried.
pub async fn send_test_alert(
    app: &AppHandle,
    state: &Arc<RwLock<AppState>>,
    project_id: Option<&str>,
) -> Result<TestAlertResult, AppError> {
    let ctx = match project_id {
        Some(id) => Some(load_context(state, id).await?),
        None => None,
    };
    let (notifications_enabled, spanish, token, chat_id, try_desktop, try_telegram) = match &ctx {
        Some(ctx) => (
            ctx.notifications_enabled,
            ctx.spanish,
            ctx.telegram_token.clone(),
            ctx.telegram_chat_id.clone(),
            ctx.settings
                .channels
                .iter()
                .any(|c| c == ALERT_CHANNEL_DESKTOP),
            ctx.settings
                .channels
                .iter()
                .any(|c| c == ALERT_CHANNEL_TELEGRAM),
        ),
        None => {
            let (token, chat_id, notifications_enabled, spanish) = {
                let state = state.clone();
                with_repo_arc(&state, move |repo| {
                    Ok((
                        secrets::get(repo, "telegram_bot_token")?.filter(|v| !v.trim().is_empty()),
                        secrets::get(repo, "telegram_chat_id")?.filter(|v| !v.trim().is_empty()),
                        repo.get_setting("notifications_enabled")?
                            .map(|v| v != "false")
                            .unwrap_or(true),
                        repo.get_setting("language")?
                            .map(|v| v == "es")
                            .unwrap_or(false),
                    ))
                })
                .await?
            };
            (notifications_enabled, spanish, token, chat_id, true, true)
        }
    };

    let text = if spanish {
        notifier::TEST_MESSAGE_ES
    } else {
        notifier::TEST_MESSAGE_EN
    };

    let mut desktop_sent = false;
    if try_desktop && notifications_enabled {
        match notifier::send_desktop(app, "OpenCrawler", text) {
            Ok(()) => desktop_sent = true,
            Err(e) => warn!("Test desktop notification failed: {}", e),
        }
    }
    let telegram_configured = token.is_some() && chat_id.is_some();
    let mut telegram_sent = false;
    if try_telegram && telegram_configured {
        match notifier::send_telegram(token.as_deref().unwrap(), chat_id.as_deref().unwrap(), text)
            .await
        {
            Ok(()) => telegram_sent = true,
            Err(e) => warn!("Test Telegram message failed: {}", e),
        }
    }
    Ok(TestAlertResult {
        desktop_sent,
        telegram_sent,
        telegram_configured,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quiet_hours_daytime_window() {
        assert!(is_quiet_now(Some("22:00"), Some("07:00"), (23, 30)));
        assert!(is_quiet_now(Some("22:00"), Some("07:00"), (3, 0)));
        assert!(!is_quiet_now(Some("22:00"), Some("07:00"), (12, 0)));
        assert!(!is_quiet_now(Some("22:00"), Some("07:00"), (7, 0)));
        assert!(is_quiet_now(Some("22:00"), Some("07:00"), (22, 0)));
    }

    #[test]
    fn test_quiet_hours_same_day_window() {
        assert!(is_quiet_now(Some("09:00"), Some("17:00"), (12, 0)));
        assert!(!is_quiet_now(Some("09:00"), Some("17:00"), (8, 59)));
        assert!(!is_quiet_now(Some("09:00"), Some("17:00"), (17, 0)));
    }

    #[test]
    fn test_quiet_hours_missing_or_malformed_bounds() {
        assert!(!is_quiet_now(None, Some("07:00"), (3, 0)));
        assert!(!is_quiet_now(Some("22:00"), None, (3, 0)));
        assert!(!is_quiet_now(Some("xx"), Some("07:00"), (3, 0)));
        assert!(!is_quiet_now(None, None, (3, 0)));
    }
}
