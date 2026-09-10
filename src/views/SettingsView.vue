<script setup lang="ts">
// 设置中心:个性化(外观/控制台)与系统设置(通用/服务器/网络/开发者)合并为单页
// 左侧竖向 tabbar 使用 cmzya 0.6.2 的 scrollSpy 联动:点击滚动到区块,滚动更新指示
import { ref, computed, onMounted, onUnmounted, onActivated, onDeactivated } from "vue";
import AppearanceCard from "@components/views/paint/AppearanceCard.vue";
import ConsoleSettingsCard from "@components/views/settings/ConsoleSettingsCard.vue";
import GeneralSettingsCard from "@components/views/settings/GeneralSettingsCard.vue";
import ServerDefaultsCard from "@components/views/settings/ServerDefaultsCard.vue";
import NetworkSettingsCard from "@components/views/settings/NetworkSettingsCard.vue";
import DeveloperModeCard from "@components/views/settings/DeveloperModeCard.vue";
import SettingsActions from "@components/views/settings/SettingsActions.vue";
import ImportSettingsModal from "@components/views/settings/ImportSettingsModal.vue";
import ResetConfirmModal from "@components/views/settings/ResetConfirmModal.vue";
import { settingsApi, getSystemFonts, type AppSettings, type SettingsGroup } from "@api/settings";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { handleError } from "@utils/errorHandler";
import {
  applyThemeWithReveal,
  themeRevealTransition,
  applyColors,
  applyMinimalMode,
} from "@utils/theme";
import { usePluginStore } from "@stores/pluginStore";
import { useToast } from "cmzya-modern-ui";
import { useLoading } from "@composables/useAsync";
import {
  dispatchSettingsUpdate,
  SETTINGS_UPDATE_EVENT,
  type SettingsUpdateEvent,
} from "@stores/settingsStore";

const toast = useToast();
const { loading, start: startLoading, stop: stopLoading } = useLoading();

const settings = ref<AppSettings | null>(null);

// 外观/控制台本地值(字符串输入框)
const fontSize = ref("14");
const consoleFontSize = ref("13");
const consoleFontFamily = ref("");
const consoleLetterSpacing = ref("0");
const maxLogLines = ref("5000");
const bgOpacity = ref("0.3");
const bgBlur = ref("0");
const bgBrightness = ref("1.0");
const bgSettingsExpanded = ref(false);

// 服务器默认值本地值
const maxMem = ref("2048");
const minMem = ref("512");
const port = ref("25565");
const defaultRunPath = ref("");

const fontFamilyOptions = ref<{ label: string; value: string }[]>([
  { label: i18n.t("settings.font_family_default"), value: "" },
]);
const fontsLoading = ref(false);

const showImportModal = ref(false);
const showResetConfirm = ref(false);

const pluginStore = usePluginStore();

// 主题被 theme-provider 插件接管时,颜色/主题选择项需要降级提示
const themeProxyPlugin = computed(() => {
  return pluginStore.plugins.find(
    (p) => p.state === "enabled" && pluginStore.hasCapability(p.manifest.id, "theme-provider"),
  );
});
const isThemeProxied = computed(() => !!themeProxyPlugin.value);
const themeProxyPluginName = computed(() => themeProxyPlugin.value?.manifest.name || "");

// 竖向 tabbar 区块定义,key 对应模板里的 data-settings-section
const activeSection = ref("general");

// close_action 在 AppSettings 里是 string,卡片组件要联合类型,这里收口转换
const closeActionModel = computed<"ask" | "minimize" | "close">({
  get: () => (settings.value?.close_action as "ask" | "minimize" | "close") ?? "ask",
  set: (v) => {
    if (settings.value) settings.value.close_action = v;
  },
});

const sectionTabs = computed(() => [
  { key: "general", label: i18n.t("settings.general") },
  { key: "appearance", label: i18n.t("settings.appearance") },
  { key: "console", label: i18n.t("settings.console") },
  { key: "server", label: i18n.t("settings.server_defaults") },
  { key: "network", label: i18n.t("settings.network") },
  { key: "developer", label: i18n.t("settings.developer_mode") },
  { key: "actions", label: i18n.t("settings.actions") },
]);

onMounted(async () => {
  await loadSettings();
  await loadSystemFonts();
});

// 监听器集中到 activated/deactivated 成对注册,避免 keep-alive 下重复监听
onActivated(() => {
  window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
});

onDeactivated(() => {
  window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
});

