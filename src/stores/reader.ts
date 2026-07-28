import { defineStore } from "pinia";
import { ref, watch } from "vue";

export type ReaderTheme = "light" | "sepia" | "dark";

interface ReaderSettings {
  fontSize: number;
  lineHeight: number;
  theme: ReaderTheme;
  fontFamily: string;
}

const STORAGE_KEY = "fanqie-reader-settings";

const defaultSettings: ReaderSettings = {
  fontSize: 18,
  lineHeight: 1.8,
  theme: "sepia",
  fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif",
};

function loadSettings(): ReaderSettings {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      return { ...defaultSettings, ...JSON.parse(saved) };
    }
  } catch {
    // ignore
  }
  return defaultSettings;
}

export const useReaderStore = defineStore("reader", () => {
  const settings = ref<ReaderSettings>(loadSettings());

  watch(
    settings,
    (val) => {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
    },
    { deep: true }
  );

  function setFontSize(size: number) {
    settings.value.fontSize = Math.max(12, Math.min(32, size));
  }

  function increaseFont() {
    setFontSize(settings.value.fontSize + 2);
  }

  function decreaseFont() {
    setFontSize(settings.value.fontSize - 2);
  }

  function setLineHeight(height: number) {
    settings.value.lineHeight = Math.max(1.2, Math.min(3, height));
  }

  function setTheme(theme: ReaderTheme) {
    settings.value.theme = theme;
  }

  function getThemeColors() {
    switch (settings.value.theme) {
      case "light":
        return { bg: "#ffffff", text: "#333333" };
      case "sepia":
        return { bg: "#f5edd6", text: "#5b4636" };
      case "dark":
        return { bg: "#1a1a1a", text: "#c8c8c8" };
    }
  }

  return {
    settings,
    setFontSize,
    increaseFont,
    decreaseFont,
    setLineHeight,
    setTheme,
    getThemeColors,
  };
});
