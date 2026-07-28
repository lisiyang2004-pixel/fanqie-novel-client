import { createRouter, createWebHistory } from "vue-router";
import type { RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    redirect: "/search",
  },
  {
    path: "/search",
    name: "search",
    component: () => import("@/views/Search.vue"),
    meta: { title: "搜索" },
  },
  {
    path: "/book/:bookId",
    name: "book-detail",
    component: () => import("@/views/BookDetail.vue"),
    props: true,
    meta: { title: "书籍详情" },
  },
  {
    path: "/reader/:bookId/:itemId",
    name: "reader",
    component: () => import("@/views/Reader.vue"),
    props: true,
    meta: { title: "阅读" },
  },
  {
    path: "/bookshelf",
    name: "bookshelf",
    component: () => import("@/views/Bookshelf.vue"),
    meta: { title: "书架" },
  },
  {
    path: "/history",
    name: "history",
    component: () => import("@/views/History.vue"),
    meta: { title: "下载历史" },
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("@/views/Settings.vue"),
    meta: { title: "设置" },
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.afterEach((to) => {
  const title = (to.meta.title as string) || "番茄小说";
  document.title = `${title} - 番茄小说下载客户端`;
});

export default router;
