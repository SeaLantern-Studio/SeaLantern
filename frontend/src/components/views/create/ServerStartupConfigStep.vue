<script setup lang="ts">
import SLInput from "@components/common/SLInput.vue";
import SLModal from "@components/common/SLModal.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLTextarea from "@components/common/SLTextarea.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLTooltip from "@components/common/SLTooltip.vue";
import CpuPolicyEditor from "@components/startup/CpuPolicyEditor.vue";
import { i18n } from "@language";
import { computed, ref } from "vue";
import {
  MVP_JVM_PRESET_IDS,
  getJvmPresetArgs,
  getJvmPresetPreviewArgs,
  serializeJvmArgsText,
} from "@utils/serverStartupConfig";
import { compactPathMiddle } from "@utils/pathDisplay";
import type { CpuPolicyConfig, JvmPresetId, LocalStartupMode } from "@type/server";

const props = defineProps<{
  serverName: string;
  maxMemory: string;
  minMemory: string;
  port: string;
  onlineMode: boolean;
  jvmArgsText: string;
  jvmPreset: JvmPresetId;
  cpuPolicy: CpuPolicyConfig;
  startupMode?: LocalStartupMode;
  startupTarget?: string;
  customCommandPreview?: string;
  showOnlineMode?: boolean;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:serverName", value: string): void;
  (e: "update:maxMemory", value: string): void;
  (e: "update:minMemory", value: string): void;
  (e: "update:port", value: string): void;
  (e: "update:onlineMode", value: boolean): void;
  (e: "update:jvmArgsText", value: string): void;
  (e: "update:jvmPreset", value: JvmPresetId): void;
  (e: "update:cpuPolicy", value: CpuPolicyConfig): void;
}>();

const jvmPresetOptions = computed(() =>
  MVP_JVM_PRESET_IDS.map((preset) => ({
    value: preset,
    label: i18n.t(`common.jvm_preset_${preset}_label`),
    subLabel: i18n.t(`common.jvm_preset_${preset}_desc`),
  })),
);

const selectedJvmPresetOption = computed(
  () => jvmPresetOptions.value.find((option) => option.value === props.jvmPreset) ?? null,
);

const parsedJvmArgs = computed(() => serializeJvmArgsText(props.jvmArgsText));
const jvmPresetPreviewArgs = computed(() => getJvmPresetPreviewArgs(props.jvmPreset));
const fullJvmPresetArgs = computed(() => getJvmPresetArgs(props.jvmPreset));
const jvmPresetArgCount = computed(() => getJvmPresetArgs(props.jvmPreset).length);
const showPresetArgsModal = ref(false);

