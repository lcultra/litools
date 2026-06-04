use async_trait::async_trait;

use crate::{DiscoveredApp, SystemAdapter};

#[derive(Default)]
pub struct WindowsSystemAdapter;

#[async_trait]
impl SystemAdapter for WindowsSystemAdapter {
    async fn discover_apps(&self) -> Vec<DiscoveredApp> {
        Vec::new()
    }

    async fn launch_app(&self, _app_id: &str) -> Result<(), String> {
        Ok(())
    }

    async fn open_file(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
}
