<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import { i18n } from "@language";
import BackgroundSection from "./BackgroundSection.vue";
import { useAppearanceSettingsSection } from "@src/pages/settings/useAppearanceSettingsSection";

const {
  isReady,
  fontsLoading,
  theme,
  fontSize,
  fontFamily,
  windowEffect,
  minimalMode,
  backgroundSize,
  backgroundOpacity,
  backgroundBlur,
  backgroundBrightness,
  backgroundExpanded,
  backgroundImagePath,
  backgroundPreviewUrl,
  backgroundImageName,
  isThemeProxied,
  themeProxyNotice,
  windowEffectHint,
  themeOptions,
  fontFamilyOptions,
  windowEffectOptions,
  backgroundSizeOptions,
  setTheme,
  setFontSize,
  commitFontSize,
  setFontFamily,
  setWindowEffect,
  setMinimalMode,
  setBackgroundSize,
  setBackgroundOpacity,
  commitBackgroundOpacity,
  setBackgroundBlur,
  commitBackgroundBlur,
  setBackgroundBrightness,
  commitBackgroundBrightness,
  pickBackgroundImage,
  removeBackgroundImage,
  setBackgroundExpanded,
} = useAppearanceSettingsSection();

function handleThemeChange(value: string | number): void {
  void setTheme(String(value));
}

function handleFontSizeInput(event: Event): void {
  setFontSize(Number((event.target as HTMLInputElement).value));
}

function handleFontFamilyChange(value: string | number): void {
  void setFontFamily(String(value));
}

function handleWindowEffectChange(value: string | number): void {
  void setWindowEffect(String(value) as typeof windowEffect.value);
}

function handleMinimalModeChange(value: boolean): void {
  void setMinimalMode(value);
}

function handleBackgroundSizeChange(value: string): void {
  void setBackgroundSize(value);
}
</script>

<template>
  <SLCard variant="outline" padding="lg" class="appearance-section">
    <p class="appearance-section__description">
      {{ i18n.t("settings.next.appearance.live_hint") }}
    </p>

    <div class="appearance-section__body" :aria-busy="!isReady">
      <div class="appearance-section__group">
        <div class="appearance-section__row">
          <div class="appearance-section__info">
            <span class="appearance-section__label">{{ i18n.t("settings.theme") }}</span>
            <span class="appearance-section__desc">{{ i18n.t("settings.theme_desc") }}</span>
          </div>

          <div class="appearance-section__control appearance-section__control--stacked">
            <SLSelect
              :model-value="theme"
              :options="themeOptions"
              :disabled="!isReady || isThemeProxied"
              @update:model-value="handleThemeChange"
            />

            <span v-if="themeProxyNotice" class="appearance-section__hint">
              {{ themeProxyNotice }}
            </span>
          </div>
        </div>

        <div class="appearance-section__row">
          <div class="appearance-section__info">
            <span class="appearance-section__label">{{ i18n.t("settings.font_size") }}</span>
            <span class="appearance-section__desc">{{ i18n.t("settings.font_size_desc") }}</span>
          </div>

          <div class="appearance-section__slider-control">
            <input
              class="appearance-section__slider"
              type="range"
              min="12"
              max="24"
              step="1"
              :value="fontSize"
              :disabled="!isReady"
              @input="handleFontSizeInput"
              @change="commitFontSize"
            />
            <span class="appearance-section__value">{{ fontSize }}px</span>
          </div>
        </div>

        <div class="appearance-section__row">
          <div class="appearance-section__info">
            <span class="appearance-section__label">{{ i18n.t("settings.font_family") }}</span>
            <span class="appearance-section__desc">{{ i18n.t("settings.font_family_desc") }}</span>
          </div>

          <div class="appearance-section__control">
            <SLSelect
              :model-value="fontFamily"
              :options="fontFamilyOptions"
              :searchable="true"
              :loading="fontsLoading"
              :preview-font="true"
              :placeholder="i18n.t('settings.search_font')"
              :disabled="!isReady"
              @update:model-value="handleFontFamilyChange"
            />
          </div>
        </div>

        <div class="appearance-section__row">
          <div class="appearance-section__info">
            <span class="appearance-section__label">{{ i18n.t("settings.window_effect") }}</span>
            <span class="appearance-section__desc">{{
              i18n.t("settings.window_effect_desc")
            }}</span>
          </div>

          <div class="appearance-section__control appearance-section__control--stacked">
            <SLSelect
              :model-value="windowEffect"
              :options="windowEffectOptions"
              :disabled="!isReady"
              @update:model-value="handleWindowEffectChange"
            />

            <span v-if="windowEffectHint" class="appearance-section__hint">
              {{ windowEffectHint }}
            </span>
          </div>
        </div>

        <div class="appearance-section__row">
          <div class="appearance-section__info">
            <span class="appearance-section__label">{{ i18n.t("settings.minimal_mode") }}</span>
            <span class="appearance-section__desc">{{ i18n.t("settings.minimal_mode_desc") }}</span>
          </div>

          <SLSwitch
            :model-value="minimalMode"
            :disabled="!isReady"
            @update:model-value="handleMinimalModeChange"
          />
        </div>
      </div>

      <BackgroundSection
        :expanded="backgroundExpanded"
        :image-path="backgroundImagePath"
        :image-name="backgroundImageName"
        :preview-url="backgroundPreviewUrl"
        :opacity="backgroundOpacity"
        :blur="backgroundBlur"
        :brightness="backgroundBrightness"
        :background-size="backgroundSize"
        :background-size-options="backgroundSizeOptions"
        :disabled="!isReady"
        @update:expanded="setBackgroundExpanded"
        @update:opacity="setBackgroundOpacity"
        @commit:opacity="commitBackgroundOpacity"
        @update:blur="setBackgroundBlur"
        @commit:blur="commitBackgroundBlur"
        @update:brightness="setBackgroundBrightness"
        @commit:brightness="commitBackgroundBrightness"
        @update:background-size="handleBackgroundSizeChange"
        @pick-image="pickBackgroundImage"
        @remove-image="removeBackgroundImage"
      />
    </div>
  </SLCard>
