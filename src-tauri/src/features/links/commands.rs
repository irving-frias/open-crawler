use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::models::{AnchorAgg, DomainAgg, LinkAnalysis};
use crate::AppState;

#[tauri::command]
pub async fn get_link_analysis(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<LinkAnalysis, AppError> {
    with_repo(&state, move |repo| repo.get_link_analysis(&project_id)).await
}

#[tauri::command]
pub async fn get_project_has_links(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
) -> Result<bool, AppError> {
    with_repo(&state, move |repo| repo.project_has_links(&project_id)).await
}

#[tauri::command]
pub async fn get_orphan_pages_page(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    page: u32,
    page_size: u32,
) -> Result<(Vec<String>, u32), AppError> {
    with_repo(&state, move |repo| {
        repo.orphan_pages_page(&project_id, page, page_size)
            .map(|(items, total)| (items, total as u32))
    })
    .await
}

#[tauri::command]
pub async fn get_dead_end_pages_page(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    page: u32,
    page_size: u32,
) -> Result<(Vec<String>, u32), AppError> {
    with_repo(&state, move |repo| {
        repo.dead_end_pages_page(&project_id, page, page_size)
            .map(|(items, total)| (items, total as u32))
    })
    .await
}

#[tauri::command]
pub async fn get_top_anchors_page(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    page: u32,
    page_size: u32,
) -> Result<(Vec<AnchorAgg>, u32), AppError> {
    with_repo(&state, move |repo| {
        repo.top_anchors_page(&project_id, page, page_size)
            .map(|(items, total)| (items, total as u32))
    })
    .await
}

#[tauri::command]
pub async fn get_external_domains_page(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    page: u32,
    page_size: u32,
) -> Result<(Vec<DomainAgg>, u32), AppError> {
    with_repo(&state, move |repo| {
        repo.external_domains_page(&project_id, page, page_size)
            .map(|(items, total)| (items, total as u32))
    })
    .await
}
