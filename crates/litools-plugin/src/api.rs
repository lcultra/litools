#[derive(Clone, Debug)]
pub enum PluginApiCapability {
    Storage,
    ClipboardRead,
    ClipboardWrite,
    UiToast,
}
