use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveredApp {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
}

pub trait SystemAdapter: Send + Sync {
    fn discover_apps(&self) -> Vec<DiscoveredApp>;

    fn launch_app(&self, app_id: &str) -> Result<(), String>;

    fn open_file(&self, path: &str) -> Result<(), String>;
}
