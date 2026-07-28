<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from "vue";
import { useRouter } from "vue-router";
import {
  NButton,
  NIcon,
  NSpace,
  NSpin,
  NDrawer,
  NDrawerContent,
  NList,
  NListItem,
  NText,
  NPopover,
  NRadioGroup,
  NRadioButton,
  NSlider,
  NDivider,
  useMessage,
} from "naive-ui";
import {
  ArrowBackOutline,
  ListOutline,
  SettingsOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
  ArrowUpOutline,
} from "@vicons/ionicons5";
import * as api from "@/api";
import { useReaderStore, type ReaderTheme } from "@/stores/reader";
import { useBookshelfStore } from "@/stores/bookshelf";
import type { ChapterContent, ChapterItem, BookDetail } from "@/types";

const props = defineProps<{
  bookId: string;
  itemId: string;
}>();

const router = useRouter();
const message = useMessage();
const readerStore = useReaderStore();
const bookshelfStore = useBookshelfStore();

const chapter = ref<ChapterContent | null>(null);
const chapters = ref<ChapterItem[]>([]);
const detail = ref<BookDetail | null>(null);
const loading = ref(false);
const showChapterList = ref(false);
const showSettings = ref(false);
const contentRef = ref<HTMLElement | null>(null);

const themeColors = computed(() => readerStore.getThemeColors());

const contentStyle = computed(() => ({
  fontSize: `${readerStore.settings.fontSize}px`,
  lineHeight: readerStore.settings.lineHeight,
  fontFamily: readerStore.settings.fontFamily,
  backgroundColor: themeColors.value.bg,
  color: themeColors.value.text,
}));

const currentChapterIndex = computed(() => {
  return chapters.value.findIndex((c) => c.item_id === props.itemId);
});

