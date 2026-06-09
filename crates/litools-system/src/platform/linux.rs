use crate::{DiscoveredApp, SystemAdapter, launcher::LaunchTarget};

#[derive(Default)]
pub struct LinuxSystemAdapter;

impl SystemAdapter for LinuxSystemAdapter {
    fn discover_apps(&self) -> Vec<DiscoveredApp> {
        Vec::new()
    }

    fn launch(&self, _target: &LaunchTarget) -> Result<(), String> {
        Ok(())
    }
}
