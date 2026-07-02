<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import ServerInstancePlayerCard from "@next-src/components/server-instance/ServerInstancePlayerCard.vue";
import ServerInstancePlayersSummaryBar from "@next-src/components/server-instance/ServerInstancePlayersSummaryBar.vue";
import { useServerInstancePlayersPage } from "./useServerInstancePlayersPage";

const {
  loading,
  refreshing,
  errorMessage,
  isRunning,
  playerCards,
  onlineCount,
  opCount,
  bannedCount,
  bannedIpCount,
  bannedIpEntries,
  refresh,
  handleKick,
} = useServerInstancePlayersPage();
</script>

<template>
  <div class="server-instance-players-page">
    <ServerInstancePlayersSummaryBar
      :total-count="playerCards.length"
      :online-count="onlineCount"
      :op-count="opCount"
      :banned-count="bannedCount"
      :banned-ip-count="bannedIpCount"
      :refreshing="refreshing"
      :running="isRunning"
      @refresh="refresh(true)"
    />

    <section v-if="errorMessage" class="server-instance-players-page__error" role="alert">
      <strong>{{ i18n.t("servers.next.error_title") }}</strong>
      <span>{{ errorMessage }}</span>
    </section>

    <section
      v-if="loading && playerCards.length === 0"
      class="server-instance-players-page__loading"
    >
      <SLCard variant="outline">
        <div class="server-instance-players-page__loading-card">
          <strong>{{ i18n.t("players.online_players") }}</strong>
          <span>{{ i18n.t("common.loading") }}</span>
        </div>
      </SLCard>
    </section>

    <section v-else-if="playerCards.length === 0" class="server-instance-players-page__empty">
      <SLCard variant="outline">
        <div class="server-instance-players-page__empty-card">
          <strong>{{ i18n.t("players.online_players") }}</strong>
          <span>{{
            isRunning ? i18n.t("players.no_players") : i18n.t("players.server_offline")
          }}</span>
        </div>
      </SLCard>
    </section>

    <section v-else class="server-instance-players-page__grid">
      <ServerInstancePlayerCard
        v-for="player in playerCards"
        :key="player.key"
        :player="player"
        :can-kick="isRunning"
        @kick="handleKick"
      />
    </section>

    <SLCard
      v-if="bannedIpEntries.length > 0"
      variant="outline"
      class="server-instance-players-page__banned-ips"
    >
      <div class="server-instance-players-page__panel-header">
        <strong>{{ i18n.t("servers.next.instance.players.ip_bans_title") }}</strong>
        <span>{{ i18n.t("servers.next.instance.players.ip_bans_hint") }}</span>
      </div>

      <div class="server-instance-players-page__ip-list">
        <article
          v-for="entry in bannedIpEntries"
          :key="`${entry.ip}-${entry.created}`"
          class="server-instance-players-page__ip-item"
        >
          <strong>{{ entry.ip }}</strong>
          <span>{{ entry.reason || i18n.t("players.empty") }}</span>
        </article>
      </div>
    </SLCard>
  </div>
</template>

<style scoped>
.server-instance-players-page {
  display: grid;
  gap: 18px;
}

.server-instance-players-page__error,
.server-instance-players-page__panel-header,
.server-instance-players-page__ip-item {
  display: flex;
}

.server-instance-players-page__error {
  flex-direction: column;
  gap: 6px;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid rgba(239, 68, 68, 0.24);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.server-instance-players-page__loading-card,
.server-instance-players-page__empty-card {
  display: grid;
  gap: 6px;
  padding: 8px 4px;
}

.server-instance-players-page__loading-card strong,
.server-instance-players-page__empty-card strong,
.server-instance-players-page__panel-header strong,
.server-instance-players-page__ip-item strong {
  color: var(--sl-text-primary);
}

.server-instance-players-page__loading-card span,
.server-instance-players-page__empty-card span,
.server-instance-players-page__panel-header span,
.server-instance-players-page__ip-item span {
  color: var(--sl-text-secondary);
}

.server-instance-players-page__grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 16px;
}

.server-instance-players-page__panel-header,
.server-instance-players-page__ip-item {
  justify-content: space-between;
  gap: 12px;
}

.server-instance-players-page__banned-ips,
.server-instance-players-page__ip-list {
  display: grid;
  gap: 12px;
}

.server-instance-players-page__ip-item {
  padding: 12px 14px;
  border-radius: 16px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 84%, transparent);
}

@media (max-width: 767px) {
  .server-instance-players-page__panel-header,
  .server-instance-players-page__ip-item {
    flex-direction: column;
  }
}
</style>
