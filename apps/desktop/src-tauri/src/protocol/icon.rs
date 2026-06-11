use std::{
    collections::hash_map::DefaultHasher,
    fs::File,
    hash::{Hash, Hasher},
    io::Cursor,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::UNIX_EPOCH,
};

use icns::{IconFamily, PixelFormat};
use image::{ColorType, ImageEncoder, codecs::png::PngEncoder};
use litools_index::repository::AppRecord;
use lru::LruCache;
use tauri::http::{StatusCode, Uri};

use super::{empty_response, ok_response, percent_decode};
pub use litools_config::icon::{DEFAULT_ICON_CACHE_CAPACITY, DEFAULT_ICON_TARGET_SIZE};

type IconResponse = tauri::http::Response<Vec<u8>>;

#[derive(Clone)]
pub struct IconProtocol {
    cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
}

impl Default for IconProtocol {
    fn default() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_ICON_CACHE_CAPACITY)
                    .expect("non-zero icon cache capacity"),
            ))),
        }
    }
}

impl IconProtocol {
    pub fn handle(&self, state: &crate::state::AppState, uri: &Uri) -> IconResponse {
        match self.icon_bytes(state, uri) {
            Ok(bytes) => ok_response(bytes, "image/png"),
            Err(status) => empty_response(status),
        }
    }

    fn icon_bytes(&self, state: &crate::state::AppState, uri: &Uri) -> Result<Vec<u8>, StatusCode> {
        let app_id = app_id_from_uri(uri).ok_or(StatusCode::BAD_REQUEST)?;
        let app = state
            .app()
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .find_app(&app_id)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
        let app_path = PathBuf::from(&app.path)
            .canonicalize()
            .map_err(|_| StatusCode::NOT_FOUND)?;
        let icon_path = validated_icon_path(&app);
        let cache_key = cache_key(&app.id, &app_path, icon_path.as_deref())
            .map_err(|_| StatusCode::NOT_FOUND)?;

        if let Some(bytes) = self.memory_cached(&cache_key) {
            return Ok(bytes);
        }

        let disk_cache_path = state
            .data_dir()
            .join("icon-cache")
            .join("apps")
            .join(format!("{cache_key}.png"));
        if let Ok(bytes) = std::fs::read(&disk_cache_path) {
            self.put_memory_cache(cache_key, bytes.clone());
            return Ok(bytes);
        }

        let bytes = app_icon_png(&app_path, icon_path.as_deref(), state)
            .map_err(|_| StatusCode::NOT_FOUND)?;
        let _ = write_disk_cache(&disk_cache_path, &bytes);
        self.put_memory_cache(cache_key, bytes.clone());
        Ok(bytes)
    }

    fn memory_cached(&self, cache_key: &str) -> Option<Vec<u8>> {
        self.cache
            .lock()
            .ok()
            .and_then(|mut cache| cache.get(cache_key).cloned())
    }

    fn put_memory_cache(&self, cache_key: String, bytes: Vec<u8>) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(cache_key, bytes);
        }
    }
}

fn app_id_from_uri(uri: &Uri) -> Option<String> {
    if uri.authority()?.as_str() != "app" {
        return None;
    }

    percent_decode(uri.path().trim_start_matches('/'))
}

fn validated_icon_path(app: &AppRecord) -> Option<PathBuf> {
    let icon_path = PathBuf::from(app.icon_path.as_ref()?);
    let resources_path = Path::new(&app.path).join("Contents/Resources");
    let icon_path = icon_path.canonicalize().ok()?;
    let resources_path = resources_path.canonicalize().ok()?;

    icon_path.starts_with(resources_path).then_some(icon_path)
}

fn cache_key(app_id: &str, app_path: &Path, icon_path: Option<&Path>) -> std::io::Result<String> {
    let modified_path = icon_path.unwrap_or(app_path);
    let metadata = std::fs::metadata(modified_path)?;
    let modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut hasher = DefaultHasher::new();
    app_id.hash(&mut hasher);
    app_path.hash(&mut hasher);
    icon_path.hash(&mut hasher);
    modified.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}

fn app_icon_png(
    app_path: &Path,
    icon_path: Option<&Path>,
    state: &crate::state::AppState,
) -> std::io::Result<Vec<u8>> {
    if let Some(icon_path) = icon_path
        && let Ok(bytes) = icns_to_png(icon_path)
    {
        return Ok(bytes);
    }

    state.app_icon_png(app_path)
}

fn icns_to_png(path: &Path) -> std::io::Result<Vec<u8>> {
    let family = IconFamily::read(File::open(path)?)?;
    let icon_type = family
        .available_icons()
        .into_iter()
        .min_by_key(|icon_type| {
            let size = icon_type.pixel_width();
            size.abs_diff(DEFAULT_ICON_TARGET_SIZE)
        })
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "missing icon image"))?;
    let icon = family.get_icon_with_type(icon_type)?;
    let rgba = rgba_bytes(&icon)?;
    let mut png = Vec::new();
    PngEncoder::new(Cursor::new(&mut png))
        .write_image(&rgba, icon.width(), icon.height(), ColorType::Rgba8.into())
        .map_err(std::io::Error::other)?;
    Ok(png)
}

fn rgba_bytes(icon: &icns::Image) -> std::io::Result<Vec<u8>> {
    let data = icon.data();
    match icon.pixel_format() {
        PixelFormat::RGBA => Ok(data.to_vec()),
        PixelFormat::RGB => Ok(data
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], u8::MAX])
            .collect()),
        PixelFormat::Gray => Ok(data
            .iter()
            .flat_map(|gray| [*gray, *gray, *gray, u8::MAX])
            .collect()),
        PixelFormat::GrayAlpha => Ok(data
            .chunks_exact(2)
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect()),
        PixelFormat::Alpha => Ok(data
            .iter()
            .flat_map(|alpha| [u8::MAX, u8::MAX, u8::MAX, *alpha])
            .collect()),
    }
}

fn write_disk_cache(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_app_id_from_uri() {
        let uri = "litools-icon://app/path%3A%2FApplications%2FMy%20App.app"
            .parse::<Uri>()
            .expect("uri");

        assert_eq!(
            app_id_from_uri(&uri),
            Some("path:/Applications/My App.app".to_string())
        );
    }

    #[test]
    fn decodes_bundle_identifier_from_uri() {
        let uri = "litools-icon://app/com.apple.ActivityMonitor"
            .parse::<Uri>()
            .expect("uri");

        assert_eq!(
            app_id_from_uri(&uri),
            Some("com.apple.ActivityMonitor".to_string())
        );
    }

    #[test]
    fn rejects_invalid_percent_encoding() {
        let uri = "litools-icon://app/%GG".parse::<Uri>().expect("uri");
        assert_eq!(app_id_from_uri(&uri), None);
    }

    #[test]
    fn rejects_unknown_icon_resource_type() {
        let uri = "litools-icon://file/com.apple.ActivityMonitor"
            .parse::<Uri>()
            .expect("uri");
        assert_eq!(app_id_from_uri(&uri), None);
    }
}
