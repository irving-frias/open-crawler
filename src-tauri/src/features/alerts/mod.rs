//! Alerts for scheduled crawls: desktop notifications (via the
//! `tauri-plugin-notification` plugin) and Telegram messages (via a plain
//! `reqwest` POST to the Bot API, no extra crate).
//!
//! The flow is: after a scheduled crawl finishes and its snapshot is
//! created, [`service::check_and_notify`] diffs the two newest snapshots
//! with [`CrawlRepo::compute_new_errors`] and notifies only when new errors
//! appeared. The last alerted snapshot is recorded so the same errors are
//! never notified twice.

pub mod commands;
pub mod notifier;
pub mod service;
