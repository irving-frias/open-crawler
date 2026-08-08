//! Linux RFCOMM transport built directly on `AF_BLUETOOTH` sockets.
//!
//! This uses the same kernel interface as libbluetooth's `rfcomm` helpers, so
//! no extra native library is needed. Channel discovery is done by trying the
//! common OBEX Object Push channels ([`super::DEFAULT_CHANNELS`]).

use std::io::{Error as IoError, ErrorKind};
use std::mem::size_of;
use std::os::fd::FromRawFd;
use std::os::unix::io::AsRawFd;

use crate::error::AppError;

use super::ObexStream;

const AF_BLUETOOTH: libc::c_int = 31;
const BTPROTO_RFCOMM: libc::c_int = 3;

/// `struct sockaddr_rc` from `<net/bluetooth/bluetooth.h>`.
#[repr(C)]
#[derive(Clone, Copy)]
struct SockaddrRc {
    family: u16,
    bdaddr: [u8; 6],
    channel: u8,
    padding: [u8; 8],
}

pub struct LinuxRfcomm {
    fd: std::os::fd::OwnedFd,
}

/// Opens an RFCOMM connection to `addr` on `channel`, with bounded timeouts.
pub fn connect(addr: &str, channel: u8) -> Result<LinuxRfcomm, AppError> {
    let mac = parse_mac(addr)?;

    let fd = unsafe {
        libc::socket(
            AF_BLUETOOTH,
            libc::SOCK_STREAM | libc::SOCK_CLOEXEC,
            BTPROTO_RFCOMM,
        )
    };
    if fd < 0 {
        return Err(IoError::last_os_error().into());
    }
    let fd = unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) };

    let sockaddr = SockaddrRc {
        family: AF_BLUETOOTH as u16,
        bdaddr: mac,
        channel,
        padding: [0; 8],
    };
    connect_with_timeout(
        fd.as_raw_fd(),
        &sockaddr as *const SockaddrRc as *const libc::sockaddr,
        size_of::<SockaddrRc>(),
    )
    .map_err(|e| {
        AppError::Crawl(format!(
            "RFCOMM connect to {addr} channel {channel} failed: {e}"
        ))
    })?;

    // Bound read/write so a dead peer can't hang a transfer forever.
    set_socket_timeout(fd.as_raw_fd(), libc::SO_RCVTIMEO, 60)?;
    set_socket_timeout(fd.as_raw_fd(), libc::SO_SNDTIMEO, 60)?;

    Ok(LinuxRfcomm { fd })
}

/// Connects a (non-blocking) socket with a poll-based timeout.
fn connect_with_timeout(
    fd: libc::c_int,
    sockaddr: *const libc::sockaddr,
    len: usize,
) -> Result<(), IoError> {
    set_nonblocking(fd, true)?;

    let ret = unsafe { libc::connect(fd, sockaddr, len as libc::socklen_t) };
    if ret < 0 {
        let err = IoError::last_os_error();
        if err.raw_os_error() != Some(libc::EINPROGRESS) {
            set_nonblocking(fd, false)?;
            return Err(err);
        }
    } else {
        set_nonblocking(fd, false)?;
        return Ok(());
    }

    let mut pollfd = libc::pollfd {
        fd,
        events: libc::POLLOUT,
        revents: 0,
    };
    let n = unsafe { libc::poll(&mut pollfd, 1, 30_000) };
    set_nonblocking(fd, false)?;
    if n == 0 {
        return Err(IoError::new(ErrorKind::TimedOut, "connection timed out"));
    }
    if n < 0 {
        return Err(IoError::last_os_error());
    }

    // Retrieve the connect() result from SO_ERROR.
    let mut err: libc::c_int = 0;
    let mut err_len = size_of::<libc::c_int>() as libc::socklen_t;
    let gso = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_ERROR,
            &mut err as *mut libc::c_int as *mut libc::c_void,
            &mut err_len,
        )
    };
    if gso < 0 {
        return Err(IoError::last_os_error());
    }
    if err != 0 {
        return Err(IoError::from_raw_os_error(err));
    }
    Ok(())
}

fn set_nonblocking(fd: libc::c_int, nonblocking: bool) -> Result<(), IoError> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(IoError::last_os_error());
    }
    let flags = if nonblocking {
        flags | libc::O_NONBLOCK
    } else {
        flags & !libc::O_NONBLOCK
    };
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags) } < 0 {
        return Err(IoError::last_os_error());
    }
    Ok(())
}

fn set_socket_timeout(
    fd: libc::c_int,
    opt: libc::c_int,
    secs: libc::time_t,
) -> Result<(), AppError> {
    let tv = libc::timeval {
        tv_sec: secs,
        tv_usec: 0,
    };
    let ret = unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            opt,
            &tv as *const libc::timeval as *const libc::c_void,
            size_of::<libc::timeval>() as libc::socklen_t,
        )
    };
    if ret < 0 {
        return Err(IoError::last_os_error().into());
    }
    Ok(())
}

fn parse_mac(addr: &str) -> Result<[u8; 6], AppError> {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() != 6 {
        return Err(AppError::Crawl(format!(
            "invalid Bluetooth address (expected AA:BB:CC:DD:EE:FF): {addr}"
        )));
    }
    let mut mac = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        mac[i] = u8::from_str_radix(part, 16)
            .map_err(|_| AppError::Crawl(format!("invalid Bluetooth address: {addr}")))?;
    }
    Ok(mac)
}

impl ObexStream for LinuxRfcomm {
    fn write_all(&mut self, data: &[u8]) -> Result<(), AppError> {
        let fd = self.fd.as_raw_fd();
        let mut written = 0usize;
        while written < data.len() {
            let n = unsafe {
                libc::write(
                    fd,
                    data[written..].as_ptr() as *const libc::c_void,
                    data.len() - written,
                )
            };
            if n < 0 {
                let err = IoError::last_os_error();
                if err.kind() == ErrorKind::Interrupted {
                    continue;
                }
                if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut {
                    return Err(AppError::Crawl("RFCOMM write timed out".into()));
                }
                return Err(AppError::Crawl(format!("RFCOMM write failed: {err}")));
            }
            if n == 0 {
                return Err(AppError::Crawl(
                    "RFCOMM connection closed during write".into(),
                ));
            }
            written += n as usize;
        }
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), AppError> {
        let fd = self.fd.as_raw_fd();
        let mut read = 0usize;
        while read < buf.len() {
            let n = unsafe {
                libc::read(
                    fd,
                    buf[read..].as_mut_ptr() as *mut libc::c_void,
                    buf.len() - read,
                )
            };
            if n < 0 {
                let err = IoError::last_os_error();
                if err.kind() == ErrorKind::Interrupted {
                    continue;
                }
                if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut {
                    return Err(AppError::Crawl("RFCOMM read timed out".into()));
                }
                return Err(AppError::Crawl(format!("RFCOMM read failed: {err}")));
            }
            if n == 0 {
                return Err(AppError::Crawl("RFCOMM connection closed".into()));
            }
            read += n as usize;
        }
        Ok(())
    }
}
