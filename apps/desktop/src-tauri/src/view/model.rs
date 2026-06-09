use serde::Serialize;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewProvider {
    Core,
    Plugin { plugin_id: String },
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewKind {
    Launcher,
    Plugin,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowHostKind {
    Main,
    Detached,
    Runtime,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewDefinition {
    pub id: String,
    pub provider: ViewProvider,
    pub kind: ViewKind,
    pub route: String,
    pub title: String,
    pub default_host: WindowHostKind,
    pub allowed_hosts: Vec<WindowHostKind>,
    pub detachable: bool,
}
