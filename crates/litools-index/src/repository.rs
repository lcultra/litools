use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexMetadataRecord {
    pub key: String,
    pub value_json: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub localized_names: Vec<String>,
    pub aliases: Vec<String>,
    pub search_text: String,
    pub platform: String,
    pub last_seen_at: String,
    pub launch_count: i64,
}

#[derive(Clone, Debug)]
pub struct AppUpsert<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub path: &'a str,
    pub icon_path: Option<&'a str>,
    pub localized_names: &'a [String],
    pub aliases: &'a [String],
    pub search_text: &'a str,
    pub platform: &'a str,
    pub last_seen_at: &'a str,
}

pub struct AppRepository<'a> {
    connection: &'a Connection,
}

impl<'a> AppRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn count_apps(&self) -> rusqlite::Result<usize> {
        self.connection
            .query_row("SELECT COUNT(*) FROM apps", [], |row| row.get(0))
    }

    pub fn count_apps_by_platform(&self, platform: &str) -> rusqlite::Result<usize> {
        self.connection.query_row(
            "SELECT COUNT(*) FROM apps WHERE platform = ?1",
            params![platform],
            |row| row.get(0),
        )
    }

    pub fn delete_apps_not_seen_at(
        &self,
        platform: &str,
        last_seen_at: &str,
    ) -> rusqlite::Result<usize> {
        self.connection.execute(
            "DELETE FROM apps WHERE platform = ?1 AND last_seen_at <> ?2",
            params![platform, last_seen_at],
        )
    }

    pub fn upsert_app(&self, app: AppUpsert<'_>) -> rusqlite::Result<()> {
        let localized_names_json = serde_json::to_string(app.localized_names)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?;
        let aliases_json = serde_json::to_string(app.aliases)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?;

        self.connection.execute(
            "INSERT INTO apps (
                id,
                name,
                path,
                icon_path,
                localized_names_json,
                aliases_json,
                search_text,
                platform,
                last_seen_at,
                launch_count
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                path = excluded.path,
                icon_path = excluded.icon_path,
                localized_names_json = excluded.localized_names_json,
                aliases_json = excluded.aliases_json,
                search_text = excluded.search_text,
                platform = excluded.platform,
                last_seen_at = excluded.last_seen_at",
            params![
                app.id,
                app.name,
                app.path,
                app.icon_path,
                localized_names_json,
                aliases_json,
                app.search_text,
                app.platform,
                app.last_seen_at
            ],
        )?;
        Ok(())
    }

    pub fn find_app(&self, id: &str) -> rusqlite::Result<Option<AppRecord>> {
        self.connection
            .query_row(
                "SELECT id,
                        name,
                        path,
                        icon_path,
                        localized_names_json,
                        aliases_json,
                        search_text,
                        platform,
                        last_seen_at,
                        launch_count
                 FROM apps
                 WHERE id = ?1",
                params![id],
                app_record_from_row,
            )
            .optional()
    }

    pub fn search_apps(&self, query: &str, limit: Option<usize>) -> rusqlite::Result<Vec<AppRecord>> {
        let query = query.trim().to_lowercase();

        if query.is_empty() {
            let mut statement = self.connection.prepare(&format!(
                "SELECT id,
                        name,
                        path,
                        icon_path,
                        localized_names_json,
                        aliases_json,
                        search_text,
                        platform,
                        last_seen_at,
                        launch_count
                 FROM apps
                 ORDER BY launch_count DESC, name ASC{}",
                limit_clause(limit)
            ))?;
            let rows = statement.query_map([], app_record_from_row)?;
            return rows.collect();
        }

        let like_query = format!("%{query}%");
        let prefix_query = format!("{query}%");
        let mut statement = self.connection.prepare(&format!(
            "SELECT id,
                    name,
                    path,
                    icon_path,
                    localized_names_json,
                    aliases_json,
                    search_text,
                    platform,
                    last_seen_at,
                    launch_count
             FROM apps
             WHERE lower(name) LIKE ?1
                OR lower(localized_names_json) LIKE ?1
                OR lower(aliases_json) LIKE ?1
                OR lower(search_text) LIKE ?1
                OR lower(id) LIKE ?1
                OR lower(path) LIKE ?1
             ORDER BY
                CASE
                    WHEN lower(name) = ?2 THEN 0
                    WHEN lower(name) LIKE ?3 THEN 1
                    WHEN lower(localized_names_json) LIKE ?1 OR lower(aliases_json) LIKE ?1 THEN 2
                    ELSE 3
                END,
                launch_count DESC,
                name ASC{}",
            limit_clause(limit)
        ))?;
        let rows = statement.query_map(
            params![like_query, query, prefix_query],
            app_record_from_row,
        )?;
        rows.collect()
    }

    pub fn increment_launch_count(&self, id: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE apps SET launch_count = launch_count + 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
}

