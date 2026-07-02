<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ChevronDown, ImagePlus, Trash2 } from "@lucide/vue";
import SLButton from "@components/common/SLButton.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { i18n } from "@language";
import type { AppearanceSelectOption } from "@src/pages/settings/useAppearanceSettingsSection";

interface Props {
  expanded: boolean;
  imagePath: string;
  imageName: string;
  previewUrl: string;
  opacity: number;
  blur: number;
  brightness: number;
  backgroundSize: string;
  backgroundSizeOptions: AppearanceSelectOption<string>[];
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
});

const emit = defineEmits<{
  (e: "update:expanded", value: boolean): void;
  (e: "update:opacity", value: number): void;
  (e: "commit:opacity"): void;
  (e: "update:blur", value: number): void;
  (e: "commit:blur"): void;
  (e: "update:brightness", value: number): void;
  (e: "commit:brightness"): void;
  (e: "update:backgroundSize", value: string): void;
  (e: "pick-image"): void;
  (e: "remove-image"): void;
}>();

const previewLoaded = ref(false);
const previewLoading = ref(false);

const hasImage = computed(() => Boolean(props.imagePath));
const opacityLabel = computed(() => `${Math.round(props.opacity * 100)}%`);
const blurLabel = computed(() => `${Math.round(props.blur)}px`);
const brightnessLabel = computed(() => `${Math.round(props.brightness * 100)}%`);
const backgroundStatus = computed(() =>
  hasImage.value
    ? props.imageName || i18n.t("settings.next.appearance.background.status_set")
    : i18n.t("settings.next.appearance.background.status_empty"),
);

function getFileExtension(path: string): string {
  return path.split(".").pop()?.toLowerCase() || "";
}

const isAnimatedImage = computed(() => {
  const extension = getFileExtension(props.imagePath);
  return extension === "gif" || extension === "webp" || extension === "apng";
});

watch(
  () => [props.previewUrl, props.expanded] as const,
  ([previewUrl, expanded], previous) => {
    const previousPreviewUrl = previous?.[0];

    if (!expanded || !previewUrl) {
      previewLoaded.value = false;
      previewLoading.value = false;
      return;
    }

    if (previewUrl !== previousPreviewUrl || !previewLoaded.value) {
      previewLoaded.value = false;
      previewLoading.value = true;
    }
  },
  { immediate: true },
);

function toggleExpanded(): void {
  emit("update:expanded", !props.expanded);
}

function handleOpacityInput(event: Event): void {
  emit("update:opacity", Number((event.target as HTMLInputElement).value));
}

function handleBlurInput(event: Event): void {
  emit("update:blur", Number((event.target as HTMLInputElement).value));
}

function handleBrightnessInput(event: Event): void {
  emit("update:brightness", Number((event.target as HTMLInputElement).value));
}

function handleBackgroundSizeChange(value: string | number): void {
  emit("update:backgroundSize", String(value));
}

function handlePreviewLoaded(): void {
  previewLoaded.value = true;
  previewLoading.value = false;
}

function handlePreviewError(): void {
  previewLoaded.value = false;
  previewLoading.value = false;
}
</script>

