use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppSettings {
    pub theme: String,
    pub palette: PaletteSettings,
    pub search: SearchSettings,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaletteSettings {
    pub global_hotkey: String,
    pub result_limit: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchSettings {
    pub enabled_providers: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            palette: PaletteSettings {
                global_hotkey: "CommandOrControl+Space".to_string(),
                result_limit: 20,
            },
            search: SearchSettings {
                enabled_providers: vec!["commands".to_string()],
            },
        }
    }
}
