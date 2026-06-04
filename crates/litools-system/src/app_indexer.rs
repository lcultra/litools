use crate::{DiscoveredApp, SystemAdapter};

pub async fn index_apps(adapter: &dyn SystemAdapter) -> Vec<DiscoveredApp> {
    adapter.discover_apps().await
}
