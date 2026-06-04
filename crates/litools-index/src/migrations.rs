use rusqlite::Connection;

use crate::schema::INITIAL_SCHEMA;

pub fn run_migrations(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(INITIAL_SCHEMA)
}
