use rusqlite::{Connection, OptionalExtension, ToSql, params, params_from_iter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginRecord {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: String,
    pub manifest_json: String,
    pub source: String,
    pub enabled: bool,
    pub trusted: bool,
    pub installed_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug)]
pub struct PluginUpsert<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub version: &'a str,
    pub path: &'a str,
    pub manifest_json: &'a str,
    pub source: &'a str,
    pub enabled: bool,
    pub trusted: bool,
    pub installed_at: &'a str,
    pub updated_at: &'a str,
}

pub struct PluginRepository<'a> {
    connection: &'a Connection,
}

impl<'a> PluginRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn count_plugins(&self) -> rusqlite::Result<usize> {
        self.connection
            .query_row("SELECT COUNT(*) FROM plugins", [], |row| row.get(0))
    }

    pub fn upsert_plugin(&self, plugin: PluginUpsert<'_>) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO plugins (id, name, version, path, manifest_json, source, enabled, trusted, installed_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name, version = excluded.version, path = excluded.path,
                manifest_json = excluded.manifest_json, source = excluded.source,
                updated_at = excluded.updated_at",
            params![
                plugin.id, plugin.name, plugin.version, plugin.path,
                plugin.manifest_json, plugin.source, plugin.enabled, plugin.trusted,
                plugin.installed_at, plugin.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_plugins(&self) -> rusqlite::Result<Vec<PluginRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, version, path, manifest_json, source, enabled, trusted, installed_at, updated_at
             FROM plugins ORDER BY name ASC, id ASC",
        )?;
        let rows = statement.query_map([], plugin_record_from_row)?;
        rows.collect()
    }

    pub fn find_plugin(&self, id: &str) -> rusqlite::Result<Option<PluginRecord>> {
        self.connection
            .query_row(
                "SELECT id, name, version, path, manifest_json, source, enabled, trusted, installed_at, updated_at
                 FROM plugins WHERE id = ?1",
                params![id],
                plugin_record_from_row,
            )
            .optional()
    }

    pub fn delete_plugins_not_in_source_ids(
        &self,
        source: &str,
        seen_ids: &[String],
    ) -> rusqlite::Result<usize> {
        if seen_ids.is_empty() {
            let removed_commands = self.connection.execute(
                "DELETE FROM plugin_commands WHERE plugin_id IN (SELECT id FROM plugins WHERE source = ?1)",
                params![source],
            )?;
            let removed_plugins = self
                .connection
                .execute("DELETE FROM plugins WHERE source = ?1", params![source])?;
            return Ok(removed_plugins + removed_commands);
        }

        let placeholders = std::iter::repeat_n("?", seen_ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let command_sql = format!(
            "DELETE FROM plugin_commands WHERE plugin_id IN (SELECT id FROM plugins WHERE source = ? AND id NOT IN ({placeholders}))"
        );
        let plugin_sql =
            format!("DELETE FROM plugins WHERE source = ? AND id NOT IN ({placeholders})");
        let mut values: Vec<&dyn ToSql> = Vec::with_capacity(seen_ids.len() + 1);
        values.push(&source);
        for id in seen_ids {
            values.push(id);
        }

        let removed_commands = self
            .connection
            .execute(&command_sql, params_from_iter(values.iter().copied()))?;
        let removed_plugins = self
            .connection
            .execute(&plugin_sql, params_from_iter(values.iter().copied()))?;
        Ok(removed_plugins + removed_commands)
    }
}

fn plugin_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PluginRecord> {
    Ok(PluginRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        version: row.get(2)?,
        path: row.get(3)?,
        manifest_json: row.get(4)?,
        source: row.get(5)?,
        enabled: row.get(6)?,
        trusted: row.get(7)?,
        installed_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

#[cfg(test)]
mod tests {
    // PluginRepository tests rely on plugin_command table foreign keys;
    // comprehensive tests live in plugin_commands and plugin_storage modules.
}
