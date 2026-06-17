use std::collections::BTreeMap;

use chrono::Utc;

use crate::{
    core::surface::model::{SurfaceBounds, SurfaceLifecycle, SurfaceMetadata},
    view::{ViewDefinition, ViewProvider, WindowHostKind},
    windowing::labels::{MAIN_WINDOW_LABEL, core_webview_label, plugin_webview_label},
};

#[derive(Debug)]
pub struct SurfaceRegistry {
    surfaces_by_id: BTreeMap<String, SurfaceMetadata>,
    pub(crate) surface_id_by_window_label: BTreeMap<String, String>,
}

impl Default for SurfaceRegistry {
    fn default() -> Self {
        Self {
            surfaces_by_id: BTreeMap::new(),
            surface_id_by_window_label: BTreeMap::new(),
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
        // surface_id 即 webview_label，按 provider 生成对应前缀的 UUID 标签
        let id = match &view.provider {
            ViewProvider::Plugin { .. } => plugin_webview_label(),
            _ => core_webview_label(),
        };
        log::debug!(
            "[surface] 注册 surface={id} provider={:?} route={} host_window={host_window_label} host_kind={:?}",
            view.provider,
            view.route,
            host_kind
        );
        let now = Self::now();
        let metadata = SurfaceMetadata {
            id: id.clone(),
            webview_label: id.clone(),
            view_id: view.id,
            provider: view.provider,
            route: view.route,
            title: view.title,
            host_window_label: host_window_label.clone(),
            host_kind,
            bounds: None,
            lifecycle: SurfaceLifecycle::Active,
            focused: false,
            created_at: now.clone(),
            updated_at: now,
        };
        self.surfaces_by_id.insert(id.clone(), metadata.clone());
        self.surface_id_by_window_label
            .insert(host_window_label, id);
        metadata
    }

    pub fn next_detached_host_label(&mut self) -> String {
        crate::windowing::labels::detach_window_label()
    }

    /// 通过 id 或 window_label 查找 surface 元数据。
    /// 由于 surface_id == webview_label，按任意标签查找均可直接命中。
    pub fn metadata(&self, target: &str) -> Option<SurfaceMetadata> {
        if target == MAIN_WINDOW_LABEL {
            let id = self.surface_id_by_window_label.get(MAIN_WINDOW_LABEL)?;
            return self.surfaces_by_id.get(id).cloned();
        }

        // 优先按 id 查找
        if let Some(metadata) = self.surfaces_by_id.get(target) {
            return Some(metadata.clone());
        }

        // 按 window_label 查找
        if let Some(id) = self.surface_id_by_window_label.get(target) {
            return self.surfaces_by_id.get(id).cloned();
        }

        None
    }

    pub fn metadata_for_webview_label(&self, label: &str) -> Option<SurfaceMetadata> {
        // surface_id == webview_label，直接按 id 查找
        self.surfaces_by_id.get(label).cloned()
    }

    pub fn host_window_label(&self, id: &str) -> Option<&str> {
        self.surfaces_by_id
            .get(id)
            .map(|s| s.host_window_label.as_str())
    }

    pub fn webview_label(&self, id: &str) -> Option<&str> {
        self.surfaces_by_id
            .get(id)
            .map(|s| s.webview_label.as_str())
    }

    pub fn host_kind(&self, id: &str) -> Option<crate::view::WindowHostKind> {
        self.surfaces_by_id.get(id).map(|s| s.host_kind.clone())
    }

    #[allow(dead_code)]
    pub fn metadata_for_window_label(&self, label: &str) -> Option<SurfaceMetadata> {
        let id = self.surface_id_by_window_label.get(label)?;
        self.surfaces_by_id.get(id).cloned()
    }

    pub fn mark_bounds(
        &mut self,
        id: &str,
        bounds: Option<SurfaceBounds>,
    ) -> Option<SurfaceMetadata> {
        let s = self.surfaces_by_id.get_mut(id)?;
        s.bounds = bounds;
        s.updated_at = Self::now();
        Some(s.clone())
    }

    pub fn list(&self) -> Vec<SurfaceMetadata> {
        self.surfaces_by_id.values().cloned().collect()
    }

    pub fn move_to_host(
        &mut self,
        webview_label: &str,
        host_window_label: String,
        host_kind: WindowHostKind,
    ) -> Option<SurfaceMetadata> {
        log::info!(
            "[surface] move_to_host: webview={webview_label} → window={host_window_label} kind={host_kind:?}"
        );
        // webview_label 即 surface_id，直接查找
        let metadata = self.surfaces_by_id.get_mut(webview_label)?;
        // 移除旧的 window_label → id 映射
        self.surface_id_by_window_label
            .remove(&metadata.host_window_label);
        metadata.host_window_label = host_window_label;
        metadata.host_kind = host_kind;
        metadata.lifecycle = SurfaceLifecycle::Active;
        metadata.focused = true;
        metadata.updated_at = Self::now();
        // 插入新的 window_label → id 映射
        self.surface_id_by_window_label
            .insert(metadata.host_window_label.clone(), metadata.id.clone());
        Some(metadata.clone())
    }

    pub fn mark_route(
        &mut self,
        webview_label: &str,
        view: ViewDefinition,
    ) -> Option<SurfaceMetadata> {
        // webview_label 即 surface_id
        let metadata = self.surfaces_by_id.get_mut(webview_label)?;
        metadata.view_id = view.id;
        metadata.provider = view.provider;
        metadata.route = view.route;
        metadata.title = view.title;
        metadata.updated_at = Self::now();
        Some(metadata.clone())
    }

    pub fn mark_lifecycle(
        &mut self,
        target: &str,
        lifecycle: SurfaceLifecycle,
    ) -> Option<SurfaceMetadata> {
        log::debug!("[surface] mark_lifecycle: target={target} → {lifecycle:?}");
        let id = self.resolve_id(target)?;
        let metadata = self.surfaces_by_id.get_mut(&id)?;
        metadata.lifecycle = lifecycle;
        metadata.updated_at = Self::now();
        Some(metadata.clone())
    }

    pub fn mark_focused(&mut self, target: &str, focused: bool) -> Option<SurfaceMetadata> {
        let id = self.resolve_id(target)?;
        let metadata = self.surfaces_by_id.get_mut(&id)?;
        metadata.focused = focused;
        metadata.updated_at = Self::now();
        Some(metadata.clone())
    }

    pub fn remove(&mut self, target: &str) -> Option<SurfaceMetadata> {
        log::debug!("[surface] remove: target={target}");
        let id = if self.surfaces_by_id.contains_key(target) {
            target.to_string()
        } else if let Some(id) = self.surface_id_by_window_label.remove(target) {
            id
        } else {
            return None;
        };

        let context = self.surfaces_by_id.remove(&id)?;
        self.surface_id_by_window_label
            .remove(&context.host_window_label);
        Some(context)
    }

    /// 通过任意 target（id、window_label）解析为 surface_id。
    fn resolve_id(&self, target: &str) -> Option<String> {
        if self.surfaces_by_id.contains_key(target) {
            return Some(target.to_string());
        }
        if let Some(id) = self.surface_id_by_window_label.get(target) {
            return Some(id.clone());
        }
        None
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

    fn test_plugin_view(route: &str) -> ViewDefinition {
        ViewDefinition {
            id: "plugin.test.cmd".to_string(),
            provider: ViewProvider::Plugin {
                plugin_id: "test".to_string(),
            },
            kind: ViewKind::Plugin,
            route: route.to_string(),
            title: "Test Plugin".to_string(),
            default_host: WindowHostKind::Detached,
            allowed_hosts: vec![WindowHostKind::Main, WindowHostKind::Detached],
            detachable: true,
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
        // surface_id == webview_label
        assert_eq!(metadata.id, metadata.webview_label);

        let found = registry.metadata(&metadata.webview_label);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, metadata.id);
    }

    #[test]
    fn core_surface_has_core_prefix() {
        let mut registry = SurfaceRegistry::default();
        let view = test_view("/");
        let metadata =
            registry.register_surface(view, MAIN_WINDOW_LABEL.to_string(), WindowHostKind::Main);
        assert!(
            metadata.id.starts_with("core-webview-"),
            "expected core- prefix, got: {}",
            metadata.id
        );
    }

    #[test]
    fn plugin_surface_has_plugin_prefix() {
        let mut registry = SurfaceRegistry::default();
        let view = test_plugin_view("/plugin/test/cmd");
        let metadata = registry.register_surface(
            view,
            "detach-test-uuid".to_string(),
            WindowHostKind::Detached,
        );
        assert!(
            metadata.id.starts_with("plugin-webview-"),
            "expected plugin- prefix, got: {}",
            metadata.id
        );
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

    #[test]
    fn finds_by_window_label() {
        let mut registry = SurfaceRegistry::default();
        let metadata = registry.register_surface(
            test_view("/"),
            "my-window".to_string(),
            WindowHostKind::Detached,
        );

        let found = registry.metadata_for_window_label("my-window");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, metadata.id);
    }

    #[test]
    fn host_window_label_and_webview_label_helpers() {
        let mut registry = SurfaceRegistry::default();
        let metadata =
            registry.register_surface(test_view("/"), "main".to_string(), WindowHostKind::Main);

        assert_eq!(registry.host_window_label(&metadata.id), Some("main"));
        assert_eq!(
            registry.webview_label(&metadata.id),
            Some(metadata.webview_label.as_str())
        );
    }

    #[test]
    fn move_to_host_updates_window_label_index() {
        let mut registry = SurfaceRegistry::default();
        let metadata =
            registry.register_surface(test_view("/"), "old-host".to_string(), WindowHostKind::Main);

        assert!(registry.metadata_for_window_label("old-host").is_some());
        assert!(registry.metadata_for_window_label("new-host").is_none());

        let moved = registry
            .move_to_host(
                &metadata.webview_label,
                "new-host".to_string(),
                WindowHostKind::Detached,
            )
            .expect("move to host");
        assert_eq!(moved.host_window_label, "new-host");
        assert_eq!(moved.host_kind, WindowHostKind::Detached);

        assert!(registry.metadata_for_window_label("old-host").is_none());
        assert!(registry.metadata_for_window_label("new-host").is_some());
    }

    #[test]
    fn mark_bounds_sets_and_returns() {
        let mut registry = SurfaceRegistry::default();
        let metadata =
            registry.register_surface(test_view("/"), "main".to_string(), WindowHostKind::Main);

        let bounds = SurfaceBounds {
            x: 0.0,
            y: 68.0,
            width: 820.0,
            height: 492.0,
        };
        let updated = registry
            .mark_bounds(&metadata.id, Some(bounds))
            .expect("mark bounds");
        assert_eq!(updated.bounds, Some(bounds));

        let cleared = registry
            .mark_bounds(&metadata.id, None)
            .expect("clear bounds");
        assert_eq!(cleared.bounds, None);
    }

    #[test]
    fn remove_cleans_window_label_index() {
        let mut registry = SurfaceRegistry::default();
        let metadata =
            registry.register_surface(test_view("/"), "my-host".to_string(), WindowHostKind::Main);

        assert!(registry.metadata_for_window_label("my-host").is_some());

        registry.remove(&metadata.id);
        assert!(registry.metadata_for_window_label("my-host").is_none());
        assert!(registry.metadata(&metadata.id).is_none());
    }

    #[test]
    fn find_by_webview_label_same_as_surface_id() {
        let mut registry = SurfaceRegistry::default();
        let metadata =
            registry.register_surface(test_view("/"), "main".to_string(), WindowHostKind::Main);

        // surface_id 即 webview_label，用 webview_label 可直接查到
        let found = registry.metadata_for_webview_label(&metadata.webview_label);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, metadata.id);
    }

    #[test]
    fn detach_label_generates_unique_values() {
        let mut registry = SurfaceRegistry::default();
        let label1 = registry.next_detached_host_label();
        let label2 = registry.next_detached_host_label();
        assert!(label1.starts_with("detach-window-"));
        assert!(label2.starts_with("detach-window-"));
        assert_ne!(label1, label2);
    }
}
