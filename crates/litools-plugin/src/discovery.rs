use std::{fs, path::PathBuf};

use thiserror::Error;

use crate::{
    manager::PluginSource,
    manifest::{PluginManifest, PluginManifestError},
};
use litools_config::plugin::PLUGIN_MANIFEST_FILE_NAME;

#[derive(Clone, Debug)]
pub struct PluginDiscoveryRoot {
    pub path: PathBuf,
    pub source: PluginSource,
}

#[derive(Clone, Debug)]
pub struct DiscoveredPlugin {
    pub manifest: PluginManifest,
    pub root_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub source: PluginSource,
}

#[derive(Debug, Error)]
pub enum PluginDiscoveryError {
    #[error("failed to read plugin directory {path}: {source}")]
    ReadDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read plugin manifest {path}: {source}")]
    ReadManifest {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse plugin manifest {path}: {source}")]
    ParseManifest {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid plugin manifest {path}: {source}")]
    InvalidManifest {
        path: PathBuf,
        #[source]
        source: PluginManifestError,
    },
    #[error("plugin entry escapes plugin directory {path}")]
    EntryEscapesRoot { path: PathBuf },
}

pub fn discover_plugins(
    roots: impl IntoIterator<Item = PluginDiscoveryRoot>,
) -> Vec<DiscoveredPlugin> {
    let mut plugins = Vec::new();

    for root in roots {
        match discover_plugins_in_root(root.path, root.source) {
            Ok(mut discovered) => plugins.append(&mut discovered),
            Err(error) => tracing_log(&error.to_string()),
        }
    }

    plugins.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    plugins
}

pub fn discover_plugins_in_root(
    root: PathBuf,
    source: PluginSource,
) -> Result<Vec<DiscoveredPlugin>, PluginDiscoveryError> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut manifest_paths = Vec::new();
    let direct_manifest = root.join(PLUGIN_MANIFEST_FILE_NAME);
    if direct_manifest.is_file() {
        manifest_paths.push(direct_manifest);
    }

    let entries = fs::read_dir(&root).map_err(|source| PluginDiscoveryError::ReadDirectory {
        path: root.clone(),
        source,
    })?;

    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join(PLUGIN_MANIFEST_FILE_NAME);
            if manifest_path.is_file() {
                manifest_paths.push(manifest_path);
            }
        }
    }

    let mut plugins = Vec::new();
    for manifest_path in manifest_paths {
        match read_discovered_plugin(manifest_path, source.clone()) {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => tracing_log(&error.to_string()),
        }
    }

    Ok(plugins)
}

fn read_discovered_plugin(
    manifest_path: PathBuf,
    source: PluginSource,
) -> Result<DiscoveredPlugin, PluginDiscoveryError> {
    let manifest_json = fs::read_to_string(&manifest_path).map_err(|source| {
        PluginDiscoveryError::ReadManifest {
            path: manifest_path.clone(),
            source,
        }
    })?;
    let manifest = serde_json::from_str::<PluginManifest>(&manifest_json).map_err(|source| {
        PluginDiscoveryError::ParseManifest {
            path: manifest_path.clone(),
            source,
        }
    })?;
    manifest
        .validate()
        .map_err(|source| PluginDiscoveryError::InvalidManifest {
            path: manifest_path.clone(),
            source,
        })?;

    let root_dir = manifest_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    validate_entry_stays_inside_root(&root_dir, &manifest)?;

    Ok(DiscoveredPlugin {
        manifest,
        root_dir,
        manifest_path,
        source,
    })
}

fn validate_entry_stays_inside_root(
    root_dir: &std::path::Path,
    manifest: &PluginManifest,
) -> Result<(), PluginDiscoveryError> {
    let Some(entry) = &manifest.entry else {
        return Ok(());
    };
    let root = root_dir
        .canonicalize()
        .map_err(|source| PluginDiscoveryError::ReadDirectory {
            path: root_dir.to_path_buf(),
            source,
        })?;
    let entry_path = root_dir.join(entry);
    if let Ok(entry) = entry_path.canonicalize()
        && !entry.starts_with(&root)
    {
        return Err(PluginDiscoveryError::EntryEscapesRoot { path: entry_path });
    }
    Ok(())
}

fn tracing_log(message: &str) {
    log::info!("插件发现: {message}");
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;

    #[test]
    fn discovers_child_manifest_with_source() {
        let root = temp_dir("discover_child_manifest");
        let plugin_dir = root.join("hello");
        fs::create_dir_all(plugin_dir.join("dist")).expect("create plugin dir");
        fs::write(plugin_dir.join("dist/index.html"), "hello").expect("write entry");
        fs::write(
            plugin_dir.join(PLUGIN_MANIFEST_FILE_NAME),
            r#"{
                "id": "dev.litools.hello",
                "name": "Hello",
                "version": "0.1.0",
                "entry": "dist/index.html",
                "icon": "dist/icon.svg",
                "commands": [{ "id": "hello", "title": "Hello", "mode": "view" }]
            }"#,
        )
        .expect("write manifest");

        let plugins =
            discover_plugins_in_root(root, PluginSource::Bundled).expect("discover plugins");

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].manifest.id, "dev.litools.hello");
        assert_eq!(plugins[0].source, PluginSource::Bundled);
    }

    fn temp_dir(name: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time")
            .as_millis();
        let path = std::env::temp_dir().join(format!("litools-{name}-{millis}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }
}
