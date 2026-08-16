pub mod commands;
pub mod crawler;
pub mod db;
pub mod error;
pub mod features;
pub mod models;
pub mod nesting_table;
pub mod pagespeed;
pub mod secrets;
pub mod seo;
pub mod transfer;

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::info;

use crate::models::CrawlProgress;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct ResultsCacheKey {
    pub project_id: String,
    pub page: u32,
    pub page_size: u32,
    pub semantic_issue_type: Option<String>,
    pub search: Option<String>,
    pub status_filter: Vec<u32>,
    pub severity_filter: Vec<String>,
    pub domain_filter: Option<String>,
    pub depth_filter: Option<u32>,
    pub missing_title: bool,
    pub duplicate_title: bool,
    pub noindex_only: bool,
    pub is_404: bool,
}

pub struct CrawlState {
    pub cancellation: tokio_util::sync::CancellationToken,
    pub progress: CrawlProgress,
}

/// Tracks a running project-wide SEO re-audit so it can be polled and stopped
/// independently per project (parallel windows).
pub struct SeoAuditState {
    pub cancellation: tokio_util::sync::CancellationToken,
    pub progress: crate::features::seo::commands::SeoAuditProgress,
}

type ResultsCacheArc = std::sync::Arc<
    std::sync::Mutex<lru::LruCache<ResultsCacheKey, (Vec<crate::models::CrawlResult>, u32)>>,
