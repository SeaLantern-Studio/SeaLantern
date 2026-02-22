<script setup lang="ts">
import { ChevronDown } from "lucide-vue-next";
import { ref, onMounted, onUnmounted, watch, computed } from "vue";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLModal from "@components/common/SLModal.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { i18n } from "@language";
import {
  settingsApi,
  checkAcrylicSupport,
  applyAcrylic,
  getSystemFonts,
  type AppSettings,
} from "@api/settings";
import { systemApi } from "@api/system";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getThemeOptions } from "@themes";
import { usePluginStore } from "@stores/pluginStore";
import {
  dispatchSettingsUpdate,
  SETTINGS_UPDATE_EVENT,
  type SettingsUpdateEvent,
} from "@stores/settingsStore";

const settings = ref<AppSettings | null>(null);
const loading = ref(true);
const fontsLoading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);

const acrylicSupported = ref(true);
const pluginStore = usePluginStore();

const themeProxyPlugin = computed(() => {
  return pluginStore.plugins.find(
    (p) => p.state === "enabled" && pluginStore.hasCapability(p.manifest.id, "theme-provider"),
  );
});

const isThemeProxied = computed(() => !!themeProxyPlugin.value);
const themeProxyPluginName = computed(() => themeProxyPlugin.value?.manifest.name || "");

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
  { label: i18n.t("settings.background_size_options.cover"), value: "cover" },
  { label: i18n.t("settings.background_size_options.contain"), value: "contain" },
  { label: i18n.t("settings.background_size_options.fill"), value: "fill" },
  { label: i18n.t("settings.background_size_options.auto"), value: "auto" },
]);

const colorOptions = computed(() => getThemeOptions());

const themeOptions = computed(() => [
  { label: i18n.t("settings.theme_options.auto"), value: "auto" },
  { label: i18n.t("settings.theme_options.light"), value: "light" },
  { label: i18n.t("settings.theme_options.dark"), value: "dark" },
]);

const fontFamilyOptions = ref<{ label: string; value: string }[]>([
  { label: i18n.t("settings.font_family_default"), value: "" },
]);

const showImportModal = ref(false);
const importJson = ref("");
const showResetConfirm = ref(false);
const bgSettingsExpanded = ref(false);
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
  try {
    acrylicSupported.value = await checkAcrylicSupport();
  } catch {
    acrylicSupported.value = false;
  }

  window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
});

onUnmounted(() => {
  window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
  if (saveTimeout) {
    clearTimeout(saveTimeout);
    saveTimeout = null;
  }
});

function handleSettingsUpdateEvent(e: CustomEvent<SettingsUpdateEvent>) {
  const { settings: newSettings } = e.detail;
  settings.value = newSettings;
  syncLocalValues(newSettings);
}

function syncLocalValues(s: AppSettings) {
  maxMem.value = String(s.default_max_memory);
  minMem.value = String(s.default_min_memory);
  port.value = String(s.default_port);
  fontSize.value = String(s.console_font_size);
  logLines.value = String(s.max_log_lines);
  bgOpacity.value = String(s.background_opacity);
  bgBlur.value = String(s.background_blur);
  bgBrightness.value = String(s.background_brightness);
  uiFontSize.value = String(s.font_size);
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

  saving.value = true;
  error.value = null;
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

      if (acrylicSupported.value) {
        try {
          const isDark = getEffectiveTheme(settings.value.theme) === "dark";
          await applyAcrylic(settings.value.acrylic_enabled, isDark);
        } catch {}
      }
    }

    dispatchSettingsUpdate(result.changed_groups, result.settings);
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

