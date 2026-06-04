use litools_search::{SearchProvider, SearchQuery, SearchResult, SearchResultAction};

#[derive(Clone, Debug)]
pub enum CommandExecution {
    OpenSettings,
    ReloadIndex,
    OpenLogs,
    QuitApp,
    ToggleTheme,
}

#[derive(Default)]
pub struct BuiltinCommandProvider;

impl SearchProvider for BuiltinCommandProvider {
    fn id(&self) -> &'static str {
        "commands"
    }

    fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let commands = [
            ("open-settings", "Open Settings", "Open the settings window"),
            (
                "reload-index",
                "Reload Index",
                "Refresh local search indexes",
            ),
            ("open-logs", "Open Logs", "Open diagnostic logs"),
            ("quit-app", "Quit App", "Exit litools"),
            (
                "toggle-theme",
                "Toggle Theme",
                "Switch between light and dark themes",
            ),
        ];

        let needle = query.text.to_lowercase();
        commands
            .iter()
            .filter(|(_, title, subtitle)| {
                needle.is_empty()
                    || title.to_lowercase().contains(&needle)
                    || subtitle.to_lowercase().contains(&needle)
            })
            .map(|(id, title, subtitle)| SearchResult {
                id: (*id).to_string(),
                title: (*title).to_string(),
                subtitle: Some((*subtitle).to_string()),
                provider: self.id().to_string(),
                score: if title.to_lowercase().starts_with(&needle) {
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
