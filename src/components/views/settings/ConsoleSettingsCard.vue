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
    <div class="sl-settings-group">
      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.console_font_size") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.console_font_size_desc") }}</span>
        </div>
        <div class="sl-input-sm">
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

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.max_log_lines") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.max_log_lines_desc") }}</span>
        </div>
        <div class="sl-input-sm">
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
