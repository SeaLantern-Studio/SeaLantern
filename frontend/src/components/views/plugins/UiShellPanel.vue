<script setup lang="ts">
import { SLButton, SLCard } from "@components/common";
import { useUiShellPanel } from "@components/views/plugins/useUiShellPanel";
import { i18n } from "@language";

const {
  shellFacts,
  currentShellName,
  loading,
  safeMode,
  errorMessage,
  browserOnly,
  refreshStatus,
} = useUiShellPanel();
</script>

<template>
  <SLCard
    class="ui-shell-panel"
    :title="i18n.t('plugins.ui_shell.title')"
    :subtitle="i18n.t('plugins.ui_shell.readonly_subtitle')"
  >
    <template #actions>
      <SLButton variant="secondary" size="sm" :loading="loading" @click="refreshStatus">
        {{ i18n.t("settings.next.refresh") }}
      </SLButton>
    </template>

    <div v-if="browserOnly" class="ui-shell-note ui-shell-note--readonly">
      {{ i18n.t("plugins.ui_shell.desktop_only") }}
    </div>

    <template v-else>
      <div v-if="safeMode" class="ui-shell-note">
        {{ i18n.t("plugins.ui_shell.safe_mode_hint") }}
      </div>

      <div v-if="errorMessage" class="ui-shell-error">
        {{ errorMessage }}
      </div>

      <div v-else-if="loading" class="ui-shell-loading">
        <div class="ui-shell-loading-spinner"></div>
        <span>{{ i18n.t("plugins.ui_shell.loading") }}</span>
      </div>

      <div v-else class="ui-shell-summary">
        <strong class="ui-shell-summary__title">{{ currentShellName }}</strong>
        <p class="ui-shell-summary__hint">{{ i18n.t("settings.next.shell.readonly_hint") }}</p>

        <dl class="ui-shell-facts">
          <div v-for="fact in shellFacts" :key="fact.label" class="ui-shell-fact">
            <dt>{{ fact.label }}</dt>
            <dd>{{ fact.value }}</dd>
          </div>
        </dl>
      </div>
    </template>
  </SLCard>
</template>

<style scoped>
.ui-shell-panel {
  border: 1px solid var(--sl-border-light);
}

.ui-shell-note,
.ui-shell-error {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
  padding: var(--sl-space-md);
  border-radius: var(--sl-radius-md);
  margin-bottom: var(--sl-space-md);
}

.ui-shell-note {
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border-light);
  color: var(--sl-text-secondary);
}

.ui-shell-note--info {
  background: color-mix(in srgb, var(--sl-primary) 10%, var(--sl-surface));
  border-color: color-mix(in srgb, var(--sl-primary) 24%, transparent);
  color: var(--sl-text-primary);
}

.ui-shell-note--readonly {
  margin-bottom: 0;
}

.ui-shell-error {
  background: var(--sl-error-bg);
  border: 1px solid rgba(var(--sl-error), 0.25);
  color: var(--sl-error);
}

.ui-shell-loading {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  min-height: 4rem;
  color: var(--sl-text-secondary);
}

.ui-shell-loading-spinner {
  width: 1.25rem;
  height: 1.25rem;
  border: 2px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 999px;
  animation: ui-shell-spin 1s linear infinite;
}

.ui-shell-summary {
  display: grid;
  gap: var(--sl-space-md);
}

.ui-shell-summary__title,
.ui-shell-summary__hint,
.ui-shell-fact dt,
.ui-shell-fact dd {
  margin: 0;
}

.ui-shell-summary__title {
  font-size: 1rem;
  color: var(--sl-text-primary);
}

.ui-shell-summary__hint {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
}

.ui-shell-facts {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: var(--sl-space-md);
}

.ui-shell-fact {
  display: grid;
  gap: 0.375rem;
  padding: var(--sl-space-md);
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border-light);
  background: var(--sl-bg-secondary);
}

.ui-shell-fact dt {
  font-size: 0.78rem;
  color: var(--sl-text-tertiary);
}

.ui-shell-fact dd {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

@keyframes ui-shell-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 720px) {
  .ui-shell-note {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
