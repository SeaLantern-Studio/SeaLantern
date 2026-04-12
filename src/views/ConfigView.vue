<script setup lang="ts">
import { ref, computed, watch, onMounted, onActivated, nextTick } from "vue";
import { useRoute } from "vue-router";
import SLSpinner from "@components/common/SLSpinner.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLModal from "@components/common/SLModal.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import { SLTabBar } from "@components/common";
import { configApi } from "@api/config";
import { m_pluginApi, type m_PluginInfo, type m_PluginConfigFile } from "@api/mcs_plugins";
import type { ConfigEntry as ConfigEntryType } from "@api/config";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import {
  Trash2,
  RefreshCw,
  Save,
  Settings,
  FileText,
  RotateCcw,
  FolderOpen,
  Edit,
} from "lucide-vue-next";

import ConfigCategories from "@components/config/ConfigCategories.vue";
import ConfigSourceDiffView from "@components/config/ConfigSourceDiffView.vue";
import ConfigSourceEditor from "@components/config/ConfigSourceEditor.vue";
import { systemApi } from "@api/system";
import { buildDiffLines } from "@utils/configDiff";
import "@styles/plugin-list.css";
import "@styles/views/ConfigView.css";

const route = useRoute();
const store = useServerStore();

const entries = ref<ConfigEntryType[]>([]);
const editValues = ref<Record<string, string>>({});
const loadedValues = ref<Record<string, string>>({});
const loading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);
const successMsg = ref<string | null>(null);
const searchQuery = ref("");
const activeCategory = ref("all");
const editorMode = ref<"visual" | "source">("visual");
const sourceDraftText = ref("");
const loadedSourceText = ref("");
const visualDraftDirty = ref(false);
const modeSwitching = ref(false);
const sourceParseError = ref<string | null>(null);
const showDiscardConfirm = ref(false);
const showSaveDiffModal = ref(false);
const sourceDiffBaseText = ref("");
const pendingSaveSourceText = ref("");
const serverPath = computed(() => {
  const server = store.servers.find((s) => s.id === store.currentServerId);
  return server?.path || "";
});

const serverPropertiesPath = computed(() => {
  const basePath = serverPath.value.replace(/[/\\]$/, "");
  if (!basePath) {
    return "server.properties";
  }

  const separator = basePath.includes("\\") ? "\\" : "/";
  return `${basePath}${separator}server.properties`;
});

const plugins = ref<m_PluginInfo[]>([]);
const pluginsLoading = ref(false);
const selectedPlugin = ref<m_PluginInfo | null>(null);
const loadedPlugins = ref<Set<string>>(new Set());
const observer = ref<IntersectionObserver | null>(null);
const activeTab = ref<"properties" | "plugins">("properties");

const configTabs = computed(() => [
  {
    key: "properties",
    label: i18n.t("config.server_properties"),
    count: "i",
    countTitle: serverPropertiesPath.value,
  },
  { key: "plugins", label: i18n.t("config.server_plugins") },
]);

const currentServerId = computed(() => store.currentServerId);
const sourceDiffOriginalText = computed(() => sourceDiffBaseText.value);
const sourceDiffTargetText = computed(() => pendingSaveSourceText.value);

const editorModeTabs = computed(() => [
  { key: "visual", label: i18n.t("config.visual_mode") },
  { key: "source", label: i18n.t("config.source_mode") },
]);

const hasUnsavedChanges = computed(() => {
  if (editorMode.value === "source") {
    return sourceDraftText.value !== loadedSourceText.value;
  }

  const sourceDirty = sourceDraftText.value !== loadedSourceText.value;
  const visualDirty = !areMapValuesEqual(editValues.value, loadedValues.value);
  return sourceDirty || visualDirty;
});

const saveStatusText = computed(() =>
  hasUnsavedChanges.value ? i18n.t("config.status_unsaved") : i18n.t("config.status_loaded"),
);

