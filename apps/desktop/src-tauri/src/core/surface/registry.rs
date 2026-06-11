use std::collections::BTreeMap;

use chrono::Utc;

use crate::{
    core::surface::model::{SurfaceLifecycle, SurfaceMetadata},
    view::{ViewDefinition, WindowHostKind},
    windowing::labels::{MAIN_WINDOW_LABEL, surface_webview_label},
};
pub use litools_config::labels::{DETACHED_HOST_ID_PREFIX, SURFACE_ID_PREFIX};

#[derive(Debug)]
pub struct SurfaceRegistry {
    next_surface_index: u64,
    next_detached_host_index: u64,
    surfaces_by_webview_label: BTreeMap<String, SurfaceMetadata>,
}

impl Default for SurfaceRegistry {
    fn default() -> Self {
        Self {
            next_surface_index: 1,
            next_detached_host_index: 1,
            surfaces_by_webview_label: BTreeMap::new(),
        }
    }
}

impl SurfaceRegistry {
    fn now() -> String {
        Utc::now().to_rfc3339()
    }

    pub fn register_surface(
        &mut self,
        view: ViewDefinition,
        host_window_label: String,
        host_kind: WindowHostKind,
    ) -> SurfaceMetadata {
        let id = format!("{SURFACE_ID_PREFIX}_{:06}", self.next_surface_index);
        self.next_surface_index += 1;
        let webview_label = surface_webview_label(&id);
        let now = Self::now();
        let metadata = SurfaceMetadata {
            id,
            webview_label: webview_label.clone(),
            view_id: view.id,
            provider: view.provider,
            route: view.route,
            title: view.title,
            host_window_label,
            host_kind,
            lifecycle: SurfaceLifecycle::Active,
            focused: false,
            created_at: now.clone(),
            updated_at: now,
        };
        self.surfaces_by_webview_label
            .insert(webview_label, metadata.clone());
        metadata
    }

    pub fn next_detached_host_label(&mut self) -> String {
        let label = format!(
            "{}{DETACHED_HOST_ID_PREFIX}_{:06}",
            crate::windowing::labels::DETACHED_PANEL_WINDOW_PREFIX,
            self.next_detached_host_index
        );
        self.next_detached_host_index += 1;
        label
    }

    pub fn metadata(&self, target: &str) -> Option<SurfaceMetadata> {
        if target == MAIN_WINDOW_LABEL {
            return self
                .surfaces_by_webview_label
                .values()
                .find(|metadata| metadata.host_window_label == MAIN_WINDOW_LABEL)
                .cloned();
        }

        self.surfaces_by_webview_label
            .get(target)
            .cloned()
            .or_else(|| {
                self.surfaces_by_webview_label
                    .values()
                    .find(|metadata| metadata.id == target || metadata.host_window_label == target)
                    .cloned()
            })
    }

    pub fn metadata_for_webview_label(&self, label: &str) -> Option<SurfaceMetadata> {
        self.surfaces_by_webview_label.get(label).cloned()
    }

    pub fn list(&self) -> Vec<SurfaceMetadata> {
        self.surfaces_by_webview_label.values().cloned().collect()
    }

    pub fn move_to_host(
        &mut self,
        webview_label: &str,
        host_window_label: String,
        host_kind: WindowHostKind,
    ) -> Option<SurfaceMetadata> {
        self.update_by_webview_label(webview_label, |metadata| {
            metadata.host_window_label = host_window_label;
            metadata.host_kind = host_kind;
            metadata.lifecycle = SurfaceLifecycle::Active;
            metadata.focused = true;
        })
    }

    pub fn mark_route(
        &mut self,
        webview_label: &str,
        view: ViewDefinition,
    ) -> Option<SurfaceMetadata> {
        self.update_by_webview_label(webview_label, |metadata| {
            metadata.view_id = view.id;
            metadata.provider = view.provider;
            metadata.route = view.route;
            metadata.title = view.title;
        })
    }

    pub fn mark_lifecycle(
        &mut self,
        target: &str,
        lifecycle: SurfaceLifecycle,
    ) -> Option<SurfaceMetadata> {
        let webview_label = self.metadata(target)?.webview_label;
        self.update_by_webview_label(&webview_label, |metadata| {
            metadata.lifecycle = lifecycle;
        })
    }

    pub fn mark_focused(&mut self, target: &str, focused: bool) -> Option<SurfaceMetadata> {
        let webview_label = self.metadata(target)?.webview_label;
        self.update_by_webview_label(&webview_label, |metadata| {
            metadata.focused = focused;
        })
    }

    pub fn remove(&mut self, target: &str) -> Option<SurfaceMetadata> {
        if self.surfaces_by_webview_label.contains_key(target) {
            return self.surfaces_by_webview_label.remove(target);
        }

        let webview_label =
            self.surfaces_by_webview_label
                .iter()
                .find_map(|(webview_label, metadata)| {
                    (metadata.id == target || metadata.host_window_label == target)
                        .then(|| webview_label.clone())
                })?;
        self.surfaces_by_webview_label.remove(&webview_label)
    }

    fn update_by_webview_label(
        &mut self,
        webview_label: &str,
        update: impl FnOnce(&mut SurfaceMetadata),
    ) -> Option<SurfaceMetadata> {
        let metadata = self.surfaces_by_webview_label.get_mut(webview_label)?;
        update(metadata);
        metadata.updated_at = Self::now();
        Some(metadata.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::view::{ViewDefinition, ViewKind, ViewProvider, WindowHostKind};

    use super::*;

    fn test_view(route: &str) -> ViewDefinition {
        ViewDefinition {
            id: "core.launcher".to_string(),
            provider: ViewProvider::Core,
            kind: ViewKind::Launcher,
            route: route.to_string(),
            title: "Test".to_string(),
            default_host: WindowHostKind::Main,
            allowed_hosts: vec![WindowHostKind::Main],
            detachable: false,
        }
    }

    #[test]
    fn registers_and_finds_surface() {
        let mut registry = SurfaceRegistry::default();
        let view = test_view("/");
        let metadata =
            registry.register_surface(view, MAIN_WINDOW_LABEL.to_string(), WindowHostKind::Main);

        assert_eq!(metadata.route, "/");
        assert_eq!(metadata.host_kind, WindowHostKind::Main);
        assert!(matches!(metadata.lifecycle, SurfaceLifecycle::Active));

        let found = registry.metadata(&metadata.webview_label);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, metadata.id);
    }

    #[test]
    fn lists_all_surfaces() {
        let mut registry = SurfaceRegistry::default();
        registry.register_surface(test_view("/a"), "main".to_string(), WindowHostKind::Main);
        registry.register_surface(test_view("/b"), "main".to_string(), WindowHostKind::Main);

        assert_eq!(registry.list().len(), 2);
    }

    #[test]
    fn marks_lifecycle_and_removes_surface() {
        let mut registry = SurfaceRegistry::default();
        let metadata =
            registry.register_surface(test_view("/"), "main".to_string(), WindowHostKind::Main);

        let updated = registry
            .mark_lifecycle(&metadata.webview_label, SurfaceLifecycle::Hidden)
            .expect("mark lifecycle");
        assert!(matches!(updated.lifecycle, SurfaceLifecycle::Hidden));

        let removed = registry.remove(&metadata.webview_label);
        assert!(removed.is_some());
        assert!(registry.metadata(&metadata.webview_label).is_none());
    }
}
