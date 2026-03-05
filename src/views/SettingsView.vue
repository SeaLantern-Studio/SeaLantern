<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import GeneralSettingsCard from "@components/views/settings/GeneralSettingsCard.vue";
import ServerDefaultsCard from "@components/views/settings/ServerDefaultsCard.vue";
import DeveloperModeCard from "@components/views/settings/DeveloperModeCard.vue";
import SettingsActions from "@components/views/settings/SettingsActions.vue";
import ImportSettingsModal from "@components/views/settings/ImportSettingsModal.vue";
import ResetConfirmModal from "@components/views/settings/ResetConfirmModal.vue";
import { settingsApi, type AppSettings, type SettingsGroup } from "@api/settings";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { dispatchSettingsUpdate, SETTINGS_UPDATE_EVENT } from "@stores/settingsStore";

const { error, showError, clearError } = useMessage();
const { loading, start: startLoading, stop: stopLoading } = useLoading();

const settings = ref<AppSettings | null>(null);

const maxMem = ref("2048");
const minMem = ref("512");
const port = ref("25565");
const defaultRunPath = ref("");

const showImportModal = ref(false);
const showResetConfirm = ref(false);

onMounted(async () => {
  await loadSettings();

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
  defaultRunPath.value = s.last_run_path || "";
}

async function loadSettings() {
  startLoading();
  clearError();
  try {
    const s = await settingsApi.get();
    settings.value = s;
    maxMem.value = String(s.default_max_memory);
    minMem.value = String(s.default_min_memory);
    port.value = String(s.default_port);
    defaultRunPath.value = s.last_run_path || "";
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

function applyFontFamily(fontFamily: string) {
  if (fontFamily) {
    document.documentElement.style.setProperty("--sl-font-sans", fontFamily);
    document.documentElement.style.setProperty("--sl-font-display", fontFamily);
  } else {
    document.documentElement.style.removeProperty("--sl-font-sans");
    document.documentElement.style.removeProperty("--sl-font-display");
  }
}

async function saveSettings() {
  if (!settings.value) return;

  settings.value.default_max_memory = parseInt(maxMem.value) || 2048;
  settings.value.default_min_memory = parseInt(minMem.value) || 512;
  settings.value.default_port = parseInt(port.value) || 25565;
  settings.value.last_run_path = defaultRunPath.value;
  settings.value.color = settings.value.color || "default";
  settings.value.developer_mode = settings.value.developer_mode || false;

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
    }

    dispatchSettingsUpdate(result.changed_groups, result.settings);
  } catch (e) {
    showError(String(e));
  }
}

async function resetSettings() {
  try {
    const s = await settingsApi.reset();
    settings.value = s;
    maxMem.value = String(s.default_max_memory);
    minMem.value = String(s.default_min_memory);
    port.value = String(s.default_port);
    defaultRunPath.value = s.last_run_path || "";
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

async function handleImport(json: string) {
  if (!json.trim()) {
    showError(i18n.t("common.paste_json"));
    return;
  }
  try {
    const s = await settingsApi.importJson(json);
    settings.value = s;
    maxMem.value = String(s.default_max_memory);
    minMem.value = String(s.default_min_memory);
    port.value = String(s.default_port);
    showImportModal.value = false;
    applyTheme(s.theme);
    applyFontSize(s.font_size);
    applyFontFamily(s.font_family);
  } catch (e) {
    showError(String(e));
  }
}

function handleJavaInstalled(path: string) {
  if (settings.value) {
    settings.value.default_java_path = path;
    markChanged();
  }
}

async function handleBrowseJavaPath() {
  const selected = await systemApi.pickJavaFile();
  if (selected) {
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
</script>

<template>
  <div class="settings-view animate-fade-in-up">
    <div v-if="error" class="msg-banner error-banner">
      <span>{{ error }}</span>
      <button @click="clearError()">x</button>
    </div>

    <div v-if="loading" class="loading-state">
      <SLSpinner />
      <span>{{ i18n.t("settings.loading") }}</span>
    </div>

    <template v-else-if="settings">
      <GeneralSettingsCard
        v-model:closeServersOnExit="settings.close_servers_on_exit"
        v-model:autoAcceptEula="settings.auto_accept_eula"
        v-model:closeAction="settings.close_action"
        @change="markChanged"
      />

      <ServerDefaultsCard
        v-model:maxMemory="maxMem"
        v-model:minMemory="minMem"
        v-model:port="port"
        v-model:defaultJavaPath="settings.default_java_path"
        v-model:defaultJvmArgs="settings.default_jvm_args"
        v-model:defaultRunPath="defaultRunPath"
        @change="markChanged"
        @javaInstalled="handleJavaInstalled"
        @browseJavaPath="handleBrowseJavaPath"
        @browseRunPath="handleBrowseRunPath"
      />

      <DeveloperModeCard v-model:developerMode="settings.developer_mode" @change="markChanged" />

      <SettingsActions
        @export="exportSettings"
        @import="showImportModal = true"
        @reset="showResetConfirm = true"
      />
    </template>

    <ImportSettingsModal v-model:visible="showImportModal" @import="handleImport" />

    <ResetConfirmModal v-model:visible="showResetConfirm" @confirm="resetSettings" />
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
  font-size: var(--sl-font-size-base);
}

.error-banner {
  background: var(--sl-error-bg);
  border: 1px solid var(--sl-error);
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
</style>
