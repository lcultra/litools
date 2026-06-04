use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

use crate::{state::ShortcutStatus, state::AppState};

pub fn register_global_shortcut(app: &AppHandle, accelerator: &str) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    let normalized = normalize_accelerator(accelerator);
    let shortcut = parse_shortcut(&normalized);
    let manager = app.global_shortcut();
    let _ = manager.unregister_all();

    match manager.register(shortcut) {
        Ok(()) => state.set_shortcut_status(ShortcutStatus {
            accelerator: normalized,
            registered: true,
            error: None,
        }),
        Err(error) => state.set_shortcut_status(ShortcutStatus {
            accelerator: normalized,
            registered: false,
            error: Some(error.to_string()),
        }),
    }
}

pub fn matches_palette_shortcut(shortcut: &Shortcut, state: &AppState) -> bool {
    *shortcut == parse_shortcut(&normalize_accelerator(&state.global_hotkey()))
}

fn normalize_accelerator(accelerator: &str) -> String {
    match accelerator.trim() {
        "Meta+Space" | "Cmd+Space" | "Command+Space" => "Meta+Space".to_string(),
        "Control+Space" | "Ctrl+Space" => "Control+Space".to_string(),
        "CommandOrControl+Space" | "" => "CommandOrControl+Space".to_string(),
        other => other.to_string(),
    }
}

fn parse_shortcut(accelerator: &str) -> Shortcut {
    match accelerator {
        "Control+Space" => Shortcut::new(Some(Modifiers::CONTROL), Code::Space),
        "Meta+Space" => Shortcut::new(Some(Modifiers::META), Code::Space),
        "CommandOrControl+Space" => Shortcut::new(Some(command_or_control_modifier()), Code::Space),
        _ => Shortcut::new(Some(command_or_control_modifier()), Code::Space),
    }
}

#[cfg(target_os = "macos")]
fn command_or_control_modifier() -> Modifiers {
    Modifiers::META
}

#[cfg(not(target_os = "macos"))]
fn command_or_control_modifier() -> Modifiers {
    Modifiers::CONTROL
}
