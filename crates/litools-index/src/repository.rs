use rusqlite::{Connection, OptionalExtension, ToSql, params, params_from_iter};

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

    pub fn search_apps(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> rusqlite::Result<Vec<AppRecord>> {
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

    pub fn list_apps_for_search(&self) -> rusqlite::Result<Vec<AppRecord>> {
        let mut statement = self.connection.prepare(
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
             ORDER BY launch_count DESC, name ASC",
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
            "INSERT INTO plugins (
                id,
                name,
                version,
                path,
                manifest_json,
                source,
                enabled,
                trusted,
                installed_at,
                updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                version = excluded.version,
                path = excluded.path,
                manifest_json = excluded.manifest_json,
                source = excluded.source,
                updated_at = excluded.updated_at",
            params![
                plugin.id,
                plugin.name,
                plugin.version,
                plugin.path,
                plugin.manifest_json,
                plugin.source,
                plugin.enabled,
                plugin.trusted,
                plugin.installed_at,
                plugin.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_plugins(&self) -> rusqlite::Result<Vec<PluginRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT id,
                    name,
                    version,
                    path,
                    manifest_json,
                    source,
                    enabled,
                    trusted,
                    installed_at,
                    updated_at
             FROM plugins
             ORDER BY name ASC, id ASC",
        )?;
        let rows = statement.query_map([], plugin_record_from_row)?;
        rows.collect()
    }

    pub fn find_plugin(&self, id: &str) -> rusqlite::Result<Option<PluginRecord>> {
        self.connection
            .query_row(
                "SELECT id,
                        name,
                        version,
                        path,
                        manifest_json,
                        source,
                        enabled,
                        trusted,
                        installed_at,
                        updated_at
                 FROM plugins
                 WHERE id = ?1",
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
                "DELETE FROM plugin_commands
                 WHERE plugin_id IN (SELECT id FROM plugins WHERE source = ?1)",
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
            "DELETE FROM plugin_commands
             WHERE plugin_id IN (
                SELECT id FROM plugins WHERE source = ? AND id NOT IN ({placeholders})
             )"
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

    pub fn set_enabled(&self, id: &str, enabled: bool, updated_at: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE plugins SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled, updated_at, id],
        )?;
        Ok(())
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

    pub fn upsert_command(&self, command: &PluginCommandUpsert) -> rusqlite::Result<()> {
        let keywords_json = serde_json::to_string(&command.keywords)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?;
        let permission_requirements_json = serde_json::to_string(&command.permission_requirements)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?;

        self.connection.execute(
            "INSERT INTO plugin_commands (
                id,
                plugin_id,
                command_id,
                title,
                subtitle,
                keywords,
                mode,
                permission_requirements
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                plugin_id = excluded.plugin_id,
                command_id = excluded.command_id,
                title = excluded.title,
                subtitle = excluded.subtitle,
                keywords = excluded.keywords,
                mode = excluded.mode,
                permission_requirements = excluded.permission_requirements",
            params![
                &command.id,
                &command.plugin_id,
                &command.command_id,
                &command.title,
                command.subtitle.as_deref(),
                keywords_json,
                &command.mode,
                permission_requirements_json,
            ],
        )?;
        Ok(())
    }

    pub fn list_enabled_plugin_commands(&self) -> rusqlite::Result<Vec<PluginCommandRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT plugin_commands.id,
                    plugin_commands.plugin_id,
                    plugins.name,
                    plugins.path,
                    json_extract(plugins.manifest_json, '$.icon'),
                    plugin_commands.command_id,
                    plugin_commands.title,
                    plugin_commands.subtitle,
                    plugin_commands.keywords,
                    plugin_commands.mode,
                    plugin_commands.permission_requirements
             FROM plugin_commands
             JOIN plugins ON plugins.id = plugin_commands.plugin_id
             WHERE plugins.enabled = 1
             ORDER BY plugins.name ASC, plugin_commands.title ASC",
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
                "SELECT plugin_commands.id,
                        plugin_commands.plugin_id,
                        plugins.name,
                        plugins.path,
                        json_extract(plugins.manifest_json, '$.icon'),
                        plugin_commands.command_id,
                        plugin_commands.title,
                        plugin_commands.subtitle,
                        plugin_commands.keywords,
                        plugin_commands.mode,
                        plugin_commands.permission_requirements
                 FROM plugin_commands
                 JOIN plugins ON plugins.id = plugin_commands.plugin_id
                 WHERE plugin_commands.plugin_id = ?1
                   AND plugin_commands.command_id = ?2
                   AND plugins.enabled = 1",
                params![plugin_id, command_id],
                plugin_command_record_from_row,
            )
            .optional()
    }
}

fn plugin_command_record_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PluginCommandRecord> {
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
             FROM plugin_storage
             WHERE plugin_id = ?1
             ORDER BY key ASC",
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

    pub fn pin(&self, target_type: &str, target_id: &str, pinned_at: &str) -> rusqlite::Result<()> {
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
    fn plugin_storage_repository_scopes_values_by_plugin() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        seed_plugin(&connection, "plugin-a");
        seed_plugin(&connection, "plugin-b");
        let repository = PluginStorageRepository::new(&connection);

        repository
            .set_json("plugin-a", "count", "1", "2026-06-08T00:00:00Z")
            .expect("write plugin a value");
        repository
            .set_json("plugin-b", "count", "2", "2026-06-08T00:00:00Z")
            .expect("write plugin b value");
        repository
            .set_json("plugin-a", "count", "3", "2026-06-08T00:00:01Z")
            .expect("overwrite plugin a value");

        assert_eq!(
            repository
                .get_json("plugin-a", "count")
                .expect("read plugin a"),
            Some("3".to_string())
        );
        assert_eq!(
            repository
                .get_json("plugin-b", "count")
                .expect("read plugin b"),
            Some("2".to_string())
        );
        assert_eq!(
            repository
                .get_json("plugin-a", "missing")
                .expect("read missing"),
            None
        );
    }

    #[test]
    fn plugin_storage_repository_removes_and_clears_values() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        seed_plugin(&connection, "plugin-a");
        let repository = PluginStorageRepository::new(&connection);

        repository
            .set_json("plugin-a", "one", "1", "2026-06-08T00:00:00Z")
            .expect("write first value");
        repository
            .set_json("plugin-a", "two", "2", "2026-06-08T00:00:00Z")
            .expect("write second value");

        assert_eq!(
            repository
                .remove("plugin-a", "one")
                .expect("remove first value"),
            1
        );
        assert_eq!(
            repository
                .get_json("plugin-a", "one")
                .expect("read removed"),
            None
        );
        assert_eq!(repository.clear("plugin-a").expect("clear plugin"), 1);
        assert!(
            repository
                .list("plugin-a")
                .expect("list cleared plugin")
                .is_empty()
        );
    }

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

        let results = repository
            .search_apps("term", Some(10))
            .expect("search apps");

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

        let by_alias = repository
            .search_apps("weixin", Some(10))
            .expect("search alias");
        let by_initials = repository
            .search_apps("wx", Some(10))
            .expect("search initials");

        assert_eq!(by_alias[0].id, "com.tencent.xin");
        assert_eq!(by_initials[0].id, "com.tencent.xin");
        assert_eq!(by_alias[0].localized_names, localized_names);
        assert_eq!(by_alias[0].aliases, aliases);
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
                search_text: "Safari com.apple.Safari /Applications/Safari.app",
                platform: "macos",
                last_seen_at: "2026-06-04T00:00:00Z",
            })
            .expect("write safari");

        let results = repository.list_apps_for_search().expect("list apps");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "com.apple.Safari");
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
