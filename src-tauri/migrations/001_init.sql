-- 番茄小说客户端 初始数据库结构

-- 书架表
CREATE TABLE IF NOT EXISTS bookshelf (
    book_id            TEXT PRIMARY KEY,
    book_name          TEXT NOT NULL,
    author             TEXT NOT NULL DEFAULT '',
    cover              TEXT NOT NULL DEFAULT '',
    abstract           TEXT NOT NULL DEFAULT '',
    category           TEXT NOT NULL DEFAULT '',
    book_status        INTEGER NOT NULL DEFAULT 0,
    last_chapter_title TEXT NOT NULL DEFAULT '',
    last_chapter_id    TEXT NOT NULL DEFAULT '',
    added_at           INTEGER NOT NULL DEFAULT 0,
    last_read_at       INTEGER,
    progress_chapter   INTEGER,
    progress_title     TEXT
);

-- 下载历史表
CREATE TABLE IF NOT EXISTS download_history (
    id             TEXT PRIMARY KEY,
    book_id        TEXT NOT NULL,
    book_name      TEXT NOT NULL,
    author         TEXT NOT NULL DEFAULT '',
    format         TEXT NOT NULL,  -- txt / epub
    file_path      TEXT NOT NULL,
    file_size      INTEGER NOT NULL DEFAULT 0,
    status         INTEGER NOT NULL DEFAULT 0,  -- 0 进行中, 1 成功, 2 失败
    downloaded_at  INTEGER NOT NULL DEFAULT 0,
    chapter_count  INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_download_history_book_id ON download_history(book_id);
CREATE INDEX IF NOT EXISTS idx_download_history_downloaded_at ON download_history(downloaded_at DESC);

-- 阅读进度表
CREATE TABLE IF NOT EXISTS reading_progress (
    book_id         TEXT PRIMARY KEY,
    item_id         TEXT NOT NULL,
    chapter_title   TEXT NOT NULL DEFAULT '',
    chapter_index   INTEGER NOT NULL DEFAULT 0,
    scroll_percent  REAL NOT NULL DEFAULT 0.0,
    updated_at      INTEGER NOT NULL DEFAULT 0
);
