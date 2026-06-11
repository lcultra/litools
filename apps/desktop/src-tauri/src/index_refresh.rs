use std::time::Duration;

use litools_core::ReloadIndexSummary;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;
pub use litools_config::events::INDEX_STATUS_CHANGED_EVENT;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatus {
    pub running: bool,
    pub pending: bool,
    pub last_trigger: Option<String>,
    pub last_error: Option<String>,
    pub last_summary: Option<ReloadIndexSummary>,
}

impl Default for IndexStatus {
    fn default() -> Self {
        Self {
            running: false,
            pending: false,
            last_trigger: None,
            last_error: None,
            last_summary: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum IndexRefreshTrigger {
    Startup,
    Manual,
    AppWatcher,
}

impl IndexRefreshTrigger {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Manual => "manual",
            Self::AppWatcher => "appWatcher",
        }
    }

    fn debounce_delay(&self) -> Option<Duration> {
        match self {
            Self::AppWatcher => Some(Duration::from_secs(1)),
            Self::Startup | Self::Manual => None,
        }
    }
}

pub fn request_index_refresh(app_handle: &AppHandle, trigger: IndexRefreshTrigger) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(delay) = trigger.debounce_delay() {
            tokio::time::sleep(delay).await;
        }
        run_or_queue_refresh(app_handle, trigger.as_str().to_string());
    });
}

fn run_or_queue_refresh(app_handle: AppHandle, trigger: String) {
    let state = app_handle.state::<AppState>();
    let should_start = state.prepare_index_refresh(&trigger);
    emit_index_status(&app_handle);

    if !should_start {
        return;
    }

    tauri::async_runtime::spawn(async move {
        loop {
            let trigger = app_handle
                .state::<AppState>()
                .index_status()
                .last_trigger
                .unwrap_or_else(|| "manual".to_string());
            let result = run_refresh_once(&app_handle, &trigger);
            if result.is_ok() {
                let data_dir = app_handle.state::<AppState>().data_dir().to_path_buf();
                tauri::async_runtime::spawn_blocking(move || {
                    let _ = crate::protocol::icon_disk_cache::prune_icon_cache(&data_dir);
                });
            }
            let rerun = app_handle.state::<AppState>().finish_index_refresh(result);
            emit_index_status(&app_handle);

            if !rerun {
                break;
            }
        }
    });
}

fn run_refresh_once(app_handle: &AppHandle, trigger: &str) -> Result<ReloadIndexSummary, String> {
    let state = app_handle.state::<AppState>();
    let mut app = state.app().lock().map_err(|error| error.to_string())?;
    app.reload_index_with_trigger(trigger)
        .map_err(|error| error.to_string())
}

pub fn emit_index_status(app_handle: &AppHandle) {
    let status = app_handle.state::<AppState>().index_status();
    let _ = app_handle.emit(INDEX_STATUS_CHANGED_EVENT, status);
}
