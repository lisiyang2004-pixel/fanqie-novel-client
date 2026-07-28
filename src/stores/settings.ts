import { defineStore } from "pinia";
import { ref, computed } from "vue";

/** 应用设置 store —— 持久化到 localStorage */

const STORAGE_KEY = "fanqie:settings";

export interface AppSettings {
  /** 下载保存目录（空表示使用系统默认下载目录/番茄小说） */
  downloadDir: string;
  /** 默认下载格式 */
  defaultFormat: "txt" | "epub";
  /** 是否在下载完成后自动打开所在目录 */
  autoOpenFolder: boolean;
}

const DEFAULT_SETTINGS: AppSettings = {
  downloadDir: "",
  defaultFormat: "txt",
  autoOpenFolder: false,
};

function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_SETTINGS };
    const parsed = JSON.parse(raw) as Partial<AppSettings>;
    return { ...DEFAULT_SETTINGS, ...parsed };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

function saveSettings(s: AppSettings) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
}

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>(loadSettings());

  const downloadDir = computed(() => settings.value.downloadDir);
  const defaultFormat = computed(() => settings.value.defaultFormat);
  const autoOpenFolder = computed(() => settings.value.autoOpenFolder);

  /** 下载目录显示文本 */
  const downloadDirDisplay = computed(() =>
    settings.value.downloadDir || "默认：系统下载目录/番茄小说"
  );

  /** 是否有自定义下载目录（用于传递给后端） */
  const effectiveOutputDir = computed(() =>
    settings.value.downloadDir || undefined
  );

  function updateSettings(patch: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...patch };
    saveSettings(settings.value);
  }

  function setDownloadDir(dir: string) {
    updateSettings({ downloadDir: dir });
  }

  function setDefaultFormat(fmt: "txt" | "epub") {
    updateSettings({ defaultFormat: fmt });
  }

  function setAutoOpenFolder(val: boolean) {
    updateSettings({ autoOpenFolder: val });
  }

  /** 重置为默认设置 */
  function resetSettings() {
    settings.value = { ...DEFAULT_SETTINGS };
    saveSettings(settings.value);
  }

  return {
    settings,
    downloadDir,
    defaultFormat,
    autoOpenFolder,
    downloadDirDisplay,
    effectiveOutputDir,
    updateSettings,
    setDownloadDir,
    setDefaultFormat,
    setAutoOpenFolder,
    resetSettings,
  };
});