fn limit_clause(limit: Option<usize>) -> String {
    limit
        .map(|limit| format!(" LIMIT {limit}"))
        .unwrap_or_default()
}

fn app_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AppRecord> {
    Ok(AppRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        icon_path: row.get(3)?,
        localized_names: json_string_array(row.get::<_, String>(4)?)?,
        aliases: json_string_array(row.get::<_, String>(5)?)?,
        search_text: row.get(6)?,
        platform: row.get(7)?,
        last_seen_at: row.get(8)?,
        launch_count: row.get(9)?,
    })
}

fn json_string_array(value: String) -> rusqlite::Result<Vec<String>> {
    serde_json::from_str(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    })
}

pub struct CommandRepository<'a> {
    connection: &'a Connection,
}

impl<'a> CommandRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn count_commands(&self) -> rusqlite::Result<usize> {
        self.connection
            .query_row("SELECT COUNT(*) FROM commands", [], |row| row.get(0))
    }

    pub fn upsert_command(
        &self,
        id: &str,
        namespace: &str,
        title: &str,
        subtitle: Option<&str>,
        action: &str,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO commands (id, namespace, title, subtitle, action, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)
             ON CONFLICT(id) DO UPDATE SET
                namespace = excluded.namespace,
                title = excluded.title,
                subtitle = excluded.subtitle,
                action = excluded.action,
                enabled = excluded.enabled",
            params![id, namespace, title, subtitle, action],
        )?;
        Ok(())
    }
}

pub struct IndexMetadataRepository<'a> {
    connection: &'a Connection,
}

impl<'a> IndexMetadataRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn get(&self, key: &str) -> rusqlite::Result<Option<IndexMetadataRecord>> {
        self.connection
            .query_row(
                "SELECT key, value_json, updated_at FROM index_metadata WHERE key = ?1",
                params![key],
                |row| {
                    Ok(IndexMetadataRecord {
                        key: row.get(0)?,
                        value_json: row.get(1)?,
                        updated_at: row.get(2)?,
                    })
                },
            )
            .optional()
    }

    pub fn set_json(&self, key: &str, value_json: &str, updated_at: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO index_metadata (key, value_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET
                value_json = excluded.value_json,
                updated_at = excluded.updated_at",
            params![key, value_json, updated_at],
        )?;
        Ok(())
    }
}

pub struct SettingsRepository<'a> {
    connection: &'a Connection,
}

impl<'a> SettingsRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn get_json(&self, key: &str) -> rusqlite::Result<Option<String>> {
        let mut statement = self
            .connection
            .prepare("SELECT value_json FROM settings WHERE key = ?1")?;
        let mut rows = statement.query(params![key])?;

        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn set_json(&self, key: &str, value_json: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO settings (key, value_json) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
            params![key, value_json],
        )?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinnedItemRecord {
    pub target_type: String,
    pub target_id: String,
    pub pinned_at: String,
    pub sort_order: i64,
}

pub struct PinnedRepository<'a> {
    connection: &'a Connection,
}

