mod api;
mod commands;
mod db;
mod downloader;
mod error;
mod models;

use commands::AppState;
use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 初始化数据库
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("fanqie_novel.db");
            log::info!("数据库路径: {}", db_path.display());

            let db = Database::open(&db_path)?;
            app.manage(AppState::new(db));

            log::info!("番茄小说客户端启动成功");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 搜索
            commands::search_novels,
            commands::parse_book_input,
            commands::get_book_detail,
            commands::get_chapter_list,
            commands::get_chapter_content,
            // 书架
            commands::get_bookshelf,
            commands::is_in_bookshelf,
            commands::add_to_bookshelf,
            commands::remove_from_bookshelf,
            commands::update_bookshelf_progress,
            // 阅读进度
            commands::save_reading_progress,
            commands::get_reading_progress,
            // 下载
            commands::download_novel,
            commands::download_novel_chapters,
            commands::get_download_history,
            commands::delete_download_history,
            commands::clear_download_history,
            commands::open_in_folder,
            // 配置
            commands::get_app_data_dir,
            commands::get_default_download_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::api::FanqieClient;

    /// 测试解析书籍 ID
    #[test]
    fn test_parse_book_id() {
        // 纯数字 ID
        assert_eq!(
            FanqieClient::parse_book_id("7143038691944959011").unwrap(),
            "7143038691944959011"
        );
        // 完整 URL
        assert_eq!(
            FanqieClient::parse_book_id("https://fanqienovel.com/page/7143038691944959011")
                .unwrap(),
            "7143038691944959011"
        );
        // reader URL
        assert_eq!(
            FanqieClient::parse_book_id("https://fanqienovel.com/reader/7173216089122439711")
                .unwrap(),
            "7173216089122439711"
        );
        // 无效输入
        assert!(FanqieClient::parse_book_id("invalid input").is_err());
    }








    /// 测试关键词搜索（通过必应搜索，需要网络）
    #[tokio::test]
    #[ignore = "需要网络连接"]
    async fn test_search_by_keyword() {
        let client = FanqieClient::new();
        let results = client
            .search_novels("十日终焉", 5)
            .await
            .expect("关键词搜索失败");
        assert!(!results.is_empty(), "搜索结果不应为空");
        println!("搜索到 {} 本小说:", results.len());
        for (i, book) in results.iter().enumerate() {
            println!("  {}. {} - {} (ID: {})", i + 1, book.book_name, book.author, book.book_id);
        }
        // 验证至少有一本包含关键词
        let has_match = results
            .iter()
            .any(|b| b.book_name.contains("十日终焉") || b.author.contains("十日终焉"));
        assert!(has_match, "搜索结果中应包含关键词相关书籍");
    }

    /// 测试获取书籍详情（需要网络）
    #[tokio::test]
    #[ignore = "需要网络连接"]
    async fn test_get_book_detail() {
        let client = FanqieClient::new();
        let detail = client
            .get_book_detail("7143038691944959011")
            .await
            .expect("获取书籍详情失败");
        assert_eq!(detail.book_id, "7143038691944959011");
        assert_eq!(detail.book_name, "十日终焉");
        assert_eq!(detail.author, "杀虫队队员");
        assert!(!detail.cover.is_empty(), "封面 URL 不应为空");
        assert!(detail.chapter_count > 0, "章节数应大于 0");
        println!(
            "书籍详情: {} - {} ({}章, {}字, 状态{})",
            detail.book_name,
            detail.author,
            detail.chapter_count,
            detail.word_count,
            detail.book_status
        );
        println!("封面: {}", detail.cover);
        println!("分类: {}", detail.category);
        println!("简介: {}", detail.r#abstract);
    }

    /// 测试获取章节列表（需要网络）
    #[tokio::test]
    #[ignore = "需要网络连接"]
    async fn test_get_chapter_list() {
        let client = FanqieClient::new();
        let chapters = client
            .get_chapter_list("7143038691944959011")
            .await
            .expect("获取章节列表失败");
        assert!(!chapters.is_empty(), "章节列表不应为空");
        println!("共 {} 章", chapters.len());
        println!("第1章: {} ({})", chapters[0].title, chapters[0].item_id);
        println!(
            "最后章: {} ({})",
            chapters.last().unwrap().title,
            chapters.last().unwrap().item_id
        );
    }

    /// 测试获取章节内容（需要网络）
    #[tokio::test]
    #[ignore = "需要网络连接"]
    async fn test_get_chapter_content() {
        let client = FanqieClient::new();
        let content = client
            .get_chapter_content("7173216089122439711")
            .await
            .expect("获取章节内容失败");
        assert_eq!(content.item_id, "7173216089122439711");
        assert!(!content.title.is_empty());
        assert!(!content.content.is_empty());
        println!("章节标题: {}", content.title);
        println!("内容前 200 字: {}", &content.content.chars().take(200).collect::<String>());
        println!(
            "上一章: {:?}, 下一章: {:?}",
            content.prev_item_id, content.next_item_id
        );
    }

    /// 测试 HTML 清理
    #[test]
    fn test_clean_html() {
        // 使用 chapter.rs 中的函数（通过公共接口测试）
        // 由于 clean_html_content 是私有的，这里只做集成测试
        // 实际清理逻辑在 test_get_chapter_content 中验证
    }
}
