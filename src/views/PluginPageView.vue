<script setup lang="ts">
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import { getLocalizedPluginName, getLocalizedPluginDescription } from "@type/plugin";
import PluginDependentSettingsList from "@views/plugins/PluginDependentSettingsList.vue";
import PluginPresetPickerCard from "@views/plugins/PluginPresetPickerCard.vue";
import PluginSettingsFormCard from "@views/plugins/PluginSettingsFormCard.vue";
import { usePluginPageSettings } from "@views/plugins/usePluginPageSettings";
import { ArrowLeft, Puzzle } from "lucide-vue-next";

const props = defineProps<{
  pluginId: string;
}>();

const router = useRouter();
const pluginStore = usePluginStore();
const {
  plugin,
  settingsForm,
  saving,
  loading,
  dependentPlugins,
  dependentSettingsForms,
  pluginPresets,
  isThemeProvider,
  getFieldStringValue,
  getFieldSelectValue,
  getFieldOptions,
  updateSettingsField,
  applyPreset,
  saveSettings,
  resetToDefault,
} = usePluginPageSettings({
  pluginId: () => props.pluginId,
});

function goBack() {
  router.back();
}

function handleMainFieldUpdate(key: string, value: string | number | boolean) {
  updateSettingsField(settingsForm, key, value);
}

function handleDependentFieldUpdate(
  pluginId: string,
  key: string,
  value: string | number | boolean,
) {
  const form = dependentSettingsForms[pluginId];
  if (!form) {
    return;
  }

  updateSettingsField(form, key, value);
}
</script>

<template>
  <div class="plugin-page-view">
    <div class="page-header">
      <SLButton variant="ghost" @click="goBack">
        <ArrowLeft :size="20" />
        <span>{{ i18n.t("plugins.back") }}</span>
      </SLButton>
      <h1 class="page-title" v-if="plugin">{{ plugin.manifest.name }}</h1>
    </div>

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>{{ i18n.t("common.loading") }}</span>
    </div>

    <div v-else-if="!plugin" class="empty-state">
      <p>{{ i18n.t("plugins.not_found") }}</p>
    </div>

    <div v-else class="plugin-content">
      <SLCard class="info-card">
        <div class="plugin-info">
          <div class="plugin-icon" v-if="pluginStore.icons[plugin.manifest.id]">
            <img :src="pluginStore.icons[plugin.manifest.id]" :alt="plugin.manifest.name" />
          </div>
          <div class="plugin-icon placeholder" v-else>
            <Puzzle :size="32" :stroke-width="1.5" />
          </div>
          <div class="plugin-details">
            <h2>
              {{ getLocalizedPluginName(plugin.manifest, i18n.getLocale()) }}
            </h2>
            <p class="description">
              {{ getLocalizedPluginDescription(plugin.manifest, i18n.getLocale()) }}
            </p>
            <div class="meta">
              <span class="version">v{{ plugin.manifest.version }}</span>
              <span class="author">{{ plugin.manifest.author.name }}</span>
            </div>
          </div>
        </div>
      </SLCard>

      <PluginPresetPickerCard
        v-if="isThemeProvider && pluginPresets"
        :title="i18n.t('plugins.preset_theme')"
        :presets="pluginPresets"
        :selected-preset="settingsForm['preset']"
        @select="applyPreset"
      />

      <PluginSettingsFormCard
        v-if="plugin.manifest.settings?.length"
        :title="i18n.t('plugins.plugin_settings')"
        :fields="plugin.manifest.settings"
        :form="settingsForm"
        :get-field-string-value="getFieldStringValue"
        :get-field-select-value="getFieldSelectValue"
        :get-field-options="getFieldOptions"
        @update-field="handleMainFieldUpdate"
      />

      <PluginDependentSettingsList
        :plugins="dependentPlugins"
        :forms="dependentSettingsForms"
        :item-description="i18n.t('plugins.depends_on', { name: plugin.manifest.name })"
        :get-field-string-value="getFieldStringValue"
        :get-field-select-value="getFieldSelectValue"
        :get-field-options="getFieldOptions"
        @update-field="handleDependentFieldUpdate"
      />

      <div class="action-buttons">
        <SLButton variant="secondary" @click="resetToDefault">{{
          i18n.t("plugins.reset_default")
        }}</SLButton>
        <SLButton variant="primary" :loading="saving" @click="saveSettings">{{
          i18n.t("plugins.save_settings")
        }}</SLButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.plugin-page-view {
  padding: 24px;
  max-width: 900px;
  margin: 0 auto;
  font-family: var(--sl-font-sans);
}

.page-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.back-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.page-title {
  font-size: var(--sl-font-size-3xl);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px;
  color: var(--text-secondary);
  gap: 16px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.plugin-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.info-card {
  padding: 20px;
}

.plugin-info {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.plugin-icon {
  width: 64px;
  height: 64px;
  border-radius: var(--radius-md);
  overflow: hidden;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.plugin-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.plugin-icon.placeholder {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.plugin-details h2 {
  margin: 0 0 8px;
  font-size: var(--sl-font-size-2xl);
  color: var(--text-primary);
}

.plugin-details .description {
  margin: 0 0 12px;
  color: var(--text-secondary);
  font-size: var(--sl-font-size-base);
  line-height: 1.5;
}

.plugin-details .meta {
  display: flex;
  gap: 12px;
  font-size: var(--sl-font-size-sm);
}

.plugin-details .version {
  color: var(--accent-primary);
  background: rgba(96, 165, 250, 0.1);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
}

.plugin-details .author {
  color: var(--text-secondary);
}

.action-buttons {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 8px;
}
</style>
