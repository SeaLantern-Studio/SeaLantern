<script setup lang="ts">
import { ref, computed, watch } from "vue";
import PlayerAvatar from "./PlayerAvatar.vue";
import { playerApi } from "@api/player";
import { i18n } from "@language";
import { useToast } from "cmzya-modern-ui";

type PlayerTab = "online" | "whitelist" | "banned" | "ops";

const props = defineProps<{
  loading?: boolean;
  tab: PlayerTab;
  serverPath?: string;
  onlinePlayers?: string[];
  whitelist?: Array<{ name: string; uuid: string }>;
  bannedPlayers?: Array<{ name: string; uuid: string; reason?: string }>;
  ops?: Array<{ name: string; uuid: string; level: number }>;
  serverRunning?: boolean;
}>();

const emit = defineEmits<{
  (e: "kick", name: string): void;
  (e: "removeWhitelist", name: string): void;
  (e: "unban", name: string): void;
  (e: "removeOp", name: string): void;
}>();

const toast = useToast();
const searchQuery = ref("");

// 切换 tab 时重置搜索词
watch(
  () => props.tab,
  () => {
    searchQuery.value = "";
  },
);

function matchesQuery(name: string): boolean {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return true;
  return name.toLowerCase().includes(q);
}

const filteredOnline = computed(() => (props.onlinePlayers || []).filter((n) => matchesQuery(n)));
const filteredWhitelist = computed(() =>
  (props.whitelist || []).filter((p) => matchesQuery(p.name)),
);
const filteredBanned = computed(() =>
  (props.bannedPlayers || []).filter((p) => matchesQuery(p.name)),
);
const filteredOps = computed(() => (props.ops || []).filter((p) => matchesQuery(p.name)));

async function handleCopyUuid(name: string, uuid?: string) {
  let uuidToCopy = uuid;
  if (!uuidToCopy) {
    // 在线玩家没有 UUID 数据，从 usercache.json 查询
    if (!props.serverPath) return;
    try {
      const profile = await playerApi.lookupPlayer(props.serverPath, name);
      uuidToCopy = profile.uuid;
    } catch {
      toast.error(i18n.t("players.lookup_not_found"));
      return;
    }
  }
  try {
    await navigator.clipboard.writeText(uuidToCopy);
    toast.success(i18n.t("players.uuid_copied"));
  } catch {
    toast.error(i18n.t("players.lookup_copy_failed"));
  }
}
</script>

<template>
  <div class="player-list">
    <!-- Search Box -->
    <div v-if="!loading" class="player-search-bar">
      <cmz-input :placeholder="i18n.t('players.search_placeholder')" v-model="searchQuery" />
    </div>

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
      <div v-else-if="!filteredOnline.length" class="player-list-empty">
        <p class="text-caption">{{ i18n.t("players.no_players") }}</p>
      </div>
      <div v-for="name in filteredOnline" :key="name" class="player-item glass-card">
        <PlayerAvatar
          :name="name"
          :size="36"
          clickable
          :title="i18n.t('players.lookup_copy_uuid')"
          @click="handleCopyUuid(name)"
        />
        <div class="player-info">
          <span class="player-name">{{ name }}</span>
          <cmz-badge :text="i18n.t('players.status_online')" color="var(--sl-success)" />
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
      <div v-if="!filteredWhitelist.length" class="player-list-empty">
        <p class="text-caption">{{ i18n.t("players.empty_whitelist") }}</p>
      </div>
      <div v-for="p in filteredWhitelist" :key="p.name" class="player-item glass-card">
        <PlayerAvatar
          :name="p.name"
          :size="36"
          clickable
          :title="i18n.t('players.lookup_copy_uuid')"
          @click="handleCopyUuid(p.name, p.uuid)"
        />
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
      <div v-if="!filteredBanned.length" class="player-list-empty">
        <p class="text-caption">{{ i18n.t("players.empty_banned") }}</p>
      </div>
      <div v-for="p in filteredBanned" :key="p.name" class="player-item glass-card">
        <PlayerAvatar
          :name="p.name"
          :size="36"
          clickable
          :title="i18n.t('players.lookup_copy_uuid')"
          @click="handleCopyUuid(p.name, p.uuid)"
        />
        <div class="player-info">
          <span class="player-name">{{ p.name }}</span>
          <span class="text-caption"
            >{{ i18n.t("players.reason") }}: {{ p.reason || i18n.t("players.empty") }}</span
          >
        </div>
        <cmz-badge :text="i18n.t('players.ban')" color="var(--sl-error)" />
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
      <div v-if="!filteredOps.length" class="player-list-empty">
        <p class="text-caption">{{ i18n.t("players.empty_ops") }}</p>
      </div>
      <div v-for="p in filteredOps" :key="p.name" class="player-item glass-card">
        <PlayerAvatar
          :name="p.name"
          :size="36"
          clickable
          :title="i18n.t('players.lookup_copy_uuid')"
          @click="handleCopyUuid(p.name, p.uuid)"
        />
        <div class="player-info">
          <span class="player-name">{{ p.name }}</span>
          <span class="text-caption">{{ i18n.t("players.level") }}: {{ p.level }}</span>
        </div>
        <cmz-badge text="OP" color="var(--sl-warning)" />
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

.player-search-bar {
  margin-bottom: var(--sl-space-xs);
}

.player-search-bar :deep(.cmz-input) {
  padding: 6px 12px;
  font-size: 13px;
}

.player-search-bar :deep(.cmz-input-container) {
  height: 32px;
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