<template>
  <section class="appearance-background" :aria-disabled="props.disabled">
    <button
      class="appearance-background__header"
      type="button"
      :disabled="props.disabled"
      @click="toggleExpanded"
    >
      <div class="appearance-background__heading">
        <span class="appearance-background__title">{{
          i18n.t("settings.next.appearance.background.title")
        }}</span>
        <span class="appearance-background__meta">{{ backgroundStatus }}</span>
      </div>

      <span class="appearance-background__toggle" :class="{ 'is-expanded': props.expanded }">
        <ChevronDown :size="18" />
      </span>
    </button>

    <Transition name="appearance-background-collapse">
      <div v-show="props.expanded" class="appearance-background__content">
        <div class="appearance-background__preview-block">
          <div v-if="hasImage" class="appearance-background__preview-frame">
            <div
              v-if="previewLoading && !previewLoaded"
              class="appearance-background__preview-loading"
            >
              <div class="appearance-background__spinner" aria-hidden="true"></div>
              <span>{{ i18n.t("settings.next.appearance.background.preview_loading") }}</span>
            </div>

            <img
              v-show="previewLoaded || !previewLoading"
              class="appearance-background__preview-image"
              :src="props.previewUrl"
              :alt="i18n.t('settings.next.appearance.background.preview_alt')"
              loading="lazy"
              @load="handlePreviewLoaded"
              @error="handlePreviewError"
            />

            <span v-if="isAnimatedImage" class="appearance-background__badge">{{
              i18n.t("settings.next.appearance.background.animated_badge")
            }}</span>

            <div class="appearance-background__preview-overlay">
              <span class="appearance-background__image-name">{{ props.imageName }}</span>

              <div class="appearance-background__preview-actions">
                <SLButton variant="secondary" size="sm" @click.stop="emit('pick-image')">
                  {{ i18n.t("settings.next.appearance.background.replace") }}
                </SLButton>
                <SLButton variant="danger" size="sm" @click.stop="emit('remove-image')">
                  {{ i18n.t("settings.next.appearance.background.remove") }}
                </SLButton>
              </div>
            </div>
          </div>

          <div v-else class="appearance-background__empty-state">
            <div class="appearance-background__empty-copy">
              <strong>{{ i18n.t("settings.next.appearance.background.empty_title") }}</strong>
              <span>{{ i18n.t("settings.next.appearance.background.empty_description") }}</span>
            </div>

            <SLButton variant="secondary" @click="emit('pick-image')">
              <ImagePlus :size="16" />
              {{ i18n.t("settings.next.appearance.background.pick") }}
            </SLButton>
          </div>
        </div>

        <div class="appearance-background__rows">
          <div class="appearance-background__row">
            <div class="appearance-background__label-wrap">
              <span class="appearance-background__label">{{
                i18n.t("settings.next.appearance.background.opacity")
              }}</span>
              <span class="appearance-background__value">{{ opacityLabel }}</span>
            </div>

            <input
              class="appearance-background__slider"
              type="range"
              min="0"
              max="1"
              step="0.05"
              :value="props.opacity"
              :disabled="props.disabled"
              @input="handleOpacityInput"
              @change="emit('commit:opacity')"
            />
          </div>

          <div class="appearance-background__row">
            <div class="appearance-background__label-wrap">
              <span class="appearance-background__label">{{
                i18n.t("settings.next.appearance.background.blur")
              }}</span>
              <span class="appearance-background__value">{{ blurLabel }}</span>
            </div>

            <input
              class="appearance-background__slider"
              type="range"
              min="0"
              max="20"
              step="1"
              :value="props.blur"
              :disabled="props.disabled"
              @input="handleBlurInput"
              @change="emit('commit:blur')"
            />
          </div>

          <div class="appearance-background__row">
            <div class="appearance-background__label-wrap">
              <span class="appearance-background__label">{{
                i18n.t("settings.next.appearance.background.brightness")
              }}</span>
              <span class="appearance-background__value">{{ brightnessLabel }}</span>
            </div>

            <input
              class="appearance-background__slider"
              type="range"
              min="0"
              max="2"
              step="0.1"
              :value="props.brightness"
              :disabled="props.disabled"
              @input="handleBrightnessInput"
              @change="emit('commit:brightness')"
            />
          </div>

          <div class="appearance-background__row appearance-background__row--select">
            <div class="appearance-background__label-wrap">
              <span class="appearance-background__label">{{
                i18n.t("settings.next.appearance.background.size")
              }}</span>
              <span class="appearance-background__hint">{{
                i18n.t("settings.next.appearance.background.size_hint")
              }}</span>
            </div>

            <div class="appearance-background__select-wrap">
              <SLSelect
                :model-value="props.backgroundSize"
                :options="props.backgroundSizeOptions"
                :disabled="props.disabled"
                @update:model-value="handleBackgroundSizeChange"
              />
            </div>
          </div>

          <div v-if="hasImage" class="appearance-background__footer-actions">
            <SLButton variant="ghost" size="sm" @click="emit('pick-image')">
              <ImagePlus :size="15" />
              {{ i18n.t("settings.next.appearance.background.reselect") }}
            </SLButton>
            <SLButton variant="ghost" size="sm" @click="emit('remove-image')">
              <Trash2 :size="15" />
              {{ i18n.t("settings.next.appearance.background.remove_image") }}
            </SLButton>
          </div>
        </div>
      </div>
    </Transition>
  </section>
</template>

<style scoped>
.appearance-background {
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 62%, transparent);
  overflow: hidden;
}

