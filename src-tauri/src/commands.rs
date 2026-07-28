use crate::api::FanqieClient;
use crate::db::Database;
use crate::downloader::{DownloadFormat, Downloader};
use crate::error::{AppError, AppResult};
use crate::models::{
    BookDetail, BookshelfItem, ChapterContent, ChapterItem, DownloadHistory,
    DownloadProgress, ReadingProgress, SearchBook,
};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State};

/// 应用共享状态
pub struct AppState {
    pub db: Database,
    pub client: FanqieClient,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            client: FanqieClient::new(),
        }
    }
}

// ==================== 搜索命令 ====================

/// 搜索小说
///
/// 支持三种输入：
/// 1. 书籍 ID: `7143038691944959011`
/// 2. 书籍 URL: `https://fanqienovel.com/page/7143038691944959011`
/// 3. 关键词（书名/作者）: `十日终焉`（通过必应搜索间接获取）
#[tauri::command]
pub async fn search_novels(
    query: String,
    _offset: Option<i64>,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> AppResult<Vec<SearchBook>> {
    let limit = limit.unwrap_or(10) as usize;
    let query = query.trim().to_string();

    if query.is_empty() {
        return Err(AppError::InvalidParam("搜索内容不能为空".to_string()));
    }

    // 先尝试解析为书籍 ID / URL
    match crate::api::FanqieClient::parse_book_id(&query) {
        Ok(book_id) => {
            // 输入的是书籍 ID 或 URL，直接获取书籍信息
            let result = state.client.get_book_as_search_result(&book_id).await?;
            Ok(vec![result])
        }
        Err(_) => {
            // 关键词搜索：通过必应搜索 site:fanqienovel.com/page 获取书籍 ID
            state.client.search_novels(&query, limit).await
        }
    }
}

/// 解析书籍输入（ID 或 URL），返回书籍 ID
#[tauri::command]
pub fn parse_book_input(input: String) -> AppResult<String> {
    crate::api::FanqieClient::parse_book_id(&input)
}

/// 获取书籍详情
#[tauri::command]
pub async fn get_book_detail(
    book_id: String,
    state: State<'_, AppState>,
) -> AppResult<BookDetail> {
    state.client.get_book_detail(&book_id).await
}

/// 获取章节目录
#[tauri::command]
pub async fn get_chapter_list(
    book_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<ChapterItem>> {
    state.client.get_chapter_list(&book_id).await
}

/// 获取章节内容
#[tauri::command]
pub async fn get_chapter_content(
    item_id: String,
    state: State<'_, AppState>,
) -> AppResult<ChapterContent> {
    state.client.get_chapter_content(&item_id).await
}

// ==================== 书架命令 ====================

/// 获取书架列表
#[tauri::command]
pub fn get_bookshelf(state: State<'_, AppState>) -> AppResult<Vec<BookshelfItem>> {
    state.db.get_bookshelf()
}

/// 检查是否在书架中
#[tauri::command]
pub fn is_in_bookshelf(
    book_id: String,
    state: State<'_, AppState>,
) -> AppResult<bool> {
    state.db.is_in_bookshelf(&book_id)
}

/// 添加到书架
#[tauri::command]
pub fn add_to_bookshelf(
    item: BookshelfItem,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state.db.add_to_bookshelf(&item)
}

/// 从书架移除
#[tauri::command]
pub fn remove_from_bookshelf(
    book_id: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state.db.remove_from_bookshelf(&book_id)
}

/// 更新书架阅读进度
#[tauri::command]
pub fn update_bookshelf_progress(
    book_id: String,
    chapter_index: i64,
    chapter_title: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state
        .db
        .update_bookshelf_progress(&book_id, chapter_index, &chapter_title)
}

// ==================== 阅读进度命令 ====================

/// 保存阅读进度
#[tauri::command]
pub fn save_reading_progress(
    progress: ReadingProgress,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state.db.save_reading_progress(&progress)
}

/// 获取阅读进度
#[tauri::command]
pub fn get_reading_progress(
    book_id: String,
    state: State<'_, AppState>,
) -> AppResult<Option<ReadingProgress>> {
    state.db.get_reading_progress(&book_id)
}

// ==================== 下载命令 ====================

/// 下载小说
///
/// `format`: "txt" 或 "epub"
/// `output_dir`: 输出目录，为空则使用默认下载目录
#[tauri::command]
pub async fn download_novel(
    book_id: String,
    format: String,
    output_dir: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> AppResult<DownloadHistory> {
    let fmt = match format.to_lowercase().as_str() {
        "txt" => DownloadFormat::Txt,
        "epub" => DownloadFormat::Epub,
        _ => {
            return Err(AppError::InvalidParam(format!(
                "不支持的格式: {}",
                format
            )))
        }
    };

    // 确定输出目录
    let output_dir = match output_dir {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => {
            let download_dir = app
                .path()
                .download_dir()
                .map_err(|e| AppError::Other(anyhow::anyhow!("获取下载目录失败: {}", e)))?;
            download_dir.join("番茄小说")
        }
    };

    let downloader = Downloader::new(state.client.clone());
    let app_handle = app.clone();
    let book_id_for_progress = book_id.clone();

    let result = downloader
        .download(&book_id, fmt, &output_dir, move |progress| {
            let _ = app_handle.emit("download-progress", &progress);
        })
        .await;

    // 获取书籍详情用于记录历史
    let detail = state.client.get_book_detail(&book_id).await.ok();

    match result {
        Ok(download_result) => {
            let history = DownloadHistory {
                id: uuid::Uuid::new_v4().to_string(),
                book_id: book_id.clone(),
                book_name: detail
                    .as_ref()
                    .map(|d| d.book_name.clone())
                    .unwrap_or_default(),
                author: detail
                    .as_ref()
                    .map(|d| d.author.clone())
                    .unwrap_or_default(),
                format: format.clone(),
                file_path: download_result.file_path.to_string_lossy().into_owned(),
                file_size: download_result.file_size as i64,
                status: 1,
                downloaded_at: chrono::Utc::now().timestamp(),
                chapter_count: download_result.chapter_count as i64,
            };

            state.db.add_download_history(&history)?;

            // 更新书架中的最新章节信息
            if let Some(d) = &detail {
                let _ = state.db.update_bookshelf_info(
                    &book_id,
                    &d.last_chapter_title,
                    &d.last_chapter_id,
                );
            }

            Ok(history)
        }
        Err(e) => {
            // 记录失败的下载
            let history = DownloadHistory {
                id: uuid::Uuid::new_v4().to_string(),
                book_id: book_id.clone(),
                book_name: detail
                    .as_ref()
                    .map(|d| d.book_name.clone())
                    .unwrap_or_default(),
                author: detail
                    .as_ref()
                    .map(|d| d.author.clone())
                    .unwrap_or_default(),
                format: format.clone(),
                file_path: String::new(),
                file_size: 0,
                status: 2,
                downloaded_at: chrono::Utc::now().timestamp(),
                chapter_count: 0,
            };
            let _ = state.db.add_download_history(&history);

            // 发送失败事件
            let _ = app.emit(
                "download-progress",
                &DownloadProgress {
                    book_id: book_id_for_progress,
                    book_name: history.book_name,
                    format,
                    current: 0,
                    total: 0,
                    status: "failed".to_string(),
                    message: e.to_string(),
                },
            );

            Err(e)
        }
    }
}

/// 下载小说指定章节
///
/// `chapters`: 选中的章节列表(ChapterItem 数组)
/// `format`: "txt" 或 "epub"
#[tauri::command]
pub async fn download_novel_chapters(
    book_id: String,
    chapters: Vec<ChapterItem>,
    format: String,
    output_dir: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> AppResult<DownloadHistory> {
    if chapters.is_empty() {
        return Err(AppError::InvalidParam("未选择任何章节".to_string()));
    }

    let fmt = match format.to_lowercase().as_str() {
        "txt" => DownloadFormat::Txt,
        "epub" => DownloadFormat::Epub,
        _ => {
            return Err(AppError::InvalidParam(format!(
                "不支持的格式: {}",
                format
            )))
        }
    };

    // 确定输出目录
    let output_dir = match output_dir {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => {
            let download_dir = app
                .path()
                .download_dir()
                .map_err(|e| AppError::Other(anyhow::anyhow!("获取下载目录失败: {}", e)))?;
            download_dir.join("番茄小说")
        }
    };

    let downloader = Downloader::new(state.client.clone());
    let app_handle = app.clone();
    let book_id_for_progress = book_id.clone();
    let selected_count = chapters.len();

    // 构造全量 item_ids 占位(下载器内部不再使用)
    let all_item_ids: Vec<String> =
        chapters.iter().map(|c| c.item_id.clone()).collect();

    let result = downloader
        .download_chapters(
            &book_id,
            fmt,
            &output_dir,
            &chapters,
            &all_item_ids,
            move |progress| {
                let _ = app_handle.emit("download-progress", &progress);
            },
        )
        .await;

    // 获取书籍详情用于记录历史
    let detail = state.client.get_book_detail(&book_id).await.ok();

    match result {
        Ok(download_result) => {
            let history = DownloadHistory {
                id: uuid::Uuid::new_v4().to_string(),
                book_id: book_id.clone(),
                book_name: detail
                    .as_ref()
                    .map(|d| d.book_name.clone())
                    .unwrap_or_default(),
                author: detail
                    .as_ref()
                    .map(|d| d.author.clone())
                    .unwrap_or_default(),
                format: format.clone(),
                file_path: download_result.file_path.to_string_lossy().into_owned(),
                file_size: download_result.file_size as i64,
                status: 1,
                downloaded_at: chrono::Utc::now().timestamp(),
                chapter_count: download_result.chapter_count as i64,
            };

            state.db.add_download_history(&history)?;

            if let Some(d) = &detail {
                let _ = state.db.update_bookshelf_info(
                    &book_id,
                    &d.last_chapter_title,
                    &d.last_chapter_id,
                );
            }

            Ok(history)
        }
        Err(e) => {
            let history = DownloadHistory {
                id: uuid::Uuid::new_v4().to_string(),
                book_id: book_id.clone(),
                book_name: detail
                    .as_ref()
                    .map(|d| d.book_name.clone())
                    .unwrap_or_default(),
                author: detail
                    .as_ref()
                    .map(|d| d.author.clone())
                    .unwrap_or_default(),
                format: format.clone(),
                file_path: String::new(),
                file_size: 0,
                status: 2,
                downloaded_at: chrono::Utc::now().timestamp(),
                chapter_count: selected_count as i64,
            };
            let _ = state.db.add_download_history(&history);

            let _ = app.emit(
                "download-progress",
                &DownloadProgress {
                    book_id: book_id_for_progress,
                    book_name: history.book_name,
                    format,
                    current: 0,
                    total: selected_count,
                    status: "failed".to_string(),
                    message: e.to_string(),
                },
            );

            Err(e)
        }
    }
}

/// 获取下载历史
#[tauri::command]
pub fn get_download_history(
    state: State<'_, AppState>,
) -> AppResult<Vec<DownloadHistory>> {
    state.db.get_download_history()
}

/// 删除下载历史记录
#[tauri::command]
pub fn delete_download_history(
    id: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state.db.delete_download_history(&id)
}

/// 清空下载历史
#[tauri::command]
pub fn clear_download_history(state: State<'_, AppState>) -> AppResult<()> {
    state.db.clear_download_history()
}

/// 在文件管理器中打开文件所在目录
#[tauri::command]
pub async fn open_in_folder(
    file_path: String,
    app: AppHandle,
) -> AppResult<()> {
    use tauri_plugin_shell::ShellExt;
    let path = PathBuf::from(&file_path);
    let dir = if path.is_file() {
        path.parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or(file_path)
    } else {
        file_path
    };

    app.shell()
        .open(dir, None)
        .map_err(|e| AppError::Other(anyhow::anyhow!("打开目录失败: {}", e)))?;

    Ok(())
}

// ==================== 配置命令 ====================

/// 获取应用数据目录
#[tauri::command]
pub fn get_app_data_dir(app: AppHandle) -> AppResult<String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Other(anyhow::anyhow!("获取数据目录失败: {}", e)))?;
    Ok(dir.to_string_lossy().into_owned())
}

/// 获取默认下载目录
#[tauri::command]
pub fn get_default_download_dir(app: AppHandle) -> AppResult<String> {
    let dir = app
        .path()
        .download_dir()
        .map_err(|e| AppError::Other(anyhow::anyhow!("获取下载目录失败: {}", e)))?;
    Ok(dir.join("番茄小说").to_string_lossy().into_owned())
}
