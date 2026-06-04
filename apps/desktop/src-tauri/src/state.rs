use std::{
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use litools_core::{LitoolsApp, LitoolsResult};
use serde::Serialize;

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
}

impl AppState {
    pub fn bootstrap(data_dir: impl AsRef<Path>) -> LitoolsResult<Self> {
        Ok(Self {
            app: Mutex::new(LitoolsApp::bootstrap(data_dir.as_ref())?),
            data_dir: data_dir.as_ref().to_path_buf(),
            quitting: AtomicBool::new(false),
            shortcut_status: Mutex::new(ShortcutStatus::default()),
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
        self.shortcut_status.lock().map(|status| status.clone()).unwrap_or_default()
    }
}
