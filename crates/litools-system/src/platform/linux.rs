use std::{io, path::PathBuf};

use crate::{DiscoveredApp, SystemAdapter, adapter::AppWatchGuard, launcher::LaunchTarget};

#[derive(Default)]
pub struct LinuxSystemAdapter;

impl SystemAdapter for LinuxSystemAdapter {
    fn discover_apps(&self) -> Vec<DiscoveredApp> {
        Vec::new()
    }

    fn launch(&self, _target: &LaunchTarget) -> Result<(), String> {
        Ok(())
    }

    fn application_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = vec![PathBuf::from("/usr/share/applications")];
        if let Some(home) = std::env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join(".local/share/applications"));
        }
        dirs
    }

    fn app_icon_png(&self, _path: &std::path::Path) -> io::Result<Vec<u8>> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "app icon extraction is not yet implemented on Linux",
        ))
    }

    fn watch_app_dirs(
        &self,
        _on_change: Box<dyn Fn() + Send + 'static>,
    ) -> io::Result<AppWatchGuard> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "app watching is not yet implemented on Linux",
        ))
    }
}
