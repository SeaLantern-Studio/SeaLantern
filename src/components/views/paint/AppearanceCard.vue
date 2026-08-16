<script setup lang="ts">
import BackgroundSettings from "./BackgroundSettings.vue";
import { i18n } from "@language";
import { computed, ref } from "vue";
import { getAllThemes } from "@themes";

const props = defineProps<{
  color: string;
  theme: string;
  fontSize: string;
  fontFamily: string;
  fontFamilyOptions: { label: string; value: string }[];
  fontsLoading: boolean;
  acrylicEnabled: boolean;
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
  (e: "update:color", value: string): void;
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
  (e: "themeChange", origin?: { x: number; y: number }): void;
  (e: "colorChange", origin?: { x: number; y: number }): void;
  (e: "fontSizeChange"): void;
  (e: "fontFamilyChange"): void;
  (e: "acrylicChange", value: boolean): void;
  (e: "minimalModeChange", value: boolean): void;
  (e: "pickImage"): void;
  (e: "clearImage"): void;
  (e: "change"): void;
}>();

// 颜色主题预设,按注册顺序展示为小卡片;rainbow 固定排最后
const allThemes = computed(() => {
  const list = Object.values(getAllThemes());
  return [...list.filter((t) => t.id !== "rainbow"), ...list.filter((t) => t.id === "rainbow")];
});

// 色块默认用亮色方案的 primary→secondary 对角渐变,明暗模式下都足够醒目;
// rainbow 主题特殊处理,显示完整彩虹圆环
function themeSwatchStyle(theme: (typeof allThemes.value)[number]) {
  if (theme.id === "rainbow") {
    return {
      background: "conic-gradient(#ef4444, #f59e0b, #84cc16, #06b6d4, #3b82f6, #a855f7, #ef4444)",
    };
  }
  return {
    background: `linear-gradient(135deg, ${theme.light.primary}, ${theme.light.secondary})`,
  };
}

function handleColorCardClick(value: string, e: MouseEvent) {
  emit("update:color", value);
  emit("change");
  // 颜色没变不播扩散动画
  if (value === props.color) return;
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  emit("colorChange", {
    x: rect.left + rect.width / 2,
    y: rect.top + rect.height / 2,
  });
}

const themeOptions = computed(() => [
  { label: i18n.t("settings.theme_options.auto"), value: "auto" },
  { label: i18n.t("settings.theme_options.light"), value: "light" },
  { label: i18n.t("settings.theme_options.dark"), value: "dark" },
]);

// 主题下拉容器 ref,切换时以触发器位置为圆形扩散起点
const themeSelectWrap = ref<HTMLElement | null>(null);

function handleThemeChange(value: string) {
  emit("update:theme", value);
  const rect = themeSelectWrap.value?.getBoundingClientRect();
  emit("themeChange", {
    x: rect ? rect.left + rect.width / 2 : window.innerWidth / 2,
    y: rect ? rect.top + rect.height / 2 : window.innerHeight / 2,
  });
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
  <cmz-card :title="i18n.t('settings.appearance')" :subtitle="i18n.t('settings.appearance_desc')">
    <div class="sl-settings-group">
      <div class="settings-group-title">{{ i18n.t("settings.group_theme") }}</div>

      <div class="settings-entry theme-color-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.color_theme") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.color_theme_desc") }}</span>
        </div>
        <div v-if="isThemeProxied" class="theme-proxied-notice">
          <span class="proxied-text">{{
            i18n.t("settings.theme_proxied_by", { plugin: themeProxyPluginName })
          }}</span>
        </div>
        <div v-else class="theme-card-grid">
          <button
            v-for="t in allThemes"
            :key="t.id"
            type="button"
            class="theme-card"
            :class="{ active: color === t.id }"
            :title="t.description"
            @click="handleColorCardClick(t.id, $event)"
          >
            <span class="theme-card-swatch" :style="themeSwatchStyle(t)"></span>
            <span class="theme-card-name">{{ t.name }}</span>
          </button>
        </div>
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.theme") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.theme_desc") }}</span>
        </div>
        <div ref="themeSelectWrap" class="sl-input-md">
          <div v-if="isThemeProxied" class="theme-proxied-notice">
            <span class="proxied-text">{{
              i18n.t("settings.theme_proxied_by", { plugin: themeProxyPluginName })
            }}</span>
          </div>
          <cmz-select
            v-else
            :model-value="theme"
            :options="themeOptions"
            @update:model-value="handleThemeChange"
          />
        </div>
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.font_size") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.font_size_desc") }}</span>
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

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.font_family") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.font_family_desc") }}</span>
        </div>
        <div class="sl-input-lg">
          <cmz-select
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

      <div class="settings-group-title">{{ i18n.t("settings.group_effect") }}</div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.advanced_material") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.advanced_material_desc") }}</span>
        </div>
        <cmz-switch :model-value="acrylicEnabled" @update:model-value="handleAcrylicChange" />
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.minimal_mode") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.minimal_mode_desc") }}</span>
        </div>
        <cmz-switch :model-value="minimalMode" @update:model-value="handleMinimalModeChange" />
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
  </cmz-card>
</template>

<style scoped>
.settings-group-title {
  margin: var(--sl-space-md) 0 var(--sl-space-xs);
  font-size: 0.8125rem;
  font-weight: 600;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.theme-proxied-notice {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  background: rgba(96, 165, 250, 0.1);
  border: 1px solid rgba(96, 165, 250, 0.3);
  border-radius: var(--sl-radius-md);
  color: var(--sl-primary);
  font-size: var(--sl-font-size-base);
  min-width: 200px;
}

.proxied-text {
  white-space: nowrap;
}

/* 颜色主题卡片行:竖排,标题在上卡片独占一行,不再和标题挤右侧 */
.theme-color-entry {
  flex-direction: column;
  align-items: stretch;
}

/* 主题色卡片:grid 自适应,一排多个,色块在左名字在右,选中用主色描边 */
.theme-card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: var(--sl-space-sm);
}

.theme-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px 8px 8px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-surface);
  color: var(--sl-text-secondary);
  cursor: pointer;
  font-size: var(--sl-font-size-sm);
  font-weight: 500;
  transition:
    border-color var(--sl-transition-fast),
    box-shadow var(--sl-transition-fast),
    background-color var(--sl-transition-fast),
    color var(--sl-transition-fast);
}

.theme-card:hover {
  border-color: var(--sl-primary);
  color: var(--sl-text-primary);
}

.theme-card.active {
  border-color: var(--sl-primary);
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  box-shadow: 0 0 0 2px var(--sl-primary-bg);
}

.theme-card-swatch {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1px solid rgba(15, 23, 42, 0.12);
}
</style>
