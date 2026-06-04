#[derive(Clone, Debug)]
pub enum PluginRuntimeState {
    Inactive,
    Active,
    Failed(String),
}
