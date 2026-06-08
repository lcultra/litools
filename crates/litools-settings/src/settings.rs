use serde::{Deserialize, Serialize};

pub const DEFAULT_GLOBAL_HOTKEY: &str = "CommandOrControl+Space";
pub const DEFAULT_ENABLED_PROVIDERS: &[&str] = &["apps", "commands", "plugins"];
pub const SUPPORTED_SEARCH_PROVIDERS: &[&str] = &["apps", "commands", "plugins"];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AppSettings {
    pub theme: String,
    pub palette: PaletteSettings,
    pub search: SearchSettings,
    pub window: WindowSettings,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PaletteSettings {
    pub global_hotkey: String,
    pub show_recent: bool,
    pub show_pinned: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchSettings {
    pub enabled_providers: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WindowSettings {
    pub hide_on_blur: bool,
    pub close_to_tray: bool,
    pub center_on_show: bool,
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        if !matches!(self.theme.as_str(), "system" | "light" | "dark") {
            self.theme = "system".to_string();
        }

        if self.palette.global_hotkey.trim().is_empty() {
            self.palette.global_hotkey = DEFAULT_GLOBAL_HOTKEY.to_string();
        }

        self.search
            .enabled_providers
            .retain(|provider| SUPPORTED_SEARCH_PROVIDERS.contains(&provider.as_str()));
        self.search.enabled_providers.sort();
        self.search.enabled_providers.dedup();

        if self.search.enabled_providers.is_empty() || self.search.enabled_providers == ["commands"]
        {
            self.search.enabled_providers = DEFAULT_ENABLED_PROVIDERS
                .iter()
                .map(|provider| provider.to_string())
                .collect();
        }

        self
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            palette: PaletteSettings {
                global_hotkey: DEFAULT_GLOBAL_HOTKEY.to_string(),
                show_recent: true,
                show_pinned: true,
            },
            search: SearchSettings {
                enabled_providers: DEFAULT_ENABLED_PROVIDERS
                    .iter()
                    .map(|provider| provider.to_string())
                    .collect(),
            },
            window: WindowSettings {
                hide_on_blur: true,
                close_to_tray: true,
                center_on_show: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_include_phase_one_window_behavior() {
        let settings = AppSettings::default();

        assert_eq!(settings.theme, "system");
        assert_eq!(settings.palette.global_hotkey, DEFAULT_GLOBAL_HOTKEY);
        assert!(settings.palette.show_recent);
        assert!(settings.palette.show_pinned);
        assert_eq!(
            settings.search.enabled_providers,
            ["apps", "commands", "plugins"]
        );
        assert!(settings.window.hide_on_blur);
        assert!(settings.window.close_to_tray);
        assert!(settings.window.center_on_show);
    }

    #[test]
    fn normalization_clamps_and_repairs_invalid_values() {
        let settings = AppSettings {
            theme: "unknown".to_string(),
            palette: PaletteSettings {
                global_hotkey: " ".to_string(),
                show_recent: false,
                show_pinned: false,
            },
            search: SearchSettings {
                enabled_providers: vec![
                    "".to_string(),
                    "commands".to_string(),
                    "commands".to_string(),
                ],
            },
            window: WindowSettings {
                hide_on_blur: false,
                close_to_tray: false,
                center_on_show: false,
            },
        }
        .normalized();

        assert_eq!(settings.theme, "system");
        assert_eq!(settings.palette.global_hotkey, DEFAULT_GLOBAL_HOTKEY);
        assert_eq!(
            settings.search.enabled_providers,
            ["apps", "commands", "plugins"]
        );
        assert!(!settings.palette.show_recent);
        assert!(!settings.palette.show_pinned);
        assert!(!settings.window.hide_on_blur);
        assert!(!settings.window.close_to_tray);
        assert!(!settings.window.center_on_show);
    }
}
