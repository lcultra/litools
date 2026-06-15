/// PluginRuntime trait —— 不与 WebView 绑定
pub trait PluginRuntime: Send + Sync {
    fn execute(&self, script_uri: &str) -> Result<(), String>;
}
