<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { useRouter } from "vue-router";
import {
  NSpace,
  NButton,
  NTag,
  NImage,
  NSpin,
  NEmpty,
  NText,
  NCard,
  NSelect,
  NModal,
  NProgress,
  NCheckbox,
  NCheckboxGroup,
  NInputNumber,
  NSwitch,
  NDivider,
  NInput,
  useMessage,
  useDialog,
} from "naive-ui";
import {
  AddOutline,
  BookOutline,
  DownloadOutline,
  CloseOutline,
  FolderOpenOutline,
} from "@vicons/ionicons5";
import * as api from "@/api";
import { useBookshelfStore } from "@/stores/bookshelf";
import { useDownloadStore } from "@/stores/download";
import { useSettingsStore } from "@/stores/settings";
import type { BookDetail, ChapterItem, ReadingProgress, DownloadProgress } from "@/types";

const props = defineProps<{
  bookId: string;
}>();

const router = useRouter();
const message = useMessage();
const dialog = useDialog();
const bookshelfStore = useBookshelfStore();
const downloadStore = useDownloadStore();
const settingsStore = useSettingsStore();

const detail = ref<BookDetail | null>(null);
const chapters = ref<ChapterItem[]>([]);
const progress = ref<ReadingProgress | null>(null);
const inBookshelf = ref(false);
const loading = ref(true);
const chaptersLoading = ref(false);

// 下载相关
const showDownloadModal = ref(false);
// 默认从设置中读取格式
const downloadFormat = ref<"txt" | "epub">(settingsStore.defaultFormat);
const downloading = ref(false);
const downloadPercent = ref(0);
const downloadMessage = ref("");
const downloadTotal = ref(0);
const downloadCurrent = ref(0);

// 下载目录（优先使用设置中的，弹窗内可临时覆盖）
const outputDir = ref<string>(settingsStore.downloadDir);

// 章节选择相关
const selectMode = ref(false);
const selectedChapterIds = ref<string[]>([]);
// 范围选择
const rangeStart = ref<number>(1);
const rangeEnd = ref<number>(1);

const formatOptions = [
  { label: "TXT 纯文本", value: "txt" },
  { label: "EPUB 电子书", value: "epub" },
];

const isFinished = computed(() => detail.value?.book_status === 1);

const currentChapterIndex = computed(() => {
  if (!progress.value || !chapters.value.length) return -1;
  return chapters.value.findIndex(
    (c) => c.item_id === progress.value!.item_id
  );
});

const selectedCount = computed(() => selectedChapterIds.value.length);

const selectedChapters = computed(() =>
  chapters.value.filter((c) => selectedChapterIds.value.includes(c.item_id))
);

const outputDirDisplay = computed(() => {
  if (!outputDir.value) return "默认：系统下载目录/番茄小说（可在设置中修改）";
  return outputDir.value;
});

/** 下载进度事件监听器卸载函数 */
let unlistenProgress: (() => void) | null = null;

