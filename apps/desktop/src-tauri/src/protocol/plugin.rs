use std::path::PathBuf;

use tauri::http::{StatusCode, Uri};

use crate::state::AppState;

use litools_plugin::PluginManifest;

use super::{empty_response, ok_response, percent_decode};
pub use litools_config::protocol::PLUGIN_PROTOCOL_SCHEME;

type PluginResponse = tauri::http::Response<Vec<u8>>;

#[derive(Clone, Default)]
pub struct PluginProtocol;

impl PluginProtocol {
    pub fn handle(&self, state: &AppState, uri: &Uri) -> PluginResponse {
        match plugin_asset_bytes(state, uri) {
            Ok((bytes, content_type)) => ok_response(bytes, content_type),
            Err(status) => empty_response(status),
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

/// 根据 manifest 的 `development` 字段决定插件入口 URL。
///
/// - 有 `development.main` → 直接返回 dev server URL
/// - 无 `development` → 返回 `litools-plugin://` 生产路径
pub fn resolve_entry_url(plugin_id: &str, manifest: &PluginManifest) -> Result<String, String> {
    if let Some(ref dev) = manifest.development {
        return Ok(dev.main.clone());
    }
    let Some(entry) = &manifest.entry else {
        return Err(format!(
            "plugin {plugin_id} has no entry and no development server configured"
        ));
    };
    plugin_entry_url(plugin_id, entry)
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

    #[test]
    fn resolve_entry_url_uses_development_when_present() {
        let manifest = PluginManifest {
            id: "dev.litools.test".to_string(),
            name: "Test".to_string(),
            version: "0.1.0".to_string(),
            entry: Some("dist/index.html".to_string()),
            description: None,
            author: None,
            icon: "dist/icon.svg".to_string(),
            commands: vec![],
            singleton: true,
            permissions: vec![],
            development: Some(litools_plugin::PluginDevelopment {
                main: "http://127.0.0.1:5173/index.html".to_string(),
            }),
        };
        let url = resolve_entry_url("dev.litools.test", &manifest).expect("resolve");
        assert_eq!(url, "http://127.0.0.1:5173/index.html");
    }

    #[test]
    fn resolve_entry_url_falls_back_to_production() {
        let manifest = PluginManifest {
            id: "dev.litools.test".to_string(),
            name: "Test".to_string(),
            version: "0.1.0".to_string(),
            entry: Some("dist/index.html".to_string()),
            description: None,
            author: None,
            icon: "dist/icon.svg".to_string(),
            commands: vec![],
            singleton: true,
            permissions: vec![],
            development: None,
        };
        let url = resolve_entry_url("dev.litools.test", &manifest).expect("resolve");
        assert_eq!(url, "litools-plugin://dev.litools.test/dist/index.html");
    }

    #[test]
    fn resolve_entry_url_errors_without_entry_or_development() {
        let manifest = PluginManifest {
            id: "dev.litools.test".to_string(),
            name: "Test".to_string(),
            version: "0.1.0".to_string(),
            entry: None,
            description: None,
            author: None,
            icon: "dist/icon.svg".to_string(),
            commands: vec![],
            singleton: true,
            permissions: vec![],
            development: None,
        };
        let result = resolve_entry_url("dev.litools.test", &manifest);
        assert!(result.is_err());
    }
}
