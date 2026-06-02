<script setup lang="ts">
import { useRoute } from "vue-router";
import SLButton from "@components/common/SLButton.vue";
import SLModal from "@components/common/SLModal.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import SLTooltip from "@components/common/SLTooltip.vue";
import { SLTabBar } from "@components/common";
import { i18n } from "@language";
import { FileDiff } from "lucide-vue-next";

import ConfigSourceDiffView from "@components/config/ConfigSourceDiffView.vue";
import ConfigPluginsSection from "@components/config/ConfigPluginsSection.vue";
import ConfigPropertiesSection from "@components/config/ConfigPropertiesSection.vue";
import ConfigStartupSection from "@components/config/ConfigStartupSection.vue";
import { useConfigViewModel } from "@views/config/useConfigViewModel";
import "@styles/plugin-list.css";
import "@styles/views/ConfigView.css";

const route = useRoute();
const viewModel = useConfigViewModel({ route });
const error = viewModel.error;
const successMsg = viewModel.successMsg;
const activeTab = viewModel.activeTab;
const currentServerId = viewModel.currentServerId;
const currentServer = viewModel.currentServer;
const serverPath = viewModel.serverPath;
const propertiesEditor = viewModel.propertiesEditor;
const compare = viewModel.compare;
const pluginsState = viewModel.pluginsState;
const configTabs = viewModel.configTabs;
const editorModeTabs = viewModel.editorModeTabs;
const gamemodeOptions = viewModel.gamemodeOptions;
const difficultyOptions = viewModel.difficultyOptions;
const translatedDescriptionByKey = viewModel.translatedDescriptionByKey;
const configSaveDiffModalWidth = viewModel.configSaveDiffModalWidth;
const currentServerName = viewModel.currentServerName;
const compareTargetServerName = viewModel.compareTargetServerName;
const handleStartupConfigSaved = viewModel.handleStartupConfigSaved;
const setError = viewModel.setError;
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
          :sourceServerName="currentServerName"
          :targetServerName="compareTargetServerName"
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

      <template v-if="activeTab === 'startup'">
        <ConfigStartupSection
          :serverPath="serverPath"
          :defaultMaxMemory="currentServer?.max_memory ?? 2048"
          :defaultMinMemory="currentServer?.min_memory ?? 512"
          @saved="handleStartupConfigSaved"
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
