use rusqlite::{Connection, OptionalExtension, params};

use super::json_string_array;

fn limit_clause(limit: Option<usize>) -> String {
    limit
        .map(|limit| format!(" LIMIT {limit}"))
        .unwrap_or_default()
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
                id, name, path, icon_path, localized_names_json, aliases_json,
                search_text, platform, last_seen_at, launch_count
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
                "SELECT id, name, path, icon_path, localized_names_json,
                        aliases_json, search_text, platform, last_seen_at, launch_count
                 FROM apps WHERE id = ?1",
                params![id],
                app_record_from_row,
            )
            .optional()
    }

    pub fn search_apps(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> rusqlite::Result<Vec<AppRecord>> {
        let query = query.trim().to_lowercase();

        if query.is_empty() {
            let mut statement = self.connection.prepare(&format!(
                "SELECT id, name, path, icon_path, localized_names_json,
                        aliases_json, search_text, platform, last_seen_at, launch_count
                 FROM apps ORDER BY launch_count DESC, name ASC{}",
                limit_clause(limit)
            ))?;
            let rows = statement.query_map([], app_record_from_row)?;
            return rows.collect();
        }

        let like_query = format!("%{query}%");
        let prefix_query = format!("{query}%");
        let mut statement = self.connection.prepare(&format!(
            "SELECT id, name, path, icon_path, localized_names_json,
                    aliases_json, search_text, platform, last_seen_at, launch_count
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
                launch_count DESC, name ASC{}",
            limit_clause(limit)
        ))?;
        let rows = statement.query_map(
            params![like_query, query, prefix_query],
            app_record_from_row,
        )?;
        rows.collect()
    }

    pub fn list_apps_for_search(&self) -> rusqlite::Result<Vec<AppRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, path, icon_path, localized_names_json,
                    aliases_json, search_text, platform, last_seen_at, launch_count
             FROM apps ORDER BY launch_count DESC, name ASC",
        )?;
        let rows = statement.query_map([], app_record_from_row)?;
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

#[cfg(test)]
mod tests {
    use crate::IndexDatabase;

    use super::*;

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
                search_text: "Example",
                platform: "macos",
                last_seen_at: "2026-06-04T00:00:00Z",
            })
            .expect("write app");
        repository
            .increment_launch_count("com.example.App")
            .expect("increment");

        let app = repository
            .find_app("com.example.App")
            .expect("find app")
            .expect("app exists");
        assert_eq!(app.launch_count, 1);
    }

    #[test]
    fn app_repository_lists_apps_for_search_without_query_filter() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = AppRepository::new(&connection);

        repository
            .upsert_app(AppUpsert {
                id: "com.apple.Safari",
                name: "Safari",
                path: "/Applications/Safari.app",
                icon_path: None,
                localized_names: &[],
                aliases: &[],
                search_text: "Safari",
                platform: "macos",
                last_seen_at: "2026-06-04T00:00:00Z",
            })
            .expect("write safari");

        let results = repository.list_apps_for_search().expect("list apps");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "com.apple.Safari");
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
                    path: "/tmp",
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
            .expect("delete stale");
        assert_eq!(removed, 1);
    }
}
