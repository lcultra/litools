use std::sync::Mutex;

use litools_config::search::{ACTION_OPEN, PLUGIN_PROVIDER_ID};
use litools_index::{
    IndexDatabase,
    repository::{PluginCommandRecord, PluginCommandRepository},
};
use litools_plugin::{PluginCommandMode, plugin_command_mode_from_str, plugin_result_id};
use litools_search::{
    FieldMatcher, FieldWeights, SearchProvider, SearchQuery, SearchResult, SearchResultAction,
    SearchResultMatches, VisibleField, match_text,
};

pub struct PluginCommandProvider {
    database: IndexDatabase,
    cache: Mutex<Option<Vec<PluginCommandRecord>>>,
}

impl PluginCommandProvider {
    pub fn new(database: IndexDatabase) -> Self {
        Self {
            database,
            cache: Mutex::new(None),
        }
    }

    /// Invalidate the cached plugin command list so the next search reloads from DB.
    pub fn invalidate_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            *cache = None;
        }
    }

    fn load_commands(&self) -> Vec<PluginCommandRecord> {
        let connection = self.database.connection();
        PluginCommandRepository::new(&connection)
            .list_enabled_plugin_commands()
            .unwrap_or_default()
    }

    fn cached_commands(&self) -> Vec<PluginCommandRecord> {
        let mut cache = self.cache.lock().expect("plugin provider cache lock");
        if cache.is_none() {
            *cache = Some(self.load_commands());
        }
        cache.clone().unwrap_or_default()
    }
}

impl SearchProvider for PluginCommandProvider {
    fn id(&self) -> &'static str {
        PLUGIN_PROVIDER_ID
    }

    fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        self.cached_commands()
            .into_iter()
            .filter(|command| command_mode(command) == Some(PluginCommandMode::View))
            .filter_map(|command| search_result_for_plugin_command(command, &query.text))
            .collect()
    }
}

/// 插件命令的字段匹配权重。
const PLUGIN_COMMAND_WEIGHTS: FieldWeights = FieldWeights {
    exact: 108.0,
    prefix: 96.0,
    contains: 70.0,
    fuzzy_cap: 64.0,
};

pub fn search_result_for_plugin_command_record(
    command: PluginCommandRecord,
    query: &str,
) -> SearchResult {
    let (score, title_ranges, subtitle_ranges) =
        plugin_command_search_match(&command, query).finish();
    SearchResult {
        id: plugin_result_id(&command.plugin_id, &command.command_id),
        title: command.title,
        subtitle: command.subtitle.or(Some(command.plugin_name)),
        icon_uri: Some(plugin_icon_uri(&command.plugin_id, &command.plugin_icon)),
        provider: PLUGIN_PROVIDER_ID.to_string(),
        score,
        matches: SearchResultMatches {
            title: title_ranges,
            subtitle: subtitle_ranges,
        },
        actions: vec![SearchResultAction {
            id: ACTION_OPEN.to_string(),
            label: "打开".to_string(),
        }],
    }
}

fn plugin_icon_uri(plugin_id: &str, icon: &str) -> String {
    format!("litools-plugin://{plugin_id}/{icon}")
}

fn search_result_for_plugin_command(
    command: PluginCommandRecord,
    query: &str,
) -> Option<SearchResult> {
    if query.trim().is_empty() || plugin_command_search_match(&command, query).has_match() {
        return Some(search_result_for_plugin_command_record(command, query));
    }
    None
}

fn command_mode(command: &PluginCommandRecord) -> Option<PluginCommandMode> {
    plugin_command_mode_from_str(&command.mode)
}

fn plugin_command_search_match(
    command: &PluginCommandRecord,
    query: &str,
) -> FieldMatcher {
    if query.trim().is_empty() {
        return FieldMatcher::with_score(95.0);
    }

    let mut matcher = FieldMatcher::new();
    matcher.consider(
        match_text(&command.title, query),
        0.0,
        VisibleField::Title,
        &PLUGIN_COMMAND_WEIGHTS,
    );
    if let Some(subtitle) = &command.subtitle {
        matcher.consider(
            match_text(subtitle, query),
            -8.0,
            VisibleField::Subtitle,
            &PLUGIN_COMMAND_WEIGHTS,
        );
    }
    matcher.consider(
        match_text(&command.plugin_name, query),
        -12.0,
        VisibleField::Subtitle,
        &PLUGIN_COMMAND_WEIGHTS,
    );
    for keyword in &command.keywords {
        matcher.consider(
            match_text(keyword, query),
            -16.0,
            VisibleField::Hidden,
            &PLUGIN_COMMAND_WEIGHTS,
        );
    }
    matcher.consider(
        match_text(&command.plugin_id, query),
        -20.0,
        VisibleField::Hidden,
        &PLUGIN_COMMAND_WEIGHTS,
    );
    matcher.consider(
        match_text(&command.command_id, query),
        -20.0,
        VisibleField::Hidden,
        &PLUGIN_COMMAND_WEIGHTS,
    );

    matcher
}

#[cfg(test)]
mod tests {
    // Plugin ID parsing tests live in litools-plugin::ids.
}
