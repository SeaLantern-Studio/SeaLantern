<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import ServerDownloadModal from "@components/common/ServerDownloadModal.vue";
import { serverApi } from "@api/server";
import { javaApi, type JavaInfo } from "@api/java";
import { systemApi } from "@api/system";
import { settingsApi } from "@api/settings";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import JavaEnvironmentCard from "@components/views/create/JavaEnvironmentCard.vue";
import ServerConfigCard from "@components/views/create/ServerConfigCard.vue";
import CreateServerActions from "@components/views/create/CreateServerActions.vue";

type StartupMode = "jar" | "bat" | "sh";

const router = useRouter();
const store = useServerStore();
const { error: errorMsg, showError, clearError } = useMessage();
const { loading: javaLoading, start: startJavaLoading, stop: stopJavaLoading } = useLoading();
const { loading: creating, start: startCreating, stop: stopCreating } = useLoading();

const serverName = ref("My Server");
const maxMemory = ref("2048");
const minMemory = ref("512");
const port = ref("25565");
type StartupMode = "jar" | "bat" | "sh";
const {
  activeTab: startupMode,
  indicatorRef: startupModeIndicator,
  switchTab: switchStartupMode,
  updateIndicator,
} = useTabSwitch<StartupMode>("jar");
const selectedJava = ref("");
const onlineMode = ref(true);

const javaList = ref<JavaInfo[]>([]);

const startupModes: StartupMode[] = ["jar", "bat", "sh"];

const showDownloadModal = ref(false);
const downloadingServer = ref(false);

const localeRef = i18n.getLocaleRef();
watch(localeRef, () => {
  updateIndicator();
});

onMounted(async () => {
  await loadDefaultSettings();
});

async function loadDefaultSettings() {
  try {
    const settings = await settingsApi.get();

    maxMemory.value = String(settings.default_max_memory);
    minMemory.value = String(settings.default_min_memory);
    port.value = String(settings.default_port);

    if (settings.cached_java_list && settings.cached_java_list.length > 0) {
      javaList.value = settings.cached_java_list;

      if (settings.default_java_path) {
        selectedJava.value = settings.default_java_path;
      } else if (javaList.value.length > 0) {
        const preferred = javaList.value.find((j) => j.is_64bit && j.major_version >= 17);
        selectedJava.value = preferred ? preferred.path : javaList.value[0].path;
      }
    }
  } catch (e) {
    console.error("Failed to load default settings:", e);
  }
}

async function detectJava() {
  startJavaLoading();
  try {
    javaList.value = await javaApi.detect();
    if (javaList.value.length > 0) {
      const preferred = javaList.value.find((j) => j.is_64bit && j.major_version >= 17);
      selectedJava.value = preferred ? preferred.path : javaList.value[0].path;
    }

    const settings = await settingsApi.get();
    settings.cached_java_list = javaList.value;
    await settingsApi.save(settings);
  } catch (e) {
    console.error("Java detection failed:", e);
    showError(String(e));
  } finally {
    stopJavaLoading();
  }
}

function handleSetStartupMode(mode: StartupMode) {
  if (startupMode.value === mode) {
    return;
  }
  switchStartupMode(mode);
}

async function pickJavaFile() {
  try {
    const result = await systemApi.pickJavaFile();
    if (result) {
      selectedJava.value = result;
    }
  } catch (e) {
    console.error("Pick file error:", e);
  }
}

function openDownloadModal() {
  clearError();
  if (!selectedJava.value) {
    showError(i18n.t("common.select_java_path"));
    return;
  }
  if (!serverName.value.trim()) {
    showError(i18n.t("common.enter_server_name"));
    return;
  }
  showDownloadModal.value = true;
}

async function handleServerSelect(jarPath: string, serverName_: string, version: string) {
  showDownloadModal.value = false;
  downloadingServer.value = true;
  startCreating();
  try {
    const finalName = serverName.value === "My Server" 
      ? `${serverName_}-${version}` 
      : serverName.value;

    await serverApi.importServer({
      name: finalName,
      jarPath: jarPath,
      startupMode: startupMode.value,
      javaPath: selectedJava.value,
      maxMemory: parseInt(maxMemory.value) || 2048,
      minMemory: parseInt(minMemory.value) || 512,
      port: parseInt(port.value) || 25565,
      onlineMode: onlineMode.value,
    });
    await systemApi.deleteFile(jarPath);
    await store.refreshList();
    router.push("/");
  } catch (e) {
    showError(String(e));
  } finally {
    stopCreating();
    downloadingServer.value = false;
  }
}

