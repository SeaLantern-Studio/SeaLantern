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
const propertiesDialogs = viewModel.propertiesDialogs;
const propertiesSectionBindings = viewModel.propertiesSectionBindings;
const pluginsState = viewModel.pluginsState;
const configTabs = viewModel.configTabs;
const editorModeTabs = viewModel.editorModeTabs;
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
          v-bind="propertiesSectionBindings.sectionProps.value"
          @updateCategory="propertiesSectionBindings.sectionHandlers.updateCategory"
          @updateSearch="propertiesSectionBindings.sectionHandlers.updateSearch"
          @updateSourceDraft="propertiesSectionBindings.sectionHandlers.updateSourceDraft"
          @updateCompareTargetSourceDraft="
            propertiesSectionBindings.sectionHandlers.updateCompareTargetSourceDraft
          "
          @updateValue="propertiesSectionBindings.sectionHandlers.updateValue"
          @updateCompareTargetValue="
            propertiesSectionBindings.sectionHandlers.updateCompareTargetValue
          "
          @addSourceValue="propertiesSectionBindings.sectionHandlers.addSourceValue"
          @addTargetValue="propertiesSectionBindings.sectionHandlers.addTargetValue"
          @updateCompareTargetServer="
            propertiesSectionBindings.sectionHandlers.updateCompareTargetServer
          "
          @reloadCurrent="propertiesSectionBindings.sectionHandlers.reloadCurrent"
          @reloadCompare="propertiesSectionBindings.sectionHandlers.reloadCompare"
          @saveProperties="propertiesSectionBindings.sectionHandlers.saveProperties"
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
        :visible="propertiesDialogs.discardDialog.value.visible"
        :title="propertiesDialogs.discardDialog.value.title"
        :message="propertiesDialogs.discardDialog.value.message"
        :confirmText="propertiesDialogs.discardDialog.value.confirmText"
        :cancelText="propertiesDialogs.discardDialog.value.cancelText"
        :confirmVariant="propertiesDialogs.discardDialog.value.confirmVariant"
        @confirm="propertiesDialogs.confirmReloadDiscard"
        @close="propertiesDialogs.closeDiscardDialog"
      />

      <SLModal
        :visible="propertiesDialogs.saveDiffDialog.value.visible"
        :title="propertiesDialogs.saveDiffDialog.value.title"
        :width="propertiesDialogs.saveDiffDialog.value.width"
        :close-on-overlay="propertiesDialogs.saveDiffDialog.value.closeOnOverlay"
        @close="propertiesDialogs.closeSaveDiffModal"
      >
        <div
          v-for="diffItem in propertiesDialogs.saveDiffDialog.value.items"
          :key="`${diffItem.serverId}-${diffItem.filePath}`"
          class="source-diff-block"
        >
          <div class="source-diff-title-row text-caption">
            <span class="source-diff-server">{{ diffItem.serverName }}</span>
            <SLTooltip :content="diffItem.filePath">
              <span class="source-diff-path-hint">i</span>
            </SLTooltip>
            <span
              >{{ propertiesDialogs.saveDiffDialog.value.originalLabel }} →
              {{ propertiesDialogs.saveDiffDialog.value.savedLabel }}</span
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
              :disabled="propertiesDialogs.saveDiffDialog.value.saving"
              @click="propertiesDialogs.closeSaveDiffModal"
            >
              {{ propertiesDialogs.saveDiffDialog.value.cancelText }}
            </SLButton>
            <SLButton
              variant="primary"
              :loading="propertiesDialogs.saveDiffDialog.value.saving"
              @click="propertiesDialogs.confirmSaveProperties"
            >
              {{ propertiesDialogs.saveDiffDialog.value.confirmText }}
            </SLButton>
          </div>
        </template>
      </SLModal>
    </template>
  </div>
</template>
