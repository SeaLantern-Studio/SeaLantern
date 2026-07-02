<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";

defineProps<{
  title: string;
  presets: Record<string, Record<string, string>>;
  selectedPreset?: string | number | boolean | null;
}>();

const emit = defineEmits<{
  (e: "select", presetKey: string): void;
}>();

function isActivePreset(
  selectedPreset: string | number | boolean | null | undefined,
  presetKey: string,
) {
  return selectedPreset != null && String(selectedPreset) === presetKey;
}

function getPresetName(presetKey: string, presetData: Record<string, string>) {
  return presetData.name ?? presetKey;
}
</script>

<template>
  <SLCard class="preset-card">
    <h3 class="section-title">{{ title }}</h3>
    <div class="presets-grid">
      <button
        v-for="(presetData, presetKey) in presets"
        :key="presetKey"
        type="button"
        class="preset-btn"
        :class="{ active: isActivePreset(selectedPreset, String(presetKey)) }"
        @click="emit('select', String(presetKey))"
      >
        <div class="preset-swatches">
          <span
            v-if="presetData.accent_primary"
            class="preset-swatch"
            :style="{ background: presetData.accent_primary }"
          ></span>
          <span
            v-if="presetData.accent_secondary"
            class="preset-swatch"
            :style="{ background: presetData.accent_secondary }"
          ></span>
        </div>
        <span class="preset-name">{{ getPresetName(String(presetKey), presetData) }}</span>
      </button>
    </div>
  </SLCard>
</template>

<style scoped>
.preset-card {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-title {
  margin: 0;
  font-size: var(--sl-font-size-lg);
  font-weight: 600;
  color: var(--sl-text-primary);
}

.presets-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.preset-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  min-width: 88px;
  padding: 12px 16px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease,
    color 0.2s ease;
}

.preset-btn:hover {
  border-color: var(--sl-border-strong, var(--sl-primary));
  background: var(--sl-bg-tertiary);
}

.preset-btn.active {
  border-color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

.preset-swatches {
  display: flex;
  gap: 4px;
  min-height: 14px;
}

.preset-swatch {
  width: 14px;
  height: 14px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.12);
}

.preset-name {
  font-size: var(--sl-font-size-xs);
}
</style>
