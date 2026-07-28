import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  SearchBook,
  BookDetail,
  ChapterItem,
  ChapterContent,
  BookshelfItem,
  DownloadHistory,
  ReadingProgress,
  DownloadProgress,
} from "@/types";

// ==================== 搜索 ====================

export async function searchNovels(
  query: string,
  offset = 0,
  limit = 20
): Promise<SearchBook[]> {
  return invoke<SearchBook[]>("search_novels", { query, offset, limit });
}

/** 解析书籍输入（ID 或 URL），返回书籍 ID */
export async function parseBookInput(input: string): Promise<string> {
  return invoke<string>("parse_book_input", { input });
}

export async function getBookDetail(bookId: string): Promise<BookDetail> {
  return invoke<BookDetail>("get_book_detail", { bookId });
}

export async function getChapterList(bookId: string): Promise<ChapterItem[]> {
  return invoke<ChapterItem[]>("get_chapter_list", { bookId });
}

export async function getChapterContent(itemId: string): Promise<ChapterContent> {
  return invoke<ChapterContent>("get_chapter_content", { itemId });
}

// ==================== 书架 ====================

export async function getBookshelf(): Promise<BookshelfItem[]> {
  return invoke<BookshelfItem[]>("get_bookshelf");
}

export async function isInBookshelf(bookId: string): Promise<boolean> {
  return invoke<boolean>("is_in_bookshelf", { bookId });
}

export async function addToBookshelf(item: BookshelfItem): Promise<void> {
  return invoke<void>("add_to_bookshelf", { item });
}

export async function removeFromBookshelf(bookId: string): Promise<void> {
  return invoke<void>("remove_from_bookshelf", { bookId });
}

export async function updateBookshelfProgress(
  bookId: string,
  chapterIndex: number,
  chapterTitle: string
): Promise<void> {
  return invoke<void>("update_bookshelf_progress", {
    bookId,
    chapterIndex,
    chapterTitle,
  });
}

// ==================== 阅读进度 ====================

export async function saveReadingProgress(
  progress: ReadingProgress
): Promise<void> {
  return invoke<void>("save_reading_progress", { progress });
}

export async function getReadingProgress(
  bookId: string
): Promise<ReadingProgress | null> {
  return invoke<ReadingProgress | null>("get_reading_progress", { bookId });
}

// ==================== 下载 ====================

export async function downloadNovel(
  bookId: string,
  format: "txt" | "epub",
  outputDir?: string
): Promise<DownloadHistory> {
  return invoke<DownloadHistory>("download_novel", {
    bookId,
    format,
    outputDir: outputDir || null,
  });
}

/** 下载指定章节 */
export async function downloadNovelChapters(
  bookId: string,
  chapters: ChapterItem[],
  format: "txt" | "epub",
  outputDir?: string
): Promise<DownloadHistory> {
  return invoke<DownloadHistory>("download_novel_chapters", {
    bookId,
    chapters,
    format,
    outputDir: outputDir || null,
  });
}

export async function getDownloadHistory(): Promise<DownloadHistory[]> {
  return invoke<DownloadHistory[]>("get_download_history");
}

export async function deleteDownloadHistory(id: string): Promise<void> {
  return invoke<void>("delete_download_history", { id });
}

export async function clearDownloadHistory(): Promise<void> {
  return invoke<void>("clear_download_history");
}

export async function openInFolder(filePath: string): Promise<void> {
  return invoke<void>("open_in_folder", { filePath });
}

// ==================== 配置 ====================

export async function getAppDataDir(): Promise<string> {
  return invoke<string>("get_app_data_dir");
}

export async function getDefaultDownloadDir(): Promise<string> {
  return invoke<string>("get_default_download_dir");
}

// ==================== 事件监听 ====================

/** 监听下载进度事件 */
export async function onDownloadProgress(
  callback: (progress: DownloadProgress) => void
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download-progress", (event) => {
    callback(event.payload);
  });
}
