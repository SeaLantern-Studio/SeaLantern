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
  <SLCard
    :title="i18n.t('settings.server_defaults')"
    :subtitle="i18n.t('settings.server_defaults_desc')"
  >
    <div class="sl-settings-group">
      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_memory") }} (MB)</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.max_memory_desc") }}</span>
        </div>
        <div class="sl-input-sm">
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

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.min_memory") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.min_memory_desc") }}</span>
        </div>
        <div class="sl-input-sm">
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

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_port") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.port_desc") }}</span>
        </div>
        <div class="sl-input-sm">
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

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.default_java") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.default_java_desc") }}</span>
        </div>
        <div class="sl-input-lg">
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

      <div class="sl-setting-row full-width">
        <JavaDownloader
          @installed="
            (path) => {
              emit('javaInstalled', path);
              emit('change');
            }
          "
        />
      </div>

      <div class="sl-setting-row full-width">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.jvm_args") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.jvm_args_desc") }}</span>
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
.sl-setting-row.full-width {
  flex-direction: column;
  align-items: stretch;
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
