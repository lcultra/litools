use std::sync::Mutex;

use litools_index::{
    IndexDatabase,
    repository::{PluginCommandRecord, PluginCommandRepository},
};
use litools_plugin::{
    PluginCommandMode, plugin_command_mode_from_str, plugin_result_id,
};
use litools_search::{
    MatchKind, MatchRange, SearchProvider, SearchQuery, SearchResult, SearchResultAction,
    SearchResultMatches, TextMatch, match_text,
};

pub const PLUGIN_PROVIDER_ID: &str = "plugins";
pub const OPEN_PLUGIN_ACTION_ID: &str = "open";

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

pub fn search_result_for_plugin_command_record(
    command: PluginCommandRecord,
    query: &str,
) -> SearchResult {
    let command_match = plugin_command_search_match(&command, query).unwrap_or_default();
    SearchResult {
        id: plugin_result_id(&command.plugin_id, &command.command_id),
        title: command.title,
        subtitle: command.subtitle.or(Some(command.plugin_name)),
        icon_uri: Some(plugin_icon_uri(&command.plugin_id, &command.plugin_icon)),
        provider: PLUGIN_PROVIDER_ID.to_string(),
        score: command_match.score,
        matches: SearchResultMatches {
            title: command_match.title_ranges,
            subtitle: command_match.subtitle_ranges,
        },
        actions: vec![SearchResultAction {
            id: OPEN_PLUGIN_ACTION_ID.to_string(),
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
    if query.trim().is_empty() || plugin_command_search_match(&command, query).is_some() {
        return Some(search_result_for_plugin_command_record(command, query));
    }
    None
}

fn command_mode(command: &PluginCommandRecord) -> Option<PluginCommandMode> {
    plugin_command_mode_from_str(&command.mode)
}

#[derive(Default)]
struct PluginCommandSearchMatch {
    score: f32,
    title_ranges: Vec<MatchRange>,
    subtitle_ranges: Vec<MatchRange>,
}

fn plugin_command_search_match(
    command: &PluginCommandRecord,
    query: &str,
) -> Option<PluginCommandSearchMatch> {
    if query.trim().is_empty() {
        return Some(PluginCommandSearchMatch {
            score: 95.0,
            ..PluginCommandSearchMatch::default()
        });
    }

    let mut best = None;
    consider_plugin_match(
        &mut best,
        match_text(&command.title, query),
        0.0,
        VisiblePluginCommandField::Title,
    );
    if let Some(subtitle) = &command.subtitle {
        consider_plugin_match(
            &mut best,
            match_text(subtitle, query),
            -8.0,
            VisiblePluginCommandField::Subtitle,
        );
    }
    consider_plugin_match(
        &mut best,
        match_text(&command.plugin_name, query),
        -12.0,
        VisiblePluginCommandField::Subtitle,
    );
    for keyword in &command.keywords {
        consider_plugin_match(
            &mut best,
            match_text(keyword, query),
            -16.0,
            VisiblePluginCommandField::Hidden,
        );
    }
    consider_plugin_match(
        &mut best,
        match_text(&command.plugin_id, query),
        -20.0,
        VisiblePluginCommandField::Hidden,
    );
    consider_plugin_match(
        &mut best,
        match_text(&command.command_id, query),
        -20.0,
        VisiblePluginCommandField::Hidden,
    );

    best
}

#[derive(Clone, Copy)]
enum VisiblePluginCommandField {
    Hidden,
    Subtitle,
    Title,
}

fn consider_plugin_match(
    best: &mut Option<PluginCommandSearchMatch>,
    text_match: Option<TextMatch>,
    adjustment: f32,
    visible_field: VisiblePluginCommandField,
) {
    let Some(text_match) = text_match else {
        return;
    };
    let score = plugin_match_score(&text_match, adjustment);
    if best.as_ref().is_some_and(|current| current.score >= score) {
        return;
    }

    *best = Some(PluginCommandSearchMatch {
        score,
        title_ranges: matches!(visible_field, VisiblePluginCommandField::Title)
            .then_some(text_match.ranges.clone())
            .unwrap_or_default(),
        subtitle_ranges: matches!(visible_field, VisiblePluginCommandField::Subtitle)
            .then_some(text_match.ranges)
            .unwrap_or_default(),
    });
}

fn plugin_match_score(text_match: &TextMatch, adjustment: f32) -> f32 {
    let base = match text_match.kind {
        MatchKind::Exact => 108.0,
        MatchKind::Prefix => 96.0,
        MatchKind::Contains => 70.0,
        MatchKind::Fuzzy => text_match.score.min(64.0),
    };

    (base + adjustment).max(1.0)
}


#[cfg(test)]
mod tests {
    // Plugin ID parsing tests live in litools-plugin::ids.
}
