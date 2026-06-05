use serde::Serialize;

use litools_search::{SearchProvider, SearchQuery, SearchResult, SearchResultAction};

#[derive(Clone, Copy, Debug)]
pub struct BuiltinCommandDefinition {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub keywords: &'static [&'static str],
}

pub const BUILTIN_COMMANDS: &[BuiltinCommandDefinition] = &[
    BuiltinCommandDefinition {
        id: "open-settings",
        title: "打开设置",
        subtitle: "打开设置窗口",
        keywords: &["settings", "open", "preferences", "config"],
    },
    BuiltinCommandDefinition {
        id: "reload-index",
        title: "重载索引",
        subtitle: "刷新本地搜索索引",
        keywords: &["reload", "index", "refresh", "rebuild"],
    },
    BuiltinCommandDefinition {
        id: "open-logs",
        title: "打开诊断",
        subtitle: "打开诊断信息",
        keywords: &["logs", "diagnostics", "debug", "status"],
    },
    BuiltinCommandDefinition {
        id: "quit-app",
        title: "退出应用",
        subtitle: "退出 litools",
        keywords: &["quit", "exit", "close"],
    },
    BuiltinCommandDefinition {
        id: "toggle-theme",
        title: "切换主题",
        subtitle: "在浅色和深色主题之间切换",
        keywords: &["theme", "toggle", "dark", "light"],
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
            .filter(|command| command.matches(&needle))
            .map(|command| SearchResult {
                id: command.id.to_string(),
                title: command.title.to_string(),
                subtitle: Some(command.subtitle.to_string()),
                provider: self.id().to_string(),
                score: command.score(&needle),
                actions: vec![SearchResultAction {
                    id: "execute".to_string(),
                    label: "执行".to_string(),
                }],
            })
            .collect()
    }
}

impl BuiltinCommandDefinition {
    fn matches(&self, needle: &str) -> bool {
        needle.is_empty()
            || self.title.to_lowercase().contains(needle)
            || self.subtitle.to_lowercase().contains(needle)
            || self.keywords.iter().any(|keyword| keyword.contains(needle))
    }

    fn score(&self, needle: &str) -> f32 {
        if needle.is_empty() || self.title.to_lowercase().starts_with(needle) {
            return 100.0;
        }

        if self
            .keywords
            .iter()
            .any(|keyword| keyword.starts_with(needle))
        {
            return 90.0;
        }

        50.0
    }
}
