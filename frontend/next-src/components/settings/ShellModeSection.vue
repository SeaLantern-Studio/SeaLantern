<script setup lang="ts">
import { computed } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { useShellRuntimeStatus } from "@composables/useShellRuntimeStatus";
import { i18n } from "@language";
const { browserOnly, loading, safeMode, errorMessage, currentShellName, refreshStatus } =
  useShellRuntimeStatus({ logScope: "ShellModeSection" });

const shellFacts = computed(() => [
  {
    label: i18n.t("settings.next.shell.current_label"),
    value: currentShellName.value,
  },
  {
    label: i18n.t("settings.next.shell.startup_label"),
    value: i18n.t("settings.next.shell.startup_value"),
  },
  {
    label: i18n.t("settings.next.shell.status_label"),
    value: safeMode.value
      ? i18n.t("settings.next.shell.safe_mode_on")
      : i18n.t("settings.next.shell.safe_mode_off"),
  },
]);
</script>

<template>
  <SLCard variant="outline" padding="lg" class="shell-mode-section">
    <div class="shell-mode-section__header">
      <div class="shell-mode-section__copy">
        <p class="shell-mode-section__eyebrow">{{ i18n.t("settings.next.shell.eyebrow") }}</p>
        <h4 class="shell-mode-section__title">{{ i18n.t("settings.next.shell.title") }}</h4>
        <p class="shell-mode-section__description">
          {{ i18n.t("settings.next.shell.description") }}
        </p>
      </div>

      <SLButton variant="secondary" size="sm" :loading="loading" @click="refreshStatus">
        {{ i18n.t("settings.next.refresh") }}
      </SLButton>
    </div>

    <div v-if="browserOnly" class="shell-mode-section__notice shell-mode-section__notice--neutral">
      {{ i18n.t("plugins.ui_shell.desktop_only") }}
    </div>

    <div
      v-else-if="safeMode"
      class="shell-mode-section__notice shell-mode-section__notice--neutral"
    >
      {{ i18n.t("settings.next.shell.safe_mode_hint") }}
    </div>

    <div v-if="errorMessage" class="shell-mode-section__notice shell-mode-section__notice--error">
      {{ errorMessage }}
    </div>

    <div v-else-if="loading" class="shell-mode-section__loading">
      <span class="shell-mode-section__spinner" aria-hidden="true" />
      <span>{{ i18n.t("plugins.ui_shell.loading") }}</span>
    </div>

    <div v-else-if="!browserOnly" class="shell-mode-section__body">
      <dl class="shell-mode-section__facts">
        <div v-for="fact in shellFacts" :key="fact.label" class="shell-mode-section__fact">
          <dt>{{ fact.label }}</dt>
          <dd>{{ fact.value }}</dd>
        </div>
      </dl>

      <p class="shell-mode-section__readonly-hint">
        {{ i18n.t("settings.next.shell.readonly_hint") }}
      </p>
    </div>
  </SLCard>
</template>

<style scoped>
.shell-mode-section {
  display: grid;
  gap: 14px;
  border-radius: 22px;
  border-color: color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 92%, var(--sl-bg));
}

.shell-mode-section__header {
  display: flex;
  gap: 16px;
  justify-content: space-between;
  align-items: flex-start;
}

.shell-mode-section__copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.shell-mode-section__eyebrow,
.shell-mode-section__title,
.shell-mode-section__description,
.shell-mode-section__fact dt,
.shell-mode-section__fact dd,
.shell-mode-section__readonly-hint {
  margin: 0;
}

.shell-mode-section__eyebrow {
  font-size: 0.76rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.shell-mode-section__title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.shell-mode-section__description {
  max-width: 56ch;
  font-size: 0.84rem;
  line-height: 1.5;
  color: var(--sl-text-secondary);
}

.shell-mode-section__notice,
.shell-mode-section__loading {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border-radius: 16px;
  font-size: 0.84rem;
  line-height: 1.5;
}

.shell-mode-section__notice--neutral {
  background: color-mix(in srgb, var(--sl-bg-secondary) 72%, transparent);
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  color: var(--sl-text-secondary);
}

.shell-mode-section__notice--info {
  background: color-mix(in srgb, var(--sl-primary) 10%, var(--sl-surface));
  border: 1px solid color-mix(in srgb, var(--sl-primary) 24%, transparent);
  color: var(--sl-text-primary);
}

.shell-mode-section__notice--error {
  background: var(--sl-error-bg);
  border: 1px solid rgba(var(--sl-error), 0.25);
  color: var(--sl-error);
}

.shell-mode-section__loading {
  color: var(--sl-text-secondary);
}

.shell-mode-section__spinner {
  width: 1rem;
  height: 1rem;
  border-radius: 999px;
  border: 2px solid color-mix(in srgb, var(--sl-border) 82%, transparent);
  border-top-color: var(--sl-primary);
  animation: shell-mode-section-spin 0.9s linear infinite;
}

.shell-mode-section__body {
  display: grid;
  gap: 14px;
}

.shell-mode-section__facts {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.shell-mode-section__fact {
  display: grid;
  gap: 6px;
  padding: 14px 16px;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
}

.shell-mode-section__fact dt {
  font-size: 0.78rem;
  color: var(--sl-text-tertiary);
}

.shell-mode-section__fact dd {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.shell-mode-section__readonly-hint {
  font-size: 0.84rem;
  color: var(--sl-text-secondary);
}

@keyframes shell-mode-section-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 720px) {
  .shell-mode-section__facts {
    grid-template-columns: 1fr;
  }

  .shell-mode-section__header {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
