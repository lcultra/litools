use std::collections::HashMap;

use litools_core::{CommandExecution, LauncherPanelResponse};
use litools_search::{SearchQuery, SearchRequest, SearchResult};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LAUNCHER_RESULT_LIMIT: usize = 20;

#[tauri::command]
pub async fn search(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let (search, enabled_providers) = {
        let app = state.app().read().unwrap();
        let s = app.context().search.clone();
        let p = app
            .context()
            .settings
            .get()
            .search
            .enabled_providers
            .clone();
        (s, p)
    };
    let results = search
        .search_with_providers(
            SearchQuery::with_limit(query, DEFAULT_LAUNCHER_RESULT_LIMIT),
            enabled_providers.iter().map(String::as_str),
        )
        .await;
    Ok(results)
}

#[tauri::command]
pub async fn launcher_panel(
    query: String,
    state: State<'_, AppState>,
) -> Result<LauncherPanelResponse, String> {
    let (search, enabled_providers, context_analyzer) = {
        let app = state.app().read().unwrap();
        (
            app.context().search.clone(),
            app.context()
                .settings
                .get()
                .search
                .enabled_providers
                .clone(),
            state.context_analyzer.clone(),
        )
    };

    let trimmed = query.trim().to_string();

    // Phase 4A: 分析输入，构建 InputContext（Arc clone 后锁外 await）
    let context = context_analyzer.analyze(&trimmed, None).await;

    let request = SearchRequest {
        query: SearchQuery::without_limit(&trimmed),
        context,
        metadata: HashMap::new(),
    };

    let search_results = search
        .search_with_request(&request, enabled_providers.iter().map(String::as_str))
        .await;

    let app = state.app().read().unwrap();
    app.launcher_panel_search_results(&trimmed, search_results)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pin_result(result_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let app = state.app().read().unwrap();
    app.pin_result(result_id)
        .map_err(|error| error.to_error_string())
}

#[tauri::command]
pub fn unpin_result(result_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let app = state.app().read().unwrap();
    app.unpin_result(result_id)
        .map_err(|error| error.to_error_string())
}

#[tauri::command]
pub fn reorder_pinned_results(
    result_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app = state.app().read().unwrap();
    app.reorder_pinned_results(result_ids)
        .map_err(|error| error.to_error_string())
}

#[tauri::command]
pub fn execute_result(
    result_id: String,
    action_id: String,
    provider: String,
    state: State<'_, AppState>,
) -> Result<CommandExecution, String> {
    let mut app = state.app().write().unwrap();
    app.execute_result(&result_id, &action_id, &provider)
        .map_err(|error| error.to_error_string())
}
