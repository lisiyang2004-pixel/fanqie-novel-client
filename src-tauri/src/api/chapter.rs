use crate::error::{AppError, AppResult};
use crate::models::ChapterContent;
use crate::api::client::FanqieClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use regex::Regex;
use once_cell::sync::Lazy;

impl FanqieClient {
    /// 获取单个章节内容（通过解析阅读页 HTML 的 __INITIAL_STATE__）
    pub async fn get_chapter_content(&self, item_id: &str) -> AppResult<ChapterContent> {
        let url = self.reader_url(item_id);
        let html = self.get_text(&url).await?;

        // 注意：不能用 html.contains("404") 判断，正常页面的脚本/JSON 中
        // 也常包含 "404" 字符串，会误判所有章节为 404。
        // HTTP 状态码已由 get_text 校验；这里只检测明确的无 __INITIAL_STATE__ 的情况。
        if !html.contains("__INITIAL_STATE__") {
            return Err(AppError::BookNotFound(format!(
                "章节 {} 不存在或被拦截（页面无 __INITIAL_STATE__）",
                item_id
            )));
        }

        let state = Self::extract_initial_state(&html)?;
        let reader = state
            .get("reader")
            .ok_or_else(|| AppError::Other(anyhow::anyhow!("未找到 reader 数据")))?;

        let chapter_data = reader
            .get("chapterData")
            .ok_or_else(|| AppError::Other(anyhow::anyhow!("未找到 chapterData")))?;

        let item_id = chapter_data
            .get("itemId")
            .and_then(|v| v.as_str())
            .unwrap_or(item_id)
            .to_string();
        let title = chapter_data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // preItemId / nextItemId 可能为空字符串
        let prev_item_id = chapter_data
            .get("preItemId")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let next_item_id = chapter_data
            .get("nextItemId")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // content 是 HTML 字符串，需要清理为纯文本
        let raw_content = chapter_data
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 调试日志：打印原始内容前 300 字，便于诊断反爬机制
        let preview: String = raw_content.chars().take(300).collect();
        log::info!(
            "章节 {} 原始内容前 300 字: {}",
            item_id,
            preview
        );
        log::info!(
            "章节 {} 原始内容总长度: {} 字节",
            item_id,
            raw_content.len()
        );

        let content = clean_html_content(raw_content);

        log::info!(
            "章节 {} 清理后内容前 200 字: {}",
            item_id,
            content.chars().take(200).collect::<String>()
        );

        if content.trim().is_empty() {
            return Err(AppError::Other(anyhow::anyhow!(
                "章节内容为空（可能是付费章节）: {}",
                title
            )));
        }

        Ok(ChapterContent {
            item_id,
            title,
            content,
            prev_item_id,
            next_item_id,
        })
    }

    /// 批量获取章节内容
    ///
    /// `item_ids`: 章节 ID 列表
    /// `on_progress`: 进度回调 (current, total)
    pub async fn get_chapters_batch<F>(
        &self,
        item_ids: &[String],
        on_progress: F,
    ) -> AppResult<Vec<ChapterContent>>
    where
        F: Fn(usize, usize) + Clone + Send + Sync + 'static,
    {
        let total = item_ids.len();
        let results: Vec<ChapterContent> = Vec::new();
        let results = Arc::new(Mutex::new(results));
        let failed = Arc::new(Mutex::new(Vec::<String>::new()));

        // 串行下载，避免触发反爬
        for (idx, item_id) in item_ids.iter().enumerate() {
            on_progress(idx, total);

            match self.get_chapter_content(item_id).await {
                Ok(content) => {
                    results.lock().await.push(content);
                }
                Err(e) => {
                    log::warn!("章节 {} 下载失败: {}", item_id, e);
                    failed.lock().await.push(item_id.clone());
                }
            }

            // 请求间隔，避免被封
            sleep(Duration::from_millis(300)).await;
        }

        on_progress(total, total);

        let results = Arc::try_unwrap(results).unwrap().into_inner();
        let failed = Arc::try_unwrap(failed).unwrap().into_inner();

        if results.is_empty() && !failed.is_empty() {
            return Err(AppError::Other(anyhow::anyhow!(
                "所有章节下载失败"
            )));
        }

        if !failed.is_empty() {
            log::warn!("{} 章下载失败: {:?}", failed.len(), failed);
        }

        Ok(results)
    }
}

