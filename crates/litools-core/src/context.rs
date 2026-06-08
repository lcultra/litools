use litools_index::IndexDatabase;
use litools_plugin::PluginManager;
use litools_search::SearchEngine;
use litools_settings::storage::SettingsStore;

pub struct AppContext {
    pub database: IndexDatabase,
    pub search: SearchEngine,
    pub plugins: PluginManager,
    pub settings: SettingsStore,
}

impl AppContext {
    pub fn new(
        database: IndexDatabase,
        search: SearchEngine,
        plugins: PluginManager,
        settings: SettingsStore,
    ) -> Self {
        Self {
            database,
            search,
            plugins,
            settings,
        }
    }
}
