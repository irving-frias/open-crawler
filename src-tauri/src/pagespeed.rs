use std::time::Duration;

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

/// How long a stored PageSpeed result is trusted before the API is called
/// again: 24 h. PageSpeed scores barely move within a day for an unchanged
/// page, while a daily refresh keeps the data from going stale. The
/// `pagespeed_fetched_at` column (migration 018) stamps every write, and
/// `CrawlRepo::get_cached_pagespeed` enforces this TTL.
pub const PAGESPEED_CACHE_TTL_HOURS: i64 = 24;

/// Retry policy for the PageSpeed Insights API.
///
/// A 429 (or a transient 5xx / transport failure) is retried with
/// exponential backoff — 1 s, 2 s, 4 s by default — up to `max_attempts`
/// tries in total. When the API answers 429 with a `Retry-After` header
/// (seconds), that value is honoured instead, capped at
/// `retry_after_max_secs`.
#[derive(Debug, Clone, Copy)]
pub struct PagespeedRetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub retry_after_max_secs: u64,
}

impl Default for PagespeedRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            base_delay_ms: 1_000,
            max_delay_ms: 30_000,
            retry_after_max_secs: 60,
        }
    }
}

impl PagespeedRetryPolicy {
    /// Delay before retry number `attempt` (0-based: 0 = after the first
    /// failed try). `Retry-After` wins over the backoff when present.
    pub fn delay(&self, attempt: u32, retry_after_secs: Option<u64>) -> Duration {
        if let Some(secs) = retry_after_secs {
            return Duration::from_secs(secs.min(self.retry_after_max_secs));
        }
        let backoff = self.base_delay_ms.saturating_mul(1u64 << attempt.min(5));
        Duration::from_millis(backoff.min(self.max_delay_ms))
    }
}

/// Minimal raw HTTP response the retry loop needs: status, an optional
/// `Retry-After` (seconds), and the body. Kept transport-agnostic so tests
/// can drive the retry logic with a scripted fake instead of the network.
pub struct RawPagespeedResponse {
    pub status: u16,
    pub retry_after_secs: Option<u64>,
    pub body: String,
}

/// A 429 or a 5xx is transient and worth retrying; any other 4xx is a
/// client error (bad URL, bad key, ...) and fails immediately.
fn is_retryable_status(status: u16) -> bool {
    status == 429 || (500..600).contains(&status)
}

fn build_request_url(url: &str, api_key: Option<&str>, endpoint: &str) -> String {
    let encoded: String = url::form_urlencoded::byte_serialize(url.as_bytes()).collect();
    let mut request_url = format!("{endpoint}?url={encoded}&strategy=desktop&category=performance");
    if let Some(key) = api_key.filter(|k| !k.is_empty()) {
        let encoded_key: String = url::form_urlencoded::byte_serialize(key.as_bytes()).collect();
        request_url.push_str(&format!("&key={encoded_key}"));
    }
    request_url
}

