<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLSelect from "@components/common/SLSelect.vue";
import BackgroundSettings from "./BackgroundSettings.vue";
import { i18n } from "@language";
import { computed } from "vue";

defineProps<{
  theme: string;
  fontSize: string;
  fontFamily: string;
  fontFamilyOptions: { label: string; value: string }[];
  fontsLoading: boolean;
  acrylicEnabled: boolean;
  acrylicSupported: boolean;
  isThemeProxied: boolean;
  themeProxyPluginName: string;
  backgroundImage: string;
  bgOpacity: string;
  bgBlur: string;
  bgBrightness: string;
  backgroundSize: string;
  bgSettingsExpanded: boolean;
}>();

const emit = defineEmits<{
  (e: "update:theme", value: string): void;
  (e: "update:fontSize", value: string): void;
  (e: "update:fontFamily", value: string): void;
  (e: "update:acrylicEnabled", value: boolean): void;
  (e: "update:bgSettingsExpanded", value: boolean): void;
  (e: "update:bgOpacity", value: string): void;
  (e: "update:bgBlur", value: string): void;
  (e: "update:bgBrightness", value: string): void;
  (e: "update:backgroundSize", value: string): void;
  (e: "themeChange"): void;
  (e: "fontSizeChange"): void;
  (e: "fontFamilyChange"): void;
  (e: "acrylicChange", value: boolean): void;
  (e: "pickImage"): void;
  (e: "clearImage"): void;
  (e: "change"): void;
}>();

const themeOptions = computed(() => [
  { label: i18n.t("settings.theme_options.auto"), value: "auto" },
  { label: i18n.t("settings.theme_options.light"), value: "light" },
  { label: i18n.t("settings.theme_options.dark"), value: "dark" },
]);

function handleThemeChange(value: string) {
  emit("update:theme", value);
  emit("themeChange");
}

function handleFontSizeChange(e: Event) {
  emit("update:fontSize", (e.target as HTMLInputElement).value);
  emit("fontSizeChange");
}

function handleFontFamilyChange(value: string) {
  emit("update:fontFamily", value);
  emit("fontFamilyChange");
}

function handleAcrylicChange(value: boolean) {
  emit("update:acrylicEnabled", value);
  emit("acrylicChange", value);
}
</script>

<template>
  <SLCard :title="i18n.t('settings.appearance')" :subtitle="i18n.t('settings.appearance_desc')">
    <div class="settings-group">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.theme") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.theme_desc") }}</span>
        </div>
        <div class="input-lg">
          <div v-if="isThemeProxied" class="theme-proxied-notice">
            <span class="proxied-text">{{
              i18n.t("settings.theme_proxied_by", { plugin: themeProxyPluginName })
            }}</span>
          </div>
          <SLSelect
            v-else
            :model-value="theme"
            :options="themeOptions"
            @update:model-value="handleThemeChange"
          />
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.font_size") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.font_size_desc") }}</span>
        </div>
        <div class="slider-control">
          <input
            type="range"
            min="12"
            max="24"
            step="1"
            :value="fontSize"
            @input="handleFontSizeChange"
            class="sl-slider"
          />
          <span class="slider-value">{{ fontSize }}px</span>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.font_family") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.font_family_desc") }}</span>
        </div>
        <div class="input-lg">
          <SLSelect
            :model-value="fontFamily"
            :options="fontFamilyOptions"
            :searchable="true"
            :loading="fontsLoading"
            :previewFont="true"
            :placeholder="i18n.t('settings.search_font')"
            @update:model-value="handleFontFamilyChange"
          />
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.acrylic") }}</span>
          <span class="setting-desc">
            {{
              acrylicSupported
                ? i18n.t("settings.acrylic_desc")
                : i18n.t("settings.acrylic_not_supported")
            }}
          </span>
        </div>
        <SLSwitch
          :model-value="acrylicEnabled"
          :disabled="!acrylicSupported"
          @update:model-value="handleAcrylicChange"
        />
      </div>

      <BackgroundSettings
        :background-image="backgroundImage"
        :bg-opacity="bgOpacity"
        :bg-blur="bgBlur"
        :bg-brightness="bgBrightness"
        :background-size="backgroundSize"
        :expanded="bgSettingsExpanded"
        @update:expanded="emit('update:bgSettingsExpanded', $event)"
        @update:bg-opacity="emit('update:bgOpacity', $event)"
        @update:bg-blur="emit('update:bgBlur', $event)"
        @update:bg-brightness="emit('update:bgBrightness', $event)"
        @update:background-size="emit('update:backgroundSize', $event)"
        @pick-image="emit('pickImage')"
        @clear-image="emit('clearImage')"
        @change="emit('change')"
      />
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

.input-lg {
  width: 320px;
  flex-shrink: 0;
}

.theme-proxied-notice {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  background: rgba(96, 165, 250, 0.1);
  border: 1px solid rgba(96, 165, 250, 0.3);
  border-radius: var(--sl-radius-md);
  color: var(--sl-primary);
  font-size: 0.875rem;
  min-width: 200px;
}

.proxied-text {
  white-space: nowrap;
}

.slider-control {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  min-width: 200px;
}

.sl-slider {
  flex: 1;
  height: 6px;
  border-radius: var(--sl-radius-full);
  background: var(--sl-border);
  outline: none;
  -webkit-appearance: none;
}

.sl-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.sl-slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.sl-slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  cursor: pointer;
  border: none;
  transition: all var(--sl-transition-fast);
}

.sl-slider::-moz-range-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.slider-value {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  min-width: 50px;
  text-align: right;
}
</style>
