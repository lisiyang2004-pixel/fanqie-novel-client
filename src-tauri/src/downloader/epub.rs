use crate::error::{AppError, AppResult};
use crate::models::{BookDetail, ChapterContent, ChapterItem};
use std::io::{Write, Seek};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::DownloadResult;

/// 生成 EPUB 文件
pub fn generate_epub(
    detail: &BookDetail,
    chapters: &[ChapterItem],
    contents: &[ChapterContent],
    file_path: &Path,
) -> AppResult<DownloadResult> {
    let file = std::fs::File::create(file_path)?;
    let mut zip = ZipWriter::new(file);

    let stored = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    let book_uid = Uuid::new_v4().to_string();
    let book_title = html_escape(&detail.book_name);
    let book_author = html_escape(&detail.author);

    // 1. mimetype (必须是第一个，Stored方式)
    zip.start_file("mimetype", stored)?;
    zip.write_all(b"application/epub+zip")?;

    // 2. META-INF/container.xml
    zip.start_file("META-INF/container.xml", deflated)?;
    let container_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
    zip.write_all(container_xml.as_bytes())?;

    // 3. OEBPS/content.opf
    zip.start_file("OEBPS/content.opf", deflated)?;
    let mut manifest = String::new();
    let mut spine = String::new();
    let mut nav_points = String::new();

    // 封面页
    manifest.push_str(&format!(
        "    <item id=\"titlepage\" href=\"title.xhtml\" media-type=\"application/xhtml+xml\"/>\n"
    ));
    spine.push_str("    <itemref idref=\"titlepage\"/>\n");
    nav_points.push_str(&format!(
        "    <navPoint id=\"nav-0\" playOrder=\"1\">\n      <navLabel><text>封面</text></navLabel>\n      <content src=\"title.xhtml\"/>\n    </navPoint>\n"
    ));

    // 各章节
    for (i, chapter) in chapters.iter().enumerate() {
        let id = format!("chapter{}", i + 1);
        let href = format!("chapter{}.xhtml", i + 1);
        let title = html_escape(&chapter.title);

        manifest.push_str(&format!(
            "    <item id=\"{}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>\n",
            id, href
        ));
        spine.push_str(&format!("    <itemref idref=\"{}\"/>\n", id));
        nav_points.push_str(&format!(
            "    <navPoint id=\"nav-{}\" playOrder=\"{}\">\n      <navLabel><text>{}</text></navLabel>\n      <content src=\"{}\"/>\n    </navPoint>\n",
            i + 1,
            i + 2,
            title,
            href
        ));
    }

    let opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="BookId" version="2.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:title>{}</dc:title>
    <dc:creator opf:role="aut">{}</dc:creator>
    <dc:language>zh-CN</dc:language>
    <dc:identifier id="BookId">{}</dc:identifier>
    <dc:description>{}</dc:description>
    <dc:publisher>番茄小说下载客户端</dc:publisher>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
{}
  </manifest>
  <spine toc="ncx">
{}
  </spine>
</package>"#,
        book_title,
        book_author,
        book_uid,
        html_escape(&detail.r#abstract),
        manifest,
        spine
    );
    zip.write_all(opf.as_bytes())?;

    // 4. OEBPS/toc.ncx
    zip.start_file("OEBPS/toc.ncx", deflated)?;
    let ncx = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="{}"/>
    <meta name="dtb:depth" content="1"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>{}</text></docTitle>
  <navMap>
{}
  </navMap>
</ncx>"#,
        book_uid, book_title, nav_points
    );
    zip.write_all(ncx.as_bytes())?;

    // 5. 封面页 title.xhtml
    zip.start_file("OEBPS/title.xhtml", deflated)?;
    let status = if detail.book_status == 1 {
        "已完结"
    } else {
        "连载中"
    };
    let title_page = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta charset="utf-8"/>
  <title>{}</title>
</head>
<body>
  <h1>{}</h1>
  <p>作者: {}</p>
  <p>分类: {}</p>
  <p>字数: {}</p>
  <p>状态: {}</p>
  <p>简介: {}</p>
</body>
</html>"#,
        book_title,
        book_title,
        book_author,
        html_escape(&detail.category),
        html_escape(&detail.word_count),
        status,
        html_escape(&detail.r#abstract)
    );
    zip.write_all(title_page.as_bytes())?;

    // 6. 各章节内容
    for (i, chapter) in chapters.iter().enumerate() {
        let filename = format!("OEBPS/chapter{}.xhtml", i + 1);
        zip.start_file(&filename, deflated)?;

        let content = contents
            .get(i)
            .map(|c| c.content.as_str())
            .unwrap_or("[内容获取失败]");

        let html_content = text_to_html(content);
        let title = html_escape(&chapter.title);

        let chapter_html = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta charset="utf-8"/>
  <title>{}</title>
</head>
<body>
  <h2>{}</h2>
{}
</body>
</html>"#,
            title, title, html_content
        );
        zip.write_all(chapter_html.as_bytes())?;
    }

    zip.finish()?;

    let file_size = std::fs::metadata(file_path)?.len();

    Ok(DownloadResult {
        file_path: PathBuf::from(file_path),
        file_size,
        chapter_count: chapters.len(),
    })
}

/// HTML 转义
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// 纯文本转 HTML(每行一个 <p>)
fn text_to_html(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| format!("  <p>{}</p>", html_escape(line)))
        .collect::<Vec<_>>()
        .join("\n")
}

// 标记 Seek trait 已使用(zip 要求 Write + Seek)
#[allow(dead_code)]
fn _ensure_seek_used<W: Write + Seek>(_w: W) {}
