<script setup lang="ts">
import { i18n } from "../../language";
import SLInput from "../common/SLInput.vue";
import SLSwitch from "../common/SLSwitch.vue";
import SLBadge from "../common/SLBadge.vue";
import type { ConfigEntry } from "../../api/config";

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
    <div class="entry-header">
      <div class="entry-key">{{ entry.key }}</div>
      <SLBadge variant="outline" size="sm" class="entry-category">{{ entry.category }}</SLBadge>
    </div>
    <div class="entry-description">{{ entry.description }}</div>
    <div class="entry-value">
      <template v-if="isBooleanType(entry)">
        <SLSwitch :modelValue="value === 'true'" @update:modelValue="handleSwitchChange" />
      </template>
      <template v-else>
        <SLInput
          :modelValue="value"
          @update:modelValue="handleValueChange"
          :placeholder="entry.default"
          style="width: 300px"
        />
      </template>
    </div>
    <div class="entry-default">{{ i18n.t("config.default") }}: {{ entry.default }}</div>
  </div>
</template>

<style scoped>
.config-entry {
  padding: var(--sl-space-md);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  margin-bottom: var(--sl-space-sm);
  transition: all var(--sl-transition-fast);
}

.config-entry:hover {
  border-color: var(--sl-border);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.entry-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--sl-space-xs);
}

.entry-key {
  font-weight: 600;
  color: var(--sl-text-primary);
}

.entry-category {
  font-size: 0.75rem;
}

.entry-description {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  margin-bottom: var(--sl-space-sm);
  line-height: 1.4;
}

.entry-value {
  margin-bottom: var(--sl-space-xs);
}

.entry-default {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  font-style: italic;
}
</style>
