<script setup lang="ts">
import { ref, computed, watch, onMounted, onActivated } from "vue";
import { useRoute } from "vue-router";
import SLButton from "@components/common/SLButton.vue";
import SLModal from "@components/common/SLModal.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import SLTooltip from "@components/common/SLTooltip.vue";
import { SLTabBar } from "@components/common";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import { FileDiff } from "lucide-vue-next";

import ConfigSourceDiffView from "@components/config/ConfigSourceDiffView.vue";
import ConfigPluginsSection from "@components/config/ConfigPluginsSection.vue";
import ConfigPropertiesSection from "@components/config/ConfigPropertiesSection.vue";
import { useConfigPlugins } from "@views/config/useConfigPlugins";
import { useConfigCompare } from "@views/config/useConfigCompare";
import { useConfigPropertiesEditor } from "@views/config/useConfigPropertiesEditor";
import "@styles/plugin-list.css";
import "@styles/views/ConfigView.css";

const route = useRoute();
const store = useServerStore();

const error = ref<string | null>(null);
const successMsg = ref<string | null>(null);
const activeTab = ref<"properties" | "plugins">("properties");
const configSaveDiffModalWidth = "1040px";

const currentServerId = computed(() => store.currentServerId);
const currentServer = computed(
  () => store.servers.find((s) => s.id === store.currentServerId) || null,
);
const serverPath = computed(() => currentServer.value?.path || "");

function buildServerPropertiesPath(path: string) {
  const basePath = path.replace(/[/\\]$/, "");
  if (!basePath) {
    return "server.properties";
  }

  const separator = basePath.includes("\\") ? "\\" : "/";
  return `${basePath}${separator}server.properties`;
}

const serverPropertiesPath = computed(() => buildServerPropertiesPath(serverPath.value));

function setError(message: string | null) {
  error.value = message;
}

function setSuccess(message: string | null) {
  successMsg.value = message;
}

function updateCurrentServerPort(port: string) {
  if (!port) return;

  const activeServer = store.servers.find((s) => s.id === store.currentServerId);
  if (activeServer) {
    activeServer.port = parseInt(port) || 25565;
  }
}

const propertiesEditor = useConfigPropertiesEditor({
  serverPath,
  serverPropertiesPath,
  currentServerId,
  currentServerName: computed(() => currentServer.value?.name || ""),
  setError,
  setSuccess,
  updateCurrentServerPort,
});

const compare = useConfigCompare({
  currentServerId,
  servers: computed(() => store.servers),
  sourceEntries: propertiesEditor.entries,
  sourceValues: propertiesEditor.editValues,
  sourceNumericFieldErrors: propertiesEditor.numericFieldErrors,
  activeCategory: propertiesEditor.activeCategory,
  searchQuery: propertiesEditor.searchQuery,
  getTranslatedPropertyDescription: propertiesEditor.getTranslatedPropertyDescription,
  setError,
});

propertiesEditor.bindCompareContext({
  compareMode: compare.compareMode,
  compareTargetServerId: compare.compareTargetServerId,
  compareTargetEntries: compare.compareTargetEntries,
  compareTargetPath: compare.compareTargetPath,
  compareTargetServerName: computed(
    () => compare.compareTargetServer.value?.name || i18n.t("config.compare.target_server"),
  ),
  compareTargetServerPropertiesPath: compare.compareTargetServerPropertiesPath,
  compareTargetDraftValues: compare.compareTargetDraftValues,
  compareTargetLoadedValues: compare.compareTargetLoadedValues,
  compareTargetSourceDraftText: compare.compareTargetSourceDraftText,
  compareTargetLoadedSourceText: compare.compareTargetLoadedSourceText,
  compareTargetNumericFieldErrors: compare.compareTargetNumericFieldErrors,
  loadCompareProperties: compare.loadCompareProperties,
  applyParsedCompareTargetState: compare.applyParsedCompareTargetState,
  applyCompareTargetSourceDraftToVisualState: compare.applyCompareTargetSourceDraftToVisualState,
  buildCompareTargetPreviewSource: compare.buildCompareTargetPreviewSource,
  prepareCompareTargetSourceDraftForSourceMode:
    compare.prepareCompareTargetSourceDraftForSourceMode,
  updateCompareTargetSourceDraft: compare.updateCompareTargetSourceDraft,
  captureDifferenceCategorySnapshot: compare.captureDifferenceCategorySnapshot,
});

const pluginsState = useConfigPlugins({
  currentServerId,
  getCurrentServer: () => currentServer.value,
  setError,
});

const configTabs = computed(() => [
  {
    key: "properties",
    label: i18n.t("config.server_properties"),
    count: "i",
    countTitle: serverPropertiesPath.value,
  },
  { key: "plugins", label: i18n.t("config.server_plugins") },
]);

