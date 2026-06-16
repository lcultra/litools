use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, SystemTime},
};

use litools_core::{AppBootstrapPaths, LitoolsApp, LitoolsResult, ReloadIndexSummary};
use litools_system::SystemAdapter;
use serde::Serialize;

use crate::{
    app_watcher::{AppWatcherState, AppWatcherStatus},
    background::manager::{BackgroundRuntimeManager, RuntimePolicy},
    core::events::PluginEventBus,
    core::executor::{BackgroundRuntimeExecutor, ExecutorRegistry, WebviewExecutor},
    core::plugins::runtime::registry::PluginRuntimeRegistry,
    core::surface::registry::SurfaceRegistry,
    index_refresh::IndexStatus,
};

#[derive(Clone, Debug, Serialize)]
pub struct ShortcutStatus {
    pub accelerator: String,
    pub registered: bool,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LauncherMonitorFingerprint {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale_millis: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LauncherWindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LauncherSavedPosition {
    pub monitor: LauncherMonitorFingerprint,
    pub position: LauncherWindowPosition,
}

#[derive(Default)]
struct LauncherPositioningState {
    saved_position: Option<LauncherSavedPosition>,
    /// 程序化布局截止时间（SystemTime nanos）。当前时间 < 此值时，
    /// 所有 WindowEvent::Moved 被视为程序控制，不写入位置记忆。
    ignore_moved_until: AtomicU64,
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
    pub surfaces: Mutex<SurfaceRegistry>,
    pub plugin_runtimes: Mutex<PluginRuntimeRegistry>,
    pub plugin_events: PluginEventBus,
    pub bg_runtime_manager: Arc<BackgroundRuntimeManager>,
    pub executor_registry: ExecutorRegistry,
    launcher_positioning: Mutex<LauncherPositioningState>,
    /// Pre-created detached window ready for instant plugin detach.
    pooled_detached: Mutex<Option<String>>,
}

impl AppState {
    pub fn bootstrap(paths: AppBootstrapPaths) -> LitoolsResult<Self> {
        let data_dir = paths.data_dir.clone();
        let bg_runtime_manager = Arc::new(BackgroundRuntimeManager::new(
            RuntimePolicy::PerPlugin,
            Duration::from_secs(300),
        ));

        let mut executor_registry = ExecutorRegistry::new();
        executor_registry.register("webview", Box::new(WebviewExecutor));
        executor_registry.register(
            "backgroundRuntime",
            Box::new(BackgroundRuntimeExecutor::new(bg_runtime_manager.clone())),
        );

        Ok(Self {
            app: Mutex::new(LitoolsApp::bootstrap(paths)?),
            data_dir,
            quitting: AtomicBool::new(false),
            shortcut_status: Mutex::new(ShortcutStatus::default()),
            index_status: Mutex::new(IndexStatus::default()),
            app_watcher: AppWatcherState::default(),
            surfaces: Mutex::new(SurfaceRegistry::default()),
            plugin_runtimes: Mutex::new(PluginRuntimeRegistry::default()),
            plugin_events: PluginEventBus::new(),
            bg_runtime_manager,
            executor_registry,
            launcher_positioning: Mutex::new(LauncherPositioningState::default()),
            pooled_detached: Mutex::new(None),
        })
    }

    pub fn app(&self) -> &Mutex<LitoolsApp> {
        &self.app
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn app_icon_png(&self, path: &std::path::Path) -> std::io::Result<Vec<u8>> {
        self.app
            .lock()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
            .system_adapter()
            .app_icon_png(path)
    }

    pub fn application_dirs(&self) -> Vec<std::path::PathBuf> {
        self.app
            .lock()
            .map(|app| app.system_adapter().application_dirs())
            .unwrap_or_default()
    }

    pub fn watch_app_dirs(
        &self,
        on_change: Box<dyn Fn() + Send + 'static>,
    ) -> std::io::Result<litools_system::adapter::AppWatchGuard> {
        let app = self
            .app
            .lock()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        app.system_adapter().watch_app_dirs(on_change)
    }

    pub fn set_app_watcher_status(&self, status: AppWatcherStatus) {
        self.app_watcher.set_status(status);
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

    pub fn launcher_saved_position(&self) -> Option<LauncherSavedPosition> {
        self.launcher_positioning
            .lock()
            .ok()
            .and_then(|state| state.saved_position)
    }

    pub fn replace_launcher_saved_position(&self, saved_position: LauncherSavedPosition) {
        if let Ok(mut state) = self.launcher_positioning.lock() {
            state.saved_position = Some(saved_position);
        }
    }

    #[allow(dead_code)]
    pub fn clear_launcher_saved_position(&self) {
        if let Ok(mut state) = self.launcher_positioning.lock() {
            state.saved_position = None;
        }
    }

    /// 开启程序化布局窗口（100ms）。窗口内所有 WindowEvent::Moved 视为程序控制，
    /// 不会被 save_launcher_position 记为用户拖拽。
    pub fn begin_programmatic_layout(&self) {
        let deadline = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .saturating_add(Duration::from_millis(100));
        if let Ok(state) = self.launcher_positioning.lock() {
            state
                .ignore_moved_until
                .store(deadline.as_nanos() as u64, Ordering::SeqCst);
        }
    }

    /// 当前是否处于程序化布局窗口内。
    pub fn is_programmatic_layout(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.launcher_positioning
            .lock()
            .ok()
            .map(|state| state.ignore_moved_until.load(Ordering::Relaxed) > now)
            .unwrap_or(false)
    }

    pub fn global_hotkey(&self) -> String {
        self.app
            .lock()
            .map(|app| app.settings().palette.global_hotkey.clone())
            .unwrap_or_else(|_| "CommandOrControl+Space".to_string())
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

    pub fn app_watcher_status(&self) -> AppWatcherStatus {
        self.app_watcher.status()
    }

    pub fn take_pooled_detached(&self) -> Option<String> {
        self.pooled_detached.lock().ok()?.take()
    }

    pub fn return_pooled_detached(&self, label: String) {
        if let Ok(mut pooled) = self.pooled_detached.lock() {
            *pooled = Some(label);
        }
    }

    // ── 锁辅助方法 ──
    //
    // 锁获取顺序（固定，防止死锁）：
    //   1. surfaces
    //   2. plugin_runtimes
    //   3. app
    // 跨方法需要同时持有多把锁时，始终按此顺序获取。

    /// 在 surface 读锁内执行闭包。
    pub fn with_surfaces<T>(&self, f: impl FnOnce(&SurfaceRegistry) -> T) -> Result<T, String> {
        self.surfaces
            .lock()
            .map(|guard| f(&guard))
            .map_err(|e| e.to_string())
    }

    /// 在 surface 写锁内执行闭包。
    pub fn with_surfaces_mut<T>(
        &self,
        f: impl FnOnce(&mut SurfaceRegistry) -> T,
    ) -> Result<T, String> {
        self.surfaces
            .lock()
            .map(|mut guard| f(&mut guard))
            .map_err(|e| e.to_string())
    }

    /// 在 runtime 读锁内执行闭包。
    pub fn with_runtimes<T>(
        &self,
        f: impl FnOnce(&PluginRuntimeRegistry) -> T,
    ) -> Result<T, String> {
        self.plugin_runtimes
            .lock()
            .map(|guard| f(&guard))
            .map_err(|e| e.to_string())
    }

    /// 在 runtime 写锁内执行闭包。
    pub fn with_runtimes_mut<T>(
        &self,
        f: impl FnOnce(&mut PluginRuntimeRegistry) -> T,
    ) -> Result<T, String> {
        self.plugin_runtimes
            .lock()
            .map(|mut guard| f(&mut guard))
            .map_err(|e| e.to_string())
    }

    /// 同时持有 surfaces + runtimes 读锁（按固定顺序 surfaces → runtimes）。
    pub fn with_surfaces_and_runtimes<T>(
        &self,
        f: impl FnOnce(&SurfaceRegistry, &PluginRuntimeRegistry) -> T,
    ) -> Result<T, String> {
        let surfaces = self.surfaces.lock().map_err(|e| e.to_string())?;
        let runtimes = self.plugin_runtimes.lock().map_err(|e| e.to_string())?;
        Ok(f(&surfaces, &runtimes))
    }
}
