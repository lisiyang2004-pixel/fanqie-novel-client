use crate::error::AppResult;
use crate::models::{BookDetail, ChapterContent, ChapterItem};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::DownloadResult;

/// 生成 TXT 文件
pub fn generate_txt(
    detail: &BookDetail,
    chapters: &[ChapterItem],
    contents: &[ChapterContent],
    file_path: &Path,
) -> AppResult<DownloadResult> {
    let mut file = std::fs::File::create(file_path)?;

    // 写入文件头: BOM + 书籍信息
    writeln!(file, "\u{FEFF}")?; // UTF-8 BOM
    writeln!(file, "书名: {}", detail.book_name)?;
    writeln!(file, "作者: {}", detail.author)?;
    if !detail.category.is_empty() {
        writeln!(file, "分类: {}", detail.category)?;
    }
    if !detail.word_count.is_empty() {
        writeln!(file, "字数: {}", detail.word_count)?;
    }
    writeln!(
        file,
        "状态: {}",
        if detail.book_status == 1 {
            "已完结"
        } else {
            "连载中"
        }
    )?;
    if !detail.r#abstract.is_empty() {
        writeln!(file)?;
        writeln!(file, "简介")?;
        writeln!(file, "{}", detail.r#abstract)?;
    }
    writeln!(file)?;
    writeln!(file, "================================")?;
    writeln!(file)?;

    // 写入各章节
    for (i, chapter) in chapters.iter().enumerate() {
        let content = contents
            .get(i)
            .map(|c| c.content.as_str())
            .unwrap_or("[内容获取失败]");

        writeln!(file, "{}", chapter.title)?;
        writeln!(file)?;
        writeln!(file, "{}", content)?;
        writeln!(file)?;
        writeln!(file, "--------------------------------")?;
        writeln!(file)?;
    }

    let file_size = std::fs::metadata(file_path)?.len();

    Ok(DownloadResult {
        file_path: PathBuf::from(file_path),
        file_size,
        chapter_count: chapters.len(),
    })
}