onUnmounted(() => {
  // 未被 deactivated 直接销毁时的兜底清理
  window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
  if (saveTimeout) {
    clearTimeout(saveTimeout);
    saveTimeout = null;
  }
});

function handleSettingsUpdateEvent(e: CustomEvent<SettingsUpdateEvent>) {
  settings.value = e.detail.settings;
  syncLocalValues(e.detail.settings);
}

function syncLocalValues(s: AppSettings) {
  fontSize.value = String(s.font_size);
  consoleFontSize.value = String(s.console_font_size);
  consoleFontFamily.value = s.console_font_family || "";
  consoleLetterSpacing.value = String(s.console_letter_spacing ?? 0);
  maxLogLines.value = String(s.max_log_lines);
  bgOpacity.value = String(s.background_opacity);
  bgBlur.value = String(s.background_blur);
  bgBrightness.value = String(s.background_brightness);
  maxMem.value = String(s.default_max_memory);
  minMem.value = String(s.default_min_memory);
  port.value = String(s.default_port);
  defaultRunPath.value = s.last_run_path || "";
}

async function loadSystemFonts() {
  fontsLoading.value = true;
  try {
    const fonts = await getSystemFonts();
    fontFamilyOptions.value = [
      { label: i18n.t("settings.font_family_default"), value: "" },
      ...fonts.map((font) => ({ label: font, value: `'${font}'` })),
    ];
  } catch (e) {
    console.error("Failed to load system fonts:", e);
  } finally {
    fontsLoading.value = false;
  }
}

async function loadSettings() {
  startLoading();
  try {
    const s = await settingsApi.get();
    settings.value = s;
    syncLocalValues(s);
    settings.value.color = s.color || "default";
    applyTheme(s.theme);
    applyFontSize(s.font_size);
    applyFontFamily(s.font_family);
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    stopLoading();
  }
}

function markChanged() {
  debouncedSave();
}

let saveTimeout: ReturnType<typeof setTimeout> | null = null;

function debouncedSave() {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
  }
  saveTimeout = setTimeout(() => {
    saveSettings();
    saveTimeout = null;
  }, 500);
}

function getEffectiveTheme(theme: string): "light" | "dark" {
  if (theme === "auto") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return theme as "light" | "dark";
}

function applyTheme(theme: string) {
  const effectiveTheme = getEffectiveTheme(theme);
  document.documentElement.setAttribute("data-theme", effectiveTheme);
  return effectiveTheme;
}

function applyFontSize(size: number) {
  document.documentElement.style.fontSize = `${size}px`;
}

function handleFontSizeChange() {
  markChanged();
  const size = parseInt(fontSize.value) || 14;
  applyFontSize(size);
}

function applyFontFamily(fontFamily: string) {
  if (fontFamily) {
    document.documentElement.style.setProperty("--sl-font-sans", fontFamily);
    document.documentElement.style.setProperty("--sl-font-display", fontFamily);
  } else {
    document.documentElement.style.removeProperty("--sl-font-sans");
    document.documentElement.style.removeProperty("--sl-font-display");
  }
}

function handleFontFamilyChange() {
  markChanged();
  if (settings.value) {
    applyFontFamily(settings.value.font_family);
  }
}

function handleAcrylicChange() {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
    saveTimeout = null;
  }
  void saveSettings();
}

function handleMinimalModeChange(enabled: boolean) {
  markChanged();
  applyMinimalMode(enabled);
}

function handleThemeChange(origin?: { x: number; y: number }) {
  markChanged();
  if (!settings.value) return;

  // 以下拉触发器为圆心做圆形扩散,主题没变时不播动画
  const x = origin?.x ?? window.innerWidth / 2;
  const y = origin?.y ?? window.innerHeight / 2;
  applyThemeWithReveal(settings.value.theme, x, y, () => {
    applyTheme(settings.value!.theme);
    applyColors(settings.value!);
  });
}

function handleColorChange(origin?: { x: number; y: number }) {
  markChanged();
  if (!settings.value) return;

  // 主题色切换同样以下拉触发器为圆心做圆形扩散
  const x = origin?.x ?? window.innerWidth / 2;
  const y = origin?.y ?? window.innerHeight / 2;
  themeRevealTransition(x, y, () => {
    applyColors(settings.value!);
  });
}

function handleJavaInstalled(path: string) {
  if (settings.value) {
    settings.value.default_java_path = path;
    markChanged();
  }
}

async function handleBrowseJavaPath() {
  const selected = await systemApi.pickJavaFile();
  if (selected && settings.value) {
    settings.value.default_java_path = selected;
    markChanged();
  }
}