const sourceDiffLines = computed(() =>
  buildDiffLines(sourceDiffOriginalText.value, sourceDiffTargetText.value),
);

const sourceDiffStats = computed(() => {
  let additions = 0;
  let deletions = 0;

  for (const line of sourceDiffLines.value) {
    if (line.type === "addition") additions += 1;
    if (line.type === "deletion") deletions += 1;
  }

  return { additions, deletions };
});

const categories = computed(() => {
  const cats = new Set(entries.value.map((e) => e.category));
  return ["all", ...Array.from(cats)];
});

const gamemodeOptions = ref([
  { label: i18n.t("config.gamemode.survival"), value: "survival" },
  { label: i18n.t("config.gamemode.creative"), value: "creative" },
  { label: i18n.t("config.gamemode.adventure"), value: "adventure" },
  { label: i18n.t("config.gamemode.spectator"), value: "spectator" },
]);

const difficultyOptions = ref([
  { label: i18n.t("config.difficulty.peaceful"), value: "peaceful" },
  { label: i18n.t("config.difficulty.easy"), value: "easy" },
  { label: i18n.t("config.difficulty.normal"), value: "normal" },
  { label: i18n.t("config.difficulty.hard"), value: "hard" },
]);

const filteredEntries = computed(() => {
  return entries.value.filter((e: ConfigEntryType) => {
    const matchCat = activeCategory.value === "all" || e.category === activeCategory.value;
    const matchSearch =
      !searchQuery.value ||
      e.key.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      (e.description ?? "").toLowerCase().includes(searchQuery.value.toLowerCase());
    return matchCat && matchSearch;
  });
});

function areMapValuesEqual(a: Record<string, string>, b: Record<string, string>) {
  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);

  if (aKeys.length !== bKeys.length) {
    return false;
  }

  for (const key of aKeys) {
    if (a[key] !== b[key]) {
      return false;
    }
  }

  return true;
}

function getTranslatedPropertyDescription(key: string) {
  const translationKey = `config.properties.${key}`;
  const translated = i18n.t(translationKey);
  return translated === translationKey ? "" : translated;
}

function applyParsedSourceState(sourceText: string, targetMode: "visual" | "source" = "visual") {
  const parsed = configApi.parseServerPropertiesSource(sourceText);
  return parsed.then((result) => {
    entries.value = result.entries as ConfigEntryType[];
    editValues.value = { ...result.raw };
    loadedValues.value = { ...result.raw };
    sourceDraftText.value = sourceText;
    loadedSourceText.value = sourceText;
    sourceDiffBaseText.value = sourceText;
    sourceParseError.value = null;
    visualDraftDirty.value = false;
    editorMode.value = targetMode;

    const port = result.raw["server-port"];
    if (port) {
      updateCurrentServerPort(port);
    }

    return result;
  });
}

onMounted(async () => {
  await store.refreshList();
  const routeId = route.params.id as string;
  if (routeId) {
    store.setCurrentServer(routeId);
  } else if (!store.currentServerId && store.servers.length > 0) {
    store.setCurrentServer(store.servers[0].id);
  }
  await loadProperties();
  await loadPlugins();
});

watch(
  () => store.currentServerId,
  async () => {
    if (store.currentServerId) {
      await loadProperties();
      await loadPlugins();
    }
  },
);

async function loadProperties() {
  if (!serverPath.value) return;

  loading.value = true;
  error.value = null;
  try {
    const sourceText = await configApi.readServerPropertiesSource(serverPath.value);
    await applyParsedSourceState(sourceText, "visual");
  } catch (e) {
    error.value = String(e);
    entries.value = [];
    editValues.value = {};
    loadedValues.value = {};
    sourceDraftText.value = "";
    loadedSourceText.value = "";
    sourceDiffBaseText.value = "";
  } finally {
    loading.value = false;
  }
}

/**
 * 更新当前服务器的端口信息
 * @param port 端口号字符串
 */