</template>

<style scoped>
.appearance-section {
  display: grid;
  gap: 12px;
  border-radius: 22px;
  border-color: color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 92%, var(--sl-bg));
}

.appearance-section__description {
  margin: 0;
  font-size: 0.84rem;
  color: var(--sl-text-secondary);
  line-height: 1.45;
}

.appearance-section__body {
  display: grid;
  gap: 16px;
}

.appearance-section__body[aria-busy="true"] {
  opacity: 0.7;
}

.appearance-section__group {
  display: grid;
  gap: 0;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
}

.appearance-section__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 14px 18px;
  border-bottom: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
}

.appearance-section__row:last-child {
  border-bottom: none;
}

.appearance-section__info {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.appearance-section__label {
  color: var(--sl-text-primary);
  font-size: 0.9rem;
  font-weight: 600;
}

.appearance-section__desc {
  color: var(--sl-text-secondary);
  font-size: 0.82rem;
  line-height: 1.45;
}

.appearance-section__control {
  width: min(300px, 100%);
  flex: none;
}

.appearance-section__control--stacked {
  display: grid;
  gap: 6px;
}

.appearance-section__hint {
  color: var(--sl-text-secondary);
  font-size: 0.8rem;
  line-height: 1.4;
}

.appearance-section__slider-control {
  width: min(300px, 100%);
  display: flex;
  align-items: center;
  gap: 10px;
  flex: none;
}

.appearance-section__slider {
  width: 100%;
  height: 4px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-border) 90%, transparent);
  outline: none;
  -webkit-appearance: none;
}

.appearance-section__slider::-webkit-slider-thumb {
  width: 14px;
  height: 14px;
  border-radius: 999px;
  border: none;
  background: var(--sl-primary);
  cursor: pointer;
  -webkit-appearance: none;
}

.appearance-section__slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 999px;
  border: none;
  background: var(--sl-primary);
  cursor: pointer;
}

.appearance-section__value {
  min-width: 44px;
  text-align: right;
  color: var(--sl-text-secondary);
  font-size: 0.82rem;
  font-weight: 600;
}

@media (max-width: 900px) {
  .appearance-section__row {
    flex-direction: column;
    align-items: stretch;
  }

  .appearance-section__control,
  .appearance-section__slider-control {
    width: 100%;
  }
}
</style>
