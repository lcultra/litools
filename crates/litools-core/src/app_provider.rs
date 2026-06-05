use litools_index::{IndexDatabase, repository::AppRepository};
use litools_search::{SearchProvider, SearchQuery, SearchResult, SearchResultAction};

pub const APP_PROVIDER_ID: &str = "apps";
pub const APP_RESULT_PREFIX: &str = "app:";
pub const OPEN_APP_ACTION_ID: &str = "open";

pub struct AppSearchProvider {
    database: IndexDatabase,
}

impl AppSearchProvider {
    pub fn new(database: IndexDatabase) -> Self {
        Self { database }
    }
}

impl SearchProvider for AppSearchProvider {
    fn id(&self) -> &'static str {
        APP_PROVIDER_ID
    }

    fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let connection = self.database.connection();
        let repository = AppRepository::new(&connection);

        repository
            .search_apps(&query.text, query.limit)
            .unwrap_or_default()
            .into_iter()
            .map(|app| {
                let score =
                    score_app_result(&app.name, &app.id, &app.path, &query.text, app.launch_count);

                SearchResult {
                    id: format!("{APP_RESULT_PREFIX}{}", app.id),
                    title: app.name,
                    subtitle: Some(app.path),
                    provider: APP_PROVIDER_ID.to_string(),
                    score,
                    actions: vec![SearchResultAction {
                        id: OPEN_APP_ACTION_ID.to_string(),
                        label: "打开".to_string(),
                    }],
                }
            })
            .collect()
    }
}

fn score_app_result(name: &str, id: &str, path: &str, query: &str, launch_count: i64) -> f32 {
    let query = query.trim().to_lowercase();
    let launch_bonus = (launch_count.min(20) as f32) * 0.5;

    if query.is_empty() {
        return 80.0 + launch_bonus;
    }

    let name = name.to_lowercase();
    let id = id.to_lowercase();
    let path = path.to_lowercase();

    if name == query {
        return 120.0 + launch_bonus;
    }

    if name.starts_with(&query) {
        return 105.0 + launch_bonus;
    }

    if id.starts_with(&query) {
        return 90.0 + launch_bonus;
    }

    if name.contains(&query) {
        return 75.0 + launch_bonus;
    }

    if id.contains(&query) || path.contains(&query) {
        return 55.0 + launch_bonus;
    }

    0.0
}

pub fn app_id_from_result_id(result_id: &str) -> Option<&str> {
    result_id.strip_prefix(APP_RESULT_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_result_id_strips_prefix() {
        assert_eq!(
            app_id_from_result_id("app:com.apple.Safari"),
            Some("com.apple.Safari")
        );
        assert_eq!(app_id_from_result_id("open-settings"), None);
    }

    #[test]
    fn exact_app_match_scores_above_prefix_match() {
        assert!(
            score_app_result(
                "Safari",
                "com.apple.Safari",
                "/Applications/Safari.app",
                "safari",
                0
            ) > score_app_result(
                "Safari Technology Preview",
                "com.apple.SafariTechnologyPreview",
                "/Applications/Safari Technology Preview.app",
                "safari",
                0
            )
        );
    }
}
