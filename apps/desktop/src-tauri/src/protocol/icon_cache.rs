use std::{fs, path::Path, time::SystemTime};

pub use litools_config::icon::{
    DEFAULT_ICON_CACHE_MAX_BYTES, DEFAULT_ICON_CACHE_MAX_FILES, ICON_CACHE_RELATIVE_DIR,
};
use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconCacheSummary {
    pub file_count: usize,
    pub total_bytes: u64,
    pub max_files: usize,
    pub max_bytes: u64,
    pub last_pruned_at: Option<String>,
    pub last_pruned_files: usize,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    path: std::path::PathBuf,
    modified: SystemTime,
    bytes: u64,
}

pub fn icon_cache_summary(data_dir: &Path) -> IconCacheSummary {
    match cache_entries(data_dir) {
        Ok(entries) => summary_from_entries(entries, None, 0, None),
        Err(error) => IconCacheSummary {
            max_files: DEFAULT_ICON_CACHE_MAX_FILES,
            max_bytes: DEFAULT_ICON_CACHE_MAX_BYTES,
            error: Some(error.to_string()),
            ..IconCacheSummary::default()
        },
    }
}

pub fn prune_icon_cache(data_dir: &Path) -> IconCacheSummary {
    let mut entries = match cache_entries(data_dir) {
        Ok(entries) => entries,
        Err(error) => {
            return IconCacheSummary {
                max_files: DEFAULT_ICON_CACHE_MAX_FILES,
                max_bytes: DEFAULT_ICON_CACHE_MAX_BYTES,
                error: Some(error.to_string()),
                ..IconCacheSummary::default()
            };
        }
    };

    entries.sort_by_key(|entry| entry.modified);
    let mut total_bytes = entries.iter().map(|entry| entry.bytes).sum::<u64>();
    let mut remove_count = entries.len().saturating_sub(DEFAULT_ICON_CACHE_MAX_FILES);
    while total_bytes > DEFAULT_ICON_CACHE_MAX_BYTES && remove_count < entries.len() {
        total_bytes = total_bytes.saturating_sub(entries[remove_count].bytes);
        remove_count += 1;
    }

    let mut removed = 0;
    let mut error = None;
    for entry in entries.iter().take(remove_count) {
        match fs::remove_file(&entry.path) {
            Ok(()) => removed += 1,
            Err(remove_error) => error = Some(remove_error.to_string()),
        }
    }

    match cache_entries(data_dir) {
        Ok(entries) => summary_from_entries(
            entries,
            Some(chrono::Utc::now().to_rfc3339()),
            removed,
            error,
        ),
        Err(read_error) => IconCacheSummary {
            max_files: DEFAULT_ICON_CACHE_MAX_FILES,
            max_bytes: DEFAULT_ICON_CACHE_MAX_BYTES,
            last_pruned_at: Some(chrono::Utc::now().to_rfc3339()),
            last_pruned_files: removed,
            error: Some(read_error.to_string()),
            ..IconCacheSummary::default()
        },
    }
}

fn icon_cache_dir(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(ICON_CACHE_RELATIVE_DIR)
}

fn cache_entries(data_dir: &Path) -> std::io::Result<Vec<CacheEntry>> {
    let dir = icon_cache_dir(data_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("png") {
            continue;
        }

        let metadata = entry.metadata()?;
        if !metadata.is_file() {
            continue;
        }

        entries.push(CacheEntry {
            path,
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            bytes: metadata.len(),
        });
    }
    Ok(entries)
}

fn summary_from_entries(
    entries: Vec<CacheEntry>,
    last_pruned_at: Option<String>,
    last_pruned_files: usize,
    error: Option<String>,
) -> IconCacheSummary {
    IconCacheSummary {
        file_count: entries.len(),
        total_bytes: entries.iter().map(|entry| entry.bytes).sum(),
        max_files: DEFAULT_ICON_CACHE_MAX_FILES,
        max_bytes: DEFAULT_ICON_CACHE_MAX_BYTES,
        last_pruned_at,
        last_pruned_files,
        error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_cache_directory_has_empty_summary() {
        let dir = std::env::temp_dir().join(format!(
            "litools-missing-icon-cache-{}",
            uuid::Uuid::new_v4()
        ));

        let summary = icon_cache_summary(&dir);

        assert_eq!(summary.file_count, 0);
        assert_eq!(summary.total_bytes, 0);
        assert_eq!(summary.max_files, DEFAULT_ICON_CACHE_MAX_FILES);
    }
}
