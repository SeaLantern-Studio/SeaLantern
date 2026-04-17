<script setup lang="ts">
import type { ComponentPublicInstance } from "vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLButton from "@components/common/SLButton.vue";
import type { m_PluginConfigFile, m_PluginInfo } from "@api/mcs_plugins";
import { i18n } from "@language";
import {
  Trash2,
  RefreshCw,
  Settings,
  FileText,
  RotateCcw,
  FolderOpen,
  Edit,
} from "lucide-vue-next";

interface Props {
  plugins: m_PluginInfo[];
  pluginsLoading: boolean;
  selectedPlugin: m_PluginInfo | null;
}

defineProps<Props>();

const emit = defineEmits<{
  refreshList: [];
  reloadPlugins: [];
  pluginClick: [plugin: m_PluginInfo];
  togglePlugin: [plugin: m_PluginInfo];
  deletePlugin: [plugin: m_PluginInfo];
  registerPluginRow: [payload: { pluginFileName: string; element: HTMLElement | null }];
  openPluginFolder: [plugin: m_PluginInfo];
  openConfigFile: [config: m_PluginConfigFile];
}>();

function formatFileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function setPluginRowRef(pluginFileName: string) {
  return (element: Element | ComponentPublicInstance | null) => {
    emit("registerPluginRow", {
      pluginFileName,
      element: element instanceof HTMLElement ? element : null,
    });
  };
}
</script>

<template>
  <div class="plugins-header">
    <h3>{{ i18n.t("config.server_plugins") }}</h3>
    <div class="plugins-header-actions">
      <SLButton
        @click="emit('refreshList')"
        :loading="pluginsLoading"
        variant="secondary"
        size="sm"
      >
        <RotateCcw :size="16" />
        {{ i18n.t("config.refresh_list") }}
      </SLButton>
      <SLButton
        @click="emit('reloadPlugins')"
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
        :ref="setPluginRowRef(plugin.file_name)"
        @click="emit('pluginClick', plugin)"
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
          <div v-if="selectedPlugin?.file_name === plugin.file_name" class="plugin-list-details">
            <p>{{ i18n.t("config.author") }}: {{ plugin.author }}</p>
            <p v-if="plugin.description">{{ plugin.description }}</p>
            <p>{{ formatFileSize(plugin.file_size) }}</p>

            <div v-if="plugin.has_config_folder" class="plugin-config-section">
              <div class="plugin-config-section-header">
                <h5>{{ i18n.t("config.config_files") }}</h5>
                <SLButton
                  size="sm"
                  variant="secondary"
                  @click.stop="emit('openPluginFolder', plugin)"
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
                  @click.stop="emit('openConfigFile', config)"
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
            @update:modelValue="emit('togglePlugin', plugin)"
            :title="plugin.enabled ? i18n.t('config.disable') : i18n.t('config.enable')"
          />
          <button
            @click.stop="emit('deletePlugin', plugin)"
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
