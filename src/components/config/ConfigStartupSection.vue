<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import SLButton from "@components/common/SLButton.vue";
import { configApi, type SLStartupConfig } from "@api/config";
import { i18n } from "@language";

const props = defineProps<{
  serverPath: string;
  defaultMaxMemory: number;
  defaultMinMemory: number;
}>();

const emit = defineEmits<{
  (e: "saved"): void;
}>();

const maxMemory = ref(props.defaultMaxMemory);
const minMemory = ref(props.defaultMinMemory);
const loading = ref(false);
const saving = ref(false);
const saved = ref(false);
const error = ref<string | null>(null);

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
  if (!props.serverPath) return;
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
    saved.value = true;
    setTimeout(() => {
      saved.value = false;
    }, 3000);
    emit("saved");
  } catch (e: any) {
    error.value = e?.toString() || "保存启动配置失败";
  } finally {
    saving.value = false;
  }
}

onMounted(loadConfig);
watch(() => props.serverPath, loadConfig);
</script>

<template>
  <div class="config-startup-section">
    <div v-if="loading" class="startup-loading">
      <p class="text-body">{{ i18n.t("common.loading") }}</p>
    </div>
    <template v-else>
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button class="banner-close" @click="error = null">x</button>
      </div>
      <div v-if="saved" class="success-banner">
        <span>{{ i18n.t("config.startup_config_saved") }}</span>
      </div>
      <div class="startup-settings-group">
        <h3 class="startup-group-title">{{ i18n.t("config.memory_settings") }}</h3>
        <p class="startup-group-desc">{{ i18n.t("config.memory_settings_desc") }}</p>
        <div class="startup-field">
          <label class="startup-field-label">{{ i18n.t("config.max_memory") }}</label>
          <input
            v-model.number="maxMemory"
            type="number"
            class="startup-field-input"
            :placeholder="'2048'"
            min="128"
            step="128"
          />
        </div>
        <div class="startup-field">
          <label class="startup-field-label">{{ i18n.t("config.min_memory") }}</label>
          <input
            v-model.number="minMemory"
            type="number"
            class="startup-field-input"
            :placeholder="'512'"
            min="128"
            step="128"
          />
        </div>
      </div>
      <div class="startup-actions">
        <SLButton variant="primary" :loading="saving" @click="saveConfig">
          {{ i18n.t("config.save_startup_config") }}
        </SLButton>
      </div>
    </template>
  </div>
</template>

<style scoped>
.config-startup-section {
  padding: 20px;
}

.startup-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 120px;
}

.error-banner,
.success-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: 8px;
  margin-bottom: 16px;
  font-size: 14px;
}

.error-banner {
  background: rgba(220, 38, 38, 0.1);
  color: #dc2626;
  border: 1px solid rgba(220, 38, 38, 0.2);
}

.success-banner {
  background: rgba(22, 163, 74, 0.1);
  color: #16a34a;
  border: 1px solid rgba(22, 163, 74, 0.2);
}

.banner-close {
  background: none;
  border: none;
  color: inherit;
  cursor: pointer;
  font-size: 16px;
  padding: 0 4px;
}

.startup-settings-group {
  margin-bottom: 24px;
}

.startup-group-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 4px 0;
}

.startup-group-desc {
  font-size: 13px;
  color: var(--text-secondary, #888);
  margin: 0 0 20px 0;
}

.startup-field {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.startup-field-label {
  font-size: 14px;
  min-width: 160px;
  color: var(--text-primary, #e0e0e0);
}

.startup-field-input {
  width: 200px;
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
  background: var(--tertiary-background, rgba(255, 255, 255, 0.05));
  color: var(--text-primary, #e0e0e0);
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.startup-field-input:focus {
  border-color: var(--primary-color, #3b82f6);
}

.startup-actions {
  display: flex;
  gap: 12px;
}
</style>
