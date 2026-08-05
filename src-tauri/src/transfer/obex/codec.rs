//! OBEX (OBject EXchange, IrOBEX 1.5) client packet codec.
//!
//! Pure encoding/decoding logic — no I/O. Used by the RFCOMM transports in
//! [`super::send_file`] to implement the OBEX Object Push profile.

/// CONNECT (client request opcode).
pub const OP_CONNECT: u8 = 0x80;
/// DISCONNECT (client request opcode).
pub const OP_DISCONNECT: u8 = 0x81;
/// PUT, not the final packet (client request opcode).
pub const OP_PUT: u8 = 0x02;
/// PUT, final packet (client request opcode).
pub const OP_PUT_FINAL: u8 = 0x82;

/// Continue (server response to a non-final PUT body chunk).
pub const RESP_CONTINUE: u8 = 0x90;
/// Success (server response to CONNECT/DISCONNECT/final PUT).
pub const RESP_SUCCESS: u8 = 0xa0;
/// Success (DISCONNECT).
pub const RESP_DISCONNECT_OK: u8 = 0xa1;

/// `Name` header (null-terminated UTF-16BE, 2-byte length).
pub const HDR_NAME: u8 = 0x01;
/// `Target` header (byte sequence, 2-byte length).
pub const HDR_TARGET: u8 = 0x46;
/// `Body` header (byte sequence, 2-byte length).
pub const HDR_BODY: u8 = 0x48;
/// `End-of-Body` header (byte sequence, 2-byte length).
pub const HDR_BODY_END: u8 = 0x49;
/// `Length` header (4-byte quantity, no length field).
pub const HDR_LENGTH: u8 = 0xc3;
/// `Connection ID` header (4-byte quantity, no length field).
pub const HDR_CONNECTION: u8 = 0xcb;

/// The OBEX Object Push Profile target UUID.
pub const OBJECT_PUSH_TARGET: [u8; 16] = [
    0x79, 0x00, 0x00, 0x11, 0xd0, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// A single OBEX header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Header {
    /// Raw bytes with the given header id.
    Bytes(u8, Vec<u8>),
    /// Numeric header (1-byte or 4-byte quantity) with the given header id.
    U32(u8, u32),
    /// Decoded `Name` header.
    Name(String),
}

/// A parsed OBEX response packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub opcode: u8,
    pub headers: Vec<Header>,
}

impl Response {
    pub fn is_success(&self) -> bool {
        self.opcode == RESP_SUCCESS || self.opcode == RESP_DISCONNECT_OK
    }

    pub fn is_continue(&self) -> bool {
        self.opcode == RESP_CONTINUE
    }

    /// Human readable description of a non-success response code.
    pub fn status_text(&self) -> &'static str {
        match self.opcode {
            RESP_CONTINUE => "continue",
            RESP_SUCCESS => "success",
            0x40 => "bad request",
            0x41 => "unauthorized",
            0x42 => "bad request",
            0x43 => "forbidden",
            0x44 => "not found",
            0x48 => "request accepted for processing",
            0x49 => "conflict",
            0x50 => "service unavailable",
            _ => "unknown response",
        }
    }

    /// Extracts the connection ID (if any) from the response headers.
    pub fn connection_id(&self) -> Option<u32> {
        self.headers.iter().find_map(|h| match h {
            Header::U32(HDR_CONNECTION, v) => Some(*v),
            _ => None,
        })
    }
}

/// Builder for outgoing packets.
pub struct PacketBuilder {
    buffer: Vec<u8>,
}

impl PacketBuilder {
    /// Starts a new packet with the given opcode.
    pub fn new(opcode: u8) -> Self {
        let mut buffer = vec![0u8; 3];
        buffer[0] = opcode;
        PacketBuilder { buffer }
    }

    /// Pushes a byte-sequence header (2-byte length field).
    fn push_bytes(&mut self, id: u8, data: &[u8]) {
        let len = 3 + data.len();
        assert!(len <= u16::MAX as usize, "OBEX header too large");
        self.buffer.push(id);
        self.buffer.extend_from_slice(&(len as u16).to_be_bytes());
        self.buffer.extend_from_slice(data);
    }

    /// Pushes a numeric header (4-byte quantity, no length field).
    fn push_u32(&mut self, id: u8, value: u32) {
        self.buffer.push(id);
        self.buffer.extend_from_slice(&value.to_be_bytes());
    }

    /// Pushes a `Name` header, encoded as null-terminated UTF-16BE.
    pub fn push_name(&mut self, name: &str) {
        let mut encoded = Vec::with_capacity(name.len() * 2 + 2);
        for unit in name.encode_utf16() {
            encoded.extend_from_slice(&unit.to_be_bytes());
        }
        encoded.extend_from_slice(&[0, 0]);
        self.push_bytes(HDR_NAME, &encoded);
    }

