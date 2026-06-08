use std::collections::BTreeMap;

use chrono::Utc;

use crate::{
    surface::model::{SurfaceLifecycle, SurfaceMetadata},
    view::model::{ViewDefinition, WindowHostKind},
    windowing::labels::{MAIN_WINDOW_LABEL, surface_webview_label},
};

const DETACHED_HOST_ID_PREFIX: &str = "panel";
const SURFACE_ID_PREFIX: &str = "surface";

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
