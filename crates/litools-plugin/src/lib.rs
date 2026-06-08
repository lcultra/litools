pub mod api;
pub mod discovery;
pub mod manager;
pub mod manifest;
pub mod marketplace;
pub mod permission;
pub mod runtime;

pub use discovery::{DiscoveredPlugin, PluginDiscoveryRoot, discover_plugins};
pub use manager::{InstalledPlugin, PluginManager, PluginSource};
pub use manifest::{
    PluginCommand, PluginCommandMode, PluginManifest, plugin_command_mode_from_str,
};
pub use permission::{PermissionDecision, PermissionEngine};
