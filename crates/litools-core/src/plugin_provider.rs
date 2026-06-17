use std::sync::{Arc, Mutex};

use litools_config::search::{ACTION_OPEN, PLUGIN_PROVIDER_ID};
use litools_index::repository::PluginCommandRecord;
use litools_plugin::{PluginCommandSearchItem, PluginManager, plugin_result_id};
use async_trait::async_trait;
use litools_search::{
    FieldMatcher, FieldWeights, SearchProvider, SearchRequest, SearchResult,
    SearchResultAction, SearchResultMatches, VisibleField, match_text,
};

/// 插件命令搜索提供器。
///
/// 优先从 [`PluginManager`] 内存直接读取（始终最新、无需缓存失效），
/// 回退到空结果以兼容未注入 PluginManager 的测试路径。
pub struct PluginCommandProvider {
    plugin_manager: Mutex<Option<Arc<PluginManager>>>,
}

impl PluginCommandProvider {
    pub fn new() -> Self {
        Self {
            plugin_manager: Mutex::new(None),
        }
    }

    /// 注入插件管理器，后续搜索直接从内存读取。
    pub fn set_plugin_manager(&self, manager: Arc<PluginManager>) {
        if let Ok(mut pm) = self.plugin_manager.lock() {
            *pm = Some(manager);
        }
    }

    /// 保持旧 API 兼容。PluginManager 模式下不需要缓存失效。
    pub fn invalidate_cache(&self) {}
}

#[async_trait]
impl SearchProvider for PluginCommandProvider {
    fn id(&self) -> &str {
        PLUGIN_PROVIDER_ID
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(20)
    }

    async fn search(&self, request: &SearchRequest) -> Vec<SearchResult> {
        let query = &request.query;
        if let Some(pm) = self.plugin_manager.lock().ok().and_then(|o| o.clone()) {
            return pm
                .enabled_view_commands()
                .into_iter()
                .filter_map(|item| search_result_for_item(item, &query.text))
                .collect();
        }
        Vec::new()
    }
}

// ── 从 PluginManager 直接生成搜索结果 ──

const PLUGIN_COMMAND_WEIGHTS: FieldWeights = FieldWeights {
    exact: 108.0,
    prefix: 96.0,
    contains: 70.0,
    fuzzy_cap: 64.0,
};

fn search_result_for_item(item: PluginCommandSearchItem, query: &str) -> Option<SearchResult> {
    if query.trim().is_empty() {
        return Some(build_result(item, FieldMatcher::with_score(95.0)));
    }
    let matcher = item_match(&item, query);
    matcher.has_match().then(|| build_result(item, matcher))
}

fn build_result(item: PluginCommandSearchItem, matcher: FieldMatcher) -> SearchResult {
    let (score, title_ranges, subtitle_ranges) = matcher.finish();
    SearchResult {
        id: plugin_result_id(&item.plugin_id, &item.command_id),
        title: item.title,
        subtitle: item.subtitle.or(Some(item.plugin_name)),
        icon_uri: Some(format!(
            "litools-plugin://{}/{}",
            item.plugin_id, item.plugin_icon
        )),
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

fn item_match(item: &PluginCommandSearchItem, query: &str) -> FieldMatcher {
    let mut matcher = FieldMatcher::new();
    matcher.consider(
        match_text(&item.title, query),
        0.0,
        VisibleField::Title,
        &PLUGIN_COMMAND_WEIGHTS,
    );
    if let Some(subtitle) = &item.subtitle {
        matcher.consider(
            match_text(subtitle, query),
            -8.0,
            VisibleField::Subtitle,
            &PLUGIN_COMMAND_WEIGHTS,
        );
    }
    matcher.consider(
        match_text(&item.plugin_name, query),
        -12.0,
        VisibleField::Subtitle,
        &PLUGIN_COMMAND_WEIGHTS,
    );
    for keyword in &item.keywords {
        matcher.consider(
            match_text(keyword, query),
            -16.0,
            VisibleField::Hidden,
            &PLUGIN_COMMAND_WEIGHTS,
        );
    }
    matcher.consider(
        match_text(&item.plugin_id, query),
        -20.0,
        VisibleField::Hidden,
        &PLUGIN_COMMAND_WEIGHTS,
    );
    matcher.consider(
        match_text(&item.command_id, query),
        -20.0,
        VisibleField::Hidden,
        &PLUGIN_COMMAND_WEIGHTS,
    );
    matcher
}

// ── 从 DB 记录生成搜索结果（launcher 中 pinned/recent 使用） ──

pub fn search_result_for_plugin_command_record(
    command: PluginCommandRecord,
    query: &str,
) -> SearchResult {
    let (score, title_ranges, subtitle_ranges) = record_match(&command, query).finish();
    SearchResult {
        id: plugin_result_id(&command.plugin_id, &command.command_id),
        title: command.title,
        subtitle: command.subtitle.or(Some(command.plugin_name)),
        icon_uri: Some(format!(
            "litools-plugin://{}/{}",
            command.plugin_id, command.plugin_icon
        )),
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

fn record_match(command: &PluginCommandRecord, query: &str) -> FieldMatcher {
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