// ============== PUA 字符映射表 ==============
// 番茄小说的反爬机制：把某些汉字替换为 Unicode 私用区 (PUA) 字符 U+E000-U+F8FF，
// 再用自定义字体渲染。纯文本提取时这些字符不可见，导致内容"丢字"。
// 本映射表通过多章明文-密文对齐提取（第3章完全匹配 + 第1章部分匹配），共 240 个。
// 字体文件名固定为 dc027189e0ba4cd，映射表也固定。
static PUA_MAP: Lazy<std::collections::HashMap<u32, char>> = Lazy::new(|| {
    let mut m = std::collections::HashMap::new();
    m.insert(0xE3E9, '在'); m.insert(0xE3EC, '家'); m.insert(0xE3EE, '然');
    m.insert(0xE3EF, '表'); m.insert(0xE3F0, '场'); m.insert(0xE3F2, '要');
    m.insert(0xE3F3, '只'); m.insert(0xE3F5, '和'); m.insert(0xE3F8, '别');
    m.insert(0xE3F9, '还'); m.insert(0xE3FB, '现'); m.insert(0xE3FC, '儿');
    m.insert(0xE400, '此'); m.insert(0xE401, '象'); m.insert(0xE404, '出');
    m.insert(0xE406, '工'); m.insert(0xE407, '相'); m.insert(0xE409, '男');
    m.insert(0xE40A, '直'); m.insert(0xE40E, '都'); m.insert(0xE40F, '平');
    m.insert(0xE410, '文'); m.insert(0xE411, '什'); m.insert(0xE414, '将');
    m.insert(0xE415, '真'); m.insert(0xE416, 't'); m.insert(0xE417, '那');
    m.insert(0xE41A, '会'); m.insert(0xE41B, '立'); m.insert(0xE41C, '些');
    m.insert(0xE41E, '是'); m.insert(0xE41F, '十'); m.insert(0xE420, '张');
    m.insert(0xE422, '气'); m.insert(0xE423, '大'); m.insert(0xE425, '两');
    m.insert(0xE426, '命'); m.insert(0xE427, '全'); m.insert(0xE428, '后');
    m.insert(0xE429, '东'); m.insert(0xE42A, '性'); m.insert(0xE42B, '通');
    m.insert(0xE42C, '被'); m.insert(0xE430, '接'); m.insert(0xE431, '而');
    m.insert(0xE432, '感'); m.insert(0xE433, '车'); m.insert(0xE434, '山');
    m.insert(0xE436, '了'); m.insert(0xE437, '常'); m.insert(0xE438, '以');
    m.insert(0xE439, '何'); m.insert(0xE43A, '可'); m.insert(0xE43B, '话');
    m.insert(0xE43C, '先'); m.insert(0xE43F, '叫'); m.insert(0xE440, '轻');
    m.insert(0xE444, '着'); m.insert(0xE445, '变'); m.insert(0xE449, '个');
    m.insert(0xE44A, '说'); m.insert(0xE44B, '少'); m.insert(0xE44C, '色');
    m.insert(0xE44D, '里'); m.insert(0xE44E, '安'); m.insert(0xE44F, '花');
    m.insert(0xE450, '远'); m.insert(0xE452, '难'); m.insert(0xE454, '放');
    m.insert(0xE457, '认'); m.insert(0xE458, '面'); m.insert(0xE459, '道');
    m.insert(0xE45D, '地'); m.insert(0xE45E, '度'); m.insert(0xE460, '好');
    m.insert(0xE461, '机'); m.insert(0xE465, '把'); m.insert(0xE467, '同');
    m.insert(0xE468, '水'); m.insert(0xE46A, '没'); m.insert(0xE46C, '电');
    m.insert(0xE46E, '像'); m.insert(0xE471, '为'); m.insert(0xE473, '白');
    m.insert(0xE474, '几'); m.insert(0xE477, '看'); m.insert(0xE478, '但');
    m.insert(0xE479, '第'); m.insert(0xE47A, '加'); m.insert(0xE47B, '候');
    m.insert(0xE47C, '作'); m.insert(0xE47D, '上'); m.insert(0xE47F, '住');
    m.insert(0xE480, '有'); m.insert(0xE481, '法'); m.insert(0xE483, '事');
    m.insert(0xE484, '应'); m.insert(0xE485, '位'); m.insert(0xE486, '利');
    m.insert(0xE487, '你'); m.insert(0xE488, '声'); m.insert(0xE489, '身');
    m.insert(0xE48B, '问'); m.insert(0xE48D, '女'); m.insert(0xE48E, '他');
    m.insert(0xE490, '比'); m.insert(0xE498, '边'); m.insert(0xE49A, '对');
    m.insert(0xE49B, '所'); m.insert(0xE49D, '活'); m.insert(0xE49E, '回');
    m.insert(0xE49F, '意'); m.insert(0xE4A0, '到'); m.insert(0xE4A2, '从');
    m.insert(0xE4A4, '知'); m.insert(0xE4A5, '又'); m.insert(0xE4A6, '内');
    m.insert(0xE4A8, '点'); m.insert(0xE4AA, '三'); m.insert(0xE4AF, '正');
    m.insert(0xE4B1, '夫'); m.insert(0xE4B2, '向'); m.insert(0xE4B4, '听');
    m.insert(0xE4B5, '更'); m.insert(0xE4B7, '得'); m.insert(0xE4B8, '告');
    m.insert(0xE4B9, '并'); m.insert(0xE4BA, '本'); m.insert(0xE4BC, '过');
    m.insert(0xE4BD, '记'); m.insert(0xE4BF, '让'); m.insert(0xE4C0, '打');
    m.insert(0xE4C2, '人'); m.insert(0xE4C3, '就'); m.insert(0xE4C4, '者');
    m.insert(0xE4C5, '去'); m.insert(0xE4C8, '体'); m.insert(0xE4C9, '做');
    m.insert(0xE4CA, '经'); m.insert(0xE4CC, '走'); m.insert(0xE4CD, '如');
    m.insert(0xE4CE, '孩'); m.insert(0xE4D1, '给'); m.insert(0xE4D2, '使');
    m.insert(0xE4D5, '最'); m.insert(0xE4DA, '等'); m.insert(0xE4DD, '行');
    m.insert(0xE4DE, '一'); m.insert(0xE4DF, '条'); m.insert(0xE4E0, '果');
    m.insert(0xE4E1, '动'); m.insert(0xE4E2, '光'); m.insert(0xE4E3, '门');
    m.insert(0xE4E4, '头'); m.insert(0xE4E5, '见'); m.insert(0xE4E7, '自');
    m.insert(0xE4E9, '成'); m.insert(0xE4EA, '处'); m.insert(0xE4EB, '天');
    m.insert(0xE4EC, '能'); m.insert(0xE4ED, '于'); m.insert(0xE4EE, '名');
    m.insert(0xE4EF, '其'); m.insert(0xE4F0, '发'); m.insert(0xE4F1, '总');
    m.insert(0xE4F3, '的'); m.insert(0xE4F4, '死'); m.insert(0xE4F5, '手');
    m.insert(0xE4F6, '入'); m.insert(0xE4F7, '路'); m.insert(0xE4F8, '进');
    m.insert(0xE4F9, '心'); m.insert(0xE4FA, '来'); m.insert(0xE4FC, '时');
    m.insert(0xE4FD, '力'); m.insert(0xE4FE, '多'); m.insert(0xE4FF, '开');
    m.insert(0xE500, '已'); m.insert(0xE501, '许'); m.insert(0xE503, '至');
    m.insert(0xE505, '很'); m.insert(0xE508, '小'); m.insert(0xE509, '与');
    m.insert(0xE50B, '想'); m.insert(0xE50D, '么'); m.insert(0xE50E, '分');
    m.insert(0xE50F, '生'); m.insert(0xE510, '口'); m.insert(0xE511, '再');
    m.insert(0xE514, '次'); m.insert(0xE515, '西'); m.insert(0xE517, '种');
    m.insert(0xE518, '带'); m.insert(0xE51B, '实'); m.insert(0xE51C, '情');
    m.insert(0xE51D, '才'); m.insert(0xE51E, '这'); m.insert(0xE521, '我');
    m.insert(0xE522, '神'); m.insert(0xE523, '格'); m.insert(0xE524, '长');
    m.insert(0xE525, '觉'); m.insert(0xE526, '间'); m.insert(0xE527, '年');
    m.insert(0xE528, '眼'); m.insert(0xE529, '无'); m.insert(0xE52A, '不');
    m.insert(0xE52D, '结'); m.insert(0xE52F, '友'); m.insert(0xE530, '信');
    m.insert(0xE531, '下'); m.insert(0xE532, '却'); m.insert(0xE533, '重');
    m.insert(0xE534, '己'); m.insert(0xE535, '老'); m.insert(0xE537, '音');
    m.insert(0xE538, '字'); m.insert(0xE53B, '明'); m.insert(0xE53C, '之');
    m.insert(0xE53D, '前'); m.insert(0xE53E, '高'); m.insert(0xE541, '目');
    m.insert(0xE542, '太'); m.insert(0xE545, '起'); m.insert(0xE547, '她');
    m.insert(0xE548, '也'); m.insert(0xE54A, '用'); m.insert(0xE54B, '方');
    m.insert(0xE54C, '子'); m.insert(0xE54E, '每'); m.insert(0xE54F, '理');
    m.insert(0xE550, '便'); m.insert(0xE551, '四'); m.insert(0xE552, '数');
    m.insert(0xE553, '期'); m.insert(0xE554, '中'); m.insert(0xE556, '外');
    m.insert(0xE557, '样'); m.insert(0xE55A, '们'); m.insert(0xE55B, '任');
    // ============== 以下为补充映射（字体解析 + 上下文推断）==============
    // 字体解析确认的映射（通过像素比对生成，排除部首类字符）
    m.insert(0xE3EA, '主'); m.insert(0xE3EB, '特'); m.insert(0xE3ED, '军');
    m.insert(0xE42E, '它'); m.insert(0xE42F, '乐'); m.insert(0xE442, '士');
    m.insert(0xE464, '写'); m.insert(0xE46B, '书'); m.insert(0xE48A, '国');
    m.insert(0xE494, '巷'); m.insert(0xE49C, '金'); m.insert(0xE4AB, '定');
    m.insert(0xE4B3, '德'); m.insert(0xE4D6, '笑'); m.insert(0xE506, '界');
    // 上下文推断的映射（通过章节内容上下文确认）
    m.insert(0xE402, '月'); m.insert(0xE40B, '失'); m.insert(0xE40C, '世');
    m.insert(0xE418, '当'); m.insert(0xE421, '学'); m.insert(0xE435, '公');
    m.insert(0xE447, '快'); m.insert(0xE453, '师'); m.insert(0xE456, '报');
    m.insert(0xE463, '民'); m.insert(0xE466, '岁'); m.insert(0xE469, '人');
    m.insert(0xE46D, '吃'); m.insert(0xE475, '日'); m.insert(0xE476, '卧');
    m.insert(0xE47E, '抱'); m.insert(0xE48C, '马'); m.insert(0xE491, '你');
    m.insert(0xE499, '美'); m.insert(0xE4A7, '认'); m.insert(0xE4D3, '商');
    m.insert(0xE4D7, '部'); m.insert(0xE4DB, '受'); m.insert(0xE4E8, '解');
    m.insert(0xE4F2, '你'); m.insert(0xE504, '觉'); m.insert(0xE50C, '代');
    m.insert(0xE512, '啊'); m.insert(0xE516, '风'); m.insert(0xE52B, '闻');
    m.insert(0xE52C, '关'); m.insert(0xE53A, '吧'); m.insert(0xE559, '海');
    // 第三批补充映射（基于章节上下文推断）
    m.insert(0xE405, '战'); // "下心头兢战"
    m.insert(0xE413, '号'); // "巷2号"
    m.insert(0xE4B0, '或'); // "或者其他情况"
    m.insert(0xE4C6, '原'); // "背后的原因太复杂"
    m.insert(0xE513, '望'); // "希望你一五一十"
    m
});

