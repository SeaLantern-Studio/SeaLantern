<script setup lang="ts">
import { ref, onMounted, watch, computed } from "vue";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLInput from "../components/common/SLInput.vue";
import SLSwitch from "../components/common/SLSwitch.vue";
import SLModal from "../components/common/SLModal.vue";
import SLSelect from "../components/common/SLSelect.vue";
import SLSpinner from "../components/common/SLSpinner.vue";
import {
  settingsApi,
  checkAcrylicSupport,
  applyAcrylic,
  getSystemFonts,
  type AppSettings,
} from "../api/settings";
import { systemApi } from "../api/system";
import { convertFileSrc } from "@tauri-apps/api/core";
import { i18n } from "../locales";

const settings = ref<AppSettings | null>(null);
const loading = ref(true);
const fontsLoading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);

// 亚克力支持检测
const acrylicSupported = ref(true);

// String versions for number inputs (avoids v-model type mismatch)
const maxMem = ref("2048");
const minMem = ref("512");
const port = ref("25565");
const fontSize = ref("13");
const logLines = ref("5000");
const bgOpacity = ref("0.3");
const bgBlur = ref("0");
const bgBrightness = ref("1.0");
const uiFontSize = ref("14");

const backgroundSizeOptions = computed(() => [
  { label: i18n.t("common.background_size_cover"), value: "cover" },
  { label: i18n.t("common.background_size_contain"), value: "contain" },
  { label: i18n.t("common.background_size_fill"), value: "fill" },
  { label: i18n.t("common.background_size_auto"), value: "auto" },
]);

const colorOptions = computed(() => [
  { label: i18n.t("common.color_default"), value: "default" },
  { label: "Midnight", value: "midnight" },
  { label: "Sunset", value: "sunset" },
  { label: "Ocean", value: "ocean" },
  { label: "Rose", value: "rose" },
  { label: i18n.t("common.color_custom"), value: "custom" },
]);

const editColorOptions = computed(() => [
  { label: i18n.t("common.edit_color_light"), value: "light" },
  { label: i18n.t("common.edit_color_dark"), value: "dark" },
  { label: i18n.t("common.edit_color_light_acrylic"), value: "light_acrylic" },
  { label: i18n.t("common.edit_color_dark_acrylic"), value: "dark_acrylic" },
]);

const themeOptions = computed(() => [
  { label: i18n.t("common.theme_auto"), value: "auto" },
  { label: i18n.t("common.theme_light"), value: "light" },
  { label: i18n.t("common.theme_dark"), value: "dark" },
]);

const fontFamilyOptions = ref<{ label: string; value: string }[]>([
  { label: i18n.t("common.font_system_default"), value: "" },
]);

const showImportModal = ref(false);
const importJson = ref("");
const showResetConfirm = ref(false);
const bgSettingsExpanded = ref(false);
const colorSettingsExpanded = ref(false);
const bgPreviewLoaded = ref(false);
const bgPreviewLoading = ref(false);

const backgroundPreviewUrl = computed(() => {
  if (!settings.value?.background_image) return "";
  if (!bgSettingsExpanded.value) return "";
  return convertFileSrc(settings.value.background_image);
});

function getFileExtension(path: string): string {
  return path.split(".").pop()?.toLowerCase() || "";
}

function isAnimatedImage(path: string): boolean {
  const ext = getFileExtension(path);
  return ext === "gif" || ext === "webp" || ext === "apng";
}

onMounted(async () => {
  await loadSettings();
  await loadSystemFonts();
  // 检测亚克力支持
  try {
    acrylicSupported.value = await checkAcrylicSupport();
  } catch {
    acrylicSupported.value = false;
  }
});

async function loadSystemFonts() {
  fontsLoading.value = true;
  try {
    const fonts = await getSystemFonts();
    fontFamilyOptions.value = [
      { label: i18n.t("common.font_system_default"), value: "" },
      ...fonts.map((font) => ({ label: font, value: `'${font}'` })),
    ];
  } catch (e) {
    console.error("Failed to load system fonts:", e);
  } finally {
    fontsLoading.value = false;
  }
}

watch(bgSettingsExpanded, (expanded) => {
  if (expanded && settings.value?.background_image) {
    bgPreviewLoaded.value = false;
    bgPreviewLoading.value = true;
  }
});

