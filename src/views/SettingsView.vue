<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import GeneralSettingsCard from "@components/views/settings/GeneralSettingsCard.vue";
import ServerDefaultsCard from "@components/views/settings/ServerDefaultsCard.vue";
import NetworkSettingsCard from "@components/views/settings/NetworkSettingsCard.vue";
import DeveloperModeCard from "@components/views/settings/DeveloperModeCard.vue";
import SettingsActions from "@components/views/settings/SettingsActions.vue";
import ImportSettingsModal from "@components/views/settings/ImportSettingsModal.vue";
import ResetConfirmModal from "@components/views/settings/ResetConfirmModal.vue";
import { settingsApi, type AppSettings } from "@api/settings";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { useMessage, useGlobalMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { useSettingsStore } from "@stores/settingsStore";

const { error, showError, clearError } = useMessage();
const { success: globalSuccess } = useGlobalMessage();
const { loading, start: startLoading, stop: stopLoading } = useLoading();
const settingsStore = useSettingsStore();

const settings = ref<AppSettings | null>(null);

const maxMem = ref("2048");
const minMem = ref("512");
const port = ref("25565");
const defaultRunPath = ref("");

const showImportModal = ref(false);
const showResetConfirm = ref(false);

type CloseAction = "ask" | "minimize" | "close";

onMounted(async () => {
  await loadSettings();
});

onUnmounted(() => {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
    saveTimeout = null;
  }
});

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
    await settingsStore.ensureLoaded();
    const nextSettings = settingsStore.cloneSettings();
    settings.value = nextSettings;
    settings.value.color = nextSettings.color || "default";
    syncLocalValues(nextSettings);
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
    await settingsStore.saveSettingsWithDiff(settings.value);
    settings.value = settingsStore.cloneSettings();
    syncLocalValues(settings.value);
  } catch (e) {
    showError(String(e));
  }
}

async function resetSettings() {
  try {
    const s = await settingsStore.resetSettings([
      "Appearance",
      "General",
      "ServerDefaults",
      "Console",
    ]);
    settings.value = settingsStore.cloneSettings(s);
    syncLocalValues(settings.value);
    showResetConfirm.value = false;
    settings.value.color = "default";
  } catch (e) {
    showError(String(e));
  }
}

async function exportSettings() {
  try {
    const json = await settingsApi.exportJson();
    await navigator.clipboard.writeText(json);
    globalSuccess(i18n.t("settings.export_success"));
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
    const s = await settingsStore.importSettingsJson(json, [
      "Appearance",
      "General",
      "ServerDefaults",
      "Console",
    ]);
    settings.value = settingsStore.cloneSettings(s);
    syncLocalValues(settings.value);
    showImportModal.value = false;
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
        v-model:closeServersOnUpdate="settings.close_servers_on_update"
        v-model:autoAcceptEula="settings.auto_accept_eula"
        v-model:closeAction="settings.close_action as CloseAction"
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

      <NetworkSettingsCard />

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