async function handleImport() {
  clearError();

  if (!selectedJava.value) {
    showError(i18n.t("common.select_java_path"));
    return;
  }
  if (!serverName.value.trim()) {
    showError(i18n.t("common.enter_server_name"));
    return;
  }

  const result = await systemApi.pickStartupFile(startupMode.value);
  if (!result) {
    return;
  }

  const serverPath = result.substring(0, result.lastIndexOf('\\') || result.lastIndexOf('/'));

  startCreating();
  try {
    const mode = startupMode.value;
    await serverApi.addExistingServer({
      name: serverName.value,
      serverPath: serverPath,
      javaPath: selectedJava.value,
      maxMemory: parseInt(maxMemory.value) || 2048,
      minMemory: parseInt(minMemory.value) || 512,
      port: parseInt(port.value) || 25565,
      startupMode: startupMode.value,
      executablePath: result,
    });
    await store.refreshList();
    router.push("/");
  } catch (e) {
    showError(String(e));
  } finally {
    stopCreating();
  }
}

function getJavaLabel(java: JavaInfo): { label: string; subLabel: string } {
  const version = java.major_version;
  const arch = java.is_64bit ? i18n.t("common.java_64bit") : i18n.t("common.java_32bit");

  let vendor = java.vendor;
  if (vendor.includes("Oracle") || vendor.includes("Sun")) {
    vendor = "Oracle";
  } else if (vendor.includes("Temurin") || vendor.includes("Adopt")) {
    vendor = "Eclipse Temurin";
  } else if (vendor.includes("Amazon")) {
    vendor = "Amazon Corretto";
  } else if (vendor.includes("Microsoft")) {
    vendor = "Microsoft";
  } else if (vendor.includes("Zulu") || vendor.includes("Azul")) {
    vendor = "Azul Zulu";
  } else if (vendor.includes("Liberica") || vendor.includes("BellSoft")) {
    vendor = "Liberica";
  }

  return {
    label: `Java ${version} ${vendor} ${arch}`,
    subLabel: java.path,
  };
}

const javaOptions = computed(() => {
  return javaList.value.map((java) => {
    const labelInfo = getJavaLabel(java);
    return {
      label: labelInfo.label,
      subLabel: labelInfo.subLabel,
      value: java.path,
    };
  });
});
</script>

<template>
  <div class="create-view animate-fade-in-up">
    <div v-if="errorMsg" class="error-banner">
      <span>{{ errorMsg }}</span>
      <button class="error-close" @click="clearError()">x</button>
    </div>

    <SLCard :title="i18n.t('create.java_env')" :subtitle="i18n.t('create.java_scan')">
      <div v-if="javaLoading" class="java-loading">
        <div class="spinner"></div>
        <span>{{ i18n.t("create.scanning") }}</span>
      </div>
      <div v-else-if="javaList.length === 0" class="java-empty">
        <p class="text-body">{{ i18n.t("create.no_java") }}</p>
        <SLButton variant="primary" @click="detectJava" style="margin-top: 12px">
          {{ i18n.t("create.scan") }}
        </SLButton>
      </div>
      <div v-else class="java-select-container">
        <div class="java-header">
          <div class="java-found text-caption">
            {{ i18n.t("create.java_found", { count: javaList.length }) }}
          </div>
          <button class="rescan-btn" @click="detectJava" :disabled="javaLoading">
            <RefreshCw :size="14" />
            {{ i18n.t("create.rescan") }}
          </button>
        </div>
        <SLSelect
          v-model="selectedJava"
          :options="javaOptions"
          :placeholder="i18n.t('create.select_java')"
          searchable
          maxHeight="240px"
        />
      </div>
      <div class="java-manual">
        <SLInput
          :label="i18n.t('create.java_path')"
          v-model="selectedJava"
          :placeholder="i18n.t('create.java_manual')"
        >
          <template #suffix>
            <button class="pick-btn" @click="pickJavaFile">{{ i18n.t("create.browse") }}</button>
          </template>
        </SLInput>
      </div>
    </SLCard>

    <SLCard :title="i18n.t('create.title')">
      <div class="form-grid">
        <div class="server-name-row">
          <SLInput
            :label="i18n.t('create.server_name')"
            :placeholder="i18n.t('create.server_name')"
            v-model="serverName"
          />
        </div>
        <div class="startup-mode-row">
          <span class="startup-mode-label">{{ i18n.t("create.startup_mode") }}</span>
          <div class="startup-mode-control">
            <div class="startup-mode-tabs">
              <div class="startup-mode-indicator" ref="startupModeIndicator"></div>
              <button
                v-for="mode in startupModes"
                :key="mode"
                type="button"
                class="startup-mode-tab"
                :class="{ active: startupMode === mode }"
                @click="handleSetStartupMode(mode)"
              >
                {{ mode === "jar" ? "JAR" : mode }}
              </button>
            </div>
          </div>
        </div>

        <SLInput
          :label="i18n.t('create.max_memory')"
          type="text"
          v-model="maxMemory"
          @input="
            (e) => {
              const value = e.target.value;
              if (value === '' || /^\d+$/.test(value)) {
                maxMemory.value = value;
              }
            }
          "
        />
        <SLInput
          :label="i18n.t('create.min_memory')"
          type="text"
          v-model="minMemory"
          @input="
            (e) => {
              const value = e.target.value;
              if (value === '' || /^\d+$/.test(value)) {
                minMemory.value = value;
              }
            }
          "
        />
        <SLInput
          :label="i18n.t('settings.default_port')"
          type="text"
          v-model="port"
          :placeholder="i18n.t('create.default_port_placeholder')"
          @input="
            (e) => {
              const value = e.target.value;
              if (value === '' || /^\d+$/.test(value)) {
                port.value = value;
              }
            }
          "
        />
        <div class="online-mode-cell">
          <span class="online-mode-label">{{ i18n.t("create.online_mode") }}</span>
          <div class="online-mode-box">
            <span class="online-mode-text">{{
              onlineMode ? i18n.t("create.online_mode_on") : i18n.t("create.online_mode_off")
            }}</span>
            <SLSwitch v-model="onlineMode" />
          </div>
        </div>
      </div>
    </SLCard>

    <div class="create-actions">
      <SLButton variant="secondary" size="lg" @click="router.push('/')">{{
        i18n.t("create.cancel")
      }}</SLButton>
      <SLButton 
        variant="primary" 
        size="lg" 
        :loading="creating || downloadingServer" 
        @click="openDownloadModal"
      >
        {{ i18n.t("create.select_and_create") }}
      </SLButton>
      <SLButton variant="primary" size="lg" :loading="creating" @click="handleImport">
        {{ i18n.t("create.import_existing") }}
      </SLButton>
    </div>

    <ServerDownloadModal
      :visible="showDownloadModal"
      @close="showDownloadModal = false"
      @select="handleServerSelect"
    />
  </div>
