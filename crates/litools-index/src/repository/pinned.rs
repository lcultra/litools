use rusqlite::{Connection, params};

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
             FROM pinned_items ORDER BY sort_order ASC, pinned_at DESC LIMIT ?1",
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
             ON CONFLICT(target_type, target_id) DO UPDATE SET pinned_at = excluded.pinned_at",
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
                "UPDATE pinned_items SET sort_order = ?1 WHERE target_type = ?2 AND target_id = ?3",
                params![sort_order as i64, target_type, target_id],
            )?;
        }

        transaction.commit()
    }
}

#[cfg(test)]
mod tests {
    use crate::IndexDatabase;

    use super::*;

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
        assert_eq!(pinned[1].target_id, "open-settings");

        repository
            .reorder(&[
                ("command".to_string(), "open-settings".to_string()),
                ("app".to_string(), "com.example.App".to_string()),
            ])
            .expect("reorder");

        let pinned = repository.list_pinned(10).expect("list reordered");
        assert_eq!(pinned[0].target_id, "open-settings");

        repository.unpin("app", "com.example.App").expect("unpin");
        assert!(
            !repository
                .is_pinned("app", "com.example.App")
                .expect("check unpinned")
        );
    }
}
