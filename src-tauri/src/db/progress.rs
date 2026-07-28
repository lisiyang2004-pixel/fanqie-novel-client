use crate::error::AppResult;
use crate::models::ReadingProgress;
use rusqlite::params;
use super::Database;

impl Database {
    /// 保存/更新阅读进度
    pub fn save_reading_progress(&self, progress: &ReadingProgress) -> AppResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO reading_progress
                 (book_id, item_id, chapter_title, chapter_index,
                  scroll_percent, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    progress.book_id,
                    progress.item_id,
                    progress.chapter_title,
                    progress.chapter_index,
                    progress.scroll_percent,
                    progress.updated_at,
                ],
            )?;
            Ok(())
        })
    }

    /// 获取阅读进度
    pub fn get_reading_progress(&self, book_id: &str) -> AppResult<Option<ReadingProgress>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT book_id, item_id, chapter_title, chapter_index,
                        scroll_percent, updated_at
                 FROM reading_progress WHERE book_id = ?1",
            )?;

            let mut items = stmt
                .query_map(params![book_id], |row| {
                    Ok(ReadingProgress {
                        book_id: row.get(0)?,
                        item_id: row.get(1)?,
                        chapter_title: row.get(2)?,
                        chapter_index: row.get(3)?,
                        scroll_percent: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(items.pop())
        })
    }

    /// 删除阅读进度
    pub fn delete_reading_progress(&self, book_id: &str) -> AppResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM reading_progress WHERE book_id = ?1",
                params![book_id],
            )?;
            Ok(())
        })
    }
}
