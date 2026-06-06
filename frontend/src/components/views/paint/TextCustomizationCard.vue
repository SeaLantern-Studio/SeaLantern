<script setup lang="ts">
import { computed } from "vue";
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import { i18n } from "@language";
import type { TextColorOverrides } from "@api/settings";
import { getDefaultTextColors } from "@utils/theme";

const props = defineProps<{
  appDisplayName: string;
  theme: string;
  color: string;
  textColorOverrides: TextColorOverrides;
  windowEffect: string;
}>();

const emit = defineEmits<{
  (e: "update:appDisplayName", value: string): void;
  (e: "update:textColorOverrides", value: TextColorOverrides): void;
  (e: "change"): void;
}>();

function updateAppDisplayName(value: string) {
  emit("update:appDisplayName", value);
  emit("change");
}

function handleColorChange(key: keyof TextColorOverrides, value: string) {
  emit("update:textColorOverrides", {
    ...props.textColorOverrides,
    [key]: value,
  });
  emit("change");
}

function resetColor(key: keyof TextColorOverrides) {
  handleColorChange(key, "");
}

const defaultTextColors = computed<TextColorOverrides>(() => {
  return getDefaultTextColors(props.color, props.theme, props.windowEffect);
});
</script>

<template>
  <SLCard
    :title="i18n.t('settings.text_customization')"
    :subtitle="i18n.t('settings.text_customization_desc')"
  >
    <div class="sl-settings-group">
      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.app_display_name") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.app_display_name_desc") }}</span>
        </div>
        <div class="sl-input-lg">
          <SLInput
            :model-value="appDisplayName"
            :placeholder="i18n.t('settings.app_display_name_placeholder')"
            @update:model-value="updateAppDisplayName"
          />
        </div>
      </div>

      <div class="text-color-grid">
        <div class="text-color-item">
          <div class="text-color-copy">
            <span class="sl-setting-label">{{ i18n.t("settings.text_color_title") }}</span>
            <span class="sl-setting-desc">{{ i18n.t("settings.text_color_title_desc") }}</span>
          </div>
          <div class="text-color-controls">
            <input
              class="text-color-picker"
              type="color"
              :value="textColorOverrides.title || defaultTextColors.title"
              @input="handleColorChange('title', ($event.target as HTMLInputElement).value)"
            />
            <SLInput
              :model-value="textColorOverrides.title"
              :placeholder="defaultTextColors.title"
              @update:model-value="handleColorChange('title', $event)"
            />
            <button class="text-color-reset" type="button" @click="resetColor('title')">
              {{ i18n.t("settings.text_color_reset") }}
            </button>
          </div>
        </div>

        <div class="text-color-item">
          <div class="text-color-copy">
            <span class="sl-setting-label">{{ i18n.t("settings.text_color_text") }}</span>
            <span class="sl-setting-desc">{{ i18n.t("settings.text_color_text_desc") }}</span>
          </div>
          <div class="text-color-controls">
            <input
              class="text-color-picker"
              type="color"
              :value="textColorOverrides.text || defaultTextColors.text"
              @input="handleColorChange('text', ($event.target as HTMLInputElement).value)"
            />
            <SLInput
              :model-value="textColorOverrides.text"
              :placeholder="defaultTextColors.text"
              @update:model-value="handleColorChange('text', $event)"
            />
            <button class="text-color-reset" type="button" @click="resetColor('text')">
              {{ i18n.t("settings.text_color_reset") }}
            </button>
          </div>
        </div>

        <div class="text-color-item">
          <div class="text-color-copy">
            <span class="sl-setting-label">{{ i18n.t("settings.text_color_description") }}</span>
            <span class="sl-setting-desc">{{
              i18n.t("settings.text_color_description_desc")
            }}</span>
          </div>
          <div class="text-color-controls">
            <input
              class="text-color-picker"
              type="color"
              :value="textColorOverrides.description || defaultTextColors.description"
              @input="handleColorChange('description', ($event.target as HTMLInputElement).value)"
            />
            <SLInput
              :model-value="textColorOverrides.description"
              :placeholder="defaultTextColors.description"
              @update:model-value="handleColorChange('description', $event)"
            />
            <button class="text-color-reset" type="button" @click="resetColor('description')">
              {{ i18n.t("settings.text_color_reset") }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.sl-settings-group {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
}

.sl-setting-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(280px, 380px);
  gap: var(--sl-space-lg);
  align-items: center;
}

.sl-setting-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sl-setting-label {
  color: var(--sl-text-primary);
  font-size: var(--sl-font-size-base);
  font-weight: 600;
}

.sl-setting-desc {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
  line-height: 1.5;
}

.sl-input-lg {
  min-width: 0;
}

.text-color-grid {
  display: grid;
  gap: var(--sl-space-md);
}

.text-color-item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(320px, 420px);
  gap: var(--sl-space-lg);
  align-items: center;
  padding: var(--sl-space-md);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: color-mix(in srgb, var(--sl-surface) 86%, transparent);
}

.text-color-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.text-color-controls {
  display: grid;
  grid-template-columns: 48px minmax(0, 1fr) auto;
  gap: var(--sl-space-sm);
  align-items: center;
}

.text-color-picker {
  width: 48px;
  height: 40px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  background: transparent;
  padding: 4px;
  cursor: pointer;
}

.text-color-picker::-webkit-color-swatch-wrapper {
  padding: 0;
}

.text-color-picker::-webkit-color-swatch {
  border: none;
  border-radius: calc(var(--sl-radius-sm) - 2px);
}

.text-color-reset {
  padding: 0 12px;
  height: 40px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  color: var(--sl-text-secondary);
  background: var(--sl-surface);
  transition:
    border-color var(--sl-transition-fast),
    color var(--sl-transition-fast),
    background-color var(--sl-transition-fast);
}

.text-color-reset:hover {
  color: var(--sl-primary);
  border-color: var(--sl-primary-light);
  background: var(--sl-primary-bg);
}

@media (max-width: 900px) {
  .sl-setting-row,
  .text-color-item {
    grid-template-columns: 1fr;
  }

  .text-color-controls {
    grid-template-columns: 48px minmax(0, 1fr);
  }

  .text-color-reset {
    grid-column: 1 / -1;
  }
}
</style>