    /// Finalizes the packet: fills in the packet length field.
    pub fn build(mut self) -> Vec<u8> {
        let len = self.buffer.len();
        assert!(len <= u16::MAX as usize, "OBEX packet too large");
        let len = len as u16;
        self.buffer[1..3].copy_from_slice(&len.to_be_bytes());
        self.buffer
    }
}

/// Builds a CONNECT request with the Object Push target.
///
/// `max_packet` is the largest OBEX packet this client is willing to receive.
pub fn connect_request(max_packet: u16) -> Vec<u8> {
    let mut p = PacketBuilder::new(OP_CONNECT);
    p.push_bytes(HDR_TARGET, &OBJECT_PUSH_TARGET);
    // The CONNECT body starts with version, flags and the max packet length.
    // Insert them right after the 3-byte packet header.
    let mut buf = p.build();
    buf.splice(3..3, [0x10, 0x00, (max_packet >> 8) as u8, max_packet as u8]);
    let len = buf.len() as u16;
    buf[1..3].copy_from_slice(&len.to_be_bytes());
    buf
}

/// Builds a PUT (or PUT-Final) request.
///
/// `name`/`length` are only sent on the first body packet of a transfer.
/// `connection_id`, if any, is included in every packet.
pub fn put_request(
    name: Option<&str>,
    length: Option<u32>,
    body: &[u8],
    final_packet: bool,
    connection_id: Option<u32>,
) -> Vec<u8> {
    let mut p = PacketBuilder::new(if final_packet { OP_PUT_FINAL } else { OP_PUT });
    if let Some(name) = name {
        p.push_name(name);
    }
    if let Some(length) = length {
        p.push_u32(HDR_LENGTH, length);
    }
    if let Some(id) = connection_id {
        p.push_u32(HDR_CONNECTION, id);
    }
    p.push_bytes(if final_packet { HDR_BODY_END } else { HDR_BODY }, body);
    p.build()
}

/// Builds a DISCONNECT request.
pub fn disconnect_request(connection_id: Option<u32>) -> Vec<u8> {
    let mut p = PacketBuilder::new(OP_DISCONNECT);
    if let Some(id) = connection_id {
        p.push_u32(HDR_CONNECTION, id);
    }
    p.build()
}

/// Parses an OBEX response packet from its first `len` bytes.
///
/// Returns an error if the buffer is shorter than the declared packet length
/// or the declared length is unreasonably small.
pub fn parse_response(buf: &[u8]) -> Result<Response, crate::error::AppError> {
    if buf.len() < 3 {
        return Err(crate::error::AppError::Crawl("short OBEX response".into()));
    }
    let opcode = buf[0];
    let len = u16::from_be_bytes([buf[1], buf[2]]) as usize;
    if len < 3 {
        return Err(crate::error::AppError::Crawl(
            "malformed OBEX response length".into(),
        ));
    }
    if buf.len() < len {
        return Err(crate::error::AppError::Crawl(format!(
            "truncated OBEX response (got {}, need {len})",
            buf.len()
        )));
    }
    let data = &buf[3..len];
    let mut headers = Vec::new();
    let mut i = 0;
    let mut iterations = 0;
    while i < data.len() {
        iterations += 1;
        if iterations > 64 {
            return Err(crate::error::AppError::Crawl(
                "OBEX header loop overrun".into(),
            ));
        }
        let id = data[i];
        let (header, consumed) = match id >> 6 {
            // Unicode string, 2-byte length.
            0b00 => {
                let hlen = read_two_byte_len(data, i)?;
                let value = data[i + 3..i + hlen].to_vec();
                let header = if id == HDR_NAME {
                    Header::Name(decode_unicode(&value))
                } else {
                    Header::Bytes(id, value)
                };
                (header, hlen)
            }
            // Byte sequence, 2-byte length.
            0b01 => {
                let hlen = read_two_byte_len(data, i)?;
                (Header::Bytes(id, data[i + 3..i + hlen].to_vec()), hlen)
            }
            // 1-byte quantity.
            0b10 => {
                if i + 2 > data.len() {
                    return Err(crate::error::AppError::Crawl(
                        "truncated OBEX 1-byte header".into(),
                    ));
                }
                (Header::U32(id, data[i + 1] as u32), 2)
            }
            // 4-byte quantity.
            0b11 => {
                if i + 5 > data.len() {
                    return Err(crate::error::AppError::Crawl(
                        "truncated OBEX 4-byte header".into(),
                    ));
                }
                let value = u32::from_be_bytes(data[i + 1..i + 5].try_into().unwrap());
                (Header::U32(id, value), 5)
            }
            _ => {
                return Err(crate::error::AppError::Crawl(
                    "unsupported OBEX header".into(),
                ));
            }
        };
        headers.push(header);
        i += consumed;
    }
    Ok(Response { opcode, headers })
}

fn read_two_byte_len(data: &[u8], i: usize) -> Result<usize, crate::error::AppError> {
    if i + 3 > data.len() {
        return Err(crate::error::AppError::Crawl(
            "truncated OBEX header".into(),
        ));
    }
    let hlen = u16::from_be_bytes([data[i + 1], data[i + 2]]) as usize;
    if hlen < 3 || i + hlen > data.len() {
        return Err(crate::error::AppError::Crawl(
            "bad OBEX header length".into(),
        ));
    }
    Ok(hlen)
}

