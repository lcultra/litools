use rusqlite::{Connection, params};

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
}
