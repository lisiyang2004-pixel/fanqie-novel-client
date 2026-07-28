use crate::error::AppResult;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::Path;
use std::sync::Mutex;

pub mod bookshelf;
pub mod history;
pub mod progress;

/// 数据库封装
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// 打开/创建数据库
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Self::migrate(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 内存数据库(用于测试)
    #[cfg(test)]
    pub fn in_memory() -> AppResult<Self> {
        let mut conn = Connection::open_in_memory()?;
        Self::migrate(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn migrate(conn: &mut Connection) -> AppResult<()> {
        let migrations = Migrations::new(vec![
            M::up(include_str!("../../migrations/001_init.sql")),
        ]);
        migrations.to_latest(conn).map_err(|e| {
            crate::error::AppError::Migration(rusqlite_migration::Error::from(e))
        })?;
        Ok(())
    }

    /// 获取连接锁(内部使用)
    pub(crate) fn with_conn<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        let conn = self.conn.lock().map_err(|e| {
            crate::error::AppError::Other(anyhow::anyhow!(
                "数据库锁获取失败: {}",
                e
            ))
        })?;
        f(&conn)
    }
}