impl<'a> PinnedRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn list_pinned(&self, limit: usize) -> rusqlite::Result<Vec<PinnedItemRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT target_type, target_id, pinned_at, sort_order
             FROM pinned_items
             ORDER BY sort_order ASC, pinned_at DESC
             LIMIT ?1",
        )?;
        let rows = statement.query_map(params![limit as i64], |row| {
            Ok(PinnedItemRecord {
                target_type: row.get(0)?,
                target_id: row.get(1)?,
                pinned_at: row.get(2)?,
                sort_order: row.get(3)?,
            })
        })?;

        rows.collect()
    }

    pub fn pin(
        &self,
        target_type: &str,
        target_id: &str,
        pinned_at: &str,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO pinned_items (target_type, target_id, pinned_at, sort_order)
             VALUES (?1, ?2, ?3, COALESCE((SELECT MAX(sort_order) + 1 FROM pinned_items), 0))
             ON CONFLICT(target_type, target_id) DO UPDATE SET
                pinned_at = excluded.pinned_at",
            params![target_type, target_id, pinned_at],
        )?;
        Ok(())
    }

    pub fn unpin(&self, target_type: &str, target_id: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "DELETE FROM pinned_items WHERE target_type = ?1 AND target_id = ?2",
            params![target_type, target_id],
        )?;
        Ok(())
    }

    pub fn is_pinned(&self, target_type: &str, target_id: &str) -> rusqlite::Result<bool> {
        self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM pinned_items WHERE target_type = ?1 AND target_id = ?2)",
            params![target_type, target_id],
            |row| row.get(0),
        )
    }

    pub fn reorder(&self, ordered_targets: &[(String, String)]) -> rusqlite::Result<()> {
        let transaction = self.connection.unchecked_transaction()?;

        for (sort_order, (target_type, target_id)) in ordered_targets.iter().enumerate() {
            transaction.execute(
                "UPDATE pinned_items
                 SET sort_order = ?1
                 WHERE target_type = ?2 AND target_id = ?3",
                params![sort_order as i64, target_type, target_id],
            )?;
        }

        transaction.commit()
    }
}

pub struct UsageRepository<'a> {
    connection: &'a Connection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageEventRecord {
    pub target_type: String,
    pub target_id: String,
    pub query: Option<String>,
    pub selected_at: String,
}

fn usage_event_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<UsageEventRecord> {
    Ok(UsageEventRecord {
        target_type: row.get(0)?,
        target_id: row.get(1)?,
        query: row.get(2)?,
        selected_at: row.get(3)?,
    })
}