async function loadSettings() {
  loading.value = true;
  error.value = null;
  try {
    const s = await settingsApi.get();
    settings.value = s;
    maxMem.value = String(s.default_max_memory);
    minMem.value = String(s.default_min_memory);
    port.value = String(s.default_port);
    fontSize.value = String(s.console_font_size);
    logLines.value = String(s.max_log_lines);
    bgOpacity.value = String(s.background_opacity);
    bgBlur.value = String(s.background_blur);
    bgBrightness.value = String(s.background_brightness);
    uiFontSize.value = String(s.font_size);
    settings.value.color = s.color || "default";
    // 应用已保存的设置
    applyTheme(s.theme);
    applyFontSize(s.font_size);
    applyFontFamily(s.font_family);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function markChanged() {
  saveSettings();
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
  const size = parseInt(uiFontSize.value) || 14;
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

async function handleAcrylicChange(enabled: boolean) {
  markChanged();
  document.documentElement.setAttribute("data-acrylic", enabled ? "true" : "false");

  if (!acrylicSupported.value) {
    return;
  }

  try {
    const theme = settings.value?.theme || "auto";
    const isDark = getEffectiveTheme(theme) === "dark";
    await applyAcrylic(enabled, isDark);
  } catch (e) {
    error.value = String(e);
  }
}

async function handleThemeChange() {
  markChanged();
  if (!settings.value) return;

  const effectiveTheme = applyTheme(settings.value.theme);

  if (settings.value.acrylic_enabled && acrylicSupported.value) {
    try {
      const isDark = effectiveTheme === "dark";
      await applyAcrylic(true, isDark);
    } catch {}
  }
}

async function saveSettings() {
  if (!settings.value) return;

  settings.value.default_max_memory = parseInt(maxMem.value) || 2048;
  settings.value.default_min_memory = parseInt(minMem.value) || 512;
  settings.value.default_port = parseInt(port.value) || 25565;
  settings.value.console_font_size = parseInt(fontSize.value) || 13;
  settings.value.max_log_lines = parseInt(logLines.value) || 5000;
  settings.value.background_opacity = parseFloat(bgOpacity.value) || 0.3;
  settings.value.background_blur = parseInt(bgBlur.value) || 0;
  settings.value.background_brightness = parseFloat(bgBrightness.value) || 1.0;
  settings.value.font_size = parseInt(uiFontSize.value) || 14;
  settings.value.color = settings.value.color || "default";
  settings.value.developer_mode = settings.value.developer_mode || false;

  saving.value = true;
  error.value = null;
  try {
    await settingsApi.save(settings.value);

    localStorage.setItem(
      "sl_theme_cache",
      JSON.stringify({
        theme: settings.value.theme || "auto",
        fontSize: settings.value.font_size || 14,
      }),
    );

    applyTheme(settings.value.theme);
    applyFontSize(settings.value.font_size);

    if (acrylicSupported.value) {
      try {
        const isDark = getEffectiveTheme(settings.value.theme) === "dark";
        await applyAcrylic(settings.value.acrylic_enabled, isDark);
      } catch {}
    }

    window.dispatchEvent(new CustomEvent("settings-updated"));
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

async function resetSettings() {
  try {
    const s = await settingsApi.reset();
    settings.value = s;
    maxMem.value = String(s.default_max_memory);
    minMem.value = String(s.default_min_memory);
    port.value = String(s.default_port);
    fontSize.value = String(s.console_font_size);
    logLines.value = String(s.max_log_lines);
    bgOpacity.value = String(s.background_opacity);
    bgBlur.value = String(s.background_blur);
    bgBrightness.value = String(s.background_brightness);
    uiFontSize.value = String(s.font_size);
    showResetConfirm.value = false;
    settings.value.color = "default";

    localStorage.setItem(
      "sl_theme_cache",
      JSON.stringify({
        theme: s.theme || "auto",
        fontSize: s.font_size || 14,
      }),
    );

    applyTheme(s.theme);
    applyFontSize(s.font_size);
    applyFontFamily(s.font_family);
  } catch (e) {
    error.value = String(e);
  }
}

async function exportSettings() {
  try {
    const json = await settingsApi.exportJson();
    await navigator.clipboard.writeText(json);
  } catch (e) {
    error.value = String(e);
  }
}

async function handleImport() {
  if (!importJson.value.trim()) {
    error.value = i18n.t("common.paste_json");
    return;
  }
  try {
    const s = await settingsApi.importJson(importJson.value);
    settings.value = s;
    maxMem.value = String(s.default_max_memory);
    minMem.value = String(s.default_min_memory);
    port.value = String(s.default_port);
    fontSize.value = String(s.console_font_size);
    logLines.value = String(s.max_log_lines);
    bgOpacity.value = String(s.background_opacity);
    bgBlur.value = String(s.background_blur);
    bgBrightness.value = String(s.background_brightness);
    uiFontSize.value = String(s.font_size);
    showImportModal.value = false;
    importJson.value = "";
    applyTheme(s.theme);
    applyFontSize(s.font_size);
    applyFontFamily(s.font_family);
  } catch (e) {
    error.value = String(e);
  }
}

async function pickBackgroundImage() {
  try {
    const result = await systemApi.pickImageFile();
    console.log("Selected image:", result);
    if (result && settings.value) {
      settings.value.background_image = result;
      markChanged();
    }
  } catch (e) {
    console.error("Pick image error:", e);
    error.value = String(e);
  }
}

function clearBackgroundImage() {
  if (settings.value) {
    settings.value.background_image = "";
    markChanged();
  }
}

function handleDeveloperModeChange() {
  markChanged();
}
</script>

<template>
  <div class="settings-view animate-fade-in-up">
    <div v-if="error" class="msg-banner error-banner">
      <span>{{ error }}</span>
      <button @click="error = null">x</button>
    </div>

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>{{ i18n.t("settings.loading") }}</span>
    </div>

    <template v-else-if="settings">
      <!-- General -->
      <SLCard :title="i18n.t('settings.general')" :subtitle="i18n.t('settings.general_desc')">
        <div class="settings-group">
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.auto_stop") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.auto_stop_desc") }}</span>
            </div>
            <SLSwitch v-model="settings.close_servers_on_exit" @update:modelValue="markChanged" />
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.auto_eula") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.auto_eula_desc") }}</span>
            </div>
            <SLSwitch v-model="settings.auto_accept_eula" @update:modelValue="markChanged" />
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.close_action") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.close_action_desc") }}</span>
            </div>
            <div class="input-md">
              <SLSelect
                v-model="settings.close_action"
                :options="[
                  { label: i18n.t('settings.close_action_ask'), value: 'ask' },
                  { label: i18n.t('settings.close_action_minimize'), value: 'minimize' },
                  { label: i18n.t('settings.close_action_close'), value: 'close' },
                ]"
                @update:modelValue="markChanged"
              />
            </div>
          </div>
        </div>
      </SLCard>

      <!-- Server Defaults -->
      <SLCard
        :title="i18n.t('settings.server_defaults')"
        :subtitle="i18n.t('settings.server_defaults_desc')"
      >
        <div class="settings-group">
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.default_memory") }} (MB)</span>
              <span class="setting-desc">{{ i18n.t("settings.max_memory_desc") }}</span>
            </div>
            <div class="input-sm">
              <SLInput v-model="maxMem" type="number" @update:modelValue="markChanged" />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.min_memory") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.min_memory_desc") }}</span>
            </div>
            <div class="input-sm">
              <SLInput v-model="minMem" type="number" @update:modelValue="markChanged" />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.default_port") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.port_desc") }}</span>
            </div>
            <div class="input-sm">
              <SLInput v-model="port" type="number" @update:modelValue="markChanged" />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.default_java") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.default_java_desc") }}</span>
            </div>
            <div class="input-lg">
              <SLInput
                v-model="settings.default_java_path"
                :placeholder="i18n.t('settings.default_java_desc')"
                @update:modelValue="markChanged"
              />
            </div>
          </div>

          <div class="setting-row full-width">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.jvm_args") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.jvm_args_desc") }}</span>
            </div>
            <textarea
              class="jvm-textarea"
              v-model="settings.default_jvm_args"
              :placeholder="i18n.t('settings.jvm_args_placeholder')"
              rows="3"
              @input="markChanged"
            ></textarea>
          </div>
        </div>
      </SLCard>

      <!-- Console -->
      <SLCard :title="i18n.t('settings.console')" :subtitle="i18n.t('settings.console_desc')">
        <div class="settings-group">
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.console_font_size") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.console_font_size_desc") }}</span>
            </div>
            <div class="input-sm">
              <SLInput v-model="fontSize" type="number" @update:modelValue="markChanged" />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.max_log_lines") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.max_log_lines_desc") }}</span>
            </div>
            <div class="input-sm">
              <SLInput v-model="logLines" type="number" @update:modelValue="markChanged" />
            </div>
          </div>
        </div>
      </SLCard>

      <!-- Developer Mode -->
      <SLCard
        :title="i18n.t('settings.developer_mode')"
        :subtitle="i18n.t('settings.developer_mode_desc')"
      >
        <div class="settings-group">
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.developer_mode_toggle") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.developer_mode_toggle_desc") }}</span>
            </div>
            <SLSwitch
              v-model="settings.developer_mode"
              @update:modelValue="handleDeveloperModeChange"
            />
          </div>
        </div>
      </SLCard>

      <!-- Actions -->
      <div class="settings-actions">
        <div class="actions-left"></div>
        <div class="actions-right">
          <SLButton variant="ghost" size="sm" @click="exportSettings">{{
            i18n.t("settings.export")
          }}</SLButton>
          <SLButton variant="ghost" size="sm" @click="showImportModal = true">{{
            i18n.t("settings.import")
          }}</SLButton>
          <SLButton variant="danger" size="sm" @click="showResetConfirm = true">{{
            i18n.t("settings.reset")
          }}</SLButton>
        </div>
      </div>
    </template>

    <SLModal
      :visible="showImportModal"
      :title="i18n.t('settings.import_title')"
      @close="showImportModal = false"
    >
      <div class="import-form">
        <p class="text-caption">{{ i18n.t("settings.import_desc") }}</p>
        <textarea
          class="import-textarea"
          v-model="importJson"
          :placeholder="i18n.t('settings.import_placeholder')"
          rows="10"
        ></textarea>
      </div>
      <template #footer>
        <SLButton variant="secondary" @click="showImportModal = false">{{
          i18n.t("settings.cancel")
        }}</SLButton>
        <SLButton variant="primary" @click="handleImport">{{ i18n.t("settings.import") }}</SLButton>
      </template>
    </SLModal>

    <SLModal
      :visible="showResetConfirm"
      :title="i18n.t('settings.reset_title')"
      @close="showResetConfirm = false"
    >
      <p class="text-body">{{ i18n.t("settings.reset_desc") }}</p>
      <template #footer>
        <SLButton variant="secondary" @click="showResetConfirm = false">{{
          i18n.t("settings.cancel")
        }}</SLButton>
        <SLButton variant="danger" @click="resetSettings">{{
          i18n.t("settings.reset_confirm")
        }}</SLButton>
      </template>
    </SLModal>
  </div>
