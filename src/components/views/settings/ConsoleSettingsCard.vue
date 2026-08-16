<script setup lang="ts">
import { i18n } from "@language";

defineProps<{
  consoleFontSize: string;
  consoleFontFamily: string;
  consoleLetterSpacing: string;
  fontFamilyOptions: { label: string; value: string }[];
  fontsLoading: boolean;
  maxLogLines: string;
}>();

const emit = defineEmits<{
  (e: "update:consoleFontSize", value: string): void;
  (e: "update:consoleFontFamily", value: string): void;
  (e: "update:consoleLetterSpacing", value: string): void;
  (e: "update:maxLogLines", value: string): void;
  (e: "change"): void;
}>();

// 控制台字体大小走拖拽滑块,与外观卡的字体大小交互保持一致
function handleConsoleFontSizeChange(e: Event) {
  const v = (e.target as HTMLInputElement).value;
  emit("update:consoleFontSize", v);
  emit("change");
}
</script>

<template>
  <cmz-card :title="i18n.t('settings.console')" :subtitle="i18n.t('settings.console_desc')">
    <div class="sl-settings-group">
      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.console_font_size") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.console_font_size_desc") }}</span>
        </div>
        <div class="sl-input-sm">
          <cmz-input
            :model-value="consoleFontSize"
            type="number"
            @update:model-value="
              (v: string) => {
                emit('update:consoleFontSize', v);
                emit('change');
              }
            "
          />
          <span class="sl-slider-value">{{ consoleFontSize }}px</span>
        </div>
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.font_family") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.console_font_family_desc") }}</span>
        </div>
        <div class="sl-input-lg">
          <cmz-select
            :model-value="consoleFontFamily"
            :options="fontFamilyOptions"
            :searchable="true"
            :loading="fontsLoading"
            :previewFont="true"
            :placeholder="i18n.t('settings.search_font')"
            @update:model-value="
              (v: string) => {
                emit('update:consoleFontFamily', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.console_letter_spacing") }}</span>
          <span class="settings-entry-desc">{{
            i18n.t("settings.console_letter_spacing_desc")
          }}</span>
        </div>
        <div class="sl-input-sm">
          <cmz-input
            :model-value="consoleLetterSpacing"
            type="number"
            @update:model-value="
              (v: string) => {
                emit('update:consoleLetterSpacing', v);
                emit('change');
              }
            "
          />
        </div>
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.max_log_lines") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.max_log_lines_desc") }}</span>
        </div>
        <div class="sl-input-sm">
          <cmz-input
            :model-value="maxLogLines"
            type="number"
            @update:model-value="
              (v: string) => {
                emit('update:maxLogLines', v);
                emit('change');
              }
            "
          />
        </div>
      </div>
    </div>
  </cmz-card>
</template>
