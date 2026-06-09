use crate::{DiscoveredApp, SystemAdapter, launcher::LaunchTarget};

#[derive(Default)]
pub struct WindowsSystemAdapter;

impl SystemAdapter for WindowsSystemAdapter {
    fn discover_apps(&self) -> Vec<DiscoveredApp> {
        Vec::new()
    }

    fn launch(&self, _target: &LaunchTarget) -> Result<(), String> {
        Ok(())
    }
}
