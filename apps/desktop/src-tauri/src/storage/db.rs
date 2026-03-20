use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

use super::migrations;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type DbResult<T> = Result<T, DbError>;

pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    pub fn open(path: &Path) -> DbResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;",
        )?;

        let db = Self {
            conn: Mutex::new(conn),
            path: path.to_path_buf(),
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_default() -> DbResult<Self> {
        let base = dirs::home_dir().expect("cannot determine home directory");
        let path = base.join(".xunji").join("db").join("xunji.db");
        Self::open(&path)
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("database mutex poisoned")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn migrate(&self) -> DbResult<()> {
        migrations::run(&self.conn())
    }
}
