use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexMetadataRecord {
    pub key: String,
    pub value_json: String,
    pub updated_at: String,
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

#[cfg(test)]
mod tests {
    use crate::IndexDatabase;

    use super::*;

    #[test]
    fn index_metadata_repository_round_trips_json() {
        let database = IndexDatabase::in_memory().expect("in-memory database");
        let connection = database.connection();
        let repository = IndexMetadataRepository::new(&connection);

        repository
            .set_json("apps_last_refresh_status", r#"{"success":true}"#, "2026-06-05T00:00:00Z")
            .expect("write metadata");

        let metadata = repository.get("apps_last_refresh_status").expect("read metadata").expect("metadata exists");
        assert_eq!(metadata.key, "apps_last_refresh_status");
        assert_eq!(metadata.value_json, r#"{"success":true}"#);
        assert_eq!(metadata.updated_at, "2026-06-05T00:00:00Z");
    }
}
