use rusqlite::{Connection, params};

use crate::schema::INITIAL_SCHEMA;

pub fn run_migrations(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(INITIAL_SCHEMA)?;
    add_column_if_missing(
        connection,
        "localized_names_json",
        "ALTER TABLE apps ADD COLUMN localized_names_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    add_column_if_missing(
        connection,
        "aliases_json",
        "ALTER TABLE apps ADD COLUMN aliases_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    add_column_if_missing(
        connection,
        "search_text",
        "ALTER TABLE apps ADD COLUMN search_text TEXT NOT NULL DEFAULT ''",
    )?;
    Ok(())
}

fn add_column_if_missing(
    connection: &Connection,
    column_name: &str,
    sql: &str,
) -> rusqlite::Result<()> {
    if app_column_exists(connection, column_name)? {
        return Ok(());
    }

    connection.execute(sql, [])?;
    Ok(())
}

fn app_column_exists(connection: &Connection, column_name: &str) -> rusqlite::Result<bool> {
    let mut statement =
        connection.prepare("SELECT 1 FROM pragma_table_info('apps') WHERE name = ?1")?;
    let mut rows = statement.query(params![column_name])?;
    Ok(rows.next()?.is_some())
}
