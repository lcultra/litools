use serde::Serialize;

use litools_search::{
    MatchKind, MatchRange, SearchProvider, SearchQuery, SearchResult, SearchResultAction,
    SearchResultMatches, TextMatch, match_text,
};

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
        id: "open-diagnostics",
        title: "打开诊断",
        subtitle: "打开诊断信息",
        keywords: &["diagnostics", "debug", "status", "health"],
    },
    BuiltinCommandDefinition {
        id: "open-plugins",
        title: "打开插件管理",
        subtitle: "打开插件中心",
        keywords: &["plugin", "plugins", "extension", "manage", "center"],
    },
    BuiltinCommandDefinition {
        id: "open-logs-directory",
        title: "打开日志目录",
        subtitle: "在系统文件管理器中打开日志目录",
        keywords: &["logs", "log", "directory", "folder", "debug"],
    },
    BuiltinCommandDefinition {
        id: "open-data-directory",
        title: "打开数据目录",
        subtitle: "在系统文件管理器中打开本地数据目录",
        keywords: &["data", "directory", "folder", "storage", "database"],
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
pub enum CommandEffect {
    None,
    OpenSettings,
    OpenDiagnostics,
    OpenPlugins,
    OpenLogsDirectory,
    OpenDataDirectory,
    OpenPluginView {
        plugin_id: String,
        command_id: String,
        route: String,
    },
    ReloadIndex,
    QuitApp,
    ToggleTheme,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecution {
    pub result_id: String,
    pub action_id: String,
    pub message: String,
    pub effect: CommandEffect,
}

#[derive(Default)]
pub struct BuiltinCommandProvider;

impl SearchProvider for BuiltinCommandProvider {
    fn id(&self) -> &'static str {
        "commands"
    }

    fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        BUILTIN_COMMANDS
            .iter()
            .filter(|command| {
                query.text.trim().is_empty() || command.search_match(&query.text).is_some()
            })
            .map(|command| search_result_for_builtin_command(command, &query.text))
            .collect()
    }
}

pub fn find_builtin_command(result_id: &str) -> Option<&'static BuiltinCommandDefinition> {
    BUILTIN_COMMANDS
        .iter()
        .find(|command| command.id == result_id)
}

pub fn search_result_for_builtin_command(
    command: &BuiltinCommandDefinition,
    query: &str,
) -> SearchResult {
    let command_match = command.search_match(query).unwrap_or_default();

    SearchResult {
        id: command.id.to_string(),
        title: command.title.to_string(),
        subtitle: Some(command.subtitle.to_string()),
        icon_uri: None,
        provider: "commands".to_string(),
        score: command_match.score,
        matches: SearchResultMatches {
            title: command_match.title_ranges,
            subtitle: command_match.subtitle_ranges,
        },
        actions: vec![SearchResultAction {
            id: "execute".to_string(),
            label: "执行".to_string(),
        }],
    }
}

#[derive(Default)]
struct CommandSearchMatch {
    score: f32,
    title_ranges: Vec<MatchRange>,
    subtitle_ranges: Vec<MatchRange>,
}

impl BuiltinCommandDefinition {
    fn search_match(&self, query: &str) -> Option<CommandSearchMatch> {
        if query.trim().is_empty() {
            return Some(CommandSearchMatch {
                score: 100.0,
                ..CommandSearchMatch::default()
            });
        }

        let mut best: Option<CommandSearchMatch> = None;
        consider_command_match(
            &mut best,
            match_text(self.title, query),
            0.0,
            VisibleCommandField::Title,
        );
        consider_command_match(
            &mut best,
            match_text(self.subtitle, query),
            -8.0,
            VisibleCommandField::Subtitle,
        );

        for keyword in self.keywords {
            consider_command_match(
                &mut best,
                match_text(keyword, query),
                keyword_score_adjustment(keyword, query),
                VisibleCommandField::Hidden,
            );
        }

        best
    }
}

#[derive(Clone, Copy)]
enum VisibleCommandField {
    Hidden,
    Subtitle,
    Title,
}

fn consider_command_match(
    best: &mut Option<CommandSearchMatch>,
    text_match: Option<TextMatch>,
    adjustment: f32,
    visible_field: VisibleCommandField,
) {
    let Some(text_match) = text_match else {
        return;
    };
    let score = command_match_score(&text_match, adjustment);
    if best.as_ref().is_some_and(|current| current.score >= score) {
        return;
    }

    *best = Some(CommandSearchMatch {
        score,
        title_ranges: matches!(visible_field, VisibleCommandField::Title)
            .then_some(text_match.ranges.clone())
            .unwrap_or_default(),
        subtitle_ranges: matches!(visible_field, VisibleCommandField::Subtitle)
            .then_some(text_match.ranges)
            .unwrap_or_default(),
    });
}

fn command_match_score(text_match: &TextMatch, adjustment: f32) -> f32 {
    let base = match text_match.kind {
        MatchKind::Exact => 112.0,
        MatchKind::Prefix => 100.0,
        MatchKind::Contains => 72.0,
        MatchKind::Fuzzy => text_match.score.min(68.0),
    };

    (base + adjustment).max(1.0)
}

fn keyword_score_adjustment(keyword: &str, query: &str) -> f32 {
    if keyword.starts_with(&query.trim().to_lowercase()) {
        -10.0
    } else {
        -18.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_match_returns_title_ranges() {
        let result = search_result_for_builtin_command(&BUILTIN_COMMANDS[0], "设置");

        assert!(result.score > 0.0);
        assert_eq!(result.matches.title, [MatchRange { start: 2, end: 4 }]);
        assert!(result.matches.subtitle.is_empty());
    }

    #[test]
    fn subtitle_match_returns_subtitle_ranges() {
        let result = search_result_for_builtin_command(&BUILTIN_COMMANDS[0], "窗口");

        assert!(result.score > 0.0);
        assert!(result.matches.title.is_empty());
        assert_eq!(result.matches.subtitle, [MatchRange { start: 4, end: 6 }]);
    }

    #[test]
    fn keyword_match_has_no_visible_ranges() {
        let result = search_result_for_builtin_command(&BUILTIN_COMMANDS[0], "settings");

        assert!(result.score > 0.0);
        assert!(result.matches.title.is_empty());
        assert!(result.matches.subtitle.is_empty());
    }

    #[test]
    fn fuzzy_keyword_match_finds_command() {
        let result = search_result_for_builtin_command(&BUILTIN_COMMANDS[3], "plg");

        assert!(result.score > 0.0);
        assert!(result.matches.title.is_empty());
    }
}
