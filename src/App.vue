<script setup lang="ts">
import { h, ref, computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  NConfigProvider,
  NLayout,
  NLayoutSider,
  NLayoutContent,
  NMenu,
  NIcon,
  NSpace,
  NText,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  NLoadingBarProvider,
  darkTheme,
  zhCN,
  dateZhCN,
  type MenuOption,
} from "naive-ui";
import {
  SearchOutline,
  BookmarksOutline,
  TimeOutline,
  BookOutline,
  SettingsOutline,
} from "@vicons/ionicons5";

const router = useRouter();
const route = useRoute();
const collapsed = ref(false);

const renderIcon = (icon: any) => () => h(NIcon, null, { default: () => h(icon) });

const menuOptions = computed<MenuOption[]>(() => [
  {
    label: "搜索小说",
    key: "search",
    icon: renderIcon(SearchOutline),
  },
  {
    label: "我的书架",
    key: "bookshelf",
    icon: renderIcon(BookmarksOutline),
  },
  {
    label: "下载历史",
    key: "history",
    icon: renderIcon(TimeOutline),
  },
  {
    label: "设置",
    key: "settings",
    icon: renderIcon(SettingsOutline),
  },
]);

const activeKey = computed(() => {
  const name = route.name as string;
  if (name === "book-detail" || name === "reader") return "search";
  return name || "search";
});

function handleMenuSelect(key: string) {
  router.push({ name: key });
}
</script>

<template>
  <NConfigProvider :theme="darkTheme" :locale="zhCN" :date-locale="dateZhCN">
    <NLoadingBarProvider>
      <NDialogProvider>
        <NMessageProvider>
          <NNotificationProvider>
            <NLayout has-sider class="app-layout">
              <NLayoutSider
                bordered
                collapse-mode="width"
                :collapsed-width="64"
                :width="200"
                :collapsed="collapsed"
                show-trigger
                @collapse="collapsed = true"
                @expand="collapsed = false"
                class="app-sider"
              >
                <div class="logo">
                  <NIcon size="28" color="#ff6b35">
                    <BookOutline />
                  </NIcon>
                  <NText v-if="!collapsed" class="logo-text">番茄小说</NText>
                </div>
                <NMenu
                  :collapsed="collapsed"
                  :collapsed-width="64"
                  :collapsed-icon-size="22"
                  :options="menuOptions"
                  :value="activeKey"
                  @update:value="handleMenuSelect"
                />
              </NLayoutSider>

              <NLayout>
                <NLayoutContent class="app-content">
                  <RouterView v-slot="{ Component }">
                    <Transition name="fade" mode="out-in">
                      <component :is="Component" />
                    </Transition>
                  </RouterView>
                </NLayoutContent>
              </NLayout>
            </NLayout>
          </NNotificationProvider>
        </NMessageProvider>
      </NDialogProvider>
    </NLoadingBarProvider>
  </NConfigProvider>
</template>

<style scoped>
.app-layout {
  height: 100vh;
}

.app-sider {
  display: flex;
  flex-direction: column;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px 18px;
  height: 64px;
}

.logo-text {
  font-size: 18px;
  font-weight: 600;
  color: #ff6b35;
}

.app-content {
  height: 100vh;
  overflow-y: auto;
  background-color: #18181c;
}
</style>
