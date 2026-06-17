<script setup lang="ts">
import { ref, watch } from "vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import GeneralSettingsCard from "@components/views/settings/GeneralSettingsCard.vue";
import DataDirectoryCard from "@components/views/settings/DataDirectoryCard.vue";
import ServerDefaultsCard from "@components/views/settings/ServerDefaultsCard.vue";
import NetworkSettingsCard from "@components/views/settings/NetworkSettingsCard.vue";
import DeveloperModeCard from "@components/views/settings/DeveloperModeCard.vue";
import SettingsActions from "@components/views/settings/SettingsActions.vue";
import ImportSettingsModal from "@components/views/settings/ImportSettingsModal.vue";
import ResetConfirmModal from "@components/views/settings/ResetConfirmModal.vue";
import { javaApi, type JavaInfo } from "@api/java";
import { systemApi } from "@api/system";
import { useSettingsPageDraft } from "@composables/useSettingsPageDraft";
import { i18n } from "@language";
import { useDataDirectory } from "@composables/useDataDirectory";
import { useGlobalMessage } from "@composables/useMessage";
import { useSettingsStore } from "@stores/settingsStore";
import type { CpuPolicyConfig, JvmPresetConfig } from "@type/server";
import {
  createDefaultCpuPolicy,
  createDefaultJvmPreset,
  deserializeJvmArgs,
  getCpuPolicyValidationError,
  normalizeCpuPolicy,
  normalizeJvmPreset,
  serializeJvmArgsText,
} from "@utils/serverStartupConfig";

const { success: globalSuccess } = useGlobalMessage();
const settingsStore = useSettingsStore();
const dataDirectory = useDataDirectory();

const maxMem = ref("2048");
const minMem = ref("512");
const port = ref("25565");
const defaultRunPath = ref("");
const dataDirDraft = ref("");
const defaultJvmArgsText = ref("");
const defaultJvmPreset = ref<JvmPresetConfig>(createDefaultJvmPreset());
const defaultCpuPolicy = ref<CpuPolicyConfig>(createDefaultCpuPolicy());
const javaList = ref<JavaInfo[]>([]);
const javaLoading = ref(false);

const settingsDraft = useSettingsPageDraft({
  changedGroups: ["Appearance", "General", "ServerDefaults", "Console"],
  syncLocalValues: (settings) => {
    maxMem.value = String(settings.default_max_memory);
    minMem.value = String(settings.default_min_memory);
    port.value = String(settings.default_port);
    defaultRunPath.value = settings.last_run_path || "";
    defaultJvmArgsText.value = deserializeJvmArgs(settings.default_jvm_args);
    defaultJvmPreset.value = normalizeJvmPreset(settings.default_jvm_preset);
    defaultCpuPolicy.value = normalizeCpuPolicy(settings.default_cpu_policy);
    javaList.value = settings.cached_java_list || [];
  },
  prepareForSave: (settings) => {
    const cpuPolicyError = getCpuPolicyValidationError(defaultCpuPolicy.value);
    if (cpuPolicyError) {
      throw new Error(i18n.t(`settings.cpu_policy_invalid_${cpuPolicyError}`));
    }

    settings.default_max_memory = parseInt(maxMem.value) || 2048;
    settings.default_min_memory = parseInt(minMem.value) || 512;
    settings.default_port = parseInt(port.value) || 25565;
    settings.last_run_path = defaultRunPath.value;
    settings.default_jvm_args = serializeJvmArgsText(defaultJvmArgsText.value);
    settings.default_jvm_preset = normalizeJvmPreset(defaultJvmPreset.value);
    settings.default_cpu_policy = normalizeCpuPolicy(defaultCpuPolicy.value);
    settings.color = settings.color || "default";
    settings.developer_mode = settings.developer_mode || false;
  },
  emptyImportMessage: () => i18n.t("common.paste_json"),
});

const settings = settingsDraft.settings;
const loading = settingsDraft.loading;
const error = settingsDraft.error;
const showImportModal = settingsDraft.showImportModal;
const showResetConfirm = settingsDraft.showResetConfirm;
const clearError = settingsDraft.clearError;
const markChanged = settingsDraft.markChanged;
const resetSettings = settingsDraft.resetSettings;

let lastResolvedDataDir = "";

watch(
  () => settingsStore.dataDirStatus?.current_data_dir || "",
  (currentPath) => {
    if (!currentPath) {
      return;
    }

    if (!dataDirDraft.value || dataDirDraft.value === lastResolvedDataDir) {
      dataDirDraft.value = currentPath;
    }

    lastResolvedDataDir = currentPath;
  },
  { immediate: true },
);

type CloseAction = "ask" | "minimize" | "close";

async function exportSettings() {
  try {
    const json = await settingsStore.exportSettingsJson();
    await navigator.clipboard.writeText(json);
    globalSuccess(i18n.t("settings.export_success"));
  } catch (e) {
    settingsDraft.setError(String(e));
  }
}

async function handleImport(json: string) {
  await settingsDraft.importSettings(json);
}

function handleJavaInstalled(path: string) {
  if (settings.value) {
    settings.value.default_java_path = path;
    markChanged();
  }
}

async function handleDetectJava() {
  if (javaLoading.value) {
    return;
  }

  javaLoading.value = true;

  try {
    const detected = await javaApi.detect();
    javaList.value = detected;

    if (settings.value) {
      settings.value.cached_java_list = detected;
      markChanged();
    }
  } catch (e) {
    settingsDraft.setError(String(e));
  } finally {
    javaLoading.value = false;
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

async function handleBrowseDataDir() {
  const selected = await dataDirectory.browseFolder();
  if (selected) {
    dataDirDraft.value = selected;
  }
}

async function handleChangeDataDir(path: string) {
  await dataDirectory.change(path, true);
  dataDirDraft.value = settingsStore.dataDirStatus?.current_data_dir || path;
}

function handleDataDirDraftUpdate(value: string) {
  dataDirDraft.value = value;
}

async function handleRefreshDataDir() {
  const status = await dataDirectory.refreshStatus();
  dataDirDraft.value = status.current_data_dir;
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

      <DataDirectoryCard
        :status="settingsStore.dataDirStatus"
        :path-draft="dataDirDraft"
        :busy="dataDirectory.isBusy.value"
        :error="dataDirectory.error.value"
        :info-message="dataDirectory.infoMessage.value"
        @update:path-draft="handleDataDirDraftUpdate"
        @browse="handleBrowseDataDir"
        @refresh="handleRefreshDataDir"
        @change="handleChangeDataDir"
      />

      <ServerDefaultsCard
        v-model:maxMemory="maxMem"
        v-model:minMemory="minMem"
        v-model:port="port"
        v-model:defaultJavaPath="settings.default_java_path"
        v-model:defaultJvmArgsText="defaultJvmArgsText"
        v-model:defaultJvmPreset="defaultJvmPreset"
        v-model:defaultCpuPolicy="defaultCpuPolicy"
        v-model:defaultRunPath="defaultRunPath"
        :java-list="javaList"
        :java-loading="javaLoading"
        @change="markChanged"
        @detectJava="handleDetectJava"
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
