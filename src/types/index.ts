// 与 Rust models.rs 对应的类型定义

export interface SearchBook {
  book_id: string;
  book_name: string;
  author: string;
  cover: string;
  abstract: string;
  category: string;
  word_count: string;
  book_status: number;
  last_chapter_title: string;
  last_chapter_id: string;
}

export interface BookDetail {
  book_id: string;
  book_name: string;
  author: string;
  cover: string;
  abstract: string;
  category: string;
  word_count: string;
  book_status: number;
  last_chapter_title: string;
  last_chapter_id: string;
  chapter_count: number;
  last_chapter_time: number;
}

export interface ChapterItem {
  item_id: string;
  title: string;
  index: number;
  is_vip: boolean;
}

export interface ChapterContent {
  item_id: string;
  title: string;
  content: string;
  prev_item_id: string | null;
  next_item_id: string | null;
}

export interface BookshelfItem {
  book_id: string;
  book_name: string;
  author: string;
  cover: string;
  abstract: string;
  category: string;
  book_status: number;
  last_chapter_title: string;
  last_chapter_id: string;
  added_at: number;
  last_read_at: number | null;
  progress_chapter: number | null;
  progress_title: string | null;
}

export interface DownloadHistory {
  id: string;
  book_id: string;
  book_name: string;
  author: string;
  format: string;
  file_path: string;
  file_size: number;
  status: number;
  downloaded_at: number;
  chapter_count: number;
}

export interface ReadingProgress {
  book_id: string;
  item_id: string;
  chapter_title: string;
  chapter_index: number;
  scroll_percent: number;
  updated_at: number;
}

export interface DownloadProgress {
  book_id: string;
  book_name: string;
  format: string;
  current: number;
  total: number;
  status: string;
  message: string;
}
