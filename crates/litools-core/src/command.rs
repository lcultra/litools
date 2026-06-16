use serde::Serialize;

use litools_search::{
    FieldMatcher, FieldWeights, SearchProvider, SearchQuery, SearchResult, SearchResultAction,
    SearchResultMatches, VisibleField, match_text,
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
        id: "reload-index",
        title: "重载索引",
        subtitle: "刷新本地搜索索引",
        keywords: &["reload", "index", "refresh", "rebuild"],
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

/// 内置命令的字段匹配权重。
const BUILTIN_COMMAND_WEIGHTS: FieldWeights = FieldWeights {
    exact: 112.0,
    prefix: 100.0,
    contains: 72.0,
    fuzzy_cap: 68.0,
};

impl SearchProvider for BuiltinCommandProvider {
    fn id(&self) -> &'static str {
        "commands"
    }

    fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        BUILTIN_COMMANDS
            .iter()
            .filter(|command| {
                query.text.trim().is_empty() || command.search_match(&query.text).has_match()
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
    let (score, title_ranges, subtitle_ranges) = command.search_match(query).finish();

    SearchResult {
        id: command.id.to_string(),
        title: command.title.to_string(),
        subtitle: Some(command.subtitle.to_string()),
        icon_uri: None,
        provider: "commands".to_string(),
        score,
        matches: SearchResultMatches {
            title: title_ranges,
            subtitle: subtitle_ranges,
        },
        actions: vec![SearchResultAction {
            id: "execute".to_string(),
            label: "执行".to_string(),
        }],
    }
}

impl BuiltinCommandDefinition {
    fn search_match(&self, query: &str) -> FieldMatcher {
        if query.trim().is_empty() {
            return FieldMatcher::with_score(100.0);
        }

        let mut matcher = FieldMatcher::new();
        matcher.consider(
            match_text(self.title, query),
            0.0,
            VisibleField::Title,
            &BUILTIN_COMMAND_WEIGHTS,
        );
        matcher.consider(
            match_text(self.subtitle, query),
            -8.0,
            VisibleField::Subtitle,
            &BUILTIN_COMMAND_WEIGHTS,
        );

        for keyword in self.keywords {
            matcher.consider(
                match_text(keyword, query),
                keyword_score_adjustment(keyword, query),
                VisibleField::Hidden,
                &BUILTIN_COMMAND_WEIGHTS,
            );
        }

        matcher
    }
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
    use litools_search::MatchRange;

    use super::*;

    #[test]
    fn title_match_returns_title_ranges() {
        // BUILTIN_COMMANDS[0] is "reload-index" with title "重载索引"
        let result = search_result_for_builtin_command(&BUILTIN_COMMANDS[0], "重载");

        assert!(result.score > 0.0);
        assert_eq!(result.matches.title, [MatchRange { start: 0, end: 2 }]);
        assert!(result.matches.subtitle.is_empty());
    }

    #[test]
    fn subtitle_match_returns_subtitle_ranges() {
        // BUILTIN_COMMANDS[0] subtitle is "刷新本地搜索索引"
        let result = search_result_for_builtin_command(&BUILTIN_COMMANDS[0], "搜索");

        assert!(result.score > 0.0);
        assert!(result.matches.title.is_empty());
        assert_eq!(result.matches.subtitle, [MatchRange { start: 4, end: 6 }]);
    }

    #[test]
    fn keyword_match_has_no_visible_ranges() {
        // BUILTIN_COMMANDS[0] keyword "reload" matches "reload"
        let result = search_result_for_builtin_command(&BUILTIN_COMMANDS[0], "reload");

        assert!(result.score > 0.0);
        assert!(result.matches.title.is_empty());
        assert!(result.matches.subtitle.is_empty());
    }

    #[test]
    fn fuzzy_keyword_match_finds_command() {
        // BUILTIN_COMMANDS[4] is "toggle-theme" with keyword "toggle"
        let result = search_result_for_builtin_command(&BUILTIN_COMMANDS[4], "tggl");

        assert!(result.score > 0.0);
        assert!(result.matches.title.is_empty());
    }
}

pub fn builtin_effect_for_result(
    result_id: &str,
) -> Result<CommandEffect, crate::error::LitoolsError> {
    match result_id {
        "open-logs-directory" => Ok(CommandEffect::OpenLogsDirectory),
        "open-data-directory" => Ok(CommandEffect::OpenDataDirectory),
        "reload-index" => Ok(CommandEffect::ReloadIndex),
        "quit-app" => Ok(CommandEffect::QuitApp),
        "toggle-theme" => Ok(CommandEffect::ToggleTheme),
        _ => Err(crate::error::LitoolsError::CommandNotFound(
            result_id.to_string(),
        )),
    }
}

pub fn message_for_effect(effect: &CommandEffect) -> &'static str {
    match effect {
        CommandEffect::None => "未执行任何操作",
        CommandEffect::OpenLogsDirectory => "正在打开日志目录",
        CommandEffect::OpenDataDirectory => "正在打开数据目录",
        CommandEffect::OpenPluginView { .. } => "正在打开插件",
        CommandEffect::ReloadIndex => "正在重载索引",
        CommandEffect::QuitApp => "正在退出应用",
        CommandEffect::ToggleTheme => "正在切换主题",
    }
}
