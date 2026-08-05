//! Windows RFCOMM transport via the WinRT `Windows.Devices.Bluetooth.Rfcomm`
//! API. Service discovery (SDP) is handled by the OS, so no channel guessing
//! is needed — the OBEX Object Push service is looked up directly.
//!
//! All WinRT objects are confined to the calling thread (they are created and
//! used inside one blocking transfer), and the thread is RoInitialized for the
//! lifetime of the stream.

use std::time::{Duration, Instant};

use windows::Devices::Bluetooth::BluetoothDevice;
use windows::Devices::Bluetooth::Rfcomm::BluetoothRfcommServiceId;
use windows::Foundation::{AsyncStatus, IAsyncOperation};
use windows::Networking::Sockets::StreamSocket;
use windows::Storage::Streams::{DataReader, DataWriter, IInputStream, IOutputStream};
use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};

use crate::error::AppError;

use super::ObexStream;

pub struct WindowsRfcomm {
    reader: DataReader,
    writer: DataWriter,
    initialized: bool,
}

impl Drop for WindowsRfcomm {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { RoUninitialize() };
        }
    }
}

/// Opens an RFCOMM connection to `addr` (e.g. `"AA:BB:CC:DD:EE:FF"`) and binds
/// the OBEX Object Push service.
pub fn connect(addr: &str) -> Result<WindowsRfcomm, AppError> {
    // Must run on an RoInitialized thread (MTA) for WinRT Bluetooth.
    unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(win_err)?;

    let address = parse_address(addr)?;
    let device = await_op(
        &BluetoothDevice::FromBluetoothAddressAsync(address).map_err(win_err)?,
        Duration::from_secs(20),
    )?;

    let service_id = BluetoothRfcommServiceId::ObexObjectPush().map_err(win_err)?;
    let services_result = await_op(
        &device.GetRfcommServicesForIdAsync(&service_id).map_err(win_err)?,
        Duration::from_secs(20),
    )?;
    let services = services_result.Services().map_err(win_err)?;
    let iterator = services.First().map_err(win_err)?;
    if !iterator.HasCurrent().map_err(win_err)? {
        return Err(AppError::Crawl(
            "device has no OBEX Object Push service".into(),
        ));
    }
    let service = iterator.Current().map_err(win_err)?;

    let host = service.ConnectionHostName().map_err(win_err)?;
    let service_name = service.ConnectionServiceName().map_err(win_err)?;

    let socket = StreamSocket::new().map_err(win_err)?;
    await_action(
        &socket.ConnectAsync(&host, &service_name).map_err(win_err)?,
        Duration::from_secs(30),
    )?;

    let output: IOutputStream = socket.OutputStream().map_err(win_err)?;
    let input: IInputStream = socket.InputStream().map_err(win_err)?;
    let writer = DataWriter::new(&output).map_err(win_err)?;
    let reader = DataReader::new(&input).map_err(win_err)?;

    Ok(WindowsRfcomm {
        reader,
        writer,
        initialized: true,
    })
}

impl ObexStream for WindowsRfcomm {
    fn write_all(&mut self, data: &[u8]) -> Result<(), AppError> {
        self.writer.WriteBytes(data).map_err(win_err)?;
        await_op(&self.writer.StoreAsync().map_err(win_err)?, Duration::from_secs(30))?;
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), AppError> {
        let mut offset = 0usize;
        while offset < buf.len() {
            let loaded = await_op(
                &self.reader.LoadAsync((buf.len() - offset) as u32).map_err(win_err)?,
                Duration::from_secs(60),
            )? as usize;
            if loaded == 0 {
                return Err(AppError::Crawl("RFCOMM connection closed".into()));
            }
            self.reader.ReadBytes(&mut buf[offset..offset + loaded]).map_err(win_err)?;
            offset += loaded;
        }
        Ok(())
    }
}

fn parse_address(addr: &str) -> Result<u64, AppError> {
    let hex = addr.replace([':', '-'], "");
    if hex.len() != 12 {
        return Err(AppError::Crawl(format!(
            "invalid Bluetooth address (expected AA:BB:CC:DD:EE:FF): {addr}"
        )));
    }
    u64::from_str_radix(&hex, 16)
        .map_err(|_| AppError::Crawl(format!("invalid Bluetooth address: {addr}")))
}

fn win_err(e: windows::core::Error) -> AppError {
    AppError::Crawl(format!("Windows Bluetooth error: {e}"))
}

/// Blocks on a WinRT `IAsyncAction` until completion or `timeout`.
fn await_action(op: &windows::Foundation::IAsyncAction, timeout: Duration) -> Result<(), AppError> {
    let deadline = Instant::now() + timeout;
    loop {
        let status = op.Status().map_err(win_err)?;
        match status {
            AsyncStatus::Completed => return Ok(()),
            AsyncStatus::Error | AsyncStatus::Canceled => {
                return Err(AppError::Crawl("WinRT operation failed".into()));
            }
            _ => {}
        }
        if Instant::now() >= deadline {
            return Err(AppError::Crawl("WinRT operation timed out".into()));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Blocks on a WinRT `IAsyncOperation<T>` until completion or `timeout`.
fn await_op<T>(
    op: &IAsyncOperation<T>,
    timeout: Duration,
) -> Result<T, AppError> {
    let deadline = Instant::now() + timeout;
    loop {
        let status = op.Status().map_err(win_err)?;
        match status {
            AsyncStatus::Completed => return op.GetResults().map_err(win_err),
            AsyncStatus::Error | AsyncStatus::Canceled => {
                return Err(AppError::Crawl("WinRT operation failed".into()));
            }
            _ => {}
        }
        if Instant::now() >= deadline {
            return Err(AppError::Crawl("WinRT operation timed out".into()));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}
