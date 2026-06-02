<script setup lang="ts">
import { i18n } from "@language";
import type { PluginInfo } from "@type/plugin";
import PluginSettingsFormCard from "@views/plugins/PluginSettingsFormCard.vue";
import type { PluginSettingsRecord, PluginSettingValue } from "@views/plugins/pluginSettingsShared";

defineProps<{
  plugins: PluginInfo[];
  forms: Record<string, PluginSettingsRecord>;
  showHeader?: boolean;
  headerTitle?: string;
  headerDescription?: string;
  itemDescription?: string;
  getFieldStringValue: (value: PluginSettingValue | undefined) => string;
  getFieldSelectValue: (value: PluginSettingValue | undefined) => string | number | undefined;
  getFieldOptions: (
    options: Array<{ value: string; label: string }> | undefined,
  ) => Array<{ value: string; label: string }>;
}>();

const emit = defineEmits<{
  (e: "update-field", pluginId: string, key: string, value: string | number | boolean): void;
}>();
</script>

<template>
  <section v-if="plugins.length > 0" class="dependent-settings-list">
    <div v-if="showHeader" class="dependent-section-header">
      <h2>{{ headerTitle }}</h2>
      <p v-if="headerDescription">{{ headerDescription }}</p>
    </div>

    <PluginSettingsFormCard
      v-for="depPlugin in plugins"
      :key="depPlugin.manifest.id"
      dependent
      :title="`${depPlugin.manifest.name} ${i18n.t('plugins.settings')}`"
      :description="itemDescription"
      :fields="depPlugin.manifest.settings || []"
      :form="forms[depPlugin.manifest.id] || {}"
      :get-field-string-value="getFieldStringValue"
      :get-field-select-value="getFieldSelectValue"
      :get-field-options="getFieldOptions"
      @update-field="emit('update-field', depPlugin.manifest.id, $event[0], $event[1])"
    />
  </section>
</template>

<style scoped>
.dependent-settings-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.dependent-section-header {
  padding-bottom: 12px;
  border-bottom: 1px solid var(--sl-border);
}

.dependent-section-header h2 {
  margin: 0 0 4px;
  font-size: var(--sl-font-size-xl);
  font-weight: 600;
  color: var(--sl-text-primary);
}

.dependent-section-header p {
  margin: 0;
  font-size: var(--sl-font-size-base);
  color: var(--sl-text-secondary);
}
</style>
