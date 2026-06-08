use std::{
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use litools_core::{AppBootstrapPaths, LitoolsApp, LitoolsResult, ReloadIndexSummary};
use serde::Serialize;

use crate::{
    app_watcher::{AppWatcherHandle, AppWatcherState, AppWatcherStatus},
    index_refresh::IndexStatus,
    plugin_runtime::{
        model::{
            PluginRuntimeBounds, PluginRuntimeContext, PluginRuntimeLifecycle,
            PluginRuntimePlacement,
        },
        registry::{PluginRuntimeRegistration, PluginRuntimeRegistry},
    },
    surface::{
        model::{SurfaceLifecycle, SurfaceMetadata},
        registry::SurfaceRegistry,
    },
    view::{
        model::{ViewDefinition, WindowHostKind},
        registry,
    },
    windowing::labels::MAIN_WINDOW_LABEL,
};

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
    surfaces: Mutex<SurfaceRegistry>,
    plugin_runtimes: Mutex<PluginRuntimeRegistry>,
}

impl AppState {
    pub fn bootstrap(paths: AppBootstrapPaths) -> LitoolsResult<Self> {
        let data_dir = paths.data_dir.clone();
        Ok(Self {
            app: Mutex::new(LitoolsApp::bootstrap(paths)?),
            data_dir,
            quitting: AtomicBool::new(false),
            shortcut_status: Mutex::new(ShortcutStatus::default()),
            index_status: Mutex::new(IndexStatus::default()),
            app_watcher: AppWatcherState::default(),
            app_watcher_handle: Mutex::new(None),
            surfaces: Mutex::new(SurfaceRegistry::default()),
            plugin_runtimes: Mutex::new(PluginRuntimeRegistry::default()),
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

    pub fn register_surface(
        &self,
        view: ViewDefinition,
        host_window_label: String,
        host_kind: WindowHostKind,
    ) -> Result<SurfaceMetadata, String> {
        self.surfaces
            .lock()
            .map_err(|error| error.to_string())
            .map(|mut registry| registry.register_surface(view, host_window_label, host_kind))
    }

    pub fn register_main_launcher_surface(&self) -> Result<SurfaceMetadata, String> {
        self.register_surface(
            registry::validate_route("/")?,
            MAIN_WINDOW_LABEL.to_string(),
            WindowHostKind::Main,
        )
    }

    pub fn next_detached_host_label(&self) -> Result<String, String> {
        self.surfaces
            .lock()
            .map_err(|error| error.to_string())
            .map(|mut registry| registry.next_detached_host_label())
    }

    pub fn surface_metadata(&self, target: &str) -> Option<SurfaceMetadata> {
        self.surfaces
            .lock()
            .ok()
            .and_then(|registry| registry.metadata(target))
    }

    pub fn surface_metadata_for_webview_label(&self, label: &str) -> Option<SurfaceMetadata> {
        self.surfaces
            .lock()
            .ok()
            .and_then(|registry| registry.metadata_for_webview_label(label))
    }

    pub fn list_surfaces(&self) -> Vec<SurfaceMetadata> {
        self.surfaces
            .lock()
            .map(|registry| registry.list())
            .unwrap_or_default()
    }

    pub fn move_surface_to_host(
        &self,
        webview_label: &str,
        host_window_label: String,
        host_kind: WindowHostKind,
    ) -> Option<SurfaceMetadata> {
        self.surfaces
            .lock()
            .ok()?
            .move_to_host(webview_label, host_window_label, host_kind)
    }

    pub fn mark_surface_route(
        &self,
        webview_label: &str,
        view: ViewDefinition,
    ) -> Option<SurfaceMetadata> {
        self.surfaces.lock().ok()?.mark_route(webview_label, view)
    }

    pub fn mark_surface_lifecycle(
        &self,
        target: &str,
        lifecycle: SurfaceLifecycle,
    ) -> Option<SurfaceMetadata> {
        self.surfaces.lock().ok()?.mark_lifecycle(target, lifecycle)
    }

    pub fn mark_surface_focused(&self, target: &str, focused: bool) -> Option<SurfaceMetadata> {
        self.surfaces.lock().ok()?.mark_focused(target, focused)
    }

    pub fn remove_surface(&self, target: &str) -> Option<SurfaceMetadata> {
        self.surfaces.lock().ok()?.remove(target)
    }

    pub fn next_plugin_runtime_id(&self) -> Result<String, String> {
        self.plugin_runtimes
            .lock()
            .map_err(|error| error.to_string())
            .map(|mut registry| registry.next_runtime_id())
    }

    pub fn register_plugin_runtime(
        &self,
        registration: PluginRuntimeRegistration,
        id: String,
        webview_label: String,
    ) -> Result<PluginRuntimeContext, String> {
        self.plugin_runtimes
            .lock()
            .map_err(|error| error.to_string())?
            .register_runtime(registration, id, webview_label)
    }

    pub fn plugin_runtime(&self, id: &str) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes.lock().ok()?.runtime(id)
    }

    pub fn plugin_runtime_for_webview_label(&self, label: &str) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes
            .lock()
            .ok()?
            .runtime_for_webview_label(label)
    }

    pub fn plugin_runtime_for_window_label(&self, label: &str) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes
            .lock()
            .ok()?
            .runtime_for_window_label(label)
    }

    pub fn plugin_runtime_for_plugin_command(
        &self,
        plugin_id: &str,
        command_id: &str,
    ) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes
            .lock()
            .ok()?
            .runtime_for_plugin_command(plugin_id, command_id)
    }

    pub fn mark_plugin_runtime_lifecycle(
        &self,
        id: &str,
        lifecycle: PluginRuntimeLifecycle,
    ) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes
            .lock()
            .ok()?
            .mark_lifecycle(id, lifecycle)
    }

    pub fn move_plugin_runtime_to_host(
        &self,
        id: &str,
        host_window_label: String,
        placement: PluginRuntimePlacement,
        bounds: Option<PluginRuntimeBounds>,
    ) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes
            .lock()
            .ok()?
            .move_to_host(id, host_window_label, placement, bounds)
    }

    pub fn mark_plugin_runtime_bounds(
        &self,
        id: &str,
        bounds: Option<PluginRuntimeBounds>,
    ) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes.lock().ok()?.mark_bounds(id, bounds)
    }

    pub fn mark_plugin_runtime_title(
        &self,
        id: &str,
        title: String,
    ) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes.lock().ok()?.mark_title(id, title)
    }

    pub fn mark_plugin_runtime_detached_window(
        &self,
        id: &str,
        detached_window_label: Option<String>,
    ) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes
            .lock()
            .ok()?
            .mark_detached_window(id, detached_window_label)
    }

    pub fn mark_plugin_runtime_header_webview(
        &self,
        id: &str,
        header_webview_label: Option<String>,
    ) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes
            .lock()
            .ok()?
            .mark_header_webview(id, header_webview_label)
    }

    pub fn mark_plugin_runtime_ready(&self, id: &str) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes.lock().ok()?.mark_ready(id)
    }

    pub fn mark_plugin_runtime_focus_enter(&self, id: &str) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes.lock().ok()?.mark_focus_enter(id)
    }

    pub fn mark_plugin_runtime_leave(&self, id: &str) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes.lock().ok()?.mark_leave(id)
    }

    pub fn remove_plugin_runtime(&self, target: &str) -> Option<PluginRuntimeContext> {
        self.plugin_runtimes.lock().ok()?.remove(target)
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
