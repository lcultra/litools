use litools_index::IndexDatabase;
use litools_plugin::PluginManager;
use litools_search::SearchEngine;
use litools_settings::storage::SettingsStore;
use litools_system::NativeSystemAdapter;

pub struct AppContext {
    pub database: IndexDatabase,
    pub search: SearchEngine,
    pub plugins: PluginManager,
    pub settings: SettingsStore,
    pub system_adapter: NativeSystemAdapter,
}

impl AppContext {
    pub fn new(
        database: IndexDatabase,
        search: SearchEngine,
        plugins: PluginManager,
        settings: SettingsStore,
        system_adapter: NativeSystemAdapter,
    ) -> Self {
        Self {
            database,
            search,
            plugins,
            settings,
            system_adapter,
        }
    }
}
