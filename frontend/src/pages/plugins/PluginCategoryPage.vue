<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import { Palette } from "@lucide/vue";
import PluginDependentSettingsList from "@src/components/plugins/PluginDependentSettingsList.vue";
import PluginPresetPickerCard from "@src/components/plugins/PluginPresetPickerCard.vue";
import PluginSettingsFormCard from "@src/components/plugins/PluginSettingsFormCard.vue";
import { getLocalizedPluginDescription } from "@type/plugin";
import { usePluginCategoryPage } from "./usePluginCategoryPage";

const { iconUrl, categorySettings, openDetailPage, updateMainField, updateDependentField } =
  usePluginCategoryPage();

const {
  plugin,
  loading,
  saving,
  categoryLabel,
  showDependents,
  pluginPresets,
  isThemeProvider,
  settingsForm,
  dependentSettingsForms,
  dependentPlugins,
  getFieldStringValue,
  getFieldSelectValue,
  getFieldOptions,
  applyPreset,
  resetToDefault,
} = categorySettings;
</script>

<template>
  <div class="plugin-category-page">
    <SLCard v-if="loading" variant="outline">
      <div class="plugin-category-page__loading">
        <div class="plugin-category-page__spinner"></div>
        <span>{{ i18n.t("common.loading") }}</span>
      </div>
    </SLCard>

    <template v-else-if="plugin">
      <SLCard class="plugin-category-page__hero" variant="outline">
        <div class="plugin-category-page__hero-copy">
          <div class="plugin-category-page__icon-shell">
            <img
              v-if="iconUrl"
              :src="iconUrl"
              :alt="plugin.manifest.name"
              class="plugin-category-page__icon"
            />
            <Palette v-else :size="24" class="plugin-category-page__icon-fallback" />
          </div>
          <div class="plugin-category-page__title-block">
            <h2>{{ categoryLabel }}</h2>
            <p>
              {{ getLocalizedPluginDescription(plugin.manifest, i18n.getLocale()) }}
            </p>
          </div>
        </div>
        <SLButton variant="secondary" size="sm" @click="openDetailPage">
          {{ i18n.t("plugins.plugin_settings") }}
        </SLButton>
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
        :title="`${plugin.manifest.name} ${i18n.t('plugins.settings')}`"
        :fields="plugin.manifest.settings"
        :form="settingsForm"
        :get-field-string-value="getFieldStringValue"
        :get-field-select-value="getFieldSelectValue"
        :get-field-options="getFieldOptions"
        @update-field="updateMainField"
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
        @update-field="updateDependentField"
      />

      <div class="plugin-category-page__footer-actions">
        <SLButton variant="secondary" size="sm" @click="resetToDefault">
          {{ i18n.t("plugins.reset_default") }}
        </SLButton>
        <span class="plugin-category-page__auto-save-hint">
          {{ saving ? i18n.t("plugins.saving") : i18n.t("plugins.auto_saved") }}
        </span>
      </div>
    </template>

    <SLCard v-else variant="outline">
      <div class="plugin-category-page__empty">
        <h2>{{ i18n.t("plugins.not_found") }}</h2>
      </div>
    </SLCard>
  </div>
</template>

<style scoped>
.plugin-category-page {
  min-width: 0;
  display: grid;
  gap: 16px;
}

.plugin-category-page__loading,
.plugin-category-page__empty {
  min-height: 180px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.plugin-category-page__spinner {
  width: 24px;
  height: 24px;
  border-radius: 999px;
  border: 3px solid color-mix(in srgb, var(--sl-primary) 18%, transparent);
  border-top-color: var(--sl-primary);
  animation: plugin-category-page-spin 0.9s linear infinite;
}

.plugin-category-page__hero,
.plugin-category-page__hero-copy,
.plugin-category-page__footer-actions {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
}

.plugin-category-page__hero-copy {
  justify-content: flex-start;
  min-width: 0;
  flex: 1;
}

.plugin-category-page__icon-shell {
  width: 56px;
  height: 56px;
  border-radius: 16px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid color-mix(in srgb, var(--sl-border) 78%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 88%, transparent);
}

.plugin-category-page__icon {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.plugin-category-page__icon-fallback {
  color: var(--sl-text-tertiary);
}

.plugin-category-page__title-block {
  min-width: 0;
}

.plugin-category-page__title-block h2,
.plugin-category-page__title-block p {
  margin: 0;
}

.plugin-category-page__title-block p,
.plugin-category-page__auto-save-hint {
  color: var(--sl-text-secondary);
}

.plugin-category-page__footer-actions {
  justify-content: flex-end;
}

@keyframes plugin-category-page-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 720px) {
  .plugin-category-page__hero,
  .plugin-category-page__hero-copy,
  .plugin-category-page__footer-actions {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
