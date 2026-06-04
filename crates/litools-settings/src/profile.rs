#[derive(Clone, Debug)]
pub struct SettingsProfile {
    pub name: String,
}

impl SettingsProfile {
    pub fn default_profile() -> Self {
        Self {
            name: "default".to_string(),
        }
    }
}
