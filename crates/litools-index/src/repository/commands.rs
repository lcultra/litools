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

#[cfg(test)]
mod tests {
    use crate::IndexDatabase;

    use super::*;

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
