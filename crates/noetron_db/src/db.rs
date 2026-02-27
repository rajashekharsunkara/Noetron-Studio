//! Database handle — opens and manages the SQLite connection.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

/// One `Db` per open `.aiproj` project.
pub struct Db {
    pub conn: Connection,
}

impl Db {
    /// Open (or create) the project database at `path`.
    /// Runs all pending migrations before returning.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("opening db at {}", path.display()))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA synchronous=NORMAL;",
        )?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    /// In-memory database — used in tests.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        crate::migrations::run(&self.conn)
    }
}
