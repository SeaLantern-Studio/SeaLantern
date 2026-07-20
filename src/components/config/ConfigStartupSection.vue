<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { configApi, type SLStartupConfig } from "@api/config";
import { i18n } from "@language";

const props = defineProps<{
  serverPath: string;
  defaultMaxMemory: number;
  defaultMinMemory: number;
}>();

const emit = defineEmits<{
  (e: "saved", maxMemory: number, minMemory: number): void;
}>();

const maxMemory = ref(props.defaultMaxMemory);
const minMemory = ref(props.defaultMinMemory);
const loading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);

let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
const AUTO_SAVE_DELAY = 800;

function scheduleAutoSave() {
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  autoSaveTimer = setTimeout(() => {
    saveConfig();
  }, AUTO_SAVE_DELAY);
}

async function loadConfig() {
  if (!props.serverPath) return;
  loading.value = true;
  error.value = null;
  try {
    const config = await configApi.readSLConfig(props.serverPath);
    maxMemory.value = config.max_memory ?? props.defaultMaxMemory;
    minMemory.value = config.min_memory ?? props.defaultMinMemory;
  } catch (e: any) {
    error.value = e?.toString() || "加载启动配置失败";
  } finally {
    loading.value = false;
  }
}

async function saveConfig() {
  if (!props.serverPath || saving.value) return;
  if (maxMemory.value < 128) {
    error.value = "最大内存不能小于 128MB";
    return;
  }
  if (minMemory.value < 128) {
    error.value = "最小内存不能小于 128MB";
    return;
  }
  if (minMemory.value > maxMemory.value) {
    error.value = "最小内存不能大于最大内存";
    return;
  }

  saving.value = true;
  error.value = null;
  try {
    const config: SLStartupConfig = {
      max_memory: maxMemory.value,
      min_memory: minMemory.value,
    };
    await configApi.writeSLConfig(props.serverPath, config);
    emit("saved", maxMemory.value, minMemory.value);
  } catch (e: any) {
    error.value = e?.toString() || "保存启动配置失败";
  } finally {
    saving.value = false;
  }
}

onMounted(loadConfig);

onUnmounted(() => {
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
});

watch(() => props.serverPath, loadConfig);
watch([maxMemory, minMemory], () => {
  scheduleAutoSave();
});
</script>

<template>
  <div class="config-startup-section">
    <div v-if="loading" class="loading-state">
      <p class="text-caption">{{ i18n.t("common.loading") }}</p>
    </div>
    <template v-else>
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button class="banner-close" @click="error = null">x</button>
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
            <cmz-input
              :modelValue="String(maxMemory)"
              @update:modelValue="maxMemory = Number($event)"
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
            <cmz-input
              :modelValue="String(minMemory)"
              @update:modelValue="minMemory = Number($event)"
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
