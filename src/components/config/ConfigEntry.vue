<script setup lang="ts">
import { i18n } from "@language";
import type { ConfigEntry } from "@api/config";

interface Props {
  entry: ConfigEntry;
  value: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "updateValue", key: string, value: string): void;
}>();

function handleValueChange(value: string) {
  emit("updateValue", props.entry.key, value);
}

function handleSwitchChange(checked: boolean) {
  emit("updateValue", props.entry.key, checked ? "true" : "false");
}

function isBooleanType(entry: ConfigEntry): boolean {
  return entry.type === "boolean" || ["true", "false"].includes(entry.default);
}
</script>

<template>
  <div class="config-entry">
    <div class="entry-info">
      <div class="entry-header">
        <div class="entry-key">{{ entry.key }}</div>
        <cmz-badge variant="outline" size="sm" class="entry-category">{{
          entry.category
        }}</cmz-badge>
      </div>
      <div class="entry-description">{{ entry.description }}</div>
      <div class="entry-default">{{ i18n.t("config.default") }}: {{ entry.default }}</div>
    </div>
    <div class="entry-value">
      <template v-if="isBooleanType(entry)">
        <cmz-switch :modelValue="value === 'true'" @update:modelValue="handleSwitchChange" />
      </template>
      <template v-else>
        <cmz-input
          :modelValue="value"
          @update:modelValue="handleValueChange"
          :placeholder="entry.default"
          style="width: 280px"
        />
      </template>
    </div>
  </div>
</template>

<style scoped>
.config-entry {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-lg);
  padding: var(--sl-space-md) var(--sl-space-lg);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  margin-bottom: var(--sl-space-sm);
  transition: all var(--sl-transition-fast);
}

.config-entry:hover {
  border-color: var(--sl-border);
}

.entry-info {
  flex: 1;
  min-width: 0;
}

.entry-header {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-xs);
}

.entry-key {
  font-weight: 600;
  color: var(--sl-text-primary);
  font-size: 0.9375rem;
}

.entry-category {
  font-size: 0.75rem;
  flex-shrink: 0;
}

.entry-description {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  line-height: 1.4;
  margin-bottom: 4px;
}

.entry-value {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.entry-default {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  font-style: italic;
}
</style>
