<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import { getLocalizedPluginDescription } from "@type/plugin";
import PluginDependentSettingsList from "@views/plugins/PluginDependentSettingsList.vue";
import PluginPresetPickerCard from "@views/plugins/PluginPresetPickerCard.vue";
import PluginSettingsFormCard from "@views/plugins/PluginSettingsFormCard.vue";
import { usePluginCategorySettings } from "@views/plugins/usePluginCategorySettings";
import { Palette } from "lucide-vue-next";

const props = defineProps<{
  pluginId: string;
}>();

const pluginStore = usePluginStore();

const {
  plugin,
  settingsForm,
  dependentSettingsForms,
  dependentPlugins,
  loading,
  saving,
  categoryLabel,
  showDependents,
  pluginPresets,
  isThemeProvider,
  getFieldStringValue,
  getFieldSelectValue,
  getFieldOptions,
  updateSettingsField,
  applyPreset,
  resetToDefault,
} = usePluginCategorySettings({
  pluginId: () => props.pluginId,
});

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
  <div class="category-view">
    <div v-if="loading" class="loading-state">
      <div class="loading-spinner"></div>
      <span>{{ i18n.t("common.loading") }}</span>
    </div>

    <template v-else-if="plugin">
      <header class="category-header">
        <div class="header-icon">
          <img
            v-if="pluginStore.icons[plugin.manifest.id]"
            :src="pluginStore.icons[plugin.manifest.id]"
            :alt="plugin.manifest.name"
          />
          <Palette v-else :size="24" />
        </div>
        <div class="header-info">
          <h1>{{ categoryLabel }}</h1>
          <p class="header-desc">
            {{ getLocalizedPluginDescription(plugin.manifest, i18n.getLocale()) }}
          </p>
        </div>
      </header>

      <PluginPresetPickerCard
        v-if="isThemeProvider && pluginPresets"
        :title="i18n.t('plugins.preset_theme')"
        :presets="pluginPresets"
        :selected-preset="settingsForm['preset']"
        @select="applyPreset"
      />

      <PluginSettingsFormCard
        v-if="plugin.manifest.settings?.length"
        :title="`${plugin.manifest.name} ${i18n.t('plugins.settings')}`"
        :fields="plugin.manifest.settings"
        :form="settingsForm"
        :get-field-string-value="getFieldStringValue"
        :get-field-select-value="getFieldSelectValue"
        :get-field-options="getFieldOptions"
        @update-field="handleMainFieldUpdate"
      />

      <PluginDependentSettingsList
        v-if="showDependents"
        :plugins="dependentPlugins"
        :forms="dependentSettingsForms"
        show-header
        :header-title="i18n.t('plugins.related_plugins')"
        :header-description="i18n.t('plugins.related_plugins_desc', { name: plugin.manifest.name })"
        :item-description="i18n.t('plugins.related_plugins_desc', { name: plugin.manifest.name })"
        :get-field-string-value="getFieldStringValue"
        :get-field-select-value="getFieldSelectValue"
        :get-field-options="getFieldOptions"
        @update-field="handleDependentFieldUpdate"
      />

      <div class="action-buttons">
        <SLButton variant="secondary" @click="resetToDefault">{{
          i18n.t("plugins.reset_default")
        }}</SLButton>
        <span class="auto-save-hint">{{
          saving ? i18n.t("plugins.saving") : i18n.t("plugins.auto_saved")
        }}</span>
      </div>
    </template>

    <div v-else class="not-found">
      <p>{{ i18n.t("plugins.not_found") }}</p>
    </div>
  </div>
</template>

<style scoped>
.category-view {
  padding: 24px;
  max-width: 900px;
  margin: 0 auto;
  font-family: var(--font-sans);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--sl-text-secondary);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.category-header {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 24px;
}

.header-icon {
  width: 64px;
  height: 64px;
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.header-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.header-icon svg {
  width: 32px;
  height: 32px;
  color: var(--sl-text-secondary);
}

.header-info h1 {
  font-size: var(--sl-font-size-3xl);
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 4px;
}

.header-desc {
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-base);
  margin: 0;
}

.action-buttons {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 12px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
}

.auto-save-hint {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
  opacity: 0.7;
}

.not-found {
  text-align: center;
  padding: 48px;
  color: var(--sl-text-secondary);
}
</style>
