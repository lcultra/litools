use crate::settings::AppSettings;

#[derive(Default)]
pub struct SettingsStore {
    settings: AppSettings,
}

impl SettingsStore {
    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
    }

    pub fn get(&self) -> &AppSettings {
        &self.settings
    }

    pub fn replace(&mut self, settings: AppSettings) {
        self.settings = settings;
    }
}
