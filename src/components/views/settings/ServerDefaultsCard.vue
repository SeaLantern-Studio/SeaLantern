<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import JavaDownloader from "@components/JavaDownloader.vue";
import { i18n } from "@language";

defineProps<{
  maxMemory: string;
  minMemory: string;
  port: string;
  defaultJavaPath: string;
  defaultJvmArgs: string;
}>();

const emit = defineEmits<{
  (e: "update:maxMemory", value: string): void;
  (e: "update:minMemory", value: string): void;
  (e: "update:port", value: string): void;
  (e: "update:defaultJavaPath", value: string): void;
  (e: "update:defaultJvmArgs", value: string): void;
  (e: "change"): void;
  (e: "javaInstalled", path: string): void;
}>();
</script>

<template>
  <SLCard :title="i18n.t('settings.server_defaults')" :subtitle="i18n.t('settings.server_defaults_desc')">
    <div class="settings-group">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.default_memory") }} (MB)</span>
          <span class="setting-desc">{{ i18n.t("settings.max_memory_desc") }}</span>
        </div>
        <div class="input-sm">
          <SLInput
            :model-value="maxMemory"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:maxMemory', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.min_memory") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.min_memory_desc") }}</span>
        </div>
        <div class="input-sm">
          <SLInput
            :model-value="minMemory"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:minMemory', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.default_port") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.port_desc") }}</span>
        </div>
        <div class="input-sm">
          <SLInput
            :model-value="port"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:port', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.default_java") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.default_java_desc") }}</span>
        </div>
        <div class="input-lg">
          <SLInput
            :model-value="defaultJavaPath"
            :placeholder="i18n.t('settings.default_java_desc')"
            @update:model-value="
              (v) => {
                emit('update:defaultJavaPath', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="setting-row full-width">
        <JavaDownloader
          @installed="
            (path) => {
              emit('javaInstalled', path);
              emit('change');
            }
          "
        />
      </div>

      <div class="setting-row full-width">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.jvm_args") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.jvm_args_desc") }}</span>
        </div>
        <textarea
          class="jvm-textarea"
          :value="defaultJvmArgs"
          :placeholder="i18n.t('settings.jvm_args_placeholder')"
          rows="3"
          @input="
            (e) => {
              emit('update:defaultJvmArgs', (e.target as HTMLTextAreaElement).value);
              emit('change');
            }
          "
        ></textarea>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.settings-group {
  display: flex;
  flex-direction: column;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md) 0;
  border-bottom: 1px solid var(--sl-border-light);
  gap: var(--sl-space-lg);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-row.full-width {
  flex-direction: column;
  align-items: stretch;
}

.setting-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.setting-label {
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.setting-desc {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
  line-height: 1.4;
}

.input-sm {
  width: 120px;
  flex-shrink: 0;
}

.input-lg {
  width: 320px;
  flex-shrink: 0;
}

.jvm-textarea {
  width: 100%;
  margin-top: var(--sl-space-sm);
  padding: var(--sl-space-sm) var(--sl-space-md);
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  color: var(--sl-text-primary);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  resize: vertical;
  line-height: 1.6;
}

.jvm-textarea:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px var(--sl-primary-bg);
  outline: none;
}
</style>
