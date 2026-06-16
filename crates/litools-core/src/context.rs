use std::sync::Arc;

use litools_index::IndexDatabase;
use litools_plugin::PluginManager;
use litools_search::SearchEngine;
use litools_settings::storage::SettingsStore;
use litools_system::NativeSystemAdapter;

use crate::executor_registry::ExecutorRegistry;

pub struct AppContext {
    pub database: IndexDatabase,
    pub search: Arc<SearchEngine>,
    pub plugins: Arc<PluginManager>,
    pub settings: SettingsStore,
    pub system_adapter: NativeSystemAdapter,
    pub executor_registry: ExecutorRegistry,
}

impl AppContext {
    pub fn new(
        database: IndexDatabase,
        search: SearchEngine,
        plugins: Arc<PluginManager>,
        settings: SettingsStore,
        system_adapter: NativeSystemAdapter,
        executor_registry: ExecutorRegistry,
    ) -> Self {
        Self {
            database,
            search: Arc::new(search),
            plugins,
            settings,
            system_adapter,
            executor_registry,
        }
    }
}
