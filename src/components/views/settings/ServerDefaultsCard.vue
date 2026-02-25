<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import SLTextarea from "@components/common/SLTextarea.vue";
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
        <SLTextarea
          :model-value="defaultJvmArgs"
          :placeholder="i18n.t('settings.jvm_args_placeholder')"
          :rows="3"
          @update:model-value="
            (v) => {
              emit('update:defaultJvmArgs', v);
              emit('change');
            }
          "
        />
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.sl-setting-row.full-width {
  flex-direction: column;
  align-items: stretch;
}

.sl-setting-row.full-width :deep(.sl-textarea) {
  margin-top: var(--sl-space-sm);
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  line-height: 1.6;
}
</style>
