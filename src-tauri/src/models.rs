use serde::{Deserialize, Serialize};

/// 搜索结果中的书籍信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBook {
    pub book_id: String,
    pub book_name: String,
    pub author: String,
    pub cover: String,
    pub r#abstract: String,
    pub category: String,
    /// 字数
    pub word_count: String,
    /// 连载状态: 0 连载中, 1 已完结
    pub book_status: i32,
    pub last_chapter_title: String,
    pub last_chapter_id: String,
}

/// 书籍详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookDetail {
    pub book_id: String,
    pub book_name: String,
    pub author: String,
    pub cover: String,
    pub r#abstract: String,
    pub category: String,
    pub word_count: String,
    pub book_status: i32,
    pub last_chapter_title: String,
    pub last_chapter_id: String,
    /// 章节总数
    pub chapter_count: i64,
    /// 最新章节时间戳(秒)
    pub last_chapter_time: i64,
}

/// 章节目录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterItem {
    pub item_id: String,
    pub title: String,
    /// 章节序号
    pub index: i64,
    /// 是否免费/VIP
    pub is_vip: bool,
}

/// 章节目录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterList {
    pub book_id: String,
    pub chapters: Vec<ChapterItem>,
}

/// 章节内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterContent {
    pub item_id: String,
    pub title: String,
    pub content: String,
    /// 上一章 item_id
    pub prev_item_id: Option<String>,
    /// 下一章 item_id
    pub next_item_id: Option<String>,
}

/// 书架中的书籍
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookshelfItem {
    pub book_id: String,
    pub book_name: String,
    pub author: String,
    pub cover: String,
    pub r#abstract: String,
    pub category: String,
    pub book_status: i32,
    pub last_chapter_title: String,
    pub last_chapter_id: String,
    /// 添加到书架的时间
    pub added_at: i64,
    /// 最后阅读时间
    pub last_read_at: Option<i64>,
    /// 阅读进度(章节序号)
    pub progress_chapter: Option<i64>,
    /// 阅读进度(章节标题)
    pub progress_title: Option<String>,
}

/// 下载历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadHistory {
    pub id: String,
    pub book_id: String,
    pub book_name: String,
    pub author: String,
    /// 文件格式: txt / epub
    pub format: String,
    /// 下载文件保存路径
    pub file_path: String,
    /// 文件大小(字节)
    pub file_size: i64,
    /// 下载状态: 0 进行中, 1 成功, 2 失败
    pub status: i32,
    /// 下载时间
    pub downloaded_at: i64,
    /// 章节数
    pub chapter_count: i64,
}

/// 阅读进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub book_id: String,
    pub item_id: String,
    pub chapter_title: String,
    pub chapter_index: i64,
    /// 滚动位置百分比 0-100
    pub scroll_percent: f64,
    pub updated_at: i64,
}

/// 下载任务进度事件
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub book_id: String,
    pub book_name: String,
    pub format: String,
    pub current: usize,
    pub total: usize,
    pub status: String,
    pub message: String,
}
