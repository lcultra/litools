use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveredApp {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
}

#[async_trait]
pub trait SystemAdapter: Send + Sync {
    async fn discover_apps(&self) -> Vec<DiscoveredApp>;

    async fn launch_app(&self, app_id: &str) -> Result<(), String>;

    async fn open_file(&self, path: &str) -> Result<(), String>;
}
