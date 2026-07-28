pub mod txt;
pub mod epub;

use crate::api::FanqieClient;
use crate::error::{AppError, AppResult};
use crate::models::{BookDetail, ChapterContent, ChapterItem, DownloadProgress};
use std::path::{Path, PathBuf};

/// 下载格式
#[derive(Debug, Clone, Copy)]
pub enum DownloadFormat {
    Txt,
    Epub,
}

impl DownloadFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            DownloadFormat::Txt => "txt",
            DownloadFormat::Epub => "epub",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadFormat::Txt => "txt",
            DownloadFormat::Epub => "epub",
        }
    }
}

/// 下载结果
#[derive(Debug)]
pub struct DownloadResult {
    pub file_path: PathBuf,
    pub file_size: u64,
    pub chapter_count: usize,
}

/// 下载器
pub struct Downloader {
    pub client: FanqieClient,
}

impl Downloader {
    pub fn new(client: FanqieClient) -> Self {
        Self { client }
    }

    /// 下载小说(全部章节)
    ///
    /// `book_id` 书籍 ID
    /// `format` 下载格式
    /// `output_dir` 输出目录
    /// `on_progress` 进度回调
    pub async fn download<F>(
        &self,
        book_id: &str,
        format: DownloadFormat,
        output_dir: &Path,
        on_progress: F,
    ) -> AppResult<DownloadResult>
    where
        F: Fn(DownloadProgress) + Clone + Send + Sync + 'static,
    {
        // 获取全部分类后调用 download_chapters
        let chapters = self.client.get_chapter_list(book_id).await?;
        let item_ids: Vec<String> =
            chapters.iter().map(|c| c.item_id.clone()).collect();
        self.download_chapters(book_id, format, output_dir, &chapters, &item_ids, on_progress)
            .await
    }

    /// 下载小说指定章节
    ///
    /// `book_id` 书籍 ID
    /// `format` 下载格式
    /// `output_dir` 输出目录
    /// `selected_chapters` 选中的章节列表(目录中所有章节的子集)
    /// `on_progress` 进度回调
    pub async fn download_chapters<F>(
        &self,
        book_id: &str,
        format: DownloadFormat,
        output_dir: &Path,
        selected_chapters: &[ChapterItem],
        _all_item_ids: &[String],
        on_progress: F,
    ) -> AppResult<DownloadResult>
    where
        F: Fn(DownloadProgress) + Clone + Send + Sync + 'static,
    {
        if selected_chapters.is_empty() {
            return Err(AppError::InvalidParam("未选择任何章节".to_string()));
        }

        // 1. 获取书籍详情
        on_progress(DownloadProgress {
            book_id: book_id.to_string(),
            book_name: String::new(),
            format: format.as_str().to_string(),
            current: 0,
            total: 0,
            status: "fetching_detail".to_string(),
            message: "正在获取书籍信息...".to_string(),
        });

        let detail = self.client.get_book_detail(book_id).await?;

        // 2. 提取选中章节的 item_id
        let item_ids: Vec<String> = selected_chapters
            .iter()
            .map(|c| c.item_id.clone())
            .collect();

        on_progress(DownloadProgress {
            book_id: book_id.to_string(),
            book_name: detail.book_name.clone(),
            format: format.as_str().to_string(),
            current: 0,
            total: item_ids.len(),
            status: "fetching_chapters".to_string(),
            message: format!("共选中 {} 章，开始下载...", item_ids.len()),
        });

        // 3. 下载选中章节内容
        let progress_clone = on_progress.clone();
        let book_name_clone = detail.book_name.clone();
        let format_str = format.as_str().to_string();
        let book_id_clone = book_id.to_string();

        let contents = self
            .client
            .get_chapters_batch(&item_ids, move |current, total| {
                progress_clone(DownloadProgress {
                    book_id: book_id_clone.clone(),
                    book_name: book_name_clone.clone(),
                    format: format_str.clone(),
                    current,
                    total,
                    status: "downloading".to_string(),
                    message: format!("下载中 {}/{}", current, total),
                });
            })
            .await?;

        // 4. 生成文件
        on_progress(DownloadProgress {
            book_id: book_id.to_string(),
            book_name: detail.book_name.clone(),
            format: format.as_str().to_string(),
            current: item_ids.len(),
            total: item_ids.len(),
            status: "generating".to_string(),
            message: "正在生成文件...".to_string(),
        });

        std::fs::create_dir_all(output_dir)?;
        let safe_name = sanitize_filename(&detail.book_name);
        let filename = format!("{}.{}", safe_name, format.extension());
        let file_path = output_dir.join(&filename);

        let result = match format {
            DownloadFormat::Txt => {
                txt::generate_txt(&detail, selected_chapters, &contents, &file_path)?
            }
            DownloadFormat::Epub => {
                epub::generate_epub(&detail, selected_chapters, &contents, &file_path)?
            }
        };

        on_progress(DownloadProgress {
            book_id: book_id.to_string(),
            book_name: detail.book_name.clone(),
            format: format.as_str().to_string(),
            current: item_ids.len(),
            total: item_ids.len(),
            status: "completed".to_string(),
            message: format!("下载完成: {}", file_path.display()),
        });

        Ok(result)
    }
}

/// 清理文件名中的非法字符
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}
