<script setup lang="ts">
import { ref, computed } from "vue";
import {
  NCard,
  NSpace,
  NButton,
  NInput,
  NText,
  NDivider,
  NSelect,
  NSwitch,
  NFormItem,
  NAlert,
  useMessage,
  useDialog,
} from "naive-ui";
import {
  FolderOpenOutline,
  SaveOutline,
  TrashOutline,
  RefreshOutline,
} from "@vicons/ionicons5";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import * as api from "@/api";
import { useSettingsStore } from "@/stores/settings";

const message = useMessage();
const dialog = useDialog();
const settingsStore = useSettingsStore();

// 本地编辑副本（点保存才写入）
const editDir = ref(settingsStore.downloadDir);
const editFormat = ref(settingsStore.defaultFormat);
const editAutoOpen = ref(settingsStore.autoOpenFolder);

const hasChanges = computed(
  () =>
    editDir.value !== settingsStore.downloadDir ||
    editFormat.value !== settingsStore.defaultFormat ||
    editAutoOpen.value !== settingsStore.autoOpenFolder
);

const formatOptions = [
  { label: "TXT 纯文本", value: "txt" },
  { label: "EPUB 电子书", value: "epub" },
];

const defaultDirDisplay = computed(() => {
  if (!editDir.value) return "默认：系统下载目录/番茄小说";
  return editDir.value;
});

// 获取系统默认下载目录（用于显示参考）
const systemDownloadDir = ref<string>("");
async function loadSystemDir() {
  try {
    systemDownloadDir.value = await api.getDefaultDownloadDir();
  } catch {
    systemDownloadDir.value = "";
  }
}
loadSystemDir();

// 选择目录
async function chooseDir() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: "选择下载保存目录",
    });
    if (typeof selected === "string" && selected) {
      editDir.value = selected;
      message.success(`已选择目录: ${selected}`);
    }
  } catch (e: any) {
    message.error(`选择目录失败: ${e}`);
  }
}

// 重置为默认目录
function resetDir() {
  editDir.value = "";
  message.info("已重置为系统默认下载目录");
}

// 保存设置
function saveSettings() {
  settingsStore.setDownloadDir(editDir.value);
  settingsStore.setDefaultFormat(editFormat.value);
  settingsStore.setAutoOpenFolder(editAutoOpen.value);
  message.success("设置已保存");
}

// 重置所有设置
function handleResetAll() {
  dialog.warning({
    title: "重置确认",
    content: "确定要将所有设置恢复为默认值吗？",
    positiveText: "重置",
    negativeText: "取消",
    onPositiveClick: () => {
      settingsStore.resetSettings();
      editDir.value = "";
      editFormat.value = "txt";
      editAutoOpen.value = false;
      message.success("已重置为默认设置");
    },
  });
}

// 打开当前下载目录
async function openCurrentDir() {
  const dir = editDir.value || systemDownloadDir.value;
  if (!dir) {
    message.warning("未配置下载目录");
    return;
  }
  try {
    await api.openInFolder(dir);
  } catch (e: any) {
    message.error(`打开目录失败: ${e}`);
  }
}
</script>

<template>
  <div class="settings-page">
    <h2 class="page-title">设置</h2>

    <!-- 下载设置 -->
    <NCard class="settings-card" title="下载设置">
      <NSpace vertical size="large">
        <!-- 下载目录 -->
        <NFormItem label="下载保存目录" :show-feedback="false">
          <NSpace vertical size="small" style="width: 100%">
            <NSpace align="center" size="small" style="width: 100%">
              <NInput
                :value="defaultDirDisplay"
                readonly
                size="medium"
                style="flex: 1; min-width: 0"
                :placeholder="defaultDirDisplay"
              />
              <NButton type="primary" ghost @click="chooseDir">
                <template #icon>
                  <FolderOpenOutline />
                </template>
                选择目录
              </NButton>
              <NButton quaternary @click="resetDir" :disabled="!editDir">
                默认
              </NButton>
              <NButton quaternary @click="openCurrentDir">
                打开
              </NButton>
            </NSpace>
            <NText depth="3" size="small">
              点击「选择目录」可指定任意盘符（如 D:\、E:\、G:\ 等）下的文件夹作为下载位置。
              留空则使用系统默认下载目录。
            </NText>
            <NText v-if="systemDownloadDir" depth="3" size="tiny">
              系统默认下载目录：{{ systemDownloadDir }}\番茄小说
            </NText>
          </NSpace>
        </NFormItem>

        <NDivider style="margin: 4px 0" />

        <!-- 默认下载格式 -->
        <NFormItem label="默认下载格式" :show-feedback="false">
          <NSpace vertical size="small" style="width: 100%">
            <NSelect
              v-model:value="editFormat"
              :options="formatOptions"
              style="max-width: 240px"
            />
            <NText depth="3" size="small">
              设置下载时默认选中的格式。TXT 适合任意设备阅读；EPUB 适合电子书阅读器。
            </NText>
          </NSpace>
        </NFormItem>

        <NDivider style="margin: 4px 0" />

        <!-- 下载完成自动打开目录 -->
        <NFormItem label="下载完成后自动打开目录" :show-feedback="false">
          <NSpace align="center" size="small">
            <NSwitch v-model:value="editAutoOpen" />
            <NText depth="3" size="small">
              开启后，每次下载完成会自动在文件管理器中打开下载目录。
            </NText>
          </NSpace>
        </NFormItem>
      </NSpace>
    </NCard>

    <!-- 操作按钮 -->
    <NSpace justify="end" size="small" style="margin-top: 16px">
      <NButton @click="handleResetAll">
        <template #icon>
          <RefreshOutline />
        </template>
        重置默认
      </NButton>
      <NButton
        type="primary"
        :disabled="!hasChanges"
        @click="saveSettings"
      >
        <template #icon>
          <SaveOutline />
        </template>
        保存设置
      </NButton>
    </NSpace>

    <!-- 提示 -->
    <NAlert
      v-if="hasChanges"
      type="warning"
      :bordered="false"
      style="margin-top: 16px"
    >
      有未保存的修改，请点击「保存设置」生效。
    </NAlert>

    <!-- 关于 -->
    <NCard class="settings-card" title="关于" style="margin-top: 24px">
      <NSpace vertical size="small">
        <NText>番茄小说下载客户端</NText>
        <NText depth="3" size="small">版本：0.1.0</NText>
        <NText depth="3" size="small">
          基于 Rust + Tauri v2 构建，支持 Windows / macOS / Linux。
        </NText>
        <NDivider style="margin: 8px 0" />
        <NText depth="3" size="tiny">
          数据说明：应用设置保存在浏览器 localStorage 中，仅本机有效。
          书架、阅读进度、下载历史保存在应用数据目录的 SQLite 数据库中。
        </NText>
      </NSpace>
    </NCard>
  </div>
</template>

<style scoped>
.settings-page {
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
}

.page-title {
  font-size: 22px;
  font-weight: 600;
  margin-bottom: 16px;
  color: #ff6b35;
}

.settings-card {
  margin-bottom: 16px;
}
</style>
