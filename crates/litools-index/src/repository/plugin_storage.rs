use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginStorageRecord {
    pub plugin_id: String,
    pub key: String,
    pub value_json: String,
    pub updated_at: String,
}

pub struct PluginStorageRepository<'a> {
    connection: &'a Connection,
}

impl<'a> PluginStorageRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn get_json(&self, plugin_id: &str, key: &str) -> rusqlite::Result<Option<String>> {
        self.connection
            .query_row(
                "SELECT value_json FROM plugin_storage WHERE plugin_id = ?1 AND key = ?2",
                params![plugin_id, key],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn set_json(
        &self,
        plugin_id: &str,
        key: &str,
        value_json: &str,
        updated_at: &str,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO plugin_storage (plugin_id, key, value_json, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(plugin_id, key) DO UPDATE SET
                value_json = excluded.value_json,
                updated_at = excluded.updated_at",
            params![plugin_id, key, value_json, updated_at],
        )?;
        Ok(())
    }

    pub fn remove(&self, plugin_id: &str, key: &str) -> rusqlite::Result<usize> {
        self.connection.execute(
            "DELETE FROM plugin_storage WHERE plugin_id = ?1 AND key = ?2",
            params![plugin_id, key],
        )
    }

    pub fn clear(&self, plugin_id: &str) -> rusqlite::Result<usize> {
        self.connection.execute(
            "DELETE FROM plugin_storage WHERE plugin_id = ?1",
            params![plugin_id],
        )
    }

    pub fn list(&self, plugin_id: &str) -> rusqlite::Result<Vec<PluginStorageRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT plugin_id, key, value_json, updated_at
             FROM plugin_storage WHERE plugin_id = ?1 ORDER BY key ASC",
        )?;
        let rows = statement.query_map(params![plugin_id], |row| {
            Ok(PluginStorageRecord {
                plugin_id: row.get(0)?,
                key: row.get(1)?,
                value_json: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::IndexDatabase;
    use crate::repository::plugins::{PluginRepository, PluginUpsert};

    use super::*;

    fn seed_plugin(connection: &Connection, id: &str) {
        PluginRepository::new(connection)
            .upsert_plugin(PluginUpsert {
                id,
                name: id,
                version: "0.1.0",
                path: "/tmp/plugin",
                manifest_json: "{}",
                source: "user",
                enabled: true,
                trusted: true,
                installed_at: "2026-06-08T00:00:00Z",
                updated_at: "2026-06-08T00:00:00Z",
            })
            .expect("seed plugin");
    }

    #[test]
    fn plugin_storage_repository_scopes_values_by_plugin() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        seed_plugin(&connection, "plugin-a");
        seed_plugin(&connection, "plugin-b");
        let repository = PluginStorageRepository::new(&connection);

        repository.set_json("plugin-a", "count", "1", "2026-06-08T00:00:00Z").expect("write a");
        repository.set_json("plugin-b", "count", "2", "2026-06-08T00:00:00Z").expect("write b");
        repository.set_json("plugin-a", "count", "3", "2026-06-08T00:00:01Z").expect("overwrite a");

        assert_eq!(repository.get_json("plugin-a", "count").expect("read a"), Some("3".to_string()));
        assert_eq!(repository.get_json("plugin-b", "count").expect("read b"), Some("2".to_string()));
        assert_eq!(repository.get_json("plugin-a", "missing").expect("read missing"), None);
    }

    #[test]
    fn plugin_storage_repository_removes_and_clears_values() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        seed_plugin(&connection, "plugin-a");
        let repository = PluginStorageRepository::new(&connection);

        repository.set_json("plugin-a", "one", "1", "2026-06-08T00:00:00Z").expect("write one");
        repository.set_json("plugin-a", "two", "2", "2026-06-08T00:00:00Z").expect("write two");

        assert_eq!(repository.remove("plugin-a", "one").expect("remove one"), 1);
        assert_eq!(repository.get_json("plugin-a", "one").expect("read removed"), None);
        assert_eq!(repository.clear("plugin-a").expect("clear"), 1);
        assert!(repository.list("plugin-a").expect("list").is_empty());
    }
}
