use litools_index::IndexDatabase;
use litools_plugin::PluginManager;
use litools_search::SearchEngine;
use litools_settings::{AppSettings, storage::SettingsStore};

pub struct AppContext {
    pub database: IndexDatabase,
    pub search: SearchEngine,
    pub plugins: PluginManager,
    pub settings: SettingsStore,
}

impl AppContext {
    pub fn new(database: IndexDatabase, search: SearchEngine) -> Self {
        Self {
            database,
            search,
            plugins: PluginManager::new(),
            settings: SettingsStore::new(AppSettings::default()),
        }
    }
}