async function handleImport() {
  if (!importJson.value.trim()) {
    error.value = i18n.t("settings.no_json");
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
      <SLCard>
        <div
          style="
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: var(--sl-space-sm);
          "
        >
          <div>
            <h3
              style="
                margin: 0;
                font-size: 1.125rem;
                font-weight: 600;
                color: var(--sl-text-primary);
              "
            >
              {{ i18n.t("settings.color_theme") }}
            </h3>
            <p
              style="
                margin: var(--sl-space-xs) 0 0 0;
                font-size: 0.875rem;
                color: var(--sl-text-secondary);
              "
            >
              {{ i18n.t("settings.color_theme_desc") }}
            </p>
          </div>
          <div class="input-lg">
            <div v-if="isThemeProxied" class="theme-proxied-notice">
              <span class="proxied-text">{{
                i18n.t("settings.theme_proxied_by", { plugin: themeProxyPluginName })
              }}</span>
            </div>
            <SLSelect
              v-else
              v-model="settings.color"
              :options="colorOptions"
              @update:modelValue="markChanged"
            />
          </div>
        </div>
      </SLCard>

      <SLCard :title="i18n.t('settings.appearance')" :subtitle="i18n.t('settings.appearance_desc')">
        <div class="settings-group">
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.theme") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.theme_desc") }}</span>
            </div>
            <div class="input-lg">
              <div v-if="isThemeProxied" class="theme-proxied-notice">
                <span class="proxied-text">{{
                  i18n.t("settings.theme_proxied_by", { plugin: themeProxyPluginName })
                }}</span>
              </div>
              <SLSelect
                v-else
                v-model="settings.theme"
                :options="themeOptions"
                @update:modelValue="handleThemeChange"
              />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.font_size") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.font_size_desc") }}</span>
            </div>
            <div class="slider-control">
              <input
                type="range"
                min="12"
                max="24"
                step="1"
                v-model="uiFontSize"
                @input="handleFontSizeChange"
                class="sl-slider"
              />
              <span class="slider-value">{{ uiFontSize }}px</span>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.font_family") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.font_family_desc") }}</span>
            </div>
            <div class="input-lg">
              <SLSelect
                v-model="settings.font_family"
                :options="fontFamilyOptions"
                :searchable="true"
                :loading="fontsLoading"
                :previewFont="true"
                :placeholder="i18n.t('settings.search_font')"
                @update:modelValue="handleFontFamilyChange"
              />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.acrylic") }}</span>
              <span class="setting-desc">
                {{
                  acrylicSupported
                    ? i18n.t("settings.acrylic_desc")
                    : i18n.t("settings.acrylic_not_supported")
                }}
              </span>
            </div>
            <SLSwitch
              v-model="settings.acrylic_enabled"
              :disabled="!acrylicSupported"
              @update:modelValue="handleAcrylicChange"
            />
          </div>

          <div class="collapsible-section">
            <div class="collapsible-header" @click="bgSettingsExpanded = !bgSettingsExpanded">
              <div class="setting-info">
                <span class="setting-label">{{ i18n.t("settings.background") }}</span>
                <span class="setting-desc">{{ i18n.t("settings.background_desc") }}</span>
              </div>
              <div class="collapsible-toggle" :class="{ expanded: bgSettingsExpanded }">
                <ChevronDown :size="20" />
              </div>
            </div>
            <Transition name="collapse">
              <div v-show="bgSettingsExpanded" class="collapsible-content">
                <div class="setting-row full-width">
                  <div class="bg-image-picker">
                    <div v-if="settings.background_image" class="bg-preview">
                      <div v-if="bgPreviewLoading && !bgPreviewLoaded" class="bg-preview-loading">
                        <div class="loading-spinner"></div>
                        <span>{{ i18n.t("settings.loading_preview") }}</span>
                      </div>
                      <img
                        v-show="bgPreviewLoaded || !bgPreviewLoading"
                        :src="backgroundPreviewUrl"
                        alt="Background preview"
                        @load="
                          bgPreviewLoaded = true;
                          bgPreviewLoading = false;
                        "
                        @loadstart="bgPreviewLoading = true"
                        @error="bgPreviewLoading = false"
                        loading="lazy"
                      />
                      <div
                        v-if="isAnimatedImage(settings.background_image)"
                        class="bg-animated-badge"
                      >
                        {{ i18n.t("settings.animated_image") }}
                      </div>
                      <div class="bg-preview-overlay">
                        <span class="bg-preview-path">{{
                          settings.background_image.split("\\").pop()
                        }}</span>
                        <SLButton variant="danger" size="sm" @click="clearBackgroundImage">{{
                          i18n.t("settings.remove")
                        }}</SLButton>
                      </div>
                    </div>
                    <SLButton v-else variant="secondary" @click="pickBackgroundImage">
                      {{ i18n.t("settings.pick_image") }}
                    </SLButton>
                    <SLButton
                      v-if="settings.background_image"
                      variant="secondary"
                      size="sm"
                      @click="pickBackgroundImage"
                    >
                      {{ i18n.t("settings.replace_image") }}
                    </SLButton>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.opacity") }}</span>
                    <span class="setting-desc">{{ i18n.t("settings.opacity_desc") }}</span>
                  </div>
                  <div class="slider-control">
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.05"
                      v-model="bgOpacity"
                      @input="markChanged"
                      class="sl-slider"
                    />
                    <span class="slider-value">{{ bgOpacity }}</span>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.blur") }}</span>
                    <span class="setting-desc">{{ i18n.t("settings.blur_desc") }}</span>
                  </div>
                  <div class="slider-control">
                    <input
                      type="range"
                      min="0"
                      max="20"
                      step="1"
                      v-model="bgBlur"
                      @input="markChanged"
                      class="sl-slider"
                    />
                    <span class="slider-value">{{ bgBlur }}px</span>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.brightness") }}</span>
                    <span class="setting-desc">{{ i18n.t("settings.brightness_desc") }}</span>
                  </div>
                  <div class="slider-control">
                    <input
                      type="range"
                      min="0"
                      max="2"
                      step="0.1"
                      v-model="bgBrightness"
                      @input="markChanged"
                      class="sl-slider"
                    />
                    <span class="slider-value">{{ bgBrightness }}</span>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.background_size") }}</span>
                    <span class="setting-desc">{{ i18n.t("settings.background_size_desc") }}</span>
                  </div>
                  <div class="input-lg">
                    <SLSelect
                      v-model="settings.background_size"
                      :options="backgroundSizeOptions"
                      @update:modelValue="markChanged"
                    />
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </div>
      </SLCard>

      <div class="settings-actions">
        <div class="actions-left"></div>
        <div class="actions-right">{{ i18n.t("settings.personalize_page_import_export") }}</div>
      </div>
    </template>

    <SLModal
      :visible="showImportModal"
      :title="i18n.t('settings.import_settings')"
      @close="showImportModal = false"
    >
      <div class="import-form">
        <p class="text-caption">{{ i18n.t("settings.import_desc") }}</p>
        <textarea
          class="import-textarea"
          v-model="importJson"
          placeholder='{"close_servers_on_exit": true, ...}'
          rows="10"
        ></textarea>
      </div>
      <template #footer>
        <SLButton variant="secondary" @click="showImportModal = false">{{
          i18n.t("settings.cancel")
        }}</SLButton>
        <SLButton variant="primary" @click="handleImport">{{
          i18n.t("settings.confirm_import")
        }}</SLButton>
      </template>
    </SLModal>

    <SLModal
      :visible="showResetConfirm"
      :title="i18n.t('settings.reset_confirm')"
      @close="showResetConfirm = false"
    >
      <p class="text-body">{{ i18n.t("settings.reset_desc") }}</p>
      <template #footer>
        <SLButton variant="secondary" @click="showResetConfirm = false">{{
          i18n.t("settings.cancel")
        }}</SLButton>
        <SLButton variant="danger" @click="resetSettings">{{
          i18n.t("settings.confirm_reset")
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

.theme-proxied-notice {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  background: rgba(96, 165, 250, 0.1);
  border: 1px solid rgba(96, 165, 250, 0.3);
  border-radius: var(--sl-radius-md);
  color: var(--sl-primary);
  font-size: 0.875rem;
  min-width: 200px;
}

.proxied-text {
  white-space: nowrap;
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
.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: sl-spin 0.8s linear infinite;
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
.input-lg {
  width: 320px;
  flex-shrink: 0;
}

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
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
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
  transition: all 0.3s ease;
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