function updateCurrentServerPort(port: string) {
  if (!port) return;

  const currentServer = store.servers.find((s) => s.id === store.currentServerId);
  if (currentServer) {
    currentServer.port = parseInt(port) || 25565;
  }
}

async function saveProperties() {
  if (!serverPath.value || !hasUnsavedChanges.value || saving.value) return;

  error.value = null;

  try {
    const latestSourceText = await configApi.readServerPropertiesSource(serverPath.value);
    sourceDiffBaseText.value = latestSourceText;

    if (editorMode.value === "visual") {
      pendingSaveSourceText.value = await configApi.previewServerPropertiesWrite(
        serverPath.value,
        editValues.value,
      );
    } else {
      pendingSaveSourceText.value = sourceDraftText.value;
    }

    if (pendingSaveSourceText.value === latestSourceText) {
      await applyParsedSourceState(latestSourceText, editorMode.value);
      successMsg.value = i18n.t("config.no_changes_to_save");
      setTimeout(() => (successMsg.value = null), 3000);
      return;
    }

    showSaveDiffModal.value = true;
  } catch (e) {
    error.value = String(e);
  }
}

function updateValue(key: string, value: string | boolean | number) {
  editValues.value[key] = String(value);
  visualDraftDirty.value = true;
}

function updateSourceDraft(value: string) {
  sourceDraftText.value = value;
  sourceParseError.value = null;
}

async function handleEditorModeChange(mode: string | null) {
  const targetMode = mode === "source" ? "source" : "visual";
  if (targetMode === editorMode.value || modeSwitching.value || !serverPath.value) return;

  modeSwitching.value = true;
  error.value = null;

  try {
    if (targetMode === "source") {
      if (visualDraftDirty.value) {
        sourceDraftText.value = await configApi.previewServerPropertiesWrite(
          serverPath.value,
          editValues.value,
        );
        visualDraftDirty.value = false;
      }
      sourceParseError.value = null;
      editorMode.value = "source";
      return;
    }

    const parsed = await configApi.parseServerPropertiesSource(sourceDraftText.value);
    entries.value = parsed.entries as ConfigEntryType[];
    editValues.value = { ...parsed.raw };
    visualDraftDirty.value = false;
    sourceParseError.value = null;
    editorMode.value = "visual";
  } catch (e) {
    sourceParseError.value = i18n.t("config.source_parse_failed");
    error.value = String(e);
  } finally {
    modeSwitching.value = false;
  }
}

