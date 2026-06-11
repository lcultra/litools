use serde::Serialize;
use tauri::{AppHandle, Manager};

use litools_system::adapter::AppWatchGuard;

use crate::index_refresh::{IndexRefreshTrigger, request_index_refresh};
use crate::state::AppState;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppWatcherStatus {
    pub platform: String,
    pub enabled: bool,
    pub status: String,
    pub error: Option<String>,
}

impl AppWatcherStatus {
    pub fn disabled(message: impl Into<String>) -> Self {
        Self {
            platform: std::env::consts::OS.to_string(),
            enabled: false,
            status: "disabled".to_string(),
            error: Some(message.into()),
        }
    }
}

pub fn start_app_watcher(app_handle: AppHandle) -> Option<AppWatchGuard> {
    let state = app_handle.state::<AppState>();
    let app_handle_for_events = app_handle.clone();

    let guard_result = state.watch_app_dirs(Box::new(move || {
        request_index_refresh(&app_handle_for_events, IndexRefreshTrigger::AppWatcher);
    }));

    match guard_result {
        Ok(guard) => {
            state.set_app_watcher_status(AppWatcherStatus {
                platform: std::env::consts::OS.to_string(),
                enabled: true,
                status: "running".to_string(),
                error: None,
            });
            Some(guard)
        }
        Err(error) => {
            state.set_app_watcher_status(AppWatcherStatus::disabled(error.to_string()));
            None
        }
    }
}

#[derive(Default)]
pub struct AppWatcherState {
    status: std::sync::Mutex<Option<AppWatcherStatus>>,
}

impl AppWatcherState {
    pub fn set_status(&self, status: AppWatcherStatus) {
        if let Ok(mut current) = self.status.lock() {
            *current = Some(status);
        }
    }

    pub fn status(&self) -> AppWatcherStatus {
        self.status
            .lock()
            .ok()
            .and_then(|s| s.clone())
            .unwrap_or_else(|| AppWatcherStatus::disabled("应用监听尚未启动"))
    }
}
