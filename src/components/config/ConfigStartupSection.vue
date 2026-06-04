<script setup lang="ts">
import { computed } from "vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLTextarea from "@components/common/SLTextarea.vue";
import { i18n } from "@language";
import { useStartupConfigSection } from "@components/config/useStartupConfigSection";
import {
  MVP_JVM_PRESET_IDS,
  getJvmPresetArgs,
  getJvmPresetPreviewArgs,
  serializeJvmArgsText,
} from "@utils/serverStartupConfig";
import type { JvmPresetId, LocalStartupMode } from "@type/server";
import { useServerStore } from "@stores/serverStore";

const props = defineProps<{
  serverPath: string;
  defaultMaxMemory: number;
  defaultMinMemory: number;
}>();

const emit = defineEmits<{
  (e: "saved", maxMemory: number, minMemory: number): void;
}>();

const serverStore = useServerStore();

const startupConfig = useStartupConfigSection({
  serverPath: computed(() => props.serverPath),
  defaultMaxMemory: computed(() => props.defaultMaxMemory),
  defaultMinMemory: computed(() => props.defaultMinMemory),
  onSaved: (maxMemory, minMemory) => {
    emit("saved", maxMemory, minMemory);
  },
});

const jvmPresetOptions = computed(() =>
  MVP_JVM_PRESET_IDS.map((preset) => ({
    value: preset,
    label: i18n.t(`common.jvm_preset_${preset}_label`),
    subLabel: i18n.t(`common.jvm_preset_${preset}_desc`),
  })),
);

const currentServer = computed(
  () => serverStore.servers.find((server) => server.path === props.serverPath) ?? null,
);

const startupModeLabel = computed(() => {
  const mode =
    currentServer.value?.runtime.kind === "local"
      ? (currentServer.value.runtime.startup_mode as LocalStartupMode)
      : undefined;
  switch (mode) {
    case "starter":
      return i18n.t("create.startup_mode_starter");
    case "jar":
      return i18n.t("create.startup_mode_jar");
    case "bat":
    case "sh":
    case "ps1":
      return i18n.t("create.startup_mode_script");
    case "custom":
      return i18n.t("create.startup_mode_custom");
    default:
      return i18n.t("common.unknown");
  }
});

const startupTargetSummary = computed(() => {
  const runtime = currentServer.value?.runtime;
  if (!runtime) {
    return i18n.t("config.startup_preview_pending");
  }

  if (runtime.kind === "local") {
    if (runtime.startup_mode === "custom") {
      return runtime.custom_command?.trim() || i18n.t("config.startup_preview_pending");
    }
    return runtime.jar_path?.trim() || i18n.t("config.startup_preview_pending");
  }

  return runtime.image
    ? `${runtime.image}:${runtime.image_tag}`
    : i18n.t("config.startup_preview_pending");
});

const selectedJvmPresetOption = computed(
  () =>
    jvmPresetOptions.value.find(
      (option) => option.value === startupConfig.jvmPreset.value.preset,
    ) ?? null,
);

const parsedJvmArgs = computed(() => serializeJvmArgsText(startupConfig.jvmArgsText.value));
const jvmPresetPreviewArgs = computed(() =>
  getJvmPresetPreviewArgs(startupConfig.jvmPreset.value.preset),
);
const jvmPresetArgCount = computed(
  () => getJvmPresetArgs(startupConfig.jvmPreset.value.preset).length,
);
</script>

