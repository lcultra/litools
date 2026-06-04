use std::path::PathBuf;

use crate::manifest::PluginManifest;

#[derive(Default)]
pub struct PluginManager {
    installed_plugins: Vec<InstalledPlugin>,
}

#[derive(Clone, Debug)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub enabled: bool,
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_local_plugin(&mut self, manifest: PluginManifest, path: PathBuf) {
        self.installed_plugins.push(InstalledPlugin {
            manifest,
            path,
            enabled: true,
        });
    }

    pub fn installed_plugins(&self) -> &[InstalledPlugin] {
        &self.installed_plugins
    }
}
