<script setup lang="ts">
import PlayerAvatar from "./PlayerAvatar.vue";
import { i18n } from "@language";

type PlayerTab = "online" | "whitelist" | "banned" | "ops";

defineProps<{
  loading?: boolean;
  tab: PlayerTab;
  onlinePlayers?: string[];
  whitelist?: Array<{ name: string; uuid: string }>;
  bannedPlayers?: Array<{ name: string; reason?: string }>;
  ops?: Array<{ name: string; level: number }>;
  serverRunning?: boolean;
}>();

const emit = defineEmits<{
  (e: "kick", name: string): void;
  (e: "removeWhitelist", name: string): void;
  (e: "unban", name: string): void;
  (e: "removeOp", name: string): void;
}>();
</script>

<template>
  <div class="player-list">
    <!-- Loading State -->
    <div v-if="loading" class="player-list-loading">
      <cmz-spinner />
      <span>{{ i18n.t("common.loading") }}</span>
    </div>

    <!-- Online Players -->
    <template v-else-if="tab === 'online'">
      <div v-if="!serverRunning" class="player-list-empty">
        <p class="text-caption">{{ i18n.t("players.server_offline") }}</p>
      </div>
      <div v-else-if="!onlinePlayers?.length" class="player-list-empty">
        <p class="text-caption">{{ i18n.t("players.no_players") }}</p>
      </div>
      <div v-for="name in onlinePlayers" :key="name" class="player-item glass-card">
        <PlayerAvatar :name="name" :size="36" />
        <div class="player-info">
          <span class="player-name">{{ name }}</span>
          <cmz-badge :text="i18n.t('players.status_online')" color="#22c55e" />
        </div>
        <div class="player-actions">
          <cmz-button variant="ghost" size="sm" @click="emit('kick', name)">{{
            i18n.t("players.kick")
          }}</cmz-button>
        </div>
      </div>
    </template>

    <!-- Whitelist -->
    <template v-else-if="tab === 'whitelist'">
      <div v-if="!whitelist?.length" class="player-list-empty">
        <p class="text-caption">{{ i18n.t("players.empty_whitelist") }}</p>
      </div>
      <div v-for="p in whitelist" :key="p.name" class="player-item glass-card">
        <PlayerAvatar :name="p.name" :size="36" />
        <div class="player-info">
          <span class="player-name">{{ p.name }}</span>
          <span class="player-uuid text-mono text-caption">{{ p.uuid }}</span>
        </div>
        <div class="player-actions">
          <cmz-button
            variant="ghost"
            size="sm"
            :disabled="!serverRunning"
            @click="emit('removeWhitelist', p.name)"
            >{{ i18n.t("players.remove") }}</cmz-button
          >
        </div>
      </div>
    </template>

    <!-- Banned -->
    <template v-else-if="tab === 'banned'">
      <div v-if="!bannedPlayers?.length" class="player-list-empty">
        <p class="text-caption">{{ i18n.t("players.empty_banned") }}</p>
      </div>
      <div v-for="p in bannedPlayers" :key="p.name" class="player-item glass-card">
        <PlayerAvatar :name="p.name" :size="36" />
        <div class="player-info">
          <span class="player-name">{{ p.name }}</span>
          <span class="text-caption"
            >{{ i18n.t("players.reason") }}: {{ p.reason || i18n.t("players.empty") }}</span
          >
        </div>
        <cmz-badge :text="i18n.t('players.ban')" color="#ef4444" />
        <div class="player-actions">
          <cmz-button
            variant="ghost"
            size="sm"
            :disabled="!serverRunning"
            @click="emit('unban', p.name)"
            >{{ i18n.t("players.unban") }}</cmz-button
          >
        </div>
      </div>
    </template>

    <!-- Ops -->
    <template v-else-if="tab === 'ops'">
      <div v-if="!ops?.length" class="player-list-empty">
        <p class="text-caption">{{ i18n.t("players.empty_ops") }}</p>
      </div>
      <div v-for="p in ops" :key="p.name" class="player-item glass-card">
        <PlayerAvatar :name="p.name" :size="36" />
        <div class="player-info">
          <span class="player-name">{{ p.name }}</span>
          <span class="text-caption">{{ i18n.t("players.level") }}: {{ p.level }}</span>
        </div>
        <cmz-badge text="OP" color="#f59e0b" />
        <div class="player-actions">
          <cmz-button
            variant="ghost"
            size="sm"
            :disabled="!serverRunning"
            @click="emit('removeOp', p.name)"
            >{{ i18n.t("players.deop") }}</cmz-button
          >
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.player-list {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.player-list-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}

.player-list-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--sl-space-2xl);
}

.player-item {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-md);
}

.player-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 2px;
}

.player-name {
  font-size: 0.9375rem;
  font-weight: 600;
}

.player-uuid {
  font-size: 0.6875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.player-actions {
  flex-shrink: 0;
}
</style>
