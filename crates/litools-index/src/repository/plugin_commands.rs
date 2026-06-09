use rusqlite::{Connection, OptionalExtension, params};

use super::json_string_array;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginCommandRecord {
    pub id: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub plugin_path: String,
    pub plugin_icon: String,
    pub command_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub keywords: Vec<String>,
    pub mode: String,
    pub permission_requirements: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PluginCommandUpsert {
    pub id: String,
    pub plugin_id: String,
    pub command_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub keywords: Vec<String>,
    pub mode: String,
    pub permission_requirements: Vec<String>,
}

pub struct PluginCommandRepository<'a> {
    connection: &'a Connection,
}

impl<'a> PluginCommandRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn replace_commands_for_plugin(
        &self,
        plugin_id: &str,
        commands: &[PluginCommandUpsert],
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "DELETE FROM plugin_commands WHERE plugin_id = ?1",
            params![plugin_id],
        )?;

        for command in commands {
            self.upsert_command(command)?;
        }

        Ok(())
    }

    fn upsert_command(&self, command: &PluginCommandUpsert) -> rusqlite::Result<()> {
        let keywords_json = serde_json::to_string(&command.keywords)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?;
        let permission_requirements_json = serde_json::to_string(&command.permission_requirements)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?;

        self.connection.execute(
            "INSERT INTO plugin_commands (id, plugin_id, command_id, title, subtitle, keywords, mode, permission_requirements)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                plugin_id = excluded.plugin_id, command_id = excluded.command_id,
                title = excluded.title, subtitle = excluded.subtitle,
                keywords = excluded.keywords, mode = excluded.mode,
                permission_requirements = excluded.permission_requirements",
            params![
                &command.id, &command.plugin_id, &command.command_id,
                &command.title, command.subtitle.as_deref(),
                keywords_json, &command.mode, permission_requirements_json,
            ],
        )?;
        Ok(())
    }

    pub fn list_enabled_plugin_commands(&self) -> rusqlite::Result<Vec<PluginCommandRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT pc.id, pc.plugin_id, p.name, p.path, json_extract(p.manifest_json, '$.icon'),
                    pc.command_id, pc.title, pc.subtitle, pc.keywords, pc.mode, pc.permission_requirements
             FROM plugin_commands pc
             JOIN plugins p ON p.id = pc.plugin_id
             WHERE p.enabled = 1
             ORDER BY p.name ASC, pc.title ASC",
        )?;
        let rows = statement.query_map([], plugin_command_record_from_row)?;
        rows.collect()
    }

    pub fn find_plugin_command(
        &self,
        plugin_id: &str,
        command_id: &str,
    ) -> rusqlite::Result<Option<PluginCommandRecord>> {
        self.connection
            .query_row(
                "SELECT pc.id, pc.plugin_id, p.name, p.path, json_extract(p.manifest_json, '$.icon'),
                        pc.command_id, pc.title, pc.subtitle, pc.keywords, pc.mode, pc.permission_requirements
                 FROM plugin_commands pc
                 JOIN plugins p ON p.id = pc.plugin_id
                 WHERE pc.plugin_id = ?1 AND pc.command_id = ?2 AND p.enabled = 1",
                params![plugin_id, command_id],
                plugin_command_record_from_row,
            )
            .optional()
    }
}

fn plugin_command_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PluginCommandRecord> {
    Ok(PluginCommandRecord {
        id: row.get(0)?,
        plugin_id: row.get(1)?,
        plugin_name: row.get(2)?,
        plugin_path: row.get(3)?,
        plugin_icon: row.get(4)?,
        command_id: row.get(5)?,
        title: row.get(6)?,
        subtitle: row.get(7)?,
        keywords: json_string_array(row.get::<_, String>(8)?)?,
        mode: row.get(9)?,
        permission_requirements: json_string_array(row.get::<_, String>(10)?)?,
    })
}
