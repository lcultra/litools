pub mod icon;
pub mod icon_disk_cache;
pub mod plugin;

use tauri::http::{Response, StatusCode};

pub use icon::IconProtocol;
pub use plugin::PluginProtocol;

pub(crate) fn percent_decode(value: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(value.len());
    let mut input = value.as_bytes().iter().copied();

    while let Some(byte) = input.next() {
        if byte != b'%' {
            bytes.push(byte);
            continue;
        }

        let high = input.next()?;
        let low = input.next()?;
        bytes.push(hex_value(high)? << 4 | hex_value(low)?);
    }

    String::from_utf8(bytes).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn ok_response(body: Vec<u8>, content_type: &'static str) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", content_type)
        .body(body)
        .expect("valid response")
}

pub(crate) fn empty_response(status: StatusCode) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .body(Vec::new())
        .expect("valid empty response")
}
