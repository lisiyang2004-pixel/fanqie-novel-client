use crate::error::AppResult;
use crate::models::BookshelfItem;
use rusqlite::{params, Connection};
use super::Database;

impl Database {
    /// 添加书籍到书架
    pub fn add_to_bookshelf(&self, item: &BookshelfItem) -> AppResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO bookshelf
                 (book_id, book_name, author, cover, abstract, category,
                  book_status, last_chapter_title, last_chapter_id,
                  added_at, last_read_at, progress_chapter, progress_title)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    item.book_id,
                    item.book_name,
                    item.author,
                    item.cover,
                    item.r#abstract,
                    item.category,
                    item.book_status,
                    item.last_chapter_title,
                    item.last_chapter_id,
                    item.added_at,
                    item.last_read_at,
                    item.progress_chapter,
                    item.progress_title,
                ],
            )?;
            Ok(())
        })
    }

    /// 从书架移除
    pub fn remove_from_bookshelf(&self, book_id: &str) -> AppResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM bookshelf WHERE book_id = ?1",
                params![book_id],
            )?;
            Ok(())
        })
    }

    /// 获取书架列表(按最后阅读时间排序)
    pub fn get_bookshelf(&self) -> AppResult<Vec<BookshelfItem>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT book_id, book_name, author, cover, abstract, category,
                        book_status, last_chapter_title, last_chapter_id,
                        added_at, last_read_at, progress_chapter, progress_title
                 FROM bookshelf
                 ORDER BY COALESCE(last_read_at, added_at) DESC",
            )?;

            let items = stmt
                .query_map([], |row| {
                    Ok(BookshelfItem {
                        book_id: row.get(0)?,
                        book_name: row.get(1)?,
                        author: row.get(2)?,
                        cover: row.get(3)?,
                        r#abstract: row.get(4)?,
                        category: row.get(5)?,
                        book_status: row.get(6)?,
                        last_chapter_title: row.get(7)?,
                        last_chapter_id: row.get(8)?,
                        added_at: row.get(9)?,
                        last_read_at: row.get(10)?,
                        progress_chapter: row.get(11)?,
                        progress_title: row.get(12)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(items)
        })
    }

    /// 检查是否在书架中
    pub fn is_in_bookshelf(&self, book_id: &str) -> AppResult<bool> {
        self.with_conn(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM bookshelf WHERE book_id = ?1",
                params![book_id],
                |row| row.get(0),
            )?;
            Ok(count > 0)
        })
    }

    /// 更新阅读进度
    pub fn update_bookshelf_progress(
        &self,
        book_id: &str,
        chapter_index: i64,
        chapter_title: &str,
    ) -> AppResult<()> {
        self.with_conn(|conn| {
            let now = chrono::Utc::now().timestamp();
            conn.execute(
                "UPDATE bookshelf
                 SET last_read_at = ?1, progress_chapter = ?2, progress_title = ?3
                 WHERE book_id = ?4",
                params![now, chapter_index, chapter_title, book_id],
            )?;
            Ok(())
        })
    }

    /// 更新书架中书籍的最新章节信息
    pub fn update_bookshelf_info(
        &self,
        book_id: &str,
        last_chapter_title: &str,
        last_chapter_id: &str,
    ) -> AppResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE bookshelf
                 SET last_chapter_title = ?1, last_chapter_id = ?2
                 WHERE book_id = ?3",
                params![last_chapter_title, last_chapter_id, book_id],
            )?;
            Ok(())
        })
    }

    /// 获取书架中的指定书籍
    pub fn get_bookshelf_item(&self, book_id: &str) -> AppResult<Option<BookshelfItem>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT book_id, book_name, author, cover, abstract, category,
                        book_status, last_chapter_title, last_chapter_id,
                        added_at, last_read_at, progress_chapter, progress_title
                 FROM bookshelf WHERE book_id = ?1",
            )?;

            let mut items = stmt
                .query_map(params![book_id], |row| {
                    Ok(BookshelfItem {
                        book_id: row.get(0)?,
                        book_name: row.get(1)?,
                        author: row.get(2)?,
                        cover: row.get(3)?,
                        r#abstract: row.get(4)?,
                        category: row.get(5)?,
                        book_status: row.get(6)?,
                        last_chapter_title: row.get(7)?,
                        last_chapter_id: row.get(8)?,
                        added_at: row.get(9)?,
                        last_read_at: row.get(10)?,
                        progress_chapter: row.get(11)?,
                        progress_title: row.get(12)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(items.pop())
        })
    }
}

/// 辅助函数：从 Connection 创建 BookshelfItem
#[allow(dead_code)]
fn row_to_bookshelf_item(row: &rusqlite::Row) -> rusqlite::Result<BookshelfItem> {
    Ok(BookshelfItem {
        book_id: row.get(0)?,
        book_name: row.get(1)?,
        author: row.get(2)?,
        cover: row.get(3)?,
        r#abstract: row.get(4)?,
        category: row.get(5)?,
        book_status: row.get(6)?,
        last_chapter_title: row.get(7)?,
        last_chapter_id: row.get(8)?,
        added_at: row.get(9)?,
        last_read_at: row.get(10)?,
        progress_chapter: row.get(11)?,
        progress_title: row.get(12)?,
    })
}

#[allow(dead_code)]
const _BOOKSHELF_SELECT: &str =
    "SELECT book_id, book_name, author, cover, abstract, category,
     book_status, last_chapter_title, last_chapter_id,
     added_at, last_read_at, progress_chapter, progress_title
     FROM bookshelf";

// 标记 Connection 类型已使用
#[allow(dead_code)]
fn _ensure_conn_used(_c: &Connection) {}
