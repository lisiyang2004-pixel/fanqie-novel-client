use crate::error::{AppError, AppResult};
use crate::models::{BookDetail, ChapterItem};
use crate::api::client::FanqieClient;

impl FanqieClient {
    /// 获取书籍详情（通过解析详情页 HTML 的 __INITIAL_STATE__）
    ///
    /// 优先使用 /page/{book_id}，失败时回退到 /reader/{book_id}
    pub async fn get_book_detail(&self, book_id: &str) -> AppResult<BookDetail> {
        let html = self.get_book_html(book_id).await?;
        let state = Self::extract_initial_state(&html)?;

        // /reader/ 页面的数据在 state.reader.chapterData 中，bookId 可能与传入的不同
        // 优先用 state.page，如果为空则尝试用 state.reader.chapterData
        let page = if let Some(p) = state.get("page") {
            let p_book_id = p.get("bookId").and_then(|v| v.as_str()).unwrap_or("");
            if p_book_id.is_empty() {
                // /reader/ 页面，从 chapterData 提取
                Self::reader_page_to_state(&state)?
            } else {
                p.clone()
            }
        } else {
            return Err(AppError::Other(anyhow::anyhow!("未找到 page 数据")));
        };

        let book_id = page
            .get("bookId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if book_id.is_empty() {
            return Err(AppError::BookNotFound(book_id.to_string()));
        }

        let book_name = page
            .get("bookName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let author = page
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let r#abstract = page
            .get("abstract")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 分类: page.category 通常为空，从 categoryV2(JSON 字符串) 中提取主分类名
        let category = page
            .get("category")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| extract_main_category(&page));

        // 字数: wordNumber 是数字
        let word_number = page
            .get("wordNumber")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // 连载状态: status=1 已完结, status=0 连载中
        let book_status = page
            .get("status")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        // 封面图: thumbUri 已是完整 URL
        let cover = page
            .get("thumbUri")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let last_chapter_id = page
            .get("lastChapterItemId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let last_chapter_title = page
            .get("lastChapterTitle")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 章节总数: 没有 chapterTotal 字段，从 chapterListWithVolume 计算
        let chapter_count = count_chapters(&page);

        // 最新章节时间: lastPublishTime 是字符串时间戳(秒)
        let last_chapter_time = page
            .get("lastPublishTime")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        Ok(BookDetail {
            book_id,
            book_name,
            author,
            cover,
            r#abstract,
            category,
            word_count: format!("{}", word_number),
            book_status,
            last_chapter_title,
            last_chapter_id,
            chapter_count,
            last_chapter_time,
        })
    }

    /// 获取章节列表（从详情页 HTML 的 __INITIAL_STATE__ 提取）
    pub async fn get_chapter_list(&self, book_id: &str) -> AppResult<Vec<ChapterItem>> {
        let html = self.get_book_html(book_id).await?;
        let state = Self::extract_initial_state(&html)?;

        let page = if let Some(p) = state.get("page") {
            let p_book_id = p.get("bookId").and_then(|v| v.as_str()).unwrap_or("");
            if p_book_id.is_empty() {
                Self::reader_page_to_state(&state)?
            } else {
                p.clone()
            }
        } else {
            return Err(AppError::Other(anyhow::anyhow!("未找到 page 数据")));
        };

        // chapterListWithVolume 是二维数组: [[{...}, {...}], [{...}]]
        // 每个一级元素是一卷的章节列表
        let mut chapters: Vec<ChapterItem> = Vec::new();
        let mut idx: i64 = 0;

        if let Some(volumes) = page.get("chapterListWithVolume").and_then(|v| v.as_array()) {
            for volume in volumes {
                // 每一卷是一个数组
                if let Some(volume_chapters) = volume.as_array() {
                    for ch in volume_chapters {
                        if let Some(item) = parse_chapter(ch, idx) {
                            chapters.push(item);
                            idx += 1;
                        }
                    }
                }
            }
        }

        // 如果分卷列表为空，尝试从扁平的 chapterList 提取
        if chapters.is_empty() {
            if let Some(ch_list) = page.get("chapterList").and_then(|v| v.as_array()) {
                for ch in ch_list {
                    if let Some(item) = parse_chapter(ch, idx) {
                        chapters.push(item);
                        idx += 1;
                    }
                }
            }
        }

        if chapters.is_empty() {
            // 最后尝试从 itemIds 提取（只有 ID 没有标题）
            if let Some(item_ids) = page.get("itemIds").and_then(|v| v.as_array()) {
                for (i, id) in item_ids.iter().enumerate() {
                    if let Some(id_str) = id.as_str() {
                        chapters.push(ChapterItem {
                            item_id: id_str.to_string(),
                            title: format!("第{}章", i + 1),
                            index: i as i64,
                            is_vip: false,
                        });
                    }
                }
            }
        }

        Ok(chapters)
    }
}

/// 计算章节总数
fn count_chapters(page: &serde_json::Value) -> i64 {
    let mut total = 0i64;
    if let Some(volumes) = page.get("chapterListWithVolume").and_then(|v| v.as_array()) {
        for volume in volumes {
            if let Some(volume_chapters) = volume.as_array() {
                total += volume_chapters.len() as i64;
            }
        }
    }
    if total == 0 {
        if let Some(ch_list) = page.get("chapterList").and_then(|v| v.as_array()) {
            total = ch_list.len() as i64;
        }
    }
    if total == 0 {
        if let Some(item_ids) = page.get("itemIds").and_then(|v| v.as_array()) {
            total = item_ids.len() as i64;
        }
    }
    total
}

/// 从 categoryV2 JSON 字符串中提取主分类名
fn extract_main_category(page: &serde_json::Value) -> String {
    let v2 = page.get("categoryV2").and_then(|v| v.as_str());
    if let Some(json_str) = v2 {
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
            // 优先取 MainCategory=true 的项
            for item in &arr {
                if item
                    .get("MainCategory")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    if let Some(name) = item.get("Name").and_then(|v| v.as_str()) {
                        return name.to_string();
                    }
                }
            }
            // 否则取第一个
            if let Some(first) = arr.first() {
                if let Some(name) = first.get("Name").and_then(|v| v.as_str()) {
                    return name.to_string();
                }
            }
        }
    }
    String::new()
}