/// 还原 PUA 字符为对应汉字
///
/// 番茄小说把部分汉字替换为 Unicode 私用区 (PUA) 字符进行反爬，
/// 纯文本提取时这些字符不可见或显示为方块，导致内容"丢字"。
/// 本函数通过映射表把 PUA 字符还原为对应汉字。
fn restore_pua_chars(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    // 收集未映射的 PUA 码点（去重），最后统一输出
    let mut unmapped_seen: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut unmapped_entries: Vec<(u32, String)> = Vec::new();

    for (i, &c) in chars.iter().enumerate() {
        let cp = c as u32;
        if (0xE000..=0xF8FF).contains(&cp) {
            if let Some(&replacement) = PUA_MAP.get(&cp) {
                out.push(replacement);
            } else {
                // 收集上下文：前后各 4 个字符，已映射的显示为汉字，未映射的显示为 □
                let start = i.saturating_sub(4);
                let end = (i + 5).min(chars.len());
                let ctx: String = chars[start..end]
                    .iter()
                    .map(|&cc| {
                        let ccp = cc as u32;
                        if (0xE000..=0xF8FF).contains(&ccp) {
                            PUA_MAP.get(&ccp).copied().unwrap_or('□')
                        } else {
                            cc
                        }
                    })
                    .collect();
                if unmapped_seen.insert(cp) {
                    unmapped_entries.push((cp, ctx));
                }
                // 保留原字符，避免完全丢失
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }

    if !unmapped_entries.is_empty() {
        let mut codes: Vec<String> = unmapped_entries
            .iter()
            .map(|(cp, ctx)| format!("U+{:04X} 上下文[{}]", cp, ctx))
            .collect();
        codes.sort();
        log::warn!(
            "发现 {} 个未映射的 PUA 字符:\n{}",
            codes.len(),
            codes.join("\n")
        );
    }

    out
}

// ============== HTML 清理 ==============

/// 匹配所有 HTML 实体：&#数字; &#x十六进制; &命名实体;
/// 注意：不限制长度，避免长实体被忽略
static ENTITY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"&#(x[0-9a-fA-F]+|[0-9]+);|&(amp|lt|gt|quot|apos|nbsp|copy|reg|mdash|ndash|hellip|ldquo|rdquo|lsquo|rsquo|nbsp|middot|bull|emsp|ensp|thinsp|zwnj|zwj|lrm|rlm);").unwrap()
});

