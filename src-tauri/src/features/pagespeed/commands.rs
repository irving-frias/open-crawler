use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::features::with_repo;
use crate::pagespeed::PageSpeedData;
use crate::AppState;

#[tauri::command]
pub async fn get_pagespeed_score(
    state: State<'_, Arc<RwLock<AppState>>>,
    project_id: String,
    url: String,
) -> Result<PageSpeedData, AppError> {
    let page_id = with_repo(&state, |repo| repo.find_page_id(&project_id, &url)).await?;

    if let Some(page_id) = page_id.as_deref() {
        let (score, json) = with_repo(&state, |repo| repo.get_pagespeed(page_id)).await?;
        if let Some(json) = json {
            if let Ok(cached) = serde_json::from_str::<PageSpeedData>(&json) {
                return Ok(cached);
            }
            let _ = score;
        }
    }

    let api_key = with_repo(&state, |repo| repo.get_setting("pagespeed_api_key")).await?;

    let data = crate::pagespeed::fetch_pagespeed(&url, api_key.as_deref()).await?;

    if let Some(page_id) = page_id.as_deref() {
        let score = data.score.map(|s| s as f64);
        let json = serde_json::to_string(&data).ok();
        let _ = with_repo(&state, |repo| repo.update_pagespeed(page_id, score, json.as_deref())).await;
    }

    Ok(data)
}
