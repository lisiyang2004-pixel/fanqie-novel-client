<script setup lang="ts">
import { ref } from "vue";
import {
  NSpace,
  NInput,
  NButton,
  NGrid,
  NGridItem,
  NEmpty,
  NSpin,
  NText,
  NCard,
  NAlert,
  useMessage,
} from "naive-ui";
import { SearchOutline, OpenOutline } from "@vicons/ionicons5";
import BookCard from "@/components/BookCard.vue";
import { searchNovels } from "@/api";
import type { SearchBook } from "@/types";

const message = useMessage();
const keyword = ref("");
const results = ref<SearchBook[]>([]);
const loading = ref(false);
const hasSearched = ref(false);

async function handleSearch() {
  const q = keyword.value.trim();
  if (!q) {
    message.warning("请输入书名、作者、书籍 ID 或 URL");
    return;
  }

  loading.value = true;
  hasSearched.value = true;
  results.value = [];

  try {
    // 后端会自动判断输入类型：
    // - 纯数字/URL → 直接获取书籍
    // - 关键词 → 通过必应搜索 site:fanqienovel.com/page 获取书籍
    const data = await searchNovels(q, 0, 10);
    results.value = data;
    if (data.length === 0) {
      message.warning("未找到相关小说，请尝试其他关键词");
    } else {
      message.success(`找到 ${data.length} 本相关小说`);
    }
  } catch (e: any) {
    message.error(`搜索失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

function handleEnter(e: KeyboardEvent) {
  if (e.key === "Enter") {
    handleSearch();
  }
}

function openFanqieWebsite() {
  window.open("https://fanqienovel.com/", "_blank");
}
</script>

<template>
  <div class="search-page">
    <div class="search-header">
      <h2 class="page-title">搜索小说</h2>
      <NSpace class="search-bar" align="center">
        <NInput
          v-model:value="keyword"
          placeholder="输入书名、作者、书籍 ID 或 URL"
          size="large"
          clearable
          style="width: 500px"
          @keyup="handleEnter"
        />
        <NButton
          type="primary"
          size="large"
          :loading="loading"
          @click="handleSearch"
        >
          <template #icon>
            <SearchOutline />
          </template>
          搜索
        </NButton>
      </NSpace>
    </div>

    <!-- 使用提示 -->
    <NAlert class="usage-tip" type="info" :bordered="false">
      <template #header>
        <NSpace align="center" size="small">
          <NText strong>支持的搜索方式</NText>
          <NButton size="tiny" tertiary type="info" @click="openFanqieWebsite">
            <template #icon>
              <OpenOutline />
            </template>
            打开番茄小说网
          </NButton>
        </NSpace>
      </template>
      <NText depth="3" size="small">
        1. <NText strong>书名/作者关键词</NText>：如「十日终焉」「杀虫队队员」（通过必应搜索间接获取）<br />
        2. <NText strong>书籍 ID</NText>：如「7143038691944959011」<br />
        3. <NText strong>书籍 URL</NText>：如「https://fanqienovel.com/page/7143038691944959011」<br />
        <NText depth="2" size="tiny">提示：关键词搜索通过必应搜索引擎间接获取，结果可能不完整。若搜不到，请使用书籍 ID 或 URL。</NText>
      </NText>
    </NAlert>

    <div class="search-results">
      <NSpin v-if="loading && results.length === 0" size="large" class="loading-spin">
        <template #description>
          <NText depth="3">正在搜索中...</NText>
        </template>
      </NSpin>

      <NEmpty
        v-else-if="hasSearched && results.length === 0"
        description="未找到相关小说，请尝试其他关键词或使用书籍 ID/URL"
        class="empty-state"
      />

      <template v-else>
        <NGrid
          :cols="2"
          :x-gap="16"
          :y-gap="16"
          responsive="screen"
          :item-responsive="true"
        >
          <NGridItem v-for="book in results" :key="book.book_id">
            <BookCard :book="book" />
          </NGridItem>
        </NGrid>

        <div v-if="!hasSearched" class="search-tip">
          <NCard class="tip-card" :bordered="false">
            <NSpace vertical align="center" size="large">
              <NText depth="3" size="large">
                输入书名、作者、书籍 ID 或 URL 开始搜索
              </NText>
              <NText depth="3" size="small">
                示例：十日终焉 / 7143038691944959011 / https://fanqienovel.com/page/7143038691944959011
              </NText>
            </NSpace>
          </NCard>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.search-page {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}

.search-header {
  margin-bottom: 16px;
}

.page-title {
  font-size: 22px;
  font-weight: 600;
  margin-bottom: 16px;
  color: #ff6b35;
}

.search-bar {
  margin-bottom: 8px;
}

.usage-tip {
  margin-bottom: 24px;
}

.search-results {
  min-height: 400px;
}

.loading-spin,
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 80px 0;
}

.search-tip {
  display: flex;
  justify-content: center;
  padding: 80px 0;
}

.tip-card {
  max-width: 500px;
  text-align: center;
  padding: 40px;
}
</style>