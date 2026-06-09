use serde::{Deserialize, Serialize};

use crate::launcher::LaunchTarget;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveredApp {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub localized_names: Vec<String>,
    pub aliases: Vec<String>,
    pub search_text: String,
}

pub trait SystemAdapter: Send + Sync {
    fn discover_apps(&self) -> Vec<DiscoveredApp>;

    fn launch(&self, target: &LaunchTarget) -> Result<(), String>;

    fn launch_app(&self, app_id: &str) -> Result<(), String> {
        self.launch(&LaunchTarget::App(app_id.to_string()))
    }

    fn open_file(&self, path: &str) -> Result<(), String> {
        self.launch(&LaunchTarget::File(path.to_string()))
    }
}
