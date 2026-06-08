use rusqlite::Connection;

use crate::schema::INITIAL_SCHEMA;

pub fn run_migrations(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(INITIAL_SCHEMA)?;
    ensure_plugin_columns(connection)?;
    ensure_plugin_command_columns(connection)?;
    Ok(())
}

fn ensure_plugin_columns(connection: &Connection) -> rusqlite::Result<()> {
    if !column_exists(connection, "plugins", "source")? {
        connection.execute(
            "ALTER TABLE plugins ADD COLUMN source TEXT NOT NULL DEFAULT 'user'",
            [],
        )?;
    }
    Ok(())
}

fn ensure_plugin_command_columns(connection: &Connection) -> rusqlite::Result<()> {
    if !column_exists(connection, "plugin_commands", "mode")? {
        connection.execute(
            "ALTER TABLE plugin_commands ADD COLUMN mode TEXT NOT NULL DEFAULT 'view'",
            [],
        )?;
    }
    connection.execute(
        "UPDATE plugin_commands SET keywords = '[]' WHERE keywords IS NULL",
        [],
    )?;
    Ok(())
}

fn column_exists(connection: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;

    for row in rows {
        if row? == column {
            return Ok(true);
        }
    }

    Ok(false)
}
