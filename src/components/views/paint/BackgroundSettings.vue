<script setup lang="ts">
import { ChevronDown } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { i18n } from "@language";
import { computed, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";

const props = defineProps<{
  backgroundImage: string;
  bgOpacity: string;
  bgBlur: string;
  bgBrightness: string;
  backgroundSize: string;
  expanded: boolean;
}>();

const emit = defineEmits<{
  (e: "update:expanded", value: boolean): void;
  (e: "update:bgOpacity", value: string): void;
  (e: "update:bgBlur", value: string): void;
  (e: "update:bgBrightness", value: string): void;
  (e: "update:backgroundSize", value: string): void;
  (e: "pickImage"): void;
  (e: "clearImage"): void;
  (e: "change"): void;
}>();

const bgPreviewLoaded = ref(false);
const bgPreviewLoading = ref(false);

const backgroundSizeOptions = computed(() => [
  { label: i18n.t("settings.background_size_options.cover"), value: "cover" },
  { label: i18n.t("settings.background_size_options.contain"), value: "contain" },
  { label: i18n.t("settings.background_size_options.fill"), value: "fill" },
  { label: i18n.t("settings.background_size_options.auto"), value: "auto" },
]);

const backgroundPreviewUrl = computed(() => {
  if (!props.backgroundImage) return "";
  if (!props.expanded) return "";
  return convertFileSrc(props.backgroundImage);
});

function getFileExtension(path: string): string {
  return path.split(".").pop()?.toLowerCase() || "";
}

function isAnimatedImage(path: string): boolean {
  const ext = getFileExtension(path);
  return ext === "gif" || ext === "webp" || ext === "apng";
}

watch(
  () => props.expanded,
  (expanded) => {
    if (expanded && props.backgroundImage) {
      bgPreviewLoaded.value = false;
      bgPreviewLoading.value = true;
    }
  },
);

watch(
  () => props.backgroundImage,
  (backgroundImage, previousBackgroundImage) => {
    if (props.expanded && backgroundImage && backgroundImage !== previousBackgroundImage) {
      bgPreviewLoaded.value = false;
      bgPreviewLoading.value = true;
    }
  },
);

function toggleExpanded() {
  emit("update:expanded", !props.expanded);
}

function handleOpacityChange(e: Event) {
  emit("update:bgOpacity", (e.target as HTMLInputElement).value);
  emit("change");
}

function handleBlurChange(e: Event) {
  emit("update:bgBlur", (e.target as HTMLInputElement).value);
  emit("change");
}

function handleBrightnessChange(e: Event) {
  emit("update:bgBrightness", (e.target as HTMLInputElement).value);
  emit("change");
}

function handleBackgroundSizeChange(value: string) {
  emit("update:backgroundSize", value);
  emit("change");
}

function handleImageLoad() {
  bgPreviewLoaded.value = true;
  bgPreviewLoading.value = false;
}
</script>

<template>
  <div class="collapsible-section">
    <div class="collapsible-header" @click="toggleExpanded">
      <div class="setting-info">
        <span class="setting-label">{{ i18n.t("settings.background") }}</span>
        <span class="setting-desc">{{ i18n.t("settings.background_desc") }}</span>
      </div>
      <div class="collapsible-toggle" :class="{ expanded }">
        <ChevronDown :size="20" />
      </div>
    </div>
    <Transition name="collapse">
      <div v-show="expanded" class="collapsible-content">
        <div class="setting-row full-width">
          <div class="bg-image-picker">
            <div v-if="backgroundImage" class="bg-preview">
              <div v-if="bgPreviewLoading && !bgPreviewLoaded" class="bg-preview-loading">
                <div class="loading-spinner"></div>
                <span>{{ i18n.t("settings.loading_preview") }}</span>
              </div>
              <img
                v-show="bgPreviewLoaded || !bgPreviewLoading"
                :src="backgroundPreviewUrl"
                alt="Background preview"
                @load="handleImageLoad"
                @loadstart="bgPreviewLoading = true"
                @error="bgPreviewLoading = false"
                loading="lazy"
              />
              <div v-if="isAnimatedImage(backgroundImage)" class="bg-animated-badge">
                {{ i18n.t("settings.animated_image") }}
              </div>
              <div class="bg-preview-overlay">
                <span class="bg-preview-path">{{ backgroundImage.split("\\").pop() }}</span>
                <SLButton variant="danger" size="sm" @click="emit('clearImage')">{{
                  i18n.t("settings.remove")
                }}</SLButton>
              </div>
            </div>
            <SLButton v-else variant="secondary" @click="emit('pickImage')">
              {{ i18n.t("settings.pick_image") }}
            </SLButton>
            <SLButton
              v-if="backgroundImage"
              variant="secondary"
              size="sm"
              @click="emit('pickImage')"
            >
              {{ i18n.t("settings.replace_image") }}
            </SLButton>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">{{ i18n.t("settings.opacity") }}</span>
            <span class="setting-desc">{{ i18n.t("settings.opacity_desc") }}</span>
          </div>
          <div class="slider-control">
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              :value="bgOpacity"
              @input="handleOpacityChange"
              class="sl-slider"
            />
            <span class="slider-value">{{ bgOpacity }}</span>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">{{ i18n.t("settings.blur") }}</span>
            <span class="setting-desc">{{ i18n.t("settings.blur_desc") }}</span>
          </div>
          <div class="slider-control">
            <input
              type="range"
              min="0"
              max="20"
              step="1"
              :value="bgBlur"
              @input="handleBlurChange"
              class="sl-slider"
            />
            <span class="slider-value">{{ bgBlur }}px</span>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">{{ i18n.t("settings.brightness") }}</span>
            <span class="setting-desc">{{ i18n.t("settings.brightness_desc") }}</span>
          </div>
          <div class="slider-control">
            <input
              type="range"
              min="0"
              max="2"
              step="0.1"
              :value="bgBrightness"
              @input="handleBrightnessChange"
              class="sl-slider"
            />
            <span class="slider-value">{{ bgBrightness }}</span>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">{{ i18n.t("settings.background_size") }}</span>
            <span class="setting-desc">{{ i18n.t("settings.background_size_desc") }}</span>
          </div>
          <div class="input-lg">
            <SLSelect
              :model-value="backgroundSize"
              :options="backgroundSizeOptions"
              @update:model-value="handleBackgroundSizeChange"
            />
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.collapsible-section {
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  margin: var(--sl-space-sm) 0;
}

.collapsible-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md);
  cursor: pointer;
  background: var(--sl-surface);
  transition: background-color var(--sl-transition-fast);
}

.collapsible-header:hover {
  background: var(--sl-surface-hover);
}

.collapsible-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-normal);
  flex-shrink: 0;
}

.collapsible-toggle:hover {
  background: var(--sl-border-light);
  color: var(--sl-text-primary);
}

.collapsible-toggle.expanded {
  transform: rotate(180deg);
}

.collapsible-content {
  padding: 0 var(--sl-space-md) var(--sl-space-md);
  background: var(--sl-surface);
}

.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  opacity: 1;
  max-height: 800px;
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

.setting-row.full-width {
  flex-direction: column;
  align-items: stretch;
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

.bg-image-picker {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
}

.bg-preview {
  position: relative;
  width: 100%;
  max-width: 400px;
  height: 200px;
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  border: 1px solid var(--sl-border);
}

.bg-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.bg-preview-loading {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  background: var(--sl-surface);
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.bg-animated-badge {
  position: absolute;
  top: var(--sl-space-sm);
  right: var(--sl-space-sm);
  padding: 2px 8px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  font-size: 0.75rem;
  font-weight: 500;
  border-radius: var(--sl-radius-sm);
}

.bg-preview-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-sm);
}

.bg-preview-path {
  font-size: 0.8125rem;
  color: white;
  font-family: var(--sl-font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
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