/// Parses one PageSpeed Insights response. Non-2xx statuses become an
/// `AppError::Pagespeed` carrying the API's own error message when present.
fn parse_pagespeed_response(status: u16, body: &str) -> Result<PageSpeedData, AppError> {
    let json: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| AppError::Pagespeed(format!("PageSpeed invalid JSON: {e}")))?;

    if !(200..300).contains(&status) {
        let msg = json
            .pointer("/error/message")
            .and_then(|v| v.as_str())
            .unwrap_or(body);
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

/// Fetches one PageSpeed audit through `transport`, retrying 429 / 5xx /
/// transport failures with backoff (see [`PagespeedRetryPolicy`]).
async fn fetch_with_transport<F, Fut>(
    transport: F,
    url: &str,
    api_key: Option<&str>,
    endpoint: &str,
    policy: PagespeedRetryPolicy,
) -> Result<PageSpeedData, AppError>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<RawPagespeedResponse, AppError>>,
{
    let request_url = build_request_url(url, api_key, endpoint);
    let mut attempt: u32 = 0;
    loop {
        match transport(request_url.clone()).await {
            Ok(raw) => match parse_pagespeed_response(raw.status, &raw.body) {
                Ok(data) => return Ok(data),
                Err(err) => {
                    if !is_retryable_status(raw.status) || attempt + 1 >= policy.max_attempts {
                        return Err(err);
                    }
                    tokio::time::sleep(policy.delay(attempt, raw.retry_after_secs)).await;
                }
            },
            Err(err) => {
                if attempt + 1 >= policy.max_attempts {
                    return Err(err);
                }
                tokio::time::sleep(policy.delay(attempt, None)).await;
            }
        }
        attempt += 1;
    }
}

/// Fetches a Lighthouse performance audit for `url` via the Google
/// PageSpeed Insights API. Works without an API key for low volumes.
///
/// Retries 429 / 5xx / transport failures with exponential backoff
/// (honouring `Retry-After`); other errors fail immediately.
pub async fn fetch_pagespeed(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
) -> Result<PageSpeedData, AppError> {
    let client = client.clone();
    fetch_with_transport(
        move |request_url| {
            let client = client.clone();
            async move {
                let resp = client
                    .get(&request_url)
                    .timeout(std::time::Duration::from_secs(90))
                    .send()
                    .await
                    .map_err(|e| AppError::Pagespeed(format!("PageSpeed request failed: {e}")))?;
                let status = resp.status().as_u16();
                let retry_after_secs = resp
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.trim().parse::<u64>().ok());
                let body = resp
                    .text()
                    .await
                    .map_err(|e| AppError::Pagespeed(format!("PageSpeed read failed: {e}")))?;
                Ok(RawPagespeedResponse {
                    status,
                    retry_after_secs,
                    body,
                })
            }
        },
        url,
        api_key,
        PSI_ENDPOINT,
        PagespeedRetryPolicy::default(),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    fn psi_json(score: f64) -> String {
        format!(
            r#"{{"lighthouseResult":{{"categories":{{"performance":{{"score":{score}}}}},"audits":{{"first-contentful-paint":{{"displayValue":"1.2 s"}},"largest-contentful-paint":{{"displayValue":"2.5 s"}},"cumulative-layout-shift":{{"displayValue":"0.01"}},"total-blocking-time":{{"displayValue":"120 ms"}},"speed-index":{{"displayValue":"3.4 s"}}}}}}}}"#
        )
    }

    fn ok_response(score: f64) -> RawPagespeedResponse {
        RawPagespeedResponse {
            status: 200,
            retry_after_secs: None,
            body: psi_json(score),
        }
    }

    fn status_response(status: u16, retry_after_secs: Option<u64>) -> RawPagespeedResponse {
        RawPagespeedResponse {
            status,
            retry_after_secs,
            body: r#"{"error":{"message":"rate limited"}}"#.to_string(),
        }
    }

    /// Policy with ~zero sleeps so retry tests run in milliseconds.
    fn fast_policy() -> PagespeedRetryPolicy {
        PagespeedRetryPolicy {
            max_attempts: 4,
            base_delay_ms: 1,
            max_delay_ms: 5,
            retry_after_max_secs: 60,
        }
    }

    /// Builds a scripted transport that answers from `scripted` in order and
    /// counts how many times it was called.
    fn scripted_transport(
        scripted: Vec<Result<RawPagespeedResponse, AppError>>,
        calls: Arc<AtomicUsize>,
    ) -> impl Fn(
        String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<RawPagespeedResponse, AppError>> + Send>,
    > {
        let queue = Arc::new(Mutex::new(VecDeque::from(scripted)));
        move |_url: String| {
            let queue = queue.clone();
            let calls = calls.clone();
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                queue
                    .lock()
                    .unwrap()
                    .pop_front()
                    .unwrap_or_else(|| Err(AppError::Pagespeed("script exhausted".to_string())))
            })
        }
    }

    #[test]
    fn retry_delay_backoff_and_retry_after_cap() {
        let policy = PagespeedRetryPolicy::default();
        assert_eq!(policy.delay(0, None), Duration::from_millis(1_000));
        assert_eq!(policy.delay(1, None), Duration::from_millis(2_000));
        assert_eq!(policy.delay(2, None), Duration::from_millis(4_000));
        assert_eq!(policy.delay(10, None), Duration::from_millis(30_000));
        // Retry-After wins over the backoff, capped at 60 s.
        assert_eq!(policy.delay(0, Some(45)), Duration::from_secs(45));
        assert_eq!(policy.delay(0, Some(3_600)), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn fetch_succeeds_first_try_without_retry() {
        let calls = Arc::new(AtomicUsize::new(0));
        let transport = scripted_transport(vec![Ok(ok_response(0.9))], calls.clone());
        let data = fetch_with_transport(
            transport,
            "https://x.com/a",
            None,
            PSI_ENDPOINT,
            fast_policy(),
        )
        .await
        .unwrap();
        assert_eq!(data.score, Some(90));
        assert_eq!(data.fcp.as_deref(), Some("1.2 s"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn fetch_retries_429_then_succeeds() {
        let calls = Arc::new(AtomicUsize::new(0));
        let transport = scripted_transport(
            vec![
                Ok(status_response(429, Some(0))),
                Ok(status_response(429, None)),
                Ok(ok_response(0.75)),
            ],
            calls.clone(),
        );
        let data = fetch_with_transport(
            transport,
            "https://x.com/a",
            None,
            PSI_ENDPOINT,
            fast_policy(),
        )
        .await
        .unwrap();
        assert_eq!(data.score, Some(75));
        assert_eq!(calls.load(Ordering::SeqCst), 3, "two 429s then a success");
    }

    #[tokio::test]
    async fn fetch_gives_up_after_max_attempts_on_429() {
        let calls = Arc::new(AtomicUsize::new(0));
        let transport = scripted_transport(
            vec![
                Ok(status_response(429, None)),
                Ok(status_response(429, None)),
                Ok(status_response(429, None)),
                Ok(status_response(429, None)),
                Ok(ok_response(0.9)),
            ],
            calls.clone(),
        );
        let err = fetch_with_transport(
            transport,
            "https://x.com/a",
            None,
            PSI_ENDPOINT,
            fast_policy(),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("429"), "error was: {err}");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            4,
            "max_attempts tries, then give up"
        );
    }

    #[tokio::test]
    async fn fetch_does_not_retry_client_errors() {
        let calls = Arc::new(AtomicUsize::new(0));
        let transport = scripted_transport(
            vec![Ok(status_response(400, None)), Ok(ok_response(0.9))],
            calls.clone(),
        );
        let err = fetch_with_transport(
            transport,
            "https://x.com/a",
            None,
            PSI_ENDPOINT,
            fast_policy(),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("400"), "error was: {err}");
        assert_eq!(calls.load(Ordering::SeqCst), 1, "4xx is not retried");
    }

    #[tokio::test]
    async fn fetch_retries_transport_failures() {
        let calls = Arc::new(AtomicUsize::new(0));
        let transport = scripted_transport(
            vec![
                Err(AppError::Pagespeed("boom".to_string())),
                Ok(ok_response(0.5)),
            ],
            calls.clone(),
        );
        let data = fetch_with_transport(
            transport,
            "https://x.com/a",
            None,
            PSI_ENDPOINT,
            fast_policy(),
        )
        .await
        .unwrap();
        assert_eq!(data.score, Some(50));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
