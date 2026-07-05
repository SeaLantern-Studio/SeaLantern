<script setup lang="ts">
import { Copy, Eye, EyeOff, GitBranch, Info, RefreshCw, X } from "@lucide/vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLModal from "@components/common/SLModal.vue";
import SLStatusIndicator from "@components/common/SLStatusIndicator.vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import WorkbenchFactGrid from "@src/components/workbench/WorkbenchFactGrid.vue";
import WorkbenchPanel from "@src/components/workbench/WorkbenchPanel.vue";
import WorkbenchStatusBanner from "@src/components/workbench/WorkbenchStatusBanner.vue";
import { i18n } from "@language";
import { useTunnelPage } from "./useTunnelPage";

const page = useTunnelPage();
</script>

<template>
  <div class="tunnel-page">
    <div class="tunnel-page__top-actions">
      <SLButton variant="secondary" size="sm" @click="page.showInfoModal.value = true">
        {{ i18n.t("tunnel.info_title") }}
      </SLButton>
    </div>

    <WorkbenchFactGrid :items="page.summaryFacts.value" />

    <WorkbenchStatusBanner v-if="!page.running.value" tone="info">
      <strong>{{ i18n.t("tunnel.status_title") }}</strong>
      <span>{{ i18n.t("tunnel.no_ticket") }}</span>
    </WorkbenchStatusBanner>

    <WorkbenchPanel :title="i18n.t('tunnel.status_title')">
      <template #actions>
        <button
          type="button"
          class="tunnel-page__icon-button"
          :title="i18n.t('tunnel.info_title')"
          :aria-label="i18n.t('tunnel.info_title')"
          @click="page.showInfoModal.value = true"
        >
          <Info :size="16" />
        </button>
      </template>

      <div class="tunnel-page__status-grid">
        <div class="tunnel-page__status-pill">
          <span class="tunnel-page__status-label">{{ i18n.t("tunnel.running") }}</span>
          <SLStatusIndicator
            :status="page.runningStatusClass.value"
            :label="page.runningStatusText.value"
          />
        </div>

        <div class="tunnel-page__status-pill">
          <span class="tunnel-page__status-label">{{ i18n.t("tunnel.mode") }}</span>
          <strong class="tunnel-page__status-value">{{ page.modeLabel.value }}</strong>
        </div>
      </div>

      <div class="tunnel-page__ticket-row">
        <span class="tunnel-page__ticket-label">{{ i18n.t("tunnel.ticket") }}</span>
        <code class="tunnel-page__ticket-value">{{
          page.status.value?.ticket || i18n.t("tunnel.no_ticket")
        }}</code>

        <div class="tunnel-page__ticket-actions">
          <template v-if="page.hasTicket.value">
            <button
              type="button"
              class="tunnel-page__icon-button"
              :title="i18n.t('tunnel.copy_ticket')"
              :aria-label="i18n.t('tunnel.copy_ticket')"
              :disabled="!page.canCopyTicket.value"
              @click="page.copyTicket"
            >
              <Copy :size="16" />
            </button>
            <button
              type="button"
              class="tunnel-page__icon-button"
              :title="i18n.t('tunnel.regenerate_ticket')"
              :aria-label="i18n.t('tunnel.regenerate_ticket')"
              :disabled="!page.canGenerateTicket.value"
              @click="page.regenerateTicket"
            >
              <RefreshCw :size="16" />
            </button>
          </template>

          <SLButton
            v-else
            variant="secondary"
            size="sm"
            :disabled="!page.canGenerateTicket.value"
            :loading="page.generateTicketLoading.value"
            @click="page.generateTicket"
          >
            {{ i18n.t("tunnel.generate_ticket") }}
          </SLButton>

          <SLButton
            v-if="page.running.value"
            variant="danger"
            size="sm"
            :disabled="!page.canStopTunnel.value"
            :loading="page.stopActionLoading.value"
            @click="page.stopTunnel"
          >
            {{ i18n.t("tunnel.stop") }}
          </SLButton>
        </div>
      </div>
    </WorkbenchPanel>

    <div class="tunnel-page__form-grid">
      <WorkbenchPanel :title="i18n.t('tunnel.host_title')">
        <div class="tunnel-page__field-grid">
          <SLInput
            v-model="page.hostPort.value"
            :label="i18n.t('tunnel.host_port')"
            :disabled="!page.canEditHostForm.value"
          />
          <SLInput
            v-model="page.hostPassword.value"
            :type="page.hostPasswordInputType.value"
            :label="i18n.t('tunnel.host_password')"
            :disabled="!page.canEditHostForm.value"
          >
            <template #suffix>
              <button
                type="button"
                class="tunnel-page__icon-button tunnel-page__input-button"
                :title="
                  page.showHostPassword.value
                    ? i18n.t('tunnel.hide_password')
                    : i18n.t('tunnel.show_password')
                "
                :aria-label="
                  page.showHostPassword.value
                    ? i18n.t('tunnel.hide_password')
                    : i18n.t('tunnel.show_password')
                "
                :disabled="!page.canEditHostForm.value"
                @click="page.showHostPassword.value = !page.showHostPassword.value"
              >
                <EyeOff v-if="page.showHostPassword.value" :size="14" />
                <Eye v-else :size="14" />
              </button>
            </template>
          </SLInput>
          <SLInput
            v-model="page.hostRelayUrl.value"
            :label="i18n.t('tunnel.host_relay_url')"
            :disabled="!page.canEditHostForm.value"
          >
            <template v-if="page.hostRelayUrl.value.length > 0" #suffix>
              <button
                type="button"
                class="tunnel-page__icon-button tunnel-page__input-button"
                :title="i18n.t('tunnel.clear_field')"
                :aria-label="i18n.t('tunnel.clear_field')"
                :disabled="!page.canClearHostRelay.value"
                @click="page.clearHostRelay"
              >
                <X :size="14" />
              </button>
            </template>
          </SLInput>
        </div>

        <template #actions>
          <SLButton
            variant="primary"
            :disabled="!page.canStartHost.value"
            :loading="page.hostActionLoading.value"
            @click="page.startHost"
          >
            {{ i18n.t("tunnel.start_host") }}
          </SLButton>
        </template>
      </WorkbenchPanel>

      <WorkbenchPanel :title="i18n.t('tunnel.join_title')">
        <div class="tunnel-page__field-grid">
          <SLInput
            :model-value="page.joinTicket.value"
            :label="i18n.t('tunnel.join_ticket')"
            :disabled="!page.canEditJoinForm.value"
            @update:model-value="page.handleJoinTicketInput"
          >
            <template v-if="page.joinTicket.value.length > 0" #suffix>
              <button
                type="button"
                class="tunnel-page__icon-button tunnel-page__input-button"
                :title="i18n.t('tunnel.clear_field')"
                :aria-label="i18n.t('tunnel.clear_field')"
                :disabled="!page.canClearJoinTicket.value"
                @click="page.clearJoinTicket"
              >
                <X :size="14" />
              </button>
            </template>
          </SLInput>
          <SLInput
            v-model="page.joinLocalPort.value"
            :label="i18n.t('tunnel.join_local_port')"
            :disabled="!page.canEditJoinForm.value"
          />
          <SLInput
            v-model="page.joinPassword.value"
            :type="page.joinPasswordInputType.value"
            :label="i18n.t('tunnel.join_password')"
            :disabled="!page.canEditJoinForm.value"
          >
            <template #suffix>
              <button
                type="button"
                class="tunnel-page__icon-button tunnel-page__input-button"
                :title="
                  page.showJoinPassword.value
                    ? i18n.t('tunnel.hide_password')
                    : i18n.t('tunnel.show_password')
                "
                :aria-label="
                  page.showJoinPassword.value
                    ? i18n.t('tunnel.hide_password')
                    : i18n.t('tunnel.show_password')
                "
                :disabled="!page.canEditJoinForm.value"
                @click="page.showJoinPassword.value = !page.showJoinPassword.value"
              >
                <EyeOff v-if="page.showJoinPassword.value" :size="14" />
                <Eye v-else :size="14" />
              </button>
            </template>
          </SLInput>
        </div>

        <template #actions>
          <SLButton
            variant="primary"
            :disabled="!page.canStartJoin.value"
            :loading="page.joinActionLoading.value"
            @click="page.startJoin"
          >
            {{ i18n.t("tunnel.start_join") }}
          </SLButton>
        </template>
      </WorkbenchPanel>
    </div>

    <WorkbenchPanel v-if="page.showConnections.value" :title="i18n.t('tunnel.connections_title')">
      <div v-if="!page.connectionRows.value.length" class="tunnel-page__empty-text">
        {{ i18n.t("tunnel.no_connections") }}
      </div>
      <div v-else class="tunnel-page__table-wrap">
        <table class="tunnel-page__table">
          <thead>
            <tr>
              <th>{{ i18n.t("tunnel.table_remote") }}</th>
              <th>{{ i18n.t("tunnel.table_route") }}</th>
              <th>{{ i18n.t("tunnel.table_rtt") }}</th>
              <th>{{ i18n.t("tunnel.table_tx") }}</th>
              <th>{{ i18n.t("tunnel.table_rx") }}</th>
              <th>{{ i18n.t("tunnel.table_alive") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in page.connectionRows.value" :key="item.remote_id">
              <td>{{ item.remote_id }}</td>
              <td>
                {{ item.is_relay ? i18n.t("tunnel.route_relay") : i18n.t("tunnel.route_direct") }}
              </td>
              <td>{{ item.rtt_ms }} ms</td>
              <td>{{ item.tx_bytes }}</td>
              <td>{{ item.rx_bytes }}</td>
              <td>{{ item.alive ? i18n.t("tunnel.yes") : i18n.t("tunnel.no") }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </WorkbenchPanel>

    <WorkbenchPanel :title="i18n.t('tunnel.logs_title')">
      <template #actions>
        <div class="tunnel-page__log-actions">
          <SLButton variant="secondary" size="sm" @click="page.copyLogs">
            {{ i18n.t("console.copy_log") }}
          </SLButton>
          <SLButton variant="ghost" size="sm" @click="page.clearLogs">
            {{ i18n.t("console.clear_log") }}
          </SLButton>
        </div>
      </template>

      <div class="tunnel-page__log-console">
        <ConsoleOutput
          :ref="page.setTunnelOutputRef"
          :console-font-size="page.consoleFontSize.value"
          :console-font-family="page.consoleFontFamily.value"
          :console-letter-spacing="page.consoleLetterSpacing.value"
          :max-log-lines="page.maxLogLines.value"
          :user-scrolled-up="page.userScrolledUp.value"
          @scroll="(value) => (page.userScrolledUp.value = value)"
          @scroll-to-bottom="
            page.userScrolledUp.value = false;
            page.tunnelOutputRef.value?.doScroll();
          "
        />
      </div>
    </WorkbenchPanel>

    <SLModal
      :visible="page.showInfoModal.value"
      :title="i18n.t('tunnel.info_title')"
      width="420px"
      @close="page.showInfoModal.value = false"
    >
      <div class="tunnel-page__info-content">
        <p>{{ i18n.t("tunnel.info_desc") }}</p>
        <button class="tunnel-page__info-link" @click="page.openTunnelInfo">
          <GitBranch :size="16" />
          <span>{{ i18n.t("tunnel.info_github") }}</span>
        </button>
      </div>
    </SLModal>
  </div>
</template>

<style scoped>
.tunnel-page {
  min-width: 0;
  display: grid;
  gap: 16px;
}

.tunnel-page__top-actions {
  display: flex;
  justify-content: flex-end;
}

.tunnel-page__status-grid,
.tunnel-page__ticket-row,
.tunnel-page__form-grid,
.tunnel-page__field-grid,
.tunnel-page__log-actions,
.tunnel-page__info-content {
  min-width: 0;
}

.tunnel-page__status-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.tunnel-page__status-pill {
  display: grid;
  gap: 6px;
  padding: 14px 16px;
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 76%, transparent);
  border: 1px solid color-mix(in srgb, var(--sl-border) 68%, transparent);
}

.tunnel-page__status-label {
  font-size: 0.8rem;
  color: var(--sl-text-tertiary);
}

.tunnel-page__status-value {
  font-size: 0.95rem;
  color: var(--sl-text-primary);
}

.tunnel-page__ticket-row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  margin-top: 14px;
}

.tunnel-page__ticket-label {
  font-size: 0.82rem;
  color: var(--sl-text-tertiary);
}

.tunnel-page__ticket-value {
  display: block;
  min-width: 0;
  padding: 10px 12px;
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 76%, transparent);
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
  word-break: break-all;
}

