<script setup lang="ts">
import { computed } from "vue";
import { SLButton, SLCard, SLConfirmDialog } from "@components/common";
import UiShellCard from "@components/views/plugins/UiShellCard.vue";
import { useUiShellPanel } from "@components/views/plugins/useUiShellPanel";
import { i18n } from "@language";

const {
  shellCards,
  loading,
  safeMode,
  restarting,
  showRestartDialog,
  errorMessage,
  restartNotice,
  browserOnly,
  pendingShellName,
  hasPendingRestart,
  switchInFlight,
  selectShell,
  requestRestart,
  confirmRestart,
  closeRestartDialog,
} = useUiShellPanel();

const restartMessage = computed(() =>
  i18n.t("plugins.ui_shell.restart_confirm_message", {
    name: pendingShellName.value,
  }),
);
</script>

<template>
  <SLCard
    class="ui-shell-panel"
    :title="i18n.t('plugins.ui_shell.title')"
    :subtitle="i18n.t('plugins.ui_shell.subtitle')"
  >
    <div v-if="browserOnly" class="ui-shell-note ui-shell-note--readonly">
      {{ i18n.t("plugins.ui_shell.desktop_only") }}
    </div>

    <template v-else>
      <div v-if="safeMode" class="ui-shell-note">
        {{ i18n.t("plugins.ui_shell.safe_mode_hint") }}
      </div>

      <div v-if="restartNotice" class="ui-shell-note ui-shell-note--info">
        {{ restartNotice }}
      </div>

      <div v-if="errorMessage" class="ui-shell-error">
        {{ errorMessage }}
      </div>

      <div v-else-if="loading" class="ui-shell-loading">
        <div class="ui-shell-loading-spinner"></div>
        <span>{{ i18n.t("plugins.ui_shell.loading") }}</span>
      </div>

      <template v-else>
        <div v-if="hasPendingRestart" class="ui-shell-pending-banner">
          <div class="ui-shell-pending-copy">
            <strong>{{ i18n.t("plugins.ui_shell.pending_banner_title") }}</strong>
            <span>
              {{ i18n.t("plugins.ui_shell.pending_banner_desc", { name: pendingShellName }) }}
            </span>
          </div>
          <SLButton
            size="sm"
            :loading="restarting"
            :disabled="switchInFlight"
            @click="requestRestart"
          >
            {{ i18n.t("plugins.ui_shell.restart_now") }}
          </SLButton>
        </div>

        <div class="ui-shell-grid">
          <UiShellCard
            v-for="shell in shellCards"
            :key="shell.id"
            :title="shell.name"
            :description="shell.description"
            :source-label="shell.sourceLabel"
            :running="shell.running"
            :pending-restart="shell.pendingRestart"
            :disabled="shell.disabled"
            :loading="switchInFlight"
            :action-label="shell.actionLabel"
            :status-badges="shell.statusBadges"
            @select="selectShell(shell.id)"
            @restart="requestRestart"
          />
        </div>
      </template>
    </template>
  </SLCard>

  <SLConfirmDialog
    :visible="showRestartDialog"
    :title="i18n.t('plugins.ui_shell.restart_confirm_title')"
    :message="restartMessage"
    :confirm-text="i18n.t('plugins.ui_shell.restart_now')"
    :cancel-text="i18n.t('plugins.cancel')"
    :loading="restarting"
    @confirm="confirmRestart"
    @cancel="closeRestartDialog"
    @close="closeRestartDialog"
    @update:visible="closeRestartDialog"
  />
</template>

<style scoped>
.ui-shell-panel {
  border: 1px solid var(--sl-border-light);
}

.ui-shell-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--sl-space-md);
}

.ui-shell-note,
.ui-shell-error,
.ui-shell-pending-banner {
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

.ui-shell-pending-banner {
  background: rgba(var(--sl-warning), 0.08);
  border: 1px solid rgba(var(--sl-warning), 0.25);
}

.ui-shell-pending-copy {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  color: var(--sl-text-primary);
}

.ui-shell-pending-copy span {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
}

@keyframes ui-shell-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 720px) {
  .ui-shell-note,
  .ui-shell-pending-banner {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
