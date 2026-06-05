use litools_index::{
    IndexDatabase,
    repository::{AppRecord, AppRepository},
};
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
            .map(|app| search_result_for_app(app, &query.text))
            .collect()
    }
}

pub fn search_result_for_app(app: AppRecord, query: &str) -> SearchResult {
    let score = score_app_result(&app, query);

    SearchResult {
        id: format!("{APP_RESULT_PREFIX}{}", app.id),
        title: app.name,
        subtitle: Some(app.path),
        icon_uri: Some(app_icon_uri(&app.id)),
        provider: APP_PROVIDER_ID.to_string(),
        score,
        actions: vec![SearchResultAction {
            id: OPEN_APP_ACTION_ID.to_string(),
            label: "打开".to_string(),
        }],
    }
}

fn app_icon_uri(app_id: &str) -> String {
    format!(
        "litools-icon://app/{}",
        percent_encode_uri_path_segment(app_id)
    )
}

fn percent_encode_uri_path_segment(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn score_app_result(app: &AppRecord, query: &str) -> f32 {
    let query = query.trim().to_lowercase();
    let launch_bonus = (app.launch_count.min(20) as f32) * 0.5;

    if query.is_empty() {
        return 80.0 + launch_bonus;
    }

    let name = app.name.to_lowercase();
    let localized_names = normalized_terms(&app.localized_names);
    let aliases = normalized_terms(&app.aliases);
    let id = app.id.to_lowercase();
    let path = app.path.to_lowercase();
    let search_text = app.search_text.to_lowercase();

    if name == query {
        return 130.0 + launch_bonus;
    }

    if localized_names.iter().any(|term| term == &query) {
        return 122.0 + launch_bonus;
    }

    if aliases.iter().any(|term| term == &query) {
        return 116.0 + launch_bonus;
    }

    if name.starts_with(&query) {
        return 108.0 + launch_bonus;
    }

    if localized_names.iter().any(|term| term.starts_with(&query)) {
        return 102.0 + launch_bonus;
    }

    if aliases.iter().any(|term| term.starts_with(&query)) {
        return 96.0 + launch_bonus;
    }

    if name.contains(&query) || localized_names.iter().any(|term| term.contains(&query)) {
        return 82.0 + launch_bonus;
    }

    if aliases.iter().any(|term| term.contains(&query)) {
        return 76.0 + launch_bonus;
    }

    if id.starts_with(&query) {
        return 68.0 + launch_bonus;
    }

    if id.contains(&query) || path.contains(&query) || search_text.contains(&query) {
        return 55.0 + launch_bonus;
    }

    0.0
}

fn normalized_terms(terms: &[String]) -> Vec<String> {
    terms
        .iter()
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect()
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
    fn app_icon_uri_encodes_app_id() {
        assert_eq!(
            app_icon_uri("path:/Applications/My App.app"),
            "litools-icon://app/path%3A%2FApplications%2FMy%20App.app"
        );
    }

    #[test]
    fn exact_app_match_scores_above_prefix_match() {
        assert!(
            score_app_result(
                &app_record("com.apple.Safari", "Safari", &[], &[], 0),
                "safari"
            ) > score_app_result(
                &app_record(
                    "com.apple.SafariTechnologyPreview",
                    "Safari Technology Preview",
                    &[],
                    &[],
                    0,
                ),
                "safari",
            )
        );
    }

    #[test]
    fn alias_match_scores_above_path_fallback() {
        assert!(
            score_app_result(
                &app_record("com.tencent.xin", "微信", &["WeChat"], &["wx", "weixin"], 0),
                "wx",
            ) > score_app_result(
                &app_record("com.example.wxhelper", "Helper", &[], &[], 0),
                "wx",
            )
        );
    }

    fn app_record(
        id: &str,
        name: &str,
        localized_names: &[&str],
        aliases: &[&str],
        launch_count: i64,
    ) -> AppRecord {
        AppRecord {
            id: id.to_string(),
            name: name.to_string(),
            path: format!("/Applications/{name}.app"),
            icon_path: None,
            localized_names: localized_names
                .iter()
                .map(|value| value.to_string())
                .collect(),
            aliases: aliases.iter().map(|value| value.to_string()).collect(),
            search_text: [name]
                .into_iter()
                .chain(localized_names.iter().copied())
                .chain(aliases.iter().copied())
                .collect::<Vec<_>>()
                .join(" "),
            platform: "macos".to_string(),
            last_seen_at: "2026-06-05T00:00:00Z".to_string(),
            launch_count,
        }
    }
}
