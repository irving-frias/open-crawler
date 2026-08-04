pub mod commands;
pub mod crawler;
pub mod db;
pub mod error;
pub mod features;
pub mod models;
pub mod nesting_table;
pub mod pagespeed;

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

type ResultsCacheArc = std::sync::Arc<std::sync::Mutex<lru::LruCache<ResultsCacheKey, (Vec<crate::models::CrawlResult>, u32)>>>;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub crawls: Arc<RwLock<HashMap<String, CrawlState>>>,
    pub results_cache: ResultsCacheArc,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Open Crawler");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_share::init())
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
                results_cache: Arc::new(Mutex::new(lru::LruCache::new(NonZeroUsize::new(512).unwrap()))),
            };

            app.manage(Arc::new(RwLock::new(state)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // app
            crate::commands::is_mobile,
            crate::commands::get_favicon,
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
            crate::commands::get_page_detail,
            crate::commands::get_semantic_issue_counts,
            crate::commands::get_page_html,
            crate::commands::inline_assets,
            crate::commands::recrawl_page,
            crate::commands::capture_page_screenshot,
            // analytics
            crate::commands::get_dashboard_stats,
            crate::commands::get_duplicate_groups,
            crate::commands::get_project_keywords,
            // snapshots
            crate::commands::list_crawl_snapshots,
            crate::commands::compare_crawls,
            // pagespeed
            crate::commands::get_pagespeed_score,
            // settings
            crate::commands::get_settings,
            crate::commands::save_settings,
            // export
            crate::commands::export_full,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
