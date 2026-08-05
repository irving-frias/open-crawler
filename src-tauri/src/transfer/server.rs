use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use serde::Serialize;
use tiny_http::{Header, Request, Response, Server, StatusCode};
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::AppError;
use crate::AppState;

pub const DEFAULT_PORT: u16 = 45231;
pub const DEFAULT_TTL_MINUTES: u64 = 15;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Serialize)]
pub struct TransferInfo {
    pub urls: Vec<String>,
    pub port: u16,
    pub token: String,
    pub expires_in_secs: u64,
    pub file_name: String,
    pub file_size_bytes: u64,
}

/// Runtime state of the one active transfer server. Dropping it while also
/// flipping `stop` lets the accept loop exit and the port be released.
pub struct TransferServerState {
    pub path: PathBuf,
    pub token: String,
    pub port: u16,
    pub urls: Vec<String>,
    pub expires_at: SystemTime,
    pub stop: Arc<AtomicBool>,
    _server: Arc<Server>,
}

/// Starts the LAN file server serving `file_path` for `minutes` (default 15).
/// Binds `DEFAULT_PORT`, falling back to an ephemeral port if taken.
pub fn start_transfer_server(
    state: &AppState,
    file_path: &Path,
    minutes: u64,
) -> Result<TransferInfo, AppError> {
    let metadata = std::fs::metadata(file_path)?;
    if !metadata.is_file() {
        return Err(AppError::Crawl(format!(
            "{} is not a file",
            file_path.display()
        )));
    }

    let ttl = if minutes == 0 { DEFAULT_TTL_MINUTES } else { minutes };
    let expires_at = SystemTime::now() + Duration::from_secs(ttl * 60);

    let server = match Server::http(("0.0.0.0", DEFAULT_PORT)) {
        Ok(s) => s,
        Err(_) => {
            warn!("Port {DEFAULT_PORT} busy, binding an ephemeral port");
            Server::http(("0.0.0.0", 0))
                .map_err(|e| AppError::Crawl(format!("failed to bind server: {e}")))?
        }
    };
    let port = server
        .server_addr()
        .to_ip()
        .map(|addr| addr.port())
        .unwrap_or(DEFAULT_PORT);

    let token = Uuid::new_v4().simple().to_string();
    let file_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "open-crawler.ocproj".to_string());
    let file_name_enc = percent_encode(&file_name);

    let mut urls = vec![format!("http://127.0.0.1:{port}/dl/{token}/{file_name_enc}")];
    if let Ok(ip) = local_ip_address::local_ip() {
        urls.insert(0, format!("http://{ip}:{port}/dl/{token}/{file_name_enc}"));
    }

    let server = Arc::new(server);
    let stop = Arc::new(AtomicBool::new(false));

    {
        let server_loop = server.clone();
        let stop_loop = stop.clone();
        let path_loop = file_path.to_path_buf();
        let token_loop = token.clone();
        std::thread::spawn(move || {
            while !stop_loop.load(Ordering::SeqCst) {
                match server_loop.recv_timeout(POLL_INTERVAL) {
                    Ok(Some(request)) => {
                        let path = path_loop.clone();
                        let token = token_loop.clone();
                        let stop_req = stop_loop.clone();
                        let expires = expires_at;
                        std::thread::spawn(move || {
                            handle_request(request, path, token, expires, stop_req);
                        });
                    }
                    Ok(None) => {}
                    Err(e) => {
                        warn!("transfer server recv error: {e}");
                        break;
                    }
                }
            }
            info!("transfer server stopped");
        });
    }

    {
        let mut slot = state.transfer_server.lock().map_err(|e| {
            AppError::Crawl(format!("failed to lock transfer server state: {e}"))
        })?;
        *slot = Some(TransferServerState {
            path: file_path.to_path_buf(),
            token: token.clone(),
            port,
            urls: urls.clone(),
            expires_at,
            stop: stop.clone(),
            _server: server,
        });
    }

    info!(
        "Transfer server listening on port {port} serving {} ({} bytes, ttl {ttl}m)",
        file_path.display(),
        metadata.len()
    );

    Ok(TransferInfo {
        urls,
        port,
        token,
        expires_in_secs: ttl * 60,
        file_name,
        file_size_bytes: metadata.len(),
    })
}

