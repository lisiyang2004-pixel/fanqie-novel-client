use crate::error::{AppError, AppResult};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER, USER_AGENT};
use reqwest::Client;
use std::time::Duration;

/// 番茄小说 API 客户端（基于网页 HTML 解析）
///
/// 由于番茄小说 API 需要字节跳动签名(a_bogus/msToken)，
/// 本客户端改为请求 SSR 网页并解析 HTML 中的 `window.__INITIAL_STATE__` 数据。
#[derive(Clone)]
pub struct FanqieClient {
    pub client: Client,
}

impl Default for FanqieClient {
    fn default() -> Self {
        Self::new()
    }
}

impl FanqieClient {
    const BASE_URL: &'static str = "https://fanqienovel.com";
    const USER_AGENT: &'static str =
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
         (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(Self::USER_AGENT));
        headers.insert(REFERER, HeaderValue::from_static("https://fanqienovel.com/"));
        headers.insert(
            "Accept",
            HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
        );
        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
        );
        // 基础 cookie，避免被立即拦截
        headers.insert(
            COOKIE,
            HeaderValue::from_static("novel_web_id=7460000000000000000"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .build()
            .expect("failed to build reqwest client");

        Self { client }
    }

    /// 发送 GET 请求并返回文本
    pub async fn get_text(&self, url: &str) -> AppResult<String> {
        log::debug!("GET {}", url);
        let resp = self.client.get(url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(anyhow::anyhow!(
                "HTTP {} : {}",
                status.as_u16(),
                body
            )));
        }
        let text = resp.text().await?;
        Ok(text)
    }

    /// 书籍详情页 URL
    pub fn book_page_url(&self, book_id: &str) -> String {
        format!("{}/page/{}", Self::BASE_URL, book_id)
    }

    /// 章节阅读页 URL
    pub fn reader_url(&self, item_id: &str) -> String {
        format!("{}/reader/{}", Self::BASE_URL, item_id)
    }

    /// 从 HTML 中提取 `window.__INITIAL_STATE__` 的 JSON 内容
    ///
    /// 注意: 必须正确处理字符串内的 `{` `}` 和转义字符 `\"`，
    /// 否则会提前结束或解析失败。
    pub fn extract_initial_state(html: &str) -> AppResult<serde_json::Value> {
        let marker = "window.__INITIAL_STATE__=";
        let start = html
            .find(marker)
            .ok_or_else(|| AppError::Other(anyhow::anyhow!("未找到 __INITIAL_STATE__")))?
            + marker.len();
        let rest = &html[start..];

        // 番茄小说的 __INITIAL_STATE__ 后面跟的是 JSON 对象
        // 找到第一个 '{' 开始位置
        let bytes = rest.as_bytes();
        let mut i = 0;
        while i < bytes.len() && bytes[i] != b'{' {
            i += 1;
        }
        if i >= bytes.len() {
            return Err(AppError::Other(anyhow::anyhow!(
                "__INITIAL_STATE__ 后未找到 '{{'"
            )));
        }

        // 使用状态机正确匹配括号，跳过字符串内的括号
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escape = false;
        let mut end = 0;
        let mut j = i;
        while j < bytes.len() {
            let b = bytes[j];
            if in_string {
                if escape {
                    escape = false;
                } else if b == b'\\' {
                    escape = true;
                } else if b == b'"' {
                    in_string = false;
                }
            } else {
                match b {
                    b'"' => in_string = true,
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = j + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            j += 1;
        }

        if end == 0 {
            return Err(AppError::Other(anyhow::anyhow!(
                "__INITIAL_STATE__ JSON 解析失败(括号未匹配)"
            )));
        }
        let json_str = &rest[i..end];

        // 番茄小说的 __INITIAL_STATE__ 是 JS 对象字面量，可能包含 undefined
        // (标准 JSON 不支持 undefined)，需要预处理为 null
        let json_str = replace_undefined(json_str);

        serde_json::from_str(&json_str).map_err(AppError::Json)
    }
}

/// 将 JSON 字符串中非字符串位置的 `undefined` 替换为 `null`
///
/// 注意: 必须跳过字符串内的 `undefined`，避免误替换。
/// 直接操作字节，避免破坏 UTF-8 编码。
fn replace_undefined(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut in_string = false;
    let mut escape = false;
    let mut i = 0;
    let n = bytes.len();

    while i < n {
        let b = bytes[i];
        if in_string {
            out.push(b);
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
        } else {
            if b == b'"' {
                in_string = true;
                out.push(b'"');
                i += 1;
            } else if i + 9 <= n && &bytes[i..i + 9] == b"undefined" {
                // 检查边界：undefined 前后应该不是字母数字/下划线
                let before_ok = i == 0
                    || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
                let after_idx = i + 9;
                let after_ok = after_idx >= n
                    || (!bytes[after_idx].is_ascii_alphanumeric() && bytes[after_idx] != b'_');
                if before_ok && after_ok {
                    out.extend_from_slice(b"null");
                    i += 9;
                } else {
                    out.push(b);
                    i += 1;
                }
            } else {
                out.push(b);
                i += 1;
            }
        }
    }

    // 安全转换回 String
    match String::from_utf8(out) {
        Ok(s) => s,
        // 极端情况下回退到原始字符串
        Err(_) => s.to_string(),
    }
}
