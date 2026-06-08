use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use chrono::Utc;
use litools_core::{LitoolsApp, LitoolsResult, ReloadIndexSummary};
use serde::Serialize;

use crate::{
    app_watcher::{AppWatcherHandle, AppWatcherState, AppWatcherStatus},
    index_refresh::IndexStatus,
    window::MAIN_WINDOW_LABEL,
};

const DETACHED_SURFACE_ID_PREFIX: &str = "dw";
const MAIN_SURFACE_ID_PREFIX: &str = "main";
const SURFACE_WEBVIEW_LABEL_PREFIX: &str = "surface-";
const DETACHED_WINDOW_LABEL_PREFIX: &str = "detached-management-";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManagedWindowKind {
    Main,
    DetachedManagement,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManagedWindowLifecycle {
    Active,
    Hidden,
    Destroyed,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedWindowMetadata {
    pub id: String,
    pub webview_label: String,
    pub owner_window_label: String,
    pub kind: ManagedWindowKind,
    pub route: Option<String>,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub lifecycle: ManagedWindowLifecycle,
    pub focused: bool,
}

#[derive(Debug)]
struct WindowRegistry {
    next_detached_index: u64,
    next_main_surface_index: u64,
    surfaces_by_webview_label: BTreeMap<String, ManagedWindowMetadata>,
}

impl Default for WindowRegistry {
    fn default() -> Self {
        Self {
            next_detached_index: 1,
            next_main_surface_index: 1,
            surfaces_by_webview_label: BTreeMap::new(),
        }
    }
}

impl WindowRegistry {
    fn now() -> String {
        Utc::now().to_rfc3339()
    }

    fn next_main_surface_metadata(&mut self) -> ManagedWindowMetadata {
        let id = format!("{MAIN_SURFACE_ID_PREFIX}_{:06}", self.next_main_surface_index);
        self.next_main_surface_index += 1;
        let webview_label = format!("{SURFACE_WEBVIEW_LABEL_PREFIX}{id}");
        let now = Self::now();
        let metadata = ManagedWindowMetadata {
            id,
            webview_label: webview_label.clone(),
            owner_window_label: MAIN_WINDOW_LABEL.to_string(),
            kind: ManagedWindowKind::Main,
            route: Some("/".to_string()),
            title: "litools".to_string(),
            created_at: now.clone(),
            updated_at: now,
            lifecycle: ManagedWindowLifecycle::Active,
            focused: false,
        };
        self.surfaces_by_webview_label
            .insert(webview_label, metadata.clone());
        metadata
    }

    fn next_detached_window_label(&mut self) -> String {
        let label = format!("{DETACHED_WINDOW_LABEL_PREFIX}{DETACHED_SURFACE_ID_PREFIX}_{:06}", self.next_detached_index);
        self.next_detached_index += 1;
        label
    }

    fn get_by_webview_label_or_id_or_window_label(&self, target: &str) -> Option<ManagedWindowMetadata> {
        self.surfaces_by_webview_label.get(target).cloned().or_else(|| {
            self.surfaces_by_webview_label
                .values()
                .find(|metadata| metadata.id == target || metadata.owner_window_label == target)
                .cloned()
        })
    }

    fn update_by_webview_label(
        &mut self,
        webview_label: &str,
        update: impl FnOnce(&mut ManagedWindowMetadata),
    ) -> Option<ManagedWindowMetadata> {
        let metadata = self.surfaces_by_webview_label.get_mut(webview_label)?;
        update(metadata);
        metadata.updated_at = Self::now();
        Some(metadata.clone())
    }

    fn remove_by_webview_label_or_id_or_window_label(&mut self, target: &str) -> Option<ManagedWindowMetadata> {
        if self.surfaces_by_webview_label.contains_key(target) {
            return self.surfaces_by_webview_label.remove(target);
        }

        let webview_label = self
            .surfaces_by_webview_label
            .iter()
            .find_map(|(webview_label, metadata)| {
                (metadata.id == target || metadata.owner_window_label == target).then(|| webview_label.clone())
            })?;
        self.surfaces_by_webview_label.remove(&webview_label)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ShortcutStatus {
    pub accelerator: String,
    pub registered: bool,
    pub error: Option<String>,
}

impl Default for ShortcutStatus {
    fn default() -> Self {
        Self {
            accelerator: "CommandOrControl+Space".to_string(),
            registered: false,
            error: None,
        }
    }
}

pub struct AppState {
    app: Mutex<LitoolsApp>,
    data_dir: PathBuf,
    quitting: AtomicBool,
    shortcut_status: Mutex<ShortcutStatus>,
    index_status: Mutex<IndexStatus>,
    app_watcher: AppWatcherState,
    app_watcher_handle: Mutex<Option<AppWatcherHandle>>,
    windows: Mutex<WindowRegistry>,
}

impl AppState {
    pub fn bootstrap(data_dir: impl AsRef<Path>) -> LitoolsResult<Self> {
        Ok(Self {
            app: Mutex::new(LitoolsApp::bootstrap(data_dir.as_ref())?),
            data_dir: data_dir.as_ref().to_path_buf(),
            quitting: AtomicBool::new(false),
            shortcut_status: Mutex::new(ShortcutStatus::default()),
            index_status: Mutex::new(IndexStatus::default()),
            app_watcher: AppWatcherState::default(),
            app_watcher_handle: Mutex::new(None),
            windows: Mutex::new(WindowRegistry::default()),
        })
    }

    pub fn app(&self) -> &Mutex<LitoolsApp> {
        &self.app
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn request_quit(&self) {
        self.quitting.store(true, Ordering::SeqCst);
    }

    pub fn is_quitting(&self) -> bool {
        self.quitting.load(Ordering::SeqCst)
    }

    pub fn close_to_tray(&self) -> bool {
        self.app
            .lock()
            .map(|app| app.settings().window.close_to_tray)
            .unwrap_or(true)
    }

    pub fn hide_on_blur(&self) -> bool {
        self.app
            .lock()
            .map(|app| app.settings().window.hide_on_blur)
            .unwrap_or(true)
    }

    pub fn center_on_show(&self) -> bool {
        self.app
            .lock()
            .map(|app| app.settings().window.center_on_show)
            .unwrap_or(true)
    }

    pub fn global_hotkey(&self) -> String {
        self.app
            .lock()
            .map(|app| app.settings().palette.global_hotkey.clone())
            .unwrap_or_else(|_| "CommandOrControl+Space".to_string())
    }

    pub fn register_main_surface(&self) -> Result<ManagedWindowMetadata, String> {
        self.windows
            .lock()
            .map_err(|error| error.to_string())
            .map(|mut registry| registry.next_main_surface_metadata())
    }

    pub fn next_detached_window_label(&self) -> Result<String, String> {
        self.windows
            .lock()
            .map_err(|error| error.to_string())
            .map(|mut registry| registry.next_detached_window_label())
    }

    pub fn window_metadata(&self, target: &str) -> Option<ManagedWindowMetadata> {
        self.windows
            .lock()
            .ok()
            .and_then(|registry| registry.get_by_webview_label_or_id_or_window_label(target))
    }

    pub fn window_metadata_for_webview_label(&self, webview_label: &str) -> Option<ManagedWindowMetadata> {
        self.window_metadata(webview_label)
    }

    pub fn list_windows(&self) -> Vec<ManagedWindowMetadata> {
        self.windows
            .lock()
            .map(|registry| registry.surfaces_by_webview_label.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn move_surface_to_window(
        &self,
        webview_label: &str,
        owner_window_label: String,
        kind: ManagedWindowKind,
        route: Option<String>,
    ) -> Option<ManagedWindowMetadata> {
        self.windows.lock().ok()?.update_by_webview_label(webview_label, |metadata| {
            metadata.owner_window_label = owner_window_label;
            metadata.kind = kind;
            metadata.route = route;
            metadata.lifecycle = ManagedWindowLifecycle::Active;
            metadata.focused = true;
        })
    }

    pub fn mark_window_route(&self, webview_label: &str, route: String) -> Option<ManagedWindowMetadata> {
        self.windows.lock().ok()?.update_by_webview_label(webview_label, |metadata| {
            metadata.route = Some(route);
        })
    }

    pub fn mark_window_lifecycle(
        &self,
        target: &str,
        lifecycle: ManagedWindowLifecycle,
    ) -> Option<ManagedWindowMetadata> {
        let webview_label = self.window_metadata(target)?.webview_label;
        self.windows.lock().ok()?.update_by_webview_label(&webview_label, |metadata| {
            metadata.lifecycle = lifecycle;
        })
    }

    pub fn mark_window_focused(&self, target: &str, focused: bool) -> Option<ManagedWindowMetadata> {
        let webview_label = self.window_metadata(target)?.webview_label;
        self.windows.lock().ok()?.update_by_webview_label(&webview_label, |metadata| {
            metadata.focused = focused;
        })
    }

    pub fn remove_window(&self, target: &str) -> Option<ManagedWindowMetadata> {
        self.windows
            .lock()
            .ok()
            .and_then(|mut registry| registry.remove_by_webview_label_or_id_or_window_label(target))
    }

    pub fn set_shortcut_status(&self, status: ShortcutStatus) {
        if let Ok(mut shortcut_status) = self.shortcut_status.lock() {
            *shortcut_status = status;
        }
    }

    pub fn shortcut_status(&self) -> ShortcutStatus {
        self.shortcut_status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_default()
    }

    pub fn prepare_index_refresh(&self, trigger: &str) -> bool {
        let Ok(mut status) = self.index_status.lock() else {
            return false;
        };

        status.last_trigger = Some(trigger.to_string());
        status.last_error = None;
        if status.running {
            status.pending = true;
            return false;
        }

        status.running = true;
        status.pending = false;
        true
    }

    pub fn finish_index_refresh(&self, result: Result<ReloadIndexSummary, String>) -> bool {
        let Ok(mut status) = self.index_status.lock() else {
            return false;
        };

        match result {
            Ok(summary) => {
                status.last_error = None;
                status.last_summary = Some(summary);
            }
            Err(error) => {
                status.last_error = Some(error);
            }
        }

        let rerun = status.pending;
        status.running = rerun;
        status.pending = false;
        rerun
    }

    pub fn index_status(&self) -> IndexStatus {
        self.index_status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_default()
    }

    pub fn set_app_watcher(&self, handle: AppWatcherHandle) {
        self.app_watcher.set_status(handle.status());
        if let Ok(mut current) = self.app_watcher_handle.lock() {
            *current = Some(handle);
        }
    }

    pub fn app_watcher_status(&self) -> AppWatcherStatus {
        self.app_watcher.status()
    }
}