</template>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
  max-width: 860px;
  margin: 0 auto;
  padding-bottom: var(--sl-space-2xl);
}

.msg-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
}
.error-banner {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: var(--sl-error);
}
.msg-banner button {
  font-weight: 600;
  color: inherit;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}

.settings-group {
  display: flex;
  flex-direction: column;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md) 0;
  border-bottom: 1px solid var(--sl-border-light);
  gap: var(--sl-space-lg);
}
.setting-row:last-child {
  border-bottom: none;
}
.setting-row.full-width {
  flex-direction: column;
  align-items: stretch;
}

.setting-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.setting-label {
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}
.setting-desc {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
  line-height: 1.4;
}

.input-sm {
  width: 120px;
  flex-shrink: 0;
}
.input-md {
  width: 200px;
  flex-shrink: 0;
}
.input-lg {
  width: 320px;
  flex-shrink: 0;
}

.jvm-textarea,
.import-textarea {
  width: 100%;
  margin-top: var(--sl-space-sm);
  padding: var(--sl-space-sm) var(--sl-space-md);
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  color: var(--sl-text-primary);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  resize: vertical;
  line-height: 1.6;
}
.jvm-textarea:focus,
.import-textarea:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px var(--sl-primary-bg);
  outline: none;
}

.settings-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md) 0;
  border-top: 1px solid var(--sl-border);
}
.actions-left,
.actions-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.import-form {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.bg-image-picker {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
}

.bg-preview {
  position: relative;
  width: 100%;
  max-width: 400px;
  height: 200px;
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  border: 1px solid var(--sl-border);
}

.bg-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.bg-preview-loading {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  background: var(--sl-surface);
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: sl-spin 1s linear infinite;
}

.bg-animated-badge {
  position: absolute;
  top: var(--sl-space-sm);
  right: var(--sl-space-sm);
  padding: 2px 8px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  font-size: 0.75rem;
  font-weight: 500;
  border-radius: var(--sl-radius-sm);
}

.bg-preview-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-sm);
}

