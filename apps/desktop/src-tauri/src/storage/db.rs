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

/// 应用数据库，封装 SQLite 连接。
///
/// 使用 Mutex 保证多线程安全（Tauri 的 invoke 在异步线程中调用）。
/// 默认路径: ~/.xunji/db/xunji.db
pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    /// 打开指定路径的数据库，自动创建目录、启用 WAL 模式并执行迁移。
    pub fn open(path: &Path) -> DbResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        log::info!("打开数据库: {}", path.display());
        let conn = Connection::open(path)?;
        // WAL 模式允许 MCP Server 只读并发访问；busy_timeout 避免写锁竞争时立即失败
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
        log::info!("数据库就绪");
        Ok(db)
    }

    /// 使用默认路径 ~/.xunji/db/xunji.db 打开数据库
    pub fn open_default() -> DbResult<Self> {
        let base = dirs::home_dir().expect("无法获取用户主目录");
        let path = base.join(".xunji").join("db").join("xunji.db");
        Self::open(&path)
    }

    /// 获取数据库连接的 MutexGuard（阻塞获取锁）
    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("数据库 Mutex 已中毒")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn migrate(&self) -> DbResult<()> {
        migrations::run(&self.conn())
    }
}
