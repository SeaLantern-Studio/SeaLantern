import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { ColorScheme, ColorMode, CustomColorScheme } from "@/types/common";

export const useUiStore = defineStore("ui", () => {
  const sidebarCollapsed = ref(false);
  const currentRoute = ref("home");
  const colorScheme = ref<ColorScheme>("blue");
  const colorMode = ref<ColorMode>("auto");
  const customColorScheme = ref<CustomColorScheme | null>(null);

  const isDarkMode = computed(() => {
    if (colorMode.value === "auto") {
      return window.matchMedia("(prefers-color-scheme: dark)").matches;
    }
    return colorMode.value === "dark";
  });

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function setCurrentRoute(route: string) {
    currentRoute.value = route;
  }

  function setColorScheme(scheme: ColorScheme, custom: CustomColorScheme | null = null) {
    colorScheme.value = scheme;
    customColorScheme.value = custom;
    applyColorScheme(scheme, custom);
  }

  function setColorMode(mode: ColorMode) {
    colorMode.value = mode;
    applyColorMode(mode);
  }

  function applyColorScheme(scheme: ColorScheme, custom: CustomColorScheme | null = null) {
    const doc = document.documentElement;
    doc.setAttribute("data-color-scheme", scheme);
    
    if (scheme === "custom" && custom) {
      applyCustomColorScheme(custom);
    } else {
      clearCustomColorScheme();
    }
  }

  function applyColorMode(mode: ColorMode) {
    const doc = document.documentElement;
    const effectiveMode = mode === "auto" 
      ? (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light")
      : mode;
    doc.setAttribute("data-color-mode", effectiveMode);
  }

  function applyCustomColorScheme(colors: CustomColorScheme) {
    const doc = document.documentElement;
    doc.style.setProperty("--custom-primary", colors.primary);
    doc.style.setProperty("--custom-primary-light", colors.primaryLight);
    doc.style.setProperty("--custom-primary-dark", colors.primaryDark);
    doc.style.setProperty("--custom-primary-bg", colors.primaryBg);
    doc.style.setProperty("--custom-accent", colors.accent);
    doc.style.setProperty("--custom-accent-light", colors.accentLight);
    doc.style.setProperty("--custom-success", colors.success);
    doc.style.setProperty("--custom-warning", colors.warning);
    doc.style.setProperty("--custom-error", colors.error);
    doc.style.setProperty("--custom-info", colors.info);
  }

  function clearCustomColorScheme() {
    const doc = document.documentElement;
    doc.style.removeProperty("--custom-primary");
    doc.style.removeProperty("--custom-primary-light");
    doc.style.removeProperty("--custom-primary-dark");
    doc.style.removeProperty("--custom-primary-bg");
    doc.style.removeProperty("--custom-accent");
    doc.style.removeProperty("--custom-accent-light");
    doc.style.removeProperty("--custom-success");
    doc.style.removeProperty("--custom-warning");
    doc.style.removeProperty("--custom-error");
    doc.style.removeProperty("--custom-info");
  }

  function initTheme(scheme: ColorScheme, mode: ColorMode, custom: CustomColorScheme | null = null) {
    colorScheme.value = scheme;
    colorMode.value = mode;
    customColorScheme.value = custom;
    applyColorScheme(scheme, custom);
    applyColorMode(mode);
  }

  return {
    sidebarCollapsed,
    currentRoute,
    colorScheme,
    colorMode,
    customColorScheme,
    isDarkMode,
    toggleSidebar,
    setCurrentRoute,
    setColorScheme,
    setColorMode,
    applyColorScheme,
    applyColorMode,
    applyCustomColorScheme,
    clearCustomColorScheme,
    initTheme,
  };
});
