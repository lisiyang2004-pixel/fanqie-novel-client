use crate::error::{AppError, AppResult};
use crate::models::SearchBook;
use crate::api::client::FanqieClient;
use regex::Regex;
use std::collections::HashSet;

impl FanqieClient {
    /// 解析用户输入，提取书籍 ID
    ///
    /// 支持：
    /// - 纯数字 ID: `7143038691944959011`
    /// - 完整 URL: `https://fanqienovel.com/page/7143038691944959011`
    /// - 短路径: `/page/7143038691944959011`
    pub fn parse_book_id(input: &str) -> AppResult<String> {
        let input = input.trim();

        // 纯数字
        if input.chars().all(|c| c.is_ascii_digit()) && input.len() >= 5 {
            return Ok(input.to_string());
        }

        // URL 中提取
        let re = Regex::new(r"/page/(\d+)").unwrap();
        if let Some(caps) = re.captures(input) {
            return Ok(caps[1].to_string());
        }

        // reader URL 中的 item_id 也可以转换
        let re2 = Regex::new(r"/reader/(\d+)").unwrap();
        if let Some(caps) = re2.captures(input) {
            return Ok(caps[1].to_string());
        }

        Err(AppError::InvalidParam(format!(
            "无法解析书籍 ID。请输入纯数字 ID 或番茄小说书籍页面 URL。\n输入: {}",
            input
        )))
    }

    /// 通过必应搜索获取番茄小说书籍 ID 列表
    ///
    /// 番茄小说官方搜索 API 需要签名认证（a_bogus/msToken），
    /// 无法直接调用。这里通过必应搜索引擎搜索
    /// `site:fanqienovel.com/page 书名`，从搜索结果中提取书籍 ID，
    /// 再调用 get_book_detail 获取完整书籍信息。
    ///
    /// `query`: 搜索关键词（书名/作者）
    /// `limit`: 最多返回结果数
    pub async fn search_novels(&self, query: &str, limit: usize) -> AppResult<Vec<SearchBook>> {
        let book_ids = self.search_book_ids_via_bing(query, limit).await?;

        if book_ids.is_empty() {
            return Ok(Vec::new());
        }

        // 逐个获取书籍详情
        let mut results: Vec<SearchBook> = Vec::new();
        for book_id in &book_ids {
            match self.get_book_as_search_result(book_id).await {
                Ok(book) => {
                    results.push(book);
                }
                Err(e) => {
                    log::warn!("获取书籍详情失败 {}: {}", book_id, e);
                }
            }
            // 找到足够数量就停止
            if results.len() >= limit {
                break;
            }
        }

        Ok(results)
    }

    /// 通过必应搜索获取书籍 ID 列表
    ///
    /// 使用多种查询策略提高覆盖率：
    /// 1. `site:fanqienovel.com/page 书名`
    /// 2. `书名 番茄小说 fanqienovel.com`
    async fn search_book_ids_via_bing(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<String>> {
        let mut all_ids: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        // 多种查询策略
        let queries = vec![
            format!("site:fanqienovel.com/page {}", query),
            format!("{} 番茄小说 fanqienovel.com/page", query),
        ];

        for search_query in &queries {
            let encoded_query = urlencoding::encode(search_query).to_string();
            let url = format!(
                "https://www.bing.com/search?q={}&setlang=zh-CN&count=30",
                encoded_query
            );

            log::info!("Bing 搜索: {}", url);
            let html = match self.get_text(&url).await {
                Ok(html) => html,
                Err(e) => {
                    log::warn!("Bing 搜索失败: {}", e);
                    continue;
                }
            };

            // 从 HTML 中提取 fanqienovel.com/page/{id} 的书籍 ID
            let re = Regex::new(r"fanqienovel\.com/page/(\d{15,25})").unwrap();
            for caps in re.captures_iter(&html) {
                let id = caps[1].to_string();
                if seen.insert(id.clone()) {
                    all_ids.push(id);
                    if all_ids.len() >= limit * 2 {
                        break;
                    }
                }
            }

            if all_ids.len() >= limit {
                break;
            }
        }

        log::info!("Bing 搜索到 {} 个书籍 ID", all_ids.len());
        Ok(all_ids)
    }

    /// 通过书籍 ID 获取搜索结果（用于统一接口）
    pub async fn get_book_as_search_result(&self, book_id: &str) -> AppResult<SearchBook> {
        let detail = self.get_book_detail(book_id).await?;
        Ok(SearchBook {
            book_id: detail.book_id,
            book_name: detail.book_name,
            author: detail.author,
            cover: detail.cover,
            r#abstract: detail.r#abstract,
            category: detail.category,
            word_count: detail.word_count,
            book_status: detail.book_status,
            last_chapter_title: detail.last_chapter_title,
            last_chapter_id: detail.last_chapter_id,
        })
    }
}