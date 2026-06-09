use litools_index::{
    IndexDatabase,
    repository::{AppRecord, AppRepository},
};
use litools_search::{
    MatchKind, MatchRange, SearchProvider, SearchQuery, SearchResult, SearchResultAction,
    SearchResultMatches, TextMatch, match_text,
};
use litools_config::search::{ACTION_OPEN, APP_PROVIDER_ID, APP_RESULT_PREFIX};


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

        let apps = if query.text.trim().is_empty() {
            repository.search_apps(&query.text, query.limit)
        } else {
            repository.list_apps_for_search()
        };

        apps.unwrap_or_default()
            .into_iter()
            .map(|app| search_result_for_app(app, &query.text))
            .filter(|result| query.text.trim().is_empty() || result.score > 0.0)
            .collect()
    }
}

pub fn search_result_for_app(app: AppRecord, query: &str) -> SearchResult {
    let app_match = match_app_result(&app, query);

    SearchResult {
        id: format!("{APP_RESULT_PREFIX}{}", app.id),
        title: app.name,
        subtitle: Some(app.path),
        icon_uri: Some(app_icon_uri(&app.id)),
        provider: APP_PROVIDER_ID.to_string(),
        score: app_match.score,
        matches: SearchResultMatches {
            title: app_match.title_ranges,
            subtitle: app_match.subtitle_ranges,
        },
        actions: vec![SearchResultAction {
            id: ACTION_OPEN.to_string(),
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

#[derive(Default)]
struct AppSearchMatch {
    score: f32,
    title_ranges: Vec<MatchRange>,
    subtitle_ranges: Vec<MatchRange>,
}

fn match_app_result(app: &AppRecord, query: &str) -> AppSearchMatch {
    let launch_bonus = (app.launch_count.min(20) as f32) * 0.5;

    if query.trim().is_empty() {
        return AppSearchMatch {
            score: 80.0 + launch_bonus,
            ..AppSearchMatch::default()
        };
    }

    let mut best = AppSearchMatch::default();
    consider_app_match(
        &mut best,
        match_text(&app.name, query),
        30.0,
        VisibleAppField::Title,
        launch_bonus,
    );

    for localized_name in &app.localized_names {
        consider_app_match(
            &mut best,
            match_text(localized_name, query),
            22.0,
            if localized_name == &app.name {
                VisibleAppField::Title
            } else {
                VisibleAppField::Hidden
            },
            launch_bonus,
        );
    }

    for alias in &app.aliases {
        consider_app_match(
            &mut best,
            match_text(alias, query),
            16.0,
            VisibleAppField::Hidden,
            launch_bonus,
        );
    }

    consider_app_match(
        &mut best,
        match_text(&app.id, query),
        -5.0,
        VisibleAppField::Hidden,
        launch_bonus,
    );
    consider_app_match(
        &mut best,
        match_text(&app.path, query),
        -15.0,
        VisibleAppField::Subtitle,
        launch_bonus,
    );
    consider_app_match(
        &mut best,
        match_text(&app.search_text, query),
        -15.0,
        VisibleAppField::Hidden,
        launch_bonus,
    );

    best
}

#[derive(Clone, Copy)]
enum VisibleAppField {
    Hidden,
    Subtitle,
    Title,
}

fn consider_app_match(
    best: &mut AppSearchMatch,
    text_match: Option<TextMatch>,
    adjustment: f32,
    visible_field: VisibleAppField,
    launch_bonus: f32,
) {
    let Some(text_match) = text_match else {
        return;
    };
    let score = app_match_score(&text_match, adjustment) + launch_bonus;
    if best.score >= score {
        return;
    }

    *best = AppSearchMatch {
        score,
        title_ranges: matches!(visible_field, VisibleAppField::Title)
            .then_some(text_match.ranges.clone())
            .unwrap_or_default(),
        subtitle_ranges: matches!(visible_field, VisibleAppField::Subtitle)
            .then_some(text_match.ranges)
            .unwrap_or_default(),
    };
}

fn app_match_score(text_match: &TextMatch, adjustment: f32) -> f32 {
    let base = match text_match.kind {
        MatchKind::Exact => 100.0,
        MatchKind::Prefix => 78.0,
        MatchKind::Contains => 52.0,
        MatchKind::Fuzzy => text_match.score.min(44.0),
    };

    (base + adjustment).max(1.0)
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
            match_app_result(
                &app_record("com.apple.Safari", "Safari", &[], &[], 0),
                "safari"
            )
            .score
                > match_app_result(
                    &app_record(
                        "com.apple.SafariTechnologyPreview",
                        "Safari Technology Preview",
                        &[],
                        &[],
                        0,
                    ),
                    "safari",
                )
                .score
        );
    }

    #[test]
    fn alias_match_scores_above_path_fallback() {
        assert!(
            match_app_result(
                &app_record("com.tencent.xin", "微信", &["WeChat"], &["wx", "weixin"], 0),
                "wx",
            )
            .score
                > match_app_result(
                    &app_record("com.example.wxhelper", "Helper", &[], &[], 0),
                    "wx",
                )
                .score
        );
    }

    #[test]
    fn fuzzy_title_match_returns_ranges() {
        let result = match_app_result(
            &app_record("com.apple.Safari", "Safari", &[], &[], 0),
            "sfi",
        );

        assert!(result.score > 0.0);
        assert_eq!(
            result.title_ranges,
            [
                MatchRange { start: 0, end: 1 },
                MatchRange { start: 2, end: 3 },
                MatchRange { start: 5, end: 6 }
            ]
        );
    }

    #[test]
    fn hidden_alias_match_does_not_highlight_title() {
        let result = match_app_result(
            &app_record("com.tencent.xin", "微信", &["WeChat"], &["wx", "weixin"], 0),
            "weixin",
        );

        assert!(result.score > 0.0);
        assert!(result.title_ranges.is_empty());
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
