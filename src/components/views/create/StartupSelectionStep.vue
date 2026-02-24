<script setup lang="ts">
import { FileCode2, FileCog, FolderOpen, RefreshCw, TerminalSquare } from "lucide-vue-next";
import SLInput from "@components/common/SLInput.vue";
import type { StartupCandidate, StartupMode } from "@components/views/create/startupTypes";
import { i18n } from "@language";

const props = withDefaults(
  defineProps<{
    loading: boolean;
    candidates: StartupCandidate[];
    selectedStartupId: string;
    customStartupCommand: string;
    customCommandHasRedirect: boolean;
    disabled?: boolean;
  }>(),
  {
    disabled: false,
  },
);

const emit = defineEmits<{
  (e: "rescan"): void;
  (e: "update:selectedStartupId", value: string): void;
  (e: "update:customStartupCommand", value: string): void;
}>();

function startupModeLabel(mode: StartupMode): string {
  switch (mode) {
    case "starter":
      return i18n.t("create.startup_mode_starter");
    case "jar":
      return i18n.t("create.startup_mode_jar");
    case "bat":
    case "sh":
    case "ps1":
      return i18n.t("create.startup_mode_script");
    default:
      return i18n.t("create.startup_mode_custom");
  }
}

function candidateRecommendedText(candidate: StartupCandidate): string {
  if (candidate.recommended <= 1) {
    return i18n.t("create.startup_recommended_high");
  }
  if (candidate.recommended === 2) {
    return i18n.t("create.startup_recommended_medium");
  }
  return i18n.t("create.startup_recommended_low");
}

function getStartupIcon(mode: StartupMode) {
  switch (mode) {
    case "starter":
      return FileCog;
    case "jar":
      return FileCode2;
    case "bat":
    case "sh":
    case "ps1":
      return TerminalSquare;
    default:
      return FolderOpen;
  }
}
</script>

<template>
  <div class="startup-step-block">
    <div class="startup-step-header">
      <span class="startup-step-title">{{ i18n.t("create.startup_title") }}</span>
      <button type="button" class="startup-step-rescan" :disabled="disabled" @click="emit('rescan')">
        <RefreshCw :size="14" />
        {{ i18n.t("create.rescan") }}
      </button>
    </div>

    <p class="startup-step-hint">{{ i18n.t("create.startup_hint") }}</p>

    <div v-if="loading" class="startup-step-empty">{{ i18n.t("create.startup_scanning") }}</div>
    <div v-else-if="candidates.length === 0" class="startup-step-empty">
      {{ i18n.t("create.startup_empty") }}
    </div>
    <div v-else class="startup-grid">
      <button
        v-for="candidate in candidates"
        :key="candidate.id"
        type="button"
        class="startup-card"
        :class="{
          active: selectedStartupId === candidate.id,
          recommended: candidate.recommended <= 2,
        }"
        :disabled="disabled"
        @click="emit('update:selectedStartupId', candidate.id)"
      >
        <div class="startup-card-icon">
          <component :is="getStartupIcon(candidate.mode)" :size="16" />
        </div>
        <div class="startup-card-copy">
          <div class="startup-card-title-row">
            <p class="startup-card-title">{{ candidate.label }}</p>
            <span class="startup-mode-badge">{{ startupModeLabel(candidate.mode) }}</span>
          </div>
          <p class="startup-card-detail">{{ candidate.detail || candidate.path || "-" }}</p>
          <span class="startup-recommend">{{ candidateRecommendedText(candidate) }}</span>
        </div>
      </button>
    </div>

    <div
      v-if="candidates.find((item) => item.id === selectedStartupId)?.mode === 'custom'"
      class="startup-custom"
    >
      <SLInput
        :label="i18n.t('create.startup_custom_label')"
        :model-value="customStartupCommand"
        :disabled="disabled"
        :placeholder="i18n.t('create.startup_custom_placeholder')"
        @update:model-value="emit('update:customStartupCommand', $event)"
      />
      <p class="startup-step-hint">{{ i18n.t("create.startup_custom_hint") }}</p>
      <p v-if="customCommandHasRedirect" class="startup-step-error">
        {{ i18n.t("create.startup_custom_redirect_forbidden") }}
      </p>
      <p class="startup-step-warning">{{ i18n.t("create.startup_custom_not_supported") }}</p>
    </div>
  </div>
</template>

<style src="@styles/components/views/create/StartupSelectionStep.css" scoped></style>
