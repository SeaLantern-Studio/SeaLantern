<script setup lang="ts">
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import type { PluginSettingField } from "@type/plugin";

defineProps<{
  fields: PluginSettingField[] | undefined;
  fieldValues: Record<string, string | number | boolean>;
}>();

const emit = defineEmits<{
  (e: "update-field", key: string, value: string | number | boolean): void;
}>();

function getTextValue(values: Record<string, string | number | boolean>, key: string): string {
  const value = values[key];
  return typeof value === "string" ? value : String(value ?? "");
}

function getNumberValue(values: Record<string, string | number | boolean>, key: string): string {
  const value = values[key];
  if (typeof value === "number") {
    return String(value);
  }
  return typeof value === "string" ? value : "0";
}

function getSelectValue(
  values: Record<string, string | number | boolean>,
  key: string,
): string | number | undefined {
  const value = values[key];
  return typeof value === "boolean" ? undefined : value;
}
</script>

<template>
  <div v-if="fields?.length" class="settings-fields">
    <div v-for="field in fields" :key="field.key" class="setting-field">
      <label class="setting-label">
        {{ field.label }}
        <span v-if="field.description" class="setting-desc">{{ field.description }}</span>
      </label>
      <SLInput
        v-if="field.type === 'string'"
        :modelValue="getTextValue(fieldValues, field.key)"
        @update:modelValue="emit('update-field', field.key, $event)"
      />
      <div v-else-if="field.type === 'color'" class="setting-color-field">
        <input
          type="color"
          :value="getTextValue(fieldValues, field.key)"
          @input="emit('update-field', field.key, ($event.target as HTMLInputElement).value)"
          class="setting-color-picker"
        />
        <SLInput
          :modelValue="getTextValue(fieldValues, field.key)"
          @update:modelValue="emit('update-field', field.key, $event)"
        />
      </div>
      <SLInput
        v-else-if="field.type === 'number'"
        :modelValue="getNumberValue(fieldValues, field.key)"
        @update:modelValue="emit('update-field', field.key, Number($event))"
        type="number"
      />
      <label v-else-if="field.type === 'boolean'" class="setting-toggle">
        <SLSwitch
          :modelValue="Boolean(fieldValues[field.key])"
          @update:modelValue="emit('update-field', field.key, $event)"
          size="sm"
        />
      </label>
      <SLSelect
        v-else-if="field.type === 'select'"
        :modelValue="getSelectValue(fieldValues, field.key)"
        @update:modelValue="emit('update-field', field.key, $event)"
        :options="field.options || []"
      />
    </div>
  </div>
</template>

<style scoped>
.settings-fields {
  display: flex;
  flex-direction: column;
}

.setting-field {
  margin-bottom: 16px;
}

.setting-field:last-child {
  margin-bottom: 0;
}

.setting-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--sl-text-primary);
  margin-bottom: 8px;
}

.setting-desc {
  display: block;
  font-size: 12px;
  font-weight: 400;
  color: var(--sl-text-tertiary);
  margin-top: 2px;
}

.setting-color-field {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-color-picker {
  width: 40px;
  height: 38px;
  padding: 2px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--sl-radius-md);
  background: transparent;
  cursor: pointer;
  flex-shrink: 0;
}
</style>