async function loadData() {
  loading.value = true;
  try {
    const [d, ch, inShelf, prog] = await Promise.all([
      api.getBookDetail(props.bookId),
      api.getChapterList(props.bookId),
      api.isInBookshelf(props.bookId),
      api.getReadingProgress(props.bookId),
    ]);
    detail.value = d;
    chapters.value = ch;
    inBookshelf.value = inShelf;
    progress.value = prog;
    rangeStart.value = 1;
    rangeEnd.value = Math.max(1, ch.length);
  } catch (e: any) {
    message.error(`加载失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function handleToggleBookshelf() {
  if (!detail.value) return;

  if (inBookshelf.value) {
    dialog.warning({
      title: "移除确认",
      content: `确定将《${detail.value.book_name}》从书架移除吗？`,
      positiveText: "移除",
      negativeText: "取消",
      onPositiveClick: async () => {
        await bookshelfStore.remove(props.bookId);
        inBookshelf.value = false;
        message.success("已从书架移除");
      },
    });
  } else {
    await bookshelfStore.add({
      book_id: detail.value.book_id,
      book_name: detail.value.book_name,
      author: detail.value.author,
      cover: detail.value.cover,
      abstract: detail.value.abstract,
      category: detail.value.category,
      book_status: detail.value.book_status,
      last_chapter_title: detail.value.last_chapter_title,
      last_chapter_id: detail.value.last_chapter_id,
      added_at: Math.floor(Date.now() / 1000),
      last_read_at: null,
      progress_chapter: null,
      progress_title: null,
    });
    inBookshelf.value = true;
    message.success("已加入书架");
  }
}

function handleRead(itemId?: string) {
  if (selectMode.value) return;
  const targetId = itemId || progress.value?.item_id || chapters.value[0]?.item_id;
  if (!targetId) {
    message.warning("暂无章节可阅读");
    return;
  }
  router.push({
    name: "reader",
    params: { bookId: props.bookId, itemId: targetId },
  });
}

// 切换选择模式
function toggleSelectMode(val: boolean) {
  selectMode.value = val;
  if (!val) {
    selectedChapterIds.value = [];
  }
}

function selectAll() {
  selectedChapterIds.value = chapters.value.map((c) => c.item_id);
}

function invertSelect() {
  const selectedSet = new Set(selectedChapterIds.value);
  selectedChapterIds.value = chapters.value
    .filter((c) => !selectedSet.has(c.item_id))
    .map((c) => c.item_id);
}

function clearSelection() {
  selectedChapterIds.value = [];
}

function selectRange() {
  const start = Math.min(rangeStart.value, rangeEnd.value);
  const end = Math.max(rangeStart.value, rangeEnd.value);
  const startIdx = Math.max(0, start - 1);
  const endIdx = Math.min(chapters.value.length, end);
  selectedChapterIds.value = chapters.value
    .slice(startIdx, endIdx)
    .map((c) => c.item_id);
  message.success(`已选中第 ${start} - ${end} 章，共 ${end - start + 1} 章`);
}

function selectFirstN(n: number) {
  selectedChapterIds.value = chapters.value
    .slice(0, Math.min(n, chapters.value.length))
    .map((c) => c.item_id);
}

// 选择下载目录（临时覆盖设置）
async function chooseDownloadDir() {
  try {
    const selected = await openDialogDir();
    if (selected) {
      outputDir.value = selected;
      message.success(`已设置下载目录: ${selected}`);
    }
  } catch (e: any) {
    message.error(`选择目录失败: ${e}`);
  }
}

async function openDialogDir(): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择下载保存目录",
  });
  if (typeof selected === "string" && selected) return selected;
  return null;
}

// 重置为设置中的目录
function resetDownloadDir() {
  outputDir.value = settingsStore.downloadDir;
  message.info(
    settingsStore.downloadDir
      ? `已重置为设置中的目录: ${settingsStore.downloadDir}`
      : "已重置为系统默认下载目录"
  );
}

// 打开下载弹窗
async function handleDownload() {
  if (!detail.value) return;

  if (selectMode.value && selectedChapterIds.value.length === 0) {
    message.warning("请先选择要下载的章节");
    return;
  }

  // 每次打开弹窗时同步设置中的目录和默认格式
  outputDir.value = settingsStore.downloadDir;
  downloadFormat.value = settingsStore.defaultFormat;

  showDownloadModal.value = true;
}

function updatePercent(p: DownloadProgress) {
  downloadCurrent.value = p.current;
  downloadTotal.value = p.total;
  if (p.total > 0) {
    downloadPercent.value = Math.round((p.current / p.total) * 100);
  } else {
    downloadPercent.value = 0;
  }
  downloadMessage.value = p.message;
}

async function startDownload() {
  if (!detail.value) return;
  downloading.value = true;
  downloadPercent.value = 0;
  downloadMessage.value = "准备下载...";
  downloadCurrent.value = 0;
  downloadTotal.value = 0;

  try {
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
    unlistenProgress = await api.onDownloadProgress((p) => {
      if (p.book_id !== props.bookId) return;
      updatePercent(p);

      if (p.status === "completed") {
        message.success(`下载完成: ${p.message}`);
        // 如果设置开启自动打开目录
        if (settingsStore.autoOpenFolder) {
          const filePath = p.message.replace(/^下载完成:\s*/, "");
          if (filePath) {
            api.openInFolder(filePath).catch(() => {});
          }
        }
      } else if (p.status === "failed") {
        message.error(`下载失败: ${p.message}`);
      }
    });
  } catch (e: any) {
    console.warn("监听下载进度失败:", e);
  }

  try {
    const dir = outputDir.value || undefined;

    if (selectMode.value && selectedChapterIds.value.length > 0) {
      downloadMessage.value = `正在下载 ${selectedChapters.value.length} 章...`;
      downloadTotal.value = selectedChapters.value.length;
      await api.downloadNovelChapters(
        props.bookId,
        selectedChapters.value,
        downloadFormat.value,
        dir
      );
    } else {
      downloadMessage.value = `正在下载全本 ${chapters.value.length} 章...`;
      downloadTotal.value = chapters.value.length;
      await downloadStore.startDownload(props.bookId, downloadFormat.value, dir);
    }
    showDownloadModal.value = false;
    message.success("下载任务已完成");
  } catch (e: any) {
    message.error(`下载失败: ${e}`);
  } finally {
    downloading.value = false;
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
  }
}

function formatTime(ts: number): string {
  if (!ts) return "";
  const d = new Date(ts * 1000);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

onMounted(() => {
  loadData();
  downloadStore.initListener();
  outputDir.value = settingsStore.downloadDir;
  downloadFormat.value = settingsStore.defaultFormat;
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
    unlistenProgress = null;
  }
});
</script>

<template>
  <div class="detail-page">
    <NSpin v-if="loading" size="large" class="loading" />

    <template v-else-if="detail">
      <!-- 书籍信息区 -->
      <div class="book-header">
        <div class="book-cover">
          <NImage
            v-if="detail.cover"
            :src="detail.cover"
            object-fit="cover"
            width="150"
            height="200"
            :preview-disabled="true"
          />
          <div v-else class="cover-placeholder">暂无封面</div>
        </div>

        <div class="book-info">
          <h1 class="book-title">{{ detail.book_name }}</h1>
          <NSpace class="book-meta" align="center" size="small">
            <NText depth="2">{{ detail.author }}</NText>
            <NTag :type="isFinished ? 'success' : 'warning'" size="small" round>
              {{ isFinished ? "已完结" : "连载中" }}
            </NTag>
            <NTag v-if="detail.category" size="small" round>
              {{ detail.category }}
            </NTag>
          </NSpace>

          <NSpace class="book-stats" size="large">
            <div class="stat">
              <NText depth="3" size="small">字数</NText>
              <NText>{{ detail.word_count || "未知" }}</NText>
            </div>
            <div class="stat">
              <NText depth="3" size="small">章节</NText>
              <NText>{{ detail.chapter_count || chapters.length }}</NText>
            </div>
            <div class="stat" v-if="detail.last_chapter_time">
              <NText depth="3" size="small">更新</NText>
              <NText>{{ formatTime(detail.last_chapter_time) }}</NText>
            </div>
          </NSpace>

          <div v-if="progress" class="reading-progress">
            <NText depth="3" size="small">
              上次读至: {{ progress.chapter_title }}
            </NText>
          </div>

          <NSpace class="book-actions" size="small">
            <NButton
              :type="inBookshelf ? 'default' : 'primary'"
              @click="handleToggleBookshelf"
            >
              <template #icon>
                <AddOutline v-if="!inBookshelf" />
                <CloseOutline v-else />
              </template>
              {{ inBookshelf ? "移出书架" : "加入书架" }}
            </NButton>
            <NButton type="info" @click="handleRead()">
              <template #icon>
                <BookOutline />
              </template>
              {{ progress ? "继续阅读" : "开始阅读" }}
            </NButton>
            <NButton @click="handleDownload">
              <template #icon>
                <DownloadOutline />
              </template>
              {{ selectMode && selectedCount > 0 ? `下载选中(${selectedCount})` : "下载" }}
            </NButton>
          </NSpace>
        </div>
      </div>

      <!-- 简介 -->
      <NCard class="book-abstract-card" title="简介">
        <p class="book-abstract">{{ detail.abstract || "暂无简介" }}</p>
      </NCard>

      <!-- 章节目录 -->
      <NCard class="chapter-list-card">
        <template #header>
          <NSpace align="center" justify="space-between">
            <NText strong>目录</NText>
            <NSpace align="center" size="small">
              <NText v-if="selectMode && selectedCount > 0" depth="3" size="small">
                已选 {{ selectedCount }} / {{ chapters.length }} 章
              </NText>
              <NSwitch
                v-model:value="selectMode"
                size="small"
                @update:value="toggleSelectMode"
              >
                <template #checked>选择</template>
                <template #unchecked>选择</template>
              </NSwitch>
            </NSpace>
          </NSpace>
        </template>

        <!-- 选择模式工具栏 -->
        <div v-if="selectMode" class="select-toolbar">
          <NSpace wrap size="small">
            <NButton size="small" @click="selectAll">全选</NButton>
            <NButton size="small" @click="invertSelect">反选</NButton>
            <NButton size="small" @click="clearSelection">清空</NButton>
            <NButton size="small" @click="selectFirstN(10)">前10章</NButton>
            <NButton size="small" @click="selectFirstN(50)">前50章</NButton>
            <NButton size="small" @click="selectFirstN(100)">前100章</NButton>
          </NSpace>

          <NDivider style="margin: 10px 0" />

          <NSpace align="center" size="small" class="range-select">
            <NText depth="3" size="small">范围选择:</NText>
            <NInputNumber
              v-model:value="rangeStart"
              size="small"
              :min="1"
              :max="chapters.length"
              style="width: 90px"
            />
            <NText depth="3" size="small">至</NText>
            <NInputNumber
              v-model:value="rangeEnd"
              size="small"
              :min="1"
              :max="chapters.length"
              style="width: 90px"
            />
            <NButton size="small" type="primary" ghost @click="selectRange">
              选择
            </NButton>
          </NSpace>

          <NDivider style="margin: 10px 0" />
        </div>

        <NSpin v-if="chaptersLoading" size="small" />
        <NEmpty v-else-if="chapters.length === 0" description="暂无章节" />

        <!-- 选择模式 -->
        <NCheckboxGroup
          v-else-if="selectMode"
          v-model:value="selectedChapterIds"
          class="chapter-list"
        >
          <div
            v-for="(ch, idx) in chapters"
            :key="ch.item_id"
            class="chapter-item-selectable"
          >
            <NCheckbox :value="ch.item_id" :label="ch.title" />
            <NTag v-if="ch.is_vip" size="tiny" type="warning">VIP</NTag>
            <NText depth="3" size="small" class="chapter-index">{{ idx + 1 }}</NText>
          </div>
        </NCheckboxGroup>

        <!-- 普通模式 -->
        <div v-else class="chapter-list">
          <div
            v-for="(ch, idx) in chapters"
            :key="ch.item_id"
            class="chapter-item"
            :class="{ active: currentChapterIndex === idx }"
            @click="handleRead(ch.item_id)"
          >
            <NText class="chapter-title text-ellipsis">{{ ch.title }}</NText>
            <NTag v-if="ch.is_vip" size="tiny" type="warning">VIP</NTag>
          </div>
        </div>

        <!-- 选择模式下的底部下载按钮 -->
        <div v-if="selectMode && selectedCount > 0" class="select-bottom-bar">
          <NSpace align="center" justify="space-between">
            <NText>
              已选中 <NText type="primary">{{ selectedCount }}</NText> 章
            </NText>
            <NButton type="primary" @click="handleDownload">
              <template #icon>
                <DownloadOutline />
              </template>
              下载选中章节
            </NButton>
          </NSpace>
        </div>
      </NCard>
    </template>

    <NEmpty v-else description="书籍信息加载失败" />

    <!-- 下载选择弹窗 -->
    <NModal
      v-model:show="showDownloadModal"
      preset="card"
      :title="selectMode ? `下载选中章节 (${selectedCount} 章)` : '下载全本'"
      style="width: 480px"
    >
      <NSpace vertical size="large">
        <!-- 下载目录选择 -->
        <div class="dir-section">
          <NText depth="3" size="small">下载保存目录</NText>
          <NSpace align="center" size="small" style="margin-top: 6px">
            <NInput
              :value="outputDirDisplay"
              readonly
              size="small"
              style="flex: 1; min-width: 0"
              :placeholder="outputDirDisplay"
            />
            <NButton size="small" type="primary" ghost @click="chooseDownloadDir">
              <template #icon>
                <FolderOpenOutline />
              </template>
              选择
            </NButton>
            <NButton
              v-if="outputDir || settingsStore.downloadDir"
              size="small"
              quaternary
              @click="resetDownloadDir"
            >
              默认
            </NButton>
          </NSpace>
          <NText depth="3" size="tiny" style="margin-top: 4px; display: block">
            可在此临时指定下载目录；要永久修改请到「设置」页面配置。
          </NText>
        </div>

        <NDivider style="margin: 4px 0" />

        <div>
          <NText depth="3">选择下载格式：</NText>
        </div>
        <NSelect
          v-model:value="downloadFormat"
          :options="formatOptions"
        />
        <NText depth="3" size="small">
          <template v-if="selectMode">
            将下载选中的 {{ selectedCount }} 章内容。
          </template>
          <template v-else>
            将下载全部 {{ chapters.length }} 章内容。
          </template>
          TXT 适合在任意设备阅读；EPUB 适合电子书阅读器，支持目录导航。
        </NText>

        <div v-if="downloading" class="download-progress">
          <NProgress
            type="line"
            :percentage="downloadPercent"
            :status="downloadPercent === 100 ? 'success' : 'default'"
          />
          <NSpace justify="space-between" style="margin-top: 6px">
            <NText depth="3" size="small">{{ downloadMessage }}</NText>
            <NText depth="3" size="small" v-if="downloadTotal > 0">
              {{ downloadCurrent }} / {{ downloadTotal }}
            </NText>
          </NSpace>
        </div>

        <NSpace justify="end">
          <NButton @click="showDownloadModal = false" :disabled="downloading">取消</NButton>
          <NButton
            type="primary"
            :loading="downloading"
            @click="startDownload"
          >
            开始下载
          </NButton>
        </NSpace>
      </NSpace>
    </NModal>
  </div>
</template>

<style scoped>
.detail-page {
  padding: 24px;
  max-width: 1000px;
  margin: 0 auto;
}

.loading {
  display: flex;
  justify-content: center;
  padding: 100px 0;
}

.book-header {
  display: flex;
  gap: 24px;
  margin-bottom: 24px;
}

.book-cover {
  flex-shrink: 0;
  width: 150px;
  height: 200px;
  border-radius: 6px;
  overflow: hidden;
  background: #2a2a2e;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #666;
  font-size: 14px;
}

.book-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.book-title {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
}

.book-stats {
  margin-top: 4px;
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.reading-progress {
  padding: 8px 12px;
  background: rgba(255, 107, 53, 0.1);
  border-radius: 4px;
  border-left: 3px solid #ff6b35;
}

.book-actions {
  margin-top: 8px;
}

.book-abstract-card {
  margin-bottom: 24px;
}

.book-abstract {
  line-height: 1.8;
  color: #ccc;
}

.chapter-list-card {
  margin-bottom: 24px;
}

.select-toolbar {
  margin-bottom: 12px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 6px;
}

.range-select {
  flex-wrap: wrap;
}

.chapter-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 4px;
  max-height: 600px;
  overflow-y: auto;
}

.chapter-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.chapter-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.chapter-item.active {
  background: rgba(255, 107, 53, 0.15);
  color: #ff6b35;
}

.chapter-item-selectable {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 4px;
  transition: background 0.2s;
}

.chapter-item-selectable:hover {
  background: rgba(255, 255, 255, 0.04);
}

.chapter-item-selectable :deep(.n-checkbox) {
  flex: 1;
  min-width: 0;
}

.chapter-item-selectable :deep(.n-checkbox__label) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chapter-index {
  flex-shrink: 0;
  width: 32px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.chapter-title {
  flex: 1;
  font-size: 13px;
}

.select-bottom-bar {
  position: sticky;
  bottom: 0;
  margin-top: 12px;
  padding: 12px 16px;
  background: rgba(24, 24, 28, 0.95);
  backdrop-filter: blur(8px);
  border-top: 1px solid rgba(255, 107, 53, 0.3);
  border-radius: 6px;
}

.download-progress {
  padding: 12px 0;
}

.dir-section {
  padding: 8px 0;
}
</style>