/// Stops the active transfer server (no-op if none is running).
pub fn stop_transfer_server(state: &AppState) -> Result<(), AppError> {
    let mut slot = state.transfer_server.lock().map_err(|e| {
        AppError::Crawl(format!("failed to lock transfer server state: {e}"))
    })?;
    if let Some(active) = slot.take() {
        active.stop.store(true, Ordering::SeqCst);
        info!("Transfer server stopped");
    }
    Ok(())
}

/// Returns the currently active transfer, if any.
pub fn active_transfer(state: &AppState) -> Option<TransferInfo> {
    let slot = state.transfer_server.lock().ok()?;
    let active = slot.as_ref()?;
    let now = SystemTime::now();
    let expired = now > active.expires_at;
    Some(TransferInfo {
        urls: active.urls.clone(),
        port: active.port,
        token: active.token.clone(),
        expires_in_secs: active
            .expires_at
            .duration_since(now)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        file_name: active
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        file_size_bytes: std::fs::metadata(&active.path).map(|m| m.len()).unwrap_or(0),
    })
    .filter(|_| !expired)
}

fn handle_request(
    request: Request,
    path: PathBuf,
    token: String,
    expires_at: SystemTime,
    stop: Arc<AtomicBool>,
) {
    if SystemTime::now() > expires_at {
        stop.store(true, Ordering::SeqCst);
        respond_text(request, 410, "This share has expired");
        return;
    }

    let url = request.url().to_string();
    let path_segment = url.split('?').next().unwrap_or(&url).to_string();

    match path_segment.as_str() {
        "/health" => respond_text(request, 200, "ok"),
        "/" => respond_landing(request, &path, &token),
        _ => {
            let expected = format!("/dl/{token}/");
            if path_segment.starts_with(&expected) {
                respond_file(request, &path);
            } else {
                respond_text(request, 404, "Not found");
            }
        }
    }
}

fn respond_landing(request: Request, path: &Path, token: &str) {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "open-crawler.ocproj".to_string());
    let name_enc = percent_encode(&file_name);
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let size_mb = size as f64 / (1024.0 * 1024.0);
    let size_label = if size_mb >= 1.0 {
        format!("{size_mb:.1} MB")
    } else {
        format!("{} KB", size / 1024)
    };
    let url = format!("/dl/{token}/{name_enc}");

    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Open Crawler transfer</title>
<style>
  body {{ font-family: system-ui, sans-serif; background:#1e1e2e; color:#e0e0e0;
         display:flex; align-items:center; justify-content:center; min-height:100vh; margin:0; }}
  .card {{ background:#2e3440; padding:2.5rem; border-radius:16px; max-width:420px;
          text-align:center; box-shadow:0 8px 30px rgba(0,0,0,.4); }}
  h1 {{ margin:0 0 .5rem; font-size:1.4rem; }}
  p {{ color:#a8b3c5; margin:.25rem 0; }}
  .btn {{ display:inline-block; margin-top:1.5rem; background:#5e81ac; color:#fff;
         text-decoration:none; padding:.8rem 1.6rem; border-radius:10px; font-weight:600; }}
  .meta {{ margin-top:1rem; font-size:.85rem; color:#7f8ea3; }}
</style>
</head>
<body>
  <div class="card">
    <h1>{file_name}</h1>
    <p>{size_label}</p>
    <a class="btn" href="{url}">Download package</a>
    <p class="meta">Sent from Open Crawler · expires in 15 minutes</p>
  </div>
</body>
</html>"#
    );
    respond_text(request, 200, &html);
}

fn respond_file(request: Request, path: &Path) {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return respond_text(request, 500, "Cannot read file"),
    };
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "open-crawler.ocproj".to_string());

    let mut response = Response::from_file(file);
    let disposition = format!("attachment; filename=\"{file_name}\"");
    if let Ok(h) = Header::from_bytes("Content-Disposition", disposition.as_str()) {
        response = response.with_header(h);
    }
    if let Ok(h) = Header::from_bytes("Content-Type", "application/octet-stream") {
        response = response.with_header(h);
    }
    let _ = request.respond(response);
}

