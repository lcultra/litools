use rusqlite::{Connection, params};

pub struct CommandRepository<'a> {
    connection: &'a Connection,
}

impl<'a> CommandRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
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
