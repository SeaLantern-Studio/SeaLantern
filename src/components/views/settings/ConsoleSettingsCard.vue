<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import { i18n } from "@language";

defineProps<{
  consoleFontSize: string;
  maxLogLines: string;
}>();

const emit = defineEmits<{
  (e: "update:consoleFontSize", value: string): void;
  (e: "update:maxLogLines", value: string): void;
  (e: "change"): void;
}>();
</script>

<template>
  <SLCard :title="i18n.t('settings.console')" :subtitle="i18n.t('settings.console_desc')">
    <div class="settings-group">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.console_font_size") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.console_font_size_desc") }}</span>
        </div>
        <div class="input-sm">
          <SLInput
            :model-value="consoleFontSize"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:consoleFontSize', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.max_log_lines") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.max_log_lines_desc") }}</span>
        </div>
        <div class="input-sm">
          <SLInput
            :model-value="maxLogLines"
            type="number"
            @update:model-value="
              (v) => {
                emit('update:maxLogLines', v);
                emit('change');
              }
            "
          />
        </div>
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
</style>