.bg-preview-path {
  font-size: 0.8125rem;
  color: white;
  font-family: var(--sl-font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.slider-control {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  min-width: 200px;
}

.sl-slider {
  flex: 1;
  height: 6px;
  border-radius: var(--sl-radius-full);
  background: var(--sl-border);
  outline: none;
  -webkit-appearance: none;
}

.sl-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.sl-slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.sl-slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  cursor: pointer;
  border: none;
  transition: all var(--sl-transition-fast);
}

.sl-slider::-moz-range-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.slider-value {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  min-width: 50px;
  text-align: right;
}

.collapsible-section {
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  margin: var(--sl-space-sm) 0;
}

.collapsible-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md);
  cursor: pointer;
  background: var(--sl-surface);
  transition: background-color var(--sl-transition-fast);
}

.collapsible-header:hover {
  background: var(--sl-surface-hover);
}

.collapsible-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-normal);
  flex-shrink: 0;
}

.collapsible-toggle:hover {
  background: var(--sl-border-light);
  color: var(--sl-text-primary);
}

.collapsible-toggle.expanded {
  transform: rotate(180deg);
}

.collapsible-content {
  padding: 0 var(--sl-space-md) var(--sl-space-md);
  background: var(--sl-surface);
}

.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  opacity: 1;
  max-height: 800px;
}
</style>
