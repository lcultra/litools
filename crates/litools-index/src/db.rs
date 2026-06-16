use std::{
    ops::{Deref, DerefMut},
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

use rusqlite::Connection;

use crate::migrations::run_migrations;

#[derive(Clone)]
pub struct IndexDatabase {
    connection: Arc<Mutex<Connection>>,
}

/// Wraps `MutexGuard<Connection>` and clears a re-entrancy flag on drop.
pub struct ConnectionGuard<'a> {
    guard: MutexGuard<'a, Connection>,
}

impl Deref for ConnectionGuard<'_> {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl DerefMut for ConnectionGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

#[cfg(debug_assertions)]
mod reentrancy {
    use std::cell::Cell;
    use std::thread;

    thread_local! {
        static LOCK_HELD: Cell<bool> = const { Cell::new(false) };
    }

    pub(super) fn check_and_mark() {
        LOCK_HELD.with(|held| {
            if held.get() {
                panic!(
                    "IndexDatabase: recursive lock on thread {:?}. \
                     A ConnectionGuard is already held — drop it before calling \
                     any method that re-acquires the database lock.",
                    thread::current().id()
                );
            }
            held.set(true);
        });
    }

    pub(super) fn clear() {
        LOCK_HELD.with(|held| held.set(false));
    }
}

impl Drop for ConnectionGuard<'_> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        reentrancy::clear();
    }
}

impl IndexDatabase {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        prepare_connection(&connection)?;
        run_migrations(&connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn in_memory() -> rusqlite::Result<Self> {
        let connection = Connection::open_in_memory()?;
        prepare_connection(&connection)?;
        run_migrations(&connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// 在数据库读锁内执行闭包，自动管理锁生命周期。
    ///
    /// 替代"获取 connection → 构造 Repository → 调用方法 → drop connection"
    /// 的样板代码。`f` 接收 `&Connection`，返回值由调用方通过 `?` 传播。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let result = db.read(|conn| AppRepository::new(&conn).find_app(id))?;
    /// ```
    pub fn read<T>(&self, f: impl FnOnce(&Connection) -> T) -> T {
        let guard = self.connection();
        f(&guard)
    }

    /// Acquire the database connection lock.
    ///
    /// # Deadlock safety
    ///
    /// `std::sync::Mutex` is not re-entrant. Do **not** call this method (or
    /// any method that internally calls it) while holding a returned guard.
    /// In debug builds a thread-local flag detects recursive attempts and
    /// panics with a descriptive message.
    pub fn connection(&self) -> ConnectionGuard<'_> {
        #[cfg(debug_assertions)]
        reentrancy::check_and_mark();

        let guard = self.connection.lock().expect("database mutex poisoned");
        ConnectionGuard { guard }
    }
}

fn prepare_connection(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch("PRAGMA foreign_keys = ON;")
}
