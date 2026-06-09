/// Normalize a user-supplied accelerator string to a canonical form.
///
/// This is framework-agnostic string normalization. The Tauri layer uses
/// the normalized accelerator to parse it into platform-specific modifiers.
pub fn normalize_accelerator(accelerator: &str) -> String {
    match accelerator.trim() {
        "Meta+Space" | "Cmd+Space" | "Command+Space" => "Meta+Space".to_string(),
        "Control+Space" | "Ctrl+Space" => "Control+Space".to_string(),
        "CommandOrControl+Space" | "" => "CommandOrControl+Space".to_string(),
        other => other.to_string(),
    }
}
