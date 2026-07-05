<script setup lang="ts">
import { computed } from "vue";
import { Crown, ShieldBan, UserRound, Wifi, WifiOff } from "@lucide/vue";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

export interface ServerInstancePlayerCardModel {
  key: string;
  name: string;
  uuid: string | null;
  online: boolean;
  isOp: boolean;
  nameBanned: boolean;
  ipBanMatched: boolean;
  ipBanKnown: boolean;
  whitelist: boolean;
  detailBadges: string[];
}

interface Props {
  player: ServerInstancePlayerCardModel;
  canKick: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  kick: [name: string];
}>();

const cardClasses = computed(() => ({
  "server-instance-player-card": true,
  "server-instance-player-card--online": props.player.online,
  "server-instance-player-card--offline": !props.player.online,
  "server-instance-player-card--danger": props.player.nameBanned || props.player.ipBanMatched,
}));

const nameClasses = computed(() => ({
  "server-instance-player-card__name": true,
  "server-instance-player-card__name--op": props.player.isOp,
  "server-instance-player-card__name--danger": props.player.nameBanned || props.player.ipBanMatched,
}));

const onlineLabel = computed(() =>
  props.player.online
    ? i18n.t("servers.next.instance.players.status_online")
    : i18n.t("servers.next.instance.players.status_offline"),
);
</script>

<template>
  <article :class="cardClasses">
    <div class="server-instance-player-card__header">
      <div class="server-instance-player-card__title-group">
        <div class="server-instance-player-card__title-row">
          <UserRound :size="18" />
          <strong :class="nameClasses">{{ player.name }}</strong>
          <span class="server-instance-player-card__status-chip">
            <Wifi v-if="player.online" :size="14" />
            <WifiOff v-else :size="14" />
            <span>{{ onlineLabel }}</span>
          </span>
        </div>

        <p v-if="player.uuid" class="server-instance-player-card__uuid">
          {{ i18n.t("servers.next.instance.players.uuid_label") }} · {{ player.uuid }}
        </p>
      </div>

      <SLButton
        v-if="player.online"
        variant="ghost"
        size="sm"
        :disabled="!canKick"
        @click="emit('kick', player.name)"
      >
        {{ i18n.t("players.kick") }}
      </SLButton>
    </div>

    <div class="server-instance-player-card__badges">
      <span
        v-if="player.isOp"
        class="server-instance-player-card__badge server-instance-player-card__badge--op"
      >
        <Crown :size="14" />
        <span>{{ i18n.t("servers.next.instance.players.op_badge") }}</span>
      </span>
      <span
        v-if="player.nameBanned"
        class="server-instance-player-card__badge server-instance-player-card__badge--danger"
      >
        <ShieldBan :size="14" />
        <span>{{ i18n.t("servers.next.instance.players.name_ban_badge") }}</span>
      </span>
      <span
        v-if="player.ipBanMatched"
        class="server-instance-player-card__badge server-instance-player-card__badge--danger"
      >
        <ShieldBan :size="14" />
        <span>{{ i18n.t("servers.next.instance.players.ip_ban_badge") }}</span>
      </span>
      <span v-else-if="player.ipBanKnown" class="server-instance-player-card__badge">
        <span>{{ i18n.t("servers.next.instance.players.ip_rules_badge") }}</span>
      </span>
      <span v-if="player.whitelist" class="server-instance-player-card__badge">
        <span>{{ i18n.t("servers.next.instance.players.whitelist_badge") }}</span>
      </span>
      <span
        v-for="badge in player.detailBadges"
        :key="badge"
        class="server-instance-player-card__badge"
      >
        <span>{{ badge }}</span>
      </span>
    </div>
  </article>
</template>

<style scoped>
.server-instance-player-card {
  display: grid;
  gap: 14px;
  padding: 18px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  border-radius: 20px;
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
}

.server-instance-player-card--online {
  box-shadow: inset 0 0 0 1px rgba(34, 197, 94, 0.28);
}

.server-instance-player-card--offline {
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--sl-text-tertiary) 28%, transparent);
}

.server-instance-player-card--danger {
  border-color: rgba(239, 68, 68, 0.36);
}

.server-instance-player-card__header,
.server-instance-player-card__title-row,
.server-instance-player-card__badges {
  display: flex;
}

.server-instance-player-card__header {
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
}

.server-instance-player-card__title-group {
  min-width: 0;
  display: grid;
  gap: 8px;
}

.server-instance-player-card__title-row {
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.server-instance-player-card__name {
  color: var(--sl-text-primary);
}

.server-instance-player-card__name--op {
  color: #d4a72c;
}

.server-instance-player-card__name--danger {
  color: var(--sl-error);
}

.server-instance-player-card__uuid {
  margin: 0;
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
  word-break: break-all;
}

.server-instance-player-card__status-chip,
.server-instance-player-card__badge {
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 84%, transparent);
  color: var(--sl-text-secondary);
  font-size: 0.82rem;
}

.server-instance-player-card__badges {
  flex-wrap: wrap;
  gap: 8px;
}

.server-instance-player-card__badge--op {
  background: rgba(212, 167, 44, 0.14);
  color: #d4a72c;
}

.server-instance-player-card__badge--danger {
  background: rgba(239, 68, 68, 0.12);
  color: var(--sl-error);
}

@media (max-width: 767px) {
  .server-instance-player-card__header {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