/// 匹配所有 HTML 标签：<...>
/// 注意：`[^>]*` 简单匹配，对番茄小说的 <p>/<br>/<img>/<span> 等足够
static TAG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<[^>]*>").unwrap()
});

/// 匹配 <br> 和 </p> </div> 后需要换行的情况
static BLOCK_END_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)</(p|div|h[1-6]|li|tr|blockquote)>|<br\s*/?>").unwrap()
});

/// 匹配 <img ...> 标签
static IMG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<img[^>]*>").unwrap()
});

/// 清理 HTML 内容为纯文本，并还原 PUA 反爬字符
///
/// 番茄小说章节内容是 HTML 格式，可能包含：
/// - `<p>...</p>` 段落
/// - `<br>` 换行
/// - `<img src="...">` 图片占位符
/// - HTML 实体（包括数字实体 &#xxxx; 反爬用）
/// - Unicode 私用区 (PUA) 字符（反爬用，需还原）
/// - 其他标签（<span> 等）
///
/// 处理步骤：
/// 1. 先把块级标签结束位置转换为换行符
/// 2. 把 <img> 替换为 [图片]
/// 3. 解码所有 HTML 实体（包括数字实体，处理反爬）
/// 4. 移除所有剩余 HTML 标签
/// 5. 还原 PUA 反爬字符为对应汉字
/// 6. 规范化空白
fn clean_html_content(html: &str) -> String {
    // 1. 块级标签结束 → 换行
    let with_newlines = BLOCK_END_RE.replace_all(html, "\n");

    // 2. <img> → [图片]
    let with_img_placeholder = IMG_RE.replace_all(&with_newlines, "[图片]");

    // 3. 解码所有 HTML 实体
    let decoded = ENTITY_RE.replace_all(&with_img_placeholder, |caps: &regex::Captures| {
        // 数字实体: &#123; 或 &#x7B;
        if let Some(m) = caps.get(1) {
            let s = m.as_str();
            let code = if let Some(hex) = s.strip_prefix('x').or_else(|| s.strip_prefix('X')) {
                u32::from_str_radix(hex, 16).unwrap_or(0)
            } else {
                s.parse::<u32>().unwrap_or(0)
            };
            return char::from_u32(code).map(|c| c.to_string()).unwrap_or_default();
        }
        // 命名实体
        if let Some(m) = caps.get(2) {
            return match m.as_str() {
                "amp" => "&",
                "lt" => "<",
                "gt" => ">",
                "quot" => "\"",
                "apos" => "'",
                "nbsp" => "\u{00A0}",
                "copy" => "©",
                "reg" => "®",
                "mdash" => "—",
                "ndash" => "–",
                "hellip" => "…",
                "ldquo" => "\u{201C}",
                "rdquo" => "\u{201D}",
                "lsquo" => "\u{2018}",
                "rsquo" => "\u{2019}",
                "middot" => "·",
                "bull" => "•",
                "emsp" => "\u{2003}",
                "ensp" => "\u{2002}",
                "thinsp" => "\u{2009}",
                "zwnj" => "\u{200C}",
                "zwj" => "\u{200D}",
                "lrm" => "\u{200E}",
                "rlm" => "\u{200F}",
                _ => "",
            }
            .to_string();
        }
        String::new()
    });

    // 4. 移除所有剩余 HTML 标签
    let no_tags = TAG_RE.replace_all(&decoded, "");

    // 5. 还原 PUA 反爬字符为对应汉字
    let restored = restore_pua_chars(&no_tags);

    // 6. 规范化空白：去除每行首尾空白，合并多余空行
    let mut cleaned = String::with_capacity(restored.len());
    let mut blank = 0;
    for line in restored.lines() {
        let t = line.trim();
        if t.is_empty() {
            blank += 1;
            if blank <= 1 {
                cleaned.push('\n');
            }
        } else {
            blank = 0;
            cleaned.push_str(t);
            cleaned.push('\n');
        }
    }
    cleaned.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_basic_html() {
        let html = "<p>第一段。</p><p>第二段。</p>";
        let text = clean_html_content(html);
        assert!(text.contains("第一段"));
        assert!(text.contains("第二段"));
    }

    #[test]
    fn test_clean_numeric_entities() {
        // 测试数字实体（反爬常用）：&#30340; = 的, &#22312; = 在, &#20013; = 中
        let html = "<p>&#30340;&#22312;&#20013;</p>";
        let text = clean_html_content(html);
        assert_eq!(text, "的在中");
    }

    #[test]
    fn test_clean_hex_entities() {
        // 测试十六进制实体：&#x7684; = 的
        let html = "<p>&#x7684;</p>";
        let text = clean_html_content(html);
        assert_eq!(text, "的");
    }

    #[test]
    fn test_clean_named_entities() {
        let html = "<p>&amp;&lt;&gt;&quot;</p>";
        let text = clean_html_content(html);
        assert_eq!(text, "&<>\"");
    }

    #[test]
    fn test_clean_img() {
        let html = "<p>前文<img src=\"x.jpg\">后文</p>";
        let text = clean_html_content(html);
        assert!(text.contains("[图片]"));
        assert!(text.contains("前文"));
        assert!(text.contains("后文"));
    }

    #[test]
    fn test_clean_br() {
        let html = "第一行<br>第二行";
        let text = clean_html_content(html);
        assert!(text.contains("第一行"));
        assert!(text.contains("第二行"));
    }

    #[test]
    fn test_clean_span() {
        // 测试 span 标签包裹的文本（番茄可能用此反爬）
        let html = "<p>前<span>中</span>后</p>";
        let text = clean_html_content(html);
        assert!(text.contains("前"));
        assert!(text.contains("中"));
        assert!(text.contains("后"));
    }

    #[test]
    fn test_restore_pua() {
        // U+E4F3 -> 的, U+E4C2 -> 人, U+E4DE -> 一
        let pua_text: String = vec![0xE4F3u32, 0xE4C2, 0xE4DE]
            .into_iter()
            .filter_map(char::from_u32)
            .collect();
        let restored = restore_pua_chars(&pua_text);
        assert_eq!(restored, "的人一");
    }

    #[test]
    fn test_clean_html_with_pua() {
        // 模拟番茄返回的内容：<p>的</p> 其中"的"被替换为 PUA
        let pua_de = char::from_u32(0xE4F3).unwrap();
        let html = format!("<p>{}人</p>", pua_de);
        let text = clean_html_content(&html);
        assert_eq!(text, "的人");
    }
}