.tunnel-page__ticket-actions,
.tunnel-page__log-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tunnel-page__form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.tunnel-page__field-grid {
  display: grid;
  gap: 12px;
}

.tunnel-page__icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
  color: var(--sl-text-secondary);
  transition:
    border-color 0.2s ease,
    color 0.2s ease,
    background-color 0.2s ease;
}

.tunnel-page__icon-button:hover:not(:disabled) {
  color: var(--sl-primary);
  border-color: color-mix(in srgb, var(--sl-primary) 32%, var(--sl-border));
  background: color-mix(in srgb, var(--sl-primary) 10%, var(--sl-surface));
}

.tunnel-page__icon-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.tunnel-page__input-button {
  width: 28px;
  height: 28px;
  border-radius: 8px;
}

.tunnel-page__empty-text {
  font-size: 0.88rem;
  color: var(--sl-text-secondary);
}

.tunnel-page__table-wrap {
  overflow-x: auto;
}

.tunnel-page__table {
  width: 100%;
  border-collapse: collapse;
  min-width: 620px;
}

.tunnel-page__table th,
.tunnel-page__table td {
  padding: 10px 12px;
  text-align: left;
  border-bottom: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  font-size: 0.84rem;
}

.tunnel-page__table th {
  color: var(--sl-text-tertiary);
  font-weight: 600;
}

.tunnel-page__table td {
  color: var(--sl-text-primary);
}

.tunnel-page__log-console {
  min-height: 320px;
  overflow: hidden;
  border-radius: 18px;
}

.tunnel-page__info-content {
  display: grid;
  gap: 12px;
}

.tunnel-page__info-content p {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.6;
  color: var(--sl-text-primary);
}

.tunnel-page__info-link {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  width: fit-content;
  padding: 10px 14px;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--sl-primary) 28%, var(--sl-border));
  background: color-mix(in srgb, var(--sl-primary) 10%, var(--sl-surface));
  color: var(--sl-primary);
}

@media (max-width: 900px) {
  .tunnel-page__status-grid,
  .tunnel-page__form-grid {
    grid-template-columns: 1fr;
  }

  .tunnel-page__ticket-row {
    grid-template-columns: 1fr;
  }
}
</style>