<template>
  <div class="config-startup-section">
    <div v-if="startupConfig.loading" class="loading-state">
      <p class="text-caption">{{ i18n.t("common.loading") }}</p>
    </div>
    <template v-else>
      <div v-if="startupConfig.error.value" class="error-banner">
        <span>{{ startupConfig.error.value }}</span>
        <button class="banner-close" @click="startupConfig.error.value = null">x</button>
      </div>
      <div class="startup-notice glass-card">
        <p class="text-caption">{{ i18n.t("config.startup_notice_not_main_thread_boost") }}</p>
      </div>
      <div class="startup-entries">
        <div class="config-entry glass-card">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key">{{ i18n.t("config.max_memory") }}</span>
            </div>
            <p class="entry-desc text-caption">{{ i18n.t("config.max_memory_desc") }}</p>
          </div>
          <div class="entry-control">
            <SLInput
              :modelValue="String(startupConfig.maxMemory.value)"
              @update:modelValue="startupConfig.maxMemory.value = Number($event)"
              type="number"
              :placeholder="'2048'"
              :min="128"
              :step="128"
            />
          </div>
        </div>
        <div class="config-entry glass-card">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key">{{ i18n.t("config.min_memory") }}</span>
            </div>
            <p class="entry-desc text-caption">{{ i18n.t("config.min_memory_desc") }}</p>
          </div>
          <div class="entry-control">
            <SLInput
              :modelValue="String(startupConfig.minMemory.value)"
              @update:modelValue="startupConfig.minMemory.value = Number($event)"
              type="number"
              :placeholder="'512'"
              :min="128"
              :step="128"
            />
          </div>
        </div>
        <div class="config-entry glass-card config-entry--stacked">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key">{{ i18n.t("config.jvm_preset_label") }}</span>
            </div>
            <p class="entry-desc text-caption">{{ i18n.t("config.jvm_preset_desc") }}</p>
          </div>
          <div class="entry-control entry-control--full">
            <SLSelect
              :model-value="startupConfig.jvmPreset.value.preset"
              :options="jvmPresetOptions"
              :placeholder="i18n.t('config.jvm_preset_placeholder')"
              :disabled="startupConfig.saving.value"
              @update:model-value="
                startupConfig.jvmPreset.value = { preset: $event as JvmPresetId }
              "
            />
          </div>
        </div>
        <div class="config-entry glass-card config-entry--stacked">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key">{{ i18n.t("config.jvm_args_label") }}</span>
            </div>
            <p class="entry-desc text-caption">{{ i18n.t("config.jvm_args_desc") }}</p>
          </div>
          <div class="entry-control entry-control--full">
            <SLTextarea
              :model-value="startupConfig.jvmArgsText.value"
              :placeholder="i18n.t('config.jvm_args_placeholder')"
              :disabled="startupConfig.saving.value"
              :rows="4"
              @update:model-value="startupConfig.jvmArgsText.value = $event"
            />
          </div>
        </div>
        <div class="config-entry glass-card config-entry--stacked startup-preview-card">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key">{{ i18n.t("config.startup_preview_title") }}</span>
            </div>
            <p class="entry-desc text-caption">{{ i18n.t("config.startup_preview_desc") }}</p>
          </div>
          <div class="startup-preview-grid">
            <div class="startup-preview-item">
              <span class="startup-preview-label">{{ i18n.t("config.startup_preview_mode") }}</span>
              <span class="startup-preview-value">{{ startupModeLabel }}</span>
            </div>
            <div class="startup-preview-item">
              <span class="startup-preview-label">{{
                i18n.t("config.startup_preview_preset")
              }}</span>
              <span class="startup-preview-value">
                {{ selectedJvmPresetOption?.label || i18n.t("common.unknown") }}
              </span>
            </div>
          </div>
          <div class="startup-preview-block">
            <span class="startup-preview-label">{{ i18n.t("config.startup_preview_target") }}</span>
            <code class="startup-preview-code">{{ startupTargetSummary }}</code>
          </div>
          <div class="startup-preview-block">
            <span class="startup-preview-label">{{
              i18n.t("config.startup_preview_preset_args")
            }}</span>
            <div v-if="jvmPresetArgCount === 0" class="startup-preview-empty">
              {{ i18n.t("config.startup_preview_none") }}
            </div>
            <div v-else class="startup-preview-tags">
              <span v-for="arg in jvmPresetPreviewArgs" :key="arg" class="startup-preview-tag">
                {{ arg }}
              </span>
              <span
                v-if="jvmPresetArgCount > jvmPresetPreviewArgs.length"
                class="startup-preview-tag startup-preview-tag--muted"
              >
                {{
                  i18n.t("config.startup_preview_more_args", {
                    count: jvmPresetArgCount - jvmPresetPreviewArgs.length,
                  })
                }}
              </span>
            </div>
          </div>
          <div class="startup-preview-block">
            <span class="startup-preview-label">{{
              i18n.t("config.startup_preview_extra_args")
            }}</span>
            <div v-if="parsedJvmArgs.length === 0" class="startup-preview-empty">
              {{ i18n.t("config.startup_preview_none") }}
            </div>
            <div v-else class="startup-preview-tags">
              <span v-for="arg in parsedJvmArgs" :key="arg" class="startup-preview-tag">
                {{ arg }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.config-startup-section {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: var(--sl-radius-md);
  font-size: var(--sl-font-size-base);
  background: var(--sl-error-bg);
  border: 1px solid color-mix(in srgb, var(--sl-error) 30%, transparent);
  color: var(--sl-error);
}

.banner-close {
  font-weight: 600;
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
}

.startup-entries {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.startup-notice {
  padding: 12px 16px;
  color: var(--sl-warning, #b45309);
}

.startup-notice p {
  margin: 0;
  line-height: 1.5;
}

.config-entry {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-config-entry-padding-block) var(--sl-space-md);
  gap: var(--sl-space-lg);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  transition: all var(--sl-transition-fast);
}

.config-entry:hover {
  border-color: var(--sl-border);
  box-shadow: var(--sl-shadow-sm);
}

.entry-header {
  flex: 1;
  min-width: 0;
}

.entry-key-row {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.entry-key {
  font-size: var(--sl-font-size-base);
  font-weight: 600;
  color: var(--sl-text-primary);
}

.entry-desc {
  margin-top: 2px;
}

.entry-control {
  flex-shrink: 0;
  min-width: var(--sl-config-control-width);
}

.config-entry--stacked {
  align-items: stretch;
  flex-direction: column;
}

.entry-control--full {
  width: 100%;
  min-width: 0;
}

.entry-control--full :deep(.sl-textarea) {
  font-family: var(--sl-font-mono);
}

.startup-preview-card {
  gap: var(--sl-space-sm);
}

.startup-preview-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--sl-space-sm);
}

.startup-preview-item,
.startup-preview-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.startup-preview-label {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-tertiary);
}

.startup-preview-value {
  color: var(--sl-text-primary);
  font-weight: 500;
}

.startup-preview-code {
  display: block;
  padding: 10px 12px;
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}

.startup-preview-empty {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
}

.startup-preview-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.startup-preview-tag {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 999px;
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  font-family: var(--sl-font-mono);
  font-size: 0.75rem;
  line-height: 1.4;
}

.startup-preview-tag--muted {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-tertiary);
}

@media (max-width: 768px) {
  .startup-preview-grid {
    grid-template-columns: 1fr;
  }
}
</style>
