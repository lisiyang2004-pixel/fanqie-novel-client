use crate::error::AppResult;
use crate::models::DownloadHistory;
use rusqlite::params;
use super::Database;

impl Database {
    /// 添加下载历史记录
    pub fn add_download_history(&self, history: &DownloadHistory) -> AppResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO download_history
                 (id, book_id, book_name, author, format, file_path,
                  file_size, status, downloaded_at, chapter_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    history.id,
                    history.book_id,
                    history.book_name,
                    history.author,
                    history.format,
                    history.file_path,
                    history.file_size,
                    history.status,
                    history.downloaded_at,
                    history.chapter_count,
                ],
            )?;
            Ok(())
        })
    }

    /// 更新下载状态
    pub fn update_download_status(
        &self,
        id: &str,
        status: i32,
        file_size: i64,
    ) -> AppResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE download_history SET status = ?1, file_size = ?2 WHERE id = ?3",
                params![status, file_size, id],
            )?;
            Ok(())
        })
    }

    /// 获取下载历史(按时间倒序)
    pub fn get_download_history(&self) -> AppResult<Vec<DownloadHistory>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, book_id, book_name, author, format, file_path,
                        file_size, status, downloaded_at, chapter_count
                 FROM download_history
                 ORDER BY downloaded_at DESC",
            )?;

            let items = stmt
                .query_map([], |row| {
                    Ok(DownloadHistory {
                        id: row.get(0)?,
                        book_id: row.get(1)?,
                        book_name: row.get(2)?,
                        author: row.get(3)?,
                        format: row.get(4)?,
                        file_path: row.get(5)?,
                        file_size: row.get(6)?,
                        status: row.get(7)?,
                        downloaded_at: row.get(8)?,
                        chapter_count: row.get(9)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(items)
        })
    }

    /// 获取指定书籍的下载历史
    pub fn get_download_history_by_book(
        &self,
        book_id: &str,
    ) -> AppResult<Vec<DownloadHistory>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, book_id, book_name, author, format, file_path,
                        file_size, status, downloaded_at, chapter_count
                 FROM download_history
                 WHERE book_id = ?1
                 ORDER BY downloaded_at DESC",
            )?;

            let items = stmt
                .query_map(params![book_id], |row| {
                    Ok(DownloadHistory {
                        id: row.get(0)?,
                        book_id: row.get(1)?,
                        book_name: row.get(2)?,
                        author: row.get(3)?,
                        format: row.get(4)?,
                        file_path: row.get(5)?,
                        file_size: row.get(6)?,
                        status: row.get(7)?,
                        downloaded_at: row.get(8)?,
                        chapter_count: row.get(9)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(items)
        })
    }

    /// 删除下载历史记录
    pub fn delete_download_history(&self, id: &str) -> AppResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM download_history WHERE id = ?1",
                params![id],
            )?;
            Ok(())
        })
    }

    /// 清空下载历史
    pub fn clear_download_history(&self) -> AppResult<()> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM download_history", [])?;
            Ok(())
        })
    }
}