const startupModeLabel = computed(() => {
  switch (props.startupMode) {
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
  if (props.startupMode === "custom") {
    return props.customCommandPreview?.trim() || i18n.t("create.startup_command_pending");
  }

  return props.startupTarget?.trim() || i18n.t("create.startup_command_pending");
});

const startupTargetDisplay = computed(() => compactPathMiddle(startupTargetSummary.value, 64));

function handleNumberInput(event: Event, type: "maxMemory" | "minMemory" | "port") {
  const target = event.target as HTMLInputElement;
  const value = target.value;
  if (value === "" || /^\d+$/.test(value)) {
    if (type === "maxMemory") {
      emit("update:maxMemory", value);
      return;
    }
    if (type === "minMemory") {
      emit("update:minMemory", value);
      return;
    }
    emit("update:port", value);
  }
}
</script>

<template>
  <div class="startup-step">
    <div class="startup-list">
      <div class="startup-row">
        <span class="startup-row-label">{{ i18n.t("create.server_name") }}</span>
        <SLInput
          :placeholder="i18n.t('create.server_name')"
          :model-value="serverName"
          :disabled="disabled"
          @update:model-value="$emit('update:serverName', $event)"
        />
      </div>

      <div class="startup-pair-row">
        <div class="startup-pair-item">
          <span class="startup-row-label">{{ i18n.t("create.max_memory") }}</span>
          <SLInput
            type="text"
            :model-value="maxMemory"
            :disabled="disabled"
            @input="handleNumberInput($event, 'maxMemory')"
          />
        </div>
        <div class="startup-pair-item">
          <span class="startup-row-label">{{ i18n.t("create.min_memory") }}</span>
          <SLInput
            type="text"
            :model-value="minMemory"
            :disabled="disabled"
            @input="handleNumberInput($event, 'minMemory')"
          />
        </div>
      </div>

      <div class="startup-pair-row">
        <div class="startup-pair-item">
          <span class="startup-row-label">{{ i18n.t("settings.default_port") }}</span>
          <SLInput
            type="text"
            :model-value="port"
            :placeholder="i18n.t('create.default_port_placeholder')"
            :disabled="disabled"
            @input="handleNumberInput($event, 'port')"
          />
        </div>

        <div v-if="showOnlineMode !== false" class="startup-pair-item">
          <span class="startup-row-label">{{ i18n.t("create.online_mode") }}</span>
          <div class="startup-row-control startup-online-box">
            <span class="startup-online-text">
              {{ onlineMode ? i18n.t("create.online_mode_on") : i18n.t("create.online_mode_off") }}
            </span>
            <SLSwitch
              :model-value="onlineMode"
              :disabled="disabled"
              @update:model-value="$emit('update:onlineMode', $event)"
            />
          </div>
        </div>
      </div>

      <div class="startup-advanced-block">
        <div class="startup-advanced-header">
          <span class="startup-advanced-title">{{ i18n.t("create.advanced_startup_title") }}</span>
          <p class="startup-advanced-desc">{{ i18n.t("create.advanced_startup_desc") }}</p>
          <p class="startup-advanced-notice">
            {{ i18n.t("create.startup_notice_not_main_thread_boost") }}
          </p>
        </div>

        <div class="startup-advanced-field">
          <span class="startup-row-label">{{ i18n.t("create.cpu_policy_label") }}</span>
          <CpuPolicyEditor
            :model-value="cpuPolicy"
            scope="create"
            :disabled="disabled"
            @update:modelValue="$emit('update:cpuPolicy', $event)"
          />
          <p class="startup-field-hint">{{ i18n.t("create.cpu_policy_desc") }}</p>
        </div>

        <div class="startup-advanced-field">
          <span class="startup-row-label">{{ i18n.t("create.jvm_preset_label") }}</span>
          <SLSelect
            :model-value="jvmPreset"
            :options="jvmPresetOptions"
            :placeholder="i18n.t('create.jvm_preset_placeholder')"
            :disabled="disabled"
            @update:model-value="$emit('update:jvmPreset', $event as JvmPresetId)"
          />
          <p class="startup-field-hint">{{ i18n.t("create.jvm_preset_desc") }}</p>
        </div>

        <div class="startup-advanced-field">
          <span class="startup-row-label">{{ i18n.t("create.jvm_args_label") }}</span>
          <SLTextarea
            :model-value="jvmArgsText"
            :placeholder="i18n.t('create.jvm_args_placeholder')"
            :disabled="disabled"
            :rows="4"
            @update:model-value="$emit('update:jvmArgsText', $event)"
          />
          <p class="startup-field-hint">{{ i18n.t("create.jvm_args_desc") }}</p>
        </div>

        <div class="startup-preview-card">
          <div class="startup-preview-header">
            <span class="startup-preview-title">{{ i18n.t("create.startup_preview_title") }}</span>
            <p class="startup-preview-desc">{{ i18n.t("create.startup_preview_desc") }}</p>
          </div>

          <div class="startup-preview-grid">
            <div class="startup-preview-item">
              <span class="startup-preview-label">{{ i18n.t("create.startup_preview_mode") }}</span>
              <span class="startup-preview-value">{{ startupModeLabel }}</span>
            </div>
            <div class="startup-preview-item">
              <span class="startup-preview-label">{{
                i18n.t("create.startup_preview_preset")
              }}</span>
              <span class="startup-preview-value">
                {{ selectedJvmPresetOption?.label || i18n.t("common.unknown") }}
              </span>
            </div>
          </div>

          <div class="startup-preview-block">
            <span class="startup-preview-label">{{ i18n.t("create.startup_preview_target") }}</span>
            <SLTooltip :content="startupTargetSummary">
              <code class="startup-preview-code startup-preview-code--single-line">{{
                startupTargetDisplay
              }}</code>
            </SLTooltip>
          </div>

          <div class="startup-preview-block">
            <span class="startup-preview-label">{{
              i18n.t("create.startup_preview_preset_args")
            }}</span>
            <div v-if="jvmPresetArgCount === 0" class="startup-preview-empty">
              {{ i18n.t("create.startup_preview_none") }}
            </div>
            <div v-else class="startup-preview-tags">
              <span v-for="arg in jvmPresetPreviewArgs" :key="arg" class="startup-preview-tag">
                {{ arg }}
              </span>
              <span
                v-if="jvmPresetArgCount > jvmPresetPreviewArgs.length"
                class="startup-preview-tag startup-preview-tag--muted startup-preview-tag--button"
                role="button"
                tabindex="0"
                @click="showPresetArgsModal = true"
                @keydown.enter.prevent="showPresetArgsModal = true"
                @keydown.space.prevent="showPresetArgsModal = true"
              >
                {{
                  i18n.t("create.startup_preview_more_args", {
                    count: jvmPresetArgCount - jvmPresetPreviewArgs.length,
                  })
                }}
              </span>
            </div>
          </div>

          <div class="startup-preview-block">
            <span class="startup-preview-label">{{
              i18n.t("create.startup_preview_extra_args")
            }}</span>
            <div v-if="parsedJvmArgs.length === 0" class="startup-preview-empty">
              {{ i18n.t("create.startup_preview_none") }}
            </div>
            <div v-else class="startup-preview-tags">
              <span v-for="arg in parsedJvmArgs" :key="arg" class="startup-preview-tag">
                {{ arg }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <SLModal
      :visible="showPresetArgsModal"
      :title="i18n.t('create.startup_preview_preset_args')"
      width="720px"
      @close="showPresetArgsModal = false"
    >
      <div class="startup-preview-modal-body">
        <p class="startup-preview-modal-desc">
          {{ selectedJvmPresetOption?.label || i18n.t("common.unknown") }}
        </p>
        <div v-if="fullJvmPresetArgs.length === 0" class="startup-preview-empty">
          {{ i18n.t("create.startup_preview_none") }}
        </div>
        <div v-else class="startup-preview-modal-list">
          <code v-for="arg in fullJvmPresetArgs" :key="arg" class="startup-preview-modal-code">
            {{ arg }}
          </code>
        </div>
      </div>
    </SLModal>
  </div>
</template>

<style src="@styles/components/views/create/ServerStartupConfigStep.css" scoped></style>
