use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

use rusqlite::Connection;

use crate::migrations::run_migrations;

#[derive(Clone)]
pub struct IndexDatabase {
    connection: Arc<Mutex<Connection>>,
}

impl IndexDatabase {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        run_migrations(&connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn in_memory() -> rusqlite::Result<Self> {
        let connection = Connection::open_in_memory()?;
        run_migrations(&connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn connection(&self) -> MutexGuard<'_, Connection> {
        self.connection.lock().expect("database mutex poisoned")
    }
}
