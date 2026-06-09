use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};

use litools_index::IndexDatabase;
use litools_settings::{storage::SettingsStore};
use litools_telemetry::init_logging;

use crate::{
    context::AppContext,
    error::LitoolsResult,
    plugin_provider::PluginCommandProvider,
};

const APP_SETTINGS_KEY: &str = "app_settings";
const APPS_INDEX_STATUS_KEY: &str = "apps_last_refresh_status";
const LAUNCHER_RESULT_LIMIT: usize = 20;
const RELOAD_INDEX_TRIGGER_DIRECT: &str = "direct";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReloadIndexSummary {
    pub trigger: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: i64,
    pub commands_upserted: usize,
    pub apps_discovered: usize,
    pub apps_upserted: usize,
    pub apps_removed: usize,
    pub success: bool,
    pub error: Option<String>,
}

pub struct AppBootstrapPaths {
    pub data_dir: PathBuf,
    pub bundled_plugins_dir: Option<PathBuf>,
}

impl AppBootstrapPaths {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            bundled_plugins_dir: None,
        }
    }
}

pub struct LitoolsApp {
    context: AppContext,
    paths: AppBootstrapPaths,
    plugin_command_provider: Arc<PluginCommandProvider>,
}

// ── bootstrap / core accessors ──
impl LitoolsApp {
    pub fn bootstrap(paths: AppBootstrapPaths) -> LitoolsResult<Self> {
        init_logging();

        std::fs::create_dir_all(&paths.data_dir)?;
        let database = IndexDatabase::open(paths.data_dir.join("database.sqlite"))?;
        let settings = crate::app::settings::load_settings(&database)?;
        let plugins = crate::app::plugins::sync_and_load_plugins(&database, &paths)?;
        let (search, plugin_command_provider) =
            crate::app::search::default_search_engine(database.clone());

        Ok(Self {
            context: AppContext::new(database, search, plugins, SettingsStore::new(settings)),
            paths,
            plugin_command_provider,
        })
    }

    pub fn bootstrap_in_memory() -> LitoolsResult<Self> {
        init_logging();

        let database = IndexDatabase::in_memory()?;
        let settings = crate::app::settings::load_settings(&database)?;
        let plugins = crate::app::plugins::load_plugins_from_database(&database)?;
        let (search, plugin_command_provider) =
            crate::app::search::default_search_engine(database.clone());

        Ok(Self {
            context: AppContext::new(database, search, plugins, SettingsStore::new(settings)),
            paths: AppBootstrapPaths::new(""),
            plugin_command_provider,
        })
    }

    pub fn context(&self) -> &AppContext {
        &self.context
    }

    pub fn settings(&self) -> &litools_settings::AppSettings {
        self.context.settings.get()
    }
}

// ── sub-modules each contribute an `impl LitoolsApp { … }` block ──
pub mod execution;
pub mod indexing;
pub mod launcher;
pub mod pinning;
pub mod plugins;
pub mod search;
pub mod settings;

pub use plugins::plugin_route;

#[cfg(test)]
mod tests {
    use litools_settings::{AppSettings, PaletteSettings, SearchSettings, WindowSettings};

    use super::*;

    #[test]
    fn bootstrap_writes_default_settings() {
        let app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        assert_eq!(app.settings(), &AppSettings::default());
    }

    #[test]
    fn launcher_panel_respects_section_settings() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");
        let mut settings = app.settings().clone();
        settings.palette.show_recent = false;
        settings.palette.show_pinned = false;
        app.update_settings(settings).expect("update settings");

        let panel = app.launcher_panel("").expect("launcher panel");

        assert!(panel.sections.is_empty());
    }

    #[test]
    fn pin_result_adds_pinned_section_item() {
        let app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.pin_result("reload-index").expect("pin command");
        let panel = app.launcher_panel("").expect("launcher panel");

        let pinned = panel
            .sections
            .iter()
            .find(|section| section.id == "pinned")
            .expect("pinned section");
        assert_eq!(pinned.items[0].result.id, "reload-index");
        assert!(pinned.items[0].is_pinned);

        app.unpin_result("reload-index").expect("unpin command");
        let panel = app.launcher_panel("").expect("launcher panel");
        assert!(panel.sections.iter().all(|section| section.id != "pinned"));
    }

    #[test]
    fn reorder_pinned_results_updates_launcher_order() {
        let app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.pin_result("reload-index").expect("pin reload");
        app.pin_result("quit-app").expect("pin quit");
        app.reorder_pinned_results(vec![
            "reload-index".to_string(),
            "quit-app".to_string(),
        ])
        .expect("reorder pinned results");

        let panel = app.launcher_panel("").expect("launcher panel");
        let pinned = panel
            .sections
            .iter()
            .find(|section| section.id == "pinned")
            .expect("pinned section");
        let ids = pinned
            .items
            .iter()
            .map(|item| item.result.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["reload-index", "quit-app"]);
    }

    #[test]
    fn launcher_panel_keeps_recent_above_pinned_without_deduplication() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.execute_result("reload-index", "execute")
            .expect("record recent usage");
        app.pin_result("reload-index").expect("pin settings");
        let panel = app.launcher_panel("").expect("launcher panel");

        assert_eq!(panel.sections[0].id, "recent");
        assert_eq!(panel.sections[1].id, "pinned");
        assert_eq!(panel.sections[0].items[0].result.id, "reload-index");
        assert_eq!(panel.sections[1].items[0].result.id, "reload-index");
        assert!(panel.sections[0].items[0].is_pinned);
        assert!(panel.sections[1].items[0].is_pinned);
    }

    #[test]
    fn update_settings_persists_normalized_settings() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");
        let settings = AppSettings {
            theme: "invalid".to_string(),
            palette: PaletteSettings {
                global_hotkey: "".to_string(),
                show_recent: false,
                show_pinned: false,
            },
            search: SearchSettings {
                enabled_providers: vec![],
            },
            window: WindowSettings {
                hide_on_blur: false,
                close_to_tray: false,
                center_on_show: false,
            },
        };

        let updated = app.update_settings(settings).expect("update settings");

        assert_eq!(updated.theme, "system");
        assert_eq!(
            updated.search.enabled_providers,
            ["apps", "commands", "plugins"]
        );
        assert_eq!(app.settings(), &updated);
    }

    #[test]
    fn toggle_theme_persists_theme_change() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.execute_result("toggle-theme", "execute")
            .expect("toggle theme");

        assert_eq!(app.settings().theme, "dark");
        assert_eq!(app.usage_event_count().expect("usage count"), 1);
    }
}
