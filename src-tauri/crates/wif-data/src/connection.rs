use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

/// Thread-safe wrapper around a `rusqlite::Connection`.
///
/// The inner connection is guarded by a `Mutex` so that `Database: Send + Sync`,
/// satisfying the `Send + Sync` bounds on all `wif-domain` repository traits.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open (or create) a database file at the given path.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Open an in-memory database (useful for tests).
    pub fn open_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Execute a function with the locked connection.
    ///
    /// All repository operations go through this method to keep lock lifetime minimal.
    pub fn with_conn<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&Connection) -> anyhow::Result<T>,
    {
        let guard = self.conn.lock().expect("database mutex poisoned");
        f(&guard)
    }

    /// Apply all pending migrations.
    pub fn run_migrations(&self) -> anyhow::Result<()> {
        let guard = self.conn.lock().expect("database mutex poisoned");
        crate::migrations::run_all(&guard)
    }
}
