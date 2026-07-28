import { defineStore } from "pinia";
import { ref, onUnmounted } from "vue";
import type { DownloadHistory, DownloadProgress } from "@/types";
import * as api from "@/api";

export const useDownloadStore = defineStore("download", () => {
  const history = ref<DownloadHistory[]>([]);
  const loading = ref(false);
  /** 当前正在下载的任务: book_id -> progress */
  const activeDownloads = ref<Map<string, DownloadProgress>>(new Map());

  let unlistenFn: (() => void) | null = null;

  /** 初始化下载进度事件监听 */
  async function initListener() {
    if (unlistenFn) return;
    unlistenFn = await api.onDownloadProgress((progress) => {
      activeDownloads.value.set(progress.book_id, progress);
      // 触发响应式更新
      activeDownloads.value = new Map(activeDownloads.value);

      if (
        progress.status === "completed" ||
        progress.status === "failed"
      ) {
        setTimeout(() => {
          activeDownloads.value.delete(progress.book_id);
          activeDownloads.value = new Map(activeDownloads.value);
          loadHistory();
        }, 1000);
      }
    });
  }

  async function loadHistory() {
    loading.value = true;
    try {
      history.value = await api.getDownloadHistory();
    } finally {
      loading.value = false;
    }
  }

  async function startDownload(
    bookId: string,
    format: "txt" | "epub",
    outputDir?: string
  ): Promise<DownloadHistory> {
    await initListener();
    return api.downloadNovel(bookId, format, outputDir);
  }

  async function deleteHistory(id: string) {
    await api.deleteDownloadHistory(id);
    await loadHistory();
  }

  async function clearHistory() {
    await api.clearDownloadHistory();
    await loadHistory();
  }

  async function openFolder(filePath: string) {
    await api.openInFolder(filePath);
  }

  function getDownloadProgress(bookId: string): DownloadProgress | undefined {
    return activeDownloads.value.get(bookId);
  }

  function isDownloading(bookId: string): boolean {
    return activeDownloads.value.has(bookId);
  }

  function cleanup() {
    if (unlistenFn) {
      unlistenFn();
      unlistenFn = null;
    }
  }

  return {
    history,
    loading,
    activeDownloads,
    initListener,
    loadHistory,
    startDownload,
    deleteHistory,
    clearHistory,
    openFolder,
    getDownloadProgress,
    isDownloading,
    cleanup,
  };
});