/// 从 JSON 对象解析章节信息
fn parse_chapter(ch: &serde_json::Value, idx: i64) -> Option<ChapterItem> {
    let item_id = ch
        .get("itemId")
        .and_then(|v| v.as_str())
        .or_else(|| ch.get("item_id").and_then(|v| v.as_str()))?
        .to_string();
    let title = ch
        .get("title")
        .and_then(|v| v.as_str())
        .or_else(|| ch.get("chapterTitle").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();
    // needPay=1 表示付费/VIP 章节
    let need_pay = ch
        .get("needPay")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let is_vip = need_pay != 0;

    Some(ChapterItem {
        item_id,
        title,
        index: idx,
        is_vip,
    })
}


impl FanqieClient {
    /// 将 /reader/ 页面的 state.reader.chapterData 转换为类似 state.page 的结构
    ///
    /// /reader/ 页面结构:
    ///   state.reader.chapterData = {
    ///     bookId, bookName, author, abstract, category, wordNumber,
    ///     status, thumbUri, chapterListWithVolume, ...
    ///   }
    fn reader_page_to_state(state: &serde_json::Value) -> AppResult<serde_json::Value> {
        let chapter_data = state
            .get("reader")
            .and_then(|v| v.get("chapterData"))
            .ok_or_else(|| AppError::Other(anyhow::anyhow!(
                "/reader/ 页面未找到 chapterData 数据"
            )))?;
        Ok(chapter_data.clone())
    }
}
