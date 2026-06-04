#[derive(Clone, Debug)]
pub struct DiagnosticSnapshot {
    pub app_version: String,
    pub enabled_plugins: usize,
    pub indexed_apps: usize,
}
