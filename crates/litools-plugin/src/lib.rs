pub mod discovery;
pub mod ids;
pub mod manager;
pub mod manifest;
pub mod permission;

pub use discovery::{DiscoveredPlugin, PluginDiscoveryRoot, discover_plugins};
pub use ids::{
    PLUGIN_RESULT_PREFIX, PLUGIN_TARGET_TYPE, plugin_command_from_result_id,
    plugin_command_from_target_id, plugin_result_id, plugin_target_id,
};
pub use manager::{InstalledPlugin, PluginManager, PluginSource};
pub use manifest::{
    PluginCommand, PluginCommandMode, PluginManifest, RuntimePolicy, plugin_command_mode_from_str,
};
pub use permission::{PermissionDecision, PermissionEngine};