async function handleBrowseRunPath() {
  const selected = await systemApi.pickFolder();
  if (selected) {
    defaultRunPath.value = selected;
    markChanged();
  }
}

async function pickBackgroundImage() {
  try {
    const result = await systemApi.pickImageFile();
    if (result && settings.value) {
      settings.value.background_image = result;
      markChanged();
    }
  } catch (e) {
    console.error("Pick image error:", e);
    toast.error(handleError(e));
  }
}

function clearBackgroundImage() {
  if (settings.value) {
    settings.value.background_image = "";
    markChanged();
  }
}

async function saveSettings() {
  if (!settings.value) return;

  // 字符串输入框回写数字字段
  settings.value.console_font_size = parseInt(consoleFontSize.value) || 13;
  settings.value.console_font_family = consoleFontFamily.value;
  settings.value.console_letter_spacing = parseInt(consoleLetterSpacing.value) || 0;
  settings.value.max_log_lines = parseInt(maxLogLines.value) || 5000;
  settings.value.background_opacity = parseFloat(bgOpacity.value) || 0.3;
  settings.value.background_blur = parseInt(bgBlur.value) || 0;
  settings.value.background_brightness = parseFloat(bgBrightness.value) || 1.0;
  settings.value.font_size = parseInt(fontSize.value) || 14;
  settings.value.default_max_memory = parseInt(maxMem.value) || 2048;
  settings.value.default_min_memory = parseInt(minMem.value) || 512;
  settings.value.default_port = parseInt(port.value) || 25565;
  settings.value.last_run_path = defaultRunPath.value;
  settings.value.color = settings.value.color || "default";
  settings.value.developer_mode = settings.value.developer_mode || false;

  try {
    const result = await settingsApi.saveWithDiff(settings.value);

    localStorage.setItem(
      "sl_theme_cache",
      JSON.stringify({
        theme: settings.value.theme || "auto",
        fontSize: settings.value.font_size || 14,
      }),
    );

    if (result.changed_groups.includes("Appearance")) {
      applyTheme(settings.value.theme);
      applyFontSize(settings.value.font_size);
      applyFontFamily(settings.value.font_family);
    }

    dispatchSettingsUpdate(result.changed_groups, result.settings);
  } catch (e) {
    toast.error(handleError(e));
  }
}

function applyAllAppearance(s: AppSettings) {
  applyTheme(s.theme);
  applyFontSize(s.font_size);
  applyFontFamily(s.font_family);
}

function cacheTheme(s: AppSettings) {
  localStorage.setItem(
    "sl_theme_cache",
    JSON.stringify({
      theme: s.theme || "auto",
      fontSize: s.font_size || 14,
    }),
  );
}

async function resetSettings() {
  try {
    const s = await settingsApi.reset();
    settings.value = s;
    syncLocalValues(s);
    showResetConfirm.value = false;
    settings.value.color = "default";

    cacheTheme(s);
    applyAllAppearance(s);
    dispatchSettingsUpdate(["Appearance"] as SettingsGroup[], s);
  } catch (e) {
    toast.error(handleError(e));
  }
}

async function exportSettings() {
  try {
    const json = await settingsApi.exportJson();
    await navigator.clipboard.writeText(json);
    toast.success(i18n.t("settings.export_success"));
  } catch (e) {
    toast.error(handleError(e));
  }
}

async function handleImport(json: string) {
  if (!json.trim()) {
    toast.error(i18n.t("common.paste_json"));
    return;
  }
  try {
    const s = await settingsApi.importJson(json);
    settings.value = s;
    syncLocalValues(s);
    showImportModal.value = false;
    applyAllAppearance(s);
    dispatchSettingsUpdate(["Appearance"] as SettingsGroup[], s);
  } catch (e) {
    toast.error(handleError(e));
  }
}
</script>

