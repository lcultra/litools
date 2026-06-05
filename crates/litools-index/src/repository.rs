use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq)]
pub struct AppRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub platform: String,
    pub last_seen_at: String,
    pub launch_count: i64,
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

    pub fn upsert_app(
        &self,
        id: &str,
        name: &str,
        path: &str,
        icon_path: Option<&str>,
        platform: &str,
        last_seen_at: &str,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO apps (id, name, path, icon_path, platform, last_seen_at, launch_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                path = excluded.path,
                icon_path = excluded.icon_path,
                platform = excluded.platform,
                last_seen_at = excluded.last_seen_at",
            params![id, name, path, icon_path, platform, last_seen_at],
        )?;
        Ok(())
    }

    pub fn find_app(&self, id: &str) -> rusqlite::Result<Option<AppRecord>> {
        self.connection
            .query_row(
                "SELECT id, name, path, icon_path, platform, last_seen_at, launch_count
                 FROM apps
                 WHERE id = ?1",
                params![id],
                app_record_from_row,
            )
            .optional()
    }

    pub fn search_apps(&self, query: &str, limit: usize) -> rusqlite::Result<Vec<AppRecord>> {
        let query = query.trim().to_lowercase();
        let limit = limit as i64;

        if query.is_empty() {
            let mut statement = self.connection.prepare(
                "SELECT id, name, path, icon_path, platform, last_seen_at, launch_count
                 FROM apps
                 ORDER BY launch_count DESC, name ASC
                 LIMIT ?1",
            )?;
            let rows = statement.query_map(params![limit], app_record_from_row)?;
            return rows.collect();
        }

        let like_query = format!("%{query}%");
        let mut statement = self.connection.prepare(
            "SELECT id, name, path, icon_path, platform, last_seen_at, launch_count
             FROM apps
             WHERE lower(name) LIKE ?1 OR lower(id) LIKE ?1 OR lower(path) LIKE ?1
             ORDER BY launch_count DESC, name ASC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![like_query, limit], app_record_from_row)?;
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
        platform: row.get(4)?,
        last_seen_at: row.get(5)?,
        launch_count: row.get(6)?,
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

pub struct UsageRepository<'a> {
    connection: &'a Connection,
}

#[derive(Clone, Debug)]
pub struct UsageEventRecord {
    pub target_type: String,
    pub target_id: String,
    pub query: Option<String>,
    pub selected_at: String,
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

        let rows = statement.query_map(params![limit], |row| {
            Ok(UsageEventRecord {
                target_type: row.get(0)?,
                target_id: row.get(1)?,
                query: row.get(2)?,
                selected_at: row.get(3)?,
            })
        })?;

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
            .upsert_app(
                "com.example.App",
                "Example",
                "/Applications/Example.app",
                Some("/Applications/Example.app/Contents/Resources/App.icns"),
                "macos",
                "2026-06-04T00:00:00Z",
            )
            .expect("write app");

        let app = repository
            .find_app("com.example.App")
            .expect("find app")
            .expect("app exists");

        assert_eq!(app.name, "Example");
        assert_eq!(repository.count_apps().expect("count apps"), 1);
    }

    #[test]
    fn app_repository_searches_apps() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = AppRepository::new(&connection);

        repository
            .upsert_app(
                "com.apple.Terminal",
                "Terminal",
                "/System/Applications/Utilities/Terminal.app",
                None,
                "macos",
                "2026-06-04T00:00:00Z",
            )
            .expect("write terminal");
        repository
            .upsert_app(
                "com.apple.Safari",
                "Safari",
                "/Applications/Safari.app",
                None,
                "macos",
                "2026-06-04T00:00:00Z",
            )
            .expect("write safari");

        let results = repository.search_apps("term", 10).expect("search apps");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "com.apple.Terminal");
    }

    #[test]
    fn app_repository_increments_launch_count() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = AppRepository::new(&connection);

        repository
            .upsert_app(
                "com.example.App",
                "Example",
                "/Applications/Example.app",
                None,
                "macos",
                "2026-06-04T00:00:00Z",
            )
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
}
