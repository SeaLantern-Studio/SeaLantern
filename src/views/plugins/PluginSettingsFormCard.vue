<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLCheckbox from "@components/common/SLCheckbox.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLTextarea from "@components/common/SLTextarea.vue";
import type { PluginSettingField } from "@type/plugin";
import { Link } from "lucide-vue-next";

type PluginSettingValue = string | number | boolean | null;

defineProps<{
  title: string;
  description?: string;
  fields: PluginSettingField[];
  form: Record<string, PluginSettingValue>;
  pluginName?: string;
  dependent?: boolean;
  getFieldStringValue: (value: PluginSettingValue | undefined) => string;
  getFieldSelectValue: (value: PluginSettingValue | undefined) => string | number | undefined;
  getFieldOptions: (
    options: Array<{ value: string; label: string }> | undefined,
  ) => Array<{ value: string; label: string }>;
}>();

const emit = defineEmits<{
  (e: "update-field", key: string, value: string | number | boolean): void;
}>();

function getColorValue(value: PluginSettingValue | undefined): string {
  return typeof value === "string" && value ? value : "#000000";
}
</script>

<template>
  <SLCard class="settings-card" :class="{ 'dependent-settings': dependent }">
    <h3 class="section-title">
      <Link v-if="dependent" class="dependent-icon" :size="16" />
      {{ title }}
    </h3>
    <p v-if="description" class="dependent-desc">{{ description }}</p>
    <div class="settings-form">
      <div v-for="field in fields" :key="field.key" class="form-field">
        <label class="field-label">
          {{ field.label }}
          <span v-if="field.description" class="field-desc">{{ field.description }}</span>
        </label>
        <template v-if="field.type === 'string'">
          <SLInput
            :modelValue="getFieldStringValue(form[field.key])"
            @update:modelValue="emit('update-field', field.key, $event)"
          />
        </template>
        <template v-else-if="field.type === 'number'">
          <SLInput
            type="number"
            :modelValue="getFieldStringValue(form[field.key])"
            @update:modelValue="emit('update-field', field.key, Number($event))"
          />
        </template>
        <template v-else-if="field.type === 'textarea'">
          <SLTextarea
            :modelValue="getFieldStringValue(form[field.key])"
            :rows="field.rows"
            :maxlength="field.maxlength"
            @update:modelValue="emit('update-field', field.key, $event)"
          />
        </template>
        <template v-else-if="field.type === 'color'">
          <div class="color-row-inline">
            <span class="color-row-value">{{ getFieldStringValue(form[field.key]) }}</span>
            <input
              type="color"
              class="color-row-picker"
              :value="getColorValue(form[field.key])"
              @input="emit('update-field', field.key, ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>
        <template v-else-if="field.type === 'boolean'">
          <SLSwitch
            :modelValue="Boolean(form[field.key])"
            @update:modelValue="emit('update-field', field.key, $event)"
            size="sm"
          />
        </template>
        <template v-else-if="field.type === 'checkbox'">
          <SLCheckbox
            :modelValue="Boolean(form[field.key])"
            @update:modelValue="emit('update-field', field.key, $event)"
          />
        </template>
        <template v-else-if="field.type === 'select' && field.display === 'button-group'">
          <div class="btn-group">
            <button
              v-for="option in getFieldOptions(field.options)"
              :key="option.value"
              type="button"
              class="btn-group-item"
              :class="{ active: form[field.key] === option.value }"
              @click="emit('update-field', field.key, option.value)"
            >
              {{ option.label }}
            </button>
          </div>
        </template>
        <template v-else-if="field.type === 'select'">
          <SLSelect
            :modelValue="getFieldSelectValue(form[field.key])"
            :options="getFieldOptions(field.options)"
            @update:modelValue="emit('update-field', field.key, $event)"
          />
        </template>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.settings-card {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-size: var(--sl-font-size-lg);
  font-weight: 600;
  color: var(--sl-text-primary);
}

.dependent-icon {
  color: var(--sl-text-secondary);
}

.dependent-desc {
  margin: -8px 0 0;
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--sl-text-primary);
  font-size: var(--sl-font-size-sm);
  font-weight: 500;
}

.field-desc {
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
  font-weight: 400;
}

.color-row-inline {
  display: flex;
  align-items: center;
  gap: 12px;
}

.color-row-value {
  min-width: 88px;
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
}

.color-row-picker {
  width: 44px;
  height: 32px;
  padding: 0;
  border: none;
  background: transparent;
  cursor: pointer;
}

.btn-group {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.btn-group-item {
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease,
    color 0.2s ease;
}

.btn-group-item.active {
  border-color: var(--sl-primary);
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}
</style>