const editorModeTabs = computed(() => [
  { key: "visual", label: i18n.t("config.visual_mode") },
  { key: "source", label: i18n.t("config.source_mode") },
]);

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

const translatedDescriptionByKey = computed(() => {
  const result: Record<string, string> = {};
  propertiesEditor.filteredEntries.value.forEach((entry) => {
    result[entry.key] = propertiesEditor.getTranslatedPropertyDescription(entry.key);
  });
  return result;
});

onMounted(async () => {
  await store.refreshList();
  const routeId = route.params.id as string;
  if (routeId) {
    store.setCurrentServer(routeId);
  } else if (!store.currentServerId && store.servers.length > 0) {
    store.setCurrentServer(store.servers[0].id);
  }

  await propertiesEditor.loadProperties();
  await pluginsState.loadPlugins();
});

watch(
  () => store.currentServerId,
  async () => {
    if (store.currentServerId) {
      // 这是刻意的设计：切换当前服务器时直接进入新的对照上下文，不保留旧的对照侧草稿。
      if (compare.compareTargetServerId.value === store.currentServerId) {
        compare.compareTargetServerId.value =
          compare.compareServerOptions.value[0]?.value?.toString() || "";
      }
      await propertiesEditor.loadProperties();
      await pluginsState.loadPlugins();
    }
  },
);

watch(compare.compareTargetServerId, async () => {
  if (compare.compareMode.value && compare.compareTargetServerId.value) {
    await compare.loadCompareProperties();
  }
});

watch(compare.hasCompareTargets, (hasTargets) => {
  if (hasTargets) {
    return;
  }

  // 这是刻意的设计：当没有可对照服务器时，直接退出对照模式并清空对照侧状态。
  compare.resetCompareState(true);
});

onActivated(async () => {
  await propertiesEditor.loadProperties();
  await pluginsState.loadPlugins();
});
</script>