</template>

<style scoped>
.create-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
  max-width: 760px;
  margin: 0 auto;
}
.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--sl-radius-md);
  color: var(--sl-error);
  font-size: 0.875rem;
}
.error-close {
  color: var(--sl-error);
  font-weight: 600;
}
.java-loading {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-lg);
  color: var(--sl-text-tertiary);
}
.java-empty {
  padding: var(--sl-space-lg);
  text-align: center;
}
.java-select-container {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}
.java-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--sl-space-xs);
}
.java-found {
  margin: 0;
}
.rescan-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}
.rescan-btn:hover:not(:disabled) {
  background: var(--sl-primary);
  color: white;
}
.rescan-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.selected-java-path {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 8px 12px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-sm);
  overflow: hidden;
}
.selected-java-path .text-mono {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.java-manual {
  padding-top: var(--sl-space-sm);
  border-top: 1px solid var(--sl-border-light);
}
.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--sl-space-md);
}
.server-name-row {
  grid-column: 1 / -1;
}
.startup-mode-row {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}
.startup-mode-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
}
.startup-mode-control {
  display: flex;
  align-items: center;
}
.startup-mode-tabs {
  display: flex;
  gap: 2px;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  padding: 3px;
  width: 100%;
  position: relative;
  overflow: hidden;
}
.startup-mode-indicator {
  position: absolute;
  top: 3px;
  bottom: 3px;
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  transition: all var(--sl-transition-normal);
  box-shadow: var(--sl-shadow-sm);
  z-index: 1;
  border: 1px solid var(--sl-primary);
  opacity: 0.9;
}
.startup-mode-tab {
  flex: 1;
  padding: 6px 14px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  position: relative;
  z-index: 2;
  background: transparent;
  border: none;
  cursor: pointer;
  text-align: center;
}
.startup-mode-tab:hover {
  color: var(--sl-text-primary);
}
.startup-mode-tab.active {
  color: var(--sl-primary);
}

@media (prefers-color-scheme: dark) {
  .startup-mode-tab {
    color: var(--sl-text-tertiary);
  }
  .startup-mode-tab:hover {
    color: var(--sl-text-primary);
  }
  .startup-mode-tab.active {
    color: var(--sl-primary);
  }
}
.jar-picker {
  grid-column: 1 / -1;
}
.pick-btn {
  padding: 4px 12px;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  cursor: pointer;
  white-space: nowrap;
}
.pick-btn:hover {
  background: var(--sl-primary);
  color: white;
}
.online-mode-cell {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}
.online-mode-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
}
.online-mode-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
  padding: 6px 12px;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  height: 36px;
  box-sizing: border-box;
}
.online-mode-text {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
}
.create-actions {
  display: flex;
  justify-content: center;
  gap: var(--sl-space-md);
}
.create-actions :deep(.sl-button) {
  min-width: 120px;
}
</style>
