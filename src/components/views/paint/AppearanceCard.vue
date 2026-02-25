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
  minimalMode: boolean;
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
  (e: "update:minimalMode", value: boolean): void;
  (e: "themeChange"): void;
  (e: "fontSizeChange"): void;
  (e: "fontFamilyChange"): void;
  (e: "acrylicChange", value: boolean): void;
  (e: "minimalModeChange", value: boolean): void;
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

function handleMinimalModeChange(value: boolean) {
  emit("update:minimalMode", value);
  emit("minimalModeChange", value);
}
</script>

<template>
  <SLCard :title="i18n.t('settings.appearance')" :subtitle="i18n.t('settings.appearance_desc')">
    <div class="sl-settings-group">
      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.theme") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.theme_desc") }}</span>
        </div>
        <div class="sl-input-lg">
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

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.font_size") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.font_size_desc") }}</span>
        </div>
        <div class="sl-slider-control">
          <input
            type="range"
            min="12"
            max="24"
            step="1"
            :value="fontSize"
            @input="handleFontSizeChange"
            class="sl-slider"
          />
          <span class="sl-slider-value">{{ fontSize }}px</span>
        </div>
      </div>

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.font_family") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.font_family_desc") }}</span>
        </div>
        <div class="sl-input-lg">
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

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.acrylic") }}</span>
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

      <div class="sl-setting-row">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.minimal_mode") }}</span>
          <span class="sl-setting-desc">{{ i18n.t("settings.minimal_mode_desc") }}</span>
        </div>
        <SLSwitch
          :model-value="minimalMode"
          @update:model-value="handleMinimalModeChange"
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
</style>