fn decode_unicode(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .filter_map(|c| {
            let unit = u16::from_be_bytes([c[0], c[1]]);
            (unit != 0).then_some(unit)
        })
        .collect();
    String::from_utf16_lossy(&units)
}

/// Reads a full OBEX packet from a blocking stream, returning the raw bytes.
///
/// `read_exact` must read exactly the requested number of bytes.
pub fn read_packet(
    read_exact: &mut dyn FnMut(&mut [u8]) -> Result<(), crate::error::AppError>,
) -> Result<Vec<u8>, crate::error::AppError> {
    let mut header = [0u8; 3];
    read_exact(&mut header)?;
    let len = u16::from_be_bytes([header[1], header[2]]) as usize;
    if len < 3 {
        return Err(crate::error::AppError::Crawl("malformed OBEX packet length".into()));
    }
    let mut rest = vec![0u8; len - 3];
    read_exact(&mut rest)?;
    let mut buf = header.to_vec();
    buf.extend_from_slice(&rest);
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_request_has_expected_shape() {
        let p = connect_request(0x1000);
        assert_eq!(p[0], OP_CONNECT);
        assert_eq!(p[1..3], [0x00, 0x1a]); // 26 bytes total
        assert_eq!(p[3], 0x10); // version
        assert_eq!(p[4], 0x00); // flags
        assert_eq!(p[5..7], [0x10, 0x00]); // max packet length
        assert_eq!(p[7], HDR_TARGET);
        assert_eq!(p[8..10], [0x00, 0x13]); // header length 19
        assert_eq!(&p[10..], &OBJECT_PUSH_TARGET[..]);
    }

    #[test]
    fn put_request_roundtrip() {
        let p = put_request(Some("hello.txt"), Some(5), b"he", false, Some(7));
        assert_eq!(p[0], OP_PUT);
        let resp = parse_response(&p).unwrap();
        assert_eq!(resp.opcode, OP_PUT);
        // Name + Length + ConnectionId + Body
        assert!(resp.headers.contains(&Header::Name("hello.txt".into())));
        assert!(resp.headers.contains(&Header::U32(HDR_LENGTH, 5)));
        assert!(resp.headers.contains(&Header::U32(HDR_CONNECTION, 7)));
        assert!(resp.headers.contains(&Header::Bytes(HDR_BODY, b"he".to_vec())));
    }

    #[test]
    fn final_put_uses_end_of_body() {
        let p = put_request(None, None, b"x", true, None);
        assert_eq!(p[0], OP_PUT_FINAL);
        let resp = parse_response(&p).unwrap();
        assert!(resp.headers.contains(&Header::Bytes(HDR_BODY_END, b"x".to_vec())));
    }

    #[test]
    fn parses_success_response_with_connection_id() {
        let mut buf = vec![RESP_SUCCESS, 0x00, 0x08, HDR_CONNECTION];
        buf.extend_from_slice(&42u32.to_be_bytes());
        let resp = parse_response(&buf).unwrap();
        assert!(resp.is_success());
        assert_eq!(resp.connection_id(), Some(42));
    }

    #[test]
    fn parses_continue_response() {
        let buf = [RESP_CONTINUE, 0x00, 0x03];
        let resp = parse_response(&buf).unwrap();
        assert!(resp.is_continue());
        assert!(!resp.is_success());
    }

    #[test]
    fn parses_one_byte_quantity_header() {
        // 1-byte quantity: id 0x80 followed by a single value byte.
        let buf = [RESP_SUCCESS, 0x00, 0x05, 0x80, 0xab];
        let resp = parse_response(&buf).unwrap();
        assert_eq!(resp.headers, vec![Header::U32(0x80, 0xab)]);
    }

    #[test]
    fn parses_byte_sequence_header() {
        // Target header (byte sequence) echoed in a response.
        let mut buf = vec![RESP_SUCCESS, 0x00, 0x00, HDR_TARGET];
        buf.extend_from_slice(&[0x00, 0x13]);
        buf.extend_from_slice(&OBJECT_PUSH_TARGET);
        buf[1] = 0x00;
        buf[2] = (buf.len()) as u8;
        let resp = parse_response(&buf).unwrap();
        assert_eq!(resp.headers, vec![Header::Bytes(HDR_TARGET, OBJECT_PUSH_TARGET.to_vec())]);
    }

    #[test]
    fn rejects_truncated_packet() {
        let buf = [RESP_SUCCESS, 0x00, 0x20, 0x01];
        assert!(parse_response(&buf).is_err());
    }

    #[test]
    fn disconnect_request_has_connection_id() {
        let p = disconnect_request(Some(9));
        assert_eq!(p[0], OP_DISCONNECT);
        let resp = parse_response(&p).unwrap();
        assert_eq!(resp.connection_id(), Some(9));
    }
}
