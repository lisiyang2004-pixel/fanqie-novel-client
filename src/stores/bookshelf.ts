import { defineStore } from "pinia";
import { ref } from "vue";
import type { BookshelfItem } from "@/types";
import * as api from "@/api";

export const useBookshelfStore = defineStore("bookshelf", () => {
  const items = ref<BookshelfItem[]>([]);
  const loading = ref(false);

  async function load() {
    loading.value = true;
    try {
      items.value = await api.getBookshelf();
    } finally {
      loading.value = false;
    }
  }

  async function checkInBookshelf(bookId: string): Promise<boolean> {
    return api.isInBookshelf(bookId);
  }

  async function add(item: BookshelfItem) {
    await api.addToBookshelf(item);
    await load();
  }

  async function remove(bookId: string) {
    await api.removeFromBookshelf(bookId);
    await load();
  }

  async function updateProgress(
    bookId: string,
    chapterIndex: number,
    chapterTitle: string
  ) {
    await api.updateBookshelfProgress(bookId, chapterIndex, chapterTitle);
    await load();
  }

  return { items, loading, load, checkInBookshelf, add, remove, updateProgress };
});
