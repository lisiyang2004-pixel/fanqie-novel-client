<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { NCard, NTag, NText, NImage } from "naive-ui";
import type { SearchBook, BookshelfItem } from "@/types";

type BookLike = SearchBook | BookshelfItem;

const props = defineProps<{
  book: BookLike;
  /** 是否显示进度信息(书架模式) */
  showProgress?: boolean;
}>();

const router = useRouter();

const isFinished = computed(() => props.book.book_status === 1);
const statusText = computed(() => (isFinished.value ? "已完结" : "连载中"));
const statusType = computed(() =>
  isFinished.value ? "success" : "warning"
);

const coverUrl = computed(() => {
  const cover = props.book.cover;
  if (!cover) return "";
  if (cover.startsWith("http")) return cover;
  return cover;
});

function goToDetail() {
  router.push({ name: "book-detail", params: { bookId: props.book.book_id } });
}
</script>

<template>
  <NCard
    class="book-card"
    hoverable
    clickable
    @click="goToDetail"
  >
    <div class="book-card-body">
      <div class="book-cover">
        <NImage
          v-if="coverUrl"
          :src="coverUrl"
          object-fit="cover"
          width="90"
          height="120"
          :preview-disabled="true"
          fallback-src="data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI5MCIgaGVpZ2h0PSIxMjAiIHZpZXdCb3g9IjAgMCA5MCAxMjAiPjxyZWN0IHdpZHRoPSI5MCIgaGVpZ2h0PSIxMjAiIGZpbGw9IiMzMzMiLz48dGV4dCB4PSI0NSIgeT0iNjAiIGZvbnQtc2l6ZT0iMTIiIGZpbGw9IiM5OTkiIHRleHQtYW5jaG9yPSJtaWRkbGUiPunoW+WbvA8L3RleHQ+PC9zdmc+"
        />
        <div v-else class="cover-placeholder">暂无封面</div>
      </div>
      <div class="book-info">
        <h3 class="book-title text-ellipsis">{{ book.book_name }}</h3>
        <div class="book-meta">
          <NText depth="2" class="text-ellipsis">{{ book.author }}</NText>
        </div>
        <div class="book-tags">
          <NTag :type="statusType" size="small" round>
            {{ statusText }}
          </NTag>
          <NTag v-if="book.category" size="small" round>
            {{ book.category }}
          </NTag>
        </div>
        <p v-if="book.abstract" class="book-abstract text-ellipsis-2">
          {{ book.abstract }}
        </p>
        <div v-if="showProgress && (book as BookshelfItem).progress_title" class="book-progress">
          <NText depth="3" size="small">
            读至: {{ (book as BookshelfItem).progress_title }}
          </NText>
        </div>
      </div>
    </div>
  </NCard>
</template>

<style scoped>
.book-card {
  cursor: pointer;
  transition: transform 0.2s;
}

.book-card:hover {
  transform: translateY(-2px);
}

.book-card-body {
  display: flex;
  gap: 14px;
}

.book-cover {
  flex-shrink: 0;
  width: 90px;
  height: 120px;
  border-radius: 4px;
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
  font-size: 12px;
}

.book-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.book-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
}

.book-meta {
  font-size: 13px;
}

.book-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.book-abstract {
  font-size: 12px;
  color: #999;
  margin: 0;
}

.book-progress {
  margin-top: auto;
}
</style>