fn respond_text(request: Request, status: u16, body: &str) {
    let response = Response::from_string(body.to_string())
        .with_status_code(StatusCode(status))
        .with_header(
            Header::from_bytes("Content-Type", "text/plain; charset=utf-8")
                .expect("valid header"),
        );
    let _ = request.respond(response);
}

fn percent_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::num::NonZeroUsize;
    use std::sync::Mutex as StdMutex;
    use tokio::sync::RwLock;

    fn test_state() -> AppState {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let cache: lru::LruCache<crate::ResultsCacheKey, (Vec<crate::models::CrawlResult>, u32)> =
            lru::LruCache::new(NonZeroUsize::new(64).unwrap());
        AppState {
            db: StdMutex::new(conn),
            crawls: Arc::new(RwLock::new(HashMap::new())),
            results_cache: Arc::new(StdMutex::new(cache)),
            transfer_server: StdMutex::new(None),
        }
    }

    fn http_get(addr: &str, path: &str) -> (u16, String) {
        let mut stream = std::net::TcpStream::connect(addr).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let req = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
        stream.write_all(req.as_bytes()).unwrap();

        let mut header_done = false;
        let mut header = Vec::new();
        let mut byte = [0u8; 1];
        while !header_done {
            match stream.read(&mut byte) {
                Ok(0) => break,
                Ok(_) => header.push(byte[0]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(e) => panic!("read header failed: {e}"),
            }
            if header.len() >= 4 && &header[header.len() - 4..] == b"\r\n\r\n" {
                header_done = true;
            }
        }

        let head = String::from_utf8_lossy(&header).to_string();
        let status = head
            .lines()
            .next()
            .unwrap_or("")
            .split(' ')
            .nth(1)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        let content_length = head
            .lines()
            .find_map(|l| {
                let l = l.trim_end_matches('\r');
                let (k, v) = l.split_once(':')?;
                (k.eq_ignore_ascii_case("content-length")).then(|| v.trim().parse::<usize>().ok())?
            })
            .unwrap_or(0);

        let mut body = vec![0u8; content_length];
        let mut read_total = 0;
        while read_total < content_length {
            match stream.read(&mut body[read_total..]) {
                Ok(0) => break,
                Ok(n) => read_total += n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(e) => panic!("read body failed: {e}"),
            }
        }
        body.truncate(read_total);
        (status, String::from_utf8_lossy(&body).to_string())
    }

    #[test]
    fn test_transfer_server_serves_file_and_health() {
        let dir = std::env::temp_dir().join(format!("oc-transfer-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("pkg.ocproj");
        std::fs::write(&path, b"package-data").unwrap();

        let state = test_state();
        let info = start_transfer_server(&state, &path, 1).unwrap();
        assert_eq!(info.file_size_bytes, 12);
        assert!(!info.urls.is_empty());

        let addr = format!("127.0.0.1:{}", info.port);

        let (status, body) = http_get(&addr, "/health");
        assert_eq!(status, 200);
        assert_eq!(body.trim(), "ok");

        let (status, body) = http_get(&addr, &format!("/dl/{}/pkg.ocproj", info.token));
        assert_eq!(status, 200);
        assert_eq!(body.trim(), "package-data");

        let (status, _) = http_get(&addr, "/dl/wrong-token/x");
        assert_eq!(status, 404);

        let (status, _) = http_get(&addr, "/");
        assert_eq!(status, 200);

        let active = active_transfer(&state).unwrap();
        assert_eq!(active.token, info.token);
        assert_eq!(active.port, info.port);
        assert!(active.expires_in_secs > 0);

        stop_transfer_server(&state).unwrap();
        assert!(active_transfer(&state).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
