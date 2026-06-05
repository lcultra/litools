use std::{
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use litools_core::{LitoolsApp, LitoolsResult, ReloadIndexSummary};
use serde::Serialize;

use crate::{
    app_watcher::{AppWatcherHandle, AppWatcherState, AppWatcherStatus},
    index_refresh::IndexStatus,
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