async function confirmSaveProperties() {
  if (!serverPath.value || saving.value) return;

  saving.value = true;
  error.value = null;
  successMsg.value = null;

  try {
    await configApi.writeServerPropertiesSource(serverPath.value, pendingSaveSourceText.value);
    await applyParsedSourceState(pendingSaveSourceText.value, editorMode.value);
    successMsg.value = i18n.t("config.saved");
    showSaveDiffModal.value = false;
    setTimeout(() => (successMsg.value = null), 3000);
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

function closeSaveDiffModal() {
  if (saving.value) return;
  showSaveDiffModal.value = false;
}

async function reloadPropertiesWithGuard() {
  if (hasUnsavedChanges.value) {
    showDiscardConfirm.value = true;
    return;
  }

  await loadProperties();
}

async function confirmReloadDiscard() {
  showDiscardConfirm.value = false;
  await loadProperties();
}

function handleCategoryChange(category: string) {
  activeCategory.value = category;
  window.scrollTo({ top: 0, behavior: "smooth" });
}

function handleSearchUpdate(value: string) {
  searchQuery.value = value;
}

async function loadPlugins() {
  if (!store.currentServerId) return;

  pluginsLoading.value = true;
  error.value = null;
  try {
    plugins.value = await m_pluginApi.m_getPlugins(store.currentServerId);
    loadedPlugins.value = new Set();
    nextTick(() => {
      setupIntersectionObserver();
    });
  } catch (e) {
    error.value = String(e);
    plugins.value = [];
  } finally {
    pluginsLoading.value = false;
  }
}

function setupIntersectionObserver() {
  if (observer.value) {
    observer.value.disconnect();
  }

  observer.value = new IntersectionObserver(
    (intersectionEntries) => {
      intersectionEntries.forEach((entry) => {
        if (entry.isIntersecting) {
          const pluginElement = entry.target as HTMLElement;
          const pluginFileName = pluginElement.getAttribute("data-plugin-file-name");
          if (pluginFileName) {
            loadPluginDetails(pluginFileName);
          }
        }
      });
    },
    {
      rootMargin: "200px 0px",
      threshold: 0.1,
    },
  );

  nextTick(() => {
    const pluginElements = document.querySelectorAll<HTMLElement>(".plugin-list-item");
    pluginElements.forEach((element) => {
      observer.value?.observe(element);
    });
  });
}

async function loadPluginDetails(pluginFileName: string) {
  if (loadedPlugins.value.has(pluginFileName)) {
    return;
  }

  loadedPlugins.value.add(pluginFileName);
}

async function togglePlugin(plugin: m_PluginInfo) {
  if (!store.currentServerId) return;

  if (!plugin.file_name.endsWith(".jar") && !plugin.file_name.endsWith(".jar.disabled")) {
    alert(i18n.t("config.not_jar_file", { file: plugin.file_name }));
    return;
  }

  try {
    await m_pluginApi.m_togglePlugin(store.currentServerId, plugin.file_name, !plugin.enabled);
    plugin.enabled = !plugin.enabled;
  } catch (e) {
    error.value = String(e);
  }
}

async function deletePlugin(plugin: m_PluginInfo) {
  if (!store.currentServerId) return;
  try {
    const pluginElement = document.querySelector<HTMLElement>(
      `.plugin-list-item[data-plugin-file-name="${plugin.file_name}"]`,
    );
    if (pluginElement) {
      const originalHeight = pluginElement.offsetHeight;
      pluginElement.style.height = `${originalHeight}px`;
      pluginElement.style.flexShrink = "0";

      pluginElement.classList.add("deleting");

      setTimeout(async () => {
        const serverId = store.currentServerId;
        if (!serverId) return;
        await m_pluginApi.m_deletePlugin(serverId, plugin.file_name);
        plugins.value = plugins.value.filter((p) => p.file_name !== plugin.file_name);
        if (selectedPlugin.value?.file_name === plugin.file_name) {
          selectedPlugin.value = null;
        }
      }, 500);
    } else {
      // 如果找不到元素，直接删除
      await m_pluginApi.m_deletePlugin(store.currentServerId, plugin.file_name);
      plugins.value = plugins.value.filter((p) => p.file_name !== plugin.file_name);
      if (selectedPlugin.value?.file_name === plugin.file_name) {
        selectedPlugin.value = null;
      }
    }
  } catch (e) {
    error.value = String(e);
  }
}

async function reloadPlugins() {
  if (!store.currentServerId) return;
  try {
    await m_pluginApi.m_reloadPlugins(store.currentServerId);
    await loadPlugins();
  } catch (e) {
    error.value = String(e);
  }
}

async function handlePluginClick(plugin: m_PluginInfo) {
  if (selectedPlugin.value?.file_name === plugin.file_name) {
    selectedPlugin.value = null;
  } else {
    if (!plugin.config_files || (plugin.config_files.length === 0 && plugin.has_config_folder)) {
      try {
        const serverId = store.currentServerId;
        if (!serverId) return;
        const configFiles = await m_pluginApi.m_getPluginConfigFiles(
          serverId,
          plugin.file_name,
          plugin.name,
        );
        const updatedPlugin = {
          ...plugin,
          config_files: configFiles,
        };
        selectedPlugin.value = updatedPlugin;
        const pluginIndex = plugins.value.findIndex((p) => p.file_name === plugin.file_name);
        if (pluginIndex !== -1) {
          plugins.value[pluginIndex] = updatedPlugin;
        }
      } catch (e) {
        console.error("Failed to load plugin config files:", e);
        selectedPlugin.value = plugin;
      }
    } else {
      selectedPlugin.value = plugin;
    }
  }
}

async function openPluginFolder(plugin: m_PluginInfo) {
  if (!store.currentServerId) return;
  const server = store.servers.find((s) => s.id === store.currentServerId);
  if (!server) return;

  const basePath = server.path.replace(/[/\\]$/, "");
  const pluginConfigPath = `${basePath}${basePath.includes("\\") ? "\\" : "/"}plugins${basePath.includes("\\") ? "\\" : "/"}${plugin.name}`;

  try {
    await systemApi.openFolder(pluginConfigPath);
  } catch (e) {
    error.value = String(e);
  }
}

async function openConfigFile(config: m_PluginConfigFile) {
  try {
    await systemApi.openFile(config.file_path);
  } catch (e) {
    error.value = String(e);
  }
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + " KB";
  return (bytes / (1024 * 1024)).toFixed(2) + " MB";
}

onActivated(async () => {
  await loadProperties();
  await loadPlugins();
});
</script>

<template>
  <div class="config-view animate-fade-in">
    <div class="config-header">
      <div class="config-tabs-row">
        <SLTabBar v-model="activeTab" :tabs="configTabs" :level="1" />
        <SLTabBar
          v-if="activeTab === 'properties'"
          class="config-editor-mode-bar"
          :modelValue="editorMode"
          :tabs="editorModeTabs"
          :level="2"
          @update:modelValue="handleEditorModeChange"
        />
      </div>
    </div>

    <div v-if="!currentServerId" class="empty-state">
      <p class="text-body">{{ i18n.t("config.no_server") }}</p>
    </div>

    <template v-else>
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button class="banner-close" @click="error = null">x</button>
      </div>
      <div v-if="successMsg" class="success-banner">
        <span>{{ i18n.t("config.saved") }}</span>
      </div>

      <template v-if="activeTab === 'properties'">
        <div v-show="editorMode === 'visual'">
          <ConfigCategories
            :categories="categories"
            :activeCategory="activeCategory"
            :searchQuery="searchQuery"
            @updateCategory="handleCategoryChange"
            @updateSearch="handleSearchUpdate"
          />

          <div v-if="loading" class="loading-state">
            <SLSpinner size="lg" />
            <span>{{ i18n.t("config.loading") }}</span>
          </div>

          <div v-else class="config-entries">
            <div v-for="entry in filteredEntries" :key="entry.key" class="config-entry glass-card">
              <div class="entry-header">
                <div class="entry-key-row">
                  <span class="entry-key text-mono">{{ entry.key }}</span>
                </div>
                <p
                  v-if="getTranslatedPropertyDescription(entry.key)"
                  class="entry-desc text-caption"
                >
                  {{ getTranslatedPropertyDescription(entry.key) }}
                </p>
              </div>
              <div class="entry-control">
                <template
                  v-if="
                    entry.value_type === 'boolean' ||
                    editValues[entry.key] === 'true' ||
                    editValues[entry.key] === 'false'
                  "
                >
                  <SLSwitch
                    :modelValue="editValues[entry.key] === 'true'"
                    @update:modelValue="updateValue(entry.key, $event)"
                  />
                </template>
                <template v-else-if="entry.key === 'gamemode'">
                  <SLSelect
                    :modelValue="editValues[entry.key]"
                    :options="gamemodeOptions"
                    @update:modelValue="updateValue(entry.key, $event)"
                    style="width: 200px"
                  />
                </template>
                <template v-else-if="entry.key === 'difficulty'">
                  <SLSelect
                    :modelValue="editValues[entry.key]"
                    :options="difficultyOptions"
                    @update:modelValue="updateValue(entry.key, $event)"
                    style="width: 200px"
                  />
                </template>
                <template v-else>
                  <SLInput
                    :modelValue="editValues[entry.key]"
                    :placeholder="entry.default_value"
                    @update:modelValue="updateValue(entry.key, $event)"
                    style="width: 200px"
                  />
                </template>
              </div>
            </div>
            <div v-if="filteredEntries.length === 0 && !loading" class="empty-state">
              <p class="text-caption">{{ i18n.t("config.no_config") }}</p>
            </div>
          </div>
        </div>

        <div v-show="editorMode === 'source'">
          <div class="source-editor-wrap">
            <ConfigSourceEditor
              :modelValue="sourceDraftText"
              @update:modelValue="updateSourceDraft"
            />
            <p v-if="sourceParseError" class="source-parse-error">
              {{ sourceParseError }}
            </p>
          </div>
        </div>

        <div
          class="config-floating-actions glass-strong"
          :class="{ 'config-floating-actions--unsaved': hasUnsavedChanges }"
        >
          <div class="floating-status text-caption">{{ saveStatusText }}</div>
          <div class="floating-center">
            <SLButton
              variant="secondary"
              size="sm"
              iconOnly
              class="config-floating-icon-btn"
              @click="reloadPropertiesWithGuard"
            >
              <RefreshCw :size="16" />
            </SLButton>
          </div>
          <div class="floating-right">
            <SLButton
              variant="primary"
              size="sm"
              iconOnly
              class="config-floating-icon-btn"
              :class="
                hasUnsavedChanges
                  ? 'config-floating-icon-btn--unsaved'
                  : 'config-floating-icon-btn--idle'
              "
              :disabled="!hasUnsavedChanges"
              :loading="saving"
              @click="saveProperties"
            >
              <span
                class="save-icon-wrap"
                :class="{ 'save-icon-wrap--unsaved': hasUnsavedChanges && !saving }"
              >
                <Save :size="16" />
              </span>
            </SLButton>
          </div>
        </div>
      </template>

      <template v-if="activeTab === 'plugins'">
        <div class="plugins-header">
          <h3>{{ i18n.t("config.server_plugins") }}</h3>
          <div class="plugins-header-actions">
            <SLButton @click="loadPlugins" :loading="pluginsLoading" variant="secondary" size="sm">
              <RotateCcw :size="16" />
              {{ i18n.t("config.refresh_list") }}
            </SLButton>
            <SLButton
              @click="reloadPlugins"
              :loading="pluginsLoading"
              variant="danger"
              size="sm"
              class="reload-btn"
              :title="i18n.t('config.reload_plugins_warning')"
            >
              <RefreshCw :size="14" />
              {{ i18n.t("config.reload_plugins") }}
            </SLButton>
          </div>
        </div>

        <div v-if="pluginsLoading" class="loading-state">
          <SLSpinner size="lg" />
          <span>{{ i18n.t("config.loading_plugins") }}</span>
        </div>

        <div v-else class="plugins-container">
          <div v-if="plugins.length === 0" class="empty-state">
            <p class="text-caption">{{ i18n.t("config.no_plugins") }}</p>
          </div>

          <div v-else class="plugin-list-view">
            <div
              v-for="plugin in plugins"
              :key="plugin.file_name"
              class="plugin-list-item"
              :class="{
                disabled: !plugin.enabled,
                expanded: selectedPlugin?.file_name === plugin.file_name,
              }"
              :data-plugin-file-name="plugin.file_name"
              @click="handlePluginClick(plugin)"
            >
              <div class="plugin-list-icon">
                {{ plugin.name.charAt(0).toUpperCase() }}
              </div>
              <div class="plugin-list-info">
                <div class="plugin-list-header">
                  <h4>{{ plugin.name }}</h4>
                  <span class="plugin-list-version">{{ plugin.version }}</span>
                  <div v-if="plugin.has_config_folder" class="config-badge">
                    <Settings :size="14" />
                    <template v-if="selectedPlugin?.file_name === plugin.file_name">
                      {{ plugin.config_files.length }} {{ i18n.t("config.config_files_count") }}
                    </template>
                    <template v-else>
                      {{ i18n.t("config.has_config_folder") }}
                    </template>
                  </div>
                  <div v-else class="no-config-badge">
                    <FileText :size="14" />
                    {{ i18n.t("config.no_config_files") }}
                  </div>
                </div>
                <div
                  v-if="selectedPlugin?.file_name === plugin.file_name"
                  class="plugin-list-details"
                >
                  <p>{{ i18n.t("config.author") }}: {{ plugin.author }}</p>
                  <p v-if="plugin.description">{{ plugin.description }}</p>
                  <p>{{ formatFileSize(plugin.file_size) }}</p>

                  <div v-if="plugin.has_config_folder" class="plugin-config-section">
                    <div class="plugin-config-section-header">
                      <h5>{{ i18n.t("config.config_files") }}</h5>
                      <SLButton
                        size="sm"
                        variant="secondary"
                        @click.stop="openPluginFolder(plugin)"
                      >
                        <FolderOpen :size="14" />
                        {{ i18n.t("common.open_folder") }}
                      </SLButton>
                    </div>
                    <div v-if="plugin.config_files.length > 0" class="plugin-config-files-list">
                      <div
                        v-for="config in plugin.config_files"
                        :key="config.file_name"
                        class="plugin-config-file-item"
                        @click.stop="openConfigFile(config)"
                      >
                        <div class="plugin-config-file-name">{{ config.file_name }}</div>
                        <div class="plugin-config-file-type">{{ config.file_type }}</div>
                        <div class="plugin-config-file-actions">
                          <SLButton size="sm" variant="secondary">
                            <Edit :size="14" />
                            {{ i18n.t("config.open") }}
                          </SLButton>
                        </div>
                      </div>
                    </div>
                    <div v-else class="empty-state">
                      <p class="text-caption">{{ i18n.t("config.empty_config_folder") }}</p>
                    </div>
                  </div>
                </div>
              </div>
              <div class="plugin-list-actions">
                <SLSwitch
                  :modelValue="plugin.enabled"
                  @update:modelValue="togglePlugin(plugin)"
                  :title="plugin.enabled ? i18n.t('config.disable') : i18n.t('config.enable')"
                />
                <button
                  @click.stop="deletePlugin(plugin)"
                  class="icon-btn"
                  :title="i18n.t('config.delete')"
                >
                  <Trash2 :size="16" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </template>

      <SLConfirmDialog
        :visible="showDiscardConfirm"
        :title="i18n.t('config.discard_title')"
        :message="i18n.t('config.discard_message')"
        :confirmText="i18n.t('config.discard_confirm')"
        :cancelText="i18n.t('common.cancel')"
        confirmVariant="danger"
        @confirm="confirmReloadDiscard"
        @close="showDiscardConfirm = false"
      />

      <SLModal
        :visible="showSaveDiffModal"
        :title="i18n.t('config.diff_modal_title')"
        width="1040px"
        :close-on-overlay="!saving"
        @close="closeSaveDiffModal"
      >
        <div class="source-diff-summary text-caption">
          <span>{{ i18n.t("config.diff_original") }} → {{ i18n.t("config.diff_after_save") }}</span>
          <span class="diff-count diff-count-add">+{{ sourceDiffStats.additions }}</span>
          <span class="diff-count diff-count-del">-{{ sourceDiffStats.deletions }}</span>
        </div>
        <ConfigSourceDiffView :original="sourceDiffOriginalText" :modified="sourceDiffTargetText" />
        <template #footer>
          <div class="diff-modal-actions">
            <SLButton variant="secondary" :disabled="saving" @click="closeSaveDiffModal">
              {{ i18n.t("common.cancel") }}
            </SLButton>
            <SLButton variant="primary" :loading="saving" @click="confirmSaveProperties">
              {{ i18n.t("config.confirm_save") }}
            </SLButton>
          </div>
        </template>
      </SLModal>
    </template>
  </div>
</template>
