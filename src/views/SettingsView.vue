<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from "vue";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLModal from "@components/common/SLModal.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import JavaDownloader from "@components/JavaDownloader.vue";
import {
  settingsApi,
  checkAcrylicSupport,
  applyAcrylic,
  getSystemFonts,
  type AppSettings,
  type SettingsGroup,
} from "@api/settings";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { dispatchSettingsUpdate, SETTINGS_UPDATE_EVENT } from "@stores/settingsStore";

const { error, showError, clearError } = useMessage();
const { loading, start: startLoading, stop: stopLoading } = useLoading();
const { loading: fontsLoading, start: startFontsLoading, stop: stopFontsLoading } = useLoading();
const { loading: saving, start: startSaving, stop: stopSaving } = useLoading();

const settings = ref<AppSettings | null>(null);

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

const fontFamilyOptions = ref<{ label: string; value: string }[]>([
  { label: i18n.t("common.font_system_default"), value: "" },
]);

const showImportModal = ref(false);
const importJson = ref("");
const showResetConfirm = ref(false);
const bgSettingsExpanded = ref(false);
const bgPreviewLoaded = ref(false);
const bgPreviewLoading = ref(false);

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

function handleSettingsUpdateEvent(
  e: CustomEvent<{ changedGroups: SettingsGroup[]; settings: AppSettings }>,
) {
  const newSettings = e.detail.settings;
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
  startFontsLoading();
  try {
    const fonts = await getSystemFonts();
    fontFamilyOptions.value = [
      { label: i18n.t("common.font_system_default"), value: "" },
      ...fonts.map((font) => ({ label: font, value: `'${font}'` })),
    ];
  } catch (e) {
    console.error("Failed to load system fonts:", e);
    showError(String(e));
  } finally {
    stopFontsLoading();
  }
}

watch(bgSettingsExpanded, (expanded) => {
  if (expanded && settings.value?.background_image) {
    bgPreviewLoaded.value = false;
    bgPreviewLoading.value = true;
  }
});

async function loadSettings() {
  startLoading();
  clearError();
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
    showError(String(e));
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
    showError(String(e));
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

  startSaving();
  clearError();
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

      if (acrylicSupported.value) {
        try {
          const isDark = getEffectiveTheme(settings.value.theme) === "dark";
          await applyAcrylic(settings.value.acrylic_enabled, isDark);
        } catch {}
      }
    }

    dispatchSettingsUpdate(result.changed_groups, result.settings);
  } catch (e) {
    showError(String(e));
  } finally {
    stopSaving();
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
    showError(String(e));
  }
}

async function exportSettings() {
  try {
    const json = await settingsApi.exportJson();
    await navigator.clipboard.writeText(json);
  } catch (e) {
    showError(String(e));
  }
}

async function handleImport() {
  if (!importJson.value.trim()) {
    showError(i18n.t("common.paste_json"));
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
    showError(String(e));
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
    showError(String(e));
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
      <button @click="clearError()">x</button>
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
            <JavaDownloader
              @installed="
                (path) => {
                  if (settings) {
                    settings.default_java_path = path;
                    markChanged();
                  }
                }
              "
            />
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

<style src="@styles/views/SettingsView.css" scoped></style>
