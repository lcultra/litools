use litools_index::{
    IndexDatabase,
    repository::SettingsRepository,
};
use litools_settings::AppSettings;

use crate::{
    app::LitoolsApp,
    error::LitoolsResult,
};

use super::APP_SETTINGS_KEY;

impl LitoolsApp {
    pub fn update_settings(&mut self, settings: AppSettings) -> LitoolsResult<AppSettings> {
        let settings = settings.normalized();
        persist_settings(&self.context.database, &settings)?;
        self.context.settings.replace(settings.clone());
        Ok(settings)
    }
}

pub(crate) fn load_settings(database: &IndexDatabase) -> LitoolsResult<AppSettings> {
    let connection = database.connection();
    let repository = SettingsRepository::new(&connection);
    let settings = match repository.get_json(APP_SETTINGS_KEY)? {
        Some(value_json) => serde_json::from_str::<AppSettings>(&value_json)
            .unwrap_or_else(|_| AppSettings::default())
            .normalized(),
        None => AppSettings::default(),
    };
    repository.set_json(APP_SETTINGS_KEY, &serde_json::to_string(&settings)?)?;
    Ok(settings)
}

fn persist_settings(database: &IndexDatabase, settings: &AppSettings) -> LitoolsResult<()> {
    let connection = database.connection();
    SettingsRepository::new(&connection)
        .set_json(APP_SETTINGS_KEY, &serde_json::to_string(settings)?)?;
    Ok(())
}
