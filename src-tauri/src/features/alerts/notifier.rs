use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::models::NewErrors;

/// Test message sent by the "send test notification" button in Settings.
pub const TEST_MESSAGE_ES: &str =
    "OpenCrawler: prueba de alertas. Si ves este mensaje, las alertas funcionan.";
pub const TEST_MESSAGE_EN: &str =
    "OpenCrawler: alert test. If you see this message, alerts are working.";

/// Builds the alert text for a set of new errors. `spanish` follows the
/// app language setting; the first line is the summary, then one bullet per
/// non-empty category with up to 5 sample URLs and a "and N more" tail.
pub fn format_alert_message(project_name: &str, errors: &NewErrors, spanish: bool) -> String {
    let total = errors.total_count();
    let mut lines = vec![if spanish {
        format!("OpenCrawler: {total} errores nuevos en {project_name}")
    } else {
        format!("OpenCrawler: {total} new errors in {project_name}")
    }];
    if errors.new_error_url_total > 0 {
        lines.push(format_bullet(
            spanish,
            if spanish {
                "páginas con error nuevas"
            } else {
                "new error pages"
            },
            errors.new_error_url_total,
            &errors.new_error_urls,
        ));
    }
    if errors.new_failed_check_total > 0 {
        let samples: Vec<String> = errors
            .new_failed_checks
            .iter()
            .map(|c| format!("{} ({})", c.check_id, c.url))
            .collect();
        lines.push(format_bullet(
            spanish,
            if spanish {
                "checks SEO fallidos nuevos"
            } else {
                "newly failing SEO checks"
            },
            errors.new_failed_check_total,
            &samples,
        ));
    }
    if errors.new_broken_link_total > 0 {
        lines.push(format_bullet(
            spanish,
            if spanish {
                "enlaces rotos nuevos"
            } else {
                "new broken links"
            },
            errors.new_broken_link_total,
            &errors.new_broken_links,
        ));
    }
    lines.join("\n")
}

fn format_bullet(spanish: bool, label: &str, total: u32, samples: &[String]) -> String {
    let shown: Vec<&str> = samples.iter().take(5).map(|s| s.as_str()).collect();
    let mut text = format!("• {total} {label}: {}", shown.join(", "));
    if total as usize > shown.len() {
        text.push_str(&if spanish {
            format!(" y {} más", total as usize - shown.len())
        } else {
            format!(" and {} more", total as usize - shown.len())
        });
    }
    text
}

/// Shows a desktop notification. The title carries the summary; the body
/// carries the per-category bullets.
pub fn send_desktop(app: &AppHandle, title: &str, body: &str) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}

/// Sends a plain-text message through the Telegram Bot API.
/// `token` is never included in the returned error (reqwest errors can
/// embed the request URL, which carries the token).
pub async fn send_telegram(token: &str, chat_id: &str, text: &str) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.without_url().to_string())?;
    let resp = client
        .post(format!("https://api.telegram.org/bot{token}/sendMessage"))
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "disable_web_page_preview": true,
        }))
        .send()
        .await
        .map_err(|e| e.without_url().to_string())?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.map_err(|e| e.without_url().to_string())?;
    if body.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        Ok(())
    } else {
        let detail = body
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown Telegram error");
        Err(format!("Telegram API error ({status}): {detail}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NewFailedCheck;

    fn sample_errors() -> NewErrors {
        NewErrors {
            new_error_urls: vec!["https://x.com/a".to_string(), "https://x.com/e".to_string()],
            new_error_url_total: 2,
            new_failed_checks: vec![NewFailedCheck {
                url: "https://x.com/b".to_string(),
                check_id: "missing_title".to_string(),
                severity: "warning".to_string(),
            }],
            new_failed_check_total: 1,
            new_broken_links: vec!["https://x.com/a".to_string()],
            new_broken_link_total: 1,
        }
    }

    #[test]
    fn test_format_alert_message_spanish() {
        let msg = format_alert_message("Mi Sitio", &sample_errors(), true);
        assert!(msg.contains("4 errores nuevos en Mi Sitio"));
        assert!(msg.contains("2 páginas con error nuevas"));
        assert!(msg.contains("missing_title (https://x.com/b)"));
        assert!(msg.contains("1 enlaces rotos nuevos"));
    }

    #[test]
    fn test_format_alert_message_english_and_more_tail() {
        let mut errors = sample_errors();
        errors.new_error_url_total = 8;
        let msg = format_alert_message("My Site", &errors, false);
        assert!(msg.contains("8 new error pages"));
        assert!(msg.contains("and 6 more"));
    }

    #[test]
    fn test_format_alert_message_empty_errors() {
        let msg = format_alert_message("S", &NewErrors::default(), true);
        assert_eq!(msg, "OpenCrawler: 0 errores nuevos en S");
    }
}
