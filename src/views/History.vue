<script setup lang="ts">
import { h, onMounted, computed } from "vue";
import { useRouter } from "vue-router";
import {
  NDataTable,
  NButton,
  NSpace,
  NTag,
  NEmpty,
  NText,
  NPopconfirm,
  NIcon,
  useMessage,
  type DataTableColumns,
} from "naive-ui";
import {
  FolderOpenOutline,
  TrashOutline,
  DownloadOutline,
} from "@vicons/ionicons5";
import { useDownloadStore } from "@/stores/download";
import type { DownloadHistory } from "@/types";

const router = useRouter();
const message = useMessage();
const downloadStore = useDownloadStore();

const statusMap: Record<number, { label: string; type: any }> = {
  0: { label: "进行中", type: "info" },
  1: { label: "成功", type: "success" },
  2: { label: "失败", type: "error" },
};

function formatSize(bytes: number): string {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

function formatTime(ts: number): string {
  if (!ts) return "";
  const d = new Date(ts * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

const columns = computed<DataTableColumns<DownloadHistory>>(() => [
  {
    title: "书名",
    key: "book_name",
    width: 200,
    render: (row) =>
      h(
        "span",
        {
          class: "book-name-link",
          onClick: () =>
            router.push({ name: "book-detail", params: { bookId: row.book_id } }),
        },
        row.book_name
      ),
  },
  {
    title: "作者",
    key: "author",
    width: 120,
    render: (row) => row.author || "-",
  },
  {
    title: "格式",
    key: "format",
    width: 80,
    render: (row) =>
      h(NTag, { size: "small", type: row.format === "epub" ? "info" : "default" }, () =>
        row.format.toUpperCase()
      ),
  },
  {
    title: "大小",
    key: "file_size",
    width: 100,
    render: (row) => formatSize(row.file_size),
  },
  {
    title: "章节",
    key: "chapter_count",
    width: 80,
    render: (row) => `${row.chapter_count} 章`,
  },
  {
    title: "状态",
    key: "status",
    width: 80,
    render: (row) => {
      const s = statusMap[row.status];
      return h(NTag, { size: "small", type: s?.type || "default" }, () =>
        s?.label || "未知"
      );
    },
  },
  {
    title: "下载时间",
    key: "downloaded_at",
    width: 160,
    render: (row) => formatTime(row.downloaded_at),
  },
  {
    title: "操作",
    key: "actions",
    width: 120,
    render: (row) =>
      h(NSpace, { size: "small" }, () => [
        row.status === 1 &&
          h(
            NButton,
            {
              size: "small",
              quaternary: true,
              onClick: () => downloadStore.openFolder(row.file_path),
            },
            {
              icon: () => h(NIcon, null, () => h(FolderOpenOutline)),
              default: () => "打开",
            }
          ),
        h(
          NPopconfirm,
          {
            onPositiveClick: () => downloadStore.deleteHistory(row.id),
          },
          {
            trigger: () =>
              h(
                NButton,
                { size: "small", quaternary: true, type: "error" },
                {
                  icon: () => h(NIcon, null, () => h(TrashOutline)),
                }
              ),
            default: () => "确定删除此记录？",
          }
        ),
      ]),
  },
]);

onMounted(() => {
  downloadStore.loadHistory();
  downloadStore.initListener();
});

async function handleClearAll() {
  try {
    await downloadStore.clearHistory();
    message.success("已清空下载历史");
  } catch (e: any) {
    message.error(`清空失败: ${e}`);
  }
}
</script>

<template>
  <div class="history-page">
    <div class="page-header">
      <h2 class="page-title">下载历史</h2>
      <NSpace align="center">
        <NText depth="3">共 {{ downloadStore.history.length }} 条</NText>
        <NPopconfirm
          v-if="downloadStore.history.length > 0"
          @positive-click="handleClearAll"
        >
          <template #trigger>
            <NButton size="small" type="error" ghost>
              <template #icon>
                <NIcon><TrashOutline /></NIcon>
              </template>
              清空
            </NButton>
          </template>
          确定清空所有下载历史吗？(不会删除已下载的文件)
        </NPopconfirm>
      </NSpace>
    </div>

    <!-- 当前下载进度 -->
    <div
      v-if="downloadStore.activeDownloads.size > 0"
      class="active-downloads"
    >
      <div
        v-for="[bookId, progress] in downloadStore.activeDownloads"
        :key="bookId"
        class="active-download-item"
      >
        <NSpace align="center" justify="space-between">
          <NSpace align="center" size="small">
            <NIcon size="16" color="#ff6b35">
              <DownloadOutline />
            </NIcon>
            <NText>{{ progress.book_name || bookId }}</NText>
            <NTag size="small">{{ progress.format.toUpperCase() }}</NTag>
          </NSpace>
          <NText depth="3" size="small">{{ progress.message }}</NText>
        </NSpace>
      </div>
    </div>

    <NEmpty
      v-if="downloadStore.history.length === 0 && downloadStore.activeDownloads.size === 0"
      description="暂无下载记录"
      class="empty-state"
    />

    <NDataTable
      v-else
      :columns="columns"
      :data="downloadStore.history"
      :bordered="false"
      :single-line="false"
      size="small"
    />
  </div>
</template>

<style scoped>
.history-page {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-title {
  font-size: 22px;
  font-weight: 600;
  color: #ff6b35;
}

.active-downloads {
  margin-bottom: 20px;
  padding: 12px 16px;
  background: rgba(255, 107, 53, 0.08);
  border-radius: 6px;
  border-left: 3px solid #ff6b35;
}

.active-download-item {
  padding: 6px 0;
}

.empty-state {
  display: flex;
  justify-content: center;
  padding: 80px 0;
}

:deep(.book-name-link) {
  cursor: pointer;
  color: #63a8ff;
}

:deep(.book-name-link:hover) {
  text-decoration: underline;
}
</style>
