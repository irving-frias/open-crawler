use std::process::Command;
use tempfile::NamedTempFile;
use tracing::{info, warn};

use crate::error::AppError;

const SCREENSHOT_WIDTH: u32 = 1280;
const SCREENSHOT_HEIGHT: u32 = 900;

/// Capture a full-page screenshot of a URL using system Chrome/Chromium.
/// Returns PNG bytes.
pub fn capture_screenshot(url: &str) -> Result<Vec<u8>, AppError> {
    let chrome_path = find_chrome()?;

    let tmp_file = NamedTempFile::new().map_err(|e| AppError::Crawl(e.to_string()))?;
    let output_path = tmp_file.path().to_str().unwrap_or("/tmp/screenshot.png");

    let output = Command::new(&chrome_path)
        .args([
            "--headless",
            "--disable-gpu",
            "--no-sandbox",
            "--disable-software-rasterizer",
            &format!("--window-size={},{}", SCREENSHOT_WIDTH, SCREENSHOT_HEIGHT),
            &format!("--screenshot={}", output_path),
            "--hide-scrollbars",
            "--virtual-time-budget=5000",
            url,
        ])
        .output()
        .map_err(|e| AppError::Crawl(format!("Failed to run Chrome: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Chrome screenshot stderr: {}", stderr);
    }

    let png_data = std::fs::read(output_path)
        .map_err(|e| AppError::Crawl(format!("Failed to read screenshot: {}", e)))?;

    info!(
        "Screenshot captured for {}: {} bytes",
        url,
        png_data.len()
    );

    Ok(png_data)
}

fn find_chrome() -> Result<String, AppError> {
    let paths = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium-browser",
        "/usr/bin/chromium",
        "/usr/bin/google-chrome-stable",
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    // Try `which` to find chrome in PATH
    if let Ok(output) = Command::new("which").args(["google-chrome", "chromium"]).output() {
        let paths_str = String::from_utf8_lossy(&output.stdout);
        if let Some(first_line) = paths_str.lines().next() {
            if !first_line.is_empty() {
                return Ok(first_line.to_string());
            }
        }
    }

    Err(AppError::Crawl(
        "Chrome/Chromium not found. Install Google Chrome to enable page screenshots.".to_string()
    ))
}
