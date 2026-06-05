use crate::{DiscoveredApp, SystemAdapter};

#[derive(Default)]
pub struct LinuxSystemAdapter;

impl SystemAdapter for LinuxSystemAdapter {
    fn discover_apps(&self) -> Vec<DiscoveredApp> {
        Vec::new()
    }

    fn launch_app(&self, _app_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn open_file(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
}
