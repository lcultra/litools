pub mod apps;
pub mod commands;
pub mod metadata;
pub mod pinned;
pub mod plugin_commands;
pub mod plugin_storage;
pub mod plugins;
pub mod settings;
pub mod usage;

pub use apps::{AppRecord, AppRepository, AppUpsert};
pub use commands::CommandRepository;
pub use metadata::IndexMetadataRepository;
pub use pinned::{PinnedItemRecord, PinnedRepository};
pub use plugin_commands::{PluginCommandRecord, PluginCommandRepository, PluginCommandUpsert};
pub use plugin_storage::PluginStorageRepository;
pub use plugins::{PluginRecord, PluginRepository, PluginUpsert};
pub use settings::SettingsRepository;
pub use usage::{UsageEventRecord, UsageRepository};

pub(crate) fn json_string_array(value: String) -> rusqlite::Result<Vec<String>> {
    serde_json::from_str(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    })
}
