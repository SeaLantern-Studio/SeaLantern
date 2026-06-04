<script setup lang="ts">
import { computed } from "vue";
import SLInput from "@components/common/SLInput.vue";
import { i18n } from "@language";
import { useStartupConfigSection } from "@components/config/useStartupConfigSection";

const props = defineProps<{
  serverPath: string;
  defaultMaxMemory: number;
  defaultMinMemory: number;
}>();

const emit = defineEmits<{
  (e: "saved", maxMemory: number, minMemory: number): void;
}>();

const startupConfig = useStartupConfigSection({
  serverPath: computed(() => props.serverPath),
  defaultMaxMemory: computed(() => props.defaultMaxMemory),
  defaultMinMemory: computed(() => props.defaultMinMemory),
  onSaved: (maxMemory, minMemory) => {
    emit("saved", maxMemory, minMemory);
  },
});
</script>

<template>
  <div class="config-startup-section">
    <div v-if="startupConfig.loading" class="loading-state">
      <p class="text-caption">{{ i18n.t("common.loading") }}</p>
    </div>
    <template v-else>
      <div v-if="startupConfig.error.value" class="error-banner">
        <span>{{ startupConfig.error.value }}</span>
        <button class="banner-close" @click="startupConfig.error.value = null">x</button>
      </div>
      <div class="startup-entries">
        <div class="config-entry glass-card">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key">{{ i18n.t("config.max_memory") }}</span>
            </div>
            <p class="entry-desc text-caption">{{ i18n.t("config.max_memory_desc") }}</p>
          </div>
          <div class="entry-control">
            <SLInput
              :modelValue="String(startupConfig.maxMemory.value)"
              @update:modelValue="startupConfig.maxMemory.value = Number($event)"
              type="number"
              :placeholder="'2048'"
              :min="128"
              :step="128"
            />
          </div>
        </div>
        <div class="config-entry glass-card">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key">{{ i18n.t("config.min_memory") }}</span>
            </div>
            <p class="entry-desc text-caption">{{ i18n.t("config.min_memory_desc") }}</p>
          </div>
          <div class="entry-control">
            <SLInput
              :modelValue="String(startupConfig.minMemory.value)"
              @update:modelValue="startupConfig.minMemory.value = Number($event)"
              type="number"
              :placeholder="'512'"
              :min="128"
              :step="128"
            />
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.config-startup-section {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: var(--sl-radius-md);
  font-size: var(--sl-font-size-base);
  background: var(--sl-error-bg);
  border: 1px solid color-mix(in srgb, var(--sl-error) 30%, transparent);
  color: var(--sl-error);
}

.banner-close {
  font-weight: 600;
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
}

.startup-entries {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.config-entry {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-config-entry-padding-block) var(--sl-space-md);
  gap: var(--sl-space-lg);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  transition: all var(--sl-transition-fast);
}

.config-entry:hover {
  border-color: var(--sl-border);
  box-shadow: var(--sl-shadow-sm);
}

.entry-header {
  flex: 1;
  min-width: 0;
}

.entry-key-row {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.entry-key {
  font-size: var(--sl-font-size-base);
  font-weight: 600;
  color: var(--sl-text-primary);
}

.entry-desc {
  margin-top: 2px;
}

.entry-control {
  flex-shrink: 0;
  min-width: var(--sl-config-control-width);
}
</style>
