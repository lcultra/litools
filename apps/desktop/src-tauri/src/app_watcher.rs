use std::{path::Path, sync::Mutex};

use serde::Serialize;
use tauri::AppHandle;

use crate::index_refresh::{IndexRefreshTrigger, request_index_refresh};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppWatcherStatus {
    pub platform: String,
    pub enabled: bool,
    pub status: String,
    pub watched_paths: Vec<String>,
    pub error: Option<String>,
}

impl AppWatcherStatus {
    pub fn disabled(message: impl Into<String>) -> Self {
        Self {
            platform: std::env::consts::OS.to_string(),
            enabled: false,
            status: "disabled".to_string(),
            watched_paths: Vec::new(),
            error: Some(message.into()),
        }
    }
}

pub struct AppWatcherHandle {
    status: AppWatcherStatus,
    #[cfg(target_os = "macos")]
    _watcher: Option<notify::RecommendedWatcher>,
}

impl AppWatcherHandle {
    pub fn status(&self) -> AppWatcherStatus {
        self.status.clone()
    }
}

pub fn start_app_watcher(app_handle: AppHandle) -> AppWatcherHandle {
    platform_app_watcher(app_handle)
}

#[cfg(target_os = "macos")]
fn platform_app_watcher(app_handle: AppHandle) -> AppWatcherHandle {
    use litools_system::platform::application_dirs;
    use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

    let watched_dirs = application_dirs()
        .into_iter()
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    let watched_paths = watched_dirs
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();

    let app_handle_for_events = app_handle.clone();
    let watcher_result = RecommendedWatcher::new(
        move |result: notify::Result<Event>| match result {
            Ok(event) if event.paths.iter().any(|path| is_app_related_path(path)) => {
                request_index_refresh(&app_handle_for_events, IndexRefreshTrigger::AppWatcher);
            }
            Ok(_) => {}
            Err(_) => {
                request_index_refresh(&app_handle_for_events, IndexRefreshTrigger::AppWatcher);
            }
        },
        Config::default(),
    );

    let mut watcher = match watcher_result {
        Ok(watcher) => watcher,
        Err(error) => {
            return AppWatcherHandle {
                status: AppWatcherStatus {
                    platform: std::env::consts::OS.to_string(),
                    enabled: false,
                    status: "error".to_string(),
                    watched_paths,
                    error: Some(error.to_string()),
                },
                _watcher: None,
            };
        }
    };

    let mut errors = Vec::new();
    for path in &watched_dirs {
        if let Err(error) = watcher.watch(path, RecursiveMode::Recursive) {
            errors.push(format!("{}: {error}", path.display()));
        }
    }

    let enabled = errors.is_empty() && !watched_paths.is_empty();
    AppWatcherHandle {
        status: AppWatcherStatus {
            platform: std::env::consts::OS.to_string(),
            enabled,
            status: if enabled { "running" } else { "error" }.to_string(),
            watched_paths,
            error: (!errors.is_empty()).then(|| errors.join("; ")),
        },
        _watcher: Some(watcher),
    }
}

#[cfg(not(target_os = "macos"))]
fn platform_app_watcher(_app_handle: AppHandle) -> AppWatcherHandle {
    AppWatcherHandle {
        status: AppWatcherStatus::disabled("应用监听暂未在当前平台实现"),
    }
}

fn is_app_related_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str().to_string_lossy().ends_with(".app"))
}

#[derive(Default)]
pub struct AppWatcherState {
    status: Mutex<Option<AppWatcherStatus>>,
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
            .and_then(|status| status.clone())
            .unwrap_or_else(|| AppWatcherStatus::disabled("应用监听尚未启动"))
    }
}
