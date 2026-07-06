<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import { i18n } from "@language";
import ServerListCard from "@src/components/servers/ServerListCard.vue";
import ServersEmptyState from "@src/components/servers/ServersEmptyState.vue";
import ServersSummaryBar from "@src/components/servers/ServersSummaryBar.vue";
import { isServerDeleteMode } from "@src/utils/serverDeleteMode";
import { useServersPage, type ServersPageTarget } from "./useServersPage";

const {
  serverStore,
  serverItems,
  totalCount,
  runningCount,
  hasServers,
  isBootstrapping,
  isRefreshing,
  errorMessage,
  deleteDialogVisible,
  deleteSubmitting,
  deleteExpectedInput,
  deletePromptMessage,
  deleteInputPlaceholder,
  deleteSelectedMode,
  deleteModeOptions,
  loadData,
  selectServer,
  navigateToServerTarget,
  confirmDeleteServer,
  closeDeleteDialog,
  navigateToCreate,
  navigateToImport,
} = useServersPage();

async function handleNavigate(payload: {
  serverId: string;
  target: ServersPageTarget;
}): Promise<void> {
  await navigateToServerTarget(payload.serverId, payload.target);
}

function handleDeleteModeUpdate(value: string): void {
  if (isServerDeleteMode(value)) {
    deleteSelectedMode.value = value;
  }
}
</script>

<template>
  <div class="servers-page">
    <ServersSummaryBar
      :total-count="totalCount"
      :running-count="runningCount"
      :refreshing="isRefreshing"
      @create="navigateToCreate"
      @import-existing="navigateToImport"
      @refresh="loadData(true)"
    />

    <section v-if="errorMessage" class="servers-page__error-banner" role="alert" aria-live="polite">
      <div class="servers-page__error-copy">
        <strong>{{ i18n.t("servers.next.error_title") }}</strong>
        <span>{{ errorMessage }}</span>
      </div>

      <div class="servers-page__error-actions">
        <SLButton variant="ghost" size="sm" @click="serverStore.clearError()">{{
          i18n.t("servers.next.close_error")
        }}</SLButton>
        <SLButton variant="secondary" size="sm" :loading="isRefreshing" @click="loadData(true)">
          {{ i18n.t("servers.next.retry") }}
        </SLButton>
      </div>
    </section>

    <SLCard
      v-if="isBootstrapping && !hasServers"
      variant="outline"
      class="servers-page__loading-card"
    >
      <div class="servers-page__loading-state">
        <div class="servers-page__spinner" aria-hidden="true"></div>
        <div>
          <strong class="servers-page__loading-title">{{
            i18n.t("servers.next.loading_title")
          }}</strong>
          <p class="servers-page__loading-description">
            {{ i18n.t("servers.next.loading_description") }}
          </p>
        </div>
      </div>
    </SLCard>

    <ServersEmptyState
      v-else-if="!hasServers"
      :error-message="errorMessage"
      @create="navigateToCreate"
      @import-existing="navigateToImport"
    />

    <section v-else class="servers-page__list">
      <ServerListCard
        v-for="server in serverItems"
        :key="server.id"
        :server="server"
        @select="selectServer"
        @navigate="handleNavigate"
      />
    </section>

    <SLConfirmDialog
      :visible="deleteDialogVisible"
      :title="i18n.t('home.delete_server')"
      :message="deletePromptMessage"
      :confirmText="i18n.t('home.delete_server')"
      :cancelText="i18n.t('common.cancel')"
      :inputPlaceholder="deleteInputPlaceholder"
      :expectedInput="deleteExpectedInput"
      :options="deleteModeOptions"
      :selectedOption="deleteSelectedMode"
      :loading="deleteSubmitting"
      confirmVariant="danger"
      dangerous
      requireInput
      @update:selectedOption="handleDeleteModeUpdate"
      @confirm="confirmDeleteServer"
      @close="closeDeleteDialog"
    />
  </div>
</template>

<style scoped>
.servers-page {
  min-width: 0;
  display: grid;
  gap: 18px;
}

.servers-page__error-banner {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: flex-start;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid rgba(239, 68, 68, 0.22);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.servers-page__error-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.servers-page__error-copy strong {
  font-size: 0.95rem;
}

.servers-page__error-copy span {
  line-height: 1.5;
  word-break: break-word;
}

.servers-page__error-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.servers-page__loading-card {
  padding: 4px;
}

.servers-page__loading-state {
  display: flex;
  gap: 16px;
  align-items: center;
  padding: 20px;
}

.servers-page__spinner {
  width: 24px;
  height: 24px;
  border-radius: 999px;
  border: 3px solid color-mix(in srgb, var(--sl-primary) 18%, transparent);
  border-top-color: var(--sl-primary);
  animation: servers-page-spin 0.9s linear infinite;
}

.servers-page__loading-title {
  display: block;
  margin-bottom: 4px;
  color: var(--sl-text-primary);
}

.servers-page__loading-description {
  margin: 0;
  color: var(--sl-text-secondary);
}

.servers-page__list {
  display: grid;
  gap: 16px;
}

@keyframes servers-page-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 720px) {
  .servers-page__error-banner {
    flex-direction: column;
  }
}
</style>
