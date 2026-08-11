use scraper::{Html, Selector};
#[cfg(mobile)]
use tauri::AppHandle;
use tauri::State;
#[cfg(not(mobile))]
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use url::Url;

use crate::crawler::fetcher::{HtmlFetcher, HttpFetcher};
use crate::error::AppError;
use crate::models::crawl_config::IMPLICIT_USER_AGENT;

/// Opens a dedicated native window for a project (`project-{id}`), so each
/// project's crawl / SEO state is isolated and can run in parallel. The window
/// loads `index.html?project={id}`, which boots the shell pinned to that
/// project. Re-opening an already-open project focuses its window instead.
/// Desktop only — mobile keeps the in-app project switcher.
#[tauri::command]
pub async fn open_project_window(
    app: AppHandle,
    project_id: String,
    title: String,
) -> Result<(), AppError> {
    #[cfg(mobile)]
    let _ = (&app, &project_id, &title);
    #[cfg(not(mobile))]
    {
        let label = format!("project-{project_id}");
        if let Some(win) = app.get_webview_window(&label) {
            let _ = win.set_focus();
            return Ok(());
        }
        // Note: no `index.html` in the path — joining it onto the dev server
        // URL (`http://localhost:5173/index.html?project=..`) 404s in dev
        // because SvelteKit has no literal `/index.html` route. A bare query
        // keeps the path at `/`, which the dev server serves and the asset
        // protocol resolves to `index.html` in production.
        let url = WebviewUrl::App(format!("?project={project_id}").into());
        let app_for_main = app.clone();
        app.run_on_main_thread(move || {
            let builder = WebviewWindowBuilder::new(&app_for_main, &label, url)
                .title(title)
                .inner_size(1200.0, 800.0)
                .min_inner_size(900.0, 600.0)
                .resizable(true);
            if let Err(e) = builder.build() {
                tracing::error!("Failed to open project window {label}: {e}");
            }
        })
        .map_err(|e| AppError::Crawl(format!("Failed to open project window: {e}")))?;
    }
    Ok(())
}

/// Closes the dedicated window for a project, if open.
#[tauri::command]
pub async fn close_project_window(app: AppHandle, project_id: String) -> Result<(), AppError> {
    #[cfg(mobile)]
    let _ = (&app, &project_id);
    #[cfg(not(mobile))]
    {
        let label = format!("project-{project_id}");
        let app_for_main = app.clone();
        app.run_on_main_thread(move || {
            if let Some(win) = app_for_main.get_webview_window(&label) {
                let _ = win.close();
            }
        })
        .map_err(|e| AppError::Crawl(format!("Failed to close project window: {e}")))?;
    }
    Ok(())
}

/// Lists the labels of all open project windows (`project-{id}`).
#[tauri::command]
pub async fn list_open_project_windows(app: AppHandle) -> Result<Vec<String>, AppError> {
    #[cfg(not(mobile))]
    {
        let labels: Vec<String> = app
            .webview_windows()
            .keys()
            .filter(|l| l.starts_with("project-"))
            .cloned()
            .collect();
        Ok(labels)
    }
    #[cfg(mobile)]
    {
        let _ = app;
        Ok(Vec::new())
    }
}

/// True when running inside a dedicated project window (`project-{id}`).
#[tauri::command]
pub async fn is_project_window(app: AppHandle) -> Result<bool, AppError> {
    #[cfg(not(mobile))]
    {
        let label = app
            .webview_windows()
            .iter()
            .find(|(_, w)| w.label().starts_with("project-"))
            .map(|(l, _)| l.clone());
        Ok(label.is_some())
    }
    #[cfg(mobile)]
    {
        let _ = app;
        Ok(false)
    }
}

#[tauri::command]
pub fn is_mobile() -> bool {
    cfg!(mobile)
}

/// The host OS as a lowercase string (`"linux"`, `"windows"`, `"macos"`).
#[tauri::command]
pub fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

/// Best-effort favicon discovery: parses the page's `<link rel="icon">` tags,
/// falling back to `/favicon.ico` at the site origin. Returns `None` when the
/// URL is invalid or the icon cannot be resolved.
#[tauri::command]
pub async fn get_favicon(
    http: State<'_, reqwest::Client>,
    url: String,
) -> Result<Option<String>, AppError> {
    let page_url = match Url::parse(&url) {
        Ok(u) if matches!(u.scheme(), "http" | "https") => u,
        _ => return Ok(None),
    };

    let fetcher = HttpFetcher::new(
        Some(http.inner().clone()),
        IMPLICIT_USER_AGENT,
        10_000,
        Vec::new(),
        Vec::new(),
        None,
        None,
    )?;

    if let Ok(response) = fetcher.fetch(&page_url).await {
        let document = Html::parse_document(&response.html);
        if let Ok(selector) = Selector::parse("link[rel~='icon']") {
            for el in document.select(&selector) {
                let Some(href) = el.value().attr("href").map(str::trim) else {
                    continue;
                };
                if href.is_empty() || href.starts_with("data:") || href.starts_with("javascript:") {
                    continue;
                }
                if let Ok(resolved) = response.url.join(href) {
                    if matches!(resolved.scheme(), "http" | "https") {
                        return Ok(Some(resolved.to_string()));
                    }
                }
            }
        }
        return Ok(origin_favicon(&response.url));
    }

    Ok(origin_favicon(&page_url))
}

fn origin_favicon(url: &Url) -> Option<String> {
    match url.scheme() {
        "http" | "https" => {
            let mut base = url.clone();
            base.set_path("/favicon.ico");
            base.set_query(None);
            base.set_fragment(None);
            Some(base.to_string())
        }
        _ => None,
    }
}
