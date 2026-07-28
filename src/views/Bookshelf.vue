<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter } from "vue-router";
import {
  NGrid,
  NGridItem,
  NEmpty,
  NSpin,
  NButton,
  NSpace,
  NText,
  useMessage,
} from "naive-ui";
import BookCard from "@/components/BookCard.vue";
import { useBookshelfStore } from "@/stores/bookshelf";

const router = useRouter();
const message = useMessage();
const bookshelfStore = useBookshelfStore();

async function handleContinueReading(bookId: string, itemId?: string) {
  if (itemId) {
    router.push({ name: "reader", params: { bookId, itemId } });
  } else {
    router.push({ name: "book-detail", params: { bookId } });
  }
}

onMounted(() => {
  bookshelfStore.load();
});
</script>

<template>
  <div class="bookshelf-page">
    <div class="page-header">
      <h2 class="page-title">我的书架</h2>
      <NText depth="3">共 {{ bookshelfStore.items.length }} 本</NText>
    </div>

    <NSpin v-if="bookshelfStore.loading" size="large" class="loading" />

    <NEmpty
      v-else-if="bookshelfStore.items.length === 0"
      description="书架还是空的，去搜索添加小说吧"
      class="empty-state"
    >
      <template #extra>
        <NButton type="primary" @click="router.push('/search')">
          去搜索
        </NButton>
      </template>
    </NEmpty>

    <NGrid
      v-else
      :cols="2"
      :x-gap="16"
      :y-gap="16"
      responsive="screen"
      :item-responsive="true"
    >
      <NGridItem v-for="book in bookshelfStore.items" :key="book.book_id">
        <div class="bookshelf-item">
          <BookCard :book="book" show-progress />
          <div v-if="book.progress_title" class="continue-btn">
            <NButton
              size="small"
              type="primary"
              secondary
              @click.stop="handleContinueReading(book.book_id, book.last_chapter_id)"
            >
              继续阅读
            </NButton>
          </div>
        </div>
      </NGridItem>
    </NGrid>
  </div>
</template>

<style scoped>
.bookshelf-page {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.page-title {
  font-size: 22px;
  font-weight: 600;
  color: #ff6b35;
}

.loading,
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 80px 0;
}

.bookshelf-item {
  position: relative;
}

.continue-btn {
  position: absolute;
  bottom: 12px;
  right: 12px;
}
</style>
