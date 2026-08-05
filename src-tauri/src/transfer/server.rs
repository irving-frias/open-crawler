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
    let file_name = file_name_of(file_path);
    let file_name_enc = percent_encode(&file_name);

    let urls = lan_urls(port, &token, &file_name_enc);

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
    let expires_in_secs = active
        .expires_at
        .duration_since(now)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let file_name = active
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let file_size_bytes = std::fs::metadata(&active.path).map(|m| m.len()).unwrap_or(0);
    (!expired).then_some(TransferInfo {
        urls: active.urls.clone(),
        port: active.port,
        token: active.token.clone(),
        expires_in_secs,
        file_name,
        file_size_bytes,
    })
}

fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "open-crawler.ocproj".to_string())
}

/// Builds the download URLs shown to the sender. The default-route address
/// (what `local_ip()` returns) goes first because it is the most likely one
/// to be reachable; every other non-loopback, non-link-local IPv4 interface
/// is listed too so the sender can pick the address that actually works (e.g.
/// WiFi vs. VPN vs. cellular). `127.0.0.1` is appended last as a self-test.
fn lan_urls(port: u16, token: &str, file_name_enc: &str) -> Vec<String> {
    let dl = |ip: &std::net::Ipv4Addr| format!("http://{ip}:{port}/dl/{token}/{file_name_enc}");

    let mut urls: Vec<String> = Vec::new();
    let push = |ip: &std::net::Ipv4Addr, urls: &mut Vec<String>| {
        let url = dl(ip);
        if !urls.contains(&url) {
            urls.push(url);
        }
    };

    if let Ok(std::net::IpAddr::V4(ip)) = local_ip_address::local_ip() {
        push(&ip, &mut urls);
    }

    if let Ok(ifas) = local_ip_address::list_afinet_netifas() {
        for (_, ip) in ifas {
            if let std::net::IpAddr::V4(v4) = ip {
                if v4.is_loopback() || v4.is_link_local() || v4.is_unspecified() {
                    continue;
                }
                push(&v4, &mut urls);
            }
        }
    }

    urls.push(dl(&std::net::Ipv4Addr::LOCALHOST));
    urls
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
        "/receive" => {
            let peer = url
                .split('?')
                .nth(1)
                .and_then(|q| {
                    q.split('&')
                        .find(|kv| kv.starts_with("peer="))
                        .map(|kv| &kv[5..])
                })
                .map(percent_decode)
                .unwrap_or_default();
            respond_receive(request, &peer);
        }
        "/" => {
            let ttl_minutes = expires_at
                .duration_since(SystemTime::now())
                .map(|d| d.as_secs().div_ceil(60))
                .unwrap_or(1);
            respond_landing(request, &path, &token, ttl_minutes);
        }
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

fn respond_landing(request: Request, path: &Path, token: &str, ttl_minutes: u64) {
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
    let expiry_label = if ttl_minutes == 1 {
        "expires in 1 minute".to_string()
    } else {
        format!("expires in {ttl_minutes} minutes")
    };

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
    <p class="meta">Sent from Open Crawler · {expiry_label}</p>
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

/// Browser-only receiver page (`/receive?peer=<peerId>`): connects to the
/// sender's WebRTC peer and offers the received package as a download. Lets
/// anyone on the same network receive a package without installing the app.
fn respond_receive(request: Request, peer: &str) {
    let peer_js = peer.replace('\\', "\\\\").replace('\'', "\\'");
    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Open Crawler — Receive</title>
<script src="https://unpkg.com/peerjs@1.5.5/dist/peerjs.min.js"></script>
<style>
  body {{ font-family: system-ui, sans-serif; background:#1e1e2e; color:#e0e0e0;
         display:flex; align-items:center; justify-content:center; min-height:100vh; margin:0; }}
  .card {{ background:#2e3440; padding:2.5rem; border-radius:16px; max-width:420px;
          text-align:center; box-shadow:0 8px 30px rgba(0,0,0,.4); width:100%; }}
  h1 {{ margin:0 0 .5rem; font-size:1.4rem; }}
  p {{ color:#a8b3c5; margin:.25rem 0; }}
  progress {{ width:100%; height:10px; margin:1.5rem 0 .5rem; appearance:none; border:none;
             border-radius:5px; overflow:hidden; background:#3b4252; }}
  progress::-webkit-progress-bar {{ background:#3b4252; }}
  progress::-webkit-progress-value {{ background:#5e81ac; }}
  .btn {{ display:none; margin-top:1.5rem; background:#5e81ac; color:#fff;
         text-decoration:none; padding:.8rem 1.6rem; border-radius:10px; font-weight:600; }}
  .meta {{ margin-top:1rem; font-size:.85rem; color:#7f8ea3; }}
</style>
</head>
<body>
  <div class="card">
    <h1>Open Crawler</h1>
    <p id="status">Connecting…</p>
    <progress id="progress" max="100" value="0"></progress>
    <p id="pct"></p>
    <a id="download" class="btn">Save package</a>
    <p class="meta">Receiving a package sent from Open Crawler</p>
  </div>
<script>
(function () {{
  var peerId = {peer_js_placeholder};
  var statusEl = document.getElementById('status');
  var progressEl = document.getElementById('progress');
  var pctEl = document.getElementById('pct');
  var downloadEl = document.getElementById('download');
  function status(msg) {{ statusEl.textContent = msg; }}
  function setPct(v) {{ progressEl.value = v; pctEl.textContent = v + '%'; }}

  if (!peerId) {{ status('Missing peer id.'); return; }}
  if (typeof window.Peer === 'undefined') {{ status('This browser cannot connect (needs WebRTC).'); return; }}

  var peer = new Peer('ocp-browser-' + Math.random().toString(36).slice(2, 12));
  peer.on('error', function (e) {{ status('Error: ' + (e.message || e.type)); }});
  peer.on('open', function () {{
    var conn = peer.connect(peerId, {{ reliable: true }});
    var chunks = [];
    var received = 0, total = 0, name = 'package.ocproj';
    conn.on('open', function () {{ status('Connected — receiving…'); }});
    conn.on('error', function (e) {{ status('Error: ' + (e.message || e.type)); }});
    conn.on('data', function (data) {{
      if (data && typeof data === 'object' && data.type === 'header') {{
        name = data.name || name; total = data.size || 0;
        conn.send({{ type: 'ack' }});
      }} else if (data && typeof data === 'object' && data.type === 'done') {{
        var blob = new Blob(chunks, {{ type: 'application/octet-stream' }});
        downloadEl.href = URL.createObjectURL(blob);
        downloadEl.download = name;
        downloadEl.style.display = 'inline-block';
        downloadEl.textContent = 'Save ' + name + ' (' + (blob.size / 1048576).toFixed(1) + ' MB)';
        status('Done — save the package file.');
        setPct(100);
        peer.destroy();
      }} else if (data && typeof data === 'object' && data.type === 'error') {{
        status('Error: ' + data.message);
      }} else {{
        chunks.push(new Uint8Array(data));
        received += (data.byteLength || 0);
        setPct(total ? Math.round((received / total) * 100) : 0);
      }}
    }});
    conn.send({{ type: 'request' }});
  }});
}})();
</script>
</body>
</html>"#,
        peer_js_placeholder = if peer_js.is_empty() {
            "null".to_string()
        } else {
            format!("'{peer_js}'")
        }
    );
    respond_html(request, 200, &html);
}

fn respond_html(request: Request, status: u16, body: &str) {
    let response = Response::from_string(body.to_string())
        .with_status_code(StatusCode(status))
        .with_header(
            Header::from_bytes("Content-Type", "text/html; charset=utf-8")
                .expect("valid header"),
        );
    let _ = request.respond(response);
}

fn percent_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8()
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| s.to_string())
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
        assert!(info
            .urls
            .iter()
            .any(|u| u.contains("127.0.0.1") && u.contains(&info.token)));
        assert_eq!(info.urls.len(), info.urls.iter().collect::<std::collections::HashSet<_>>().len());

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
