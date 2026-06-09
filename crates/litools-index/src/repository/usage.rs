use rusqlite::{Connection, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageEventRecord {
    pub target_type: String,
    pub target_id: String,
    pub query: Option<String>,
    pub selected_at: String,
}

pub struct UsageRepository<'a> {
    connection: &'a Connection,
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
             FROM usage_events ORDER BY selected_at DESC LIMIT ?1",
        )?;
        let rows = statement.query_map(params![limit], usage_event_from_row)?;
        rows.collect()
    }

    pub fn recent_unique_targets(&self, limit: usize) -> rusqlite::Result<Vec<UsageEventRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT latest.target_type, latest.target_id, latest.query, latest.selected_at
             FROM usage_events latest
             JOIN (
                SELECT target_type, target_id, MAX(selected_at) AS selected_at
                FROM usage_events GROUP BY target_type, target_id
             ) grouped
                ON latest.target_type = grouped.target_type
               AND latest.target_id = grouped.target_id
               AND latest.selected_at = grouped.selected_at
             GROUP BY latest.target_type, latest.target_id
             ORDER BY latest.selected_at DESC LIMIT ?1",
        )?;
        let rows = statement.query_map(params![limit], usage_event_from_row)?;
        rows.collect()
    }
}

fn usage_event_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<UsageEventRecord> {
    Ok(UsageEventRecord {
        target_type: row.get(0)?,
        target_id: row.get(1)?,
        query: row.get(2)?,
        selected_at: row.get(3)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::IndexDatabase;

    use super::*;

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
            repository.record_selection(id, target_type, target_id, None, selected_at).expect("record usage");
        }

        let recent = repository.recent_unique_targets(10).expect("list recent unique targets");
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].target_type, "app");
        assert_eq!(recent[0].target_id, "com.example.App");
        assert_eq!(recent[0].selected_at, "2026-06-07T00:00:00Z");
    }
}