<template>
  <div class="settings-view animate-stagger-in">
    <div v-if="loading" class="loading-state">
      <cmz-spinner />
      <span>{{ i18n.t("settings.loading") }}</span>
    </div>

    <div v-else-if="settings" class="settings-layout">
      <!-- 竖向 tabbar:scrollSpy 指示随 .app-content 滚动联动,点击滚到对应区块 -->
      <div class="settings-tabbar-sticky">
        <cmz-tab-bar
          v-model="activeSection"
          :tabs="sectionTabs"
          :level="1"
          vertical
          scroll-spy
          scroll-container=".app-content"
          :scroll-offset="24"
          section-selector="[data-settings-section='{key}']"
        />
      </div>

      <div class="settings-main">
        <section data-settings-section="general" class="settings-section">
          <GeneralSettingsCard
            v-model:close-servers-on-exit="settings.close_servers_on_exit"
            v-model:close-servers-on-update="settings.close_servers_on_update"
            v-model:auto-accept-eula="settings.auto_accept_eula"
            v-model:auto-lightweight-minutes="settings.auto_lightweight_minutes"
            v-model:close-action="closeActionModel"
            @change="markChanged"
          />
        </section>

        <section data-settings-section="appearance" class="settings-section">
          <AppearanceCard
            :color="settings.color"
            :theme="settings.theme"
            :font-size="fontSize"
            :font-family="settings.font_family"
            :font-family-options="fontFamilyOptions"
            :fonts-loading="fontsLoading"
            :acrylic-enabled="settings.acrylic_enabled"
            :is-theme-proxied="isThemeProxied"
            :theme-proxy-plugin-name="themeProxyPluginName"
            :background-image="settings.background_image"
            :bg-opacity="bgOpacity"
            :bg-blur="bgBlur"
            :bg-brightness="bgBrightness"
            :background-size="settings.background_size"
            :bg-settings-expanded="bgSettingsExpanded"
            :minimal-mode="settings.minimal_mode"
            @update:color="settings.color = $event"
            @update:theme="settings.theme = $event"
            @update:font-size="fontSize = $event"
            @update:font-family="settings.font_family = $event"
            @update:acrylic-enabled="settings.acrylic_enabled = $event"
            @update:bg-settings-expanded="bgSettingsExpanded = $event"
            @update:bg-opacity="bgOpacity = $event"
            @update:bg-blur="bgBlur = $event"
            @update:bg-brightness="bgBrightness = $event"
            @update:background-size="settings.background_size = $event"
            @update:minimal-mode="settings.minimal_mode = $event"
            @theme-change="handleThemeChange"
            @color-change="handleColorChange"
            @font-size-change="handleFontSizeChange"
            @font-family-change="handleFontFamilyChange"
            @acrylic-change="handleAcrylicChange"
            @minimal-mode-change="handleMinimalModeChange"
            @pick-image="pickBackgroundImage"
            @clear-image="clearBackgroundImage"
            @change="markChanged"
          />
        </section>

        <section data-settings-section="console" class="settings-section">
          <ConsoleSettingsCard
            v-model:console-font-size="consoleFontSize"
            v-model:console-font-family="consoleFontFamily"
            v-model:console-letter-spacing="consoleLetterSpacing"
            v-model:max-log-lines="maxLogLines"
            :font-family-options="fontFamilyOptions"
            :fonts-loading="fontsLoading"
            @change="markChanged"
          />
        </section>

        <section data-settings-section="server" class="settings-section">
          <ServerDefaultsCard
            v-model:max-memory="maxMem"
            v-model:min-memory="minMem"
            v-model:port="port"
            v-model:default-java-path="settings.default_java_path"
            v-model:default-jvm-args="settings.default_jvm_args"
            v-model:default-run-path="defaultRunPath"
            @change="markChanged"
            @javaInstalled="handleJavaInstalled"
            @browseJavaPath="handleBrowseJavaPath"
            @browseRunPath="handleBrowseRunPath"
          />
        </section>

        <section data-settings-section="network" class="settings-section">
          <NetworkSettingsCard :proxy="settings.proxy" />
        </section>

        <section data-settings-section="developer" class="settings-section">
          <DeveloperModeCard
            v-model:developer-mode="settings.developer_mode"
            @change="markChanged"
          />
        </section>

        <section data-settings-section="actions" class="settings-section">
          <SettingsActions
            @export="exportSettings"
            @import="showImportModal = true"
            @reset="showResetConfirm = true"
          />
        </section>
      </div>
    </div>

    <ImportSettingsModal v-model:visible="showImportModal" @import="handleImport" />

    <ResetConfirmModal v-model:visible="showResetConfirm" @confirm="resetSettings" />
  </div>
</template>

<style scoped>
.settings-view {
  padding-bottom: var(--sl-space-2xl);
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}

.settings-layout {
  display: flex;
  align-items: flex-start;
  gap: 0;
}

/* 竖 tabbar 吸顶,跟随内容滚动;宽度由 app.css 全局统一 */
.settings-tabbar-sticky {
  position: sticky;
  top: var(--sl-space-md);
  flex-shrink: 0;
  z-index: 1;
}

.settings-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
  /* 点击 tab 滚动定位时与容器顶部留出呼吸空间 */
  scroll-margin-top: var(--sl-space-md);
}
</style>
