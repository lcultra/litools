use std::path::PathBuf;

use tauri::http::{Response, StatusCode, Uri};

use crate::state::AppState;

pub const PLUGIN_PROTOCOL_SCHEME: &str = "litools-plugin";

type PluginResponse = Response<Vec<u8>>;

#[derive(Clone, Default)]
pub struct PluginProtocol;

impl PluginProtocol {
    pub fn handle(&self, state: &AppState, uri: &Uri) -> PluginResponse {
        match plugin_asset_bytes(state, uri) {
            Ok((bytes, content_type)) => response(StatusCode::OK, bytes, Some(content_type)),
            Err(status) => response(status, Vec::new(), None),
        }
    }
}

pub fn plugin_asset_url(plugin_id: &str, asset_path: &str) -> String {
    format!("{PLUGIN_PROTOCOL_SCHEME}://{plugin_id}/{asset_path}")
}

/// Builds the asset URL for a plugin entry file, rejecting absolute paths and
/// parent-directory traversal so a manifest cannot point outside its own root.
pub fn plugin_entry_url(plugin_id: &str, entry: &str) -> Result<String, String> {
    let entry_path = std::path::Path::new(entry);
    if entry_path.is_absolute()
        || entry_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!("invalid plugin entry: {entry}"));
    }

    Ok(plugin_asset_url(plugin_id, entry))
}

fn plugin_asset_bytes(state: &AppState, uri: &Uri) -> Result<(Vec<u8>, &'static str), StatusCode> {
    let plugin_id = uri.authority().ok_or(StatusCode::BAD_REQUEST)?.as_str();
    let asset_path = percent_decode(uri.path().trim_start_matches('/'))
        .filter(|path| !path.trim().is_empty())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let requested_path = PathBuf::from(&asset_path);
    if requested_path.is_absolute()
        || requested_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let plugin_root = {
        let app = state
            .app()
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let plugin = app
            .context()
            .plugins
            .find_plugin(plugin_id)
            .ok_or(StatusCode::NOT_FOUND)?;
        if !plugin.enabled {
            return Err(StatusCode::FORBIDDEN);
        }
        plugin.path.clone()
    };

    let root = plugin_root
        .canonicalize()
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let asset = plugin_root
        .join(&requested_path)
        .canonicalize()
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if !asset.starts_with(&root) {
        return Err(StatusCode::FORBIDDEN);
    }

    let bytes = std::fs::read(&asset).map_err(|_| StatusCode::NOT_FOUND)?;
    let content_type = content_type_for_path(&asset).ok_or(StatusCode::NOT_FOUND)?;
    Ok((bytes, content_type))
}

fn content_type_for_path(path: &std::path::Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "html" => Some("text/html; charset=utf-8"),
        "js" | "mjs" => Some("text/javascript; charset=utf-8"),
        "css" => Some("text/css; charset=utf-8"),
        "json" => Some("application/json"),
        "svg" => Some("image/svg+xml"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

fn percent_decode(value: &str) -> Option<String> {
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

fn response(status: StatusCode, body: Vec<u8>, content_type: Option<&str>) -> PluginResponse {
    let mut builder = Response::builder().status(status);
    if let Some(content_type) = content_type {
        builder = builder.header("content-type", content_type);
    }
    builder.body(body).expect("valid plugin protocol response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_plugin_asset_url() {
        assert_eq!(
            plugin_asset_url("dev.litools.hello-world", "dist/index.html"),
            "litools-plugin://dev.litools.hello-world/dist/index.html"
        );
    }

    #[test]
    fn decodes_percent_encoded_path() {
        assert_eq!(
            percent_decode("dist/My%20Page.html"),
            Some("dist/My Page.html".to_string())
        );
    }

    #[test]
    fn rejects_invalid_percent_encoding() {
        assert_eq!(percent_decode("dist/%GG.html"), None);
    }

    #[test]
    fn returns_known_content_types() {
        assert_eq!(
            content_type_for_path(std::path::Path::new("index.html")),
            Some("text/html; charset=utf-8")
        );
        assert_eq!(
            content_type_for_path(std::path::Path::new("app.css")),
            Some("text/css; charset=utf-8")
        );
    }
}