>;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub crawls: Arc<RwLock<HashMap<String, CrawlState>>>,
    pub seo_audits: Arc<RwLock<HashMap<String, SeoAuditState>>>,
    pub results_cache: ResultsCacheArc,
    pub transfer_server: std::sync::Mutex<Option<crate::transfer::server::TransferServerState>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Open Crawler");

    let builder = {
        let base = tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_fs::init())
            .plugin(tauri_plugin_notification::init())
            .plugin(tauri_plugin_share::init());

        #[cfg(mobile)]
        {
            base.plugin(tauri_plugin_mobile_sharetarget::init())
        }
        #[cfg(not(mobile))]
        {
            base
        }
    };

    builder
        .setup(|app| {
            let app_data_dir = match app.path().app_data_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    tracing::error!("Failed to get app data dir: {}", e);
                    return Err(e.into());
                }
            };

            if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
                tracing::error!("Failed to create app data dir: {}", e);
                return Err(e.into());
            }

            let db_path = app_data_dir.join("open-crawler.db");
            info!("Opening database at: {:?}", db_path);

            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("Failed to open database: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>);
                }
            };

            if let Err(e) = conn.pragma_update(None, "journal_mode", "WAL") {
                tracing::error!("Failed to enable WAL mode: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
            if let Err(e) = conn.pragma_update(None, "busy_timeout", 5000) {
                tracing::error!("Failed to set busy_timeout: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
            // WAL + synchronous=NORMAL is the recommended fast/durable combo: the
            // checkpoint guarantees crash-consistency and NORMAL only risks losing
            // the latest transactions on OS power loss (never DB corruption).
            // This is the single biggest write-throughput lever for the DbWriter.
            if let Err(e) = conn.pragma_update(None, "synchronous", "NORMAL") {
                tracing::error!("Failed to set synchronous=NORMAL: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
            // 64MB page cache (negative value = kilobytes) and mmap for fast reads.
            if let Err(e) = conn.pragma_update(None, "cache_size", -65536) {
                tracing::error!("Failed to set cache_size: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
            if let Err(e) = conn.pragma_update(None, "mmap_size", 268435456) {
                tracing::error!("Failed to set mmap_size: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
            // Keep sort/intermediate results out of the disk for the json_each
            // grouping and ORDER BY queries.
            if let Err(e) = conn.pragma_update(None, "temp_store", "MEMORY") {
                tracing::error!("Failed to set temp_store: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }

            // Run migrations
            if let Err(e) = crate::db::schema::run_migrations(&conn) {
                tracing::error!("Failed to run migrations: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }

            let state = AppState {
                db: Mutex::new(conn),
                crawls: Arc::new(RwLock::new(HashMap::new())),
                seo_audits: Arc::new(RwLock::new(HashMap::new())),
                results_cache: Arc::new(Mutex::new(lru::LruCache::new(
                    NonZeroUsize::new(512).unwrap(),
                ))),
                transfer_server: std::sync::Mutex::new(None),
            };

            app.manage(Arc::new(RwLock::new(state)));

            // Shared HTTP client (connection pool + DNS reused across commands
            // instead of building a fresh client per request). Redirects are
            // followed manually by the crawler so chains can be captured; other
            // consumers hit fixed endpoints and handle 3xx themselves.
            let http_client = reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .redirect(reqwest::redirect::Policy::none())
                .gzip(true)
                .brotli(true)
                .build()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            app.manage(http_client);

            // Start the cron scheduler (background task, one tick per minute).
            {
                let app_handle = app.handle().clone();
                let state = app.state::<Arc<RwLock<AppState>>>().inner().clone();
                tauri::async_runtime::spawn(crate::features::schedule::scheduler::run_scheduler(
                    app_handle, state,
                ));
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // app
            crate::commands::is_mobile,
            crate::commands::get_platform,
            crate::commands::get_favicon,
            crate::commands::open_project_window,
            crate::commands::close_project_window,
            crate::commands::list_open_project_windows,
            crate::commands::is_project_window,
            // projects
            crate::commands::create_project,
            crate::commands::list_projects,
            crate::commands::get_project,
            crate::commands::rename_project,
            crate::commands::delete_project,
            crate::commands::get_project_stats,
            // crawl
            crate::commands::start_crawl,
            crate::commands::stop_crawl,
            crate::commands::get_crawl_status,
            crate::commands::get_running_crawls,
            crate::commands::check_resumable_crawl,
            crate::commands::get_last_crawl_config,
            // results
            crate::commands::get_results,
            crate::commands::get_site_tree,
            crate::commands::get_site_tree_full,
            crate::commands::get_site_tree_stream,
            crate::commands::get_page_detail,
            crate::commands::get_semantic_issue_counts,
            crate::commands::get_page_html,
            crate::commands::inline_assets,
            crate::commands::recrawl_page,
            crate::commands::capture_page_screenshot,
            // analytics
            crate::commands::get_dashboard_stats,
            crate::commands::get_duplicate_groups,
            crate::commands::get_duplicate_groups_page,
            crate::commands::get_project_keywords,
            crate::commands::get_project_keywords_page,
            crate::commands::get_link_analysis,
            crate::commands::get_project_has_links,
            crate::commands::get_orphan_pages_page,
            crate::commands::get_dead_end_pages_page,
            crate::commands::get_top_anchors_page,
            crate::commands::get_external_domains_page,
            // snapshots
            crate::commands::list_crawl_snapshots,
            crate::commands::compare_crawls,
            crate::commands::compare_crawls_page,
            // pagespeed
            crate::commands::get_pagespeed_score,
            // seo
            crate::commands::get_seo_audit,
            crate::commands::run_seo_audit,
            crate::commands::get_seo_overview,
            crate::commands::run_seo_audit_all,
            crate::commands::get_seo_audit_status,
            crate::commands::stop_seo_audit,
            crate::commands::suggest_fix,
            // settings
            crate::commands::get_settings,
            crate::commands::save_settings,
            // schedule
            crate::commands::list_scheduled_jobs,
            crate::commands::create_scheduled_job,
            crate::commands::update_scheduled_job,
            crate::commands::delete_scheduled_job,
            // export
            crate::commands::export_full,
            // transfer
            crate::transfer::commands::export_package,
            crate::transfer::commands::import_package,
            crate::transfer::commands::start_transfer_server,
            crate::transfer::commands::stop_transfer_server,
            crate::transfer::commands::get_active_transfer,
            crate::transfer::commands::download_transfer,
            crate::transfer::commands::import_shared_intent,
            crate::transfer::commands::open_share_sheet,
            crate::transfer::commands::bt_send,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
