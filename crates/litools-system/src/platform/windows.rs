use std::{io, path::PathBuf};

use crate::{DiscoveredApp, SystemAdapter, adapter::AppWatchGuard, launcher::LaunchTarget};

#[derive(Default)]
pub struct WindowsSystemAdapter;

impl SystemAdapter for WindowsSystemAdapter {
    fn discover_apps(&self) -> Vec<DiscoveredApp> {
        Vec::new()
    }

    fn launch(&self, _target: &LaunchTarget) -> Result<(), String> {
        Ok(())
    }

    fn application_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            dirs.push(PathBuf::from(program_files));
        }
        if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
            dirs.push(PathBuf::from(program_files_x86));
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            dirs.push(
                PathBuf::from(appdata)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Start Menu")
                    .join("Programs"),
            );
        }
        dirs
    }

    fn app_icon_png(&self, _path: &std::path::Path) -> io::Result<Vec<u8>> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "app icon extraction is not yet implemented on Windows",
        ))
    }

    fn watch_app_dirs(
        &self,
        _on_change: Box<dyn Fn() + Send + 'static>,
    ) -> io::Result<AppWatchGuard> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "app watching is not yet implemented on Windows",
        ))
    }
}