.appearance-background__header {
  width: 100%;
  padding: 14px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  border: none;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.appearance-background__header:hover {
  background: color-mix(in srgb, var(--sl-primary) 4%, transparent);
}

.appearance-background__header:disabled {
  cursor: not-allowed;
  opacity: 0.65;
}

.appearance-background__heading {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.appearance-background__title {
  font-size: 0.92rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.appearance-background__meta {
  color: var(--sl-text-secondary);
  font-size: 0.82rem;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.appearance-background__toggle {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-surface) 82%, transparent);
  color: var(--sl-text-secondary);
  transition:
    transform 0.2s ease,
    background-color 0.2s ease,
    color 0.2s ease;
  flex: none;
}

.appearance-background__toggle.is-expanded {
  transform: rotate(180deg);
  background: color-mix(in srgb, var(--sl-primary) 10%, transparent);
  color: var(--sl-primary);
}

.appearance-background__content {
  padding: 14px 16px 16px;
  display: grid;
  gap: 16px;
  border-top: 1px solid color-mix(in srgb, var(--sl-border) 68%, transparent);
}

.appearance-background__preview-block {
  display: grid;
}

.appearance-background__preview-frame {
  position: relative;
  min-height: 208px;
  border-radius: 14px;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 88%, transparent);
}

.appearance-background__preview-image {
  width: 100%;
  height: 208px;
  object-fit: cover;
  display: block;
}

.appearance-background__preview-loading {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--sl-text-secondary);
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
}

.appearance-background__spinner {
  width: 24px;
  height: 24px;
  border-radius: 999px;
  border: 3px solid color-mix(in srgb, var(--sl-primary) 18%, transparent);
  border-top-color: var(--sl-primary);
  animation: appearance-background-spin 0.8s linear infinite;
}

.appearance-background__badge {
  position: absolute;
  top: 10px;
  right: 10px;
  padding: 3px 8px;
  border-radius: 999px;
  background: rgba(15, 23, 42, 0.72);
  color: white;
  font-size: 0.72rem;
}

.appearance-background__preview-overlay {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  background: linear-gradient(180deg, transparent, rgba(15, 23, 42, 0.76));
}

.appearance-background__image-name {
  min-width: 0;
  color: white;
  font-size: 0.82rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.appearance-background__preview-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.appearance-background__empty-state {
  padding: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  border: 1px dashed color-mix(in srgb, var(--sl-border) 72%, transparent);
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-surface) 72%, transparent);
}

.appearance-background__empty-copy {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.appearance-background__empty-copy strong,
.appearance-background__empty-copy span {
  margin: 0;
}

.appearance-background__empty-copy strong {
  color: var(--sl-text-primary);
  font-size: 0.9rem;
}

.appearance-background__empty-copy span {
  color: var(--sl-text-secondary);
  font-size: 0.82rem;
  line-height: 1.45;
}

.appearance-background__rows {
  display: grid;
  gap: 0;
}

.appearance-background__row {
  display: grid;
  gap: 8px;
  padding: 10px 0;
}

.appearance-background__row + .appearance-background__row {
  border-top: 1px solid color-mix(in srgb, var(--sl-border) 64%, transparent);
}

.appearance-background__row--select {
  grid-template-columns: minmax(0, 1fr) minmax(220px, 320px);
  align-items: center;
  gap: 14px;
}

.appearance-background__label-wrap {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.appearance-background__row--select .appearance-background__label-wrap {
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  gap: 4px;
}

.appearance-background__label {
  color: var(--sl-text-primary);
  font-size: 0.88rem;
  font-weight: 600;
}

.appearance-background__value,
.appearance-background__hint {
  color: var(--sl-text-secondary);
  font-size: 0.82rem;
}

.appearance-background__slider {
  width: 100%;
  height: 4px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-border) 90%, transparent);
  outline: none;
  -webkit-appearance: none;
}

.appearance-background__slider::-webkit-slider-thumb {
  width: 14px;
  height: 14px;
  border-radius: 999px;
  border: none;
  background: var(--sl-primary);
  cursor: pointer;
  -webkit-appearance: none;
}

.appearance-background__slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 999px;
  border: none;
  background: var(--sl-primary);
  cursor: pointer;
}

.appearance-background__select-wrap {
  width: 100%;
}

.appearance-background__footer-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  padding-top: 12px;
  border-top: 1px solid color-mix(in srgb, var(--sl-border) 64%, transparent);
}

.appearance-background-collapse-enter-active,
.appearance-background-collapse-leave-active {
  transition: all 0.22s ease;
  overflow: hidden;
}

.appearance-background-collapse-enter-from,
.appearance-background-collapse-leave-to {
  opacity: 0;
  max-height: 0;
}

.appearance-background-collapse-enter-to,
.appearance-background-collapse-leave-from {
  opacity: 1;
  max-height: 1000px;
}

@keyframes appearance-background-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 900px) {
  .appearance-background__row--select {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .appearance-background__empty-state,
  .appearance-background__preview-overlay {
    flex-direction: column;
    align-items: flex-start;
  }

  .appearance-background__preview-actions {
    justify-content: flex-start;
  }
}
</style>
