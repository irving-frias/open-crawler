//! OBEX Object Push Bluetooth file transfer over RFCOMM.
//!
//! [`send_file`] is a blocking function: it opens an RFCOMM connection to a
//! Bluetooth device, runs an OBEX CONNECT/PUT/DISCONNECT exchange and streams
//! the file in chunks. Callers should run it on a blocking thread.
//!
//! The OBEX packet codec lives in [`codec`] (platform independent and unit
//! tested). The transports are behind `cfg`:
//!   - Linux: raw `AF_BLUETOOTH`/`BTPROTO_RFCOMM` sockets.
//!   - Windows: WinRT `Windows.Devices.Bluetooth.Rfcomm`.
//!   - Other platforms: unsupported (system share sheet instead).
//!
//! RFCOMM channel discovery uses a fallback list of the common OBEX Object
//! Push channels; proper SDP service discovery is a known limitation (tracked
//! in PROGRESS.md).

pub mod codec;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

use std::io::Read;
use std::path::Path;

use crate::error::AppError;

/// Common OBEX Object Push RFCOMM channels, tried in order until one works.
pub const DEFAULT_CHANNELS: &[u8] = &[9, 10, 12, 11, 5, 7, 8];

/// OBEX body chunk size — comfortably below the 64 KiB OBEX packet limit
/// including header overhead.
const BODY_CHUNK: usize = 0x3c00;

/// Blocking, timeout-bounded Bluetooth RFCOMM stream.
pub trait ObexStream: Send {
    fn write_all(&mut self, data: &[u8]) -> Result<(), AppError>;
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), AppError>;
}

/// Opens an RFCOMM connection to `addr` (e.g. `"AA:BB:CC:DD:EE:FF"`).
#[cfg(any(target_os = "linux", target_os = "windows"))]
fn connect(addr: &str, channel: u8) -> Result<Box<dyn ObexStream>, AppError> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::connect(addr, channel)?))
    }
    #[cfg(target_os = "windows")]
    {
        let _ = channel;
        Ok(Box::new(windows::connect(addr)?))
    }
}

/// Sends a file to a Bluetooth device via OBEX Object Push.
///
/// `on_progress(received_by_server, total)` is called after every body chunk.
pub fn send_file(
    addr: &str,
    path: &Path,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<(), AppError> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("package.ocproj")
        .to_string();
    let total = std::fs::metadata(path)?.len();
    if total > u32::MAX as u64 {
        return Err(AppError::Crawl(
            "file is larger than the 4 GiB OBEX length limit".to_string(),
        ));
    }

    let mut last_err: Option<AppError> = None;
    for channel in DEFAULT_CHANNELS {
        let mut stream = match connect(addr, *channel) {
            Ok(stream) => stream,
            Err(e) => {
                last_err = Some(e);
                continue;
            }
        };
        match transfer_file(&mut *stream, &name, total, path, &mut on_progress) {
            Ok(()) => {
                let _ = disconnect(&mut *stream);
                return Ok(());
            }
            Err(e) => {
                last_err = Some(e);
                let _ = disconnect(&mut *stream);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| {
        AppError::Crawl("Bluetooth send failed on all RFCOMM channels".to_string())
    }))
}

fn transfer_file(
    stream: &mut dyn ObexStream,
    name: &str,
    total: u64,
    path: &Path,
    on_progress: &mut dyn FnMut(u64, u64),
) -> Result<(), AppError> {
    stream.write_all(&codec::connect_request(0xffff))?;
    let response = read_response(stream)?;
    if !response.is_success() {
        return Err(AppError::Crawl(format!(
            "OBEX CONNECT rejected ({})",
            response.status_text()
        )));
    }
    let connection_id = response.connection_id();

    let mut file = std::fs::File::open(path)?;
    let mut buffer = vec![0u8; BODY_CHUNK];
    let mut sent: u64 = 0;
    let mut first = true;
    loop {
        let n = file.read(&mut buffer)?;
        let final_packet = n < BODY_CHUNK;
        let body = &buffer[..n];
        let packet = if first {
            codec::put_request(
                Some(name),
                Some(total as u32),
                body,
                final_packet,
                connection_id,
            )
        } else {
            codec::put_request(None, None, body, final_packet, connection_id)
        };
        stream.write_all(&packet)?;

        let response = read_response(stream)?;
        if final_packet {
            if !response.is_success() {
                return Err(AppError::Crawl(format!(
                    "OBEX PUT failed ({})",
                    response.status_text()
                )));
            }
            sent += n as u64;
            on_progress(sent, total);
            return Ok(());
        }
        if !response.is_continue() {
            return Err(AppError::Crawl(format!(
                "OBEX PUT interrupted ({})",
                response.status_text()
            )));
        }
        sent += n as u64;
        on_progress(sent, total);
        first = false;
    }
}

fn read_response(stream: &mut dyn ObexStream) -> Result<codec::Response, AppError> {
    let packet = codec::read_packet(&mut |buf| stream.read_exact(buf))?;
    codec::parse_response(&packet)
}

/// Best-effort OBEX DISCONNECT; the socket is dropped either way.
fn disconnect(stream: &mut dyn ObexStream) -> Result<(), AppError> {
    let _ = stream.write_all(&codec::disconnect_request(None));
    let _ = codec::read_packet(&mut |buf| stream.read_exact(buf));
    Ok(())
}