async function loadChapterData() {
  loading.value = true;
  chapter.value = null;

  try {
    // 并行获取章节内容和目录(目录只加载一次)
    const promises: Promise<any>[] = [api.getChapterContent(props.itemId)];

    if (chapters.value.length === 0) {
      promises.push(api.getChapterList(props.bookId));
    }
    if (!detail.value) {
      promises.push(api.getBookDetail(props.bookId));
    }

    const results = await Promise.all(promises);
    chapter.value = results[0];

    if (results.length > 1) {
      let idx = 1;
      if (chapters.value.length === 0) {
        chapters.value = results[idx++];
      }
      if (!detail.value && results[idx]) {
        detail.value = results[idx];
      }
    }

    // 滚动到顶部
    await nextTick();
    if (contentRef.value) {
      contentRef.value.scrollTop = 0;
    }

    // 保存阅读进度
    await saveProgress();
  } catch (e: any) {
    message.error(`加载章节失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function saveProgress() {
  if (!chapter.value) return;

  const chapterIndex = currentChapterIndex.value + 1;
  const progress = {
    book_id: props.bookId,
    item_id: props.itemId,
    chapter_title: chapter.value.title,
    chapter_index: chapterIndex,
    scroll_percent: 0,
    updated_at: Math.floor(Date.now() / 1000),
  };

  try {
    await api.saveReadingProgress(progress);
    // 如果在书架中，更新书架进度
    const inShelf = await api.isInBookshelf(props.bookId);
    if (inShelf) {
      await bookshelfStore.updateProgress(
        props.bookId,
        chapterIndex,
        chapter.value.title
      );
    }
  } catch (e) {
    // 进度保存失败不影响阅读
    console.error("保存进度失败:", e);
  }
}

function goPrev() {
  if (!chapter.value?.prev_item_id) {
    message.info("已是第一章");
    return;
  }
  router.push({
    name: "reader",
    params: { bookId: props.bookId, itemId: chapter.value.prev_item_id },
  });
}

function goNext() {
  if (!chapter.value?.next_item_id) {
    message.info("已是最后一章");
    return;
  }
  router.push({
    name: "reader",
    params: { bookId: props.bookId, itemId: chapter.value.next_item_id },
  });
}

function goToChapter(item_id: string) {
  showChapterList.value = false;
  router.push({
    name: "reader",
    params: { bookId: props.bookId, itemId: item_id },
  });
}

function goBack() {
  if (detail.value) {
    router.push({ name: "book-detail", params: { bookId: props.bookId } });
  } else {
    router.back();
  }
}

function scrollToTop() {
  if (contentRef.value) {
    contentRef.value.scrollTo({ top: 0, behavior: "smooth" });
  }
}

// 监听路由参数变化(上一章/下一章)
watch(
  () => props.itemId,
  () => {
    loadChapterData();
  }
);

// 键盘快捷键
function handleKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowLeft") {
    goPrev();
  } else if (e.key === "ArrowRight") {
    goNext();
  } else if (e.key === "Escape") {
    goBack();
  }
}

onMounted(() => {
  loadChapterData();
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <div class="reader-page" :style="contentStyle">
    <!-- 顶部工具栏 -->
    <div class="reader-toolbar">
      <NSpace align="center" size="small">
        <NButton quaternary circle @click="goBack">
          <template #icon>
            <NIcon><ArrowBackOutline /></NIcon>
          </template>
        </NButton>
        <NText class="toolbar-title">
          {{ detail?.book_name || "阅读" }}
        </NText>
      </NSpace>

      <NSpace align="center" size="small">
        <NButton quaternary circle @click="showChapterList = true">
          <template #icon>
            <NIcon><ListOutline /></NIcon>
          </template>
        </NButton>
        <NPopover trigger="click" placement="bottom-end" :width="280">
          <template #trigger>
            <NButton quaternary circle>
              <template #icon>
                <NIcon><SettingsOutline /></NIcon>
              </template>
            </NButton>
          </template>
          <div class="settings-panel">
            <div class="setting-row">
              <NText depth="2">字体大小</NText>
              <NSpace align="center" size="small">
                <NButton size="small" quaternary @click="readerStore.decreaseFont">
                  A-
                </NButton>
                <NText>{{ readerStore.settings.fontSize }}</NText>
                <NButton size="small" quaternary @click="readerStore.increaseFont">
                  A+
                </NButton>
              </NSpace>
            </div>
            <NDivider />
            <div class="setting-row">
              <NText depth="2">行间距</NText>
              <NSlider
                :value="readerStore.settings.lineHeight"
                :min="1.2"
                :max="3"
                :step="0.1"
                style="width: 140px"
                @update:value="(v: number) => readerStore.setLineHeight(v)"
              />
            </div>
            <NDivider />
            <div class="setting-row">
              <NText depth="2">主题</NText>
              <NRadioGroup
                :value="readerStore.settings.theme"
                size="small"
                @update:value="(v: ReaderTheme) => readerStore.setTheme(v)"
              >
                <NRadioButton value="light">白</NRadioButton>
                <NRadioButton value="sepia">黄</NRadioButton>
                <NRadioButton value="dark">黑</NRadioButton>
              </NRadioGroup>
            </div>
          </div>
        </NPopover>
      </NSpace>
    </div>

    <!-- 内容区 -->
    <div ref="contentRef" class="reader-content-wrapper">
      <NSpin v-if="loading" size="large" class="reader-loading" />

      <div v-else-if="chapter" class="reader-content">
        <h1 class="chapter-title">{{ chapter.title }}</h1>
        <div class="chapter-body">
          <p v-for="(para, i) in chapter.content.split('\n')" :key="i">
            {{ para }}
          </p>
        </div>

        <!-- 章节导航 -->
        <div class="chapter-nav">
          <NButton
            :disabled="!chapter.prev_item_id"
            @click="goPrev"
          >
            <template #icon>
              <NIcon><ChevronBackOutline /></NIcon>
            </template>
            上一章
          </NButton>
          <NButton quaternary @click="scrollToTop">
            <template #icon>
              <NIcon><ArrowUpOutline /></NIcon>
            </template>
          </NButton>
          <NButton
            :disabled="!chapter.next_item_id"
            @click="goNext"
          >
            下一章
            <template #icon>
              <NIcon><ChevronForwardOutline /></NIcon>
            </template>
          </NButton>
        </div>
      </div>
    </div>

    <!-- 章节目录抽屉 -->
    <NDrawer v-model:show="showChapterList" :width="360" placement="left">
      <NDrawerContent title="目录" closable>
        <NList class="chapter-drawer-list" hoverable clickable>
          <NListItem
            v-for="(ch, idx) in chapters"
            :key="ch.item_id"
            :class="{ active: ch.item_id === itemId }"
            @click="goToChapter(ch.item_id)"
          >
            <NText
              class="text-ellipsis"
              :type="ch.item_id === itemId ? 'primary' : 'default'"
            >
              {{ ch.title }}
            </NText>
          </NListItem>
        </NList>
      </NDrawerContent>
    </NDrawer>
  </div>
</template>

<style scoped>
.reader-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
  transition: background-color 0.3s, color 0.3s;
}

.reader-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 20px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.2);
  flex-shrink: 0;
}

.toolbar-title {
  font-size: 15px;
  font-weight: 500;
}

.reader-content-wrapper {
  flex: 1;
  overflow-y: auto;
  position: relative;
}

.reader-loading {
  display: flex;
  justify-content: center;
  padding: 100px 0;
}

.reader-content {
  max-width: 800px;
  margin: 0 auto;
  padding: 30px 24px 60px;
}

.chapter-title {
  font-size: 22px;
  font-weight: 700;
  text-align: center;
  margin-bottom: 30px;
}

.chapter-body p {
  margin: 1em 0;
  text-indent: 2em;
}

.chapter-nav {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 40px;
  padding-top: 20px;
  border-top: 1px solid rgba(128, 128, 128, 0.2);
}

.settings-panel {
  padding: 8px 0;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
}

.chapter-drawer-list {
  cursor: pointer;
}

.chapter-drawer-list :deep(.n-list-item.active) {
  background: rgba(255, 107, 53, 0.1);
}
</style>
