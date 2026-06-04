use serde::Serialize;

use litools_search::{SearchProvider, SearchQuery, SearchResult, SearchResultAction};

#[derive(Clone, Copy, Debug)]
pub struct BuiltinCommandDefinition {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
}

pub const BUILTIN_COMMANDS: &[BuiltinCommandDefinition] = &[
    BuiltinCommandDefinition {
        id: "open-settings",
        title: "Open Settings",
        subtitle: "Open the settings window",
    },
    BuiltinCommandDefinition {
        id: "reload-index",
        title: "Reload Index",
        subtitle: "Refresh local search indexes",
    },
    BuiltinCommandDefinition {
        id: "open-logs",
        title: "Open Logs",
        subtitle: "Open diagnostic logs",
    },
    BuiltinCommandDefinition {
        id: "quit-app",
        title: "Quit App",
        subtitle: "Exit litools",
    },
    BuiltinCommandDefinition {
        id: "toggle-theme",
        title: "Toggle Theme",
        subtitle: "Switch between light and dark themes",
    },
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BuiltinCommandEffect {
    None,
    OpenSettings,
    ReloadIndex,
    OpenLogs,
    QuitApp,
    ToggleTheme,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecution {
    pub result_id: String,
    pub action_id: String,
    pub message: String,
    pub effect: BuiltinCommandEffect,
}

#[derive(Default)]
pub struct BuiltinCommandProvider;

impl SearchProvider for BuiltinCommandProvider {
    fn id(&self) -> &'static str {
        "commands"
    }

    fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let needle = query.text.to_lowercase();
        BUILTIN_COMMANDS
            .iter()
            .filter(|command| {
                needle.is_empty()
                    || command.title.to_lowercase().contains(&needle)
                    || command.subtitle.to_lowercase().contains(&needle)
            })
            .map(|command| SearchResult {
                id: command.id.to_string(),
                title: command.title.to_string(),
                subtitle: Some(command.subtitle.to_string()),
                provider: self.id().to_string(),
                score: if command.title.to_lowercase().starts_with(&needle) {
                    100.0
                } else {
                    50.0
                },
                actions: vec![SearchResultAction {
                    id: "execute".to_string(),
                    label: "Execute".to_string(),
                }],
            })
            .collect()
    }
}