<template>
  <div class="config-view animate-fade-in">
    <div class="config-header">
      <div class="config-tabs-row">
        <SLTabBar v-model="activeTab" :tabs="configTabs" :level="1" />
        <div v-if="activeTab === 'properties'" class="config-properties-header-actions">
          <SLButton
            v-if="compare.hasCompareTargets.value"
            size="sm"
            :variant="compare.compareMode.value ? 'primary' : 'secondary'"
            class="config-compare-toggle"
            @click="compare.handleCompareModeChange(!compare.compareMode.value)"
          >
            <FileDiff :size="16" />
            {{ i18n.t("config.compare.toggle") }}
          </SLButton>
          <SLTabBar
            class="config-editor-mode-bar"
            :modelValue="propertiesEditor.editorMode.value"
            :tabs="editorModeTabs"
            :level="2"
            @update:modelValue="propertiesEditor.handleEditorModeChange"
          />
        </div>
      </div>
    </div>

    <div v-if="!currentServerId" class="empty-state">
      <p class="text-body">{{ i18n.t("config.no_server") }}</p>
    </div>

    <template v-else>
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button class="banner-close" @click="setError(null)">x</button>
      </div>
      <div v-if="successMsg" class="success-banner">
        <span>{{ i18n.t("config.saved") }}</span>
      </div>

      <template v-if="activeTab === 'properties'">
        <ConfigPropertiesSection
          :editorMode="propertiesEditor.editorMode.value"
          :loading="propertiesEditor.loading.value"
          :compareLoading="compare.compareLoading.value"
          :compareMode="compare.compareMode.value"
          :hasCompareTargets="compare.hasCompareTargets.value"
          :compareTargetServerId="compare.compareTargetServerId.value"
          :compareServerOptions="compare.compareServerOptions.value"
          :compareDifferenceBadgeText="compare.compareDifferenceBadgeText.value"
          :comparePanelRows="compare.comparePanelRows.value"
          :sourceServerName="currentServer?.name || i18n.t('config.compare.source_server')"
          :targetServerName="
            compare.compareTargetServer.value?.name || i18n.t('config.compare.target_server')
          "
          :categories="propertiesEditor.categories.value"
          :activeCategory="propertiesEditor.activeCategory.value"
          :searchQuery="propertiesEditor.searchQuery.value"
          :filteredEntries="propertiesEditor.filteredEntries.value"
          :translatedDescriptionByKey="translatedDescriptionByKey"
          :editValues="propertiesEditor.editValues.value"
          :numericFieldErrors="propertiesEditor.numericFieldErrors.value"
          :gamemodeOptions="gamemodeOptions"
          :difficultyOptions="difficultyOptions"
          :sourceDraftText="propertiesEditor.sourceDraftText.value"
          :compareTargetSourceDraftText="compare.compareTargetSourceDraftText.value"
          :sourceParseError="propertiesEditor.sourceParseError.value"
          :hasUnsavedChanges="propertiesEditor.hasUnsavedChanges.value"
          :saveStatusText="propertiesEditor.saveStatusText.value"
          :saving="propertiesEditor.saving.value"
          :reloadCurrentTooltipText="propertiesEditor.reloadCurrentTooltipText.value"
          :reloadCompareTooltipText="propertiesEditor.reloadCompareTooltipText.value"
          @updateCategory="propertiesEditor.handleCategoryChange"
          @updateSearch="propertiesEditor.handleSearchUpdate"
          @updateSourceDraft="propertiesEditor.updateSourceDraft"
          @updateCompareTargetSourceDraft="propertiesEditor.updateCompareTargetSourceDraft"
          @updateValue="propertiesEditor.updateValue($event.key, $event.value)"
          @updateCompareTargetValue="compare.updateCompareTargetValue($event.key, $event.value)"
          @addSourceValue="propertiesEditor.updateValue($event.key, $event.value)"
          @addTargetValue="compare.updateCompareTargetValue($event.key, $event.value)"
          @updateCompareTargetServer="compare.handleCompareTargetServerChange"
          @reloadCurrent="propertiesEditor.reloadPropertiesWithGuard"
          @reloadCompare="propertiesEditor.reloadComparePropertiesWithGuard"
          @saveProperties="propertiesEditor.saveProperties"
        />
      </template>

      <template v-if="activeTab === 'plugins'">
        <ConfigPluginsSection
          :plugins="pluginsState.plugins.value"
          :pluginsLoading="pluginsState.pluginsLoading.value"
          :selectedPlugin="pluginsState.selectedPlugin.value"
          @refreshList="pluginsState.loadPlugins"
          @reloadPlugins="pluginsState.reloadPlugins"
          @pluginClick="pluginsState.handlePluginClick"
          @togglePlugin="pluginsState.togglePlugin"
          @deletePlugin="pluginsState.deletePlugin"
          @registerPluginRow="pluginsState.registerPluginRow"
          @openPluginFolder="pluginsState.openPluginFolder"
          @openConfigFile="pluginsState.openConfigFile"
        />
      </template>

      <SLConfirmDialog
        :visible="propertiesEditor.showDiscardConfirm.value"
        :title="propertiesEditor.discardConfirmTitle.value"
        :message="propertiesEditor.discardConfirmMessage.value"
        :confirmText="i18n.t('config.discard_confirm')"
        :cancelText="i18n.t('common.cancel')"
        confirmVariant="danger"
        @confirm="propertiesEditor.confirmReloadDiscard"
        @close="
          propertiesEditor.showDiscardConfirm.value = false;
          propertiesEditor.pendingReloadSide.value = null;
        "
      />

      <SLModal
        :visible="propertiesEditor.showSaveDiffModal.value"
        :title="i18n.t('config.diff_modal_title')"
        :width="configSaveDiffModalWidth"
        :close-on-overlay="!propertiesEditor.saving.value"
        @close="propertiesEditor.closeSaveDiffModal"
      >
        <div
          v-for="diffItem in propertiesEditor.pendingSaveItemsWithStats.value"
          :key="`${diffItem.serverId}-${diffItem.filePath}`"
          class="source-diff-block"
        >
          <div class="source-diff-title-row text-caption">
            <span class="source-diff-server">{{ diffItem.serverName }}</span>
            <SLTooltip :content="diffItem.filePath">
              <span class="source-diff-path-hint">i</span>
            </SLTooltip>
            <span
              >{{ i18n.t("config.diff_original") }} → {{ i18n.t("config.diff_after_save") }}</span
            >
            <span class="diff-count diff-count-add">+{{ diffItem.additions }}</span>
            <span class="diff-count diff-count-del">-{{ diffItem.deletions }}</span>
          </div>
          <ConfigSourceDiffView
            :original="diffItem.originalText"
            :modified="diffItem.modifiedText"
          />
        </div>
        <template #footer>
          <div class="diff-modal-actions">
            <SLButton
              variant="secondary"
              :disabled="propertiesEditor.saving.value"
              @click="propertiesEditor.closeSaveDiffModal"
            >
              {{ i18n.t("common.cancel") }}
            </SLButton>
            <SLButton
              variant="primary"
              :loading="propertiesEditor.saving.value"
              @click="propertiesEditor.confirmSaveProperties"
            >
              {{ i18n.t("config.confirm_save") }}
            </SLButton>
          </div>
        </template>
      </SLModal>
    </template>
  </div>
</template>
