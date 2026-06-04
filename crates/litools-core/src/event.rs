#[derive(Clone, Debug)]
pub enum AppEvent {
    SearchCompleted,
    CommandExecuted(String),
    PluginPermissionDenied {
        plugin_id: String,
        permission: String,
    },
}
