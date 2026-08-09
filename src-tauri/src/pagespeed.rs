use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageSpeedData {
    pub score: Option<u32>,
    pub fcp: Option<String>,
    pub lcp: Option<String>,
    pub cls: Option<String>,
    pub tbt: Option<String>,
    pub speed_index: Option<String>,
    pub error: Option<String>,
}

const PSI_ENDPOINT: &str = "https://www.googleapis.com/pagespeedonline/v5/runPagespeed";

/// Fetches a Lighthouse performance audit for `url` via the Google
/// PageSpeed Insights API. Works without an API key for low volumes.
pub async fn fetch_pagespeed(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
) -> Result<PageSpeedData, AppError> {
    let encoded: String = url::form_urlencoded::byte_serialize(url.as_bytes()).collect();
    let mut request_url =
        format!("{PSI_ENDPOINT}?url={encoded}&strategy=desktop&category=performance");
    if let Some(key) = api_key.filter(|k| !k.is_empty()) {
        let encoded_key: String = url::form_urlencoded::byte_serialize(key.as_bytes()).collect();
        request_url.push_str(&format!("&key={encoded_key}"));
    }

    let resp = client
        .get(&request_url)
        .timeout(std::time::Duration::from_secs(90))
        .send()
        .await
        .map_err(|e| AppError::Pagespeed(format!("PageSpeed request failed: {e}")))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| AppError::Pagespeed(format!("PageSpeed read failed: {e}")))?;

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| AppError::Pagespeed(format!("PageSpeed invalid JSON: {e}")))?;

    if !status.is_success() {
        let msg = json
            .pointer("/error/message")
            .and_then(|v| v.as_str())
            .unwrap_or(&body);
        return Err(AppError::Pagespeed(format!(
            "PageSpeed API error ({status}): {msg}"
        )));
    }

    let audits = json.pointer("/lighthouseResult/audits");
    let mut data = PageSpeedData {
        score: json
            .pointer("/lighthouseResult/categories/performance/score")
            .and_then(|v| v.as_f64())
            .map(|s| (s * 100.0).round() as u32),
        ..Default::default()
    };

    if let Some(audits) = audits {
        for (field, name) in [
            ("fcp", "first-contentful-paint"),
            ("lcp", "largest-contentful-paint"),
            ("cls", "cumulative-layout-shift"),
            ("tbt", "total-blocking-time"),
            ("speed_index", "speed-index"),
        ] {
            let display = audits
                .pointer(&format!("/{name}/displayValue"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            match field {
                "fcp" => data.fcp = display,
                "lcp" => data.lcp = display,
                "cls" => data.cls = display,
                "tbt" => data.tbt = display,
                _ => data.speed_index = display,
            }
        }
    }

    if data.score.is_none() {
        data.error = Some("No performance score in PageSpeed response".to_string());
    }

    Ok(data)
}
