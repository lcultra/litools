use crate::{DiscoveredApp, SystemAdapter};

pub fn index_apps(adapter: &dyn SystemAdapter) -> Vec<DiscoveredApp> {
    adapter.discover_apps()
}