impl<'a> UsageRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn count_events(&self) -> rusqlite::Result<usize> {
        self.connection
            .query_row("SELECT COUNT(*) FROM usage_events", [], |row| row.get(0))
    }

    pub fn record_selection(
        &self,
        id: &str,
        target_type: &str,
        target_id: &str,
        query: Option<&str>,
        selected_at: &str,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO usage_events (id, target_type, target_id, query, selected_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, target_type, target_id, query, selected_at],
        )?;
        Ok(())
    }

    pub fn recent_events(&self, limit: usize) -> rusqlite::Result<Vec<UsageEventRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT target_type, target_id, query, selected_at
             FROM usage_events
             ORDER BY selected_at DESC
             LIMIT ?1",
        )?;

        let rows = statement.query_map(params![limit], usage_event_from_row)?;

        rows.collect()
    }

    pub fn recent_unique_targets(&self, limit: usize) -> rusqlite::Result<Vec<UsageEventRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT latest.target_type,
                    latest.target_id,
                    latest.query,
                    latest.selected_at
             FROM usage_events latest
             JOIN (
                SELECT target_type, target_id, MAX(selected_at) AS selected_at
                FROM usage_events
                GROUP BY target_type, target_id
             ) grouped
                ON latest.target_type = grouped.target_type
               AND latest.target_id = grouped.target_id
               AND latest.selected_at = grouped.selected_at
             GROUP BY latest.target_type, latest.target_id
             ORDER BY latest.selected_at DESC
             LIMIT ?1",
        )?;

        let rows = statement.query_map(params![limit], usage_event_from_row)?;

        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::IndexDatabase;

    use super::*;

    #[test]
    fn settings_repository_round_trips_json() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = SettingsRepository::new(&connection);

        repository
            .set_json("app_settings", r#"{"theme":"dark"}"#)
            .expect("write settings");

        assert_eq!(
            repository.get_json("app_settings").expect("read settings"),
            Some(r#"{"theme":"dark"}"#.to_string())
        );
    }

    #[test]
    fn command_repository_counts_commands() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = CommandRepository::new(&connection);

        repository
            .upsert_command("open-settings", "builtin", "Open Settings", None, "execute")
            .expect("write command");

        assert_eq!(repository.count_commands().expect("count commands"), 1);
    }

    #[test]
    fn app_repository_upserts_and_finds_app() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = AppRepository::new(&connection);

        repository
            .upsert_app(AppUpsert {
                id: "com.example.App",
                name: "Example",
                path: "/Applications/Example.app",
                icon_path: Some("/Applications/Example.app/Contents/Resources/App.icns"),
                localized_names: &[],
                aliases: &[],
                search_text: "Example com.example.App /Applications/Example.app",
                platform: "macos",
                last_seen_at: "2026-06-04T00:00:00Z",
            })
            .expect("write app");

        let app = repository
            .find_app("com.example.App")
            .expect("find app")
            .expect("app exists");

        assert_eq!(app.name, "Example");
        assert_eq!(
            app.search_text,
            "Example com.example.App /Applications/Example.app"
        );
        assert_eq!(repository.count_apps().expect("count apps"), 1);
    }

    #[test]
    fn app_repository_searches_apps() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = AppRepository::new(&connection);

        repository
            .upsert_app(AppUpsert {
                id: "com.apple.Terminal",
                name: "Terminal",
                path: "/System/Applications/Utilities/Terminal.app",
                icon_path: None,
                localized_names: &[],
                aliases: &[],
                search_text: "Terminal com.apple.Terminal /System/Applications/Utilities/Terminal.app",
                platform: "macos",
                last_seen_at: "2026-06-04T00:00:00Z",
            })
            .expect("write terminal");
        repository
            .upsert_app(AppUpsert {
                id: "com.apple.Safari",
                name: "Safari",
                path: "/Applications/Safari.app",
                icon_path: None,
                localized_names: &[],
                aliases: &[],
                search_text: "Safari com.apple.Safari /Applications/Safari.app",
                platform: "macos",
                last_seen_at: "2026-06-04T00:00:00Z",
            })
            .expect("write safari");

        let results = repository.search_apps("term", Some(10)).expect("search apps");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "com.apple.Terminal");
    }

    #[test]
    fn app_repository_searches_localized_aliases_and_search_text() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = AppRepository::new(&connection);
        let localized_names = vec!["微信".to_string(), "WeChat".to_string()];
        let aliases = vec!["wx".to_string(), "weixin".to_string()];

        repository
            .upsert_app(AppUpsert {
                id: "com.tencent.xin",
                name: "微信",
                path: "/Applications/WeChat.app",
                icon_path: None,
                localized_names: &localized_names,
                aliases: &aliases,
                search_text: "微信 WeChat wx weixin com.tencent.xin /Applications/WeChat.app",
                platform: "macos",
                last_seen_at: "2026-06-04T00:00:00Z",
            })
            .expect("write app");

        let by_alias = repository.search_apps("weixin", Some(10)).expect("search alias");
        let by_initials = repository.search_apps("wx", Some(10)).expect("search initials");

        assert_eq!(by_alias[0].id, "com.tencent.xin");
        assert_eq!(by_initials[0].id, "com.tencent.xin");
        assert_eq!(by_alias[0].localized_names, localized_names);
        assert_eq!(by_alias[0].aliases, aliases);
    }

    #[test]
    fn app_repository_increments_launch_count() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = AppRepository::new(&connection);

        repository
            .upsert_app(AppUpsert {
                id: "com.example.App",
                name: "Example",
                path: "/Applications/Example.app",
                icon_path: None,
                localized_names: &[],
                aliases: &[],
                search_text: "Example com.example.App /Applications/Example.app",
                platform: "macos",
                last_seen_at: "2026-06-04T00:00:00Z",
            })
            .expect("write app");
        repository
            .increment_launch_count("com.example.App")
            .expect("increment launch count");

        let app = repository
            .find_app("com.example.App")
            .expect("find app")
            .expect("app exists");

        assert_eq!(app.launch_count, 1);
    }

    #[test]
    fn app_repository_deletes_apps_not_seen_on_platform() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = AppRepository::new(&connection);

        for (id, platform, last_seen_at) in [
            ("com.example.Current", "macos", "2026-06-05T00:00:00Z"),
            ("com.example.Stale", "macos", "2026-06-04T00:00:00Z"),
            (
                "com.example.OtherPlatform",
                "windows",
                "2026-06-04T00:00:00Z",
            ),
        ] {
            repository
                .upsert_app(AppUpsert {
                    id,
                    name: id,
                    path: "/Applications/Example.app",
                    icon_path: None,
                    localized_names: &[],
                    aliases: &[],
                    search_text: id,
                    platform,
                    last_seen_at,
                })
                .expect("write app");
        }

        let removed = repository
            .delete_apps_not_seen_at("macos", "2026-06-05T00:00:00Z")
            .expect("delete stale apps");

        assert_eq!(removed, 1);
        assert!(
            repository
                .find_app("com.example.Current")
                .expect("find current")
                .is_some()
        );
        assert!(
            repository
                .find_app("com.example.Stale")
                .expect("find stale")
                .is_none()
        );
        assert!(
            repository
                .find_app("com.example.OtherPlatform")
                .expect("find other platform")
                .is_some()
        );
        assert_eq!(
            repository
                .count_apps_by_platform("macos")
                .expect("count macos apps"),
            1
        );
    }

    #[test]
    fn pinned_repository_pins_lists_and_unpins_items() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = PinnedRepository::new(&connection);

        repository
            .pin("app", "com.example.App", "2026-06-05T00:00:00Z")
            .expect("pin app");
        repository
            .pin("command", "open-settings", "2026-06-06T00:00:00Z")
            .expect("pin command");

        assert!(
            repository
                .is_pinned("app", "com.example.App")
                .expect("check pinned")
        );

        let pinned = repository.list_pinned(10).expect("list pinned");
        assert_eq!(pinned.len(), 2);
        assert_eq!(pinned[0].target_id, "com.example.App");
        assert_eq!(pinned[0].sort_order, 0);
        assert_eq!(pinned[1].target_id, "open-settings");
        assert_eq!(pinned[1].sort_order, 1);

        repository
            .pin("app", "com.example.App", "2026-06-07T00:00:00Z")
            .expect("update pinned app");
        let pinned = repository.list_pinned(10).expect("list updated pinned");
        assert_eq!(pinned[0].target_id, "com.example.App");
        assert_eq!(pinned[0].pinned_at, "2026-06-07T00:00:00Z");
        assert_eq!(pinned[0].sort_order, 0);

        repository
            .reorder(&[
                ("command".to_string(), "open-settings".to_string()),
                ("app".to_string(), "com.example.App".to_string()),
            ])
            .expect("reorder pinned items");
        let pinned = repository.list_pinned(10).expect("list reordered pinned");
        assert_eq!(pinned[0].target_id, "open-settings");
        assert_eq!(pinned[0].sort_order, 0);
        assert_eq!(pinned[1].target_id, "com.example.App");
        assert_eq!(pinned[1].sort_order, 1);

        repository
            .unpin("app", "com.example.App")
            .expect("unpin app");
        assert!(
            !repository
                .is_pinned("app", "com.example.App")
                .expect("check unpinned")
        );
    }

    #[test]
    fn usage_repository_lists_recent_unique_targets() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = UsageRepository::new(&connection);

        for (id, target_type, target_id, selected_at) in [
            ("1", "app", "com.example.App", "2026-06-05T00:00:00Z"),
            ("2", "command", "open-settings", "2026-06-06T00:00:00Z"),
            ("3", "app", "com.example.App", "2026-06-07T00:00:00Z"),
        ] {
            repository
                .record_selection(id, target_type, target_id, None, selected_at)
                .expect("record usage");
        }

        let recent = repository
            .recent_unique_targets(10)
            .expect("list recent unique targets");

        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].target_type, "app");
        assert_eq!(recent[0].target_id, "com.example.App");
        assert_eq!(recent[0].selected_at, "2026-06-07T00:00:00Z");
        assert_eq!(recent[1].target_type, "command");
        assert_eq!(recent[1].target_id, "open-settings");
    }

    #[test]
    fn index_metadata_repository_round_trips_json() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = IndexMetadataRepository::new(&connection);

        repository
            .set_json(
                "apps_last_refresh_status",
                r#"{"success":true}"#,
                "2026-06-05T00:00:00Z",
            )
            .expect("write metadata");

        let metadata = repository
            .get("apps_last_refresh_status")
            .expect("read metadata")
            .expect("metadata exists");

        assert_eq!(metadata.key, "apps_last_refresh_status");
        assert_eq!(metadata.value_json, r#"{"success":true}"#);
        assert_eq!(metadata.updated_at, "2026-06-05T00:00:00Z");
    }
}
