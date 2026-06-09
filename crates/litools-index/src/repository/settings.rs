use rusqlite::{Connection, params};

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
}
